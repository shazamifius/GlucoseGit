//! PLACEMENT-1, joué sur le vrai `GlucoseApp`, par le chemin de la souris : le fantôme suit le
//! curseur aimanté, et le clic pose l'élément là où il était.

use super::*;
use glucose_core::types::{Annotation, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Une carte voisine, dont le bord gauche est en `x = 100`. Plus large qu'une carte neuve : sans
/// quoi leurs centres s'aligneraient en même temps que leurs bords, et le guide montrerait le
/// centre.
const VOISINE: (f64, f64, f64, f64) = (100.0, 100.0, 300.0, 60.0);

/// Où l'origine du monde tombe à l'écran : loin de la barre, des onglets et des panneaux.
const VUE: (f64, f64) = (300.0, 150.0);

/// Une application au tableau vide, hormis la voisine, vue à l'échelle 1.
fn app_avec_une_voisine(outil: ActiveTool) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let tableau = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.folders.clear();
    }
    app.store.set_viewport(
        &tableau,
        Viewport {
            scale: 1.0,
            x: VUE.0,
            y: VUE.1,
        },
    );
    let (x, y, w, h) = VOISINE;
    let mut voisine = Annotation::text("voisine", x, y, "Une voisine");
    if let Annotation::Text { width, height, .. } = &mut voisine {
        (*width, *height) = (Some(w), Some(h));
    }
    app.store.add_annotation(&tableau, voisine);
    app.ui.active_tool = outil;
    app
}

/// Le curseur au point `(wx, wy)` du monde.
fn survoler(app: &mut GlucoseApp, (wx, wy): (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(wx + VUE.0, wy + VUE.1));
}

fn cliquer(app: &mut GlucoseApp, point: (f64, f64)) {
    survoler(app, point);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
}

/// La dernière annotation posée, et son coin.
fn coin_de_la_derniere(app: &GlucoseApp) -> (f64, f64) {
    let a = app
        .store
        .active_board()
        .and_then(|b| b.annotations.last())
        .expect("une annotation");
    let r = a.rect().expect("une boîte");
    (r.left, r.top)
}

/// **Le fantôme s'aimante, et le clic pose là où il était.** Trois pixels à droite du bord
/// gauche de la voisine, sous le seuil de l'aimant : le fantôme s'aligne sur elle, le guide
/// le dit, et la carte naît alignée — sans avoir eu à la glisser.
#[test]
fn test_placement_1_le_fantome_s_aimante_et_le_clic_pose_la_ou_il_est() {
    for outil in [ActiveTool::Text, ActiveTool::Sticky, ActiveTool::Membrane] {
        let mut app = app_avec_une_voisine(outil);
        let point = (VOISINE.0 + 3.0, 400.0);
        survoler(&mut app, point);
        let fantome = app
            .fantome_montre()
            .expect("un fantôme sous un outil de création");
        assert_eq!(
            fantome.rect.left, VOISINE.0,
            "{outil:?} : le fantôme s'aligne sur la voisine"
        );
        assert_eq!(
            fantome.rect.top, 400.0,
            "{outil:?} : rien à aligner en hauteur"
        );
        assert_eq!(
            fantome.guides.x.as_deref(),
            Some(&[VOISINE.0][..]),
            "{outil:?} : le guide montre à quoi il s'aligne"
        );
        cliquer(&mut app, point);
        assert_eq!(
            coin_de_la_derniere(&app),
            (VOISINE.0, 400.0),
            "{outil:?} : l'élément naît là où était son fantôme"
        );
        assert_eq!(
            app.fantome_montre(),
            None,
            "{outil:?} : posé, l'outil rend la main et le fantôme part"
        );
    }
}

/// **Un dossier aussi** — il n'est pas une annotation, et il naît par un autre chemin.
#[test]
fn test_placement_1_un_dossier_nait_aligne() {
    let mut app = app_avec_une_voisine(ActiveTool::Folder);
    cliquer(&mut app, (VOISINE.0 - 4.0, 500.0));
    let dossier = app
        .store
        .active_board()
        .and_then(|b| b.folders.last())
        .expect("un dossier");
    assert_eq!((dossier.x, dossier.y), (VOISINE.0, 500.0));
}

/// **L'Aimant éteint, le fantôme suit le curseur tel quel** — et la carte naît sous lui.
#[test]
fn test_placement_1_sans_l_aimant_rien_ne_s_aligne() {
    let mut app = app_avec_une_voisine(ActiveTool::Text);
    app.ui.smart_align = false;
    let point = (VOISINE.0 + 3.0, 400.0);
    survoler(&mut app, point);
    let fantome = app.fantome_montre().expect("un fantôme");
    assert_eq!((fantome.rect.left, fantome.rect.top), point);
    assert_eq!(fantome.guides, SnapGuides::default());
    cliquer(&mut app, point);
    assert_eq!(coin_de_la_derniere(&app), point);
}

/// **Loin de tout, rien ne bouge** : l'aimant ne tire qu'à portée.
#[test]
fn test_placement_1_loin_de_tout_le_fantome_reste_sous_le_curseur() {
    let mut app = app_avec_une_voisine(ActiveTool::Text);
    survoler(&mut app, (800.0, 600.0));
    let fantome = app.fantome_montre().expect("un fantôme");
    assert_eq!((fantome.rect.left, fantome.rect.top), (800.0, 600.0));
    assert_eq!(fantome.guides, SnapGuides::default());
}

/// **Sous un outil qui ne crée pas de boîte, pas de fantôme** — et un outil changé sans que la
/// souris bouge ne laisse pas traîner celui d'avant.
#[test]
fn test_placement_1_pas_de_fantome_hors_des_outils_de_creation() {
    let mut app = app_avec_une_voisine(ActiveTool::Text);
    survoler(&mut app, (800.0, 600.0));
    assert!(app.fantome_montre().is_some());
    for outil in [ActiveTool::Select, ActiveTool::Pan, ActiveTool::Arrow] {
        app.ui.active_tool = outil;
        assert_eq!(app.fantome_montre(), None, "{outil:?}");
    }
}

/// **Les cibles de l'aimant suivent le document** : une carte posée après que le fantôme a
/// commencé de suivre le curseur l'attire aussi.
#[test]
fn test_placement_1_les_cibles_suivent_le_document() {
    let mut app = app_avec_une_voisine(ActiveTool::Text);
    survoler(&mut app, (800.0, 600.0));
    let tableau = app.store.project.active_board_id.clone();
    let mut nouvelle = Annotation::text("nouvelle", 500.0, 900.0, "Nouvelle");
    if let Annotation::Text { width, height, .. } = &mut nouvelle {
        (*width, *height) = (Some(200.0), Some(50.0));
    }
    app.store.add_annotation(&tableau, nouvelle);
    survoler(&mut app, (504.0, 600.0));
    let fantome = app.fantome_montre().expect("un fantôme");
    assert_eq!(
        fantome.rect.left, 500.0,
        "la carte posée entre-temps attire le fantôme"
    );
}

/// **Le fantôme se voit** : l'image, rendue avec et sans lui, diffère sur son bord et pas
/// ailleurs que dans sa boîte.
#[test]
fn test_placement_1_le_fantome_se_dessine_la_ou_il_se_poserait() {
    let mut app = app_avec_une_voisine(ActiveTool::Text);
    // Le message d'accueil s'estompe avec le temps : deux rendus ne le montreraient pas pareil.
    app.ui.current_toast = None;
    survoler(&mut app, (800.0, 500.0));
    let fantome = app.fantome_montre().expect("un fantôme");
    let rendu = |app: &mut GlucoseApp, avec: bool| {
        let taille = (ECRAN.0 as u32, ECRAN.1 as u32);
        let mut pixmap = tiny_skia::Pixmap::new(taille.0, taille.1).expect("pixmap");
        let overlay = crate::params::SceneOverlay {
            fantome: avec.then_some(fantome.rect),
            ..crate::params::SceneOverlay::sans_rien(&app.active_guides)
        };
        app.renderer.render(
            &mut pixmap.as_mut(),
            &app.store,
            &mut app.ui,
            overlay,
            crate::params::Pointer { x: 0.0, y: 0.0 },
            crate::renderer::Regard::immobile(),
        );
        pixmap
    };
    let (avec, sans) = (rendu(&mut app, true), rendu(&mut app, false));
    let (x0, y0) = (fantome.rect.left + VUE.0, fantome.rect.top + VUE.1);
    let (x1, y1) = (x0 + fantome.rect.width, y0 + fantome.rect.height);
    let mut au_bord = 0;
    for (i, (a, b)) in avec.pixels().iter().zip(sans.pixels()).enumerate() {
        if a == b {
            continue;
        }
        let (x, y) = (
            (i as u32 % ECRAN.0 as u32) as f64,
            (i as u32 / ECRAN.0 as u32) as f64,
        );
        assert!(
            x >= x0 - 1.0 && x <= x1 + 1.0 && y >= y0 - 1.0 && y <= y1 + 1.0,
            "le fantôme change ({x}, {y}), hors de sa boîte"
        );
        au_bord += usize::from((x - x0).abs() < 1.0 || (y - y0).abs() < 1.0);
    }
    assert!(au_bord > 0, "le bord du fantôme doit se voir");
}

/// **Le seuil de l'aimant est en pixels logiques** (DPI-1) : sur un écran à 150 %, huit
/// pixels logiques en font douze. À dix unités de la voisine, vue à l'échelle 1, le fantôme
/// s'aimante à 150 % et pas à 100 % — comme un glisser.
#[test]
fn test_placement_1_le_seuil_suit_la_densite_de_l_ecran() {
    let point = (VOISINE.0 + 10.0, 400.0);
    let mut cent = app_avec_une_voisine(ActiveTool::Sticky);
    survoler(&mut cent, point);
    assert_eq!(
        cent.fantome_montre().map(|f| f.rect.left),
        Some(point.0),
        "a 100 %, dix pixels depassent le seuil"
    );
    let mut cent_cinquante = app_avec_une_voisine(ActiveTool::Sticky);
    cent_cinquante.ui.scale_factor = 1.5;
    survoler(&mut cent_cinquante, point);
    assert_eq!(
        cent_cinquante.fantome_montre().map(|f| f.rect.left),
        Some(VOISINE.0),
        "a 150 %, ils sont sous le seuil"
    );
}
