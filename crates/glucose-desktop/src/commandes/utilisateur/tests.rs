//! Changer ses raccourcis dans Glucose (P4, fiche 74) — par la vraie application, ses touches et
//! sa palette, dans un dossier à elle : jamais celui que partagent les autres épreuves.

use super::*;
use crate::interactions::question::tests::repondre;
use glucose_core::types::BoardImage;
use std::path::PathBuf;
use winit::event::ElementState;
use winit::keyboard::{ModifiersState, SmolStr};

/// Un dossier d'application à cette épreuve, vidé.
fn dossier(nom: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("glucose-raccourcis-{nom}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("un dossier d'épreuve");
    d
}

/// Une application qui habite ce dossier, deux images choisies.
fn application(dossier: &std::path::Path) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.habiter(dossier);
    let board = app.store.project.active_board_id.clone();
    for k in 0..2 {
        app.store.add_image(
            &board,
            BoardImage::new(format!("i{k}"), 0.0, 0.0, 10.0, 10.0),
        );
    }
    app.store
        .set_selected_image_ids(vec!["i0".to_string(), "i1".to_string()]);
    app
}

fn raccourci(app: &mut GlucoseApp, c: &str, mods: ModifiersState) {
    app.modifiers = mods;
    app.handle_shortcut_input(&Key::Character(SmolStr::new(c)), ElementState::Pressed);
    app.modifiers = ModifiersState::empty();
}

fn images(app: &GlucoseApp) -> usize {
    app.store.active_board().map_or(0, |b| b.images.len())
}

/// **Une touche, comme la fenêtre l'envoie** : telle qu'elle écrit, telle que les raccourcis la
/// lisent (sur un clavier français, la touche marquée 5 écrit « ( »), avec ces modificateurs.
fn toucher(app: &mut GlucoseApp, (logique, raccourci): (Key, Key), mods: ModifiersState) -> bool {
    app.modifiers = mods;
    let texte = match &logique {
        Key::Character(c) => Some(c.to_string()),
        _ => None,
    };
    let prise = app.prendre_avant_tout((&logique, &raccourci), true, texte.as_deref());
    app.modifiers = ModifiersState::empty();
    app.une_image_sans_fenetre((1280, 720));
    prise
}

/// Une touche qui se lit pareil des deux façons.
fn pareille(k: Key) -> (Key, Key) {
    (k.clone(), k)
}

fn lettre(c: &str) -> Key {
    Key::Character(SmolStr::new(c))
}

/// La touche frappée pendant une question, sans modificateur.
fn frapper(app: &mut GlucoseApp, c: &str) {
    toucher(app, pareille(lettre(c)), ModifiersState::empty());
}

/// **Changer un raccourci, de bout en bout** : « Changer un raccourci… », écrire « dupl »,
/// Entrée, frapper `Ctrl+B`. `Ctrl+B` duplique, `Ctrl+D` plus rien ; le message dit ce que
/// `Ctrl+B` faisait ; le fichier s'écrit ; une application relancée le relit.
#[test]
fn test_p4_changer_un_raccourci_de_bout_en_bout() {
    let d = dossier("bout");
    let mut app = application(&d);
    assert!(app.executer(Commande::ChangerUnRaccourci));
    for c in ["d", "u", "p", "l"] {
        frapper(&mut app, c);
    }
    toucher(
        &mut app,
        pareille(Key::Named(NamedKey::Enter)),
        ModifiersState::empty(),
    );
    assert!(
        matches!(
            app.ui.question,
            Some((_, Suite::Raccourci(Commande::Dupliquer)))
        ),
        "la palette doit mener à la question du raccourci de « Dupliquer »"
    );
    // Un modificateur seul attend la touche.
    let ctrl = ModifiersState::CONTROL;
    assert!(toucher(
        &mut app,
        pareille(Key::Named(NamedKey::Control)),
        ctrl
    ));
    assert!(app.ui.question.is_some(), "Ctrl seul n'est pas un accord");
    assert!(toucher(&mut app, pareille(lettre("b")), ctrl));
    assert!(app.ui.question.is_none());
    let message = app.ui.toast_message().unwrap_or_default().to_string();
    assert!(
        message.contains("Dupliquer") && message.contains("Retirer les bordures"),
        "{message}"
    );
    raccourci(&mut app, "d", ModifiersState::CONTROL);
    assert_eq!(images(&app), 2, "Ctrl+D ne duplique plus");
    raccourci(&mut app, "b", ModifiersState::CONTROL);
    assert_eq!(images(&app), 4, "Ctrl+B duplique");
    let ecrit = std::fs::read_to_string(d.join(FICHIER)).expect("le fichier écrit");
    assert!(ecrit.contains("dupliquer"), "{ecrit}");
    let mut relancee = application(&d);
    raccourci(&mut relancee, "b", ModifiersState::CONTROL);
    assert_eq!(images(&relancee), 4, "relancée, Ctrl+B duplique encore");
}

/// **Retirer, puis rétablir** : sans raccourci, `Ctrl+D` ne fait rien ; rétabli, il duplique,
/// et le fichier ne porte plus aucune liaison.
#[test]
fn test_p4_retirer_puis_retablir_un_raccourci() {
    let d = dossier("retirer");
    let mut app = application(&d);
    app.demander_le_raccourci(Commande::Dupliquer);
    repondre(&mut app, RETIRER);
    raccourci(&mut app, "d", ModifiersState::CONTROL);
    assert_eq!(images(&app), 2, "retiré, Ctrl+D ne fait rien");
    app.demander_le_raccourci(Commande::Dupliquer);
    repondre(&mut app, RETABLIR);
    raccourci(&mut app, "d", ModifiersState::CONTROL);
    assert_eq!(images(&app), 4, "rétabli, Ctrl+D duplique");
    let ecrit = std::fs::read_to_string(d.join(FICHIER)).expect("le fichier écrit");
    let liaisons = Couche::lire(COUCHE, &ecrit).0;
    assert!(liaisons.est_vide(), "{ecrit}");
}

/// **Échap renonce** : rien ne change, ni au clavier ni sur le disque.
#[test]
fn test_p4_echap_renonce_au_raccourci() {
    let d = dossier("echap");
    let mut app = application(&d);
    app.demander_le_raccourci(Commande::Dupliquer);
    assert!(toucher(
        &mut app,
        pareille(Key::Named(NamedKey::Escape)),
        ModifiersState::empty()
    ));
    assert!(app.ui.question.is_none());
    raccourci(&mut app, "d", ModifiersState::CONTROL);
    assert_eq!(images(&app), 4);
    assert!(!d.join(FICHIER).exists(), "renoncer n'écrit rien");
}

/// **Un fichier écrit à la main, avec une ligne fausse, n'est jamais écrasé** : ses bonnes
/// lignes agissent, la fausse se dit, et un changement fait dans Glucose vaut pour la session
/// sans toucher au fichier — octet pour octet.
#[test]
fn test_p4_un_fichier_faux_n_est_jamais_ecrase() {
    let d = dossier("faux");
    let ecrit = "# le mien\nCtrl+K  dupliquer\nCtrl+Zorglub  rien\nCtrl+M  inconnue\n";
    std::fs::write(d.join(FICHIER), ecrit).expect("le fichier de l'épreuve");
    let mut app = application(&d);
    let message = app.ui.toast_message().unwrap_or_default().to_string();
    assert!(
        message.contains("ligne 3") && message.contains("« inconnue »"),
        "les deux se disent, en un message : {message}"
    );
    raccourci(&mut app, "k", ModifiersState::CONTROL);
    assert_eq!(images(&app), 4, "sa bonne ligne agit");
    app.demander_le_raccourci(Commande::ToutChoisir);
    toucher(&mut app, pareille(lettre("q")), ModifiersState::CONTROL);
    let message = app.ui.toast_message().unwrap_or_default().to_string();
    assert!(message.contains("pour cette session"), "{message}");
    assert_eq!(
        std::fs::read_to_string(d.join(FICHIER)).expect("le fichier"),
        ecrit,
        "le fichier a été réécrit"
    );
}

/// **« Rétablir tous les raccourcis de Glucose » demande, puis rend tout** — même un fichier
/// protégé, puisqu'on l'a confirmé.
#[test]
fn test_p4_retablir_tous_les_raccourcis() {
    let d = dossier("tous");
    let ecrit = "Ctrl+K dupliquer\nCtrl+D -\nCtrl+Zorglub faux\n";
    std::fs::write(d.join(FICHIER), ecrit).expect("écrit");
    let mut app = application(&d);
    assert!(
        app.disque.raccourcis_proteges.is_some(),
        "l'épreuve doit partir d'un fichier protégé"
    );
    assert!(app.executer(Commande::RetablirLesRaccourcis));
    repondre(&mut app, Reponse::Oui);
    assert_eq!(app.ui.clavier, clavier_de_glucose().0);
    raccourci(&mut app, "d", ModifiersState::CONTROL);
    assert_eq!(images(&app), 4, "Ctrl+D est rendu");
    let ecrit = std::fs::read_to_string(d.join(FICHIER)).expect("le fichier");
    assert!(Couche::lire(COUCHE, &ecrit).0.est_vide(), "{ecrit}");
}

/// **Au clavier français, un chiffre est sa touche** : la touche marquée 5 écrit « ( », et
/// l'accord retenu est `Ctrl+5` — celui que les raccourcis liront (ACCORD-1).
#[test]
fn test_p4_un_chiffre_se_retient_par_sa_touche() {
    let d = dossier("chiffre");
    let mut app = application(&d);
    app.demander_le_raccourci(Commande::Dupliquer);
    toucher(
        &mut app,
        (lettre("("), lettre("5")),
        ModifiersState::CONTROL,
    );
    assert_eq!(
        app.ui.clavier.accords_de("dupliquer"),
        vec![Accord::lire("Ctrl+5").expect("un accord")]
    );
    raccourci(&mut app, "5", ModifiersState::CONTROL);
    assert_eq!(images(&app), 4, "Ctrl+5 duplique");
}
