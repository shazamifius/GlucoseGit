//! Le bas du panneau DOMAINES, et la liste « où ? » qui remplace ses rangées.

use super::Painter;
use crate::dock::domains::{DomainsPanelLayout, DomainsUi, Ou};
use crate::dock::WidgetRect;
use crate::params::ScaledRect;
use crate::typography::{Face, TextStyle};
use tiny_skia::PixmapMut;

/// **La liste « où ? »** : chaque place où vit ce domaine, et le retour.
pub(super) fn draw_ou(
    p: &Painter,
    pixmap: &mut PixmapMut,
    frame: ScaledRect,
    layout: &DomainsPanelLayout,
    ou: &Ou,
) {
    let m = &p.m;
    let titre = format!("OÙ EST « {} »", ou.nom);
    let titre = coupe(p, &titre, frame.w - 2.0 * m.pad, m.title_size, Face::Bold);
    let style = |size, color, face| TextStyle { size, color, face };
    let theme = p.brush.theme;
    let (x, y) = (frame.x + m.pad, frame.y + m.title_baseline);
    let gras = style(m.title_size, theme.text_primary, Face::Bold);
    p.brush.text_styled(pixmap, &titre, x, y, gras);
    let compte = format!("{} nœud(s)", ou.lignes.len());
    let y = frame.y + m.header_baseline;
    let discret = style(m.hint_size, theme.text_muted, Face::Regular);
    p.brush.text_styled(pixmap, &compte, x, y, discret);
    if let Some((premier, lignes)) = &layout.ou {
        for (k, rect) in lignes.iter().enumerate() {
            let Some(ligne) = ou.lignes.get(premier + k) else {
                continue;
            };
            if p.brush.hovered(*rect) {
                p.brush.fill(pixmap, *rect, m.pad / 4.0, theme.bg_hover);
            }
            let fichier = if ligne.ici {
                "ici".to_string()
            } else {
                crate::persist::documents::nom_de(std::path::Path::new(&ligne.chemin))
            };
            let texte = format!("{fichier} · {}", ligne.texte);
            let texte = coupe(p, &texte, rect.w - m.pad / 2.0, m.step_size, Face::Regular);
            let y = rect.y + rect.h / 2.0 - m.step_size * 0.7;
            let corps = style(m.step_size, theme.text_secondary, Face::Regular);
            p.brush
                .text_styled(pixmap, &texte, rect.x + m.pad / 4.0, y, corps);
        }
    }
    draw_button(p, pixmap, layout.add_button, "Retour");
}

/// **Le texte, coupé à `largeur`** avec « … » quand il ne tient pas.
pub(super) fn coupe(p: &Painter, texte: &str, largeur: f32, corps: f32, face: Face) -> String {
    let mesure = |t: &str| p.brush.typo.measure_text(t, corps, face).0;
    if mesure(texte) <= largeur {
        return texte.to_string();
    }
    let mut court = texte.to_string();
    while !court.is_empty() && mesure(&format!("{court}…")) > largeur {
        court.pop();
    }
    format!("{}…", court.trim_end())
}

pub(super) fn draw_footer(
    p: &Painter,
    pixmap: &mut PixmapMut,
    frame: ScaledRect,
    (layout, _ui): (&DomainsPanelLayout, &DomainsUi),
    selected: usize,
) {
    let m = &p.m;
    let hint = if layout.rows.iter().any(|r| r.question.is_some()) {
        "Même nom ailleurs : Oui réunit, Non fait préciser le mot.".to_string()
    } else {
        match (layout.hidden, selected) {
            (0, 0) => "Selectionne un noeud pour lui donner une ponderation.".to_string(),
            (0, _) => "Palier 20-100 % : assigne. Bouton - : retire. Nom : renomme.".to_string(),
            (n, _) => format!("{n} de plus : fais défiler à la molette."),
        }
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
    draw_button(p, pixmap, layout.add_button, "+ Nouveau domaine");
}

/// Le bouton du bas du panneau.
fn draw_button(p: &Painter, pixmap: &mut PixmapMut, rect: WidgetRect, label: &str) {
    let m = &p.m;
    let hovered = p.brush.hovered(rect);
    p.brush.fill(
        pixmap,
        rect,
        m.pad / 3.0,
        if hovered {
            p.brush.theme.bg_hover
        } else {
            p.brush.theme.btn_bg
        },
    );
    p.brush.stroke(
        pixmap,
        rect,
        m.pad / 3.0,
        p.brush.theme.btn_border,
        m.pad / 12.0,
    );
    let (width, _) = p.brush.typo.measure_text(label, m.name_size, Face::Regular);
    p.brush.text_styled(
        pixmap,
        label,
        rect.x + (rect.w - width) / 2.0,
        rect.y + rect.h / 2.0 - m.name_size * 0.7,
        TextStyle {
            size: m.name_size,
            color: p.brush.theme.text_secondary,
            face: Face::Regular,
        },
    );
}
