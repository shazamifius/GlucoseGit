//! Les lois de l'arène de texte.

use super::*;

fn id(i: usize) -> NodeId {
    NodeId::from_index(i)
}

/// Un intervalle pèse huit octets par nœud, contenu exclu — à comparer aux 24 octets d'en-tête
/// d'un `String`, plus son allocation.
#[test]
fn test_un_intervalle_pese_huit_octets_contre_vingt_quatre_pour_une_chaine() {
    assert_eq!(TextArena::BYTES_PER_NODE, 8);
    assert_eq!(std::mem::size_of::<String>(), 24);
}

/// Ce qu'on écrit est ce qu'on relit, y compris hors de l'ASCII : l'arène range des octets
/// mais rend des chaînes, et les frontières de caractères sont tenues.
#[test]
fn test_le_texte_relu_est_celui_qui_a_ete_ecrit() {
    let mut t = TextArena::new();
    let cas = [
        (0, "Un nœud de départ"),
        (1, ""),
        (2, "日本語とémojis 🌱🜂"),
        (3, "a"),
        (7, "un trou avant moi : les nœuds 4, 5 et 6 n'ont pas de texte"),
    ];
    for (i, texte) in cas {
        assert!(t.set(id(i), texte));
    }
    for (i, texte) in cas {
        assert_eq!(t.get(id(i)), texte, "nœud {i}");
        assert_eq!(t.has(id(i)), !texte.is_empty());
    }
    // Un nœud jamais écrit, ou hors de l'arène, n'a pas de texte plutôt qu'une erreur.
    assert_eq!(t.get(id(5)), "");
    assert_eq!(t.get(id(9_999)), "");
    assert_eq!(t.get(NodeId::NONE), "");
    assert!(!t.has(id(5)));
    t.check().unwrap();
}

/// Poser un texte identique à celui en place ne recopie rien et ne gaspille rien — la frappe
/// d'une touche qui ne change pas le texte ne doit rien coûter.
#[test]
fn test_reecrire_le_meme_texte_ne_coute_rien() {
    let mut t = TextArena::new();
    t.set(id(0), "inchangé");
    let (octets, perdu) = (t.bytes_used(), t.waste());
    for _ in 0..100 {
        assert!(t.set(id(0), "inchangé"));
    }
    assert_eq!((t.bytes_used(), t.waste()), (octets, perdu));
    assert_eq!(t.waste(), 0);
    t.check().unwrap();
}

/// Réécrire abandonne exactement l'ancien texte, ni plus ni moins. C'est le prix annoncé par
/// l'en-tête du module, et il est mesuré à tout instant.
#[test]
fn test_reecrire_abandonne_exactement_l_ancien_texte() {
    let mut t = TextArena::new();
    t.set(id(0), "0123456789"); // 10 octets
    assert_eq!((t.bytes_used(), t.waste()), (10, 0));

    t.set(id(0), "abc"); // 3 de plus, 10 abandonnés
    assert_eq!((t.bytes_used(), t.waste()), (13, 10));
    assert_eq!(t.get(id(0)), "abc");

    t.clear(id(0)); // 3 de plus abandonnés
    assert_eq!((t.bytes_used(), t.waste()), (13, 13));
    assert_eq!(t.get(id(0)), "");
    t.check().unwrap();
}

/// Le compactage rend exactement les octets gaspillés, sans perdre un seul texte.
#[test]
fn test_le_compactage_rend_exactement_ce_qui_etait_perdu() {
    let mut t = TextArena::new();
    // Une session d'édition : dix nœuds, chacun réécrit plusieurs fois.
    for tour in 0..5 {
        for i in 0..10 {
            t.set(id(i), &format!("nœud {i}, version {tour} — du texte pour occuper la place"));
        }
    }
    let attendus: Vec<String> = (0..10).map(|i| t.get(id(i)).to_string()).collect();
    let (occupe, perdu) = (t.bytes_used(), t.waste());
    assert!(perdu > 0, "la session doit avoir gaspillé quelque chose");
    t.check().unwrap();

    let rendu = t.compact();
    assert_eq!(rendu, perdu, "le compactage rend exactement le gaspillage");
    assert_eq!(t.bytes_used(), occupe - perdu);
    assert_eq!(t.waste(), 0);
    for (i, attendu) in attendus.iter().enumerate() {
        assert_eq!(t.get(id(i)), attendu, "nœud {i} après compactage");
    }
    t.check().unwrap();

    // Compacter une arène propre ne fait rien et ne coûte rien.
    assert_eq!(t.compact(), 0);
    t.check().unwrap();
}

/// Le compactage range le texte dans l'ordre des nœuds : ce qui est voisin dans le document
/// devient voisin en mémoire.
#[test]
fn test_le_compactage_remet_le_texte_dans_l_ordre_des_noeuds() {
    let mut t = TextArena::new();
    // Écrits dans le désordre, donc dispersés dans le bloc.
    for i in [4usize, 0, 2, 1, 3] {
        t.set(id(i), &format!("<{i}>"));
    }
    t.set(id(2), "<2 réécrit>"); // et un trou au milieu
    t.compact();

    // Après compactage, les textes se suivent dans l'ordre des identifiants.
    let recompose: String = (0..5).map(|i| t.get(id(i))).collect();
    assert_eq!(recompose, "<0><1><2 réécrit><3><4>");
    t.check().unwrap();
}

/// La règle de saturation, vérifiée sur des nombres : fabriquer l'état qui la déclenche
/// demanderait quatre gigaoctets, ce qu'un test ne doit pas faire.
#[test]
fn test_la_regle_de_saturation_borne_l_arene_a_quatre_giga_octets() {
    let plein = u32::MAX as usize;
    assert!(!deborde(0, plein));
    assert!(!deborde(plein - 10, 10));
    assert!(deborde(plein - 10, 11));
    assert!(deborde(plein, 1));
    assert!(deborde(usize::MAX, 1), "aucun débordement d'entier en chemin");
}

/// Un identifiant qui ne désigne aucun nœud ne se voit pas attribuer de texte.
#[test]
fn test_l_absence_de_noeud_ne_recoit_pas_de_texte() {
    let mut t = TextArena::new();
    assert!(!t.set(NodeId::NONE, "nulle part"));
    assert_eq!(t.bytes_used(), 0);
    assert_eq!(t.slots(), 0);
    t.clear(NodeId::NONE);
    t.check().unwrap();
}

/// La vérification attrape une incohérence plutôt que de la laisser passer.
#[test]
fn test_la_verification_attrape_une_incoherence() {
    let mut t = TextArena::new();
    t.set(id(0), "héros"); // 6 octets : le é en fait deux
    t.check().unwrap();

    t.spans[0].len += 1; // intervalle hors du bloc
    assert!(t.check().unwrap_err().contains("hors des"));
    t.spans[0].len -= 1;

    // Couper le « é » en deux demande un intervalle d'UN octet à partir de sa première moitié :
    // « héros » vaut [h, C3, A9, r, o, s], et start=1 len=2 capturerait le « é » entier.
    t.spans[0] = Span { start: 1, len: 1 };
    assert!(t.check().unwrap_err().contains("frontières"));
    t.spans[0] = Span { start: 0, len: 6 };
    t.check().unwrap();

    t.waste += 1; // comptable menteur
    assert!(t.check().unwrap_err().contains("gaspillés"));
}
