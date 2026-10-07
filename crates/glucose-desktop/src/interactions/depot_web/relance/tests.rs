//! DEPOT-WEB-6 : un dépôt replié en lien le dit, et se rattrape par le clic droit.
//!
//! Chaque épreuve passe par l'**entrée réelle** — le dépôt qui pose le lien, le clic sur
//! l'entrée du menu, le relevé de la boucle —, et la recherche est celle que l'épreuve fournit :
//! aucune n'atteint le réseau.

use super::*;
use crate::params::{Pointer, ScreenFrame};
use crate::plateforme::moisson::Recu;
use crate::ui::context_menu::{layout_context_menu, MenuAction, MenuRow};

const EPINGLE: &str = "https://fr.pinterest.com/pin/104427285106027754/";
const ECRAN: ScreenFrame = ScreenFrame {
    width: 1440.0,
    height: 900.0,
    header_h: 0.0,
    scale: 1.0,
};

fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app.store.journal.clear();
    app
}

/// Une recherche qui trouve : un PNG minuscule mais vrai.
fn trouve(_: &[String]) -> Result<Moisson, String> {
    let mut pixmap = tiny_skia::Pixmap::new(4, 3).expect("pixmap");
    pixmap.fill(tiny_skia::Color::from_rgba8(30, 200, 30, 255));
    let recu = Recu::nouveau("epingle.png", pixmap.encode_png().expect("png")).expect("un reçu");
    Ok(Moisson {
        recus: vec![recu],
        ..Moisson::default()
    })
}

/// Une recherche que le réseau laisse sans réponse.
fn muette(_: &[String]) -> Result<Moisson, String> {
    Err("fr.pinterest.com : délai dépassé (reponse)".into())
}

/// **Le repli d'un dépôt**, par le chemin du dépôt : la livraison d'un rapatriement raté,
/// avec sa raison. Rend l'identifiant du lien posé, choisi.
fn deposer_un_repli(app: &mut GlucoseApp) -> String {
    let numero = 41;
    app.recevoir_le_depot(Depot::EnChemin {
        numero,
        ou: None,
        hote: "fr.pinterest.com".into(),
    });
    app.recevoir_le_depot(Depot::Pose {
        numero: Some(numero),
        moisson: Moisson {
            liens: vec![EPINGLE.into()],
            echec: Some("fr.pinterest.com : délai dépassé (reponse)".into()),
            ..Moisson::default()
        },
    });
    let id = app.store.active_board().expect("un tableau").annotations[0]
        .id()
        .to_string();
    app.store.selected_annotation_ids = vec![id.clone()];
    id
}

/// Les entrées que le menu ouvert ici propose.
fn entrees(app: &GlucoseApp) -> Vec<MenuAction> {
    layout_context_menu(
        &app.store,
        &app.renderer.typography,
        ((40.0, 40.0), None),
        (ECRAN.width, ECRAN.height),
        ECRAN.scale,
    )
    .map(|m| {
        m.rows
            .iter()
            .filter_map(|r| match r {
                MenuRow::Item { action, .. } => Some(*action),
                MenuRow::Separator(_) => None,
            })
            .collect()
    })
    .unwrap_or_default()
}

/// **Ouvre le menu et clique sur l'entrée**, comme la main : au centre de sa ligne.
fn cliquer(app: &mut GlucoseApp, voulue: MenuAction) {
    app.ui.context_menu_at = Some((40.0, 40.0));
    let menu = layout_context_menu(
        &app.store,
        &app.renderer.typography,
        ((40.0, 40.0), None),
        (ECRAN.width, ECRAN.height),
        ECRAN.scale,
    )
    .expect("un menu");
    let (x, y, w, h) = menu
        .rows
        .iter()
        .find_map(|r| match r {
            MenuRow::Item { action, rect, .. } if *action == voulue => Some(*rect),
            _ => None,
        })
        .expect("l'entrée est proposée");
    let pointer = Pointer {
        x: x + w / 2.0,
        y: y + h / 2.0,
    };
    assert!(
        app.click_context_menu(pointer, ECRAN),
        "le menu prend le clic"
    );
}

/// La boucle relève ce qui est arrivé, jusqu'à ce que plus rien ne soit en chemin.
fn attendre_les_livraisons(app: &mut GlucoseApp) {
    let fin = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !app.depot.en_chemin.is_empty() {
        assert!(std::time::Instant::now() < fin, "la livraison n'arrive pas");
        std::thread::sleep(std::time::Duration::from_millis(2));
        app.relever_les_depots();
    }
}

fn nombre(app: &GlucoseApp) -> (usize, usize) {
    let b = app.store.active_board().expect("un tableau");
    (b.annotations.len(), b.images.len())
}

/// **Un dépôt replié en lien le dit** : le site, la raison, et le geste qui rattrape — là où le
/// compte-rendu annonçait « 1 élément posé », comme une réussite.
#[test]
fn test_un_depot_replie_en_lien_dit_pourquoi() {
    let mut app = app();
    deposer_un_repli(&mut app);
    let message = app.ui.toast_message().expect("un message").to_string();
    assert!(message.contains("Image introuvable"), "{message}");
    assert!(message.contains("délai dépassé"), "la raison : {message}");
    assert!(
        message.contains("Remplacer par l'image"),
        "le geste : {message}"
    );
    assert!(
        !message.contains("élément posé"),
        "pas une réussite : {message}"
    );
}

/// **Le menu propose l'entrée sur un lien seul, et jamais sur une note** : remplacer une note
/// par une image effacerait les mots de l'utilisateur.
#[test]
fn test_l_entree_n_existe_que_pour_un_lien_seul() {
    let mut app = app();
    deposer_un_repli(&mut app);
    assert!(entrees(&app).contains(&MenuAction::RemplacerParLImage));

    let board = app.store.project.active_board_id.clone();
    let note = Annotation::text(
        "note-1",
        0.0,
        0.0,
        format!("ma référence : {}", moisson::lien_markdown(EPINGLE)),
    );
    app.store.add_annotation(&board, note);
    app.store.selected_annotation_ids = vec!["note-1".into()];
    assert!(!entrees(&app).contains(&MenuAction::RemplacerParLImage));
}

/// **Par le menu, le lien laisse sa place à son image, en un seul geste** : l'image à son coin,
/// le marqueur parti, et un `Ctrl+Z` rend le lien.
#[test]
fn test_le_lien_se_remplace_par_son_image_en_un_geste() {
    let mut app = app();
    let id = deposer_un_repli(&mut app);
    let coin = match &app.store.active_board().expect("un tableau").annotations[0] {
        Annotation::Text { x, y, .. } => (*x, *y),
        _ => unreachable!(),
    };
    app.depot.relances.recherche = trouve;
    let gestes = app.store.undo_depth();

    cliquer(&mut app, MenuAction::RemplacerParLImage);
    assert_eq!(
        app.depot.en_chemin.len(),
        1,
        "le marqueur paraît au coin du lien"
    );
    assert_eq!(app.depot.en_chemin[0].monde, coin);
    attendre_les_livraisons(&mut app);

    assert_eq!(nombre(&app), (0, 1), "le lien est parti, l'image est là");
    let image = &app.store.active_board().expect("un tableau").images[0];
    assert_eq!((image.x, image.y), coin, "à la place du lien");
    assert_eq!(app.store.undo_depth(), gestes + 1, "un seul geste");

    assert!(app.store.undo());
    assert_eq!(nombre(&app), (1, 0), "Ctrl+Z rend le lien");
    assert_eq!(app.store.active_board().expect("").annotations[0].id(), id);
}

/// **Si l'image ne vient toujours pas, le lien reste, et le message dit pourquoi.**
#[test]
fn test_une_relance_qui_echoue_laisse_le_lien_et_le_dit() {
    let mut app = app();
    deposer_un_repli(&mut app);
    app.depot.relances.recherche = muette;
    let gestes = app.store.undo_depth();

    cliquer(&mut app, MenuAction::RemplacerParLImage);
    attendre_les_livraisons(&mut app);

    assert_eq!(nombre(&app), (1, 0), "le lien reste");
    assert_eq!(app.store.undo_depth(), gestes, "aucun geste");
    let message = app.ui.toast_message().expect("un message").to_string();
    assert!(message.contains("toujours introuvable"), "{message}");
    assert!(message.contains("délai dépassé"), "{message}");
}

/// **Un lien qu'on a modifié pendant la recherche ne se remplace pas** : ce sont désormais les
/// mots de l'utilisateur.
#[test]
fn test_un_lien_modifie_pendant_la_recherche_ne_se_remplace_pas() {
    let mut app = app();
    let id = deposer_un_repli(&mut app);
    app.depot.relances.recherche = trouve;
    cliquer(&mut app, MenuAction::RemplacerParLImage);
    let board = app.store.project.active_board_id.clone();
    app.store.ecrire_le_texte(&board, &id, "à revoir : ce lien");
    attendre_les_livraisons(&mut app);
    assert_eq!(nombre(&app), (1, 0), "la note reste, rien n'est posé");
}

/// **Une livraison attend la fin du geste de la main** : posée pendant un glisser, elle
/// refermerait ce glisser-là.
#[test]
fn test_une_livraison_attend_la_fin_du_geste() {
    let mut app = app();
    deposer_un_repli(&mut app);
    app.depot.relances.recherche = trouve;
    cliquer(&mut app, MenuAction::RemplacerParLImage);

    app.store.begin_live_edit();
    std::thread::sleep(std::time::Duration::from_millis(200));
    app.relever_les_depots();
    assert_eq!(
        app.depot.en_chemin.len(),
        1,
        "rien ne se pose pendant le geste"
    );
    assert!(app.store.in_live_edit(), "le geste de la main reste ouvert");
    app.store.end_live_edit();

    attendre_les_livraisons(&mut app);
    assert_eq!(nombre(&app), (0, 1), "posée une fois la main levée");
}
