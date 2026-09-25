use super::*;
use crate::text_anchors::{create_anchor, resolve_text_sel};

/// Sa carte, et une flèche qui en part, ancrée au **second** « bonjours ».
fn sa_carte() -> (Store, String) {
    let texte = "bonjours\ntest\ntest\nbonjours";
    let mut store = Store::new("ancres");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("carte", 0.0, 0.0, texte));
    let second = texte.rfind("bonjours").expect("le second");
    let ancre = create_anchor(texte, second, second + 8).expect("une ancre");
    let mut fleche = Annotation::arrow("f", 0.0, 0.0, 500.0, 0.0);
    if let Annotation::Arrow {
        source_id,
        source_text_sel,
        ..
    } = &mut fleche
    {
        *source_id = Some("carte".to_string());
        *source_text_sel = Some(TextSelection::Anchors(vec![ancre]));
    }
    store.add_annotation(&board, fleche);
    store.journal.clear();
    (store, board)
}

/// Ce que l'ancre de la flèche désigne dans le texte de la carte.
fn designe(store: &Store, board: &str) -> Option<String> {
    let texte = store.project.annotation(board, "carte")?.own_text()?;
    let Annotation::Arrow {
        source_text_sel, ..
    } = store.project.annotation(board, "f")?
    else {
        return None;
    };
    let plage = *resolve_text_sel(&texte, source_text_sel.as_ref()).first()?;
    Some(format!(
        "{}|{}",
        &texte[..plage.start],
        &texte[plage.start..plage.end]
    ))
}

fn ecrire(store: &mut Store, board: &str, texte: &str) {
    store.begin_live_edit();
    store.ecrire_le_texte(board, "carte", texte);
    store.end_live_edit();
}

/// **Écrire devant le second « bonjours » : l'ancre le désigne encore**, et un seul `Ctrl+Z`
/// rend le texte et l'ancre ensemble.
#[test]
fn test_fleche_4_ecrire_devant_garde_le_second() {
    let (mut store, board) = sa_carte();
    let avant = store.project.clone();
    ecrire(
        &mut store,
        &board,
        "Élève, écoute :\nbonjours\ntest\ntest\nbonjours",
    );
    assert_eq!(
        designe(&store, &board).as_deref(),
        Some("Élève, écoute :\nbonjours\ntest\ntest\n|bonjours")
    );
    assert!(store.undo());
    assert!(store.project == avant, "un seul geste");
}

/// **Réécrire le mot ancré garde l'ancre sur ce qu'on a écrit à sa place** ; taper juste après
/// ne l'allonge pas.
#[test]
fn test_fleche_4_reecrire_le_mot_ancre() {
    let (mut store, board) = sa_carte();
    ecrire(&mut store, &board, "bonjours\ntest\ntest\nbonsoirs");
    assert_eq!(
        designe(&store, &board).as_deref(),
        Some("bonjours\ntest\ntest\n|bonsoirs")
    );
    ecrire(&mut store, &board, "bonjours\ntest\ntest\nbonsoirs !");
    assert_eq!(
        designe(&store, &board).as_deref(),
        Some("bonjours\ntest\ntest\n|bonsoirs")
    );
}

/// **Effacer le passage efface l'ancre** : la flèche garde sa carte, sans passage.
#[test]
fn test_fleche_4_effacer_le_passage() {
    let (mut store, board) = sa_carte();
    ecrire(&mut store, &board, "bonjours\ntest\ntest\n");
    let Some(Annotation::Arrow {
        source_id,
        source_text_sel,
        ..
    }) = store.project.annotation(&board, "f")
    else {
        panic!("la flèche");
    };
    assert_eq!(source_id.as_deref(), Some("carte"));
    assert!(source_text_sel.is_none());
}
