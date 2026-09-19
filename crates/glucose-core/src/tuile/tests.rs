//! Ce que TUILE-1 doit garantir — la géométrie, et la propriété de partage.

use super::*;

fn occupant(left: f64, top: f64, aspect: u64) -> Occupant {
    Occupant {
        boite: Rect::new(left, top, 100.0, 80.0),
        aspect,
    }
}

// ── La grille ───────────────────────────────────────────────────────────────

/// **La grille est ancrée au monde, pas à l'écran**, et c'est toute la différence avec les
/// vignettes : la tuile qui contient un point du document ne dépend d'aucune vue.
#[test]
fn la_tuile_d_un_point_ne_depend_que_du_point_et_du_niveau() {
    let a = Adresse::contenant(0, 300.0, 700.0);
    assert_eq!(
        a,
        Adresse {
            niveau: 0,
            x: 1,
            y: 2
        }
    );
    assert!(a
        .couvre()
        .contains_point(crate::geometry::Point { x: 300.0, y: 700.0 }));

    // Les coordonnées négatives tombent du bon côté : `floor`, et non une troncature qui
    // ferait partager la tuile zéro aux deux moitiés du plan.
    assert_eq!(
        Adresse::contenant(0, -1.0, -1.0),
        Adresse {
            niveau: 0,
            x: -1,
            y: -1
        }
    );
    assert_eq!(
        Adresse::contenant(0, 0.0, 0.0),
        Adresse {
            niveau: 0,
            x: 0,
            y: 0
        }
    );
}

/// Les niveaux sont dyadiques : monter d'un niveau double l'échelle et **divise par deux** ce
/// qu'une tuile couvre du monde.
#[test]
fn un_niveau_de_plus_couvre_deux_fois_moins_de_monde() {
    for niveau in -4..8 {
        assert!(
            (Adresse::cote_monde(niveau) - 2.0 * Adresse::cote_monde(niveau + 1)).abs() < 1e-9,
            "au niveau {niveau}"
        );
    }
    assert_eq!(Adresse::cote_monde(0), f64::from(COTE));
}

/// **On descend, on ne monte pas.** Le niveau retenu est celui dont l'échelle ne dépasse pas
/// celle demandée : agrandir une tuile d'un facteur entre un et deux reste dans le régime que
/// `bench_tuiles` mesure, tandis que réduire jetterait du détail déjà payé.
#[test]
fn le_niveau_retenu_ne_depasse_jamais_l_echelle_demandee() {
    for exposant in -6..10 {
        let exacte = f64::from(exposant).exp2();
        assert_eq!(Adresse::niveau_pour(exacte), exposant, "a {exacte}");
        // Juste au-dessus d'une puissance de deux : on reste au niveau du dessous.
        assert_eq!(Adresse::niveau_pour(exacte * 1.99), exposant);
    }
    // Et l'échelle du niveau retenu ne dépasse jamais celle demandée.
    for echelle in [0.03, 0.5, 1.0, 1.7, 3.2, 19.9] {
        let niveau = Adresse::niveau_pour(echelle);
        assert!(
            Adresse::echelle(niveau) <= echelle * (1.0 + 1e-12),
            "a {echelle} le niveau {niveau} donne {}",
            Adresse::echelle(niveau)
        );
    }
}

/// Une échelle absurde ne doit ni paniquer ni rendre un niveau insensé.
#[test]
fn une_echelle_insensee_retombe_sur_le_niveau_zero() {
    for echelle in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(Adresse::niveau_pour(echelle), 0, "a {echelle}");
    }
}

// ── La couverture ───────────────────────────────────────────────────────────

/// **Toutes les tuiles rendues rencontrent la région, et aucune ne manque.** C'est la seule
/// chose que `couvrant` promet, et un trou laisserait une bande vide à l'écran.
#[test]
fn le_couvrant_rend_exactement_les_tuiles_qui_rencontrent_la_region() {
    let region = Rect::new(100.0, 100.0, 600.0, 400.0);
    let tuiles: Vec<Adresse> = couvrant(0, region).collect();

    // Aucune tuile inutile.
    for t in &tuiles {
        let c = t.couvre();
        assert!(
            c.right() > region.left
                && c.left < region.right()
                && c.bottom() > region.top
                && c.top < region.bottom(),
            "{t:?} ne rencontre pas la region"
        );
    }
    // Et aucun point de la région hors des tuiles rendues.
    for (x, y) in [
        (100.0, 100.0),
        (400.0, 300.0),
        (699.9, 499.9),
        (256.0, 256.0),
    ] {
        assert!(
            tuiles.contains(&Adresse::contenant(0, x, y)),
            "le point ({x}, {y}) n'est couvert par aucune tuile"
        );
    }
}

/// Un rectangle qui s'arrête **pile** sur une frontière ne demande pas la tuile d'après :
/// elle n'en montrerait aucun pixel, et la peindre serait du travail pur perdu.
#[test]
fn un_bord_pose_sur_une_frontiere_ne_reclame_pas_la_tuile_suivante() {
    let cote = f64::from(COTE);
    let tuiles: Vec<Adresse> = couvrant(0, Rect::new(0.0, 0.0, cote, cote)).collect();
    assert_eq!(tuiles.len(), 1, "une seule tuile, pas quatre : {tuiles:?}");
    assert_eq!(
        tuiles[0],
        Adresse {
            niveau: 0,
            x: 0,
            y: 0
        }
    );
}

/// Une région vide ou insensée ne rend aucune tuile — plutôt que des milliards.
#[test]
fn une_region_insensee_ne_reclame_aucune_tuile() {
    for region in [
        Rect::new(0.0, 0.0, 0.0, 100.0),
        Rect::new(0.0, 0.0, -50.0, 100.0),
        Rect::new(f64::NAN, 0.0, 100.0, 100.0),
        Rect::new(0.0, 0.0, f64::INFINITY, 100.0),
    ] {
        assert_eq!(couvrant(0, region).count(), 0, "pour {region:?}");
    }
}

/// L'ordre est celui de la lecture, et il est **stable** : deux parcours de la même région
/// donnent la même suite, donc le même ordre de chantier.
#[test]
fn le_parcours_est_stable_et_se_lit_ligne_par_ligne() {
    let region = Rect::new(0.0, 0.0, 600.0, 600.0);
    let un: Vec<Adresse> = couvrant(0, region).collect();
    let deux: Vec<Adresse> = couvrant(0, region).collect();
    assert_eq!(un, deux);
    assert_eq!(un.len(), 9, "trois par trois : {un:?}");
    // Ligne par ligne : `y` ne décroît jamais, et `x` repart à chaque ligne.
    for paire in un.windows(2) {
        assert!(paire[1].y > paire[0].y || (paire[1].y == paire[0].y && paire[1].x > paire[0].x));
    }
}

// ── L'empreinte ─────────────────────────────────────────────────────────────

/// **Deux régions identiques du canevas donnent la même empreinte, où qu'elles soient.**
///
/// C'est la propriété de Hashlife transposée, et la seule raison d'être de ce module : un mur
/// de photos posé deux fois ne se peint qu'une, parce que les coordonnées sont **relatives au
/// coin de la tuile** et non au monde.
///
/// La première version de ce test écrivait `10.0 + decalage - decalage` : le décalage
/// s'annulait, le test était vrai par construction et ne prouvait rien. Celui-ci part de
/// vraies positions du monde et les ramène dans le repère de leur tuile, comme l'appelant
/// devra le faire.
#[test]
fn deux_regions_identiques_partagent_leur_empreinte_ou_qu_elles_soient() {
    /// Ce que porte la tuile qui contient `(x, y)`, vu depuis SON coin.
    fn ce_que_porte(niveau: i32, monde: &[(f64, f64, u64)]) -> Empreinte {
        let tuile = Adresse::contenant(niveau, monde[0].0, monde[0].1);
        let (ox, oy) = tuile.origine_en_pixels();
        let echelle = Adresse::echelle(niveau);
        Empreinte::de(monde.iter().map(|&(x, y, aspect)| Occupant {
            boite: Rect::new(x * echelle - ox, y * echelle - oy, 100.0, 80.0),
            aspect,
        }))
    }

    let ici = [(1000.0, 2000.0, 0xaa), (1040.0, 2030.0, 0xbb)];
    // Le même motif, déplacé d'un nombre ENTIER de tuiles : il retombe donc à la même place
    // dans sa tuile, et doit se peindre une seule fois pour les deux.
    let cote = Adresse::cote_monde(0);
    let ailleurs: Vec<(f64, f64, u64)> = ici
        .iter()
        .map(|&(x, y, a)| (x + 4096.0 * cote, y - 777.0 * cote, a))
        .collect();

    assert_eq!(
        ce_que_porte(0, &ici),
        ce_que_porte(0, &ailleurs),
        "le meme motif, a mille lieues de la, doit partager son rendu"
    );

    // Et un motif qui ne tombe PAS au même endroit dans sa tuile ne se confond pas avec lui.
    let decale: Vec<(f64, f64, u64)> = ici.iter().map(|&(x, y, a)| (x + 7.0, y, a)).collect();
    assert_ne!(ce_que_porte(0, &ici), ce_que_porte(0, &decale));
}

/// **Une tuile vide a toujours la même empreinte** — et c'est le cas le plus fréquent d'un
/// canevas infini. Se déplacer sur du vide ne dessine donc rien, quel que soit le nombre de
/// tuiles traversées.
#[test]
fn toutes_les_tuiles_vides_se_valent() {
    assert_eq!(Empreinte::vide(), Empreinte::de([]));
    assert_ne!(Empreinte::vide(), Empreinte::de([occupant(0.0, 0.0, 1)]));
}

/// **L'ordre de dessin compte.** Deux nœuds qui se recouvrent ne donnent pas la même image
/// selon lequel passe devant ; une empreinte qui les confondrait ferait réutiliser le mauvais
/// rendu, et le défaut serait invisible jusqu'à ce qu'on regarde de près.
#[test]
fn l_ordre_de_dessin_change_l_empreinte() {
    let a = occupant(0.0, 0.0, 0x11);
    let b = occupant(0.0, 0.0, 0x22);
    assert_ne!(Empreinte::de([a, b]), Empreinte::de([b, a]));
}

/// Chaque champ d'un occupant compte : en oublier un ferait réutiliser un rendu pour un
/// contenu qui a changé — exactement le défaut qu'un cache ne doit jamais avoir.
#[test]
fn aucun_champ_d_un_occupant_n_est_ignore() {
    let base = Occupant {
        boite: Rect::new(1.0, 2.0, 3.0, 4.0),
        aspect: 7,
    };
    let variantes = [
        Occupant {
            boite: Rect::new(9.0, 2.0, 3.0, 4.0),
            ..base
        },
        Occupant {
            boite: Rect::new(1.0, 9.0, 3.0, 4.0),
            ..base
        },
        Occupant {
            boite: Rect::new(1.0, 2.0, 9.0, 4.0),
            ..base
        },
        Occupant {
            boite: Rect::new(1.0, 2.0, 3.0, 9.0),
            ..base
        },
        Occupant { aspect: 8, ..base },
    ];
    let reference = Empreinte::de([base]);
    for (rang, variante) in variantes.into_iter().enumerate() {
        assert_ne!(
            Empreinte::de([variante]),
            reference,
            "le champ {rang} ne compte pas dans l'empreinte"
        );
    }
}

/// Un décalage d'un millième de pixel donne des pixels différents une fois le filtre
/// appliqué : l'empreinte doit le voir, sous peine de réutiliser un rendu qui n'est pas le bon.
#[test]
fn un_decalage_sous_le_pixel_ne_passe_pas_inapercu() {
    assert_ne!(
        Empreinte::de([occupant(10.0, 20.0, 1)]),
        Empreinte::de([occupant(10.001, 20.0, 1)])
    );
}
