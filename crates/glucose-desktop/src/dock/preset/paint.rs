//! Le dessin du panneau MOOD BOARD (« Presets » dans Glucose Tauri, renommé à sa parole, fiche 71).

use super::{layout_presets_panel, Contenu, PresetRowLayout, SectionSuivie, CREATE_LABEL};
use crate::dock::paint::{Brush, ButtonLook};
use crate::params::ScaledRect;
use crate::typography::Face;
use glucose_core::gabarits::Choix;
use glucose_core::store::Store;
use tiny_skia::{Color, PixmapMut};

/// L'opacité du fond d'une catégorie, sur 255 — son contour, lui, est à la couleur pleine.
const SLOT_FILL_ALPHA: u8 = 55;

pub fn render_presets_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    frame: ScaledRect,
    store: &Store,
) {
    let contenu = Contenu::du(store);
    let layout = layout_presets_panel(frame, &contenu);
    brush.title(pixmap, frame, "MOOD BOARD");
    if let (Some(section), Some((nom, pastilles))) = (&layout.suivie, &contenu.suivi) {
        draw_suivi(pixmap, brush, section, (&contenu.tableau, nom, pastilles));
    }
    let legende = if contenu.suivi.is_some() {
        "CHANGER DE MOOD BOARD".to_string()
    } else {
        format!(
            "CHOISIR UN MOOD BOARD POUR « {} »",
            contenu.tableau.to_uppercase()
        )
    };
    brush.text(
        pixmap,
        &legende,
        layout.caption_at,
        9.0,
        brush.theme.text_muted,
        Face::Bold,
    );
    for row in &layout.rows {
        draw_preset(pixmap, brush, row, &contenu.choix[row.index]);
    }
    let look = ButtonLook {
        active: false,
        disabled: true,
        quiet: false,
    };
    brush.button(pixmap, layout.create_button, CREATE_LABEL, 10.5, look);
}

/// Le gabarit que suit le tableau : sa légende, son nom, « Retirer », et ses pastilles —
/// pleines pour une catégorie où une image est posée.
fn draw_suivi(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    section: &SectionSuivie,
    (tableau, nom, pastilles): (&str, &str, &[super::Pastille]),
) {
    let theme = brush.theme;
    let legende = format!("TABLEAU ACTIF : {}", tableau.to_uppercase());
    brush.text(
        pixmap,
        &legende,
        section.legende_at,
        9.0,
        theme.text_muted,
        Face::Bold,
    );
    brush.text(
        pixmap,
        nom,
        section.nom_at,
        11.5,
        theme.text_primary,
        Face::Bold,
    );
    let look = ButtonLook {
        active: false,
        disabled: false,
        quiet: false,
    };
    brush.button(pixmap, section.retirer, "Retirer", 10.0, look);
    for (
        rect,
        super::Pastille {
            nom,
            couleur,
            remplie,
        },
    ) in section.pastilles.iter().zip(pastilles)
    {
        let couleur = rgb(*couleur, 255);
        if *remplie {
            brush.fill(
                pixmap,
                *rect,
                brush.px(9.0),
                rgb_de(couleur, SLOT_FILL_ALPHA),
            );
            brush.stroke(pixmap, *rect, brush.px(9.0), couleur, brush.px(0.8));
            brush.text_centered(pixmap, *rect, nom, 8.5, couleur, Face::Regular);
        } else {
            brush.stroke(
                pixmap,
                *rect,
                brush.px(9.0),
                theme.border_medium,
                brush.px(0.8),
            );
            brush.text_centered(pixmap, *rect, nom, 8.5, theme.text_muted, Face::Regular);
        }
    }
}

/// Un gabarit : son titre, la rangée de ses catégories, sa phrase.
fn draw_preset(pixmap: &mut PixmapMut, brush: &Brush, row: &PresetRowLayout, choix: &Choix<'_>) {
    let theme = brush.theme;
    if brush.hovered(row.rect) {
        brush.fill(pixmap, row.rect, brush.px(3.0), theme.bg_hover);
    }
    brush.text(
        pixmap,
        choix.nom,
        row.title_at,
        11.5,
        theme.text_primary,
        Face::Bold,
    );
    for (rect, (_, nom, couleur)) in row.slots.iter().zip(&choix.categories) {
        let couleur = rgb(*couleur, 255);
        brush.fill(
            pixmap,
            *rect,
            brush.px(2.0),
            rgb_de(couleur, SLOT_FILL_ALPHA),
        );
        brush.stroke(pixmap, *rect, brush.px(2.0), couleur, brush.px(0.8));
        brush.text_centered_if_fits(pixmap, *rect, nom, 7.5, couleur);
    }
    brush.text(
        pixmap,
        choix.description,
        row.summary_at,
        9.0,
        theme.text_muted,
        Face::Regular,
    );
}

fn rgb(rgb: u32, alpha: u8) -> Color {
    Color::from_rgba8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, alpha)
}

fn rgb_de(couleur: Color, alpha: u8) -> Color {
    let c = couleur.to_color_u8();
    Color::from_rgba8(c.red(), c.green(), c.blue(), alpha)
}
