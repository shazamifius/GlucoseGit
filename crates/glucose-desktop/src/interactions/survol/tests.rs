//! **Au survol d'une flèche ancrée au second « bonjours », seul le second brille** — le défaut de
//! Tauri qu'il a relevé, éprouvé par la vraie souris et la vraie image.

use crate::app::GlucoseApp;
use glucose_core::text_anchors::create_anchor;
use glucose_core::types::{Annotation, TextSelection};
use winit::dpi::PhysicalPosition;

const SA_CARTE: &str = "bonjours\ntest\ntest\nbonjours";
const TAILLE: (u32, u32) = (1200, 800);

/// Sa carte en `(100, 100)`, une flèche ancrée au second « bonjours » vers un point à droite,
/// à la vue 1:1.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("carte", 100.0, 100.0, SA_CARTE));
    let second = SA_CARTE.rfind("bonjours").expect("le second");
    let mut fleche = Annotation::arrow("f", 100.0, 100.0, 900.0, 600.0);
    if let Annotation::Arrow {
        source_id,
        source_text_sel,
        ..
    } = &mut fleche
    {
        *source_id = Some("carte".to_string());
        *source_text_sel = Some(TextSelection::Anchors(vec![create_anchor(
            SA_CARTE,
            second,
            second + 8,
        )
        .expect("une ancre")]));
    }
    app.store.add_annotation(&board, fleche);
    app.store.clear_selection();
    app.store.set_viewport(
        &board,
        glucose_core::types::Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
    );
    app.une_image_sans_fenetre(TAILLE);
    app
}

/// Un point du trait de la flèche, à mi-chemin, en pixels d'écran.
fn sur_la_fleche(app: &GlucoseApp) -> (f64, f64) {
    let board = app.store.active_board().expect("un tableau");
    let ann = board
        .annotations
        .iter()
        .find(|a| a.id() == "f")
        .expect("la flèche");
    let noeuds = crate::renderer::arrow::NoeudsDuRendu {
        board,
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
    };
    let chemin = glucose_core::arrow::path_with(ann, noeuds).expect("un chemin");
    let (a, b) = (chemin[0], chemin[chemin.len() - 1]);
    ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
}

/// **La souris sur la flèche : seul le second « bonjours » est à éclairer** ; hors d'elle,
/// rien.
#[test]
fn test_fleche_4_seul_le_second_bonjours_brille() {
    let mut app = application();
    let p = sur_la_fleche(&app);
    app.handle_cursor_moved(PhysicalPosition::new(p.0, p.1));
    assert_eq!(app.ui.fleche_survolee.as_deref(), Some("f"));
    let second = SA_CARTE.rfind("bonjours").expect("le second");
    let eclairages = app.eclairages();
    assert_eq!(eclairages.len(), 1);
    assert_eq!(eclairages[0].carte, "carte");
    assert_eq!(eclairages[0].plages, vec![(second, second + 8)]);

    app.handle_cursor_moved(PhysicalPosition::new(
        1150.0,
        50.0 + f64::from(app.ui.header_height()),
    ));
    assert!(app.ui.fleche_survolee.is_none(), "hors de la flèche, rien");
    assert!(app.eclairages().is_empty());
}

/// **Et à l'image, c'est bien lui qui brille** : sous la souris, le fond du second « bonjours »
/// prend la couleur de la carte ; celui du premier ne bouge pas.
#[test]
fn test_fleche_4_a_l_image_le_second_brille_et_le_premier_non() {
    let mut app = application();
    // Le fond de chaque mot, juste à gauche de sa première lettre, avant le survol.
    let ligne = crate::renderer::card::text_box(240.0).line_height;
    let (ox, oy) = crate::renderer::card::TEXT_ORIGIN;
    let fond = |app: &GlucoseApp, rang: f32| {
        let image = app.pixmap.as_ref().expect("une image");
        let (x, y) = (100.0 + ox - 1.5, 100.0 + oy + (rang + 0.5) * ligne);
        let p = image.pixel(x as u32, y as u32).expect("dedans");
        (p.red(), p.green(), p.blue())
    };
    let (premier, second) = (fond(&app, 0.0), fond(&app, 3.0));
    let p = sur_la_fleche(&app);
    app.handle_cursor_moved(PhysicalPosition::new(p.0, p.1));
    app.une_image_sans_fenetre(TAILLE);
    assert_eq!(fond(&app, 0.0), premier, "le premier ne brille pas");
    assert_ne!(fond(&app, 3.0), second, "le second brille");
}

/// **Pendant qu'on tire une flèche, la carte visée s'avive** — et, l'outil seulement armé,
/// celle dont elle partirait (LUEUR-1). L'indice qu'il réclamait : savoir, avant de lâcher, à
/// quoi la flèche va se lier.
#[test]
fn test_lueur_1_la_carte_visee_se_designe() {
    use winit::event::MouseButton;
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    app.store
        .add_annotation(&board, Annotation::text("source", 100.0, 100.0, "a"));
    app.store
        .add_annotation(&board, Annotation::text("cible", 700.0, 100.0, "b"));
    app.store.clear_selection();
    app.store.set_viewport(
        &board,
        glucose_core::types::Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
    );
    app.une_image_sans_fenetre(TAILLE);
    app.ui.active_tool = crate::ui::ActiveTool::Arrow;
    app.handle_cursor_moved(PhysicalPosition::new(110.0, 110.0));
    assert_eq!(
        app.cartes_designees(),
        ["source"],
        "l'outil armé désigne l'origine"
    );

    app.handle_mouse_down(MouseButton::Left, TAILLE.0 as f32, TAILLE.1 as f32);
    for x in [200.0, 400.0, 600.0, 710.0] {
        app.handle_cursor_moved(PhysicalPosition::new(x, 110.0));
    }
    assert_eq!(app.cartes_designees(), ["cible"], "la pointe vise la cible");
    app.handle_mouse_up(MouseButton::Left);
    assert!(
        app.cartes_designees().is_empty(),
        "l'outil rendu, rien n'est désigné"
    );
}

/// **La carte visée s'avive en deux cents millisecondes, par les vrais gestes** (LUEUR-2) : le
/// réveil demande des images tant que sa lueur glisse, et plus aucune ensuite.
#[test]
fn test_lueur_2_la_carte_visee_s_avive_par_le_reveil() {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    app.store
        .add_annotation(&board, Annotation::text("source", 100.0, 100.0, "a"));
    app.store.clear_selection();
    app.store.set_viewport(
        &board,
        glucose_core::types::Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        },
    );
    app.une_image_sans_fenetre(TAILLE);
    app.ui.active_tool = crate::ui::ActiveTool::Arrow;
    app.handle_cursor_moved(PhysicalPosition::new(110.0, 110.0));
    let vive = |app: &mut GlucoseApp| {
        app.suivre_la_designation()
            .into_iter()
            .find(|(id, _)| id == "source")
            .map_or(0.0, |(_, v)| v)
    };
    // Le réveil seul, sans que l'épreuve suive quoi que ce soit : c'est lui qui prend la
    // désignation en charge et fait glisser la lueur d'image en image.
    assert!(
        app.prochain_reveil().is_some(),
        "la lueur glisse : une image est demandée"
    );
    assert!(
        app.vivacites.cartes.en_cours(app.now_ms() as f64),
        "le réveil suit la carte visée"
    );
    let depart = vive(&mut app);
    assert!(depart < 0.2, "elle part du repos : {depart}");
    app.click_epoch -= std::time::Duration::from_millis(250);
    assert_eq!(
        vive(&mut app),
        1.0,
        "au bout de deux cents millisecondes, pleinement vive"
    );
    let _ = app.prochain_reveil();
    assert!(
        !app.vivacites.cartes.en_cours(app.now_ms() as f64),
        "plus rien ne glisse : plus d'image demandée pour elle"
    );
}

// ── BADGE-1 : la pastille de relation ──────────────────────────────────────────────────────

/// Une flèche droite de (100, 300) à (700, 300), qui « contredit », vue au zoom `zoom` et
/// centrée à l'écran.
fn fleche_qui_contredit(zoom: f64) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    let mut fleche = Annotation::arrow("f", 100.0, 300.0, 700.0, 300.0);
    if let Annotation::Arrow { predicate, .. } = &mut fleche {
        *predicate = Some(glucose_core::types::ArrowPredicate::Contredit);
    }
    app.store.add_annotation(&board, fleche);
    app.store.clear_selection();
    app.store.set_viewport(
        &board,
        glucose_core::types::Viewport {
            x: 600.0 - 400.0 * zoom,
            y: 400.0 - 300.0 * zoom,
            scale: zoom,
        },
    );
    app.une_image_sans_fenetre(TAILLE);
    app.ui.current_toast = None;
    app
}

/// Combien de colonnes, sur la ligne de la flèche, la pastille couvre à l'écran : ce qui change
/// entre l'image et celle d'une flèche sans relation.
fn largeur_de_la_pastille(zoom: f64) -> usize {
    let mut avec = fleche_qui_contredit(zoom);
    let mut sans = fleche_qui_contredit(zoom);
    let board = sans.store.project.active_board_id.clone();
    if let Some(Annotation::Arrow { predicate, .. }) = sans
        .store
        .project
        .boards
        .iter_mut()
        .find(|b| b.id == board)
        .and_then(|b| b.annotations.iter_mut().find(|a| a.id() == "f"))
    {
        *predicate = None;
    }
    let rendre = |app: &mut GlucoseApp| {
        app.ui.current_toast = None;
        app.une_image_sans_fenetre(TAILLE);
        app.pixmap.clone().expect("une image")
    };
    let (a, b) = (rendre(&mut avec), rendre(&mut sans));
    // La ligne juste au-dessus du trait : la pastille y est, le trait non.
    let y = 400 - 4;
    (0..TAILLE.0)
        .filter(|&x| a.pixel(x, y) != b.pixel(x, y))
        .count()
}

/// **BADGE-1 — la pastille rapetisse quand on dézoome**, comme chez Tauri où tout le calque
/// des flèches suit le zoom : à ×0,5 elle couvre deux fois moins de colonnes qu'à ×1. Dézoomée
/// très loin, elle était plus grande qu'un groupe entier d'images (sa capture du 26/09).
#[test]
fn test_badge_1_la_pastille_suit_le_zoom() {
    let (un, demi) = (largeur_de_la_pastille(1.0), largeur_de_la_pastille(0.5));
    assert!(un > 12, "la pastille se voit a x1 : {un} colonnes");
    assert!(
        (un as f64 / demi as f64 - 2.0).abs() < 0.35,
        "a x0,5 la pastille couvre {demi} colonnes, pour {un} a x1"
    );
}

/// **BADGE-1 — la pastille survolée s'efface, puis revient** : la souris sur elle, elle
/// s'efface en deux cents millisecondes ; la souris partie, elle revient de même. Par la vraie
/// souris et le vrai réveil.
#[test]
fn test_badge_1_la_pastille_survolee_s_efface() {
    let mut app = fleche_qui_contredit(1.0);
    let effacement = |app: &mut GlucoseApp| {
        app.suivre_les_badges()
            .into_iter()
            .find(|(id, _)| id == "f")
            .map_or(0.0, |(_, e)| e)
    };
    // Le centre de la pastille : le milieu du tracé, (400, 300), à l'écran (600, 400).
    app.handle_cursor_moved(PhysicalPosition::new(603.0, 402.0));
    assert_eq!(effacement(&mut app), 0.0, "elle commence visible");
    app.click_epoch -= std::time::Duration::from_millis(250);
    assert!(effacement(&mut app) > 0.99, "elle s'est effacee");
    // Et l'image le montre : là où la pastille était, plus rien d'elle — la même ligne que sur
    // une flèche sans relation.
    let mut sans = fleche_qui_contredit(1.0);
    let board = sans.store.project.active_board_id.clone();
    if let Some(Annotation::Arrow { predicate, .. }) = sans
        .store
        .project
        .boards
        .iter_mut()
        .find(|b| b.id == board)
        .and_then(|b| b.annotations.iter_mut().find(|a| a.id() == "f"))
    {
        *predicate = None;
    }
    for a in [&mut app, &mut sans] {
        a.ui.current_toast = None;
        a.une_image_sans_fenetre(TAILLE);
    }
    let (vu, attendu) = (
        app.pixmap.as_ref().expect("une image"),
        sans.pixmap.as_ref().expect("une image"),
    );
    for x in 585..615 {
        assert_eq!(vu.pixel(x, 396), attendu.pixel(x, 396), "colonne {x}");
    }
    assert!(
        !app.vivacites.en_cours(app.now_ms() as f64),
        "la transition est finie"
    );

    app.handle_cursor_moved(PhysicalPosition::new(603.0, 460.0));
    let _ = effacement(&mut app);
    assert!(
        app.vivacites.en_cours(app.now_ms() as f64),
        "elle revient : le reveil doit le savoir"
    );
    app.click_epoch -= std::time::Duration::from_millis(250);
    assert_eq!(effacement(&mut app), 0.0, "elle est revenue");
}
