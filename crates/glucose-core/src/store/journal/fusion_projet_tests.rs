//! REVENIR-1, le document entier : le travail des deux côtés gardé, et la salle qui, en appliquant ce
//! qui part par le vrai chemin du journal (`Edit::apply`), retrouve le document fusionné.

use super::fondre_le_document;
use crate::types::{Board, BoardImage, Project};

fn image(id: &str, x: f64) -> BoardImage {
    BoardImage::new(id, x, 0.0, 100.0, 100.0)
}

fn depart() -> Project {
    let mut p = Project::new("partage");
    p.boards[0].images = vec![image("a", 0.0), image("b", 0.0)];
    p
}

/// Fusionne, et vérifie que la salle retrouve exactement le document fusionné.
fn fondre(a: &Project, l: &Project, r: &Project) -> super::FusionDuDocument {
    let f = fondre_le_document(a, l, r);
    let mut salle = r.clone();
    for e in &f.a_envoyer {
        assert!(e.apply(&mut salle), "une édition ne s'applique pas : {e:?}");
    }
    assert_eq!(salle, f.projet, "la salle ne retrouve pas la fusion");
    f
}

#[test]
fn test_un_noeud_change_ici_un_autre_la_bas_les_deux_restent() {
    let a = depart();
    let mut l = a.clone();
    l.boards[0].images[0].x = 5.0;
    let mut r = a.clone();
    r.boards[0].images[1].x = 9.0;
    let f = fondre(&a, &l, &r);
    let xs: Vec<f64> = f.projet.boards[0].images.iter().map(|i| i.x).collect();
    assert_eq!(xs, [5.0, 9.0]);
    assert!(f.conflits.is_empty());
}

#[test]
fn test_un_tableau_pose_ici_part_entier() {
    let a = depart();
    let mut l = a.clone();
    let mut neuf = Board::new("t2", "Ici");
    neuf.images.push(image("c", 0.0));
    l.boards.push(neuf.clone());
    let f = fondre(&a, &l, &a);
    assert_eq!(f.projet.boards[1], neuf);
}

#[test]
fn test_un_tableau_retire_ici_et_garni_la_bas_reste() {
    let mut a = depart();
    a.boards.push(Board::new("t2", "Deux"));
    let mut l = a.clone();
    l.boards.pop();
    let mut r = a.clone();
    r.boards[1].images.push(image("loin", 0.0));
    let f = fondre(&a, &l, &r);
    assert_eq!(
        f.projet.boards.len(),
        2,
        "le travail de loin n'est pas effacé"
    );
    assert_eq!(f.conflits, ["t2"]);
}

#[test]
fn test_le_nom_du_document_et_d_un_tableau_suivent_la_regle() {
    let a = depart();
    let mut l = a.clone();
    l.name = "renommé ici".into();
    l.boards[0].name = "tableau ici".into();
    let mut r = a.clone();
    r.boards[0].name = "tableau là-bas".into();
    let f = fondre(&a, &l, &r);
    assert_eq!(f.projet.name, "renommé ici");
    assert_eq!(f.projet.boards[0].name, "tableau là-bas");
    assert_eq!(f.conflits, ["le nom d'un tableau"]);
}

#[test]
fn test_sans_ancetre_pour_un_tableau_commun_rien_ne_se_perd() {
    let mut a = depart();
    a.boards.clear();
    let mut l = depart();
    l.boards[0].images.push(image("ici", 0.0));
    let r = depart();
    let f = fondre(&a, &l, &r);
    assert_eq!(f.projet.boards.len(), 1);
    assert_eq!(f.projet.boards[0].images.len(), 3);
}
