//! Symbiose chromatique — calcul pur de teinte organique selon la position et le voisinage.
//!
//! 100% Rust std (0 dépendance externe).

use crate::types::Annotation;

/// Hash déterministe pour dériver une variation d'angle propre à chaque bloc.
fn id_hash(id: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in id.bytes() {
        h = ((h << 5).wrapping_add(h)).wrapping_add(b as u64);
    }
    h
}

/// Lissage cubique doux : t * t * (3 - 2 * t).
#[inline(always)]
fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Pseudo-aléatoire 2D déterministe continu.
#[inline(always)]
fn random2d(ix: f64, iy: f64) -> f64 {
    let dot = ix * 12.9898 + iy * 78.233;
    let sin = (dot.sin() * 43758.5453).fract();
    sin.abs()
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
    let id_offset = ((id_hash(aid) % 80) as f64) - 40.0;
    let mut my_base_hue = (get_zone_hue(ax, ay) + id_offset).rem_euclid(360.0);

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
            let other_id_offset = ((id_hash(other.id()) % 80) as f64) - 40.0;
            let other_hue = (get_zone_hue(other.x(), other.y()) + other_id_offset).rem_euclid(360.0);

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
