//! Les épreuves de REGION-1 et de NEUTRE-1, lues sur l'image que l'application peint (voie du
//! processeur ; l'égalité des deux voies est l'affaire de `tests/voies_suite.rs`).

use glucose_core::couleur::Oklch;
use glucose_core::types::{Annotation, Viewport};

/// Une membrane de cette boîte et de cette couleur.
fn membrane(id: &str, (x, y, l, h): (f64, f64, f64, f64), couleur: Option<&str>) -> Annotation {
    let mut m = Annotation::membrane(id, x, y, l, h);
    if let Annotation::Membrane { color, .. } = &mut m {
        *color = couleur.map(str::to_string);
    }
    m
}

/// L'image de ces membranes, et d'un storyboard d'une case si on en donne le coin : 800 × 600,
/// le monde `(0, 0)` en `(40, 300)` à l'échelle 1.
fn rendre(membranes: Vec<Annotation>, case: Option<(f64, f64)>) -> tiny_skia::Pixmap {
    let mut app = crate::app::GlucoseApp::new();
    app.ui.current_toast = None;
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let tableau = app.store.project.active_board_id.clone();
    for m in membranes {
        app.store.add_annotation(&tableau, m);
    }
    if let Some(coin) = case {
        let grille = glucose_core::store::storyboard::Grille {
            largeur: 120.0,
            colonnes: 1,
            ..Default::default()
        };
        app.store.poser_le_storyboard(&tableau, grille, coin);
    }
    app.store.clear_selection();
    let vue = Viewport {
        x: 40.0,
        y: 300.0,
        scale: 1.0,
    };
    app.store.set_viewport(&tableau, vue);
    app.une_image_sans_fenetre((800, 600));
    app.pixmap.take().expect("une image")
}

/// Le pixel du monde `(x, y)` : l'écran le pose en `(x + 40, y + 300)`.
fn au_monde(image: &tiny_skia::Pixmap, (x, y): (f64, f64)) -> (u8, u8, u8) {
    let (sx, sy) = ((x + 40.0) as usize, (y + 300.0) as usize);
    let p = image.pixels()[sy * image.width() as usize + sx].demultiply();
    (p.red(), p.green(), p.blue())
}

/// Le ton d'une région de cette couleur, seule sur le canevas du thème.
fn ton(couleur: (u8, u8, u8)) -> (u8, u8, u8) {
    let canevas = crate::theme::Theme::dark().bg_canvas.to_color_u8();
    let f = super::force();
    let melange = |c: u8, t: u8| (f32::from(c) + f * (f32::from(t) - f32::from(c))).round() as u8;
    (
        melange(canevas.red(), couleur.0),
        melange(canevas.green(), couleur.1),
        melange(canevas.blue(), couleur.2),
    )
}

fn proche(a: (u8, u8, u8), b: (u8, u8, u8)) -> bool {
    a.0.abs_diff(b.0) <= 1 && a.1.abs_diff(b.1) <= 1 && a.2.abs_diff(b.2) <= 1
}

const MERE: (f64, f64, f64, f64) = (-400.0, -400.0, 1200.0, 900.0);
const FILLE: (f64, f64, f64, f64) = (100.0, -300.0, 600.0, 400.0);
/// Au cœur de la fille, loin de ses bords et de la grille ; puis dans la mère, loin de la fille.
const DANS_LA_FILLE: (f64, f64) = (400.0, -100.0);
const DANS_LA_MERE: (f64, f64) = (300.0, 250.0);

/// **REGION-1 — la couleur en un point est celle de la plus petite région qui le contient**
/// (fiche 105 ; sa parole du 11/11/2026, *« je suis d'accord »*). Une fille bleue dans une mère
/// rouge, puis verte : au cœur de la fille, le ton qu'elle aurait seule sur le canevas, quelle
/// que soit sa mère ; dans la mère, celui de la mère. Et la fille posée **avant** sa mère dans le
/// modèle passe encore par-dessus : c'est la surface qui ordonne, pas le modèle.
///
/// **La preuve à l'envers** : la mère rouge et la mère verte donnent bien deux tons différents —
/// sous la loi d'avant, où chaque membrane voilait ses mères, le cœur de la fille les aurait
/// mélangés.
#[test]
fn region_1_la_plus_petite_region_l_emporte() {
    let bleu = (59, 130, 246);
    for (mere, couleur_de_la_mere) in [("#ef4444", (239, 68, 68)), ("#22c55e", (34, 197, 94))] {
        let dans_l_ordre = vec![
            membrane("m", MERE, Some(mere)),
            membrane("f", FILLE, Some("#3b82f6")),
        ];
        let a_l_envers = dans_l_ordre.iter().rev().cloned().collect();
        for annotations in [dans_l_ordre, a_l_envers] {
            let image = rendre(annotations, None);
            let fille = au_monde(&image, DANS_LA_FILLE);
            assert!(
                proche(fille, ton(bleu)),
                "la fille : {fille:?} contre {:?}",
                ton(bleu)
            );
            let autour = au_monde(&image, DANS_LA_MERE);
            let attendu = ton(couleur_de_la_mere);
            assert!(proche(autour, attendu), "la mère {mere} : {autour:?}");
        }
    }
    assert_ne!(
        ton((239, 68, 68)),
        ton((34, 197, 94)),
        "deux mères, deux tons"
    );
}

/// **La grille de points reste visible dans une membrane** (fiche 105) : posée après le sol des
/// régions, comme une carte pose son quadrillage par-dessus les pays. Le point du monde
/// `(360, −120)` est au cœur de la fille ; sous un corps opaque posé après la grille, il
/// disparaîtrait — la preuve à l'envers est le pixel voisin, qui n'a que le ton de la région.
#[test]
fn region_1_la_grille_se_voit_dans_une_membrane() {
    let image = rendre(
        vec![
            membrane("m", MERE, Some("#ef4444")),
            membrane("f", FILLE, Some("#3b82f6")),
        ],
        None,
    );
    let point = au_monde(&image, (360.0, -120.0));
    let region = au_monde(&image, DANS_LA_FILLE);
    for (p, r) in [
        (point.0, region.0),
        (point.1, region.1),
        (point.2, region.2),
    ] {
        assert!(p >= r + 15, "le point {point:?} sur la région {region:?}");
    }
}

/// **Une case de storyboard rangée dans une membrane garde son cadre** (fiche 105) : le corps des
/// zones et des cases se pose après le sol des régions. Sous un corps opaque, la case aurait pris
/// le ton de la membrane.
#[test]
fn region_1_une_case_dans_une_membrane_se_voit() {
    let image = rendre(
        vec![membrane("m", MERE, Some("#ef4444"))],
        Some((-20.0, 150.0)),
    );
    let cadre = au_monde(&image, (40.0, 175.0));
    assert_eq!(cadre, (0x1a, 0x1a, 0x1a), "le cadre de la case");
    assert!(
        proche(au_monde(&image, DANS_LA_MERE), ton((239, 68, 68))),
        "et la membrane autour"
    );
}

/// **NEUTRE-1 — une membrane sans couleur choisie est grise** (fiche 104, sa parole du 11/11/2026,
/// *« oui aussi »*) : un gris sans chroma, de la clarté perçue du bleu de Tauri qu'elle prenait.
/// Une couleur illisible ne ramène pas le bleu ; une couleur choisie reste la sienne.
#[test]
fn neutre_1_une_membrane_sans_couleur_est_grise() {
    let gris = super::teinte_de_membrane(None);
    assert!(gris.0 == gris.1 && gris.1 == gris.2, "neutre : {gris:?}");
    let (l, bleu) = (
        Oklch::des_octets(gris).l,
        Oklch::des_octets(super::BLEU_DE_TAURI).l,
    );
    assert!(
        (l - bleu).abs() < 0.01,
        "la clarté du bleu : {l} contre {bleu}"
    );
    assert_eq!(super::teinte_de_membrane(Some("pas une couleur")), gris);
    assert_eq!(super::teinte_de_membrane(Some("#ef4444")), (239, 68, 68));
    let image = rendre(vec![membrane("m", MERE, None)], None);
    assert!(proche(au_monde(&image, DANS_LA_MERE), ton(gris)));
}
