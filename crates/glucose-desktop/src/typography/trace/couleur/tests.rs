//! EMOJI-2 — les émojis en couleur, éprouvés sur les trois chemins qui posent une lettre.

use crate::typography::{Face, TextStyle, Typography};
use tiny_skia::{Color, Pixmap};

const VERT: char = '\u{1F7E2}';
const ROUGE: char = '\u{1F534}';
const SOURIRE: char = '\u{1F600}';

/// La couleur moyenne de ce qui n'est plus le fond blanc, pondérée par l'écart au blanc, et le
/// nombre de pixels touchés.
fn encre_moyenne(p: &Pixmap) -> ([f32; 3], usize) {
    let (mut somme, mut poids, mut n) = ([0.0f32; 3], 0.0f32, 0);
    for px in p.pixels() {
        let c = px.demultiply();
        let rgb = [c.red(), c.green(), c.blue()].map(f32::from);
        let ecart = rgb.iter().map(|v| 255.0 - v).sum::<f32>();
        if ecart > 30.0 {
            n += 1;
            poids += ecart;
            for k in 0..3 {
                somme[k] += rgb[k] * ecart;
            }
        }
    }
    (somme.map(|s| s / poids.max(1.0)), n)
}

fn dessiner(typo: &Typography, texte: &str, corps: f32, encre: Color) -> Pixmap {
    let mut p = Pixmap::new((corps * 3.0) as u32, (corps * 2.0) as u32).expect("tampon");
    p.fill(Color::WHITE);
    let style = TextStyle {
        size: corps,
        color: encre,
        face: Face::Regular,
    };
    typo.draw_text(&mut p.as_mut(), texte, corps * 0.5, corps * 0.25, style);
    p
}

/// **le rond vert est vert et le rond rouge est rouge**, en pixels (une petite lettre, son image en cache) comme en
/// grand (tracée par son contour, GRAND-1), dans un texte d'encre noire. Preuve à l'envers : la
/// loi d'EMOJI-1 les posait à l'encre du texte — noirs tous deux, le même rond.
#[test]
fn test_emoji_2_le_vert_est_vert_et_le_rouge_rouge() {
    let typo = Typography::new();
    for corps in [16.0_f32, 32.0, 300.0] {
        let (vert, n_vert) =
            encre_moyenne(&dessiner(&typo, &VERT.to_string(), corps, Color::BLACK));
        let (rouge, n_rouge) =
            encre_moyenne(&dessiner(&typo, &ROUGE.to_string(), corps, Color::BLACK));
        assert!(n_vert > 20 && n_rouge > 20, "à {corps} : ils se dessinent");
        assert!(
            vert[1] > vert[0] + 40.0 && vert[1] > vert[2] + 40.0,
            "à {corps} : le rond vert est vert, {vert:?}"
        );
        assert!(
            rouge[0] > rouge[1] + 80.0 && rouge[0] > rouge[2] + 80.0,
            "à {corps} : le rond rouge est rouge, {rouge:?}"
        );
        assert!(
            vert.iter().chain(&rouge).any(|&v| v > 60.0),
            "à {corps} : pas l'encre noire du texte"
        );
    }
}

/// **Un émoji pâlit avec son texte** : les lentilles estompent par l'alpha (FRISE-2,
/// DOMAINE-1) ; un émoji qui garderait sa couleur pleine sortirait de son texte estompé.
#[test]
fn test_emoji_2_un_emoji_palit_avec_son_texte() {
    let typo = Typography::new();
    let plein = dessiner(&typo, &ROUGE.to_string(), 32.0, Color::BLACK);
    let estompe = dessiner(
        &typo,
        &ROUGE.to_string(),
        32.0,
        Color::from_rgba8(0, 0, 0, 64),
    );
    let vert_moyen = |p: &Pixmap| {
        let (c, n) = encre_moyenne(p);
        (c[1], n)
    };
    let ((g_plein, _), (g_estompe, n)) = (vert_moyen(&plein), vert_moyen(&estompe));
    assert!(n > 0, "il se dessine encore");
    assert!(
        g_estompe > g_plein + 60.0,
        "le rouge estompé tire vers le blanc du fond : {g_plein} puis {g_estompe}"
    );
}

/// **Lu d'une seule encre, un émoji est sa silhouette** : son encre de loin (GRIS-1) est celle de
/// l'union de ses couches — un disque plein pour 🔴. Preuve à l'envers : sa base, lue seule, est
/// un trait sans surface, et ne laisse aucune encre.
#[test]
fn test_emoji_2_de_loin_un_emoji_a_l_encre_de_sa_silhouette() {
    let typo = Typography::new();
    let corps = 64.0;
    let encre = typo.encre(ROUGE, corps, Face::Regular);
    // Le disque de Twemoji remplit le 😀 de Noto : environ un em de diamètre.
    let disque = std::f32::consts::PI * (corps / 2.0).powi(2);
    assert!(
        (encre / disque - 1.0).abs() < 0.15,
        "l'encre du rond rouge, {encre}, est celle d'un disque d'un em, {disque}"
    );
    // L'ancienne loi : le glyphe que le choix donne, rempli seul.
    let base = typo.choix(ROUGE, Face::Regular);
    let mut seule = Pixmap::new(128, 128).expect("tampon");
    typo.contours.remplir(
        &mut seule.as_mut(),
        base,
        (10.0, 100.0),
        corps,
        Color::BLACK,
    );
    assert!(
        seule.pixels().iter().all(|px| px.alpha() == 0),
        "la base seule n'a pas d'encre"
    );
    assert!(typo.encre(SOURIRE, corps, Face::Regular) > 0.5 * disque);
}

/// **Les métriques d'avant** : l'avance d'un émoji est celle de Noto Emoji (2 600 pour un em de
/// 2 048), pour qu'aucun texte déjà posé ne se remette en page ; et le visage souriant tient dans
/// la boîte de celui de Noto, à un pixel près à 64 de corps.
#[test]
fn test_emoji_2_les_metriques_de_noto_sont_gardees() {
    let typo = Typography::new();
    let corps = 64.0;
    for ch in [VERT, ROUGE, SOURIRE] {
        assert_eq!(
            typo.advance(ch, corps, Face::Regular),
            2600.0 * corps / 2048.0,
            "{ch:?}"
        );
    }
    let m = typo.metriques(typo.choix(SOURIRE, Face::Regular), corps);
    // Le 😀 de Noto : x de 260 à 2 340, y de −335 à 1 735, pour un em de 2 048.
    let noto = [260.0, -335.0, 2340.0, 1735.0].map(|v: f32| v * corps / 2048.0);
    let boite = [
        m.xmin as f32,
        m.ymin as f32,
        (m.xmin + m.width as i32) as f32,
        (m.ymin + m.height as i32) as f32,
    ];
    for (b, n) in boite.iter().zip(noto) {
        assert!((b - n).abs() <= 2.0, "la boîte {boite:?} contre {noto:?}");
    }
}
