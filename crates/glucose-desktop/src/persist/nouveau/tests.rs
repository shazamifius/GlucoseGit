//! `Ctrl+N` (fiche 51 § 4), de bout en bout : de vrais documents sur le disque.

use crate::interactions::question::tests::repondre;
use crate::persist::disque::tests::{application, dossier, image_suivante, noter};
use crate::ui::question::Reponse;

/// **Un document nommé se quitte intact, et le nouveau est vierge** : rien à défaire, aucun
/// fichier, aucun nœud — et le premier geste fait naître un brouillon, jamais une écriture dans
/// l'ancien fichier.
#[test]
fn test_ctrl_n_laisse_le_document_intact_et_en_ouvre_un_vierge() {
    let d = dossier("nouveau-document");
    let chemin = d.join("ancien.glucose");
    let mut app = application(&d);
    app.save_to(chemin.clone());
    noter(&mut app, "garde", "ce texte reste dans l'ancien document");
    image_suivante(&mut app);

    app.modifiers = winit::keyboard::ModifiersState::CONTROL;
    assert!(app.handle_file_shortcut("n"), "Ctrl+N est consommé");
    app.modifiers = winit::keyboard::ModifiersState::empty();
    repondre(&mut app, Reponse::Oui);
    let board = app.store.active_board().expect("un tableau");
    assert!(
        board.annotations.is_empty() && board.images.is_empty(),
        "vierge"
    );
    assert!(app.project_path.is_none(), "sans fichier");
    assert!(!app.store.can_undo(), "rien à défaire");
    assert!(!app.is_dirty(), "rien à enregistrer");

    let taille = std::fs::metadata(&chemin).expect("l'ancien").len();
    noter(&mut app, "neuf", "un premier geste");
    image_suivante(&mut app);
    assert_eq!(
        std::fs::metadata(&chemin).expect("l'ancien").len(),
        taille,
        "l'ancien document ne reçoit rien"
    );
    let ecriture = app.disque.ecriture.as_ref().expect("un brouillon est né");
    assert!(ecriture.brouillon, "c'est un brouillon");

    let relu = crate::persist::read_project_file(&chemin).expect("relu");
    assert!(
        relu.project.boards[0]
            .annotations
            .iter()
            .any(|a| a.own_text().as_deref() == Some("ce texte reste dans l'ancien document")),
        "l'ancien document garde son travail"
    );
}

/// **Du travail sans nom pose la question** : après `Ctrl+N`, un geste dans le document
/// vierge en fait un travail à enregistrer, et un second `Ctrl+N` doit demander (BROUILLON-1).
#[test]
fn test_ctrl_n_sur_un_travail_sans_nom_le_dit_a_enregistrer() {
    let d = dossier("nouveau-sans-nom");
    let mut app = application(&d);
    app.save_to(d.join("nomme.glucose"));
    app.modifiers = winit::keyboard::ModifiersState::CONTROL;
    assert!(app.handle_file_shortcut("n"));
    app.modifiers = winit::keyboard::ModifiersState::empty();
    repondre(&mut app, Reponse::Oui);
    noter(&mut app, "neuf", "un travail sans nom");
    image_suivante(&mut app);
    assert!(
        app.is_dirty(),
        "un travail sans nom passe pour enregistre : Ctrl+N le quitterait sans demander"
    );
}

/// **NOUVEAU-1 — `Ctrl+N` demande, et « Non » ne change rien** : le document reste celui qu'on
/// regarde, avec son travail et son fichier. Son essai du 07/10 : un raccourci tapé par erreur
/// ne doit jamais faire croire au travail perdu.
#[test]
fn test_nouveau_1_ctrl_n_demande_et_non_ne_change_rien() {
    let d = dossier("nouveau-demande");
    let chemin = d.join("fusee.glucose");
    let mut app = application(&d);
    app.save_to(chemin.clone());
    noter(&mut app, "garde", "du travail");
    app.modifiers = winit::keyboard::ModifiersState::CONTROL;
    assert!(app.handle_file_shortcut("n"), "Ctrl+N est consommé");
    app.modifiers = winit::keyboard::ModifiersState::empty();
    repondre(&mut app, Reponse::Annuler);
    assert_eq!(
        app.project_path.as_ref(),
        Some(&chemin),
        "le document reste le meme"
    );
    let board = app.store.active_board().expect("un tableau");
    assert!(
        board
            .annotations
            .iter()
            .any(|a| a.own_text().as_deref() == Some("du travail")),
        "son travail est toujours la"
    );
}
