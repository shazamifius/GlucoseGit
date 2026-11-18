//! Les épreuves de GRILLE-1 : chacune rejoue la typographie du monde sur les mêmes données et
//! vérifie qu'elle concluait l'inverse.

use super::super::{Face, TextStyle, Typography};
use super::SOUS;
use tiny_skia::{Color, Pixmap};

/// La plus forte couverture de chaque rangée d'un tampon où une seule lettre blanche est écrite.
fn encre_par_rangee(p: &Pixmap) -> Vec<u8> {
    let w = p.width() as usize;
    p.data()
        .chunks(w * 4)
        .map(|rangee| rangee.chunks(4).map(|px| px[3]).max().unwrap_or(0))
        .collect()
}

fn ecrire(typo: &Typography, texte: &str, (x, y): (f32, f32), size: f32) -> Pixmap {
    let mut p = Pixmap::new(64, 48).unwrap();
    let style = TextStyle {
        size,
        color: Color::WHITE,
        face: Face::Regular,
    };
    typo.draw_text(&mut p.as_mut(), texte, x, y, style);
    p
}

/// La hauteur d'x du visage droit, en pixels, au corps `size`.
fn hauteur_d_x(typo: &Typography, size: f32) -> f32 {
    typo.polices.hauteurs_d_x[Face::Regular as usize].unwrap() * size
}

/// **Ce que la loi exige d'un « z »** (ses deux barres sont plates, sans débord) : rien sous la
/// ligne de base ni au-dessus de la hauteur d'x entière, et chacune de ses barres dans une rangée
/// pleine. La construction ne laisse fuir, de part et d'autre, que l'interpolation de l'étirement
/// entre deux sous-rangées : au plus un quart de rangée.
fn z_sur_la_grille(typo: &Typography, size: f32, y: f32) -> Result<(), String> {
    let p = ecrire(typo, "z", (3.3, y), size);
    let encre = encre_par_rangee(&p);
    let base = (y + size).round() as usize;
    let haut = base - hauteur_d_x(typo, size).round() as usize;
    let fuite = (255 / SOUS) as u8;
    let constat = format!(
        "corps {size}, y {y} : rangées {:?}",
        &encre[haut - 2..base + 2]
    );
    if encre[base] > fuite || encre[haut - 1] > fuite {
        return Err(format!("de l'encre hors de la grille — {constat}"));
    }
    if encre[base - 1] < 128 || encre[haut] < 128 {
        return Err(format!("une barre à cheval sur deux rangées — {constat}"));
    }
    Ok(())
}

#[test]
fn test_grille_1_la_ligne_de_base_et_la_hauteur_d_x_tombent_sur_des_pixels_entiers() {
    let monde = Typography::new();
    let ecran = monde.posee_sur_l_ecran();
    let mut le_monde_echoue = false;
    for dixiemes in (100..=180).step_by(5) {
        let size = dixiemes as f32 / 10.0;
        for y in [5.0, 5.25, 5.37, 5.5, 5.81] {
            if let Err(faute) = z_sur_la_grille(&ecran, size, y) {
                panic!("l'écran : {faute}");
            }
            le_monde_echoue |= z_sur_la_grille(&monde, size, y).is_err();
        }
    }
    // La preuve à l'envers : la typographie du monde pose ses lettres hors de la grille.
    assert!(
        le_monde_echoue,
        "la typographie du monde passait déjà : l'épreuve ne distingue rien"
    );
}

/// La première et la dernière colonne où une variante dépose de l'encre, relatives à la plume.
fn colonnes_d_encre(typo: &Typography, ch: char, size: f32, phase: u8) -> Option<(i32, i32)> {
    let g = typo.glyph_variant(ch, size, Face::Regular, phase);
    let colonnes: Vec<i32> = (0..g.width)
        .filter(|c| (0..g.height).any(|r| g.bitmap[r * g.width + c] > 0))
        .map(|c| g.metrics.xmin + c as i32)
        .collect();
    Some((*colonnes.first()?, *colonnes.last()?))
}

#[test]
fn test_grille_1_l_encre_ne_sort_jamais_de_la_boite_du_contour_decale() {
    let monde = Typography::new();
    let ecran = monde.posee_sur_l_ecran();
    let mut le_monde_deborde = false;
    for ch in "lHmo|iT".chars() {
        for size in [11.0, 12.0, 13.5, 16.0] {
            let b = ecran.font(Face::Regular).metrics(ch, size).bounds;
            for phase in 0..super::super::glyph::SUBPIXEL_PHASES {
                let decalage = f32::from(phase) / SOUS as f32;
                let boite = (
                    (b.xmin + decalage).floor() as i32,
                    (b.xmin + b.width + decalage).ceil() as i32 - 1,
                );
                let (g, d) = colonnes_d_encre(&ecran, ch, size, phase).unwrap();
                assert!(
                    g >= boite.0 && d <= boite.1,
                    "« {ch} » au corps {size}, phase {phase} : encre de {g} à {d}, boîte {boite:?}"
                );
                let (g, d) = colonnes_d_encre(&monde, ch, size, phase).unwrap();
                le_monde_deborde |= g < boite.0 || d > boite.1;
            }
        }
    }
    // La preuve à l'envers : le ré-échantillonnage de GLYPH-1 étale l'encre d'une colonne.
    assert!(
        le_monde_deborde,
        "le ré-échantillonnage restait dans la boîte : rien n'est éprouvé"
    );
}

#[test]
fn test_grille_1_la_mesure_et_les_avances_ne_changent_pas() {
    let monde = Typography::new();
    let ecran = monde.posee_sur_l_ecran();
    let texte = "Mood board · Time Machine — été 2026 ✓";
    for size in [11.0, 12.0, 12.5, 15.0] {
        for face in Face::ALL {
            assert_eq!(
                monde.measure_text(texte, size, face),
                ecran.measure_text(texte, size, face)
            );
        }
        let style = TextStyle {
            size,
            color: Color::WHITE,
            face: Face::Regular,
        };
        let mut p = Pixmap::new(400, 40).unwrap();
        let fin_monde = monde.draw_text(&mut p.as_mut(), texte, 2.3, 4.6, style);
        let fin_ecran = ecran.draw_text(&mut p.as_mut(), texte, 2.3, 4.6, style);
        assert_eq!(
            fin_monde, fin_ecran,
            "le texte finit au même endroit sur les deux voies"
        );
    }
}

#[test]
fn test_grille_1_les_deux_typographies_partagent_leurs_polices() {
    let monde = Typography::new();
    let ecran = monde.posee_sur_l_ecran();
    assert!(std::rc::Rc::ptr_eq(&monde.polices, &ecran.polices));
    assert!(ecran.sur_l_ecran && !monde.sur_l_ecran);
}
