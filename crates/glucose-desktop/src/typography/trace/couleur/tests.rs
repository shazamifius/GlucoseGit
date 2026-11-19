//! EMOJI-2 — les émojis en couleur, éprouvés sur les trois chemins qui posent une lettre.

use crate::typography::choix::COULEUR;
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

const COCHE: char = '\u{2714}';

/// APCA 0.0.98G-4g (Myndex, la mesure de LIEN-BLEU-1, fiche 113) : la grandeur du contraste d'une
/// couleur sur un fond. Sous 15, « treat as invisible » ; 45, « the minimum for pictograms with fine
/// details » (APCA in a Nutshell).
fn apca(texte: [u8; 3], fond: [u8; 3]) -> f64 {
    let y = |c: [u8; 3]| {
        let [r, g, b] = c.map(|v| (f64::from(v) / 255.0).powf(2.4));
        let y = 0.212_672_9 * r + 0.715_152_2 * g + 0.072_175 * b;
        if y < 0.022 {
            y + (0.022 - y).powf(1.414)
        } else {
            y
        }
    };
    let (t, f) = (y(texte), y(fond));
    if (f - t).abs() < 0.0005 {
        return 0.0;
    }
    let s = if f > t {
        (f.powf(0.56) - t.powf(0.57)) * 1.14
    } else {
        (t.powf(0.62) - f.powf(0.65)) * 1.14
    };
    if s < 0.1 {
        0.0
    } else {
        (s - 0.027) * 100.0
    }
}

fn octets(c: Color) -> [u8; 3] {
    let c = c.to_color_u8();
    [c.red(), c.green(), c.blue()]
}

/// **Le pixel le plus lisible** de `ch` dessiné à l'encre `encre` sur le papier `papier` : son
/// contraste, et sa couleur — l'intérieur d'une couche, entièrement couvert.
fn le_plus_lisible(typo: &Typography, ch: char, encre: Color, papier: Color) -> (f64, [u8; 3]) {
    let corps = 32.0;
    let mut p = Pixmap::new(96, 64).expect("tampon");
    p.fill(papier);
    let style = TextStyle {
        size: corps,
        color: encre,
        face: Face::Regular,
    };
    typo.draw_text(&mut p.as_mut(), &ch.to_string(), 16.0, 8.0, style);
    p.pixels()
        .iter()
        .map(|px| {
            let c = px.demultiply();
            let rgb = [c.red(), c.green(), c.blue()];
            (apca(rgb, octets(papier)), rgb)
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .expect("des pixels")
}

/// La couleur d'une couche de `ch` dans Twemoji, pour un papier clair (0) ou sombre (1).
fn couleur_de(typo: &Typography, ch: char, palette: usize) -> Color {
    let indice = typo.contours.indice(COULEUR, ch).expect("dans Twemoji");
    let couches = typo.contours.couches((COULEUR, indice));
    let couleurs = couches.iter().find_map(|c| c.couleur);
    couleurs.expect("une couche de couleur")[palette]
}

/// **EMOJI-3 — sur le papier d'un nœud, ✔ se lit** : dessiné à l'encre du corps sur le papier d'un
/// texte, il se lit comme un pictogramme (Lc 45). Preuve à l'envers : sa couleur d'origine, celle
/// d'un papier clair, y est invisible (sous Lc 15) — ce que la 2.0.4-beta.3 aurait montré.
#[test]
fn test_emoji_3_sur_le_papier_d_un_noeud_la_coche_se_lit() {
    let (typo, theme) = (Typography::new(), crate::theme::Theme::dark());
    let (lc, _) = le_plus_lisible(&typo, COCHE, theme.card_body, theme.card_paper);
    assert!(lc >= 45.0, "✔ sur le papier d'un nœud : Lc {lc:.1}");
    let origine = octets(couleur_de(&typo, COCHE, 0));
    assert!(
        apca(origine, octets(theme.card_paper)) < 15.0,
        "sa couleur d'origine {origine:?} y est invisible"
    );
}

/// **Sur un post-it, ✔ garde la couleur de son auteur** : l'encre y est foncée, le papier clair —
/// la palette d'origine, faite pour lui, s'y lit.
#[test]
fn test_emoji_3_sur_un_post_it_la_coche_garde_sa_couleur() {
    let (typo, theme) = (Typography::new(), crate::theme::Theme::dark());
    let (encre, papier) = (theme.sticky_yellow_text, theme.sticky_yellow_bg);
    let (lc, rgb) = le_plus_lisible(&typo, COCHE, encre, papier);
    let origine = octets(couleur_de(&typo, COCHE, 0));
    assert!(lc >= 45.0, "✔ sur un post-it : Lc {lc:.1}");
    assert!(
        rgb.iter().zip(origine).all(|(&a, b)| a.abs_diff(b) <= 2),
        "sa couleur d'origine {origine:?}, pas {rgb:?}"
    );
}

/// **La loi de la palette sombre, sur toute la police, pour le papier et l'encre du thème** : un
/// émoji dont une couleur se voit sur le papier y garde toutes les siennes, à l'octet ; un émoji
/// invisible (sous Lc 15) y prend la clarté de l'encre du corps (au huitième de niveau près), sa
/// teinte gardée à moins d'une différence à peine visible. Un thème qui change sans que la police
/// soit reconstruite (`outils/emojis/construire.sh`) fait tomber cette épreuve.
#[test]
fn test_emoji_3_la_palette_sombre_suit_le_papier_et_l_encre_du_theme() {
    use glucose_core::couleur::{Oklch, DIFFERENCE_A_PEINE_VISIBLE};
    let (typo, theme) = (Typography::new(), crate::theme::Theme::dark());
    let papier = octets(theme.card_paper);
    let oklch = |c: Color| Oklch::des_octets(octets(c).into());
    let mut eclaircis = 0;
    for ch in (0..0x20000).filter_map(char::from_u32) {
        let Some(indice) = typo.contours.indice(COULEUR, ch) else {
            continue;
        };
        let couches: Vec<[Color; 2]> = typo
            .contours
            .couches((COULEUR, indice))
            .iter()
            .filter_map(|c| c.couleur)
            .collect();
        // Un glyphe sans couche de couleur (l'espace) n'a rien à éclaircir.
        if couches.is_empty() {
            continue;
        }
        if couches
            .iter()
            .any(|[c, _]| apca(octets(*c), papier) >= 15.0)
        {
            assert!(
                couches.iter().all(|[c, s]| octets(*c) == octets(*s)),
                "{ch:?} se voyait"
            );
            continue;
        }
        eclaircis += 1;
        let claire = couches.iter().map(|[_, s]| oklch(*s).l).fold(0.0, f64::max);
        assert!(
            (claire - oklch(theme.card_body).l).abs() < 0.01,
            "{ch:?} : la clarté de l'encre, {claire}"
        );
        for [c, s] in &couches {
            let (avant, apres) = (oklch(*c), oklch(*s));
            let ecart = (apres.h - avant.h + std::f64::consts::PI)
                .rem_euclid(std::f64::consts::TAU)
                - std::f64::consts::PI;
            assert!(
                avant.c < DIFFERENCE_A_PEINE_VISIBLE
                    || apres.c * ecart.abs() < DIFFERENCE_A_PEINE_VISIBLE,
                "{ch:?} : la teinte gardée, {avant:?} puis {apres:?}"
            );
        }
    }
    assert!(
        eclaircis >= 30,
        "les signes d'encre de Twemoji : {eclaircis}"
    );
}

/// **La palette sombre se lit à la déclaration de la police** (CPAL 1) ; une police qui n'en
/// déclare aucune — ou n'a pas de palette — se lit dans sa palette 0.
#[test]
fn test_emoji_3_la_palette_sombre_est_celle_que_la_police_declare() {
    let couleur = ttf_parser::Face::parse(crate::typography::COULEUR_BYTES, 0).expect("police");
    assert_eq!(super::palette_du_fond_sombre(&couleur), 1);
    let inter = ttf_parser::Face::parse(crate::typography::FACE_BYTES[0], 0).expect("police");
    assert_eq!(super::palette_du_fond_sombre(&inter), 0);
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
