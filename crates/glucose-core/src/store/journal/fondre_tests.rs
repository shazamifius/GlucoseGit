//! FONDRE-1 : ce qu'un geste publie est la composée exacte de ce qu'il a écrit, et rien de plus.

use super::super::edit::{Bouts, Edit};
use super::super::{Journal, Slot, Transaction, Whole};
use crate::types::{BoardImage, Project};

fn image(id: &str, x: f64) -> BoardImage {
    BoardImage::new(id, x, 0.0, 100.0, 100.0)
}

fn change(index: usize, avant: f64, apres: f64) -> Edit {
    Edit::Image {
        board: "main".into(),
        slot: Slot::changed(index, image("i", avant), image("i", apres)),
    }
}

fn fondre(edits: Vec<Edit>) -> Vec<Edit> {
    let mut tx = Transaction { edits };
    tx.fondre();
    tx.edits
}

/// **Cent changements de la même case n'en font qu'un** : l'avant du premier, l'après du
/// dernier — le redimensionnement d'une carte pendant cinq secondes.
#[test]
fn test_fondre_1_cent_changements_d_une_case_n_en_font_qu_un() {
    let edits = (0..100)
        .map(|k| change(3, f64::from(k) * 0.1, f64::from(k + 1) * 0.1))
        .collect();
    assert_eq!(fondre(edits), vec![change(3, 0.0, 100.0 * 0.1)]);
}

/// **Insérer, changer, puis retirer la même case ne laisse rien** ; retirer puis insérer à la
/// même place est un changement.
#[test]
fn test_fondre_1_ce_qui_ne_change_rien_disparait() {
    let i = |x| Some(Box::new(image("i", x)));
    let slot = |avant, apres| Edit::Image {
        board: "main".into(),
        slot: Slot {
            index: 2,
            before: avant,
            after: apres,
        },
    };
    assert!(fondre(vec![
        slot(None, i(1.0)),
        slot(i(1.0), i(2.0)),
        slot(i(2.0), None)
    ])
    .is_empty());
    assert_eq!(
        fondre(vec![slot(i(1.0), None), slot(None, i(5.0))]),
        vec![slot(i(1.0), i(5.0))]
    );
}

/// **Seules des éditions adjacentes se fondent**, et seulement quand la seconde part de là où
/// la première arrive : une autre case entre deux, ou un avant qui ne suit pas, et tout reste.
#[test]
fn test_fondre_1_seules_des_voisines_qui_se_suivent_se_fondent() {
    let entrelacees = vec![
        change(1, 0.0, 1.0),
        change(2, 0.0, 1.0),
        change(1, 1.0, 2.0),
    ];
    assert_eq!(fondre(entrelacees.clone()), entrelacees);
    let decousues = vec![change(1, 0.0, 1.0), change(1, 7.0, 8.0)];
    assert_eq!(fondre(decousues.clone()), decousues);
    // Deux cases différentes, dont les valeurs s'enchaînent pourtant : rien ne se fond.
    let deux_cases = vec![change(1, 0.0, 1.0), change(2, 1.0, 2.0)];
    assert_eq!(fondre(deux_cases.clone()), deux_cases);
    let tableaux = vec![
        change(1, 0.0, 1.0),
        Edit::Image {
            board: "autre".into(),
            slot: Slot::changed(1, image("i", 1.0), image("i", 2.0)),
        },
    ];
    assert_eq!(fondre(tableaux.clone()), tableaux);
}

/// **Une translation ne se fond jamais** : elle porte un pas, et additionner des pas ne redonne
/// pas la position qu'ils ont laissée (GLISSER-1 s'en charge).
#[test]
fn test_fondre_1_une_translation_ne_se_fond_pas() {
    let pas = |d| Edit::Translation {
        board: "main".into(),
        delta: (d, 0.0),
        images: vec![0],
        annotations: vec![],
        folders: vec![],
        bouts: vec![(
            0,
            Bouts {
                origine: true,
                cible: false,
            },
        )],
    };
    assert_eq!(fondre(vec![pas(0.1), pas(0.2)]), vec![pas(0.1), pas(0.2)]);
}

/// Un nom changé deux fois de suite n'en fait qu'un changement.
#[test]
fn test_fondre_1_une_valeur_entiere_aussi() {
    let nom = |a: &str, b: &str| Edit::ProjectName {
        whole: Whole::new(a.to_string(), b.to_string()),
    };
    assert_eq!(
        fondre(vec![nom("a", "b"), nom("b", "c")]),
        vec![nom("a", "c")]
    );
    let decousus = vec![nom("a", "b"), nom("x", "y")];
    assert_eq!(
        fondre(decousus.clone()),
        decousus,
        "un avant qui ne suit pas"
    );
}

/// **Le geste publié rejoue exactement ce que le geste a fait** : cent pas fractionnaires d'une
/// même case, appliqués un à un au document, puis le geste fondu rejoué sur le document de
/// départ — les deux documents sont égaux, au bit près ; et le geste défait rend le départ.
#[test]
fn test_fondre_1_le_geste_publie_rejoue_exactement_le_geste() {
    let mut depart = Project::new("fondre");
    depart
        .boards
        .iter_mut()
        .find(|b| b.id == "main")
        .expect("le tableau principal")
        .images
        .push(image("i", 0.0));

    let mut vivant = depart.clone();
    let mut journal = Journal::new(10);
    journal.begin();
    let mut x = 0.0;
    for k in 0..100 {
        let suivant = x + 0.1 * f64::from(k % 7) - 0.03;
        let edit = change(0, x, suivant);
        assert!(edit.apply(&mut vivant));
        journal.record(edit);
        x = suivant;
    }
    assert!(journal.end(), "le geste publie");
    let publies = journal.prendre_les_ecrits();
    assert_eq!(publies.len(), 1);
    assert_eq!(publies[0].edits.len(), 1, "une seule édition");

    let mut rejoue = depart.clone();
    assert!(publies[0].apply(&mut rejoue));
    assert_eq!(rejoue, vivant);
    assert!(journal.undo(&mut vivant));
    assert_eq!(vivant, depart);
}
