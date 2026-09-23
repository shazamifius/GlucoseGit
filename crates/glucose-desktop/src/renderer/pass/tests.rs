//! Ce que la passe des annotations promet des liens trans-domaines (fiche 03 § 11.6) : en
//! pointillés quand ils se montrent, absents quand le bouton est coupé — et un lien qui reste
//! dans son domaine, toujours plein.

use super::*;
use crate::renderer::math::MathRenderer;
use crate::renderer::DomainTints;
use glucose_core::types::{Board, BoardImage};
use tiny_skia::{Color, Pixmap};

/// Le fond de la toile, pour reconnaître ce qui a été peint.
const FOND: (u8, u8, u8) = (13, 14, 18);

/// Deux photos, l'une en `(150, 300)`, l'autre en `(650, 300)`, qui portent ces domaines, et
/// une flèche accrochée de l'une à l'autre : sa tige court le long de `y = 300`.
fn plateau(de: &str, vers: &str) -> Board {
    let mut board = Board::new("b", "plateau");
    for (id, x, domaine) in [("a", 150.0, de), ("b", 650.0, vers)] {
        let mut photo = BoardImage::new(id, x, 300.0, 100.0, 100.0);
        photo.domains.push(DomainAssignment {
            domain_id: domaine.to_string(),
            weight: 1.0,
        });
        board.images.push(photo);
    }
    let mut fleche = Annotation::arrow("f", 150.0, 300.0, 650.0, 300.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut fleche
    {
        *source_id = Some("a".into());
        *target_id = Some("b".into());
    }
    board.annotations.push(fleche);
    board
}

/// Combien de tronçons d'encre la tige laisse sur `y = 300`, entre les deux photos.
fn troncons(board: &Board, trans_domaines: bool) -> usize {
    let mut pixmap = Pixmap::new(800, 600).expect("pixmap de test");
    pixmap.fill(Color::from_rgba8(FOND.0, FOND.1, FOND.2, 255));
    let (typography, math) = (Typography::new(), MathRenderer::new());
    let (tints, theme) = (DomainTints::default(), Theme::dark());
    let vp = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    let ctx = Pass {
        typography: &typography,
        math: &math,
        tints: &tints,
        theme: &theme,
        vp,
        scale: WorldScale::new(vp.scale),
        clip: Clip {
            width: 800.0,
            height: 600.0,
            top: 0.0,
        },
        trans_domaines,
    };
    draw_arrow_node(
        &ctx,
        &mut pixmap.as_mut(),
        &board.annotations[0],
        board,
        false,
        None,
    );
    let encre: Vec<bool> = (215..585)
        .map(|x| {
            let p = pixmap.pixel(x, 300).expect("un pixel");
            (p.red(), p.green(), p.blue()) != FOND
        })
        .collect();
    usize::from(encre[0]) + encre.windows(2).filter(|w| !w[0] && w[1]).count()
}

/// **Un lien qui reste dans son domaine est plein** ; **un lien qui en traverse un se dessine
/// en pointillés** — trois cent soixante-dix pixels de tige, donc des dizaines de tirets.
#[test]
fn test_un_lien_trans_domaine_se_dessine_en_pointilles() {
    assert_eq!(
        troncons(&plateau("sciences", "sciences"), true),
        1,
        "un trait plein"
    );
    let tirets = troncons(&plateau("sciences", "arts"), true);
    assert!(tirets > 20, "des pointillés : {tirets} tronçon(s)");
}

/// **Le bouton coupé, le lien trans-domaine disparaît** — et le lien intra-domaine reste.
#[test]
fn test_le_bouton_coupe_masque_les_seuls_liens_trans_domaines() {
    assert_eq!(troncons(&plateau("sciences", "arts"), false), 0, "masqué");
    assert_eq!(
        troncons(&plateau("sciences", "sciences"), false),
        1,
        "toujours là"
    );
}
