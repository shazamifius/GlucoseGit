//! EXPORT-PNG-1 — l'export en image, de bout en bout : ce qu'il porte, ce qu'il tait, et des
//! bandes qui se recousent sans couture.

use super::*;
use crate::epreuve::Dossier;
use glucose_core::types::BoardImage;
use std::time::{Duration, Instant};

/// Une application sans rien dessus.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app
}

/// Une photo unie de 100 × 100, écrite dans le dossier, posée à sa taille, centrée en `x`.
fn poser(app: &mut GlucoseApp, dossier: &Dossier, id: &str, rvb: [u8; 3], x: f64) {
    let fichier = dossier.join(format!("{id}.png"));
    let [r, v, b] = rvb;
    image::RgbaImage::from_pixel(100, 100, image::Rgba([r, v, b, 255]))
        .save(&fichier)
        .expect("une photo d'épreuve");
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new(id, x, 0.0, 100.0, 100.0);
    img.original_width = 100.0;
    img.original_height = 100.0;
    img.src = Some(fichier.to_string_lossy().into_owned());
    app.store.add_image(&board, img);
    app.store.clear_selection();
}

/// Rouge, vert et bleu sur une rangée, cinquante unités entre chacune.
fn trois_photos(dossier: &Dossier) -> GlucoseApp {
    let mut app = app();
    poser(&mut app, dossier, "rouge", [220, 30, 30], 0.0);
    poser(&mut app, dossier, "vert", [30, 200, 60], 150.0);
    poser(&mut app, dossier, "bleu", [40, 60, 230], 300.0);
    app
}

/// Attend que l'export ait fini, en relevant ce que son fil dit, comme la boucle.
fn attendre(app: &mut GlucoseApp) {
    let debut = Instant::now();
    while app.attentes.export.is_some() {
        assert!(
            debut.elapsed() < Duration::from_secs(120),
            "l'export ne finit pas"
        );
        app.suivre_l_export();
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// L'image écrite, relue.
fn relire(chemin: &Path) -> image::RgbImage {
    image::open(chemin).expect("une image lisible").to_rgb8()
}

/// Le pixel de l'image où tombe ce point du monde.
fn pixel(image: &image::RgbImage, scene: &Scene, (x, y): (f64, f64)) -> [u8; 3] {
    let px = (scene.vue.x + x * scene.vue.scale) as u32;
    let py = (scene.vue.y + y * scene.vue.scale) as u32;
    image.get_pixel(px, py).0
}

/// La couleur du canevas : celle de tout ce qui n'est pas du contenu.
fn fond(app: &GlucoseApp) -> [u8; 3] {
    let c = app.renderer.theme.bg_canvas.to_color_u8();
    [c.red(), c.green(), c.blue()]
}

/// **La sélection seule, dans sa disposition** : le rouge et le bleu choisis, le vert entre eux
/// ne l'est pas — l'image porte le rouge et le bleu à leur place, et le fond où était le vert.
/// Ni grille de points, ni cadre de sélection : le fond est uni partout où il n'y a rien.
#[test]
fn test_export_png_1_la_selection_seule_dans_sa_disposition() {
    let dossier = Dossier::nouveau("export-png-selection");
    let mut app = trois_photos(&dossier);
    // Une carte de texte et un dossier entre les photos, non choisis : ils n'y sont pas non plus.
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.push(glucose_core::types::Annotation::text(
            "t",
            190.0,
            -20.0,
            "Hors du choix",
        ));
        let mut d = glucose_core::types::CanvasFolder::new("d", "Hors du choix", "b-d");
        (d.x, d.y, d.width, d.height) = (95.0, -10.0, 20.0, 20.0);
        b.folders.push(d);
    }
    app.store.selected_image_ids = vec!["rouge".into(), "bleu".into()];
    let scene = preparer(&app.store).expect("une sélection");
    assert_eq!((scene.largeur, scene.hauteur), (400, 100), "bord à bord");
    let chemin = dossier.join("selection.png");
    app.exporter_en_image(chemin.clone());
    attendre(&mut app);

    let image = relire(&chemin);
    assert_eq!(
        (image.width(), image.height()),
        (scene.largeur, scene.hauteur)
    );
    assert_eq!(pixel(&image, &scene, (0.0, 0.0)), [220, 30, 30], "le rouge");
    assert_eq!(
        pixel(&image, &scene, (300.0, 0.0)),
        [40, 60, 230],
        "le bleu"
    );
    assert_eq!(
        pixel(&image, &scene, (150.0, 0.0)),
        fond(&app),
        "le vert n'y est pas"
    );
    assert_eq!(
        pixel(&image, &scene, (105.0, 0.0)),
        fond(&app),
        "le dossier non plus"
    );
    assert_eq!(
        pixel(&image, &scene, (200.0, -10.0)),
        fond(&app),
        "la carte non plus"
    );
    // Entre deux photos, sur toute la hauteur d'une photo : rien que le fond.
    for y in -45..45 {
        for x in [60.0, 75.0, 90.0, 210.0, 240.0] {
            let p = pixel(&image, &scene, (x, f64::from(y)));
            assert_eq!(p, fond(&app), "un repère ou un cadre en ({x}, {y})");
        }
    }
    let toast = app.ui.current_toast.as_ref().expect("un compte-rendu");
    assert!(
        toast.message.contains("selection.png exporté"),
        "{}",
        toast.message
    );
}

/// **Sans sélection, tout le tableau** : les trois photos — de -50 à 350, quatre cents unités —,
/// bord à bord : une photo ne peint rien hors de sa boîte. Une carte de texte en plus, et la
/// marge devient la portée exacte de sa lueur, de chaque côté.
#[test]
fn test_export_png_1_sans_selection_tout_le_tableau() {
    let dossier = Dossier::nouveau("export-png-tout");
    let mut app = trois_photos(&dossier);
    let scene = preparer(&app.store).expect("un tableau");
    assert_eq!((scene.largeur, scene.hauteur), (400, 100), "bord à bord");
    // Par le chemin du bouton, une fois le nom choisi : le nom dit l'image.
    let chemin = dossier.join("tout.png");
    app.exporter_vers(chemin.clone());
    attendre(&mut app);

    let image = relire(&chemin);
    assert_eq!(pixel(&image, &scene, (0.0, 0.0)), [220, 30, 30]);
    assert_eq!(pixel(&image, &scene, (150.0, 0.0)), [30, 200, 60]);
    assert_eq!(pixel(&image, &scene, (300.0, 0.0)), [40, 60, 230]);

    let board = app.store.project.active_board_id.clone();
    let carte = glucose_core::types::Annotation::text("t", 0.0, 200.0, "Une légende");
    let bas = carte.bounds().top + carte.bounds().height;
    app.store.add_annotation(&board, carte);
    app.store.clear_selection();
    let marge = crate::renderer::export::debord(1.0);
    assert!(marge > 0.0);
    let scene = preparer(&app.store).expect("un tableau");
    assert_eq!(scene.largeur, (400.0 + 2.0 * marge).ceil() as u32);
    assert_eq!(scene.hauteur, ((bas + 50.0) + 2.0 * marge).ceil() as u32);
}

/// **Les bandes se recousent sans couture** : la même scène, écrite d'une seule bande ou par
/// bandes de sept rangées, donne les mêmes pixels, au bit près.
#[test]
fn test_export_png_1_les_bandes_se_recousent_sans_couture() {
    let dossier = Dossier::nouveau("export-png-bandes");
    let mut app = trois_photos(&dossier);
    // Des rangées qui diffèrent : trois photos unies alignées donnent des rangées toutes
    // pareilles, et une bande recopiée partout ne se verrait pas. Une carte dessous, et sa lueur.
    let board = app.store.project.active_board_id.clone();
    app.store.add_annotation(
        &board,
        glucose_core::types::Annotation::text("t", 20.0, 90.0, "Une légende\nsur deux lignes"),
    );
    app.store.clear_selection();
    let mut images = Vec::new();
    for (nom, rangees) in [("une.png", None), ("bandes.png", Some(7u64))] {
        let scene = preparer(&app.store).expect("un tableau");
        let surface = rangees.map_or(u64::MAX, |r| r * u64::from(scene.largeur));
        let peintre = Peintre {
            theme: app.renderer.theme.clone(),
            atelier: app.renderer.magasin.atelier.a_partager(),
        };
        let chemin = dossier.join(nom);
        let en_cours = lancer(scene, peintre, chemin.clone(), surface, None);
        let debut = Instant::now();
        let fini = loop {
            assert!(debut.elapsed() < Duration::from_secs(120), "ne finit pas");
            if let Some(Nouvelle::Fini(f)) = en_cours.nouvelles().into_iter().last() {
                break f;
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        assert!(fini.is_ok(), "{fini:?}");
        images.push(relire(&chemin));
    }
    assert!(images[0] == images[1], "les bandes ont une couture");
}

/// **Un export qui échoue ne laisse rien**, et le dit : un dossier qui n'existe pas.
#[test]
fn test_export_png_1_un_echec_ne_laisse_rien_et_se_dit() {
    let dossier = Dossier::nouveau("export-png-echec");
    let mut app = trois_photos(&dossier);
    let chemin = dossier.join("n-existe-pas").join("image.png");
    app.exporter_en_image(chemin.clone());
    attendre(&mut app);
    assert!(!chemin.exists());
    let toast = app.ui.current_toast.as_ref().expect("un compte-rendu");
    assert!(
        toast.message.contains("n'a pas pu s'écrire"),
        "{}",
        toast.message
    );
}

/// Un tableau vide n'a rien à exporter, et le dit.
#[test]
fn test_export_png_1_un_tableau_vide_le_dit() {
    let mut app = app();
    assert!(preparer(&app.store).is_none());
    app.exporter_en_image(std::env::temp_dir().join("glucose-jamais-ecrite.png"));
    assert!(app.attentes.export.is_none());
    let toast = app.ui.current_toast.as_ref().expect("un compte-rendu");
    assert!(
        toast.message.contains("Rien à exporter"),
        "{}",
        toast.message
    );
}

/// Le nom dit le format : `.png` est une image, et c'est le repli d'un nom sans extension.
#[test]
fn test_export_png_1_le_nom_dit_l_image() {
    use super::super::{destination, Format};
    assert_eq!(Format::du_chemin(Path::new("x.PNG")), Some(Format::Png));
    let (chemin, format) = destination(PathBuf::from("planche"));
    assert_eq!(
        (chemin, format),
        (PathBuf::from("planche.png"), Format::Png)
    );
}
