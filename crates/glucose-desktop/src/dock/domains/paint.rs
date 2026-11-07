//! Le dessin du panneau DOMAINES.
//!
//! Séparé de [`super`] parce que la mise en page et le test de clic sont une chose — la
//! liste unique de widgets de DOM-UI-2 — et que poser des pixels en est une autre. Ce module
//! ne décide de rien : il lit la liste que [`super::layout_domains_panel`] a produite et le
//! document, et il peint. Aucune coordonnée n'est recalculée ici (loi L4).

use super::{layout_domains_panel, Cible, DomainRowLayout, DomainsUi, Metrics, WEIGHT_STEPS};
use crate::interactions::text_entry::TextEntry;
use crate::params::ScaledRect;
use crate::typography::{Face, TextStyle};
use glucose_core::store::Store;
use glucose_core::types::Domain;
use tiny_skia::{Color, PixmapMut};

use crate::dock::paint::Brush;
use crate::dock::WidgetRect;

mod pied;

use pied::{coupe, draw_footer, draw_ou};

/// Ce que le dessin du panneau garde constant.
///
/// Il emprunte le [`Brush`] du dock plutôt que de recopier ses champs : c'était un second
/// pinceau, et il écrivait son texte en court-circuitant le premier. Tant que les deux
/// existaient, l'origine du tampon d'un panneau mis en cache n'était appliquée que par l'un
/// des deux, et le texte des domaines atterrissait à côté (DOCK-CACHE-1).
pub(super) struct Painter<'a> {
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
    if let Some(ou) = &ui.ou {
        draw_ou(&p, pixmap, frame, &layout, ou);
        return;
    }
    let selected = store.selected_annotation_ids.len() + store.selected_image_ids.len();

    draw_header(&p, pixmap, frame, store.project.domains.len(), selected);
    if store.project.domains.is_empty() && ui.connus.ailleurs.is_empty() {
        draw_empty_state(&p, pixmap, frame);
    }
    let domaines = store.project.domains.len();
    for row in &layout.rows {
        let domain = match store.project.domains.get(row.index) {
            Some(d) => d,
            None => &ui.connus.ailleurs[row.index - domaines].definition,
        };
        draw_row(&p, pixmap, row, domain, ui, selected > 0);
    }
    draw_footer(&p, pixmap, frame, (&layout, ui), selected);
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
    draw_ou_button(p, pixmap, row);
    if matches!(row.cible, Cible::Ailleurs(_)) {
        draw_steps(p, pixmap, row, tint, enabled);
        return;
    }

    // Croix de suppression.
    let delete_hot = p.brush.hovered(row.delete) || row.confirm.is_some();
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

    match (row.confirm, row.question) {
        (Some(reponses), _) => draw_confirm(p, pixmap, row, reponses, ("Supprimer ?", true)),
        (None, Some(reponses)) => draw_confirm(p, pixmap, row, reponses, ("Même sujet ?", false)),
        (None, None) => {
            draw_steps(p, pixmap, row, tint, enabled);
            draw_unassign(p, pixmap, row, enabled);
        }
    }
}

/// Le bouton « où ? » d'une rangée : les places où vit ce domaine.
fn draw_ou_button(p: &Painter, pixmap: &mut PixmapMut, row: &DomainRowLayout) {
    let m = &p.m;
    let (width, _) = p.brush.typo.measure_text("où", m.step_size, Face::Regular);
    let color = if p.brush.hovered(row.ou) {
        p.brush.theme.text_primary
    } else {
        p.brush.theme.text_muted
    };
    p.brush.text_styled(
        pixmap,
        "où",
        row.ou.x + (row.ou.w - width) / 2.0,
        row.ou.y + row.ou.h / 2.0 - m.step_size * 0.7,
        TextStyle {
            size: m.step_size,
            color,
            face: Face::Regular,
        },
    );
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
    let hovered = p.brush.hovered(row.swatch);
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
    if let Some(rename) = editing {
        draw_entry(p, pixmap, row.name, &rename.entry);
        return;
    }
    let baseline = row.name.y + row.name.h / 2.0 - m.name_size * 0.7;
    let (nom, compte) = (&domain.name, compte_de(row, ui));
    let (largeur_du_compte, _) = p
        .brush
        .typo
        .measure_text(&compte, m.hint_size, Face::Regular);
    let place = row.name.w - m.pad / 3.0;
    let tient = !compte.is_empty()
        && p.brush.typo.measure_text(nom, m.name_size, Face::Regular).0 + largeur_du_compte + m.pad
            <= place;
    let largeur_du_nom = if tient {
        place - largeur_du_compte - m.pad
    } else {
        place
    };
    p.brush.text_styled(
        pixmap,
        &coupe(p, nom, largeur_du_nom, m.name_size, Face::Regular),
        row.name.x + m.pad / 3.0,
        baseline,
        TextStyle {
            size: m.name_size,
            // Un domaine d'un autre fichier n'est pas encore dans celui-ci : il se lit en retrait.
            color: match row.cible {
                Cible::Ici(_) => p.brush.theme.text_primary,
                Cible::Ailleurs(_) => p.brush.theme.text_muted,
            },
            face: Face::Regular,
        },
    );
    if tient {
        p.brush.text_styled(
            pixmap,
            &compte,
            row.name.x + row.name.w - largeur_du_compte,
            row.name.y + row.name.h / 2.0 - m.hint_size * 0.7,
            TextStyle {
                size: m.hint_size,
                color: p.brush.theme.text_muted,
                face: Face::Regular,
            },
        );
    }
}

/// Ce qu'une rangée dit de ce qui vit ailleurs. Un domaine d'ici : combien de nœuds le portent aussi
/// dans d'autres fichiers (« +3 ailleurs »), rien s'il ne vit qu'ici. Un domaine d'ailleurs : ses
/// nœuds et ses fichiers.
fn compte_de(row: &DomainRowLayout, ui: &DomainsUi) -> String {
    let fichiers = |f: usize| if f > 1 { "fichiers" } else { "fichier" };
    match &row.cible {
        Cible::Ici(id) => ui
            .connus
            .aussi
            .iter()
            .find(|(d, _, _)| d == id)
            .map_or_else(String::new, |(_, n, _)| format!("+{n} ailleurs")),
        Cible::Ailleurs(lignee) => ui
            .connus
            .ailleurs
            .iter()
            .find(|a| a.lignee == *lignee)
            .map_or_else(String::new, |a| {
                format!("{} · {} {}", a.noeuds, a.fichiers, fichiers(a.fichiers))
            }),
    }
}

/// Un nom en cours de frappe, dans son cadre, et son curseur.
fn draw_entry(p: &Painter, pixmap: &mut PixmapMut, rect: WidgetRect, entry: &TextEntry) {
    let m = &p.m;
    let baseline = rect.y + rect.h / 2.0 - m.name_size * 0.7;
    p.brush
        .fill(pixmap, rect, m.pad / 4.0, p.brush.theme.input_bg);
    p.brush.stroke(
        pixmap,
        rect,
        m.pad / 4.0,
        p.brush.theme.border_accent,
        m.pad / 12.0,
    );
    p.brush.text_styled(
        pixmap,
        entry.text(),
        rect.x + m.pad / 3.0,
        baseline,
        TextStyle {
            size: m.name_size,
            color: p.brush.theme.text_primary,
            face: Face::Regular,
        },
    );
    // Caret plein, sans clignotement : une animation demanderait un réveil périodique de la
    // boucle d'événements, et un caret fixe dit exactement la même chose.
    let (prefix, _) = p
        .brush
        .typo
        .measure_text(entry.before_cursor(), m.name_size, Face::Regular);
    let caret = WidgetRect::new(
        rect.x + m.pad / 3.0 + prefix,
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
        let hovered = enabled && p.brush.hovered(*rect);
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
}

/// Le bouton qui détache la sélection de ce domaine — le « − » au bout de la rangée.
fn draw_unassign(p: &Painter, pixmap: &mut PixmapMut, row: &DomainRowLayout, enabled: bool) {
    let m = &p.m;
    let hovered = enabled && p.brush.hovered(row.unassign);
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
    (question, grave): (&str, bool),
) {
    let m = &p.m;
    let (yes, no) = buttons;
    p.brush.text_styled(
        pixmap,
        question,
        row.steps.first().map_or(row.unassign.x, |r| r.x),
        yes.y + yes.h / 2.0 - m.step_size * 0.7,
        TextStyle {
            size: m.step_size,
            color: p.brush.theme.text_secondary,
            face: Face::Regular,
        },
    );
    for (rect, label, danger) in [(yes, "Oui", grave), (no, "Non", false)] {
        let hovered = p.brush.hovered(rect);
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
