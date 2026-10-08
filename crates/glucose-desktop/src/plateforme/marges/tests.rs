//! BORD-1 — l'interface se cale dans ce que le système laisse ; le canevas va jusqu'au bord.

use super::*;
use crate::app::GlucoseApp;
use crate::typography::Typography;
use crate::ui::UiState;

/// Les marges d'un téléphone debout : la barre d'état, la navigation.
fn telephone() -> Marges {
    Marges {
        haut: 48.0,
        bas: 96.0,
        ..Marges::default()
    }
}

/// **Le système dit ses marges, la boucle les prend une fois**, et se réveille pour cela.
#[test]
fn test_bord_1_les_marges_arrivent_par_la_boite_aux_lettres() {
    let reveils = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let compte = reveils.clone();
    brancher(std::sync::Arc::new(move || {
        compte.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }));
    recevoir(telephone());
    assert_eq!(nouvelles(), Some(telephone()));
    assert_eq!(nouvelles(), None, "une seule fois");
    assert!(
        reveils.load(std::sync::atomic::Ordering::Relaxed) >= 1,
        "la boucle se réveille"
    );
}

/// **La barre du haut descend sous la barre d'état** : son fond la couvre, ses boutons et les
/// onglets se posent dessous, exactement de la marge.
#[test]
fn test_bord_1_la_barre_descend_sous_la_barre_d_etat() {
    let typo = Typography::new();
    let mut ui = UiState::new();
    let avant = crate::ui::layout_topbar(1440.0, &ui, &typo, 0);
    let (barre, entete) = (ui.topbar_height(), ui.header_height());
    ui.marges = telephone();
    let apres = crate::ui::layout_topbar(1440.0, &ui, &typo, 0);
    assert_eq!(
        ui.topbar_height(),
        barre + 48.0,
        "le fond couvre la barre d'état"
    );
    assert_eq!(
        ui.header_height(),
        entete + 48.0,
        "le canevas commence dessous"
    );
    for (a, b) in avant.buttons.iter().zip(&apres.buttons) {
        assert_eq!(b.y, a.y + 48.0, "chaque bouton descend de la marge");
    }
    let store = glucose_core::store::Store::new("bord");
    let onglets = crate::ui::layout_tabs(&store, &ui, &typo);
    assert_eq!(
        onglets[0].y,
        ui.topbar_height(),
        "les onglets sous la barre"
    );
}

/// **Le rail s'écarte de l'encoche**, la minimap et le message de la navigation — et du
/// clavier quand il est sorti.
#[test]
fn test_bord_1_le_rail_la_minimap_et_le_message_s_ecartent_des_bords() {
    let typo = Typography::new();
    let mut ui = UiState::new();
    ui.marges = Marges {
        gauche: 30.0,
        ..Marges::default()
    };
    let rail = crate::ui::rail::layout_rail(&ui, &typo, 900.0);
    assert_eq!(rail.languette.0, 30.0, "le rail commence après l'encoche");

    let mut store = glucose_core::store::Store::new("bord");
    let board = store.project.active_board_id.clone();
    store.add_image(
        &board,
        glucose_core::types::BoardImage::new("a", 0.0, 0.0, 100.0, 100.0),
    );
    let sans = crate::ui::layout_minimap(&store, 1440.0, 900.0, 1.0, (0.0, 0.0)).unwrap();
    let avec = crate::ui::layout_minimap(&store, 1440.0, 900.0, 1.0, (20.0, 96.0)).unwrap();
    assert_eq!((avec.mm_x, avec.mm_y), (sans.mm_x - 20.0, sans.mm_y - 96.0));
    assert_eq!(
        avec.vp_h, sans.vp_h,
        "le cadre de la caméra reste celui de l'écran"
    );

    ui.marges = telephone();
    assert_eq!(ui.ecran_visible((720.0, 1600.0)), (720.0, 1504.0));
    ui.marges.clavier = 700.0;
    assert_eq!(
        ui.ecran_visible((720.0, 1600.0)),
        (720.0, 900.0),
        "le clavier recouvre la navigation"
    );
}

/// **La question se pose au-dessus du clavier**, et répond là où elle est dessinée.
#[test]
fn test_bord_1_la_question_reste_au_dessus_du_clavier() {
    use crate::ui::question::{placer, Reponse};
    let mut app = GlucoseApp::new();
    app.ui.marges = Marges {
        clavier: 500.0,
        ..telephone()
    };
    app.nouveau_document();
    let (question, _) = app.ui.question.clone().expect("la question");
    let (w, h) = app.taille_de_la_fenetre();
    let visible = app.ui.ecran_visible((w, h));
    let placee = placer(
        &question,
        &app.renderer.typography,
        visible,
        app.ui.scale_factor,
    );
    let (_, y, _, hauteur) = placee.carte;
    assert!(y + hauteur <= h - 500.0, "au-dessus du clavier");
    let ((x, by, bw, bh), _, _) = placee
        .boutons
        .iter()
        .find(|(_, _, r)| *r == Reponse::Oui)
        .expect("Créer")
        .clone();
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(
        f64::from(x + bw / 2.0),
        f64::from(by + bh / 2.0),
    ));
    app.handle_mouse_down(winit::event::MouseButton::Left, w, h);
    app.handle_mouse_up(winit::event::MouseButton::Left);
    assert!(
        app.ui.question.is_none(),
        "elle répond où elle est dessinée"
    );
}

/// **Le clavier qui sort remonte la ligne qu'on écrit** au-dessus de lui (CLAVIER-3) : en
/// bord à bord, la fenêtre ne rétrécit plus, c'est la marge du clavier qui le dit.
#[test]
fn test_bord_1_le_clavier_qui_sort_remonte_la_ligne() {
    use glucose_core::types::{Annotation, Viewport};
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    app.store.set_viewport(&board, vue);
    app.store
        .add_annotation(&board, Annotation::text("c", 0.0, 600.0, "en bas"));
    app.start_text_edit("c".into(), "en bas".into());
    app.adopter_les_marges(Marges {
        clavier: 500.0,
        ..Marges::default()
    });
    let vue = app.store.active_board().unwrap().viewport;
    let hauteur = f64::from(crate::renderer::card::text_box(400.0).line_height);
    let bas = 600.0 + f64::from(crate::renderer::card::TEXT_ORIGIN.1) + hauteur + vue.y;
    let plancher = f64::from(app.taille_de_la_fenetre().1) - 500.0;
    assert!(
        bas <= plancher + 1e-9,
        "la ligne au-dessus du clavier : {bas} > {plancher}"
    );
    assert_eq!(app.ui.marges.clavier, 500.0);
}

/// **Rien du message ni de la barre d'action ne se dessine sous la navigation** : l'écran
/// entier, rendu hors écran avec 300 pixels de navigation — les 300 rangées du bas sont les
/// mêmes avec et sans eux. Ils se dessinent pourtant : les deux images diffèrent ailleurs.
#[test]
fn test_bord_1_rien_du_message_ni_des_barres_sous_la_navigation() {
    let (w, h, bas) = (720u32, 1600u32, 300u32);
    let store = |choisie: bool| {
        let mut store = glucose_core::store::Store::new("bord");
        let board = store.project.active_board_id.clone();
        store.add_image(
            &board,
            glucose_core::types::BoardImage::new("a", 0.0, 0.0, 200.0, 200.0),
        );
        store.clear_selection();
        if choisie {
            store.select_image("a".into(), false);
        }
        store
    };
    let rendu = |store: &glucose_core::store::Store, message: bool| {
        let png = crate::bench::capture_with(store, w, h, |ui| {
            ui.scale_factor = 2.0;
            ui.marges.bas = bas as f32;
            if message {
                ui.show_toast("Un message qui doit rester au-dessus de la navigation");
            }
        });
        tiny_skia::Pixmap::decode_png(&png).expect("un PNG")
    };
    let sans = rendu(&store(false), false);
    let avec = rendu(&store(true), true);
    let rangee = (w * 4) as usize;
    let dessous = ((h - bas) as usize) * rangee;
    assert_eq!(
        &avec.data()[dessous..],
        &sans.data()[dessous..],
        "le message ou la barre d'action passe sous la navigation"
    );
    assert_ne!(avec.data(), sans.data(), "ils se dessinent bien, plus haut");
}
