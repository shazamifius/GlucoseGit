//! **Les images en SVG** (P5, fiche 69) : la sœur de [`super::draw_images`] — chaque image à sa
//! boîte, ses octets d'origine en `data:`, son recadrage et sa rotation dits comme le rendu les
//! dit.
//!
//! Le recadrage (RECADRAGE-1) ne se recalcule pas : la source **entière** se pose sur sa boîte
//! agrandie ([`super::boite_source`], la loi même du rendu), découpée par la boîte du nœud ; le
//! tout tourne autour du centre du nœud. Une image dont les octets ne se lisent pas laisse un
//! cadre gris à sa place — elle n'est pas oubliée, elle manque.

use super::prevision::boite_ecran;
use crate::params::ViewPass;
use crate::renderer::svg::{base64, n, Svg};
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::BoardImage;

/// Le type d'image que ces octets portent, lu dans leurs premiers octets.
pub fn type_d_image(octets: &[u8]) -> Option<&'static str> {
    if octets.starts_with(&[0x89, b'P', b'N', b'G']) {
        Some("image/png")
    } else if octets.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if octets.starts_with(b"GIF8") {
        Some("image/gif")
    } else if octets.starts_with(b"RIFF") && octets.get(8..12) == Some(b"WEBP") {
        Some("image/webp")
    } else if octets.starts_with(b"BM") {
        Some("image/bmp")
    } else {
        None
    }
}

/// **Écrit chaque image visible** ; `octets` rend ceux d'une image, s'ils se lisent.
pub(in crate::renderer) fn images_en_svg(
    store: &Store,
    pass: ViewPass<'_>,
    svg: &mut Svg,
    octets: &mut dyn FnMut(&BoardImage) -> Option<Vec<u8>>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    for img in Visibles::nouvelles(pass.visibles, board).images() {
        let ecran = boite_ecran(img, &pass);
        let (x, y, l, h) = ecran;
        let tourne = if img.rotation == 0.0 {
            String::new()
        } else {
            format!(
                r#" transform="rotate({} {} {})""#,
                n(img.rotation.to_degrees() as f32),
                n(x + l / 2.0),
                n(y + h / 2.0)
            )
        };
        let lu = octets(img).and_then(|o| type_d_image(&o).map(|t| (t, o)));
        let Some((genre, octets)) = lu else {
            svg.poser(&format!(
                r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#262626" stroke="#555" stroke-width="1"{tourne}/>"##,
                n(x), n(y), n(l), n(h)
            ));
            continue;
        };
        let (vx, vy, vl, vh) = super::boite_source(img, ecran);
        let decoupe = svg.nom("cadre");
        svg.definir(&format!(
            r#"<clipPath id="{decoupe}"><rect x="{}" y="{}" width="{}" height="{}"/></clipPath>"#,
            n(x),
            n(y),
            n(l),
            n(h)
        ));
        svg.poser(&format!(
            r#"<g clip-path="url(#{decoupe})"{tourne}><image x="{}" y="{}" width="{}" height="{}" preserveAspectRatio="none" xlink:href="data:{genre};base64,{}"/></g>"#,
            n(vx), n(vy), n(vl), n(vh), base64(&octets)
        ));
    }
}
