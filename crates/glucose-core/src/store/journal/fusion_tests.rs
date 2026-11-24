//! REVENIR-1 : la fusion à trois garde le travail des deux côtés, et ce qui part refait exactement la
//! liste fusionnée quand la salle l'applique à la sienne.

use super::super::Slot;
use super::{fondre_la_liste, fondre_la_valeur, Conflit, FusionDeListe};
use crate::types::BoardImage;

fn n(id: &str, x: f64) -> BoardImage {
    BoardImage::new(id, x, 0.0, 100.0, 100.0)
}

/// Ce que fait la salle des éditions reçues : les appliquer dans l'ordre à sa liste. Chaque « avant »
/// doit être ce qui est là — sinon l'édition viserait autre chose.
fn rejouer(la_bas: &[BoardImage], editions: &[Slot<BoardImage>]) -> Vec<BoardImage> {
    let mut liste = la_bas.to_vec();
    for s in editions {
        match (&s.before, &s.after) {
            (None, Some(neuf)) => liste.insert(s.index, (**neuf).clone()),
            (Some(avant), apres) => {
                assert_eq!(liste[s.index], **avant, "l'avant n'est pas ce qui est là");
                match apres {
                    Some(neuf) => liste[s.index] = (**neuf).clone(),
                    None => {
                        liste.remove(s.index);
                    }
                }
            }
            (None, None) => panic!("une édition vide"),
        }
    }
    liste
}

/// Fusionne, et vérifie que la salle, en appliquant ce qui part, obtient la même liste.
fn fondre(a: &[BoardImage], l: &[BoardImage], r: &[BoardImage]) -> FusionDeListe<BoardImage> {
    let f = fondre_la_liste(a, l, r);
    assert_eq!(
        rejouer(r, &f.a_envoyer),
        f.liste,
        "la salle ne retrouve pas la fusion"
    );
    f
}

#[test]
fn test_rien_n_a_change() {
    let a = [n("1", 0.0), n("2", 0.0)];
    let f = fondre(&a, &a, &a);
    assert_eq!(f.liste, a);
    assert!(f.a_envoyer.is_empty() && f.conflits.is_empty());
}

#[test]
fn test_change_la_bas_seulement_on_prend_la_salle() {
    let a = [n("1", 0.0), n("2", 0.0)];
    let r = [n("1", 5.0), n("3", 0.0)];
    let f = fondre(&a, &a, &r);
    assert_eq!(f.liste, r);
    assert!(f.a_envoyer.is_empty() && f.conflits.is_empty());
}

#[test]
fn test_change_ici_seulement_part_dans_la_salle() {
    let a = [n("1", 0.0), n("2", 0.0)];
    let l = [n("1", 7.0), n("2", 0.0)];
    let f = fondre(&a, &l, &a);
    assert_eq!(f.liste, l);
    assert_eq!(
        f.a_envoyer,
        vec![Slot::changed(0, n("1", 0.0), n("1", 7.0))]
    );
}

#[test]
fn test_le_meme_changement_des_deux_cotes_se_fond_sans_conflit() {
    // L'ancêtre trop ancien d'un arrêt net : ici et là-bas ont tous deux le même geste.
    let a = [n("1", 0.0)];
    let l = [n("1", 4.0)];
    let f = fondre(&a, &l, &l);
    assert_eq!(f.liste, l);
    assert!(f.a_envoyer.is_empty() && f.conflits.is_empty());
}

#[test]
fn test_change_des_deux_cotes_la_salle_gagne_et_la_version_d_ici_est_rendue() {
    let a = [n("1", 0.0)];
    let f = fondre(&a, &[n("1", 1.0)], &[n("1", 2.0)]);
    assert_eq!(f.liste, [n("1", 2.0)]);
    assert!(f.a_envoyer.is_empty());
    let conflit = Conflit {
        nom: "1".into(),
        ici: Some(n("1", 1.0)),
        la_bas: Some(n("1", 2.0)),
    };
    assert_eq!(f.conflits, vec![conflit]);
}

#[test]
fn test_pose_ici_prend_place_derriere_son_voisin() {
    let a = [n("1", 0.0), n("2", 0.0)];
    let l = [n("1", 0.0), n("neuf", 3.0), n("2", 0.0)];
    let r = [n("0", 0.0), n("1", 0.0), n("2", 0.0)];
    let f = fondre(&a, &l, &r);
    assert_eq!(
        f.liste,
        [n("0", 0.0), n("1", 0.0), n("neuf", 3.0), n("2", 0.0)]
    );
    assert_eq!(f.a_envoyer, vec![Slot::inserted(2, n("neuf", 3.0))]);
}

#[test]
fn test_pose_ici_sans_voisin_va_en_tete() {
    let a = [n("1", 0.0)];
    let l = [n("neuf", 3.0), n("1", 0.0)];
    let f = fondre(&a, &l, &a);
    assert_eq!(f.liste, l);
}

#[test]
fn test_retire_ici_seulement_est_retire_de_la_salle() {
    let a = [n("1", 0.0), n("2", 0.0)];
    let f = fondre(&a, &[n("2", 0.0)], &a);
    assert_eq!(f.liste, [n("2", 0.0)]);
    assert_eq!(f.a_envoyer, vec![Slot::removed(0, n("1", 0.0))]);
}

#[test]
fn test_change_ici_retire_la_bas_n_efface_pas_le_travail_d_ici() {
    let a = [n("1", 0.0)];
    let f = fondre(&a, &[n("1", 9.0)], &[]);
    assert!(f.liste.is_empty(), "la salle gagne");
    assert_eq!(
        f.conflits[0].ici,
        Some(n("1", 9.0)),
        "la version d'ici est rendue"
    );
    assert_eq!(f.conflits[0].la_bas, None);
}

#[test]
fn test_retire_ici_change_la_bas_garde_le_travail_de_loin() {
    let a = [n("1", 0.0)];
    let f = fondre(&a, &[], &[n("1", 6.0)]);
    assert_eq!(f.liste, [n("1", 6.0)]);
    assert!(f.a_envoyer.is_empty());
    assert_eq!((f.conflits[0].ici.clone(), f.conflits.len()), (None, 1));
}

#[test]
fn test_tout_ensemble_la_salle_retrouve_la_fusion() {
    let a = [n("1", 0.0), n("2", 0.0), n("3", 0.0), n("4", 0.0)];
    let l = [
        n("2", 1.0),
        n("x", 0.0),
        n("3", 0.0),
        n("4", 8.0),
        n("y", 0.0),
    ];
    let r = [n("1", 0.0), n("2", 0.0), n("z", 0.0), n("4", 5.0)];
    let f = fondre(&a, &l, &r);
    // 1 retiré ici ; 2 changé ici ; x posé ici ; 3 retiré là-bas ; 4 changé des deux côtés ; y posé
    // ici ; z posé là-bas.
    let attendu = [
        n("2", 1.0),
        n("x", 0.0),
        n("z", 0.0),
        n("4", 5.0),
        n("y", 0.0),
    ];
    assert_eq!(f.liste, attendu);
    let noms: Vec<_> = f.conflits.iter().map(|c| c.nom.as_str()).collect();
    assert_eq!(noms, ["4"]);
}

#[test]
fn test_une_valeur_suit_la_meme_regle() {
    let v = |a: &str, l: &str, r: &str| {
        fondre_la_valeur(&a.to_string(), &l.to_string(), &r.to_string())
    };
    assert_eq!(v("a", "a", "b").valeur, "b");
    let ici = v("a", "c", "a");
    assert_eq!(
        (ici.valeur.as_str(), ici.a_envoyer.unwrap().after),
        ("c", "c".into())
    );
    assert_eq!(v("a", "b", "b").conflit, None);
    let conflit = v("a", "c", "b");
    assert_eq!(
        (conflit.valeur.as_str(), conflit.conflit),
        ("b", Some("c".into()))
    );
}

#[test]
fn test_un_nom_en_double_ici_ne_se_pose_qu_une_fois() {
    let l = [n("neuf", 1.0), n("neuf", 1.0)];
    let f = fondre(&[], &l, &[]);
    assert_eq!(f.liste, [n("neuf", 1.0)]);
}
