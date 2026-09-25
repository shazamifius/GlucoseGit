//! Où un passage se trouve dans une carte.

use super::*;
use crate::renderer::math::MathRenderer;

const SA_CARTE: &str = "bonjours\ntest\ntest\nbonjours";

fn rects(plages: &[(usize, usize)]) -> Vec<(f32, f32, f32, f32)> {
    rectangles(
        (&Typography::new(), &MathRenderer::new()),
        SA_CARTE,
        240.0,
        plages,
    )
}

/// **Le second « bonjours » est sur la quatrième ligne, et seulement là** ; sa largeur est
/// celle du mot.
#[test]
fn test_fleche_4_le_passage_est_la_ou_est_le_mot() {
    let second = SA_CARTE.rfind("bonjours").expect("le second");
    let r = rects(&[(second, second + 8)]);
    assert_eq!(r.len(), 1, "une ligne");
    let ligne = text_box(240.0).line_height;
    let (x, y, w, h) = r[0];
    assert_eq!(
        (x, y, h),
        (TEXT_ORIGIN.0, TEXT_ORIGIN.1 + 3.0 * ligne, ligne)
    );
    let (mot, _) = Typography::new().measure_text(
        "bonjours",
        text_box(240.0).body,
        crate::typography::Face::Regular,
    );
    assert!((w - mot).abs() < 0.01, "{w} contre {mot}");
}

/// **Un passage à cheval sur deux lignes rend deux rectangles**, un par ligne.
#[test]
fn test_fleche_4_un_passage_sur_deux_lignes() {
    let r = rects(&[(2, 12)]);
    assert_eq!(r.len(), 2, "{r:?}");
    assert!(r[1].1 > r[0].1);
}
