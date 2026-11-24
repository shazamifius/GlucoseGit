//! FONDRE-145 : les pas d'une même saisie, un seul Ctrl+Z (fiche 145).

use super::super::*;
use crate::types::{BoardImage, Project};

fn image(id: &str, x: f64) -> BoardImage {
    BoardImage::new(id, x, 0.0, 10.0, 10.0)
}

/// Un geste court : une édition, fermée sous une étiquette ou sans.
fn pas(j: &mut Journal, p: &mut Project, edit: Edit, etiquette: Option<u64>) {
    j.begin();
    assert!(edit.apply(p), "l'édition doit s'appliquer");
    j.record(edit);
    match etiquette {
        Some(e) => j.end_etiquete(e),
        None => j.end(),
    };
}

fn poser(rang: usize) -> Edit {
    let slot = Slot::inserted(rang, image(&format!("img-{rang}"), 0.0));
    Edit::Image {
        board: "main".into(),
        slot,
    }
}

fn nombre(p: &Project) -> usize {
    p.boards
        .iter()
        .find(|b| b.id == "main")
        .expect("main")
        .images
        .len()
}

#[test]
fn les_pas_d_une_saisie_font_un_seul_geste() {
    let (mut p, mut j) = (Project::new("t"), Journal::new(200));
    for rang in 0..3 {
        pas(&mut j, &mut p, poser(rang), Some(7));
    }
    assert_eq!(j.depth(), 1, "trois pas d'une saisie, un seul geste");
    assert!(j.undo(&mut p));
    assert_eq!(nombre(&p), 0, "un Ctrl+Z défait toute la saisie");
    assert!(j.redo(&mut p));
    assert_eq!(nombre(&p), 3, "et la refait toute");
}

#[test]
fn un_autre_geste_entre_deux_pas_les_separe() {
    let (mut p, mut j) = (Project::new("t"), Journal::new(200));
    pas(&mut j, &mut p, poser(0), None);
    pas(&mut j, &mut p, poser(1), None);
    assert_eq!(j.depth(), 2, "deux gestes sans étiquette ne fondent jamais");
    pas(&mut j, &mut p, poser(2), Some(7));
    pas(&mut j, &mut p, poser(3), None);
    pas(&mut j, &mut p, poser(4), Some(7));
    assert_eq!(
        j.depth(),
        5,
        "un agent entre deux pas : trois gestes, dans l'ordre vrai"
    );
    pas(&mut j, &mut p, poser(5), Some(8));
    assert_eq!(
        j.depth(),
        6,
        "une autre saisie ne fond pas dans la première"
    );
}

#[test]
fn un_geste_refait_ne_fond_plus() {
    let (mut p, mut j) = (Project::new("t"), Journal::new(200));
    pas(&mut j, &mut p, poser(0), Some(7));
    assert!(j.undo(&mut p) && j.redo(&mut p));
    pas(&mut j, &mut p, poser(1), Some(7));
    assert_eq!(
        j.depth(),
        2,
        "défaire puis refaire efface l'étiquette du sommet"
    );
}

#[test]
fn une_saisie_revenue_a_son_depart_ne_laisse_rien() {
    let (mut p, mut j) = (Project::new("t"), Journal::new(200));
    pas(&mut j, &mut p, poser(0), None);
    let aller = Slot::changed(0, image("img-0", 0.0), image("img-0", 5.0));
    let retour = Slot::changed(0, image("img-0", 5.0), image("img-0", 0.0));
    pas(
        &mut j,
        &mut p,
        Edit::Image {
            board: "main".into(),
            slot: aller,
        },
        Some(7),
    );
    pas(
        &mut j,
        &mut p,
        Edit::Image {
            board: "main".into(),
            slot: retour,
        },
        Some(7),
    );
    assert_eq!(
        j.depth(),
        1,
        "seule la pose reste : la saisie ne laisse rien à défaire"
    );
}

#[test]
fn un_pas_vide_ne_coupe_pas_la_saisie() {
    let (mut p, mut j) = (Project::new("t"), Journal::new(200));
    pas(&mut j, &mut p, poser(0), Some(7));
    j.begin();
    j.end_etiquete(7); // rien d'écrit : rien de publié
    pas(&mut j, &mut p, poser(1), Some(7));
    assert_eq!(j.depth(), 1, "un pas vide ne coupe pas la saisie");
}

#[test]
fn l_etiquette_ne_survit_pas_a_un_geste_vide() {
    let (mut p, mut j) = (Project::new("t"), Journal::new(200));
    pas(&mut j, &mut p, poser(0), Some(7));
    j.begin();
    j.end_etiquete(9); // rien d'écrit : l'étiquette ne doit pas rester en attente
    pas(&mut j, &mut p, poser(1), None);
    pas(&mut j, &mut p, poser(2), Some(9));
    assert_eq!(
        j.depth(),
        3,
        "le geste suivant, sans étiquette, ne prend pas celle du vide"
    );
}
