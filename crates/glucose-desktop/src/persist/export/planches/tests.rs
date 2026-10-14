//! L'export en planches (P2.7, fiche 68), de bout en bout : chaque planche est le canevas, et le
//! PDF porte ces planches mêmes, octet pour octet.

use super::*;
use crate::epreuve::Dossier;
use glucose_core::store::storyboard::{Format, Grille};
use glucose_core::types::BoardImage;
use std::time::{Duration, Instant};

/// Une photo unie de 100 × 100, écrite dans le dossier, posée centrée en `centre`.
fn poser(app: &mut GlucoseApp, dossier: &Dossier, (id, rvb): (&str, [u8; 3]), centre: (f64, f64)) {
    let fichier = dossier.join(format!("{id}.png"));
    let [r, v, b] = rvb;
    image::RgbaImage::from_pixel(100, 100, image::Rgba([r, v, b, 255]))
        .save(&fichier)
        .expect("une photo d'épreuve");
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new(id, centre.0, centre.1, 60.0, 60.0);
    img.original_width = 100.0;
    img.original_height = 100.0;
    img.src = Some(fichier.to_string_lossy().into_owned());
    app.store.add_image(&board, img);
}

/// Un storyboard de sept cases sur trois colonnes, une photo rouge dans la première et une
/// bleue dans la dernière ; rend son nom.
fn storyboard(dossier: &Dossier) -> (GlucoseApp, String) {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    let board = app.store.project.active_board_id.clone();
    let grille = Grille {
        format: Format::Classique,
        largeur: 120.0,
        colonnes: 3,
        ecart: 20.0,
    };
    let d = app
        .store
        .disposition_au_clic((&board, None), grille, (50.0, 40.0));
    let d = glucose_core::store::storyboard::Disposition { nombre: 7, ..d };
    let sb = app.store.disposer((&board, None), d).expect("posé");
    let cases: Vec<_> = app.store.cases(&board, &sb).into_iter().cloned().collect();
    let centre =
        |c: &glucose_core::types::StoryboardPanel| (c.x + c.width / 2.0, c.y + c.height / 2.0);
    poser(
        &mut app,
        dossier,
        ("rouge", [220, 30, 30]),
        centre(&cases[0]),
    );
    poser(
        &mut app,
        dossier,
        ("bleu", [40, 60, 230]),
        centre(&cases[6]),
    );
    app.store.clear_selection();
    app.ui.storyboard.courant = Some(sb.clone());
    (app, sb)
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

/// Le pixel de cette planche où tombe ce point du monde — placé d'après le rectangle que la loi
/// du noyau donne à la planche, et non d'après la vue du rendu : une vue fausse, lue par
/// elle-même, se lirait juste (fiche 68).
fn pixel(image: &image::RgbImage, planche: (f64, f64, f64, f64), (x, y): (f64, f64)) -> [u8; 3] {
    let (px, py) = ((x - planche.0) * ECHELLE, (y - planche.1) * ECHELLE);
    assert!(px >= 0.0 && py >= 0.0, "le point est dans la planche");
    image.get_pixel(px as u32, py as u32).0
}

/// **Chaque planche est le canevas** : une image par planche, à la taille des planches ; la
/// photo rouge au centre de la première case de la première, la bleue au centre de la dernière
/// case de la dernière — chacune à sa place dans sa planche.
#[test]
fn test_p2_7_les_planches_en_images_sont_le_canevas() {
    let dossier = Dossier::nouveau("planches-png");
    let (mut app, sb) = storyboard(&dossier);
    let board = app.store.project.active_board_id.clone();
    // Un autre storyboard juste sous la dernière rangée : il tombe dans le rectangle de la
    // dernière planche, et n'y doit pas être (vu sur l'aperçu, fiche 68).
    let d = app.store.disposition_actuelle(&board, &sb).expect("rangé");
    let sous = (d.origine.0, d.origine.1 + 3.0 * d.grille.pas().1);
    let autre = app
        .store
        .poser_le_storyboard(&board, d.grille, sous)
        .expect("un autre");
    let sa_case = app.store.cases(&board, &autre)[0].clone();
    let au_milieu = (sa_case.x + sa_case.width / 2.0, sa_case.y + 10.0);
    let planches = preparer(&app.store, &sb).expect("des planches");
    let nombre = planches.vues.len();
    assert!(nombre >= 2, "sept cases font plusieurs planches : {nombre}");
    let chemin = dossier.join("planches.png");
    app.exporter_les_planches_vers(chemin.clone());
    attendre(&mut app);
    let cases: Vec<_> = app.store.cases(&board, &sb).into_iter().cloned().collect();
    let centre = |k: usize| {
        let c = &cases[k];
        (c.x + c.width / 2.0, c.y + c.height / 2.0)
    };
    let page = |k: usize| {
        let image = image::open(chemin_de_la_planche(&chemin, k, nombre))
            .expect("une planche lisible")
            .to_rgb8();
        assert_eq!(
            (image.width(), image.height()),
            (planches.largeur, planches.hauteur)
        );
        image
    };
    let rectangles = d.planches();
    assert_eq!(rectangles.len(), nombre);
    assert_eq!(pixel(&page(0), rectangles[0], centre(0)), [220, 30, 30]);
    let derniere = nombre - 1;
    let bleu = pixel(&page(derniere), rectangles[derniere], centre(6));
    assert_eq!(
        bleu,
        [40, 60, 230],
        "la dernière case, sur la dernière planche"
    );
    let r = rectangles[derniere];
    assert!(
        au_milieu.1 > r.1 && au_milieu.1 < r.1 + r.3,
        "l'autre storyboard tombe bien dans la dernière planche"
    );
    let c = app.renderer.theme.bg_canvas.to_color_u8();
    assert_eq!(
        pixel(&page(derniere), r, au_milieu),
        [c.red(), c.green(), c.blue()],
        "le fond, et non l'autre storyboard"
    );
}

/// **Le PDF porte une page par planche, et chaque page est la planche même** : l'image d'une
/// page a les octets compressés de la planche en PNG, au bit près.
#[test]
fn test_p2_7_le_pdf_porte_les_planches_memes() {
    let dossier = Dossier::nouveau("planches-pdf");
    let (mut app, sb) = storyboard(&dossier);
    let nombre = preparer(&app.store, &sb).expect("des planches").vues.len();
    let images = dossier.join("planches.png");
    app.exporter_les_planches_vers(images.clone());
    attendre(&mut app);
    let pdf = dossier.join("planches.pdf");
    app.exporter_les_planches_vers(pdf.clone());
    attendre(&mut app);
    let octets = std::fs::read(&pdf).expect("le PDF");
    assert!(octets.starts_with(b"%PDF-1.4"));
    let compter = |motif: &[u8]| octets.windows(motif.len()).filter(|w| *w == motif).count();
    assert_eq!(compter(b"/Type /Page "), nombre, "une page par planche");
    for k in 0..nombre {
        let png = std::fs::read(chemin_de_la_planche(&images, k, nombre)).expect("la planche");
        let planche = crate::persist::export::pdf::planche(&png).expect("un PNG RVB");
        assert_eq!(
            compter(&planche.idat),
            1,
            "la planche {k} est dans le PDF, telle quelle"
        );
    }
}

/// **Les noms des planches** : celui qu'on a choisi pour une seule, numérotés pour plusieurs.
#[test]
fn test_p2_7_le_nom_de_chaque_planche() {
    let choisi = Path::new("dossier").join("scene 1.png");
    assert_eq!(chemin_de_la_planche(&choisi, 0, 1), choisi);
    assert_eq!(
        chemin_de_la_planche(&choisi, 2, 3),
        Path::new("dossier").join("scene 1-3.png")
    );
}
