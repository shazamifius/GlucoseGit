//! Ce que le geste promet (DETOURER-1 à 3, fiche 121) : il passe par la commande, attend son fil
//! sans geler, pose d'un seul geste des octets que le document gardera, et `Ctrl+Z` rend tout.

use super::*;
use crate::commandes::Commande;
use crate::renderer::magasin::Entree;
use crate::renderer::photo::Pyramide;
use glucose_core::types::BoardImage;
use std::time::{Duration, Instant};
use tiny_skia::Pixmap;

fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app.store.journal.clear();
    app
}

/// Une photo de 40 × 30 : un fond blanc, un carré noir au milieu, un pixel gris à son bord —
/// décodée dans le magasin sous ce nom, posée, et choisie.
fn poser_une_photo_sur_blanc(app: &mut GlucoseApp, id: &str, src: &str) {
    let mut pixmap = Pixmap::new(40, 30).expect("une image");
    for (i, p) in pixmap
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let (x, y) = (i % 40, i / 40);
        let v = match (x, y) {
            (15..25, 10..20) => 0,
            (25, 10..20) => 128,
            _ => 255,
        };
        p.copy_from_slice(&[v, v, v, 255]);
    }
    app.renderer.magasin.cache.insert(
        src.to_string(),
        Entree::pour_test(Pyramide::nouvelle(pixmap)),
    );
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new(id, 100.0, 100.0, 200.0, 150.0);
    img.src = Some(src.to_string());
    app.store.add_image(&board, img);
    app.store.set_selected_image_ids(vec![id.to_string()]);
}

fn src_de(app: &GlucoseApp, id: &str) -> String {
    let board = &app.store.project.active_board_id;
    app.store
        .image(board, id)
        .and_then(|i| i.src.clone())
        .expect("l'image et sa clé")
}

/// Le détourage avance jusqu'à sa pose — comme la boucle à chaque réveil —, borné dans le temps.
fn jusqu_a_la_pose(app: &mut GlucoseApp) {
    let depart = Instant::now();
    while app.attentes.detourage.is_some() {
        assert!(
            depart.elapsed() < Duration::from_secs(60),
            "le détourage ne finit pas"
        );
        app.poursuivre_le_detourage();
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// **Le fond uni part, d'un seul geste, et `Ctrl+Z` rend la photo d'origine.** Les octets posés
/// sont ceux que le document gardera (la promesse que le scribe attend) : un PNG dont le blanc
/// est transparent et le carré noir opaque.
#[test]
fn test_detourer_3_le_fond_uni_part_d_un_geste_et_ctrl_z_le_rend() {
    let mut app = app();
    poser_une_photo_sur_blanc(&mut app, "i", "blanc.png");
    let gestes = app.store.journal.depth();
    assert!(app.executer(Commande::RetirerLeFondUni));
    assert_eq!(app.current_cursor(), winit::window::CursorIcon::Progress);
    jusqu_a_la_pose(&mut app);
    assert_ne!(app.current_cursor(), winit::window::CursorIcon::Progress);
    let cle = src_de(&app, "i");
    assert!(
        cle.starts_with("detouree:"),
        "la photo porte ses octets neufs : {cle}"
    );
    assert_eq!(app.store.journal.depth(), gestes + 1, "un seul geste");
    app.renderer.magasin.attendre_le_chantier();
    let png = app
        .disque
        .objets
        .lire(&cle)
        .expect("les octets promis au scribe");
    let rvba = image::load_from_memory(&png).expect("un PNG").to_rgba8();
    assert_eq!(rvba.dimensions(), (40, 30));
    assert_eq!(rvba.get_pixel(0, 0)[3], 0, "le blanc est transparent");
    assert_eq!(rvba.get_pixel(20, 15).0, [0, 0, 0, 255], "le noir reste");
    let gris = rvba.get_pixel(25, 15).0;
    assert!(
        (gris[3] as i32 - 127).abs() <= 1 && gris[0] == 0,
        "le gris devient du noir à demi : {gris:?}"
    );
    app.store.undo();
    assert_eq!(
        src_de(&app, "i"),
        "blanc.png",
        "Ctrl+Z rend la photo d'origine"
    );
}

/// **Une photo qu'on a changée pendant le calcul n'est pas touchée** : le geste ne pose que ce
/// qu'il a calculé sur ce qui est encore là.
#[test]
fn test_detourer_1_une_photo_changee_pendant_le_calcul_garde_ce_qu_on_lui_a_fait() {
    let mut app = app();
    poser_une_photo_sur_blanc(&mut app, "i", "blanc.png");
    app.executer(Commande::RetirerLeFondUni);
    let board = app.store.project.active_board_id.clone();
    app.store
        .update_image(&board, "i", |img| img.src = Some("autre.png".into()));
    let gestes = app.store.journal.depth();
    jusqu_a_la_pose(&mut app);
    assert_eq!(src_de(&app, "i"), "autre.png");
    assert_eq!(app.store.journal.depth(), gestes, "aucun geste vide");
}

/// Une photo verrouillée ne se détoure pas ; sans photo choisie, la commande ne s'offre pas.
#[test]
fn test_detourer_1_ni_verrouillee_ni_sans_photo() {
    let mut app = app();
    assert!(!app.s_applique(Commande::Detourer));
    poser_une_photo_sur_blanc(&mut app, "i", "blanc.png");
    assert!(app.s_applique(Commande::Detourer));
    let board = app.store.project.active_board_id.clone();
    app.store.update_image(&board, "i", |img| img.locked = true);
    assert!(
        !app.s_applique(Commande::Detourer),
        "verrouillée, elle refuse qu'on la change"
    );
    app.executer(Commande::Detourer);
    assert!(app.attentes.detourage.is_none(), "rien à détourer");
    assert_eq!(src_de(&app, "i"), "blanc.png");
}

/// **Détourer, par le modèle** : la photo reçoit des octets neufs, de la taille de l'original.
#[test]
fn test_detourer_1_la_forme_principale_se_pose() {
    let mut app = app();
    poser_une_photo_sur_blanc(&mut app, "i", "blanc.png");
    app.executer(Commande::Detourer);
    jusqu_a_la_pose(&mut app);
    let cle = src_de(&app, "i");
    assert!(cle.starts_with("detouree:"));
    app.renderer.magasin.attendre_le_chantier();
    let png = app.disque.objets.lire(&cle).expect("les octets");
    let rvba = image::load_from_memory(&png).expect("un PNG").to_rgba8();
    assert_eq!(rvba.dimensions(), (40, 30));
    // Le carré noir sur blanc est la forme principale : plus opaque que le coin.
    assert!(rvba.get_pixel(20, 15)[3] > rvba.get_pixel(0, 0)[3]);
}

/// **Deux photos, un seul geste** : un `Ctrl+Z` les rend toutes les deux.
#[test]
fn test_detourer_1_deux_photos_un_seul_geste() {
    let mut app = app();
    poser_une_photo_sur_blanc(&mut app, "i", "blanc.png");
    poser_une_photo_sur_blanc(&mut app, "j", "blanc-2.png");
    app.store
        .set_selected_image_ids(vec!["i".to_string(), "j".to_string()]);
    let gestes = app.store.journal.depth();
    app.executer(Commande::RetirerLeFondUni);
    jusqu_a_la_pose(&mut app);
    assert!(
        src_de(&app, "i").starts_with("detouree:") && src_de(&app, "j").starts_with("detouree:")
    );
    assert_eq!(
        app.store.journal.depth(),
        gestes + 1,
        "un seul geste pour les deux"
    );
    app.store.undo();
    assert_eq!(
        (src_de(&app, "i"), src_de(&app, "j")),
        ("blanc.png".into(), "blanc-2.png".into())
    );
}

/// Les pixels du magasin sont prémultipliés : le détourage les relit droits.
#[test]
fn test_detourer_1_les_couleurs_reviennent_droites() {
    let mut p = vec![64, 32, 0, 128, 10, 20, 30, 255, 0, 0, 0, 0];
    depremultiplier(&mut p);
    assert_eq!(p, vec![128, 64, 0, 128, 10, 20, 30, 255, 0, 0, 0, 0]);
}

/// **Une photo à demi transparente** : le magasin la tient prémultipliée ; relue droite, un gris
/// à moitié opaque sur blanc devient du noir au quart (son opacité de moitié, gardée, fois la
/// moitié qu'il couvrait du fond). Relue telle quelle, il serait noir aux trois huitièmes.
#[test]
fn test_detourer_3_une_photo_a_demi_transparente_se_relit_droite() {
    let mut app = app();
    let mut pixmap = Pixmap::new(40, 30).expect("une image");
    pixmap.data_mut().fill(255);
    pixmap.data_mut()[(5 * 40 + 5) * 4..][..4].copy_from_slice(&[64, 64, 64, 128]);
    let entree = Entree::pour_test(Pyramide::nouvelle(pixmap));
    app.renderer
        .magasin
        .cache
        .insert("demi.png".to_string(), entree);
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("i", 100.0, 100.0, 200.0, 150.0);
    img.src = Some("demi.png".to_string());
    app.store.add_image(&board, img);
    app.store.set_selected_image_ids(vec!["i".to_string()]);
    app.executer(Commande::RetirerLeFondUni);
    jusqu_a_la_pose(&mut app);
    app.renderer.magasin.attendre_le_chantier();
    let png = app
        .disque
        .objets
        .lire(&src_de(&app, "i"))
        .expect("les octets");
    let p = image::load_from_memory(&png)
        .expect("un PNG")
        .to_rgba8()
        .get_pixel(5, 5)
        .0;
    assert!(p[0] == 0 && (i32::from(p[3]) - 64).abs() <= 1, "{p:?}");
}
