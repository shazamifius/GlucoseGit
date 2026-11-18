//! Les épreuves de GRILLE-2, avec leur preuve à l'envers : le tracé d'avant, rejoué sur les mêmes
//! données, conclut l'inverse.

use super::super::{draw_icon_scaled, IconType};
use tiny_skia::{Color, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// L'icône « Exporter » (une grille de 16 unités) : sa tige verticale `M8 2v8` et son socle
/// horizontal `M2 13h12`.
const TIGE_X: f32 = 8.0;
const SOCLE_Y: f32 = 13.0;

/// La couverture des pixels qui ont de l'encre, le long d'une rangée (`Some(y)`, colonnes de
/// `de` à `a`) ou d'une colonne (`None` : la colonne `fixe`, rangées de `de` à `a`).
fn encre(p: &Pixmap, rangee: Option<u32>, fixe: u32, (de, a): (f32, f32)) -> Vec<u8> {
    let alpha = |x: u32, y: u32| p.pixel(x, y).map_or(0, |c| c.alpha());
    (de.max(0.0) as u32..=a as u32)
        .map(|i| match rangee {
            Some(y) => alpha(i, y),
            None => alpha(fixe, i),
        })
        .filter(|a| *a > 8)
        .collect()
}

/// **Ce que la loi exige** : à mi-tige, autant de colonnes d'encre que le trait a de pixels, toutes
/// pleines ; au milieu du socle, autant de rangées, toutes pleines. Chaque mesure se borne au
/// voisinage de son trait : la pointe de la flèche n'y entre pas.
fn sur_la_grille(p: &Pixmap, (x, y, s): (f32, f32, f32)) -> Result<(), String> {
    let k = 14.0 * s / 16.0;
    let largeur = (1.3 * s).round().max(1.0);
    let marge = largeur / 2.0 + 1.0;
    let autour = |c: f32| (c - marge, c + marge);
    let tige = encre(p, Some((y + 5.0 * k) as u32), 0, autour(x + TIGE_X * k));
    let socle = encre(p, None, (x + 4.0 * k) as u32, autour(y + SOCLE_Y * k));
    let pleines = |v: &[u8]| v.len() == largeur as usize && v.iter().all(|a| *a >= 247);
    if pleines(&tige) && pleines(&socle) {
        Ok(())
    } else {
        Err(format!(
            "échelle {s}, coin ({x}, {y}) : tige {tige:?}, socle {socle:?}"
        ))
    }
}

/// Le tracé d'avant GRILLE-2 : la tige et le socle tracés tels quels, à 1,3 pixel.
fn trace_d_avant(p: &mut Pixmap, (x, y, s): (f32, f32, f32)) {
    let mut pb = PathBuilder::new();
    pb.move_to(TIGE_X, 2.0);
    pb.line_to(TIGE_X, 10.0);
    pb.move_to(2.0, SOCLE_Y);
    pb.line_to(14.0, SOCLE_Y);
    let path = pb.finish().unwrap();
    let mut paint = Paint::default();
    paint.set_color(Color::WHITE);
    let k = 14.0 * s / 16.0;
    let stroke = Stroke {
        width: 1.3 * s / k,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Default::default()
    };
    let ts = Transform::from_scale(k, k).post_translate(x, y);
    p.as_mut().stroke_path(&path, &paint, &stroke, ts, None);
}

#[test]
fn test_grille_2_un_trait_d_icone_couvre_des_pixels_entiers() {
    let mut l_avant_echoue = false;
    for s in [1.0, 1.25, 1.5, 2.0] {
        for x in [10.0, 10.3, 10.5, 10.8] {
            for y in [4.0, 4.6] {
                let mut p = Pixmap::new(64, 64).unwrap();
                let couleur = Color::WHITE;
                draw_icon_scaled(
                    &mut p.as_mut(),
                    IconType::Export,
                    x,
                    y,
                    14.0 * s,
                    couleur,
                    1.3 * s,
                );
                if let Err(faute) = sur_la_grille(&p, (x, y, s)) {
                    panic!("une icône hors de la grille : {faute}");
                }
                let mut avant = Pixmap::new(64, 64).unwrap();
                trace_d_avant(&mut avant, (x, y, s));
                l_avant_echoue |= sur_la_grille(&avant, (x, y, s)).is_err();
            }
        }
    }
    assert!(
        l_avant_echoue,
        "le tracé d'avant passait déjà : l'épreuve ne distingue rien"
    );
}
