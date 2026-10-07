//! `Ctrl+N` (fiche 51 § 4), de bout en bout : de vrais documents sur le disque.

use crate::persist::disque::tests::{application, dossier, image_suivante, noter};

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
    noter(&mut app, "neuf", "un travail sans nom");
    image_suivante(&mut app);
    assert!(
        app.is_dirty(),
        "un travail sans nom passe pour enregistre : Ctrl+N le quitterait sans demander"
    );
}

/// **Quitter un document nommé se dit** : il est enregistré, et un autre commence. Sans ce
/// mot, son essai du 07/10 a lu le silence comme un travail abandonné.
#[test]
fn test_ctrl_n_dit_que_le_document_nomme_est_enregistre() {
    let d = dossier("nouveau-dit");
    let mut app = application(&d);
    app.save_to(d.join("fusee.glucose"));
    noter(&mut app, "garde", "du travail");
    // L'enregistrement vient de se dire, avec le même nom : sans ce silence, l'épreuve lisait
    // son message et passait même quand `Ctrl+N` se taisait (sabotage du 07/10).
    app.ui.current_toast = None;
    app.modifiers = winit::keyboard::ModifiersState::CONTROL;
    assert!(app.handle_file_shortcut("n"));
    let message = app.ui.toast_message().unwrap_or_default().to_string();
    assert!(
        message.contains("fusee") && message.contains("enregistré"),
        "Ctrl+N se tait sur le document qu'il quitte : {message:?}"
    );
}
