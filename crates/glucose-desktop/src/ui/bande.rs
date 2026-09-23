//! La bande du haut — la barre et les onglets — rendue **une fois par changement**, et posée
//! d'une copie le reste du temps.
//!
//! # Ce que ça coûtait, mesuré
//!
//! `bench_salissure`, sur 429 photos en glissement : la barre coûtait 13 à 15 % de l'image, et
//! tout entier dans ses **boutons** — une dizaine d'icônes tracées par `fill_path` anti-aliasé,
//! ~0,1 ms chacune — pendant que la bande était identique à l'image précédente sur 119 images
//! sur 119. Le fond, le logo et le layout ne coûtaient rien.
//!
//! # Pourquoi la clé est exacte, et non devinée
//!
//! La barre et les onglets sont déjà des **layouts** : [`super::layout_topbar`] et
//! [`super::layout_tabs`] produisent des données, et le dessin ne fait que les lire. Ce dont
//! le dessin dépend est donc ce dont le layout dépend — la largeur, l'échelle, l'état de
//! l'interface, le document — plus le seul état de survol, que le pointeur ne donne qu'à
//! travers le bouton ou l'onglet qu'il désigne. Comme `bench_chrome` l'a établi pour les
//! panneaux : la clé n'est pas la position du pointeur, c'est **ce qui est survolé**.
//!
//! Le thème n'y figure pas : l'application n'en a qu'un. Le jour où elle en aura deux, il y
//! entrera, et un test le réclamera avant.
//!
//! # Pourquoi le rendu en cache est identique au rendu direct, au bit près
//!
//! La bande commence à l'origine de l'écran : rendue dans un tampon de sa taille, chaque
//! primitive reçoit exactement les coordonnées qu'elle recevait dans l'écran entier. Aucun
//! décalage, donc aucune phase de glyphe à préserver (GLYPH-1 ne se pose pas). Et la bande
//! est opaque : elle se **remplace**, une copie par ligne.

use super::{
    layout_tabs, layout_topbar, ActiveTool, TabButtonLayout, TopbarButtonDef, TopbarLayout, UiState,
};
use crate::icons::{draw_icon_scaled, IconType};
use crate::params::{ButtonState, Pointer, ScaledRect};
use crate::renderer::scale::fill_crisp;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::report::Melange;
use glucose_core::store::Store;
use std::hash::{Hash, Hasher};
use tiny_skia::{Color, Pixmap, PixmapMut, Rect};

/// Tout ce dont l'aspect de la bande dépend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandeKey {
    largeur: u32,
    echelle: u32,
    outil: ActiveTool,
    smart_align: bool,
    images: usize,
    /// Les tableaux — identifiant, nom, lequel est actif — réduits à une empreinte.
    tableaux: u64,
    survol: Survol,
}

/// **Ce que le pointeur survole dans la bande** : un bouton, un onglet — la seule chose de la
/// bande qu'il change.
///
/// Une seule définition, lue deux fois : par le dessin, pour sa clé, et par la souris, pour
/// savoir s'il y a quelque chose à redessiner. Deux définitions finiraient par ne plus
/// désigner le même bouton, et la souris redessinerait pour rien — ou ne redessinerait pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Survol {
    bouton: Option<usize>,
    onglet: Option<usize>,
}

/// Ce que le pointeur survole dans des layouts déjà calculés.
fn survol(barre: &TopbarLayout, onglets: &[TabButtonLayout], pointer: Pointer) -> Survol {
    Survol {
        bouton: barre
            .buttons
            .iter()
            .position(|b| contient(b.x, b.y, b.w, b.h, pointer)),
        onglet: onglets
            .iter()
            .position(|t| contient(t.x, t.y, t.width, t.height, pointer)),
    }
}

/// **Ce que le pointeur survole dans la bande d'une fenêtre large de `largeur`.**
pub fn survol_de_la_bande(
    store: &Store,
    ui: &UiState,
    typo: &Typography,
    largeur: f32,
    pointer: Pointer,
) -> Survol {
    let barre = layout_topbar(largeur, ui, typo, store.nombre_d_images());
    let onglets = layout_tabs(store, typo, ui.topbar_height(), ui.scale());
    survol(&barre, &onglets, pointer)
}

/// La bande déjà dessinée, et la clé sous laquelle elle l'a été.
pub struct BandeCache {
    pub(super) pixmap: Pixmap,
    cle: BandeKey,
    /// Combien de fois la bande a été **réellement** dessinée depuis le début de la session.
    ///
    /// C'est ce qui rend le cache testable : sans ce compte, un cache qui repeindrait tout à
    /// chaque image rendrait les mêmes pixels et passerait toutes les épreuves d'aspect. On
    /// ne saurait qu'il est inutile qu'au chronomètre — la leçon des vignettes, qui ont servi
    /// à un pour cent pendant cinq sessions.
    pub(super) dessins: u64,
}

/// Dessine la bande du haut : depuis le cache si rien n'a changé, sinon à neuf.
pub(super) fn render_bande(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &mut UiState,
    typo: &Typography,
    theme: &Theme,
    largeur: f32,
    pointer: Pointer,
) {
    let images = store.nombre_d_images();
    let barre = layout_topbar(largeur, ui, typo, images);
    let onglets = layout_tabs(store, typo, ui.topbar_height(), ui.scale());
    let cle = BandeKey {
        largeur: largeur.to_bits(),
        echelle: ui.scale_factor.to_bits(),
        outil: ui.active_tool,
        smart_align: ui.smart_align,
        images,
        tableaux: empreinte_des_tableaux(&onglets),
        survol: survol(&barre, &onglets, pointer),
    };
    let hauteur = ui.header_height().ceil() as u32;
    let perime = ui
        .bande_cache
        .as_ref()
        .is_none_or(|c| c.cle != cle || c.pixmap.width() != largeur as u32);
    if perime {
        let Some(mut tampon) = Pixmap::new(largeur.max(1.0) as u32, hauteur.max(1)) else {
            return;
        };
        {
            let mut vue = tampon.as_mut();
            render_topbar(
                &mut vue,
                ui,
                typo,
                theme,
                largeur,
                &barre,
                cle.survol.bouton,
            );
            render_board_tabs(
                &mut vue,
                ui,
                typo,
                theme,
                largeur,
                &onglets,
                cle.survol.onglet,
            );
        }
        let dessins = ui.bande_cache.as_ref().map_or(0, |c| c.dessins) + 1;
        ui.bande_cache = Some(BandeCache {
            pixmap: tampon,
            cle,
            dessins,
        });
        crate::perf::compteur("bande_dessins", dessins as f64);
    }
    if let Some(cache) = &ui.bande_cache {
        crate::composition::poser(pixmap, &cache.pixmap, (0.0, 0.0), Melange::Remplacer);
    }
}

/// Le pointeur est-il dans cette boîte ?
fn contient(x: f32, y: f32, w: f32, h: f32, p: Pointer) -> bool {
    p.x >= x && p.x < x + w && p.y >= y && p.y < y + h
}

/// Ce qui, dans les onglets, change leur dessin : les noms, l'ordre, et lequel est actif.
fn empreinte_des_tableaux(onglets: &[TabButtonLayout]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for t in onglets {
        t.board_id.hash(&mut h);
        t.name.hash(&mut h);
        t.is_active.hash(&mut h);
    }
    h.finish()
}

/// Rectangle d'un bouton de barre d'outils, échelle UI comprise.
fn box_of(btn: &TopbarButtonDef, scale: f32) -> ScaledRect {
    ScaledRect {
        x: btn.x,
        y: btn.y,
        w: btn.w,
        h: btn.h,
        scale,
    }
}

/// Un trait vertical d'un pixel, **posé sur la grille de pixels**.
///
/// Les boutons sont désormais mesurés, donc leurs abscisses sont fractionnaires. Un trait
/// d'un pixel à une abscisse fractionnaire s'étale en gris sur deux colonnes ; et dans un
/// build de debug, `tiny-skia` refuse par assertion le rectangle intérieur de largeur nulle
/// que produit son anti-aliasing sur ce cas. Arrondir est la seule bonne réponse aux deux.
fn draw_separator(pixmap: &mut PixmapMut, x: f32, y: f32, h: f32, color: Color) {
    if let Some(rect) = Rect::from_xywh(x.round(), y.round(), 1.0, h) {
        fill_crisp(pixmap, rect, color);
    }
}

fn render_topbar(
    pixmap: &mut PixmapMut,
    ui: &UiState,
    typo: &Typography,
    theme: &Theme,
    width: f32,
    layout: &TopbarLayout,
    survole: Option<usize>,
) {
    let s = ui.scale();
    let topbar_h = ui.topbar_height();

    // Fond et bordure inférieure : deux rectangles alignés, donc nets (SCALE-3).
    if let Some(rect) = Rect::from_xywh(0.0, 0.0, width, topbar_h) {
        fill_crisp(pixmap, rect, theme.bg_header);
    }
    if let Some(rect) = Rect::from_xywh(0.0, topbar_h - 1.0, width, 1.0) {
        fill_crisp(pixmap, rect, theme.border_subtle);
    }

    typo.draw_text(
        pixmap,
        "GLUCOSE",
        12.0 * s,
        (topbar_h - 14.0 * s) / 2.0,
        TextStyle {
            size: 14.0 * s,
            color: theme.text_primary,
            face: Face::Bold,
        },
    );

    for sep_x in &layout.separators {
        draw_separator(
            pixmap,
            *sep_x,
            (topbar_h - 20.0 * s) / 2.0,
            20.0 * s,
            theme.border_subtle,
        );
    }

    for (i, btn) in layout.buttons.iter().enumerate() {
        let state = ButtonState {
            active: btn.active,
            hover: survole == Some(i),
        };
        if btn.is_tool {
            super::boutons::draw_tool_button(pixmap, theme, box_of(btn, s), btn.icon, state);
        } else {
            super::boutons::draw_action_button(
                pixmap,
                typo,
                theme,
                box_of(btn, s),
                btn.icon,
                btn.label,
                state,
            );
        }
    }

    // Badge nombre d'images à droite
    if let Some((badge_x, ref badge_txt)) = layout.img_badge {
        typo.draw_text(
            pixmap,
            badge_txt,
            badge_x,
            (topbar_h - 11.0 * s) / 2.0,
            TextStyle {
                size: 11.0 * s,
                color: theme.badge_text,
                face: Face::Regular,
            },
        );
    }
}

fn render_board_tabs(
    pixmap: &mut PixmapMut,
    ui: &UiState,
    typo: &Typography,
    theme: &Theme,
    width: f32,
    onglets: &[TabButtonLayout],
    survole: Option<usize>,
) {
    let s = ui.scale();
    let y_start = ui.topbar_height();
    let tabs_h = ui.tabs_height();

    if let Some(rect) = Rect::from_xywh(0.0, y_start, width, tabs_h) {
        fill_crisp(pixmap, rect, theme.bg_canvas);
    }
    if let Some(rect) = Rect::from_xywh(0.0, y_start + tabs_h - 1.0, width, 1.0) {
        fill_crisp(pixmap, rect, theme.border_subtle);
    }

    for (i, tab) in onglets.iter().enumerate() {
        let is_hover = survole == Some(i);
        if tab.is_plus {
            if is_hover {
                if let Some(rect) = Rect::from_xywh(tab.x, y_start + 5.0 * s, 24.0 * s, 24.0 * s) {
                    fill_crisp(pixmap, rect, theme.bg_hover);
                }
            }
            draw_icon_scaled(
                pixmap,
                IconType::Plus,
                tab.x + 5.0 * s,
                y_start + 10.0 * s,
                14.0 * s,
                theme.text_muted,
                1.5 * s,
            );
        } else {
            dessiner_un_onglet(pixmap, typo, theme, tab, is_hover, (s, y_start, tabs_h));
        }
    }
}

/// Un onglet nommé : son survol, son nom, et le trait de l'onglet actif.
fn dessiner_un_onglet(
    pixmap: &mut PixmapMut,
    typo: &Typography,
    theme: &Theme,
    tab: &TabButtonLayout,
    is_hover: bool,
    (s, y_start, tabs_h): (f32, f32, f32),
) {
    if is_hover && !tab.is_active {
        if let Some(rect) = Rect::from_xywh(tab.x, y_start + 4.0 * s, tab.width, tabs_h - 6.0 * s) {
            fill_crisp(pixmap, rect, theme.bg_hover);
        }
    }
    let text_color = if tab.is_active {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    typo.draw_text(
        pixmap,
        &tab.name,
        tab.x + 14.0 * s,
        y_start + 10.0 * s,
        TextStyle {
            size: 12.0 * s,
            color: text_color,
            face: if tab.is_active {
                Face::Bold
            } else {
                Face::Regular
            },
        },
    );
    // Fiche 06 § 10.2 : l'onglet actif porte une bordure inférieure de 2 px, blanc
    // pur — pas l'accent, qui n'est jamais décoratif.
    if tab.is_active {
        if let Some(rect) = Rect::from_xywh(tab.x, y_start + tabs_h - 2.0 * s, tab.width, 2.0 * s) {
            fill_crisp(pixmap, rect, theme.text_accent);
        }
    }
}

#[cfg(test)]
mod tests;
