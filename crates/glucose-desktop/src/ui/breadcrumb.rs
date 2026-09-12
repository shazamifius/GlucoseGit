//! Le fil d'Ariane des dossiers (fiche 08 § 5.2).
//!
//! # Ce qu'il répare
//!
//! Le store tenait `folder_stack` depuis longtemps, testé, et **rien ne l'affichait**. Entrer
//! dans un dossier changeait le canevas sans dire où l'on était ni comment revenir : la seule
//! sortie était un raccourci que personne ne pouvait deviner.
//!
//! # Où il se place, et pourquoi il n'apparaît qu'au besoin
//!
//! Sous la barre d'onglets, aligné à gauche. À la racine du projet, il **ne s'affiche pas** :
//! un fil d'Ariane à un seul segment n'apprend rien et volerait une bande de canevas à tous
//! les utilisateurs qui n'entrent jamais dans un dossier. Il apparaît dès le premier niveau.
//!
//! # La mise en page est une fonction pure
//!
//! [`layout_breadcrumb`] rend les rectangles des segments sans rien dessiner : le rendu et le
//! clic lisent la même liste, donc ne peuvent pas viser des endroits différents. C'est la même
//! discipline que `layout_minimap` et `compute_panel_layouts`.

use crate::theme::Theme;
use crate::typography::{TextStyle, Typography};
use glucose_core::store::Store;
use tiny_skia::{Color, PixmapMut, Rect, Transform};

/// Hauteur de la bande du fil d'Ariane, en pixels logiques.
pub const HEIGHT: f32 = 26.0;
/// Marge à gauche et retrait sous la barre d'onglets.
pub const MARGIN: f32 = 12.0;
/// Taille de police des segments.
pub const FONT: f32 = 12.0;
/// Le séparateur entre deux segments.
pub const SEPARATOR: &str = "  ›  ";
/// Longueur au-delà de laquelle un segment est tronqué.
pub const SEGMENT_MAX_CHARS: usize = 24;

/// Un segment cliquable du fil d'Ariane.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// Le texte affiché, déjà tronqué.
    pub label: String,
    /// La profondeur à laquelle un clic remonte : 0 pour la racine.
    pub depth: usize,
    /// Le rectangle cliquable, en pixels écran.
    pub rect: (f32, f32, f32, f32),
    /// Vrai pour le dernier segment — celui où l'on est déjà, donc sans effet.
    pub current: bool,
}

/// Les segments du fil d'Ariane, ou une liste vide à la racine du projet.
///
/// Fonction pure : elle ne dessine rien et ne lit que le store et la typographie. C'est ce qui
/// permet au clic et au rendu de partager exactement la même géométrie.
pub fn layout_breadcrumb(
    store: &Store,
    typography: &Typography,
    top: f32,
    scale: f32,
) -> Vec<Segment> {
    let chemin = store.folder_path();
    if chemin.len() < 2 {
        return Vec::new();
    }
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    let dernier = chemin.len() - 1;
    let mut x = MARGIN * s;
    let mut segments = Vec::with_capacity(chemin.len());

    for (depth, nom) in chemin.iter().enumerate() {
        let label = crate::renderer::folder::truncate(nom, SEGMENT_MAX_CHARS);
        let (w, _) = typography.measure_text(&label, font, depth == dernier);
        segments.push(Segment {
            label,
            depth,
            rect: (x, top, w, HEIGHT * s),
            current: depth == dernier,
        });
        x += w;
        if depth != dernier {
            x += typography.measure_text(SEPARATOR, font, false).0;
        }
    }
    segments
}

/// Dessine le fil d'Ariane et rend la hauteur qu'il occupe — zéro à la racine.
pub fn draw_breadcrumb(
    pixmap: &mut PixmapMut,
    store: &Store,
    typography: &Typography,
    theme: &Theme,
    top: f32,
    scale: f32,
) -> f32 {
    let segments = layout_breadcrumb(store, typography, top, scale);
    let Some(premier) = segments.first() else {
        return 0.0;
    };
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    let hauteur = HEIGHT * s;

    // Une bande discrète : le fil appartient à la chrome, il ne doit pas concurrencer le
    // canevas qu'il surplombe.
    if let Some(r) = Rect::from_xywh(0.0, top, pixmap.width() as f32, hauteur) {
        let mut bg = tiny_skia::Paint::default();
        bg.set_color(theme.bg_header);
        pixmap.fill_rect(r, &bg, Transform::identity(), None);
    }

    let base = top + hauteur * 0.7;
    for seg in &segments {
        // Le segment courant est celui où l'on est : plus clair et gras, les autres en retrait
        // — ce sont eux qu'on clique, et la hiérarchie doit se lire sans réfléchir.
        let couleur = if seg.current { theme.text_primary } else { theme.text_muted };
        typography.draw_text(
            pixmap,
            &seg.label,
            seg.rect.0,
            base,
            TextStyle { size: font, color: couleur, bold: seg.current },
        );
        if !seg.current {
            typography.draw_text(
                pixmap,
                SEPARATOR,
                seg.rect.0 + seg.rect.2,
                base,
                TextStyle { size: font, color: separator_color(theme), bold: false },
            );
        }
    }
    let _ = premier;
    hauteur
}

/// La couleur des chevrons : plus effacée encore que les segments cliquables, parce qu'ils ne
/// portent aucune information — ils ponctuent.
fn separator_color(theme: &Theme) -> Color {
    let c = theme.text_muted;
    Color::from_rgba8(
        (c.red() * 255.0) as u8,
        (c.green() * 255.0) as u8,
        (c.blue() * 255.0) as u8,
        128,
    )
}

/// La profondeur à laquelle un clic renvoie, s'il tombe sur un segment remontant.
///
/// Rend `None` pour un clic hors du fil, et pour le segment courant : y cliquer n'est pas une
/// erreur, c'est un geste sans effet.
pub fn hit_breadcrumb(
    store: &Store,
    typography: &Typography,
    top: f32,
    scale: f32,
    at: (f32, f32),
) -> Option<usize> {
    layout_breadcrumb(store, typography, top, scale)
        .into_iter()
        .find(|seg| {
            !seg.current
                && at.0 >= seg.rect.0
                && at.0 <= seg.rect.0 + seg.rect.2
                && at.1 >= seg.rect.1
                && at.1 <= seg.rect.1 + seg.rect.3
        })
        .map(|seg| seg.depth)
}

#[cfg(test)]
mod tests;
