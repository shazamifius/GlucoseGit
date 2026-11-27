use super::super::ECART;
use super::{Maison, Place};
use crate::geometry::Rect;
use crate::store::Store;
use crate::types::Annotation;

/// Un tableau neuf, ces nœuds posés dans l'ordre.
fn un_tableau(noeuds: Vec<Annotation>) -> (Store, String) {
    let mut store = Store::new("place");
    let board = store.project.active_board_id.clone();
    for n in noeuds {
        store.add_annotation(&board, n);
    }
    (store, board)
}

fn boite(store: &Store, board: &str, id: &str) -> Rect {
    store.boite_d_un_noeud(board, id).expect("le nœud")
}

fn place(store: &Store, board: &str, voulue: Rect, maison: Maison<'_>) -> Place {
    store
        .place_libre(board, voulue, maison, &["neuf"])
        .expect("une place")
}

/// **Libre, la place demandée reste la sienne.**
#[test]
fn place_libre_la_place_demandee_reste() {
    let (store, board) = un_tableau(vec![Annotation::text("v", 0.0, 0.0, "un voisin")]);
    let v = boite(&store, &board, "v");
    let voulue = Rect::new(v.right() + 200.0, v.top, 240.0, 100.0);
    let p = place(&store, &board, voulue, Maison::OuIlTombe);
    assert_eq!((p.x, p.y, p.libre), (voulue.left, voulue.top, true));
}

/// **Sur un voisin, elle glisse au plus près** : il dépasse de 10 à droite, elle passe juste
/// derrière lui plutôt que devant, dessous ou dessus — et ne le chevauche plus.
#[test]
fn place_libre_sur_un_voisin_glisse_au_plus_pres() {
    let (store, board) = un_tableau(vec![Annotation::text("v", 0.0, 0.0, "un voisin")]);
    let v = boite(&store, &board, "v");
    let voulue = Rect::new(v.right() - 10.0, v.top, 240.0, 100.0);
    let p = place(&store, &board, voulue, Maison::OuIlTombe);
    assert_eq!((p.x, p.y, p.libre), (v.right() + ECART, v.top, true));
}

/// **À cheval sur une membrane, il en sort** ; tout entier dedans, il y reste : elle devient la
/// sienne, comme sous la main qui lâche.
#[test]
fn place_libre_a_cheval_sur_une_membrane_il_en_sort() {
    let m = Annotation::membrane("m", 0.0, 0.0, 1000.0, 1000.0);
    let (store, board) = un_tableau(vec![m]);
    let a_cheval = Rect::new(950.0, 100.0, 240.0, 100.0);
    let p = place(&store, &board, a_cheval, Maison::OuIlTombe);
    assert_eq!((p.x, p.y, p.libre), (1000.0 + ECART, 100.0, true));
    let dedans = Rect::new(500.0, 100.0, 240.0, 100.0);
    let p = place(&store, &board, dedans, Maison::OuIlTombe);
    assert_eq!((p.x, p.y, p.libre), (500.0, 100.0, true));
}

/// **Une membrane qui n'est pas la sienne ne le contient pas** : posé « dans » une autre, ou
/// hors de toute membrane, il sort de celle où il tomberait — sinon l'œil le verrait dedans et
/// l'IA ailleurs (l'appartenance contredite du contrôleur).
#[test]
fn place_libre_une_membrane_etrangere_ne_le_contient_pas() {
    let ailleurs = Annotation::membrane("ailleurs", 0.0, 0.0, 1000.0, 1000.0);
    let sienne = Annotation::membrane("sienne", 3000.0, 0.0, 1000.0, 1000.0);
    let (store, board) = un_tableau(vec![ailleurs, sienne]);
    let dedans = Rect::new(500.0, 100.0, 240.0, 100.0);
    for maison in [Maison::Dite("sienne"), Maison::Aucune] {
        let p = place(&store, &board, dedans, maison);
        let r = Rect::new(p.x, p.y, 240.0, 100.0);
        assert!(
            p.libre && !r.overlaps(boite(&store, &board, "ailleurs")),
            "{maison:?} {p:?}"
        );
    }
    // Sa propre membrane ne gêne jamais : elle grandira pour lui.
    let chez_elle = Rect::new(3500.0, 100.0, 240.0, 100.0);
    let p = place(&store, &board, chez_elle, Maison::Dite("sienne"));
    assert_eq!((p.x, p.y, p.libre), (3500.0, 100.0, true));
}

/// **Sa membrane ne grandit pas sur un voisin** : si elle le toucherait, la place n'est pas
/// libre — et, aucune ne l'étant, la plus proche est prise quand même, et le dit.
#[test]
fn place_libre_sa_membrane_ne_grandit_pas_sur_un_voisin() {
    let m = Annotation::membrane("m", 0.0, 0.0, 500.0, 500.0);
    let (store, board) = un_tableau(vec![m, Annotation::text("v", 700.0, 0.0, "un voisin")]);
    // Le cadre grandi va jusqu'au bord droit de la place, plus l'écart.
    let tient = Rect::new(450.0, 100.0, 700.0 - ECART - 450.0 - 1.0, 100.0);
    assert!(place(&store, &board, tient, Maison::Dite("m")).libre);
    let deborde = Rect::new(450.0, 100.0, 700.0 - ECART - 450.0 + 1.0, 100.0);
    let p = place(&store, &board, deborde, Maison::Dite("m"));
    assert_eq!((p.x, p.y, p.libre), (450.0, 100.0, false));
}

/// **Déplacer une membrane : ce qu'elle emporte ne la gêne pas**, ce qui reste, si.
#[test]
fn place_libre_deplacer_une_membrane_avec_son_contenu() {
    let m = Annotation::membrane("m", 0.0, 0.0, 1000.0, 1000.0);
    let (mut store, board) = un_tableau(vec![m, Annotation::text("a", 100.0, 100.0, "dedans")]);
    assert_eq!(
        store
            .project
            .annotation(&board, "a")
            .and_then(Annotation::membrane_id),
        Some("m")
    );
    let p = store
        .geste_d_agent(|s| s.deplacer_pour_un_agent(&board, "m", (50.0, 50.0)))
        .expect("déplacée");
    assert_eq!((p.x, p.y, p.libre), (50.0, 50.0, true));
    store.add_annotation(&board, Annotation::text("v", 1100.0, 100.0, "un voisin"));
    let v = boite(&store, &board, "v");
    let p = store
        .geste_d_agent(|s| s.deplacer_pour_un_agent(&board, "m", (200.0, 50.0)))
        .expect("déplacée");
    // La boîte vraie, pas seulement ce que la réponse dit.
    let r = boite(&store, &board, "m");
    assert_eq!((r.left, r.top), (p.x, p.y));
    assert!(p.libre && !r.overlaps(v), "{r:?} chevauche {v:?}");
}
