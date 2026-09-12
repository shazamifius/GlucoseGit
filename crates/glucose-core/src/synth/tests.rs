//! Les lois du générateur. Elles portent toutes sur la seule chose qu'on lui demande :
//! produire deux fois le même document, et produire la forme annoncée.

use super::*;

/// **La même graine rend le même document.** C'est la propriété qui donne son sens à un banc :
/// sans elle, deux mesures ne comparent pas la même scène.
#[test]
fn test_la_meme_graine_rend_le_meme_document() {
    let a = document(400, 5_000.0, Shape::Clustered, 0x5eed);
    let b = document(400, 5_000.0, Shape::Clustered, 0x5eed);
    let board_a = a.active_board().expect("un tableau");
    let board_b = b.active_board().expect("un tableau");

    assert_eq!(board_a.annotations, board_b.annotations, "les annotations diffèrent");
    assert_eq!(board_a.images.len(), board_b.images.len());
    for (x, y) in board_a.images.iter().zip(&board_b.images) {
        assert_eq!((x.id.as_str(), x.x, x.y), (y.id.as_str(), y.x, y.y));
    }
}

/// Une autre graine rend un autre document — sinon la graine ne servirait à rien.
#[test]
fn test_une_autre_graine_rend_un_autre_document() {
    let a = document(200, 5_000.0, Shape::Uniform, 1);
    let b = document(200, 5_000.0, Shape::Uniform, 2);
    assert_ne!(
        a.active_board().expect("a").annotations,
        b.active_board().expect("b").annotations
    );
}

/// Le document contient exactement les `n` nœuds demandés — plus les longues flèches de la
/// forme Wikipédia, qui s'ajoutent et ne remplacent rien.
#[test]
fn test_le_compte_de_noeuds_est_celui_demande() {
    for shape in [Shape::Uniform, Shape::Clustered] {
        let s = document(100, 4_000.0, shape, 7);
        let b = s.active_board().expect("un tableau");
        assert_eq!(b.images.len() + b.annotations.len(), 100, "{shape:?}");
    }

    let w = document(100, 4_000.0, Shape::Wikipedia, 7);
    let b = w.active_board().expect("un tableau");
    let total = b.images.len() + b.annotations.len();
    assert!(total > 100, "la forme Wikipédia ajoute ses arêtes : {total}");
}

/// **Les amas laissent du vide, la pluie uniforme n'en laisse pas.**
///
/// La mesure est directe : on pose une grille sur le carré et on compte les cellules vides.
/// C'est exact, ça ne dépend d'aucun seuil arbitraire, et c'est la propriété qui fait qu'un
/// document en amas met un index spatial en difficulté là où une pluie uniforme le flatte.
#[test]
fn test_les_amas_laissent_du_vide_la_pluie_n_en_laisse_pas() {
    const SPAN: f64 = 10_000.0;
    const COTE: usize = 20;

    fn cellules_vides(store: &Store) -> usize {
        let mut occupee = [false; COTE * COTE];
        let board = store.active_board().expect("un tableau");
        let mut marquer = |x: f64, y: f64| {
            let i = ((x / SPAN * COTE as f64) as isize).clamp(0, COTE as isize - 1) as usize;
            let j = ((y / SPAN * COTE as f64) as isize).clamp(0, COTE as isize - 1) as usize;
            occupee[j * COTE + i] = true;
        };
        for a in &board.annotations {
            marquer(a.x(), a.y());
        }
        for img in &board.images {
            marquer(img.x, img.y);
        }
        occupee.iter().filter(|o| !**o).count()
    }

    let uniforme = cellules_vides(&document(2_000, SPAN, Shape::Uniform, 3));
    let amas = cellules_vides(&document(2_000, SPAN, Shape::Clustered, 3));

    assert!(uniforme < 40, "la pluie uniforme doit couvrir presque tout : {uniforme} vides");
    assert!(amas > uniforme * 3, "les amas doivent laisser bien plus de vide : {amas} contre {uniforme}");
}

/// Les longues arêtes de la forme Wikipédia traversent vraiment le document : elles relient
/// deux amas, donc leur longueur est de l'ordre du carré, pas de celui d'un amas.
#[test]
fn test_la_forme_wikipedia_a_de_vraies_longues_aretes() {
    const SPAN: f64 = 10_000.0;
    let s = document(1_000, SPAN, Shape::Wikipedia, 11);
    let longueurs: Vec<f64> = s
        .active_board()
        .expect("un tableau")
        .annotations
        .iter()
        .filter_map(|a| match a {
            Annotation::Arrow { x, y, x2, y2, .. } => Some(((x2 - x).powi(2) + (y2 - y).powi(2)).sqrt()),
            _ => None,
        })
        .collect();

    assert!(!longueurs.is_empty(), "il doit y avoir des arêtes");
    let plus_longue = longueurs.iter().cloned().fold(0.0_f64, f64::max);
    assert!(plus_longue > SPAN / 4.0, "l'arête la plus longue traverse : {plus_longue}");
}

/// La forme uniforme n'a aucune flèche : ses nœuds ne sont reliés à rien, et c'est voulu.
#[test]
fn test_les_formes_sans_aretes_n_en_ont_aucune() {
    for shape in [Shape::Uniform, Shape::Clustered] {
        let s = document(200, 4_000.0, shape, 5);
        let fleches = s
            .active_board()
            .expect("un tableau")
            .annotations
            .iter()
            .filter(|a| matches!(a, Annotation::Arrow { .. }))
            .count();
        assert_eq!(fleches, 0, "{shape:?}");
    }
}

/// Un document synthétique arrive **sans historique** : un banc mesure un document, pas
/// l'annulation de sa fabrication.
#[test]
fn test_un_document_synthetique_n_a_pas_d_historique() {
    let s = document(50, 1_000.0, Shape::Uniform, 1);
    assert!(!s.can_undo(), "rien à annuler");
    assert!(s.selected_annotation_ids.is_empty(), "aucune annotation sélectionnée");
    assert!(s.selected_image_ids.is_empty(), "aucune image sélectionnée");
}

/// **La scène témoin contient un exemplaire de chaque chose qui se dessine.** Ce test est la
/// garde de son rôle : le jour où une fonctionnalité de rendu arrive sans y ajouter son
/// exemplaire, c'est ici qu'on doit s'en apercevoir.
#[test]
fn test_la_scene_temoin_contient_un_de_chaque() {
    let s = witness();
    let b = s.active_board().expect("un tableau");

    let a_du = |f: fn(&Annotation) -> bool| b.annotations.iter().any(f);
    assert!(a_du(|a| matches!(a, Annotation::Text { .. })), "une carte de texte");
    assert!(a_du(|a| matches!(a, Annotation::Sticky { .. })), "un pense-bête");
    assert!(a_du(|a| matches!(a, Annotation::Arrow { .. })), "une flèche");
    assert!(a_du(|a| matches!(a, Annotation::Membrane { .. })), "une membrane");
    assert!(!b.folders.is_empty(), "un dossier");

    // Et le texte y couvre ce que le moteur doit savoir rendre.
    let corpus: String = b
        .annotations
        .iter()
        .filter_map(|a| match a {
            Annotation::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert!(corpus.contains("# "), "un titre");
    assert!(corpus.contains("## "), "un sous-titre");
    assert!(corpus.contains("- "), "une puce");
    assert!(corpus.contains('é'), "des accents");
}

/// La scène témoin est elle aussi reproductible : elle ne tire rien au hasard.
#[test]
fn test_la_scene_temoin_est_toujours_la_meme() {
    let a = witness();
    let b = witness();
    assert_eq!(
        a.active_board().expect("a").annotations,
        b.active_board().expect("b").annotations
    );
}

/// Les bits de poids faible d'un LCG ont une période très courte — c'est le piège classique de
/// cette famille. `next` n'en rend aucun, et ce test le vérifie là où le défaut serait visible :
/// la parité d'une suite de tirages ne doit pas alterner.
#[test]
fn test_la_suite_ne_trahit_pas_ses_bits_de_poids_faible() {
    let mut rng = Lcg::new(1);
    let mut alternances = 0;
    let mut precedent = rng.next_u64() % 2;
    for _ in 0..1_000 {
        let parite = rng.next_u64() % 2;
        if parite != precedent {
            alternances += 1;
        }
        precedent = parite;
    }
    // Une suite sans structure alterne une fois sur deux, soit ~500 sur 1 000. Un LCG qui
    // rendrait son bit de poids faible alternerait 1 000 fois sur 1 000.
    assert!((400..600).contains(&alternances), "alternances : {alternances}");
}
