//! La table des commandes, la couche de Glucose et la palette (P4, fiche 72) — par la vraie
//! application et ses touches.

use super::palette::{rang_de, trouvees};
use super::*;
use crate::app::GlucoseApp;
use glucose_core::raccourcis::Accord;
use glucose_core::types::BoardImage;
use winit::event::ElementState;
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

/// **Chaque commande a un identifiant à elle, qui la retrouve**, et un nom.
#[test]
fn test_p4_chaque_commande_se_retrouve_par_son_identifiant() {
    let toutes = Commande::toutes();
    let mut ids: Vec<String> = toutes.iter().map(|c| c.id()).collect();
    for c in &toutes {
        assert_eq!(Commande::de_l_id(&c.id()), Some(*c), "{c:?}");
        assert!(!c.nom().is_empty(), "{c:?} sans nom");
        assert!(!c.id().contains(char::is_whitespace), "{c:?}");
    }
    let n = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), n, "deux commandes portent le même identifiant");
}

/// **La couche de Glucose se lit sans une ligne fausse, et ne nomme que des commandes qui
/// existent** : un identifiant mal tapé serait un raccourci mort, en silence.
#[test]
fn test_p4_la_couche_de_glucose_est_saine() {
    let (clavier, erreurs) = clavier_de_glucose();
    assert!(erreurs.is_empty(), "{erreurs:?}");
    for id in clavier.couches()[0].commandes() {
        assert!(
            Commande::de_l_id(id).is_some(),
            "« {id} » n'est pas une commande"
        );
    }
}

/// Une application dont deux images sont choisies.
fn avec_deux_images() -> GlucoseApp {
    let mut app = GlucoseApp::new();
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

fn touche(app: &mut GlucoseApp, c: &str, mods: ModifiersState) {
    app.modifiers = mods;
    app.handle_shortcut_input(&Key::Character(SmolStr::new(c)), ElementState::Pressed);
}

fn images(app: &GlucoseApp) -> usize {
    app.store.active_board().map_or(0, |b| b.images.len())
}

/// **Un raccourci se change par la table** — le critère de P4 : une couche posée sur celle de
/// Glucose lie `Ctrl+K` à « dupliquer » et retire `Ctrl+D`. `Ctrl+K` duplique, `Ctrl+D` ne
/// fait plus rien, et le reste du clavier n'a pas bougé.
#[test]
fn test_p4_un_raccourci_se_change_par_la_table() {
    let mut app = avec_deux_images();
    let (dessus, erreurs) = Couche::lire("essai", "Ctrl+K dupliquer\nCtrl+D -\n");
    assert!(erreurs.is_empty());
    let (glucose, _) = clavier_de_glucose();
    app.ui.clavier = Clavier::de(vec![glucose.couches()[0].clone(), dessus]);

    touche(&mut app, "d", ModifiersState::CONTROL);
    assert_eq!(images(&app), 2, "Ctrl+D est retiré");
    touche(&mut app, "k", ModifiersState::CONTROL);
    assert_eq!(images(&app), 4, "Ctrl+K duplique");
    touche(&mut app, "z", ModifiersState::CONTROL);
    assert_eq!(images(&app), 2, "Ctrl+Z annule toujours");
    assert_eq!(
        app.ui.clavier.accords_de("dupliquer"),
        [Accord::lire("Ctrl+K").unwrap()],
        "ce que le menu et la palette diront"
    );
}

/// **La palette trouve sans majuscules ni accents**, les meilleures d'abord : le début du nom,
/// puis le début d'un mot, puis le dedans, puis les lettres dans l'ordre.
#[test]
fn test_p4_la_palette_trouve_comme_on_cherche() {
    assert_eq!(
        rang_de("retab", "Rétablir", "x"),
        Some(0),
        "sans accent, dans le nom seul"
    );
    assert_eq!(
        rang_de("RÉTAB", "Rétablir", "x"),
        Some(0),
        "ni majuscule ni accent"
    );
    assert_eq!(
        rang_de("supp", "x", "supprimer"),
        Some(0),
        "l'identifiant compte aussi"
    );
    assert_eq!(
        rang_de("sel", "Cadrer la sélection", "cadrer-la-selection"),
        Some(1)
    );
    assert_eq!(rang_de("adr", "Cadrer la sélection", "x"), Some(2));
    assert_eq!(rang_de("cdr", "Cadrer la sélection", "x"), Some(3));
    assert_eq!(rang_de("zzz", "Cadrer la sélection", "x"), None);
    assert_eq!(trouvees("retab").first(), Some(&Commande::Retablir));
    assert_eq!(
        trouvees("signet.poser.3").first(),
        Some(&Commande::PoserLeSignet(3))
    );
    assert!(
        !trouvees("").contains(&Commande::Palette),
        "elle ne se trouve pas elle-même"
    );
    assert!(trouvees("").len() <= crate::persist::documents::MONTRES);
}

/// **La palette, de bout en bout, au clavier** : `Ctrl+Maj+P` l'ouvre, « dupl » écrit dans son
/// champ — une image peinte entre deux frappes, comme la vraie boucle —, la première réponse
/// est « Dupliquer » avec son raccourci, et Entrée duplique la sélection.
#[test]
fn test_p4_la_palette_de_bout_en_bout() {
    let mut app = avec_deux_images();
    touche(
        &mut app,
        "P",
        ModifiersState::CONTROL | ModifiersState::SHIFT,
    );
    assert!(
        matches!(
            app.ui.question,
            Some((_, crate::ui::question::Suite::Palette(_)))
        ),
        "Ctrl+Maj+P ouvre la palette"
    );
    app.modifiers = ModifiersState::empty();
    for c in ["d", "u", "p", "l"] {
        assert!(app.prendre_la_touche(&Key::Character(SmolStr::new(c)), true, Some(c)));
        app.une_image_sans_fenetre((1280, 720));
    }
    let (question, _) = app.ui.question.clone().expect("la palette");
    assert_eq!(question.choix[0].0, "Dupliquer   Ctrl+D");
    assert!(app.prendre_la_touche(&Key::Named(NamedKey::Enter), true, None));
    assert!(app.ui.question.is_none(), "la réponse est partie");
    assert_eq!(images(&app), 4, "Entrée a dupliqué");
}

/// **L'évidence se déplace, et l'image suivante ne la ramène pas** : la palette ne refait ses
/// réponses que si son champ a changé — sinon la flèche vers le bas ne servirait à rien.
#[test]
fn test_p4_la_palette_garde_l_evidence_d_une_image_a_l_autre() {
    let mut app = avec_deux_images();
    touche(
        &mut app,
        "P",
        ModifiersState::CONTROL | ModifiersState::SHIFT,
    );
    app.modifiers = ModifiersState::empty();
    for c in ["c", "o", "p"] {
        assert!(app.prendre_la_touche(&Key::Character(SmolStr::new(c)), true, Some(c)));
        app.une_image_sans_fenetre((1280, 720));
    }
    let attendue = trouvees("cop")[1];
    assert!(app.prendre_la_touche(&Key::Named(NamedKey::ArrowDown), true, None));
    app.une_image_sans_fenetre((1280, 720));
    let (question, _) = app.ui.question.clone().expect("la palette");
    assert_eq!(question.focus, 1, "l'image n'a pas ramené l'évidence");
    assert!(question.choix[1].0.starts_with(&attendue.nom()));
}

/// **La palette a son entrée au menu du vide** — au doigt, il n'y a pas de `Ctrl+Maj+P` (R1) —,
/// et le menu dit son raccourci, lu dans le clavier ; une couche qui le change, le menu le suit.
#[test]
fn test_p4_la_palette_au_menu_et_son_raccourci_lu_dans_le_clavier() {
    use crate::ui::context_menu::{layout_context_menu, MenuAction, MenuRow};
    let mut app = GlucoseApp::new();
    app.store.clear_selection();
    let raccourci_de_la_palette = |app: &GlucoseApp| {
        let menu = layout_context_menu(
            &app.store,
            (&app.renderer.typography, &app.ui.clavier),
            ((100.0, 100.0), None),
            (1280.0, 720.0),
            1.0,
            false,
        )
        .expect("un menu");
        menu.rows.iter().find_map(|r| match r {
            MenuRow::Item {
                action: MenuAction::Palette,
                shortcut,
                ..
            } => Some(shortcut.clone()),
            _ => None,
        })
    };
    assert_eq!(raccourci_de_la_palette(&app).as_deref(), Some("Ctrl+Maj+P"));
    let (dessus, _) = Couche::lire("essai", "F1 palette\n");
    app.ui.clavier = Clavier::de(vec![dessus, clavier_de_glucose().0.couches()[0].clone()]);
    assert_eq!(
        raccourci_de_la_palette(&app).as_deref(),
        Some("Ctrl+Maj+P"),
        "la couche du dessus est la dernière : celle de Glucose"
    );
    let (dessus, _) = Couche::lire("essai", "F1 palette\nCtrl+Maj+P -\n");
    app.ui.clavier = Clavier::de(vec![clavier_de_glucose().0.couches()[0].clone(), dessus]);
    assert_eq!(raccourci_de_la_palette(&app).as_deref(), Some("F1"));
}
