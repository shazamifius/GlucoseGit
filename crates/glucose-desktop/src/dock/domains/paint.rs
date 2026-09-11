//! Le dessin du panneau DOMAINES.
//!
//! Séparé de [`super`] parce que la mise en page et le test de clic sont une chose — la
//! liste unique de widgets de DOM-UI-2 — et que poser des pixels en est une autre. Ce module
//! ne décide de rien : il lit la liste que [`super::layout_domains_panel`] a produite et le
//! document, et il peint. Aucune coordonnée n'est recalculée ici (loi L4).

use super::{layout_domains_panel, DomainRowLayout, DomainsPanelLayout, DomainsUi, Metrics, WEIGHT_STEPS};
use crate::params::{Pointer, ScaledRect};
use crate::theme::Theme;
use crate::typography::{TextStyle, Typography};
use glucose_core::store::Store;
use glucose_core::types::Domain;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Stroke, Transform};

use crate::dock::{push_rounded_rect, WidgetRect};

/// Ce que le dessin du panneau garde constant.
struct Painter<'a> {
    typo: &'a Typography,
    theme: &'a Theme,
    m: Metrics,
    pointer: Pointer,
}

/// Dessine le panneau : le catalogue du document, l'état de saisie, les actions.
pub fn render_domains_panel(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &DomainsUi,
    typo: &Typography,
    theme: &Theme,
    frame: ScaledRect,
    pointer: Pointer,
) {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let p = Painter { typo, theme, m: Metrics::scaled(s), pointer };
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

fn draw_header(p: &Painter, pixmap: &mut PixmapMut, frame: ScaledRect, domains: usize, selected: usize) {
    let m = &p.m;
    p.typo.draw_text(
        pixmap,
        "DOMAINES",
        frame.x + m.pad,
        frame.y + m.title_baseline,
        TextStyle { size: m.title_size, color: p.theme.text_primary, bold: true },
    );
    let summary = format!("{domains} domaine(s) · {selected} nœud(s) sélectionné(s)");
    p.typo.draw_text(
        pixmap,
        &summary,
        frame.x + m.pad,
        frame.y + m.header_baseline,
        TextStyle { size: m.hint_size, color: p.theme.text_muted, bold: false },
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
        let (width, _) = p.typo.measure_text(line, size, false);
        p.typo.draw_text(
            pixmap,
            line,
            frame.x + (frame.w - width) / 2.0,
            y,
            TextStyle { size, color: p.theme.text_muted, bold: false },
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
        (p.theme.domain_fallback.red() * 255.0) as u8,
        (p.theme.domain_fallback.green() * 255.0) as u8,
        (p.theme.domain_fallback.blue() * 255.0) as u8,
    );
    let tint = Color::from_rgba8(r, g, b, 255);

    // Pastille de couleur — un clic la fait tourner dans la palette du thème.
    let cx = row.swatch.x + row.swatch.w / 2.0;
    let cy = row.swatch.y + row.swatch.h / 2.0;
    let hovered = row.swatch.contains(p.pointer.x, p.pointer.y);
    fill_circle(pixmap, (cx, cy), row.swatch.w / 2.0 - if hovered { 0.0 } else { m.pad / 6.0 }, tint);

    // Sigle — un clic le fait tourner lui aussi.
    fill_rect(pixmap, row.sigil, m.pad / 4.0, Color::from_rgba8(r, g, b, 40));
    stroke_rect(pixmap, row.sigil, m.pad / 4.0, Color::from_rgba8(r, g, b, 130), m.pad / 12.0);
    let (sigil_w, _) = p.typo.measure_text(&domain.icon, m.sigil_size, true);
    p.typo.draw_text(
        pixmap,
        &domain.icon,
        row.sigil.x + (row.sigil.w - sigil_w) / 2.0,
        row.sigil.y + row.sigil.h / 2.0 - m.sigil_size * 0.7,
        TextStyle { size: m.sigil_size, color: tint, bold: true },
    );

    draw_name(p, pixmap, row, domain, ui);

    // Croix de suppression.
    let delete_hot = row.delete.contains(p.pointer.x, p.pointer.y) || row.confirm.is_some();
    let cross = if delete_hot { p.theme.text_primary } else { p.theme.text_muted };
    p.typo.draw_text(
        pixmap,
        "x",
        row.delete.x + row.delete.w / 2.0 - m.name_size / 4.0,
        row.delete.y + row.delete.h / 2.0 - m.name_size * 0.7,
        TextStyle { size: m.name_size, color: cross, bold: true },
    );

    match row.confirm {
        Some((yes, no)) => draw_confirm(p, pixmap, row, (yes, no)),
        None => draw_steps(p, pixmap, row, tint, enabled),
    }
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
        fill_rect(pixmap, row.name, m.pad / 4.0, p.theme.input_bg);
        stroke_rect(pixmap, row.name, m.pad / 4.0, p.theme.border_accent, m.pad / 12.0);
    }
    p.typo.draw_text(
        pixmap,
        text,
        row.name.x + m.pad / 3.0,
        baseline,
        TextStyle { size: m.name_size, color: p.theme.text_primary, bold: false },
    );
    let Some(rename) = editing else {
        return;
    };
    // Caret plein, sans clignotement : une animation demanderait un réveil périodique de la
    // boucle d'événements, et un caret fixe dit exactement la même chose.
    let (prefix, _) = p.typo.measure_text(rename.entry.before_cursor(), m.name_size, false);
    let caret = WidgetRect::new(
        row.name.x + m.pad / 3.0 + prefix,
        baseline,
        (m.pad / 8.0).max(1.0),
        m.name_size * 1.2,
    );
    fill_rect(pixmap, caret, 0.0, p.theme.accent_primary);
}

/// Les cinq paliers de pondération.
fn draw_steps(p: &Painter, pixmap: &mut PixmapMut, row: &DomainRowLayout, tint: Color, enabled: bool) {
    let m = &p.m;
    let (r, g, b) = (
        (tint.red() * 255.0) as u8,
        (tint.green() * 255.0) as u8,
        (tint.blue() * 255.0) as u8,
    );
    for (index, rect) in row.steps.iter().enumerate() {
        let hovered = enabled && rect.contains(p.pointer.x, p.pointer.y);
        let alpha = if enabled { 60 } else { 24 };
        fill_rect(pixmap, *rect, m.pad / 4.0, Color::from_rgba8(r, g, b, if hovered { 150 } else { alpha }));
        let label = format!("{}", (WEIGHT_STEPS[index] * 100.0).round() as i32);
        let (width, _) = p.typo.measure_text(&label, m.step_size, false);
        let color = if enabled { p.theme.text_secondary } else { p.theme.text_muted };
        p.typo.draw_text(
            pixmap,
            &label,
            rect.x + (rect.w - width) / 2.0,
            rect.y + rect.h / 2.0 - m.step_size * 0.7,
            TextStyle { size: m.step_size, color, bold: false },
        );
    }

    let hovered = enabled && row.unassign.contains(p.pointer.x, p.pointer.y);
    fill_rect(pixmap, row.unassign, m.pad / 4.0, if hovered { p.theme.bg_hover } else { p.theme.btn_bg });
    stroke_rect(pixmap, row.unassign, m.pad / 4.0, p.theme.btn_border, m.pad / 12.0);
    let (width, _) = p.typo.measure_text("-", m.step_size, true);
    let color = if enabled { p.theme.text_secondary } else { p.theme.text_muted };
    p.typo.draw_text(
        pixmap,
        "-",
        row.unassign.x + (row.unassign.w - width) / 2.0,
        row.unassign.y + row.unassign.h / 2.0 - m.step_size * 0.7,
        TextStyle { size: m.step_size, color, bold: true },
    );
}

/// La question de confirmation et ses deux réponses.
fn draw_confirm(p: &Painter, pixmap: &mut PixmapMut, row: &DomainRowLayout, buttons: (WidgetRect, WidgetRect)) {
    let m = &p.m;
    let (yes, no) = buttons;
    p.typo.draw_text(
        pixmap,
        "Supprimer ?",
        row.steps.first().map_or(row.unassign.x, |r| r.x),
        yes.y + yes.h / 2.0 - m.step_size * 0.7,
        TextStyle { size: m.step_size, color: p.theme.text_secondary, bold: false },
    );
    for (rect, label, danger) in [(yes, "Oui", true), (no, "Non", false)] {
        let hovered = rect.contains(p.pointer.x, p.pointer.y);
        let background = match (danger, hovered) {
            (true, true) => p.theme.snap_guide,
            (true, false) => p.theme.bg_hover,
            (false, true) => p.theme.bg_hover,
            (false, false) => p.theme.btn_bg,
        };
        fill_rect(pixmap, rect, m.pad / 4.0, background);
        stroke_rect(pixmap, rect, m.pad / 4.0, p.theme.btn_border, m.pad / 12.0);
        let (width, _) = p.typo.measure_text(label, m.step_size, true);
        p.typo.draw_text(
            pixmap,
            label,
            rect.x + (rect.w - width) / 2.0,
            rect.y + rect.h / 2.0 - m.step_size * 0.7,
            TextStyle { size: m.step_size, color: p.theme.text_primary, bold: true },
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
    p.typo.draw_text(
        pixmap,
        &hint,
        frame.x + m.pad,
        layout.add_button.y - m.hint_height,
        TextStyle { size: m.hint_size, color: p.theme.text_muted, bold: false },
    );

    let hovered = layout.add_button.contains(p.pointer.x, p.pointer.y);
    fill_rect(pixmap, layout.add_button, m.pad / 3.0, if hovered { p.theme.bg_hover } else { p.theme.btn_bg });
    stroke_rect(pixmap, layout.add_button, m.pad / 3.0, p.theme.btn_border, m.pad / 12.0);
    let label = "+ Nouveau domaine";
    let (width, _) = p.typo.measure_text(label, m.name_size, false);
    p.typo.draw_text(
        pixmap,
        label,
        layout.add_button.x + (layout.add_button.w - width) / 2.0,
        layout.add_button.y + layout.add_button.h / 2.0 - m.name_size * 0.7,
        TextStyle { size: m.name_size, color: p.theme.text_secondary, bold: false },
    );
}

// ── Primitives ──────────────────────────────────────────────────────────────

fn fill_rect(pixmap: &mut PixmapMut, rect: WidgetRect, radius: f32, color: Color) {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, rect.x, rect.y, rect.w, rect.h, radius);
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint { anti_alias: true, ..Default::default() };
    paint.set_color(color);
    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
}

fn stroke_rect(pixmap: &mut PixmapMut, rect: WidgetRect, radius: f32, color: Color, width: f32) {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    let mut pb = PathBuilder::new();
    push_rounded_rect(&mut pb, rect.x, rect.y, rect.w, rect.h, radius);
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint { anti_alias: true, ..Default::default() };
    paint.set_color(color);
    pixmap.stroke_path(&path, &paint, &Stroke { width: width.max(0.5), ..Default::default() }, Transform::identity(), None);
}

fn fill_circle(pixmap: &mut PixmapMut, center: (f32, f32), radius: f32, color: Color) {
    if radius <= 0.0 {
        return;
    }
    let mut pb = PathBuilder::new();
    pb.push_circle(center.0, center.1, radius);
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint { anti_alias: true, ..Default::default() };
    paint.set_color(color);
    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
}
