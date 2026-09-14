//! Le menu contextuel du clic droit.
//!
//! # Le conflit qu'il fallait résoudre d'abord
//!
//! Dans Glucose, le clic droit **déplace la vue** — c'est écrit dans la spécification
//! d'origine, et c'est un geste qu'on fait des centaines de fois par séance. La même
//! spécification annonce pourtant un menu contextuel au clic droit. Les deux ne peuvent pas
//! cohabiter naïvement : ouvrir un menu à chaque fin de pan serait insupportable.
//!
//! La résolution est celle de PICK-1 : **le geste se décide au relâchement**. Un clic droit
//! qui a déplacé le curseur était un pan, et rien ne s'ouvre. Un clic droit relâché sur
//! place — moins de quelques pixels — était une demande de menu. Aucun mode, aucun
//! modificateur, aucun réglage : l'intention se lit dans le geste lui-même.
//!
//! # Ce qu'il propose, et ce qu'il tait
//!
//! Seulement des gestes qui existent. La spécification d'origine décrit aussi « Couper »,
//! « Copier », « Grouper dans une nouvelle membrane » et « Créer un sous-dossier » ; aucun
//! n'est écrit en Rust, donc aucun n'est proposé. Un menu qui offre ce qui n'existe pas est
//! la même faute qu'un bouton qui affiche un toast sans rien faire (R-33).
//!
//! Chaque entrée porte son raccourci. C'est la seule façon qu'un menu a d'apprendre à s'en
//! passer.
//!
//! # Les dimensions sont assumées, pas relevées
//!
//! La référence n'a jamais eu ce composant — `ContextMenu.tsx` est décrit dans la
//! spécification d'origine et n'existe dans aucun commit du dépôt. Il n'y a donc aucune
//! capture à égaler : les valeurs ci-dessous sont choisies pour s'accorder à la barre
//! d'action, qui est la surface flottante voisine, et n'ont pas à être défendues comme une
//! parité.

use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::store::Store;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Transform};

/// Marges intérieures du menu.
const PAD: f32 = 4.0;
/// Hauteur d'une entrée.
const ROW: f32 = 24.0;
/// Marge horizontale d'une entrée.
const ROW_PAD_X: f32 = 10.0;
/// Espace minimal entre le libellé et son raccourci.
const SHORTCUT_GAP: f32 = 24.0;
/// Corps du texte.
const FONT: f32 = 12.0;
/// Rayon des coins, celui de la barre d'action.
const RADIUS: f32 = 6.0;
/// Hauteur de la bande d'un séparateur, filet compris.
const SEPARATOR: f32 = 7.0;

/// Ce qu'une entrée du menu déclenche.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    Duplicate,
    ToggleLock,
    ToFront,
    ToBack,
    Delete,
    Paste,
    SelectAll,
}

/// Une ligne du menu : soit une entrée, soit un filet.
#[derive(Clone, Debug, PartialEq)]
pub enum MenuRow {
    Item {
        action: MenuAction,
        label: &'static str,
        shortcut: &'static str,
        rect: (f32, f32, f32, f32),
    },
    /// Un filet de séparation, à cette ordonnée.
    Separator(f32),
}

/// Le menu, une fois placé.
#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenu {
    pub rect: (f32, f32, f32, f32),
    pub rows: Vec<MenuRow>,
}

/// Les entrées que l'état courant justifie, `None` entre deux groupes.
///
/// Rien n'est grisé : une entrée sans objet **disparaît**. Griser, c'est montrer une porte
/// fermée ; retirer, c'est ne pas parler de porte du tout.
type Def = Option<(MenuAction, &'static str, &'static str)>;

fn entrees(store: &Store) -> Option<Vec<Def>> {
    let images = store.selected_image_ids.len();
    let sur_selection = images + store.selected_annotation_ids.len() > 0;
    let mut defs: Vec<Option<(MenuAction, &'static str, &'static str)>> = Vec::new();
    if sur_selection {
        defs.push(Some((MenuAction::Duplicate, "Dupliquer", "Ctrl+D")));
        if images > 0 {
            let board = store.active_board()?;
            let toutes_fermees = board
                .images
                .iter()
                .filter(|i| store.selected_image_ids.contains(&i.id))
                .all(|i| i.locked);
            let label = if toutes_fermees {
                "Déverrouiller"
            } else {
                "Verrouiller"
            };
            defs.push(Some((MenuAction::ToggleLock, label, "L")));
        }
        defs.push(None);
        defs.push(Some((MenuAction::ToFront, "Au premier plan", "Ctrl+]")));
        defs.push(Some((MenuAction::ToBack, "À l'arrière-plan", "Ctrl+[")));
        defs.push(None);
        defs.push(Some((MenuAction::Delete, "Supprimer", "Suppr")));
    } else {
        defs.push(Some((MenuAction::Paste, "Coller", "Ctrl+V")));
        defs.push(Some((MenuAction::SelectAll, "Tout sélectionner", "Ctrl+A")));
    }
    Some(defs)
}

/// Ce qu'un menu proposerait à cet endroit, ouvert en `(ax, ay)`.
///
/// Fonction pure : elle ne lit que le store et la typographie, et ne dessine rien. Rend `None`
/// quand il n'y aurait rien à proposer.
pub fn layout_context_menu(
    store: &Store,
    typography: &Typography,
    at: (f32, f32),
    screen: (f32, f32),
    scale: f32,
) -> Option<ContextMenu> {
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    let defs = entrees(store)?;

    let largeur_ligne = |label: &str, shortcut: &str| {
        let (a, _) = typography.measure_text(label, font, Face::Regular);
        let (b, _) = typography.measure_text(shortcut, font, Face::Regular);
        a + SHORTCUT_GAP * s + b + ROW_PAD_X * s * 2.0
    };
    let largeur = defs
        .iter()
        .flatten()
        .map(|(_, l, k)| largeur_ligne(l, k))
        .fold(0.0f32, f32::max);
    let hauteur = PAD * s * 2.0
        + defs
            .iter()
            .map(|d| if d.is_some() { ROW * s } else { SEPARATOR * s })
            .sum::<f32>();

    // Le menu reste dans la fenêtre : s'il déborde, il bascule de l'autre côté du curseur,
    // comme n'importe quel menu système. Sans cela, un clic droit près du bord ouvrirait un
    // menu dont la moitié est hors de l'écran.
    let x = if at.0 + largeur <= screen.0 {
        at.0
    } else {
        (at.0 - largeur).max(0.0)
    };
    let y = if at.1 + hauteur <= screen.1 {
        at.1
    } else {
        (at.1 - hauteur).max(0.0)
    };

    let mut curseur = y + PAD * s;
    let mut rows = Vec::with_capacity(defs.len());
    for def in defs {
        match def {
            Some((action, label, shortcut)) => {
                rows.push(MenuRow::Item {
                    action,
                    label,
                    shortcut,
                    rect: (x + PAD * s, curseur, largeur - PAD * s * 2.0, ROW * s),
                });
                curseur += ROW * s;
            }
            None => {
                rows.push(MenuRow::Separator(curseur + SEPARATOR * s / 2.0));
                curseur += SEPARATOR * s;
            }
        }
    }

    Some(ContextMenu {
        rect: (x, y, largeur, hauteur),
        rows,
    })
}

/// L'entrée visée en `(px, py)`, s'il y en a une.
pub fn hit_context_menu(menu: &ContextMenu, px: f32, py: f32) -> Option<MenuAction> {
    menu.rows.iter().find_map(|row| match row {
        MenuRow::Item { action, rect, .. } if dans(*rect, px, py) => Some(*action),
        _ => None,
    })
}

/// Le point tombe-t-il sur le menu ? Ce qui tombe dessus ne descend jamais au canevas.
pub fn covers(menu: &ContextMenu, px: f32, py: f32) -> bool {
    dans(menu.rect, px, py)
}

fn dans((x, y, w, h): (f32, f32, f32, f32), px: f32, py: f32) -> bool {
    px >= x && px <= x + w && py >= y && py <= y + h
}

/// Dessine le menu, avec l'entrée survolée en surbrillance.
pub fn draw_context_menu(
    pixmap: &mut PixmapMut,
    menu: &ContextMenu,
    typography: &Typography,
    theme: &Theme,
    pointer: (f32, f32),
    scale: f32,
) {
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;

    let mut pb = PathBuilder::new();
    let (x, y, w, h) = menu.rect;
    crate::renderer::push_rounded_rect(&mut pb, x, y, w, h, RADIUS * s);
    if let Some(path) = pb.finish() {
        let mut paint = Paint {
            anti_alias: true,
            ..Default::default()
        };
        paint.set_color(theme.btn_bg);
        pixmap.fill_path(
            &path,
            &paint,
            tiny_skia::FillRule::Winding,
            Transform::identity(),
            None,
        );
        paint.set_color(theme.btn_border);
        pixmap.stroke_path(
            &path,
            &paint,
            &tiny_skia::Stroke {
                width: 1.0,
                ..Default::default()
            },
            Transform::identity(),
            None,
        );
    }

    for row in &menu.rows {
        match row {
            MenuRow::Separator(sy) => filet(pixmap, x + PAD * s, *sy, w - PAD * s * 2.0, theme),
            MenuRow::Item { .. } => draw_row(pixmap, row, typography, theme, pointer, font),
        }
    }
}

/// Une entrée : sa surbrillance si le curseur est dessus, son libellé, son raccourci.
///
/// Le rouge de la suppression n'apparaît qu'au survol, comme dans la barre d'action : au
/// repos, une liste d'entrées se lit d'un seul ton.
fn draw_row(
    pixmap: &mut PixmapMut,
    row: &MenuRow,
    typography: &Typography,
    theme: &Theme,
    pointer: (f32, f32),
    font: f32,
) {
    let MenuRow::Item {
        action,
        label,
        shortcut,
        rect,
    } = row
    else {
        return;
    };
    let survol = dans(*rect, pointer.0, pointer.1);
    if survol {
        remplir_arrondi(pixmap, *rect, RADIUS / 2.0, theme.bg_hover);
    }
    let ink = match (survol, action) {
        (true, MenuAction::Delete) => theme.alert,
        (true, _) => theme.text_primary,
        (false, _) => theme.text_secondary,
    };
    let base = rect.1 + (rect.3 - font) / 2.0;
    let pad = ROW_PAD_X * font / FONT;
    typography.draw_text(
        pixmap,
        label,
        rect.0 + pad,
        base,
        TextStyle {
            size: font,
            color: ink,
            face: Face::Regular,
        },
    );
    let (kw, _) = typography.measure_text(shortcut, font, Face::Regular);
    typography.draw_text(
        pixmap,
        shortcut,
        rect.0 + rect.2 - pad - kw,
        base,
        TextStyle {
            size: font,
            color: theme.text_muted,
            face: Face::Regular,
        },
    );
}

/// Un filet horizontal d'un pixel, posé sur la grille de pixels et sans anti-aliasage.
fn filet(pixmap: &mut PixmapMut, x: f32, y: f32, largeur: f32, theme: &Theme) {
    let Some(rect) = Rect::from_xywh(x.round(), y.round(), largeur.round().max(1.0), 1.0) else {
        return;
    };
    let mut paint = Paint {
        anti_alias: false,
        ..Default::default()
    };
    paint.set_color(theme.border_medium);
    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
}

fn remplir_arrondi(pixmap: &mut PixmapMut, (x, y, w, h): (f32, f32, f32, f32), r: f32, c: Color) {
    let mut pb = PathBuilder::new();
    crate::renderer::push_rounded_rect(&mut pb, x, y, w, h, r);
    let Some(path) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(c);
    pixmap.fill_path(
        &path,
        &paint,
        tiny_skia::FillRule::Winding,
        Transform::identity(),
        None,
    );
}

#[cfg(test)]
mod tests;
