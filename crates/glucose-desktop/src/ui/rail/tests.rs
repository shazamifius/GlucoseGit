//! Le rail (fiche 55) : quand la barre part sur le côté, ce qu'il porte, et le doigt qui s'en
//! sert — jusqu'à la vraie souris, sur une fenêtre de la taille de son Redmi 9.

use super::*;
use crate::ui::{layout_topbar, TABS_HEIGHT};

/// Son téléphone : 720 × 1600 pixels, interface à 200 %.
const TELEPHONE: (f32, f32) = (720.0, 1600.0);

fn au_telephone() -> UiState {
    let mut ui = UiState::new();
    ui.scale_factor = 2.0;
    ui
}

/// **La mesure décide** : sur son téléphone debout, même les icônes seules ne tiennent pas, et la
/// barre part sur le côté ; dans une fenêtre de bureau, elle reste en haut.
#[test]
fn test_la_mesure_decide_barre_ou_rail() {
    let (typo, store) = (Typography::new(), Store::new("t"));
    let mut ui = au_telephone();
    decider(&mut ui, &typo, &store, TELEPHONE.0);
    assert!(ui.rail.actif, "un téléphone debout");
    let mut ui = UiState::new();
    decider(&mut ui, &typo, &store, 1440.0);
    assert!(!ui.rail.actif, "une fenêtre de bureau");
}

/// **Avec le rail, la bande n'a plus que les onglets** : tout ce qui se mesure sur elle — le
/// canevas, les clics — commence sous eux.
#[test]
fn test_avec_le_rail_la_bande_n_a_que_les_onglets() {
    let mut ui = au_telephone();
    ui.rail.actif = true;
    assert_eq!(ui.topbar_height(), 0.0);
    assert_eq!(ui.header_height(), TABS_HEIGHT * 2.0);
}

/// **Le rail porte exactement les boutons de la barre**, dans le même ordre, avec leur état : une
/// seule liste, deux mises en page.
#[test]
fn test_le_rail_porte_les_boutons_de_la_barre() {
    let typo = Typography::new();
    let mut ui = au_telephone();
    ui.rail = Rail {
        actif: true,
        ouvert: true,
    };
    let rail = layout_rail(&ui, &typo, TELEPHONE.1);
    let barre = layout_topbar(4000.0, &UiState::new(), &typo, 0);
    let actions = |b: &[TopbarButtonDef]| -> Vec<(UiAction, bool)> {
        b.iter().map(|b| (b.action.clone(), b.active)).collect()
    };
    assert_eq!(actions(&rail.boutons), actions(&barre.buttons));
}

/// **Au doigt** : chaque case fait 48 points de côté au moins, et tout le panneau tient dans
/// l'écran — debout comme couché, où les colonnes se multiplient au lieu de déborder.
#[test]
fn test_chaque_case_fait_48_points_et_tout_tient() {
    let typo = Typography::new();
    let mut ui = au_telephone();
    ui.rail = Rail {
        actif: true,
        ouvert: true,
    };
    for (largeur, hauteur) in [TELEPHONE, (TELEPHONE.1, TELEPHONE.0)] {
        let rail = layout_rail(&ui, &typo, hauteur);
        for b in &rail.boutons {
            assert!(b.w >= 48.0 * 2.0 && b.h >= 48.0 * 2.0, "{b:?}");
            assert!(
                b.x + b.w <= largeur && b.y + b.h <= hauteur,
                "{b:?} hors de {largeur} × {hauteur}"
            );
        }
        let (lx, ly, lw, lh) = rail.languette;
        assert!(lx + lw <= largeur && ly + lh <= hauteur && lh >= 48.0 * 2.0);
    }
}

/// **La languette déplie et replie ; un bouton agit, une seule fois, et replie** : l'aimant du
/// rail bascule comme celui de la barre, et le dit.
#[test]
fn test_la_languette_et_les_boutons() {
    let typo = Typography::new();
    let mut ui = au_telephone();
    ui.rail.actif = true;
    let (lx, ly, ..) = layout_rail(&ui, &typo, TELEPHONE.1).languette;
    assert_eq!(
        clic((lx + 1.0, ly + 1.0), TELEPHONE.1, &mut ui, &typo),
        Some(None)
    );
    assert!(ui.rail.ouvert, "déplié");
    let rail = layout_rail(&ui, &typo, TELEPHONE.1);
    let aimant = rail
        .boutons
        .iter()
        .find(|b| b.action == UiAction::ToggleMagnet)
        .expect("l'aimant");
    let avant = ui.smart_align;
    let action = clic(
        (aimant.x + 2.0, aimant.y + 2.0),
        TELEPHONE.1,
        &mut ui,
        &typo,
    );
    assert_eq!(action, Some(Some(UiAction::ToggleMagnet)));
    assert_eq!(ui.smart_align, !avant, "basculé une fois");
    assert!(!ui.rail.ouvert, "un bouton choisi replie le panneau");
    assert_eq!(
        clic((600.0, 1000.0), TELEPHONE.1, &mut ui, &typo),
        None,
        "le canevas"
    );
}

/// **À la vraie souris, sur une fenêtre de son téléphone** : l'image rendue décide le rail ; un
/// appui sur la languette le déplie, un appui sur le bouton Texte arme l'outil Texte.
#[test]
fn test_a_la_vraie_souris_sur_son_telephone() {
    use crate::params::{Pointer, SceneOverlay};
    use winit::dpi::PhysicalPosition;
    use winit::event::MouseButton;
    let mut app = crate::app::GlucoseApp::new();
    app.ui.scale_factor = 2.0;
    let mut image = Pixmap::new(TELEPHONE.0 as u32, TELEPHONE.1 as u32).expect("une image");
    let mut rendre = |app: &mut crate::app::GlucoseApp| {
        let overlay = SceneOverlay::sans_rien(&app.active_guides);
        app.renderer.render(
            &mut image.as_mut(),
            &app.store,
            &mut app.ui,
            overlay,
            Pointer { x: 0.0, y: 0.0 },
            crate::renderer::Regard::immobile(),
        );
    };
    rendre(&mut app);
    assert!(app.ui.rail.actif, "la barre est partie sur le côté");
    let appuyer = |app: &mut crate::app::GlucoseApp, (x, y): (f32, f32)| {
        app.handle_cursor_moved(PhysicalPosition::new(f64::from(x), f64::from(y)));
        app.handle_mouse_down(MouseButton::Left, TELEPHONE.0, TELEPHONE.1);
        app.handle_mouse_up(MouseButton::Left);
    };
    let (lx, ly, ..) = layout_rail(&app.ui, &app.renderer.typography, TELEPHONE.1).languette;
    appuyer(&mut app, (lx + 4.0, ly + 4.0));
    assert!(app.ui.rail.ouvert);
    rendre(&mut app);
    let texte = layout_rail(&app.ui, &app.renderer.typography, TELEPHONE.1)
        .boutons
        .into_iter()
        .find(|b| b.action == UiAction::SelectTool(crate::ui::ActiveTool::Text))
        .expect("le bouton Texte");
    appuyer(&mut app, (texte.x + 4.0, texte.y + 4.0));
    assert_eq!(app.ui.active_tool, crate::ui::ActiveTool::Text);
    assert!(!app.ui.rail.ouvert);
}

/// **Le rail se dessine une fois par changement** : deux images identiques, un seul dessin.
#[test]
fn test_le_rail_se_dessine_une_fois_par_changement() {
    let (typo, theme) = (Typography::new(), Theme::default());
    let mut ui = au_telephone();
    ui.rail = Rail {
        actif: true,
        ouvert: true,
    };
    let mut image = Pixmap::new(TELEPHONE.0 as u32, TELEPHONE.1 as u32).expect("une image");
    for _ in 0..3 {
        render_rail(&mut image.as_mut(), &mut ui, &typo, &theme);
    }
    assert_eq!(ui.rail_cache.as_ref().map(|c| c.dessins), Some(1));
    ui.active_tool = crate::ui::ActiveTool::Text;
    render_rail(&mut image.as_mut(), &mut ui, &typo, &theme);
    assert_eq!(
        ui.rail_cache.as_ref().map(|c| c.dessins),
        Some(2),
        "l'outil a changé"
    );
}

/// **Avec le rail, la bande ne porte plus aucun bouton de la barre** : posés au-dessus d'une
/// barre de hauteur nulle, ils mordraient sur les onglets, et le pointeur les y survolerait.
/// Là où le premier outil de la barre tomberait, le pointeur ne survole aucun bouton.
#[test]
fn test_avec_le_rail_la_bande_ne_porte_aucun_bouton() {
    let (typo, store) = (Typography::new(), Store::new("t"));
    let mut ui = au_telephone();
    ui.rail.actif = true;
    let premier = layout_topbar(TELEPHONE.0, &ui, &typo, 0).buttons[0].clone();
    let pointer = crate::params::Pointer {
        x: premier.x + 1.0,
        y: 1.0,
    };
    let survol = crate::ui::bande::survol_de_la_bande(&store, &ui, &typo, TELEPHONE.0, pointer);
    assert_eq!(survol.bouton(), None);
}

/// **Sur un téléphone, la toute première image est déjà au rail** — et la suivante lui est
/// identique. Décidé dans l'interface, le rail arrivait une image trop tard : la première
/// plaçait le canevas sous une barre qui n'existait plus à la seconde, 120 pixels plus bas.
#[test]
fn test_la_premiere_image_est_deja_au_rail() {
    let store = crate::bench::ouvert(Store::new("t"));
    let premiere = crate::bench::capture_with(&store, 720, 1600, |ui| ui.scale_factor = 2.0);
    let mut renderer = crate::renderer::Renderer::new();
    let mut ui = au_telephone();
    let une = crate::bench::render_frame(&mut renderer, &mut ui, &store, 720, 1600);
    assert!(ui.rail.actif, "le rail dès la première image");
    let deux = crate::bench::render_frame(&mut renderer, &mut ui, &store, 720, 1600);
    assert_eq!(une.data(), deux.data(), "la seconde image est la même");
    assert_eq!(une.encode_png().ok(), Some(premiere));
}
