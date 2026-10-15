//! **La vitrine de l'export** (P5, fiche 70) : un tableau qui porte chaque sorte de dessin que
//! l'export SVG doit savoir écrire — l'aperçu (`examples/apercu_svg`) et l'épreuve de
//! superposition (`persist/export/svg/tests.rs`) partent du même.
//!
//! La vitrine du noyau ([`synth::showcase`] : cartes, formules, listes, code, barré, post-it,
//! membrane, flèches, dossier vide), plus ce qu'elle n'a pas : deux photos (l'une recadrée,
//! l'autre tournée), un domaine sur une carte, une photo et la membrane, un gabarit et ses
//! zones, un storyboard (une case décrite, une autre avec son plan), un dossier qui contient
//! une note, la pilule d'un opérateur, une flèche qui porte un libellé et une relation.

use crate::app::GlucoseApp;
use glucose_core::store::storyboard::{Format, Grille, Texte};
use glucose_core::synth;
use glucose_core::types::{
    Annotation, ArrowPredicate, BoardImage, CanvasFolder, Domain, Plan, Recadrage, StickyOperator,
};
use std::path::Path;

/// **La vitrine de l'export**, ses photos écrites dans `dossier`.
pub fn vitrine_de_l_export(dossier: &Path) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    app.store = super::ouvert(synth::showcase());
    let board = app.store.project.active_board_id.clone();
    let bornes = app.store.content_bounds(&board).expect("une vitrine");
    let droite = bornes.left + bornes.width + 200.0;
    let bas = bornes.top + bornes.height + 160.0;
    poser_les_photos(&mut app, (&board, dossier), droite, bornes.top + 150.0);
    poser_les_domaines(&mut app, &board);
    let gabarit = ("charadesign", &["ref", "sketch"][..]);
    app.store
        .poser_un_gabarit(&board, gabarit, (bornes.left + 900.0, bas + 700.0));
    poser_le_storyboard(&mut app, &board, (droite, bas));
    poser_le_reste(&mut app, &board, (bornes.left, bas + 60.0));
    app.store.clear_selection();
    app
}

/// Un dégradé recadré, un damier tourné.
fn poser_les_photos(app: &mut GlucoseApp, (board, dossier): (&str, &Path), x: f64, y: f64) {
    let photos = [
        (
            "degrade",
            Recadrage::depuis_les_marges(0.2, 0.1, 0.1, 0.3),
            0.0,
        ),
        (
            "damier",
            Recadrage::depuis_les_marges(0.0, 0.0, 0.0, 0.0),
            0.35,
        ),
    ];
    for (k, (nom, recadrage, rotation)) in photos.into_iter().enumerate() {
        let chemin = dossier.join(format!("{nom}.png"));
        image::RgbImage::from_fn(240, 160, |x, y| match nom {
            "degrade" => image::Rgb([x as u8, y as u8, 200]),
            _ if (x / 20 + y / 20) % 2 == 0 => image::Rgb([230, 230, 230]),
            _ => image::Rgb([40, 40, 40]),
        })
        .save(&chemin)
        .expect("une photo de la vitrine");
        let mut img = BoardImage::new(nom, x, y + k as f64 * 260.0, 240.0, 160.0);
        (img.original_width, img.original_height) = (240.0, 160.0);
        img.src = Some(chemin.to_string_lossy().into_owned());
        img.crop = recadrage;
        img.rotation = rotation;
        app.store.add_image(board, img);
    }
}

/// Un domaine, porté par une carte, une photo et la membrane, à trois poids.
fn poser_les_domaines(app: &mut GlucoseApp, board: &str) {
    let domaine = Domain {
        id: "dom-info".into(),
        name: "Information".into(),
        color: "#f472b6".into(),
        icon: "inf".into(),
        created_at: 0,
    };
    app.store.try_add_domain(domaine).expect("un domaine");
    for (noeud, poids) in [("s-kraft", 0.6), ("degrade", 0.3), ("s-membrane", 1.0)] {
        app.store
            .try_assign_domain_to_node(board, noeud, "dom-info", poids)
            .expect("une affectation");
    }
}

/// Un storyboard de quatre cases : la première décrite, la deuxième en gros plan.
fn poser_le_storyboard(app: &mut GlucoseApp, board: &str, coin: (f64, f64)) {
    let grille = Grille {
        format: Format::Cinema,
        largeur: 200.0,
        colonnes: 2,
        ecart: 24.0,
    };
    let sb = app
        .store
        .poser_le_storyboard(board, grille, coin)
        .expect("un storyboard");
    let cases: Vec<_> = app.store.cases(board, &sb).into_iter().cloned().collect();
    let texte = "Plan large, la ville au loin";
    app.store
        .ecrire_dans_la_case(board, &cases[0].id, Texte::Description, texte);
    app.store
        .choisir_le_plan(board, &cases[1].id, Some(Plan::Gros));
}

/// Un dossier qui contient une note, la pilule d'un opérateur, et la flèche qui parle.
fn poser_le_reste(app: &mut GlucoseApp, board: &str, (x, y): (f64, f64)) {
    let mut dossier = CanvasFolder::new("s-plein", "Exemples", String::new());
    (dossier.x, dossier.y) = (x, y);
    (dossier.width, dossier.height) = (300.0, 200.0);
    dossier.color = "#60a5fa".to_string();
    let note = Annotation::text("s-dedans", 0.0, 0.0, "Huffman, pas à pas");
    app.store
        .create_folder_with_content(board, dossier, vec![note]);

    let mut et = Annotation::sticky("s-et", x + 400.0, y, "et");
    if let Annotation::Sticky { operator, .. } = &mut et {
        *operator = Some(StickyOperator::And);
    }
    app.store.add_annotation(board, et);

    let mut fleche = Annotation::arrow("s-borne", x + 400.0, y + 120.0, x + 700.0, y + 160.0);
    if let Annotation::Arrow {
        text, predicate, ..
    } = &mut fleche
    {
        *text = Some("borne".to_string());
        *predicate = Some(ArrowPredicate::Illustre);
    }
    app.store.add_annotation(board, fleche);
}
