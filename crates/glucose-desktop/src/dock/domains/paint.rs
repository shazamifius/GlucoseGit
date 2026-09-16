//! Le dessin du panneau DOMAINES.
//!
//! Séparé de [`super`] parce que la mise en page et le test de clic sont une chose — la
//! liste unique de widgets de DOM-UI-2 — et que poser des pixels en est une autre. Ce module
//! ne décide de rien : il lit la liste que [`super::layout_domains_panel`] a produite et le
//! document, et il peint. Aucune coordonnée n'est recalculée ici (loi L4).

use super::{
    layout_domains_panel, DomainRowLayout, DomainsPanelLayout, DomainsUi, Metrics, WEIGHT_STEPS,
};
use crate::params::ScaledRect;
use crate::typography::{Face, TextStyle};
use glucose_core::store::Store;
use glucose_core::types::Domain;
use tiny_skia::{Color, PixmapMut};

use crate::dock::paint::Brush;
use crate::dock::WidgetRect;

/// Ce que le dessin du panneau garde constant.
/// Ce que le dessin du panneau garde constant.
///
/// Il emprunte le [`Brush`] du dock plutôt que de recopier ses champs : c'était un second
/// pinceau, et il écrivait son texte en court-circuitant le premier. Tant que les deux
/// existaient, l'origine du tampon d'un panneau mis en cache n'était appliquée que par l'un
/// des deux, et le texte des domaines atterrissait à côté (DOCK-CACHE-1).
struct Painter<'a> {
    brush: &'a Brush<'a>,
    m: Metrics,
}

/// Dessine le panneau : le catalogue du document, l'état de saisie, les actions.
pub fn render_domains_panel(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &DomainsUi,
    brush: &Brush<'_>,
    frame: ScaledRect,
) {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let p = Painter {
        brush,
        m: Metrics::scaled(s),
    };
    let layout = layout_domains_panel(frame, store, ui);
    let selected = store.selected_annotation_ids.len() + store.selected_image_ids.len();

    draw_header(&p, pixmap, frame, store.project.domains.len(), selected);
    if store.project.domains.is_empty() {
        draw_empty_state(&p, pixmap, frame);
    }
    for row in &layout.rows {
        let Some(domain) = store.project.domains.get(row.index) else {
            continue;
        };
        draw_row(&p, pixmap, row, domain, ui, selected > 0);
    }
    draw_footer(&p, pixmap, frame, &layout, selected);
}

fn draw_header(
    p: &Painter,
    pixmap: &mut PixmapMut,
    frame: ScaledRect,
    domains: usize,
    selected: usize,
) {
    let m = &p.m;
    p.brush.text_styled(
        pixmap,
        "DOMAINES",
        frame.x + m.pad,
        frame.y + m.title_baseline,
        TextStyle {
            size: m.title_size,
            color: p.brush.theme.text_primary,
            face: Face::Bold,
        },
    );
    let summary = format!("{domains} domaine(s) · {selected} nœud(s) sélectionné(s)");
    p.brush.text_styled(
        pixmap,
        &summary,
        frame.x + m.pad,
        frame.y + m.header_baseline,
        TextStyle {
            size: m.hint_size,
            color: p.brush.theme.text_muted,
            face: Face::Regular,
        },
    );
}

fn draw_empty_state(p: &Painter, pixmap: &mut PixmapMut, frame: ScaledRect) {
    let m = &p.m;
    let lines = [
        "Aucun domaine.",
        "Crée-en un, puis assigne-le a ta selection :",
        "sa reglette apparait au-dessus du noeud.",
    ];
    let mut y = frame.y + m.rows_top + m.row_gap;
    for (index, line) in lines.iter().enumerate() {
        let size = if index == 0 { m.name_size } else { m.hint_size };
        let (width, _) = p.brush.typo.measure_text(line, size, Face::Regular);
        p.brush.text_styled(
            pixmap,
            line,
            frame.x + (frame.w - width) / 2.0,
            y,
            TextStyle {
                size,
                color: p.brush.theme.text_muted,
                face: Face::Regular,
            },
        );
        y += m.line_gap * 0.8;
    }
}

fn draw_row(
    p: &Painter,
    pixmap: &mut PixmapMut,
    row: &DomainRowLayout,
    domain: &Domain,
    ui: &DomainsUi,
    enabled: bool,
) {
    let m = &p.m;
    let (r, g, b) = crate::renderer::parse_hex_color(
        &domain.color,
        (p.brush.theme.domain_fallback.red() * 255.0) as u8,
        (p.brush.theme.domain_fallback.green() * 255.0) as u8,
        (p.brush.theme.domain_fallback.blue() * 255.0) as u8,
    );
    let tint = Color::from_rgba8(r, g, b, 255);

    draw_mark(p, pixmap, row, domain, (r, g, b));
    draw_name(p, pixmap, row, domain, ui);

    // Croix de suppression.
    let delete_hot =
        row.delete.contains(p.brush.pointer.x, p.brush.pointer.y) || row.confirm.is_some();
    let cross = if delete_hot {
        p.brush.theme.text_primary
    } else {
        p.brush.theme.text_muted
    };
    p.brush.text_styled(
        pixmap,
        "x",
        row.delete.x + row.delete.w / 2.0 - m.name_size / 4.0,
        row.delete.y + row.delete.h / 2.0 - m.name_size * 0.7,
        TextStyle {
            size: m.name_size,
            color: cross,
            face: Face::Bold,
        },
    );

    match row.confirm {
        Some((yes, no)) => draw_confirm(p, pixmap, row, (yes, no)),
        None => draw_steps(p, pixmap, row, tint, enabled),
    }
}

/// La marque d'un domaine : sa pastille de couleur et son sigle.
///
/// Les deux tournent au clic — l'une dans la palette du thème, l'autre dans la table des
/// sigles — et se dessinent à partir de la même teinte.
fn draw_mark(
    p: &Painter,
    pixmap: &mut PixmapMut,
    row: &DomainRowLayout,
    domain: &Domain,
    (r, g, b): (u8, u8, u8),
) {
    let m = &p.m;
    let tint = Color::from_rgba8(r, g, b, 255);

    let cx = row.swatch.x + row.swatch.w / 2.0;
    let cy = row.swatch.y + row.swatch.h / 2.0;
    let hovered = row.swatch.contains(p.brush.pointer.x, p.brush.pointer.y);
    let rayon = row.swatch.w / 2.0 - if hovered { 0.0 } else { m.pad / 6.0 };
    p.brush.circle(pixmap, (cx, cy), rayon, tint);

    p.brush.fill(
        pixmap,
        row.sigil,
        m.pad / 4.0,
        Color::from_rgba8(r, g, b, 40),
    );
    p.brush.stroke(
        pixmap,
        row.sigil,
        m.pad / 4.0,
        Color::from_rgba8(r, g, b, 130),
        m.pad / 12.0,
    );
    let (sigil_w, _) = p
        .brush
        .typo
        .measure_text(&domain.icon, m.sigil_size, Face::Bold);
    p.brush.text_styled(
        pixmap,
        &domain.icon,
        row.sigil.x + (row.sigil.w - sigil_w) / 2.0,
        row.sigil.y + row.sigil.h / 2.0 - m.sigil_size * 0.7,
        TextStyle {
            size: m.sigil_size,
            color: tint,
            face: Face::Bold,
        },
    );
}

/// Le nom du domaine, ou sa saisie en cours.
fn draw_name(
    p: &Painter,
    pixmap: &mut PixmapMut,
    row: &DomainRowLayout,
    domain: &Domain,
    ui: &DomainsUi,
) {
    let m = &p.m;
    let editing = ui.rename.as_ref().filter(|r| r.domain_id == domain.id);
    let text = editing.map_or(domain.name.as_str(), |r| r.entry.text());
    let baseline = row.name.y + row.name.h / 2.0 - m.name_size * 0.7;

    if editing.is_some() {
        p.brush
            .fill(pixmap, row.name, m.pad / 4.0, p.brush.theme.input_bg);
        p.brush.stroke(
            pixmap,
            row.name,
            m.pad / 4.0,
            p.brush.theme.border_accent,
            m.pad / 12.0,
        );
    }
    p.brush.text_styled(
        pixmap,
        text,
        row.name.x + m.pad / 3.0,
        baseline,
        TextStyle {
            size: m.name_size,
            color: p.brush.theme.text_primary,
            face: Face::Regular,
        },
    );
    let Some(rename) = editing else {
        return;
    };
    // Caret plein, sans clignotement : une animation demanderait un réveil périodique de la
    // boucle d'événements, et un caret fixe dit exactement la même chose.
    let (prefix, _) =
        p.brush
            .typo
            .measure_text(rename.entry.before_cursor(), m.name_size, Face::Regular);
    let caret = WidgetRect::new(
        row.name.x + m.pad / 3.0 + prefix,
        baseline,
        (m.pad / 8.0).max(1.0),
        m.name_size * 1.2,
    );
    p.brush.fill(pixmap, caret, 0.0, p.brush.theme.text_accent);
}

/// Les cinq paliers de pondération.
fn draw_steps(
    p: &Painter,
    pixmap: &mut PixmapMut,
    row: &DomainRowLayout,
    tint: Color,
    enabled: bool,
) {
    let m = &p.m;
    let (r, g, b) = (
        (tint.red() * 255.0) as u8,
        (tint.green() * 255.0) as u8,
        (tint.blue() * 255.0) as u8,
    );
    for (index, rect) in row.steps.iter().enumerate() {
        let hovered = enabled && rect.contains(p.brush.pointer.x, p.brush.pointer.y);
        let alpha = if enabled { 60 } else { 24 };
        p.brush.fill(
            pixmap,
            *rect,
            m.pad / 4.0,
            Color::from_rgba8(r, g, b, if hovered { 150 } else { alpha }),
        );
        let label = format!("{}", (WEIGHT_STEPS[index] * 100.0).round() as i32);
        let (width, _) = p
            .brush
            .typo
            .measure_text(&label, m.step_size, Face::Regular);
        let color = if enabled {
            p.brush.theme.text_secondary
        } else {
            p.brush.theme.text_muted
        };
        p.brush.text_styled(
            pixmap,
            &label,
            rect.x + (rect.w - width) / 2.0,
            rect.y + rect.h / 2.0 - m.step_size * 0.7,
            TextStyle {
                size: m.step_size,
                color,
                face: Face::Regular,
            },
        );
    }

    draw_unassign(p, pixmap, row, enabled);
}

/// Le bouton qui détache la sélection de ce domaine — le « − » au bout de la rangée.
fn draw_unassign(p: &Painter, pixmap: &mut PixmapMut, row: &DomainRowLayout, enabled: bool) {
    let m = &p.m;
    let hovered = enabled && row.unassign.contains(p.brush.pointer.x, p.brush.pointer.y);
    let fond = if hovered {
        p.brush.theme.bg_hover
    } else {
        p.brush.theme.btn_bg
    };
    p.brush.fill(pixmap, row.unassign, m.pad / 4.0, fond);
    p.brush.stroke(
        pixmap,
        row.unassign,
        m.pad / 4.0,
        p.brush.theme.btn_border,
        m.pad / 12.0,
    );
    let (width, _) = p.brush.typo.measure_text("-", m.step_size, Face::Bold);
    let color = if enabled {
        p.brush.theme.text_secondary
    } else {
        p.brush.theme.text_muted
    };
    p.brush.text_styled(
        pixmap,
        "-",
        row.unassign.x + (row.unassign.w - width) / 2.0,
        row.unassign.y + row.unassign.h / 2.0 - m.step_size * 0.7,
        TextStyle {
            size: m.step_size,
            color,
            face: Face::Bold,
        },
    );
}

/// La question de confirmation et ses deux réponses.
fn draw_confirm(
    p: &Painter,
    pixmap: &mut PixmapMut,
    row: &DomainRowLayout,
    buttons: (WidgetRect, WidgetRect),
) {
    let m = &p.m;
    let (yes, no) = buttons;
    p.brush.text_styled(
        pixmap,
        "Supprimer ?",
        row.steps.first().map_or(row.unassign.x, |r| r.x),
        yes.y + yes.h / 2.0 - m.step_size * 0.7,
        TextStyle {
            size: m.step_size,
            color: p.brush.theme.text_secondary,
            face: Face::Regular,
        },
    );
    for (rect, label, danger) in [(yes, "Oui", true), (no, "Non", false)] {
        let hovered = rect.contains(p.brush.pointer.x, p.brush.pointer.y);
        let background = match (danger, hovered) {
            (true, true) => p.brush.theme.danger,
            (true, false) => p.brush.theme.bg_hover,
            (false, true) => p.brush.theme.bg_hover,
            (false, false) => p.brush.theme.btn_bg,
        };
        p.brush.fill(pixmap, rect, m.pad / 4.0, background);
        p.brush.stroke(
            pixmap,
            rect,
            m.pad / 4.0,
            p.brush.theme.btn_border,
            m.pad / 12.0,
        );
        let (width, _) = p.brush.typo.measure_text(label, m.step_size, Face::Bold);
        p.brush.text_styled(
            pixmap,
            label,
            rect.x + (rect.w - width) / 2.0,
            rect.y + rect.h / 2.0 - m.step_size * 0.7,
            TextStyle {
                size: m.step_size,
                color: p.brush.theme.text_primary,
                face: Face::Bold,
            },
        );
    }
}

fn draw_footer(
    p: &Painter,
    pixmap: &mut PixmapMut,
    frame: ScaledRect,
    layout: &DomainsPanelLayout,
    selected: usize,
) {
    let m = &p.m;
    let hint = match (layout.hidden, selected) {
        (0, 0) => "Selectionne un noeud pour lui donner une ponderation.".to_string(),
        (0, _) => "Palier 20-100 % : assigne. Bouton - : retire. Nom : renomme.".to_string(),
        (n, _) => format!("{n} domaine(s) de plus — agrandis la fenetre pour les voir."),
    };
    p.brush.text_styled(
        pixmap,
        &hint,
        frame.x + m.pad,
        layout.add_button.y - m.hint_height,
        TextStyle {
            size: m.hint_size,
            color: p.brush.theme.text_muted,
            face: Face::Regular,
        },
    );

    let hovered = layout
        .add_button
        .contains(p.brush.pointer.x, p.brush.pointer.y);
    p.brush.fill(
        pixmap,
        layout.add_button,
        m.pad / 3.0,
        if hovered {
            p.brush.theme.bg_hover
        } else {
            p.brush.theme.btn_bg
        },
    );
    p.brush.stroke(
        pixmap,
        layout.add_button,
        m.pad / 3.0,
        p.brush.theme.btn_border,
        m.pad / 12.0,
    );
    let label = "+ Nouveau domaine";
    let (width, _) = p.brush.typo.measure_text(label, m.name_size, Face::Regular);
    p.brush.text_styled(
        pixmap,
        label,
        layout.add_button.x + (layout.add_button.w - width) / 2.0,
        layout.add_button.y + layout.add_button.h / 2.0 - m.name_size * 0.7,
        TextStyle {
            size: m.name_size,
            color: p.brush.theme.text_secondary,
            face: Face::Regular,
        },
    );
}
