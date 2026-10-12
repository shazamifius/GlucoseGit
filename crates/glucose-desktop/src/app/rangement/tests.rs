//! ORDONNER-2 — ranger par la couleur, de bout en bout : la couleur de chaque image, l'attente
//! de celles qui ne sont pas lues, un seul geste à annuler. PRESET-1 — « Par slot preset » :
//! chaque zone de gabarit range ses images en elle-même.

use super::*;
use crate::dock::{LayoutMode, SortType};
use crate::renderer::magasin::Entree;
use crate::renderer::photo::Pyramide;
use glucose_core::store::zones::{BANDEAU, MARGE};
use glucose_core::types::BoardZone;
use tiny_skia::Pixmap;

/// Une application sans rien dessus.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app.store.journal.clear();
    app
}

/// Une photo unie, décodée dans le magasin sous ce nom.
fn decoder_unie(app: &mut GlucoseApp, src: &str, [r, g, b]: [u8; 3]) {
    let mut pixmap = Pixmap::new(16, 9).expect("une image");
    for bloc in pixmap.data_mut().as_chunks_mut::<4>().0 {
        *bloc = [r, g, b, 255];
    }
    app.renderer.magasin.cache.insert(
        src.to_string(),
        Entree::pour_test(Pyramide::nouvelle(pixmap)),
    );
}

/// Pose une image carrée qui porte ce fichier, sur une rangée, à l'abscisse `x`.
fn poser(app: &mut GlucoseApp, id: &str, src: &str, x: f64) {
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new(id, x, 0.0, 200.0, 200.0);
    img.original_width = 200.0;
    img.original_height = 200.0;
    img.src = Some(src.to_string());
    app.store.add_image(&board, img);
    // Poser choisit l'image ; sans choix, Ordonner range tout le tableau.
    app.store.selected_image_ids.clear();
}

/// Une seule rangée de grille, dans l'ordre qu'on lui donne.
fn en_une_rangee(sort_by: SortType, combien: usize) -> OrganizeState {
    OrganizeState {
        sort_by,
        layout: LayoutMode::Grid,
        size: 200.0,
        gap: 10.0,
        cols: combien,
    }
}

/// Les images, de gauche à droite.
fn de_gauche_a_droite(app: &GlucoseApp) -> Vec<String> {
    let mut images = app.store.active_board().expect("un tableau").images.clone();
    images.sort_by(|a, b| a.x.total_cmp(&b.x));
    images.into_iter().map(|i| i.id).collect()
}

/// Quatre images aux couleurs connues, posées dans le désordre.
fn quatre_couleurs() -> GlucoseApp {
    let mut app = app();
    for (k, (id, rgb)) in [
        ("gris", [128, 128, 128]),
        ("jaune", [255, 255, 0]),
        ("rouge", [255, 0, 0]),
        ("bleu", [0, 0, 255]),
    ]
    .into_iter()
    .enumerate()
    {
        let src = format!("{id}.png");
        decoder_unie(&mut app, &src, rgb);
        poser(&mut app, id, &src, k as f64 * 300.0);
    }
    app.store.journal.clear();
    app
}

/// **Par teinte** : l'arc-en-ciel coupé au plus grand vide — du bleu au rouge par le magenta,
/// puis le jaune —, et le gris après les teintes. **Par clarté** : le bleu est le plus sombre
/// et le jaune le plus clair, quoi qu'en dise HSL.
#[test]
fn test_ordonner_2_les_images_se_rangent_par_leur_couleur() {
    let attendus = [
        (SortType::Color, ["bleu", "rouge", "jaune", "gris"]),
        (SortType::LumAsc, ["bleu", "gris", "rouge", "jaune"]),
        (SortType::LumDesc, ["jaune", "rouge", "gris", "bleu"]),
    ];
    for (tri, attendu) in attendus {
        let mut app = quatre_couleurs();
        app.apply_dock_layout(&en_une_rangee(tri, 4));
        assert!(
            app.attentes.rangement.is_none(),
            "tout était lu : rien n'attend"
        );
        assert_eq!(de_gauche_a_droite(&app), attendu, "{tri:?}");
    }
}

/// **Un seul geste** : un `Ctrl+Z` rend toutes les places d'un coup.
#[test]
fn test_ordonner_2_un_rangement_s_annule_d_un_coup() {
    let mut app = quatre_couleurs();
    let avant = de_gauche_a_droite(&app);
    app.apply_dock_layout(&en_une_rangee(SortType::Color, 4));
    assert_ne!(de_gauche_a_droite(&app), avant);
    assert!(app.store.undo(), "il y a quelque chose à annuler");
    assert_eq!(de_gauche_a_droite(&app), avant);
    assert!(!app.store.undo(), "un seul geste : une seule entrée");
}

/// **Une image qui ne se lira jamais n'a pas de couleur** : elle va à la fin, et le
/// compte-rendu le dit — Glucose Tauri lui donnait une luminosité de 0,5 en silence.
#[test]
fn test_ordonner_2_une_image_illisible_va_a_la_fin_et_se_dit() {
    let mut app = quatre_couleurs();
    poser(&mut app, "perdue", "perdue.png", -300.0);
    app.renderer.magasin.echecs.insert("perdue.png".to_string());
    app.apply_dock_layout(&en_une_rangee(SortType::LumAsc, 5));
    assert_eq!(
        de_gauche_a_droite(&app),
        ["bleu", "gris", "rouge", "jaune", "perdue"]
    );
    let toast = app.ui.current_toast.as_ref().expect("un compte-rendu");
    assert!(
        toast.message.contains("1 sans couleur"),
        "{}",
        toast.message
    );
}

/// **Une couleur qui n'est pas encore lue est attendue, sans geler** : rien ne bouge tant que
/// l'atelier décode le fichier, et le rangement s'applique de lui-même, d'un bloc, à l'image
/// où la couleur arrive — le réveil le fait repasser.
#[test]
fn test_ordonner_2_le_rangement_attend_une_couleur_en_chemin() {
    let dossier = crate::epreuve::Dossier::nouveau("ordonner-2");
    let fichier = dossier.join("violet.png");
    image::RgbaImage::from_pixel(12, 8, image::Rgba([120, 0, 200, 255]))
        .save(&fichier)
        .expect("une image sur le disque");
    let src = fichier.to_string_lossy().into_owned();

    let mut app = quatre_couleurs();
    // Loin hors de l'écran : seul le rangement peut le demander à l'atelier — à l'écran, le
    // dessin l'aurait demandé de lui-même, et caché un rangement qui ne demande rien.
    poser(&mut app, "violet", &src, 100_000.0);
    app.store.journal.clear();
    // Le mot d'accueil réveillerait l'application de lui-même, et cacherait un rangement qui ne
    // la réveille pas.
    app.ui.current_toast = None;
    let avant = de_gauche_a_droite(&app);

    app.apply_dock_layout(&en_une_rangee(SortType::Color, 5));
    assert!(app.attentes.rangement.is_some(), "le violet n'est pas lu");
    assert_eq!(de_gauche_a_droite(&app), avant, "rien ne bouge avant");

    for _ in 0..1000 {
        if app.attentes.rangement.is_none() || app.prochain_reveil().is_none() {
            break;
        }
        app.une_image_sans_fenetre((800, 600));
        app.renderer.magasin.attendre_le_chantier();
    }
    assert!(
        app.attentes.rangement.is_none(),
        "le rangement attend encore : plus rien ne l'a fait repasser"
    );
    assert_eq!(
        de_gauche_a_droite(&app),
        ["bleu", "violet", "rouge", "jaune", "gris"]
    );
    assert!(app.store.undo(), "il y a quelque chose à annuler");
    assert_eq!(de_gauche_a_droite(&app), avant);
    assert!(!app.store.undo(), "un seul geste : une seule entrée");
}

/// « Par slot preset », avec ce tri.
fn par_zone(sort_by: SortType) -> OrganizeState {
    OrganizeState {
        sort_by,
        layout: LayoutMode::BySlot,
        size: 200.0,
        gap: 10.0,
        cols: 0,
    }
}

fn boite(app: &GlucoseApp, id: &str) -> (f64, f64, f64, f64) {
    let board = app.store.project.active_board_id.clone();
    let i = app.store.image(&board, id).expect("l'image");
    (i.x, i.y, i.width, i.height)
}

/// **Chaque zone range ses images en elle-même**, sous son bandeau, dans l'ordre de la couleur ;
/// l'image hors de toute zone ne bouge pas ; un seul `Ctrl+Z` rend tout.
#[test]
fn test_preset_1_par_slot_preset_range_chaque_zone() {
    let mut app = quatre_couleurs();
    poser(&mut app, "dehors", "gris.png", 5000.0);
    let board = app.store.project.active_board_id.clone();
    app.store.poser_les_zones(
        &board,
        vec![
            BoardZone::new("g/gauche", -150.0, -150.0, 600.0, 400.0),
            BoardZone::new("g/droite", 500.0, -150.0, 600.0, 400.0),
        ],
    );
    app.store.journal.clear();
    let dehors = boite(&app, "dehors");

    app.apply_dock_layout(&par_zone(SortType::LumAsc));
    // Gris et jaune (en 0 et 300) dans la première ; rouge et bleu (600, 900) dans la seconde.
    let (gris, jaune) = (boite(&app, "gris"), boite(&app, "jaune"));
    let (rouge, bleu) = (boite(&app, "rouge"), boite(&app, "bleu"));
    let haut = -150.0 + BANDEAU + MARGE;
    assert_eq!(
        gris.0 - gris.2 / 2.0,
        -150.0 + MARGE,
        "le gris ouvre sa zone"
    );
    assert!(jaune.0 > gris.0, "le gris est plus sombre que le jaune");
    assert_eq!(
        bleu.0 - bleu.2 / 2.0,
        500.0 + MARGE,
        "le bleu ouvre la sienne"
    );
    assert!(rouge.0 > bleu.0, "le bleu est plus sombre que le rouge");
    for b in [gris, jaune, rouge, bleu] {
        assert_eq!(b.1 - b.3 / 2.0, haut, "sous le bandeau");
    }
    assert_eq!(
        boite(&app, "dehors"),
        dehors,
        "hors des zones, rien ne bouge"
    );
    let toast = app.ui.current_toast.as_ref().expect("un compte-rendu");
    assert!(
        toast.message.starts_with("Par slot preset"),
        "{}",
        toast.message
    );

    assert!(app.store.undo(), "il y a quelque chose à annuler");
    assert_eq!(boite(&app, "rouge").0, 600.0);
    assert!(!app.store.undo(), "un seul geste : une seule entrée");
}

/// **Sans zone, rien ne bouge, et on dit pourquoi** — et rien n'entre au journal.
#[test]
fn test_preset_1_par_slot_preset_sans_zone_le_dit() {
    let mut app = quatre_couleurs();
    let avant = de_gauche_a_droite(&app);
    let rouge = boite(&app, "rouge");
    app.apply_dock_layout(&par_zone(SortType::None));
    assert_eq!(de_gauche_a_droite(&app), avant);
    assert_eq!(boite(&app, "rouge"), rouge);
    let toast = app.ui.current_toast.as_ref().expect("un mot");
    assert!(toast.message.contains("Aucune zone"), "{}", toast.message);
    assert!(!app.store.undo(), "rien à annuler");
}
