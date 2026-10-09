//! Le toucher (fiche 54), joué sur le vrai [`GlucoseApp`] par les mêmes événements que `winit`
//! livre sur un téléphone — un doigt qui se pose, bouge, se lève.

use super::*;
use crate::canvas::screen_to_world;
use glucose_core::types::BoardImage;
use winit::dpi::PhysicalPosition;
use winit::event::DeviceId;

fn doigt(app: &mut GlucoseApp, id: u64, phase: TouchPhase, (x, y): (f64, f64)) {
    app.toucher(&Touch {
        device_id: DeviceId::dummy(),
        phase,
        location: PhysicalPosition::new(x, y),
        force: None,
        id,
    });
}

/// Une application sans rien, vue centrée.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.viewport.x = 400.0;
        b.viewport.y = 300.0;
    }
    app
}

/// Joue des images, comme la boucle le fait — chacune présentée un balayage d'un écran à
/// 240 Hz après la précédente : c'est le temps qui passe qui fait glisser le doigt.
fn images(app: &mut GlucoseApp, n: usize) {
    let periode = std::time::Duration::from_micros(4_167);
    app.horloge.accorder(periode);
    let mut t = app
        .horloge
        .derniere_presentation()
        .unwrap_or_else(std::time::Instant::now);
    for _ in 0..n {
        t += periode;
        app.horloge.presentee(t, true);
        app.bouger_la_camera(1280, 720);
    }
}

/// **Deux doigts décrivent une similitude** : de (100, 100)–(200, 100) à (50, 100)–(250, 100),
/// l'écart double et le milieu (150, 100) reste — `r = 2`, `b = (150, 100) − 2 · (150, 100)`.
/// Deux doigts au même point n'en décrivent aucune.
#[test]
fn test_deux_doigts_decrivent_une_similitude() {
    let m = similitude(
        [(100.0, 100.0), (200.0, 100.0)],
        [(50.0, 100.0), (250.0, 100.0)],
    )
    .expect("une similitude");
    assert_eq!(m.echelle, 2.0);
    assert_eq!(m.decalage, (-150.0, -100.0));
    assert_eq!(
        similitude([(5.0, 5.0), (5.0, 5.0)], [(5.0, 5.0), (9.0, 9.0)]),
        None
    );
}

/// **Un doigt sur le vide déplace le canevas, et glisse** : aucun cadre de sélection, et la vue
/// continue après que le doigt s'est levé — l'élan du doigt, que la souris n'a pas.
#[test]
fn test_un_doigt_sur_le_vide_deplace_le_canevas_et_glisse() {
    let mut app = app();
    let avant = app.store.viewport();
    doigt(&mut app, 1, TouchPhase::Started, (300.0, 200.0));
    for k in 1..=8 {
        doigt(
            &mut app,
            1,
            TouchPhase::Moved,
            (300.0 + 10.0 * k as f64, 200.0),
        );
        images(&mut app, 1);
    }
    assert!(app.selection_box.is_none(), "aucun cadre de sélection");
    doigt(&mut app, 1, TouchPhase::Ended, (380.0, 200.0));
    let au_lever = app.store.viewport().x;
    images(&mut app, 20);
    let apres = app.store.viewport();
    assert!(
        au_lever > avant.x,
        "la vue suit le doigt : {} -> {au_lever}",
        avant.x
    );
    assert!(
        apres.x > au_lever,
        "elle glisse après lui — la souris, elle, s'arrête net : {au_lever} -> {}",
        apres.x
    );
    assert!(!app.au_doigt(), "le doigt levé, plus rien ne le suit");
}

/// **Un doigt sur un nœud le glisse**, comme la souris.
#[test]
fn test_un_doigt_sur_un_noeud_le_glisse() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_image(&board, BoardImage::new("a", 0.0, 0.0, 100.0, 100.0));
    let vp = app.store.viewport();
    let (sx, sy) = crate::canvas::world_to_screen(0.0, 0.0, &vp);
    let profondeur = app.store.undo_depth();
    doigt(&mut app, 1, TouchPhase::Started, (sx, sy));
    doigt(&mut app, 1, TouchPhase::Moved, (sx + 30.0, sy));
    doigt(&mut app, 1, TouchPhase::Moved, (sx + 60.0, sy));
    doigt(&mut app, 1, TouchPhase::Ended, (sx + 60.0, sy));
    let x = app.store.active_board().map(|b| b.images[0].x).expect("a");
    assert!((x - 60.0 / vp.scale).abs() < 1e-6, "le nœud a suivi : {x}");
    assert_eq!(app.store.undo_depth(), profondeur + 1, "un geste");
}

/// **Deux doigts pincent autour de leur milieu** : écartés du double, l'échelle double, et le
/// point du monde sous leur milieu y reste.
#[test]
fn test_deux_doigts_pincent_autour_de_leur_milieu() {
    let mut app = app();
    let avant = app.store.viewport();
    let sous_le_milieu = screen_to_world(500.0, 300.0, &avant);
    doigt(&mut app, 1, TouchPhase::Started, (450.0, 300.0));
    doigt(&mut app, 2, TouchPhase::Started, (550.0, 300.0));
    doigt(&mut app, 2, TouchPhase::Moved, (600.0, 300.0));
    doigt(&mut app, 1, TouchPhase::Moved, (400.0, 300.0));
    images(&mut app, 1);
    let apres = app.store.viewport();
    assert!((apres.scale / avant.scale - 2.0).abs() < 1e-9, "{apres:?}");
    let ici = screen_to_world(500.0, 300.0, &apres);
    assert!(
        (ici.0 - sous_le_milieu.0).abs() < 1e-6 && (ici.1 - sous_le_milieu.1).abs() < 1e-6,
        "le point sous le milieu reste : {ici:?} contre {sous_le_milieu:?}"
    );
    assert!(app.selection_box.is_none());
}

/// **Le deuxième doigt termine le glisser du premier** : le nœud se pose où il est, et les
/// doigts pincent ensuite sans plus le déplacer.
#[test]
fn test_le_deuxieme_doigt_termine_le_glisser_du_premier() {
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_image(&board, BoardImage::new("a", 0.0, 0.0, 100.0, 100.0));
    let vp = app.store.viewport();
    let (sx, sy) = crate::canvas::world_to_screen(0.0, 0.0, &vp);
    let profondeur = app.store.undo_depth();
    doigt(&mut app, 1, TouchPhase::Started, (sx, sy));
    doigt(&mut app, 1, TouchPhase::Moved, (sx + 30.0, sy));
    doigt(&mut app, 2, TouchPhase::Started, (sx + 200.0, sy));
    let pose = app.store.active_board().map(|b| b.images[0].x).expect("a");
    assert!(!app.is_dragging_item, "le glisser est terminé");
    doigt(&mut app, 1, TouchPhase::Moved, (sx - 100.0, sy));
    doigt(&mut app, 2, TouchPhase::Moved, (sx + 300.0, sy));
    let x = app.store.active_board().map(|b| b.images[0].x).expect("a");
    assert_eq!(x, pose, "les doigts ne déplacent plus le nœud");
    assert_eq!(
        app.store.undo_depth(),
        profondeur + 1,
        "le glisser est un geste"
    );
    doigt(&mut app, 1, TouchPhase::Ended, (0.0, 0.0));
    doigt(&mut app, 2, TouchPhase::Ended, (0.0, 0.0));
    assert!(!app.au_doigt());
}

/// **`a` puis `b`, composés, envoient chaque point où les deux l'envoient l'un après l'autre** :
/// c'est ce qui permet de n'appliquer qu'une similitude par image, quel que soit le nombre de
/// doigts qui ont bougé.
#[test]
fn test_composer_est_appliquer_l_une_puis_l_autre() {
    let a = Mouvement {
        echelle: 1.5,
        decalage: (-225.0, 10.0),
    };
    let b = Mouvement {
        echelle: 4.0 / 3.0,
        decalage: (-200.0, -3.0),
    };
    let ab = composer(a, b);
    let suit = |m: &Mouvement, (x, y): (f64, f64)| {
        (m.echelle * x + m.decalage.0, m.echelle * y + m.decalage.1)
    };
    for q in [(0.0, 0.0), (450.0, 300.0), (-80.0, 1200.0)] {
        let (u, v) = (suit(&ab, q), suit(&b, suit(&a, q)));
        assert!(
            (u.0 - v.0).abs() < 1e-9 && (u.1 - v.1).abs() < 1e-9,
            "{q:?}"
        );
    }
}

/// Un toucher : le doigt se pose et se lève, sans bouger.
fn toucher_en(app: &mut GlucoseApp, ici: (f64, f64)) {
    doigt(app, 1, TouchPhase::Started, ici);
    doigt(app, 1, TouchPhase::Ended, ici);
}

/// **Deux touchers rapprochés sur le vide ouvrent le menu, au doigt** (fiche 56) : la main n'a
/// pas de clic droit. Un seul n'ouvre rien ; deux trop espacés dans le temps non plus.
#[test]
fn test_deux_touchers_sur_le_vide_ouvrent_le_menu_au_doigt() {
    let mut app = app();
    toucher_en(&mut app, (300.0, 400.0));
    assert_eq!(app.ui.context_menu_at, None, "un toucher seul n'ouvre rien");
    toucher_en(&mut app, (303.0, 398.0));
    assert_eq!(
        app.ui.context_menu_at,
        Some((303.0, 398.0)),
        "le menu, sous le doigt"
    );
    assert!(app.ui.menu_au_doigt, "à la taille du doigt");

    // Trop lents : le premier toucher date d'une seconde.
    let mut app = self::app();
    toucher_en(&mut app, (300.0, 400.0));
    if let Some(dernier) = &mut app.last_click {
        dernier.at_ms -= 1_000;
    }
    toucher_en(&mut app, (300.0, 400.0));
    assert_eq!(
        app.ui.context_menu_at, None,
        "deux touchers espacés ne sont pas un double"
    );
}

/// **Le clic droit, lui, ouvre le menu de la souris** — même si un doigt l'a ouvert avant.
#[test]
fn test_le_clic_droit_ouvre_le_menu_de_la_souris() {
    let mut app = app();
    toucher_en(&mut app, (300.0, 400.0));
    toucher_en(&mut app, (300.0, 400.0));
    assert!(app.ui.menu_au_doigt);
    let (largeur, hauteur) = app.taille_de_la_fenetre();
    app.handle_cursor_moved(PhysicalPosition::new(500.0, 500.0));
    app.handle_mouse_down(MouseButton::Right, largeur, hauteur);
    app.handle_mouse_up(MouseButton::Right);
    assert_eq!(app.ui.context_menu_at, Some((500.0, 500.0)));
    assert!(!app.ui.menu_au_doigt, "à la souris, le menu de la souris");
}

// ── APPUI-1 : l'appui long, le clic droit du doigt ──────────────────────────

/// L'échéance de l'appui en cours est atteinte : le temps a passé, doigt immobile.
fn echeance_atteinte(app: &mut GlucoseApp) {
    let maintenant = std::time::Instant::now();
    if let Some(appui) = &mut app.toucher.appui {
        appui.echeance = maintenant;
    }
}

/// **Un doigt qui tient ouvre le menu quand il se lève**, à la taille du doigt (GESTES-1 :
/// l'appui prend à l'échéance, et ce qui suit décide — rien n'a suivi, c'est le menu).
#[test]
fn test_appui_1_un_doigt_qui_tient_ouvre_le_menu_au_doigt() {
    let mut app = app();
    doigt(&mut app, 1, TouchPhase::Started, (300.0, 400.0));
    let attente = app.attente_de_l_appui().expect("la boucle attend l'appui");
    let delai = crate::plateforme::doigt::appui_long().as_millis() as u64;
    assert!(
        attente > 0 && attente <= delai,
        "réveillée à l'échéance : {attente} ms"
    );
    assert_eq!(app.ui.context_menu_at, None, "pas avant l'échéance");
    echeance_atteinte(&mut app);
    assert_eq!(app.attente_de_l_appui(), None, "pris, il n'attend plus");
    assert_eq!(app.ui.context_menu_at, None, "la suite décide");
    doigt(&mut app, 1, TouchPhase::Ended, (300.0, 400.0));
    assert_eq!(
        app.ui.context_menu_at,
        Some((300.0, 400.0)),
        "levé : le menu"
    );
    assert!(app.ui.menu_au_doigt);
}

/// **Un doigt qui part, ou qui se lève avant l'échéance, n'est pas un appui long.**
#[test]
fn test_appui_1_glisser_ou_se_lever_n_est_pas_un_appui() {
    let mut app = app();
    doigt(&mut app, 1, TouchPhase::Started, (300.0, 400.0));
    doigt(&mut app, 1, TouchPhase::Moved, (300.0, 460.0));
    echeance_atteinte(&mut app);
    assert_eq!(app.attente_de_l_appui(), None);
    assert_eq!(app.ui.context_menu_at, None, "il glissait : aucun menu");

    let mut app = self::app();
    doigt(&mut app, 1, TouchPhase::Started, (300.0, 400.0));
    // Un tremblement reste un appui.
    doigt(&mut app, 1, TouchPhase::Moved, (302.0, 401.0));
    assert!(
        app.toucher.appui.is_some(),
        "une main qui tremble tient encore"
    );
    doigt(&mut app, 1, TouchPhase::Ended, (302.0, 401.0));
    assert_eq!(app.toucher.appui, None, "levé avant : un toucher");

    let mut app = self::app();
    doigt(&mut app, 1, TouchPhase::Started, (300.0, 400.0));
    doigt(&mut app, 2, TouchPhase::Started, (500.0, 400.0));
    assert_eq!(
        app.toucher.appui, None,
        "deux doigts pincent, ils n'appuient pas"
    );
}

/// **Dans le texte qu'on écrit, l'appui long choisit le mot sous le doigt** — et n'ouvre aucun
/// menu.
#[test]
fn test_appui_1_dans_le_texte_le_mot_se_selectionne() {
    use glucose_core::types::{Annotation, Viewport};
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    app.store.set_viewport(&board, vue);
    let mut carte = Annotation::text("c", 0.0, 0.0, "bonjour le monde");
    if let Annotation::Text { width, height, .. } = &mut carte {
        *width = Some(400.0);
        *height = Some(200.0);
    }
    app.store.add_annotation(&board, carte);
    app.start_text_edit("c".into(), "bonjour le monde".into());
    doigt(&mut app, 1, TouchPhase::Started, (30.0, 20.0));
    echeance_atteinte(&mut app);
    app.attente_de_l_appui();
    let session = app.editing_session.as_ref().expect("la saisie continue");
    let (debut, fin) = (
        session.selection.anchor.min(session.selection.head),
        session.selection.anchor.max(session.selection.head),
    );
    assert_eq!(
        &session.buffer[debut..fin],
        "bonjour",
        "le mot sous le doigt"
    );
    assert_eq!(app.ui.context_menu_at, None, "aucun menu dans le texte");
}

/// Le centre de la réponse `r` de la question posée.
fn centre_de(app: &GlucoseApp, r: crate::ui::question::Reponse) -> (f64, f64) {
    let (question, _) = app.ui.question.clone().expect("une question");
    let (w, h) = app.taille_de_la_fenetre();
    let placee = crate::ui::question::placer(
        &question,
        &app.renderer.typography,
        (w, h),
        app.ui.scale_factor,
    );
    let ((x, y, bw, bh), _, _) = placee
        .boutons
        .iter()
        .find(|(_, _, reponse)| *reponse == r)
        .expect("la réponse")
        .clone();
    (f64::from(x + bw / 2.0), f64::from(y + bh / 2.0))
}

/// **Une question répond au relâchement, pas à l'appui** (APPUI-1) : un doigt qui glisse hors
/// de la réponse, ou qui tient jusqu'à l'appui long, n'a rien répondu ; et le doigt qui a ouvert
/// la question ne lui répond pas en se levant.
#[test]
fn test_appui_1_la_question_repond_au_relachement_sur_la_reponse_pressee() {
    use crate::ui::question::Reponse;
    let mut app = app();
    // Le doigt est posé quand la question paraît — comme sous un menu qui l'ouvre.
    doigt(&mut app, 1, TouchPhase::Started, (5.0, 5.0));
    app.nouveau_document();
    let creer = centre_de(&app, Reponse::Oui);
    doigt(&mut app, 1, TouchPhase::Moved, creer);
    doigt(&mut app, 1, TouchPhase::Ended, creer);
    assert!(app.ui.question.is_some(), "le doigt d'avant ne répond pas");
    assert!(
        !app.is_panning,
        "et son geste se termine : le canevas ne suit plus un doigt levé"
    );

    // Pressée, puis le doigt glisse ailleurs : rien.
    doigt(&mut app, 1, TouchPhase::Started, creer);
    assert!(app.ui.question.is_some(), "l'appui seul ne répond pas");
    doigt(&mut app, 1, TouchPhase::Moved, (creer.0, creer.1 + 300.0));
    doigt(&mut app, 1, TouchPhase::Ended, (creer.0, creer.1 + 300.0));
    assert!(app.ui.question.is_some(), "relâchée ailleurs : rien");

    // Tenue jusqu'à l'appui long : rien.
    doigt(&mut app, 1, TouchPhase::Started, creer);
    echeance_atteinte(&mut app);
    app.attente_de_l_appui();
    doigt(&mut app, 1, TouchPhase::Ended, creer);
    assert!(app.ui.question.is_some(), "un appui long ne répond pas");

    // Touchée et relâchée dessus : elle répond.
    doigt(&mut app, 1, TouchPhase::Started, creer);
    doigt(&mut app, 1, TouchPhase::Ended, creer);
    assert!(app.ui.question.is_none(), "répondue, elle s'en va");
}

/// **L'appui long choisit ce que choisit un toucher** : sur deux images superposées, le menu
/// s'ouvre sur celle qu'un toucher aurait prise — le doigt qui se lève ensuite ne relâche rien
/// une seconde fois, qui ferait descendre le choix à l'image du dessous.
#[test]
fn test_appui_1_l_appui_long_choisit_ce_que_choisit_un_toucher() {
    let superposees = || {
        let mut app = app();
        let board = app.store.project.active_board_id.clone();
        app.store
            .add_image(&board, BoardImage::new("dessous", 0.0, 0.0, 100.0, 100.0));
        app.store
            .add_image(&board, BoardImage::new("dessus", 0.0, 0.0, 100.0, 100.0));
        let vp = app.store.viewport();
        let (sx, sy) = crate::canvas::world_to_screen(0.0, 0.0, &vp);
        (app, (sx + 10.0, sy + 10.0))
    };
    let (mut tap, ici) = superposees();
    doigt(&mut tap, 1, TouchPhase::Started, ici);
    doigt(&mut tap, 1, TouchPhase::Ended, ici);
    let choisie = tap.store.selected_image_ids.clone();
    assert_eq!(choisie.len(), 1, "un toucher choisit une image");

    let (mut long, ici) = superposees();
    doigt(&mut long, 1, TouchPhase::Started, ici);
    echeance_atteinte(&mut long);
    long.attente_de_l_appui();
    doigt(&mut long, 1, TouchPhase::Ended, ici);
    assert_eq!(
        long.store.selected_image_ids, choisie,
        "la même que le toucher"
    );
    assert!(long.ui.context_menu_at.is_some(), "et son menu");
}

/// **L'appui long garde la sélection multiple** : sur l'une de deux images choisies, le menu
/// s'ouvre sur les deux — un toucher, lui, la ramène à celle qu'il touche (SEL-MULTI-1). Et
/// le doigt ne mène plus aucun geste ensuite.
#[test]
fn test_appui_1_l_appui_long_garde_la_selection_multiple() {
    let deux_choisies = || {
        let mut app = app();
        let board = app.store.project.active_board_id.clone();
        app.store
            .add_image(&board, BoardImage::new("a", 0.0, 0.0, 100.0, 100.0));
        app.store
            .add_image(&board, BoardImage::new("b", 300.0, 0.0, 100.0, 100.0));
        app.store.select_image("a".into(), false);
        app.store.select_image("b".into(), true);
        let vp = app.store.viewport();
        let (sx, sy) = crate::canvas::world_to_screen(0.0, 0.0, &vp);
        (app, (sx + 10.0, sy + 10.0))
    };
    let (mut tap, ici) = deux_choisies();
    doigt(&mut tap, 1, TouchPhase::Started, ici);
    doigt(&mut tap, 1, TouchPhase::Ended, ici);
    assert_eq!(
        tap.store.selected_image_ids.len(),
        1,
        "un toucher la réduit"
    );

    let (mut long, ici) = deux_choisies();
    doigt(&mut long, 1, TouchPhase::Started, ici);
    echeance_atteinte(&mut long);
    long.attente_de_l_appui();
    assert!(!long.toucher.doigts.seul, "le doigt ne mène plus de geste");
    doigt(&mut long, 1, TouchPhase::Ended, ici);
    assert_eq!(
        long.store.selected_image_ids.len(),
        2,
        "l'appui long la garde"
    );
    assert!(long.ui.context_menu_at.is_some());
}

/// **Un appui long au milieu d'une membrane vide la choisit, et son menu offre « Supprimer »**
/// (fiche 59) : « on n'a pas de bouton supprimer pour une membrane » — sur l'APK du 07/10, sans
/// appui long, la barre d'action passait sous la navigation. Le corps d'un conteneur vient en
/// dernier chez l'arbitre (PICK-2), mais sur le vide d'une membrane il n'y a rien d'autre.
#[test]
fn test_appui_1_l_appui_long_dans_une_membrane_offre_supprimer() {
    use crate::ui::context_menu::{layout_context_menu, MenuAction, MenuRow};
    use glucose_core::types::Annotation;
    let mut app = app();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::membrane("m", 0.0, 0.0, 400.0, 300.0));
    app.store.clear_selection();
    let vp = app.store.viewport();
    let ici = crate::canvas::world_to_screen(200.0, 150.0, &vp);
    doigt(&mut app, 1, TouchPhase::Started, ici);
    echeance_atteinte(&mut app);
    app.attente_de_l_appui();
    doigt(&mut app, 1, TouchPhase::Ended, ici);
    assert_eq!(
        app.store.selected_annotation_ids,
        vec!["m".to_string()],
        "la membrane est choisie"
    );
    let at = app.ui.context_menu_at.expect("son menu s'ouvre");
    let typo = crate::typography::Typography::new();
    let menu = layout_context_menu(&app.store, &typo, (at, None), (1280.0, 720.0), 1.0, true)
        .expect("un menu");
    let supprimer = menu.rows.iter().any(|r| {
        matches!(
            r,
            MenuRow::Item {
                action: MenuAction::Delete,
                ..
            }
        )
    });
    assert!(supprimer, "« Supprimer » est dans le menu");
}
