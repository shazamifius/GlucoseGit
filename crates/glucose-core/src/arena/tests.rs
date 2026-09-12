//! Les lois de l'arène. Chacune est une propriété du modèle, pas un exemple d'usage.

use super::*;
use crate::types::Annotation;

fn px(v: i32) -> Fx {
    Fx::from_px(v)
}

fn boite(x: i32, y: i32, w: i32, h: i32) -> Box2 {
    Box2::new(px(x), px(y), px(w), px(h))
}

/// Une arène de départ : trois nœuds de genres différents, posés à des places connues.
fn trois() -> (Arena, NodeId, NodeId, NodeId) {
    let mut a = Arena::new();
    let t = a.spawn(Kind::Text, boite(0, 0, 260, 80), NodeId::NONE);
    let s = a.spawn(Kind::Sticky, boite(400, 0, 200, 200), NodeId::NONE);
    let m = a.spawn(Kind::Membrane, boite(-100, -100, 1000, 800), NodeId::NONE);
    (a, t, s, m)
}

// ── ARN-6 : le poids, mesuré ─────────────────────────────────────────────────

/// Le tronc pèse ce qu'il annonce, et l'écart avec l'ancien modèle est celui qui justifie tout
/// ce module. Le chiffre de gauche est la somme des champs ; celui de droite, la taille que
/// l'énumération imposait à un nœud texte comme à tous les autres.
#[test]
fn test_le_tronc_pese_vingt_deux_octets_par_noeud_contre_cinq_cent_douze() {
    use std::mem::size_of;
    let tronc = 4 * size_of::<Fx>() + size_of::<Kind>() + size_of::<Flags>() + size_of::<NodeId>();
    assert_eq!(tronc, 22);
    assert_eq!(tronc, Arena::BYTES_PER_NODE);
    assert_eq!(
        size_of::<Annotation>(),
        512,
        "le modèle historique, pour mémoire"
    );

    // Et le SoA n'ajoute aucun remplissage : il n'y a pas de structure par nœud à aligner.
    assert_eq!(size_of::<Fx>(), 4);
    assert_eq!(size_of::<Kind>(), 1);
    assert_eq!(size_of::<Flags>(), 1);
    assert_eq!(size_of::<NodeId>(), 4);

    // Dix millions de nœuds, tronc seul : à comparer aux 5,1 Go de l'ancien modèle.
    let dix_millions = tronc * 10_000_000;
    assert!(dix_millions < 250 * 1_048_576, "{dix_millions} octets");
}

/// `NodeId::NONE` ne coûte pas un octet de plus qu'un `NodeId` : c'est tout l'intérêt de la
/// valeur réservée sur un `Option`.
#[test]
fn test_l_absence_de_parent_ne_coute_rien() {
    assert_eq!(std::mem::size_of::<NodeId>(), 4);
    assert_eq!(
        std::mem::size_of::<Option<NodeId>>(),
        8,
        "ce qu'un Option aurait coûté"
    );
    assert!(!NodeId::NONE.is_some());
}

/// `index` et `from_index` sont réciproques, et le rang irreprésentable devient l'absence de
/// nœud plutôt qu'un identifiant qui mentirait.
#[test]
fn test_l_indice_et_l_identifiant_sont_reciproques() {
    for i in [0usize, 1, 42, 1_000_000, u32::MAX as usize - 1] {
        assert_eq!(NodeId::from_index(i).index(), i);
    }
    assert_eq!(NodeId::from_index(u32::MAX as usize), NodeId::NONE);
    assert_eq!(NodeId::from_index(usize::MAX), NodeId::NONE);

    let (a, t, s, m) = trois();
    for (rang, id) in [(0, t), (1, s), (2, m)] {
        assert_eq!(NodeId::from_index(rang), id);
        assert!(a.alive(id));
    }
}

// ── ARN-1 et ARN-2 : l'identité d'un nœud ────────────────────────────────────

/// L'invariant de cohérence tient à travers une vie complète : créations, suppressions,
/// résurrections, reparentages.
#[test]
fn test_l_invariant_de_coherence_tient_a_travers_toute_une_vie() {
    let (mut a, t, s, m) = trois();
    a.check().unwrap();
    a.set_parent(t, m);
    a.set_parent(s, m);
    a.check().unwrap();
    a.kill(s);
    a.check().unwrap();
    a.revive(s);
    a.check().unwrap();
    a.kill(t);
    a.kill(m);
    a.check().unwrap();
    assert_eq!(a.len(), 1);
    assert_eq!(a.dead(), 2);
    assert_eq!(a.slots(), 3);
}

/// **ARN-2 — un identifiant ne désigne jamais un autre nœud que le sien.** C'est ce qu'une
/// génération par entrée achèterait à 4 octets près ; le tombstone le donne gratuitement.
#[test]
fn test_un_identifiant_ne_change_jamais_de_noeud() {
    let (mut a, t, s, m) = trois();
    let boite_s = a.box_of(s).unwrap();

    a.kill(t);
    let nouveau = a.spawn(Kind::Image, boite(50, 50, 100, 100), NodeId::NONE);

    // Le nouveau nœud ne réutilise pas la place du mort : l'identifiant du mort reste le sien.
    assert_ne!(nouveau, t);
    assert_eq!(nouveau.index(), 3);
    assert!(a.holds(t), "l'emplacement existe toujours");
    assert!(!a.alive(t), "mais le nœud est mort");
    assert_eq!(
        a.kind_of(t),
        Some(Kind::Text),
        "et il n'a pas changé de genre"
    );

    // Les voisins n'ont pas bougé d'un octet.
    assert_eq!(a.box_of(s), Some(boite_s));
    assert_eq!(a.kind_of(m), Some(Kind::Membrane));
}

/// **ARN-3 — supprimer puis annuler rend exactement l'état d'avant**, sans réinsertion et sans
/// réparer une seule référence. C'est ce que la pile d'undo attend d'un modèle.
#[test]
fn test_supprimer_puis_annuler_rend_exactement_l_etat_d_avant() {
    let (mut a, t, s, m) = trois();
    a.set_parent(t, m);
    let avant = (
        a.box_of(t),
        a.kind_of(t),
        a.parent_of(t),
        a.flags_of(t),
        a.len(),
    );

    assert!(a.kill(t));
    assert!(!a.alive(t));
    assert_eq!(a.len(), 2);
    assert!(!a.kill(t), "tuer deux fois ne compte qu'une");

    assert!(a.revive(t));
    assert_eq!(
        (
            a.box_of(t),
            a.kind_of(t),
            a.parent_of(t),
            a.flags_of(t),
            a.len()
        ),
        avant
    );
    assert!(!a.revive(t), "ressusciter un vivant ne fait rien");
    assert_eq!(a.dead(), 0);
    a.check().unwrap();
    let _ = s;
}

/// Un nœud mort est invisible à tout ce qui lit le document, et ne se laisse pas modifier.
#[test]
fn test_un_noeud_mort_est_invisible_et_immuable() {
    let (mut a, t, _s, _m) = trois();
    a.kill(t);
    assert_eq!(a.box_of(t), None);
    assert!(!a.set_box(t, boite(1, 1, 1, 1)));
    assert!(!a.translate(t, px(10), px(10)));

    let mut vus = Vec::new();
    a.cull(boite(-10_000, -10_000, 20_000, 20_000), &mut vus);
    assert!(!vus.contains(&t));
}

/// Un nœud verrouillé (touche L, fiche 08) ne se déplace ni ne se redimensionne, mais reste
/// lisible et sélectionnable.
#[test]
fn test_un_noeud_verrouille_ne_bouge_pas() {
    let (mut a, t, _s, _m) = trois();
    let avant = a.box_of(t).unwrap();
    a.set_flags(t, Flags::default().with(Flags::LOCKED));

    assert!(!a.translate(t, px(100), px(0)));
    assert!(!a.set_box(t, boite(9, 9, 9, 9)));
    assert_eq!(a.box_of(t), Some(avant));

    a.set_flags(t, Flags::default());
    assert!(a.translate(t, px(100), px(0)));
    assert_eq!(a.box_of(t).unwrap().x, px(100));
}

/// Poser les drapeaux ne ressuscite ni ne tue personne : la vie passe par `kill` et `revive`,
/// seuls responsables du compteur.
#[test]
fn test_poser_les_drapeaux_ne_touche_pas_a_la_vie() {
    let (mut a, t, _s, _m) = trois();
    a.kill(t);
    a.set_flags(t, Flags::default().with(Flags::LOCKED));
    assert!(!a.alive(t), "un nœud mort le reste");
    assert_eq!(a.dead(), 1);
    a.check().unwrap();

    a.revive(t);
    a.set_flags(t, Flags::default().with(Flags::DEAD));
    assert!(a.alive(t), "et un vivant ne meurt pas par un drapeau");
    assert_eq!(a.dead(), 0);
    a.check().unwrap();
}

// ── ARN-4 : la flèche rangée comme une boîte ─────────────────────────────────

/// **Les deux points d'une flèche sont un rectangle plus deux bits.** L'aller-retour est exact
/// dans les quatre orientations et les deux sens, y compris dégénérées (horizontale, verticale,
/// nulle).
#[test]
fn test_une_fleche_traverse_sa_boite_sans_rien_perdre() {
    let mut a = Arena::new();
    let f = a.spawn(
        Kind::Arrow,
        Box2::new(Fx::ZERO, Fx::ZERO, Fx::ZERO, Fx::ZERO),
        NodeId::NONE,
    );

    // Les quatre coins sources possibles, chacun dans les deux sens : c'est exactement ce que
    // la première version, à un seul bit, ne savait pas distinguer.
    let cas = [
        (0, 0, 300, 200),  // source en haut à gauche
        (300, 200, 0, 0),  // source en bas à droite — même boîte, sens inverse
        (300, 0, 0, 200),  // source en haut à droite
        (0, 200, 300, 0),  // source en bas à gauche — même boîte, sens inverse
        (10, 50, 400, 50), // horizontale
        (400, 50, 10, 50), // horizontale, sens inverse
        (50, 10, 50, 400), // verticale
        (50, 400, 50, 10), // verticale, sens inverse
        (77, 77, 77, 77),  // nulle
        (-1000, -2000, -3, -4),
    ];
    for (ax, ay, bx, by) in cas {
        assert!(a.set_arrow(f, px(ax), px(ay), px(bx), px(by)));
        let (rx, ry, rbx, rby) = a.arrow_endpoints(f).unwrap();
        assert_eq!(
            (rx, ry, rbx, rby),
            (px(ax), px(ay), px(bx), px(by)),
            "flèche ({ax},{ay}) → ({bx},{by})"
        );
        // Et sa boîte est bien la boîte englobante : le culling n'a rien à recalculer.
        let b = a.box_of(f).unwrap();
        assert_eq!(b.x, px(ax.min(bx)));
        assert_eq!(b.y, px(ay.min(by)));
        assert_eq!(b.right(), px(ax.max(bx)));
        assert_eq!(b.bottom(), px(ay.max(by)));
    }
}

/// Les extrémités ne se lisent que sur une flèche : demander celles d'une carte n'a pas de sens
/// et ne rend rien.
#[test]
fn test_seules_les_fleches_ont_des_extremites() {
    let (mut a, t, _s, _m) = trois();
    assert_eq!(a.arrow_endpoints(t), None);
    assert!(!a.set_arrow(t, px(0), px(0), px(1), px(1)));
    assert_eq!(a.arrow_endpoints(NodeId::NONE), None);
}

// ── ARN-5 : la requête de viewport ───────────────────────────────────────────

/// Le culling rend exactement les nœuds vivants qui croisent la vue — vérifié contre un
/// balayage naïf, sur une grille dont on connaît la réponse.
#[test]
fn test_le_culling_rend_exactement_les_noeuds_qui_croisent_la_vue() {
    let mut a = Arena::with_capacity(400);
    let mut ids = Vec::new();
    for i in 0..20 {
        for j in 0..20 {
            ids.push(a.spawn(Kind::Text, boite(i * 100, j * 100, 50, 50), NodeId::NONE));
        }
    }
    // Un nœud sur sept meurt : le culling doit les ignorer.
    for (k, id) in ids.iter().enumerate() {
        if k % 7 == 0 {
            a.kill(*id);
        }
    }

    let mut vus = Vec::new();
    for vue in [
        boite(0, 0, 250, 250),
        boite(-500, -500, 100, 100),
        boite(950, 950, 10, 10),
        boite(-1000, -1000, 5000, 5000),
        boite(175, 175, 1, 1),
    ] {
        a.cull(vue, &mut vus);
        let attendu: Vec<NodeId> = ids
            .iter()
            .copied()
            .filter(|&id| a.box_of(id).is_some_and(|b| b.overlaps(vue)))
            .collect();
        assert_eq!(vus, attendu, "vue {vue:?}");
    }
}

/// Le recouvrement et l'inclusion sont exacts au bord : un nœud qui touche la vue par un seul
/// point est vu. Aucun epsilon n'intervient, donc aucun cas limite ne dépend d'un réglage.
#[test]
fn test_le_recouvrement_est_exact_au_bord() {
    let b = boite(0, 0, 100, 100);
    assert!(b.overlaps(boite(100, 100, 10, 10)), "coin contre coin");
    assert!(
        !b.overlaps(Box2::new(px(100) + Fx::EPSILON, px(100), px(10), px(10))),
        "un 256e de pixel plus loin, non"
    );
    assert!(b.contains(px(0), px(0)));
    assert!(b.contains(px(100), px(100)));
    assert!(!b.contains(px(100) + Fx::EPSILON, px(100)));
}

// ── Les gardes ───────────────────────────────────────────────────────────────

/// Un identifiant étranger, absent ou nul ne fait rien tomber et ne modifie rien.
#[test]
fn test_un_identifiant_inconnu_ne_fait_rien_tomber() {
    let (mut a, _t, _s, _m) = trois();
    let etranger = NodeId::NONE;
    let hors_bornes = Arena::new().spawn(Kind::Text, boite(0, 0, 1, 1), NodeId::NONE);
    let lointain = NodeId(999);

    for id in [etranger, lointain] {
        assert!(!a.alive(id));
        assert_eq!(a.box_of(id), None);
        assert_eq!(a.kind_of(id), None);
        assert_eq!(a.flags_of(id), None);
        assert_eq!(a.parent_of(id), None);
        assert!(!a.kill(id));
        assert!(!a.revive(id));
        assert!(!a.set_box(id, boite(0, 0, 1, 1)));
        assert!(!a.translate(id, px(1), px(1)));
        assert!(!a.set_flags(id, Flags::default()));
        assert!(!a.set_parent(id, NodeId::NONE));
    }
    // Un identifiant né dans une autre arène est un indice comme un autre : il désigne ici le
    // nœud de même rang. C'est une limite connue des arènes, et la raison pour laquelle un
    // document n'en a qu'une.
    assert!(a.alive(hors_bornes));
    a.check().unwrap();
}

/// `check` détecte une incohérence plutôt que de la laisser passer — vérifié en la fabriquant.
#[test]
fn test_la_verification_attrape_une_incoherence() {
    let (mut a, t, _s, m) = trois();
    a.dead += 1; // compteur menteur
    assert!(a.check().unwrap_err().contains("morts"));
    a.dead -= 1;
    a.check().unwrap();

    a.parent[t.index()] = NodeId(42); // parent hors arène
    assert!(a.check().unwrap_err().contains("hors arène"));
    a.parent[t.index()] = m;
    a.check().unwrap();

    a.y.push(Fx::ZERO); // tableau désaligné
    assert!(a.check().unwrap_err().contains("tableau y"));
}
