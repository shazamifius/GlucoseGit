//! La copie d'abord, l'original ensuite (fiche 53 § 9), par le chemin des livraisons : une
//! copie annoncée puis posée, puis l'original qui la remplace.

use super::*;
use crate::plateforme::moisson::{Depot, Moisson};

fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app.store.journal.clear();
    app
}

/// Un PNG vrai, de ces dimensions.
fn png(nom: &str, (w, h): (u32, u32)) -> Recu {
    let mut pixmap = tiny_skia::Pixmap::new(w, h).expect("pixmap");
    pixmap.fill(tiny_skia::Color::from_rgba8(30, 200, 30, 255));
    Recu::nouveau(nom, pixmap.encode_png().expect("png")).expect("un reçu")
}

/// Une copie annoncée sous ce numéro, puis livrée en aperçu : rend son nœud.
fn poser_une_copie(app: &mut GlucoseApp, numero: u64, dimensions: (u32, u32)) -> String {
    app.recevoir_le_depot(Depot::EnChemin {
        numero,
        ou: None,
        hote: "i.pinimg.com".into(),
    });
    app.recevoir_le_depot(Depot::Pose {
        numero: Some(numero),
        moisson: Moisson {
            recus: vec![png("236x.png", dimensions)],
            apercu: true,
            ..Moisson::default()
        },
    });
    let images = &app.store.active_board().expect("un tableau").images;
    assert_eq!(images.len(), 1, "la copie est posée tout de suite");
    images[0].id.clone()
}

fn la_seule(app: &GlucoseApp) -> glucose_core::types::BoardImage {
    app.store.active_board().expect("un tableau").images[0].clone()
}

/// **La copie se pose à la taille qu'aura l'original, et l'original prend sa place sans que
/// rien ne bouge** : même nœud, même endroit, même taille — la source et les dimensions
/// d'origine seules changent.
#[test]
fn test_l_original_prend_la_place_de_sa_copie_sans_rien_bouger() {
    let mut app = application();
    let id = poser_une_copie(&mut app, 7, (236, 354));
    let copie = la_seule(&app);
    assert_eq!(
        (copie.width, copie.height),
        (400.0, 600.0),
        "la taille de l'original"
    );

    app.recevoir_le_depot(Depot::Ameliore {
        numero: 7,
        recu: Some(png("original.png", (1200, 1800))),
    });
    let original = la_seule(&app);
    assert_eq!(original.id, id, "le même nœud");
    assert_eq!(
        (original.x, original.y),
        (copie.x, copie.y),
        "au même endroit"
    );
    assert_eq!(
        (original.width, original.height),
        (400.0, 600.0),
        "à la même taille"
    );
    assert_eq!(
        (original.original_width, original.original_height),
        (1200.0, 1800.0)
    );
    assert_ne!(original.src, copie.src, "la source de l'original");
    assert!(app.depot.copies.is_empty(), "l'attente est close");
}

/// **Un original plus petit que la copie ne le présumait se pose à sa vraie taille**, et un
/// nœud que l'utilisateur a agrandi entre-temps garde la sienne.
#[test]
fn test_la_taille_suit_l_original_sauf_si_l_utilisateur_l_a_changee() {
    let mut app = application();
    poser_une_copie(&mut app, 1, (236, 354));
    app.recevoir_le_depot(Depot::Ameliore {
        numero: 1,
        recu: Some(png("o.png", (300, 450))),
    });
    let petit = la_seule(&app);
    assert_eq!((petit.width, petit.height), (300.0, 450.0));

    let mut app = application();
    let id = poser_une_copie(&mut app, 2, (236, 354));
    let board = app.store.project.active_board_id.clone();
    app.store
        .update_image(&board, &id, |i| (i.width, i.height) = (800.0, 1200.0));
    app.recevoir_le_depot(Depot::Ameliore {
        numero: 2,
        recu: Some(png("o.png", (1200, 1800))),
    });
    let garde = la_seule(&app);
    assert_eq!(
        (garde.width, garde.height),
        (800.0, 1200.0),
        "sa taille à lui"
    );
    assert_eq!(garde.original_width, 1200.0, "mais l'original");
}

/// **Une copie effacée ne revient pas**, et une copie déjà la meilleure reste telle quelle :
/// dans les deux cas, l'attente se clôt.
#[test]
fn test_une_copie_effacee_ne_revient_pas_et_la_meilleure_reste() {
    let mut app = application();
    let id = poser_une_copie(&mut app, 3, (236, 354));
    let board = app.store.project.active_board_id.clone();
    app.store.remove_images(&board, &[id.as_str()]);
    app.recevoir_le_depot(Depot::Ameliore {
        numero: 3,
        recu: Some(png("o.png", (1200, 1800))),
    });
    assert!(app.store.active_board().expect("").images.is_empty());

    let mut app = application();
    poser_une_copie(&mut app, 4, (736, 1104));
    let avant = la_seule(&app);
    app.recevoir_le_depot(Depot::Ameliore {
        numero: 4,
        recu: None,
    });
    assert_eq!(la_seule(&app), avant);
    assert!(app.depot.copies.is_empty());
}
