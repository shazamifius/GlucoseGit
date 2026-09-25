//! **Le rectangle arrondi** — un seul traceur pour toute l'application (ARC-1) : le rendu, les
//! panneaux et les icônes l'empruntent. Il en existait trois copies, identiques au caractère
//! près, dans ces trois modules.

use tiny_skia::PathBuilder;

/// Ajoute un rectangle à coins arrondis dans un PathBuilder — **le seul** de l'application :
/// l'interface, les panneaux et les icônes l'empruntent.
///
/// Le rayon est ramené à la moitié du plus petit côté : c'est une contrainte **géométrique**
/// — un coin ne peut pas être plus rond que la forme — et non une borne sur une valeur
/// dérivée du zoom, puisque rayon et côtés subissent la même mise à l'échelle.
///
/// # Des arcs de cercle, comme le `border-radius` de Glucose Tauri
///
/// Les coins étaient des Bézier **quadratiques** dont le point de contrôle est le coin : une
/// parabole, qui bombe de six pour cent du rayon au milieu du coin — deux pixels sur une
/// carte à l'échelle 1, vingt à l'échelle 10. Ce sont maintenant des cubiques dont les points
/// de contrôle sont à [`ARC`] du rayon : un quart de cercle à 0,03 % près (MEMB-FORME-1 avait
/// fait de même pour les membranes, par leur distance exacte).
pub(crate) fn push_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = r * ARC;
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
    pb.line_to(x, y + r);
    pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    pb.close();
}

/// **Où poser les points de contrôle d'un quart de cercle**, en fraction du rayon :
/// `4/3 · tan(π/8) = 4/3 · (√2 − 1)`. C'est la seule cubique qui part et arrive tangente aux
/// deux côtés et passe par le milieu de l'arc ; l'écart au cercle ne dépasse nulle part
/// 0,03 % du rayon. Un nombre déduit, pas choisi.
const ARC: f32 = 4.0 / 3.0 * (std::f32::consts::SQRT_2 - 1.0);

#[cfg(test)]
mod tests {
    use super::*;

    /// **Les coins sont des quarts de cercle** (ARC-1) : chaque point de chaque coin est à la
    /// distance du rayon de son centre, à 0,03 % près — une parabole s'en écartait de 6 %.
    #[test]
    fn test_arc_1_les_coins_sont_des_quarts_de_cercle() {
        let (x, y, w, h, r) = (10.0_f32, 20.0, 300.0, 200.0, 50.0);
        let mut pb = PathBuilder::new();
        push_rounded_rect(&mut pb, x, y, w, h, r);
        let chemin = pb.finish().expect("un chemin");
        let centres = [
            (x + w - r, y + r),
            (x + w - r, y + h - r),
            (x + r, y + h - r),
            (x + r, y + r),
        ];
        let mut depart = tiny_skia::Point::zero();
        let mut coins = 0;
        for segment in chemin.segments() {
            match segment {
                tiny_skia::PathSegment::MoveTo(p) | tiny_skia::PathSegment::LineTo(p) => {
                    depart = p;
                }
                tiny_skia::PathSegment::CubicTo(a, b, p) => {
                    let (cx, cy) = centres[coins];
                    for i in 0..=64 {
                        let t = i as f32 / 64.0;
                        let u = 1.0 - t;
                        let (k0, k1, k2, k3) =
                            (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                        let px = k0 * depart.x + k1 * a.x + k2 * b.x + k3 * p.x;
                        let py = k0 * depart.y + k1 * a.y + k2 * b.y + k3 * p.y;
                        let ecart = ((px - cx).hypot(py - cy) - r).abs() / r;
                        assert!(ecart < 3e-4, "coin {coins}, t = {t} : écart {ecart}");
                    }
                    depart = p;
                    coins += 1;
                }
                tiny_skia::PathSegment::Close => {}
                other => panic!("un segment inattendu : {other:?}"),
            }
        }
        assert_eq!(coins, 4, "quatre coins");
    }
}
