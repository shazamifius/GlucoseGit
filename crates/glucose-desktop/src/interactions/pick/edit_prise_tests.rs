//! **EDIT-PRISE — en écriture, l'appui dans le texte qu'on écrit est au texte** (sa note du
//! 23/11, `text-kpll-370` : *« parfois on ne peut pas sélectionner à la souris une partie du
//! texte, car… ça quitte le bloc texte carrément »*).
//!
//! Trois voleurs, joués par le vrai chemin de la souris : la partie d'un texte qui a grandi
//! pendant qu'on l'écrit (l'arbitre ne connaît que la boîte du document), une flèche qui passe
//! dessus (rang 20 contre 40), une poignée de son bord (sa prise entre dans le texte). Chaque
//! épreuve vérifie d'abord que l'ancienne loi donnait l'appui à un autre.

use crate::app::GlucoseApp;
use crate::canvas::world_to_screen;
use glucose_core::geometry::{Point, Rect};
use glucose_core::hit_priority::{PickKind, PICK_RANK_TEXT};
use glucose_core::types::Annotation;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Un texte de trois lignes en `(400, 300)`, ouvert à la saisie, vu à l'échelle 1.
fn qui_ecrit(fleche: Option<Annotation>) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.images.clear();
        b.annotations.clear();
        b.viewport.x = 0.0;
        b.viewport.y = 0.0;
        b.viewport.scale = 1.0;
    }
    let texte = "un\ndeux\ntrois";
    let mut carte = Annotation::text("carte", 400.0, 300.0, texte);
    if let Annotation::Text { height, .. } = &mut carte {
        *height = Some(120.0);
    }
    app.store.add_annotation(&board, carte);
    if let Some(f) = fleche {
        app.store.add_annotation(&board, f);
    }
    app.store.clear_selection();
    app.store.journal.clear();
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    app.start_text_edit("carte".to_string(), texte.to_string());
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    app
}

fn appuyer(app: &mut GlucoseApp, (wx, wy): (f64, f64)) {
    let vp = app.store.viewport();
    let (x, y) = world_to_screen(wx, wy, &vp);
    app.handle_cursor_moved(PhysicalPosition::new(x, y));
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
}

/// La boîte du texte dans le document, et celle qu'il montre pendant qu'on l'écrit.
fn boites(app: &GlucoseApp) -> (Rect, Rect) {
    let board = app.store.active_board().expect("un tableau");
    let carte = board.annotations.iter().find(|a| a.id() == "carte");
    let document = carte.and_then(Annotation::rect).expect("la carte");
    let montree = app
        .renderer
        .boite_vivante
        .clone()
        .expect("la boîte montrée");
    (document, montree.1)
}

/// L'appui est allé au texte : la saisie continue, un glisser de sélection commence, rien
/// d'autre n'est choisi ni pris.
fn l_appui_est_au_texte(app: &GlucoseApp) {
    let saisie = app.editing_session.as_ref().map(|s| s.ann_id.as_str());
    assert_eq!(saisie, Some("carte"), "l'appui a validé la saisie");
    assert!(
        app.text_drag.is_some(),
        "aucun glisser de sélection ne commence"
    );
    assert!(
        app.resize_session.is_none(),
        "l'appui redimensionne le texte"
    );
    assert!(!app.is_dragging_item, "l'appui emporte un nœud");
}

/// **Le texte qui a grandi** : huit lignes tapées, l'appui dans la partie neuve — que le
/// document ne connaît pas encore — pose le curseur dans la ligne visée.
#[test]
fn test_edit_prise_la_partie_qui_a_grandi_est_au_texte() {
    let mut app = qui_ecrit(None);
    if let Some(s) = app.editing_session.as_mut() {
        s.buffer = "un\ndeux\ntrois\nquatre\ncinq\nsix\nsept\nhuit\nneuf\ndix".to_string();
    }
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    let (document, montree) = boites(&app);
    let ici = (montree.left + 30.0, montree.bottom() - 12.0);
    assert!(
        ici.1 > document.bottom() + 20.0,
        "le texte n'a pas grandi hors de sa boîte : l'épreuve ne distinguerait rien"
    );
    // L'envers : l'arbitre seul ne voit pas le texte à cet endroit — l'appui validait la
    // saisie et traçait un cadre de sélection.
    assert!(
        app.pick_candidates_at(ici.0, ici.1)
            .iter()
            .all(|c| c.id != "carte"),
        "l'arbitre connaît déjà la partie neuve : l'épreuve ne distinguerait rien"
    );

    appuyer(&mut app, ici);
    l_appui_est_au_texte(&app);
    let tete = app.editing_session.as_ref().map(|s| s.selection.head);
    assert!(
        tete.is_some_and(|t| t > "un\ndeux\ntrois\nquatre".len()),
        "le curseur n'est pas dans les dernières lignes : {tete:?}"
    );
}

/// **La flèche qui passe dessus** : une flèche libre dont le bout tombe dans le texte (elle
/// contourne les nœuds qu'elle traverse, pas celui où elle arrive) ; l'appui sur son trait,
/// dans le texte, est au texte.
#[test]
fn test_edit_prise_une_fleche_par_dessus_ne_vole_pas_l_appui() {
    let mut app = qui_ecrit(Some(Annotation::arrow("f", 100.0, 340.0, 470.0, 340.0)));
    let (_, montree) = boites(&app);
    let ici = (montree.left + 40.0, 340.0);
    assert!(montree.contains_point(Point::new(ici.0, ici.1)));
    // L'envers : le trait de la flèche passe devant le texte.
    let candidats = app.pick_candidates_at(ici.0, ici.1);
    assert!(
        candidats
            .iter()
            .any(|c| c.id == "f" && c.rank < PICK_RANK_TEXT),
        "la flèche ne passait pas devant le texte : l'épreuve ne distinguerait rien ({candidats:?})"
    );

    appuyer(&mut app, ici);
    l_appui_est_au_texte(&app);
    let choisis = &app.store.selected_annotation_ids;
    assert!(!choisis.contains(&"f".to_string()), "la flèche est choisie");
}

/// **La poignée du bord** : le texte choisi, l'appui juste dans son coin — dans la prise de la
/// poignée — est au texte ; la poignée se prend du dehors.
#[test]
fn test_edit_prise_une_poignee_ne_vole_pas_l_appui_dans_le_texte() {
    let mut app = qui_ecrit(None);
    app.store.select_annotation("carte".to_string(), false);
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    let (_, montree) = boites(&app);
    let dedans = (montree.right() - 3.0, montree.bottom() - 3.0);
    // L'envers : la prise de la poignée entre dans le texte.
    let prise = app.pick_candidate_at(dedans.0, dedans.1);
    assert!(
        prise.is_some_and(|c| c.kind == PickKind::Handle),
        "la poignée ne prenait pas ce point : l'épreuve ne distinguerait rien"
    );

    appuyer(&mut app, dedans);
    l_appui_est_au_texte(&app);
}

/// **Du dehors, la poignée se prend toujours** : l'appui juste hors du coin redimensionne.
#[test]
fn test_edit_prise_la_poignee_se_prend_du_dehors() {
    let mut app = qui_ecrit(None);
    app.store.select_annotation("carte".to_string(), false);
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    let (_, montree) = boites(&app);
    appuyer(&mut app, (montree.right() + 3.0, montree.bottom() + 3.0));
    assert!(
        app.resize_session.is_some(),
        "la poignée ne se prend plus du dehors"
    );
}

/// **La boîte d'un texte quitté ne prête pas l'appui au suivant** : entre deux images, la boîte
/// vivante est encore celle du texte qu'on vient de quitter ; un appui sur lui va à lui (la saisie
/// du second se valide), jamais au texte qu'on vient d'ouvrir ailleurs.
#[test]
fn test_edit_prise_la_boite_d_un_texte_quitte_ne_prete_pas_l_appui() {
    let mut app = qui_ecrit(None);
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("autre", 400.0, 700.0, "loin"));
    let (_, montree) = boites(&app);
    app.commit_editing();
    app.start_text_edit("autre".to_string(), "loin".to_string());
    // Aucune image : la boîte vivante est encore celle de « carte ».
    let vivante = app
        .renderer
        .boite_vivante
        .as_ref()
        .map(|(id, _)| id.as_str());
    assert_eq!(vivante, Some("carte"), "l'épreuve ne distinguerait rien");

    appuyer(&mut app, (montree.left + 30.0, montree.top + 20.0));
    assert!(
        app.editing_session.is_none(),
        "l'appui sur le texte quitté est allé au texte qu'on écrit ailleurs"
    );
    assert!(app
        .store
        .selected_annotation_ids
        .contains(&"carte".to_string()));
}
