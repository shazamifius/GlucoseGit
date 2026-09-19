//! Les gestes de la vague 1.B joués au clavier, sur le **vrai** `GlucoseApp` : verrouiller,
//! empiler, pousser d'un cran.
//!
//! Le noyau les tient déjà sans écran (`lock_suite`, `store::order::tests`). Ce qui se vérifie
//! ici est l'autre moitié : que la touche arrive, qu'elle vise le board actif, et qu'aucune
//! ne se fait voler par une famille de raccourcis voisine — `L` par un outil, `Ctrl+[` par
//! l'édition.

use super::*;
use glucose_core::types::{Annotation, ArrowPredicate, BoardImage};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{Key, ModifiersState, NamedKey, SmolStr};

/// Un board de `n` images superposées à l'origine, toutes sélectionnées.
fn app_with(n: usize) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    for k in 0..n {
        app.store.add_image(
            &board,
            BoardImage::new(format!("i{k}"), 0.0, 0.0, 100.0, 100.0),
        );
    }
    app.store
        .set_selected_image_ids((0..n).map(|k| format!("i{k}")).collect());
    app.store.journal.clear();
    app
}

fn touche(app: &mut GlucoseApp, c: &str, mods: ModifiersState) {
    app.modifiers = mods;
    app.handle_shortcut_input(&Key::Character(SmolStr::new(c)), ElementState::Pressed);
}

fn fleche(app: &mut GlucoseApp, key: NamedKey, mods: ModifiersState) {
    app.modifiers = mods;
    app.handle_shortcut_input(&Key::Named(key), ElementState::Pressed);
}

fn image(app: &GlucoseApp, id: &str) -> BoardImage {
    app.store
        .active_board()
        .and_then(|b| b.images.iter().find(|i| i.id == id).cloned())
        .expect("l'image existe")
}

fn ordre(app: &GlucoseApp) -> Vec<String> {
    app.store
        .active_board()
        .map(|b| b.images.iter().map(|i| i.id.clone()).collect())
        .unwrap_or_default()
}

#[test]
fn test_l_verrouille_et_deverrouille_la_selection() {
    let mut app = app_with(2);
    touche(&mut app, "l", ModifiersState::empty());
    assert!(image(&app, "i0").locked && image(&app, "i1").locked);
    assert_eq!(app.ui.toast_message(), Some("Images verrouillées"));

    touche(&mut app, "l", ModifiersState::empty());
    assert!(!image(&app, "i0").locked);
    assert_eq!(app.ui.toast_message(), Some("Images déverrouillées"));
}

/// `L` est une lettre nue : tenue par un modificateur, elle ne doit pas verrouiller.
#[test]
fn test_ctrl_l_ne_verrouille_rien() {
    let mut app = app_with(1);
    touche(&mut app, "l", ModifiersState::CONTROL);
    assert!(!image(&app, "i0").locked);
}

#[test]
fn test_ctrl_crochets_portent_la_selection_au_premier_et_au_dernier_plan() {
    let mut app = app_with(3);
    app.store.set_selected_image_ids(vec!["i0".into()]);

    touche(&mut app, "]", ModifiersState::CONTROL);
    assert_eq!(ordre(&app), ["i1", "i2", "i0"]);

    touche(&mut app, "[", ModifiersState::CONTROL);
    assert_eq!(ordre(&app), ["i0", "i1", "i2"]);
}

/// Sans `Ctrl`, les crochets ne sont pas un geste : ils ne doivent pas non plus tomber dans
/// le sélecteur d'outils et changer l'outil actif en silence.
#[test]
fn test_les_crochets_nus_ne_font_rien() {
    let mut app = app_with(3);
    app.store.set_selected_image_ids(vec!["i0".into()]);
    let outil = app.ui.active_tool;

    touche(&mut app, "]", ModifiersState::empty());
    assert_eq!(ordre(&app), ["i0", "i1", "i2"]);
    assert_eq!(app.ui.active_tool, outil);
}

#[test]
fn test_les_fleches_poussent_la_selection_dun_cran() {
    let mut app = app_with(1);
    fleche(&mut app, NamedKey::ArrowRight, ModifiersState::empty());
    assert_eq!((image(&app, "i0").x, image(&app, "i0").y), (1.0, 0.0));

    fleche(&mut app, NamedKey::ArrowDown, ModifiersState::empty());
    assert_eq!((image(&app, "i0").x, image(&app, "i0").y), (1.0, 1.0));

    fleche(&mut app, NamedKey::ArrowLeft, ModifiersState::empty());
    fleche(&mut app, NamedKey::ArrowUp, ModifiersState::empty());
    assert_eq!(
        (image(&app, "i0").x, image(&app, "i0").y),
        (0.0, 0.0),
        "les quatre directions se compensent"
    );
}

#[test]
fn test_maj_fleche_pousse_de_dix_crans() {
    let mut app = app_with(1);
    fleche(&mut app, NamedKey::ArrowRight, ModifiersState::SHIFT);
    assert_eq!(image(&app, "i0").x, NUDGE_DECADE);
}

/// Ce que le verrou promet, par le chemin du clavier : une image fermée ne bouge plus, même
/// si on insiste.
#[test]
fn test_une_image_verrouillee_ignore_les_fleches() {
    let mut app = app_with(1);
    touche(&mut app, "l", ModifiersState::empty());
    fleche(&mut app, NamedKey::ArrowRight, ModifiersState::empty());
    fleche(&mut app, NamedKey::ArrowRight, ModifiersState::SHIFT);
    assert_eq!(image(&app, "i0").x, 0.0);
}

// ── Le menu contextuel, par le chemin de la souris ────────────────────────────

use crate::ui::context_menu::MenuAction;
use winit::dpi::PhysicalPosition;

fn clic_droit(app: &mut GlucoseApp, de: (f64, f64), a: (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(de.0, de.1));
    app.handle_mouse_down(MouseButton::Right, 1440.0, 900.0);
    if a != de {
        app.handle_cursor_moved(PhysicalPosition::new(a.0, a.1));
    }
    app.handle_mouse_up(MouseButton::Right);
}

/// Le geste se décide au relâchement : sur place, c'est un menu ; en glissant, c'était un pan.
#[test]
fn test_un_clic_droit_sur_place_ouvre_le_menu_et_un_glisser_non() {
    let mut app = app_with(1);
    clic_droit(&mut app, (400.0, 300.0), (400.0, 300.0));
    assert_eq!(app.ui.context_menu_at, Some((400.0, 300.0)));

    app.ui.context_menu_at = None;
    clic_droit(&mut app, (400.0, 300.0), (600.0, 380.0));
    assert_eq!(
        app.ui.context_menu_at, None,
        "un clic droit qui déplace la vue n'ouvre rien"
    );
}

#[test]
fn test_echap_referme_le_menu() {
    let mut app = app_with(1);
    clic_droit(&mut app, (400.0, 300.0), (400.0, 300.0));
    app.modifiers = ModifiersState::empty();
    app.handle_shortcut_input(&Key::Named(NamedKey::Escape), ElementState::Pressed);
    assert_eq!(app.ui.context_menu_at, None);
}

/// Un clic gauche referme le menu — sur une entrée comme à côté.
#[test]
fn test_un_clic_gauche_referme_le_menu() {
    let mut app = app_with(1);
    clic_droit(&mut app, (400.0, 300.0), (400.0, 300.0));
    app.handle_cursor_moved(PhysicalPosition::new(900.0, 700.0));
    app.handle_mouse_down(MouseButton::Left, 1440.0, 900.0);
    app.handle_mouse_up(MouseButton::Left);
    assert_eq!(app.ui.context_menu_at, None);
}

/// L'entrée « Au premier plan » du menu fait la même chose que `Ctrl+]`.
#[test]
fn test_une_entree_du_menu_agit_vraiment() {
    let mut app = app_with(3);
    app.store.set_selected_image_ids(vec!["i0".into()]);
    clic_droit(&mut app, (400.0, 300.0), (400.0, 300.0));

    let menu = crate::ui::context_menu::layout_context_menu(
        &app.store,
        &app.renderer.typography,
        app.ui.context_menu_at.expect("un menu ouvert"),
        (1440.0, 900.0),
        app.ui.scale_factor,
    )
    .expect("un menu");
    let cible = menu
        .rows
        .iter()
        .find_map(|r| match r {
            crate::ui::context_menu::MenuRow::Item {
                action: MenuAction::ToFront,
                rect,
                ..
            } => Some(*rect),
            _ => None,
        })
        .expect("l'entrée « Au premier plan »");

    let centre = (
        (cible.0 + cible.2 / 2.0) as f64,
        (cible.1 + cible.3 / 2.0) as f64,
    );
    app.handle_cursor_moved(PhysicalPosition::new(centre.0, centre.1));
    app.handle_mouse_down(MouseButton::Left, 1440.0, 900.0);
    app.handle_mouse_up(MouseButton::Left);

    assert_eq!(ordre(&app), ["i1", "i2", "i0"]);
    assert_eq!(app.ui.context_menu_at, None, "et le menu se referme");
}

// ── PRED-1 : les chiffres posent un prédicat sémantique ───────────────────

/// Un tableau d'une flèche et d'une carte, la flèche sélectionnée.
fn app_avec_fleche() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
    }
    app.store
        .add_annotation(&board, Annotation::arrow("a", 0.0, 0.0, 200.0, 0.0));
    app.store
        .add_annotation(&board, Annotation::text("t", 300.0, 0.0, "une carte"));
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    app.store.journal.clear();
    app
}

fn tape(app: &mut GlucoseApp, key: &str) {
    app.handle_shortcut_input(&Key::Character(SmolStr::new(key)), ElementState::Pressed);
}

fn predicat(app: &GlucoseApp, id: &str) -> Option<ArrowPredicate> {
    app.store
        .active_board()?
        .annotations
        .iter()
        .find(|a| a.id() == id)
        .and_then(|a| match a {
            Annotation::Arrow { predicate, .. } => *predicate,
            _ => None,
        })
}

/// Les six chiffres posent les six prédicats, dans l'ordre du modèle.
#[test]
fn test_pred_1_each_digit_sets_its_predicate() {
    let mut app = app_avec_fleche();
    for (rang, attendu) in ArrowPredicate::ALL.iter().enumerate() {
        tape(&mut app, &(rang + 1).to_string());
        assert_eq!(
            predicat(&app, "a"),
            Some(*attendu),
            "la touche {} devrait poser {}",
            rang + 1,
            attendu.as_str()
        );
    }
}

/// `0` retire le prédicat : la relation redevient un simple lien.
#[test]
fn test_pred_1_zero_clears_the_predicate() {
    let mut app = app_avec_fleche();
    tape(&mut app, "3");
    assert!(predicat(&app, "a").is_some());
    tape(&mut app, "0");
    assert_eq!(predicat(&app, "a"), None);
}

/// Le geste porte sur **toute** la sélection : c'est ce qui permet d'annoter un graphe.
#[test]
fn test_pred_1_the_whole_selection_is_qualified_at_once() {
    let mut app = app_avec_fleche();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::arrow("b", 0.0, 90.0, 200.0, 90.0));
    app.store
        .set_selected_annotation_ids(vec!["a".into(), "b".into()]);

    tape(&mut app, "2");
    assert_eq!(predicat(&app, "a"), Some(ArrowPredicate::Contredit));
    assert_eq!(predicat(&app, "b"), Some(ArrowPredicate::Contredit));
}

/// Une sélection mêlée sert à ce qu'elle peut servir : les flèches sont qualifiées, et ce
/// qui n'en est pas est laissé intact.
#[test]
fn test_pred_1_a_mixed_selection_qualifies_only_its_arrows() {
    let mut app = app_avec_fleche();
    app.store
        .set_selected_annotation_ids(vec!["a".into(), "t".into()]);
    let carte_avant = app
        .store
        .active_board()
        .and_then(|b| b.annotations.iter().find(|a| a.id() == "t").cloned());
    tape(&mut app, "4");

    assert_eq!(predicat(&app, "a"), Some(ArrowPredicate::Inspire));
    let carte_apres = app
        .store
        .active_board()
        .and_then(|b| b.annotations.iter().find(|a| a.id() == "t").cloned());
    assert_eq!(carte_avant, carte_apres, "la carte n'a pas été touchée");
}

/// Sans flèche dans la sélection, le prédicat **rend la main** — il n'avale pas la touche.
///
/// Un raccourci qui avale une touche pour ne rien faire est pire qu'un raccourci absent : il
/// empêche la suivante de servir, et il le fait en silence.
///
/// Ce que ce test protégeait a fini par servir : les signets de vue ont pris les chiffres
/// libres sans qu'une ligne du prédicat ne change. Il vérifie donc désormais les deux
/// moitiés — le prédicat n'écrit rien, et **quelqu'un d'autre a pu répondre**.
#[test]
fn test_pred_1_a_digit_without_an_arrow_is_not_swallowed() {
    let mut app = app_avec_fleche();
    app.store.set_selected_annotation_ids(vec!["t".into()]);
    let version = app.store.version;
    tape(&mut app, "5");
    assert_eq!(app.store.version, version, "le predicat n'a rien écrit");
    assert!(
        app.ui
            .toast_message()
            .is_some_and(|m| m.contains("Signet 5")),
        "la touche est restée disponible pour la suite : {:?}",
        app.ui.toast_message()
    );
}

/// Tout le geste tient dans une entrée d'annulation, même sur plusieurs flèches.
#[test]
fn test_pred_1_qualifying_a_selection_undoes_in_one_step() {
    let mut app = app_avec_fleche();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::arrow("b", 0.0, 90.0, 200.0, 90.0));
    app.store
        .set_selected_annotation_ids(vec!["a".into(), "b".into()]);
    app.store.journal.clear();

    tape(&mut app, "6");
    app.store.undo();
    assert_eq!(predicat(&app, "a"), None);
    assert_eq!(predicat(&app, "b"), None);
}

/// `Ctrl`+un chiffre n'est pas un prédicat : les modificateurs restent libres.
#[test]
fn test_pred_1_a_modified_digit_is_left_alone() {
    let mut app = app_avec_fleche();
    app.modifiers = ModifiersState::CONTROL;
    tape(&mut app, "1");
    assert_eq!(predicat(&app, "a"), None);
}

// ── KEY-2 : une touche maintenue ne crée pas cinquante objets ───────────────

fn lettre(c: &str) -> Key {
    Key::Character(SmolStr::new(c))
}

/// **Les actions qui créent quelque chose ne se répètent pas.**
///
/// Mesuré avant ce filtre : maintenir `Ctrl+V` collait cinquante images par seconde, chacune
/// écrivant un PNG sur le disque. Trois cent cinquante-huit images pour un seul geste, six
/// cent cinquante mégaoctets décodés, une frame à 1,7 seconde — et le symptôme observé était
/// « l'application lague à l'import », à mille lieues de la cause.
#[test]
fn test_les_actions_ponctuelles_ne_se_repetent_pas() {
    for touche in ["v", "V", "s", "S", "o", "O", "i", "I", "d", "D", "a", "A"] {
        assert!(
            !super::repetition_utile(&lettre(touche)),
            "« {touche} » a un effet ponctuel : sa répétition automatique ne doit pas passer"
        );
    }
    assert!(!super::repetition_utile(&Key::Named(NamedKey::Delete)));
    assert!(!super::repetition_utile(&Key::Named(NamedKey::Space)));
}

/// Les flèches, elles, doivent se répéter : chaque répétition avance d'un pas de plus, et
/// c'est le geste même du déplacement fin au clavier.
#[test]
fn test_les_fleches_se_repetent() {
    for touche in [
        NamedKey::ArrowLeft,
        NamedKey::ArrowRight,
        NamedKey::ArrowUp,
        NamedKey::ArrowDown,
    ] {
        assert!(super::repetition_utile(&Key::Named(touche)));
    }
}

/// L'annulation et le rétablissement se répètent : remonter dix crans d'un coup est une
/// intention courante, et rien n'est créé.
#[test]
fn test_annuler_et_retablir_se_repetent() {
    for touche in ["z", "Z", "y", "Y"] {
        assert!(super::repetition_utile(&lettre(touche)));
    }
}

// ── Les signets de vue ──────────────────────────────────────────────────────

/// **Poser puis revenir.** Le geste entier, sans fenêtre : `Ctrl+3` retient le cadrage, on
/// s'en va, `3` déclenche le vol de retour vers exactement celui-là.
#[test]
fn un_signet_retient_la_vue_et_le_chiffre_y_ramene() {
    let mut app = GlucoseApp::new();
    let tableau = app.store.project.active_board_id.clone();
    let depart = glucose_core::types::Viewport {
        x: -820.0,
        y: 340.0,
        scale: 2.5,
    };
    app.store.set_viewport(&tableau, depart);

    touche(&mut app, "3", ModifiersState::CONTROL);
    assert_eq!(
        app.store.bookmark(&tableau, "3"),
        Some(depart),
        "Ctrl+3 retient le cadrage courant"
    );

    // On s'en va ailleurs, puis on rappelle le signet.
    app.store.set_viewport(
        &tableau,
        glucose_core::types::Viewport {
            x: 9000.0,
            y: -9000.0,
            scale: 0.3,
        },
    );
    touche(&mut app, "3", ModifiersState::empty());
    assert!(app.vol.en_cours(), "le chiffre declenche un vol de retour");

    // Le vol y arrive : c'est la garantie de `vol`, on vérifie ici qu'il vise le bon endroit.
    let ecran = glucose_core::membrane_focus::ScreenSize {
        width: 1920.0,
        height: 1080.0,
    };
    for _ in 0..600 {
        let vue = app.store.viewport();
        match app.vol.avancer(vue, ecran, 1.0 / 60.0) {
            Some(suivante) => app.store.set_viewport(&tableau, suivante),
            None => break,
        }
    }
    assert_eq!(
        app.store.viewport(),
        depart,
        "on revient exactement au signet"
    );
}

/// **Un signet vide ne téléporte pas et ne se tait pas non plus** : rien ne se voit, donc il
/// faut le dire — et dire comment le poser.
#[test]
fn un_signet_vide_le_dit_au_lieu_de_ne_rien_faire() {
    let mut app = GlucoseApp::new();
    let avant = app.store.viewport();

    touche(&mut app, "7", ModifiersState::empty());
    assert!(!app.vol.en_cours(), "rien a rejoindre");
    assert_eq!(app.store.viewport(), avant, "et la vue n'a pas bouge");
    assert!(
        app.ui.toast_message().is_some_and(|m| m.contains('7')),
        "le message nomme le signet : {:?}",
        app.ui.toast_message()
    );
}

/// **Poser un signet ne modifie pas le document.** C'est un état de vue, comme le cadrage :
/// `Ctrl+Z` n'a rien à défaire, et le titre ne doit pas se marquer « modifié ».
#[test]
fn poser_un_signet_ne_salit_pas_le_document() {
    let mut app = GlucoseApp::new();
    let version = app.store.version;
    touche(&mut app, "1", ModifiersState::CONTROL);
    assert_eq!(app.store.version, version);
    assert!(!app.is_dirty(), "le document n'a pas change");
}

/// **La flèche l'emporte sur le signet, et seulement quand elle est là.** Les chiffres
/// qualifient les flèches sélectionnées (PRED-1) ; sans sélection, ils transportent. L'ordre
/// d'appel suffit à départager, et ce test le verrouille dans les deux sens.
#[test]
fn un_chiffre_qualifie_la_fleche_en_main_et_transporte_les_mains_vides() {
    let mut app = GlucoseApp::new();
    let tableau = app.store.project.active_board_id.clone();
    app.store
        .set_bookmark(&tableau, "2", glucose_core::types::Viewport::default());

    // Une flèche en main : le chiffre la qualifie, et aucun vol ne part.
    let fleche = glucose_core::types::Annotation::arrow("fl1", 0.0, 0.0, 100.0, 100.0);
    app.store.add_annotation(&tableau, fleche);
    app.store
        .set_selected_annotation_ids(vec!["fl1".to_string()]);
    touche(&mut app, "2", ModifiersState::empty());
    assert!(!app.vol.en_cours(), "la fleche a pris la touche");

    // Les mains vides : le même chiffre transporte.
    app.store.clear_selection();
    touche(&mut app, "2", ModifiersState::empty());
    assert!(
        app.vol.en_cours(),
        "sans selection, le chiffre est un signet"
    );
}

/// **Un clavier AZERTY doit déclencher les raccourcis numériques.** C'est le défaut qui a
/// rendu les signets de vue — et les prédicats de flèche avant eux — inatteignables pour
/// leur auteur : la touche marquée « 3 » y produit `"`, jamais `3`.
///
/// Le test porte sur la touche **physique**, parce que c'est la seule chose qu'un clavier
/// français, allemand ou russe ait en commun avec un clavier américain.
#[test]
fn les_chiffres_se_lisent_sur_la_touche_physique_pas_sur_la_disposition() {
    use winit::keyboard::{KeyCode, PhysicalKey};

    assert_eq!(
        chiffre_de_la_touche(PhysicalKey::Code(KeyCode::Digit3)),
        Some('3'),
        "la touche marquee 3 vaut 3, meme quand elle ecrit un guillemet"
    );
    assert_eq!(
        chiffre_de_la_touche(PhysicalKey::Code(KeyCode::Numpad7)),
        Some('7'),
        "le pave numerique repond pareil"
    );
    assert_eq!(chiffre_de_la_touche(PhysicalKey::Code(KeyCode::KeyA)), None);
    assert_eq!(
        chiffre_de_la_touche(PhysicalKey::Code(KeyCode::Quote)),
        None,
        "la touche qui PORTE le guillemet, elle, n'est pas un chiffre"
    );

    // Les dix chiffres, et chacun le sien : une permutation ratée se verrait ici.
    for (rang, code) in [
        KeyCode::Digit0,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ]
    .into_iter()
    .enumerate()
    {
        let attendu = char::from_digit(rang as u32, 10);
        assert_eq!(chiffre_de_la_touche(PhysicalKey::Code(code)), attendu);
    }
}
