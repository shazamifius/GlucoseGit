//! **TITRE-1 à l'écran** : aucun titre de membrane ne se cache, à aucune distance, et chacun a la
//! taille de sa région — lus sur l'image que l'application peint, par la voie processeur.

use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, Viewport};

const TAILLE: (u32, u32) = (800, 600);
/// Où tombe le bord haut de la membrane, à l'écran : sous le bandeau, avec de la place au-dessus.
const HAUT: f64 = 400.0;

/// Une application sans fenêtre, une seule membrane titrée de cette taille, vue à ce zoom, son
/// coin haut-gauche à (40, `HAUT`) de l'écran — puis une image.
fn image(largeur: f64, hauteur: f64, zoom: f64) -> GlucoseApp {
    image_titree("Hydrologie", (largeur, hauteur), zoom)
}

fn image_titree(titre: &str, taille: (f64, f64), zoom: f64) -> GlucoseApp {
    image_vue(titre, taille, zoom, HAUT)
}

/// La même, le bord haut de la membrane à `haut` pixels du haut de l'écran.
fn image_vue(titre: &str, (largeur, hauteur): (f64, f64), zoom: f64, haut: f64) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    let mut m = Annotation::membrane("m", 0.0, 0.0, largeur, hauteur);
    if let Annotation::Membrane { text, .. } = &mut m {
        *text = Some(titre.to_string());
    }
    app.store.add_annotation(&board, m);
    app.store.clear_selection();
    app.store.set_viewport(
        &board,
        Viewport {
            x: 40.0,
            y: haut,
            scale: zoom,
        },
    );
    app.une_image_sans_fenetre(TAILLE);
    app
}

/// **Les lignes de l'écran qui portent le titre** : au-dessus du bord haut (le pointillé exclu),
/// au droit de la membrane, un pixel de sa teinte vive — `#60a5fa`, que ni le fond du canevas, ni
/// le voile de la membrane, ni ses halos n'atteignent.
fn lignes_du_titre(app: &GlucoseApp, largeur_a_l_ecran: f64) -> usize {
    let droite = (40.0 + largeur_a_l_ecran).min(f64::from(TAILLE.0)) as usize;
    (0..(HAUT as usize - 3))
        .filter(|&y| (40..droite).any(|x| vif(app, x, y)))
        .count()
}

/// Un pixel de la teinte vive du titre.
fn vif(app: &GlucoseApp, x: usize, y: usize) -> bool {
    let pixmap = app.pixmap.as_ref().expect("une image");
    let p = pixmap.pixels()[y * pixmap.width() as usize + x].demultiply();
    p.blue() > 170 && p.red() < 150 && p.green() > 110
}

/// **Un nom ne déborde jamais sur la région voisine** : une colonne étroite et très haute — sa
/// surface lui donnerait un titre de plus de mille pixels de large — pose le sien dans sa
/// largeur, au plus grand corps qui y tient.
#[test]
fn titre_1_un_titre_tient_dans_la_largeur_de_sa_membrane() {
    let titre = "L'eau guérisseuse et protectrice";
    let corps_de_la_loi =
        glucose_core::membrane_titre::CORPS * glucose_core::membrane_titre::echelle(200.0, 6000.0);
    assert!(
        corps_de_la_loi * 0.4 * titre.chars().count() as f64 > 200.0,
        "l'épreuve n'exerce pas un titre plus large que sa membrane"
    );
    let app = image_titree(titre, (200.0, 6000.0), 1.0);
    let lignes: Vec<usize> = (0..(HAUT as usize - 3))
        .filter(|&y| (0..TAILLE.0 as usize).any(|x| vif(&app, x, y)))
        .collect();
    assert!(
        lignes.len() >= 3,
        "le titre se dessine : {} lignes",
        lignes.len()
    );
    let plus_a_droite = (0..TAILLE.0 as usize)
        .rev()
        .find(|&x| lignes.iter().any(|&y| vif(&app, x, y)))
        .expect("un pixel du titre");
    assert!(
        plus_a_droite <= 40 + 200,
        "le titre déborde de sa membrane jusqu'à x = {plus_a_droite}"
    );
}

/// **Aucun titre ne se cache selon la distance** (sa parole du 8/11/2026). Une région cent fois
/// plus large qu'une membrane neuve, vue à 1 % : son titre fait seize pixels, et se dessine. Avant
/// TITRE-1, sous 20 % de zoom, aucun titre ne se dessinait — et celui-ci, au corps fixe de 16,
/// n'aurait fait qu'un sixième de pixel.
#[test]
fn titre_1_un_titre_se_dessine_a_toute_distance() {
    let app = image(32_000.0, 24_000.0, 0.01);
    let lignes = lignes_du_titre(&app, 320.0);
    assert!(
        lignes >= 8,
        "vu à 1 %, le nom de la région ne se dessine que sur {lignes} lignes"
    );
}

/// **Le nom d'une région à la taille de la région** : seize fois la surface d'une membrane neuve,
/// un titre quatre fois plus haut, au même zoom. Une membrane neuve garde le titre d'aujourd'hui.
#[test]
fn titre_1_le_titre_grandit_comme_la_racine_de_la_surface() {
    let petite = lignes_du_titre(&image(320.0, 240.0, 1.0), 320.0);
    let grande = lignes_du_titre(&image(1280.0, 960.0, 1.0), 1280.0);
    assert!(
        petite >= 5,
        "le titre d'une membrane neuve : {petite} lignes"
    );
    assert!(
        grande >= 3 * petite,
        "seize fois la surface : {grande} lignes de titre, contre {petite}"
    );
    assert!(
        grande <= 5 * petite,
        "le titre a trop grandi : {grande} contre {petite}"
    );
}

/// **Un nom se voit avant sa région** : une membrane encore sous l'écran — 800 unités au-delà, bien
/// plus que la marge du culling — montre déjà son titre au bas de l'écran. Ce qui est visible se
/// décide sur l'encre (TITRE-1), pas sur la boîte.
#[test]
fn titre_1_le_titre_d_une_membrane_sous_l_ecran_se_voit() {
    let app = image_vue("Hydrologie", (32_000.0, 24_000.0), 0.05, 640.0);
    let bas = TAILLE.1 as usize;
    let lignes = (bas - 50..bas)
        .filter(|&y| (40..TAILLE.0 as usize).any(|x| vif(&app, x, y)))
        .count();
    assert!(
        lignes >= 20,
        "le nom de la région sous l'écran : {lignes} lignes"
    );
}
