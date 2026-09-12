//! Symbiose chromatique — calcul pur de teinte organique selon la position et le voisinage.
//!
//! 100% Rust std (0 dépendance externe).
//!
//! # Fidélité à la référence, au bit près
//!
//! Une carte doit avoir **la même couleur** dans Glucose Rust que dans Glucose Tauri : la
//! teinte est une fonction chaotique de l'identifiant et de la position, et le moindre écart
//! d'arithmétique donne une couleur sans rapport. La référence est du JavaScript, donc du
//! `f64` — pas le `f32` que la fiche 06 écrivait — avec deux particularités reproduites ici :
//!
//! - `idHash` calcule `((h << 5) + h) + code` où `<<` tronque `h` en **int32 signé** mais
//!   où les additions restent en flottant : le hash dépasse 2³² sans jamais boucler. Ni un
//!   `u32` ni un `u64` en arithmétique modulaire ne donnent la même chose.
//! - `random2D` rend `sin − ⌊sin⌋`, toujours dans `[0, 1)` — pas `|fract(sin)|`, qui replie
//!   les négatifs sur eux-mêmes.
//!
//! `test_the_hue_matches_the_reference_on_its_vectors` tient six valeurs produites par la
//! source TypeScript exécutée telle quelle.

use crate::types::Annotation;

/// `ToInt32` d'ECMAScript pour un entier fini : modulo 2³², vers le signé.
#[inline(always)]
fn to_int32(v: f64) -> i32 {
    v.rem_euclid(4_294_967_296.0) as u32 as i32
}

/// Le `idHash` de la référence, tel que JavaScript l'évalue : `h << 5` en int32, le reste en
/// flottant, unités UTF-16 comme `charCodeAt`, valeur absolue à la fin.
fn id_hash(id: &str) -> f64 {
    let mut h: f64 = 5381.0;
    for unit in id.encode_utf16() {
        let shifted = to_int32(h).wrapping_shl(5);
        h = f64::from(shifted) + h + f64::from(unit);
    }
    h.abs()
}

/// Variation d'angle propre à un bloc : `(idHash % 80) − 40`, dans `[−40, 39]`.
#[inline(always)]
fn id_offset(id: &str) -> f64 {
    id_hash(id) % 80.0 - 40.0
}

/// Lissage cubique doux : t * t * (3 - 2 * t).
#[inline(always)]
fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Pseudo-aléatoire 2D déterministe continu, dans `[0, 1)`.
#[inline(always)]
fn random2d(ix: f64, iy: f64) -> f64 {
    let dot = ix * 12.9898 + iy * 78.233;
    let sin = dot.sin() * 43758.5453;
    sin - sin.floor()
}

/// Calcule la teinte de base (0° à 360°) à une position (x, y) dans l'espace infini.
pub fn get_zone_hue(x: f64, y: f64) -> f64 {
    let scale = 2000.0; // Les zones de couleur changent tous les ~2000 pixels
    let cx = x / scale;
    let cy = y / scale;

    let x0 = cx.floor();
    let x1 = x0 + 1.0;
    let y0 = cy.floor();
    let y1 = y0 + 1.0;

    let sx = smooth(cx - x0);
    let sy = smooth(cy - y0);

    let nx0 = random2d(x0, y0) * (1.0 - sx) + random2d(x1, y0) * sx;
    let nx1 = random2d(x0, y1) * (1.0 - sx) + random2d(x1, y1) * sx;
    let value = nx0 * (1.0 - sy) + nx1 * sy;

    (value * 360.0).clamp(0.0, 360.0)
}

/// Calcule la teinte symbiotique d'une annotation en tenant compte de sa position
/// et de l'influence vectorielle circulaire de ses voisines.
pub fn get_symbiotic_hue(ann: &Annotation, all_annotations: &[Annotation]) -> f64 {
    let (ax, ay, aid) = (ann.x(), ann.y(), ann.id());

    // 1. Teinte de base du biome + variation individuelle
    let mut my_base_hue = (get_zone_hue(ax, ay) + id_offset(aid)).rem_euclid(360.0);

    // 2. Moyenne vectorielle circulaire des voisines dans un rayon de 1200px
    const RAYON: f64 = 1200.0;
    let mut sum_x = 0.0f64;
    let mut sum_y = 0.0f64;
    let mut env_weight_sum = 0.0f64;

    for other in all_annotations {
        if other.id() == aid || !matches!(other, Annotation::Text { .. }) {
            continue;
        }

        let dx = ax - other.x();
        if dx.abs() > RAYON {
            continue;
        }
        let dy = ay - other.y();
        if dy.abs() > RAYON {
            continue;
        }

        let dist = (dx * dx + dy * dy).sqrt();
        if dist < RAYON {
            let weight = (1.0 - (dist / RAYON)).powi(2);
            let other_hue = get_zone_hue(other.x(), other.y()) + id_offset(other.id());

            let rad = other_hue.to_radians();
            sum_x += rad.cos() * weight;
            sum_y += rad.sin() * weight;
            env_weight_sum += weight;
        }
    }

    // 3. Attraction symbiotique
    if env_weight_sum > 0.0 {
        let mut env_hue = sum_y.atan2(sum_x).to_degrees();
        if env_hue < 0.0 {
            env_hue += 360.0;
        }

        let mut diff = env_hue - my_base_hue;
        if diff > 180.0 {
            diff -= 360.0;
        }
        if diff < -180.0 {
            diff += 360.0;
        }

        let influence = 0.5 * (1.0 - 1.0 / (1.0 + env_weight_sum));
        my_base_hue += diff * influence;
    }

    my_base_hue.rem_euclid(360.0)
}

/// Convertit une teinte HSL (h en degrés [0, 360], s dans [0, 1], l dans [0, 1]) en RGB [0, 255].
pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let h = h.rem_euclid(360.0);
    let s = s.clamp(0.0, 1.0);
    let l = l.clamp(0.0, 1.0);

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = l - c / 2.0;

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((g_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        ((b_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    /// Six valeurs produites par `src/utils/symbioticHue.ts` exécuté par Node, à 17 chiffres.
    /// Le dernier identifiant fait dépasser 2³² au hash : c'est lui qui distingue
    /// l'arithmétique JavaScript d'un `u32` ou d'un `u64`.
    #[test]
    fn test_the_hue_matches_the_reference_on_its_vectors() {
        use super::*;
        use crate::types::Annotation;
        fn text(id: &str, x: f64, y: f64) -> Annotation {
            Annotation::Text {
                id: id.into(),
                x,
                y,
                width: None,
                height: None,
                text: String::new(),
                font_size: None,
                color: None,
                cursor_pos: None,
                source_file: None,
                membrane_id: None,
                domains: Vec::new(),
                mirror_of: None,
                temporal_anchor: None,
            }
        }
        let long = "un-identifiant-assez-long-pour-deborder-trente-deux-bits";
        let anns = vec![
            text("welcome-card", 0.0, 0.0),
            text("ann-7", 350.0, -120.0),
            text("img-42", -900.0, 400.0),
            text("mirror-folder-7", 2500.0, 2500.0),
            Annotation::Sticky {
                id: "s-1".into(),
                x: 100.0,
                y: 100.0,
                width: None,
                height: None,
                text: String::new(),
                font_size: None,
                color: None,
                bg_color: None,
                cursor_pos: None,
                operator: None,
                source_file: None,
                membrane_id: None,
                domains: Vec::new(),
                mirror_of: None,
                temporal_anchor: None,
            },
            text(long, 1234.5, -987.25),
        ];
        let expected = [
            ("welcome-card", 1644031048.0, 332.72058264985753),
            ("ann-7", 253260966.0, 351.19852975857077),
            ("img-42", 79322901.0, 359.27187202248672),
            ("mirror-folder-7", 810722093.0, 196.17118597762749),
            ("s-1", 193503766.0, 331.84629458576660),
            (long, 14715327716.0, 245.21736873941506),
        ];
        for (ann, (id, hash, hue)) in anns.iter().zip(expected) {
            assert_eq!(ann.id(), id);
            assert_eq!(id_hash(id), hash, "idHash({id})");
            let got = get_symbiotic_hue(ann, &anns);
            assert!((got - hue).abs() < 1e-6, "{id} : {got} au lieu de {hue}");
        }
    }

    use super::*;

    #[test]
    fn test_zone_hue_is_continuous_and_in_range() {
        let h1 = get_zone_hue(0.0, 0.0);
        let h2 = get_zone_hue(10.0, 10.0);
        let h3 = get_zone_hue(2000.0, 2000.0);

        assert!((0.0..=360.0).contains(&h1));
        assert!((0.0..=360.0).contains(&h2));
        assert!((0.0..=360.0).contains(&h3));
        // Continuité : petit déplacement donne petit delta
        assert!((h1 - h2).abs() < 15.0);
    }

    #[test]
    fn test_hsl_to_rgb_primaries() {
        assert_eq!(hsl_to_rgb(0.0, 1.0, 0.5), (255, 0, 0));
        assert_eq!(hsl_to_rgb(120.0, 1.0, 0.5), (0, 255, 0));
        assert_eq!(hsl_to_rgb(240.0, 1.0, 0.5), (0, 0, 255));
    }
}
