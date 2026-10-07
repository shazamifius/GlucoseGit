//! **Le rail** (fiche 55) : la barre du haut, sur le côté, quand elle ne tient plus.
//!
//! Sa demande, le 08/10, Glucose ouvert sur son téléphone : *« tous les boutons normalement en
//! haut pour PC doivent être sur le côté, avec une petite flèche pour déplier et replier le
//! panneau »*.
//!
//! # La mesure décide, pas un « mode téléphone »
//!
//! La barre se pose déjà du plus riche au plus sobre — les libellés, puis sans ceux de droite,
//! puis des icônes seules — et garde la première qui tient ([`super::layout_topbar`]). Le rail
//! est la marche suivante : quand **même les icônes seules ne tiennent plus** — un téléphone
//! debout, une fenêtre très étroite —, la barre quitte le haut. La bande ne garde que les
//! onglets, et tout ce qui se mesure sur elle suit ([`UiState::topbar_height`]).
//!
//! # Une seule liste de boutons, deux mises en page (loi L4)
//!
//! Le rail ne recopie pas la barre : il reprend **sa** liste ([`super::boutons::les_boutons`])
//! et la repose en grille. Un bouton ajouté demain à la barre paraît seul dans le rail, avec la
//! même action et le même effet ([`super::boutons::effet_du_bouton`]). Le dessin et le clic
//! lisent la même mise en page ([`layout_rail`]).
//!
//! # Au doigt
//!
//! Des cases de 48 points de côté — le minimum d'Android pour une cible tactile, la skill
//! `glucose-telephone` § 1. Replié, il ne reste qu'une languette au bord gauche, une flèche
//! dedans ; déplié, **autant de rangées que l'écran en tient, et les colonnes qu'il faut** —
//! deux sur un téléphone debout, quatre couché. Le nombre de colonnes n'est pas choisi : la
//! première version en fixait deux, et le panneau sortait de l'écran couché. Choisir un bouton
//! le replie : le canevas revient tout entier.

use super::boutons::{draw_tool_button, effet_du_bouton, les_boutons, TopbarButtonDef};
use super::{UiAction, UiState};
use crate::params::{ButtonState, ScaledRect};
use crate::renderer::scale::fill_crisp;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::report::Melange;
use glucose_core::store::Store;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, PixmapMut, Rect, Transform};

/// Le côté d'une case, en points : la cible tactile minimale d'Android.
const CASE: f32 = 48.0;
/// L'écart entre deux cases, et entre les cases et les bords du panneau.
const ECART: f32 = 4.0;
/// La languette : assez large pour un pouce, aussi haute qu'une case.
const LANGUETTE: (f32, f32) = (28.0, CASE);

/// Un rectangle, `(x, y, largeur, hauteur)`, en pixels de la fenêtre.
pub type Boite = (f32, f32, f32, f32);

/// **Où en est le rail.**
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rail {
    /// La barre ne tient pas en haut : elle est sur le côté.
    pub actif: bool,
    /// Le panneau est déplié.
    pub ouvert: bool,
}

impl Rail {
    /// La part de la barre du haut qui reste : 1 sans rail, 0 avec.
    pub fn barre(&self) -> f32 {
        if self.actif {
            0.0
        } else {
            1.0
        }
    }
}

/// Le rail placé : sa languette, son panneau s'il est déplié, et les boutons du panneau.
#[derive(Debug, Clone)]
pub struct RailLayout {
    pub languette: Boite,
    pub panneau: Option<Boite>,
    pub boutons: Vec<TopbarButtonDef>,
}

/// **Barre ou rail ?** La barre, si elle tient en icônes seules dans `largeur`. Replié chaque
/// fois qu'il disparaît : le retrouver ouvert en rétrécissant la fenêtre surprendrait.
pub fn decider(ui: &mut UiState, typo: &Typography, store: &Store, largeur: f32) {
    let tient = super::boutons::tient_en_icones(largeur, ui, typo, store.nombre_d_images());
    ui.rail.actif = !tient;
    if tient {
        ui.rail.ouvert = false;
    }
}

/// **Le rail placé**, sous la bande, au bord gauche d'une fenêtre haute de `hauteur`.
pub fn layout_rail(ui: &UiState, typo: &Typography, hauteur: f32) -> RailLayout {
    let s = ui.scale();
    let haut = ui.header_height() + ECART * s;
    let (lw, lh) = (LANGUETTE.0 * s, LANGUETTE.1 * s);
    if !ui.rail.ouvert {
        return RailLayout {
            languette: (0.0, haut, lw, lh),
            panneau: None,
            boutons: Vec::new(),
        };
    }
    let pas = (CASE + ECART) * s;
    let liste = les_boutons(ui, typo);
    // Les rangées que la hauteur tient (une au moins), puis les colonnes qu'il faut.
    let rangees_max = (((hauteur - haut - ECART * s) / pas).floor() as usize).max(1);
    let colonnes = liste.len().div_ceil(rangees_max).max(1);
    let boutons: Vec<TopbarButtonDef> = liste
        .into_iter()
        .enumerate()
        .map(|(i, b)| TopbarButtonDef {
            x: ECART * s + (i % colonnes) as f32 * pas,
            y: haut + ECART * s + (i / colonnes) as f32 * pas,
            w: CASE * s,
            h: CASE * s,
            label: "",
            is_tool: true,
            ..b
        })
        .collect();
    let rangees = boutons.len().div_ceil(colonnes) as f32;
    let largeur = ECART * s + colonnes as f32 * pas;
    let haut_du_panneau = ECART * s + rangees * pas;
    RailLayout {
        languette: (largeur, haut, lw, lh),
        panneau: Some((0.0, haut, largeur, haut_du_panneau)),
        boutons,
    }
}

fn dans((x, y, w, h): Boite, px: f32, py: f32) -> bool {
    px >= x && px < x + w && py >= y && py < y + h
}

/// **Un clic au rail** : `None` s'il n'y est pas — le clic continue sa descente ; sinon ce
/// qu'il déclenche, qui peut n'être rien (la languette, le fond du panneau).
pub fn clic(
    (x, y): (f32, f32),
    hauteur: f32,
    ui: &mut UiState,
    typo: &Typography,
) -> Option<Option<UiAction>> {
    if !ui.rail.actif {
        return None;
    }
    let rail = layout_rail(ui, typo, hauteur);
    if dans(rail.languette, x, y) {
        ui.rail.ouvert = !ui.rail.ouvert;
        return Some(None);
    }
    if let Some(b) = rail
        .boutons
        .iter()
        .find(|b| dans((b.x, b.y, b.w, b.h), x, y))
    {
        ui.rail.ouvert = false;
        return Some(Some(effet_du_bouton(ui, b.action.clone())));
    }
    rail.panneau.filter(|p| dans(*p, x, y)).map(|_| None)
}

/// Ce dont le dessin du rail dépend : chaque case (sa place, son état, son icône), la
/// languette, l'échelle.
type CleDuRail = (Vec<(u32, u32, bool, crate::icons::IconType)>, Boite, u32);

/// Le rail déjà dessiné, et ce qui l'a décidé.
pub struct RailCache {
    pixmap: Pixmap,
    cle: CleDuRail,
    origine: (f32, f32),
    pub(super) dessins: u64,
}

/// **Dessine le rail**, depuis son cache s'il n'a pas changé : vingt icônes à chaque image
/// coûteraient deux millisecondes, et le rail ne change qu'au toucher.
pub(super) fn render_rail(
    pixmap: &mut PixmapMut,
    ui: &mut UiState,
    typo: &Typography,
    theme: &Theme,
) {
    if !ui.rail.actif {
        return;
    }
    let rail = layout_rail(ui, typo, pixmap.height() as f32);
    let s = ui.scale();
    let cle = (
        rail.boutons
            .iter()
            .map(|b| (b.x.to_bits(), b.y.to_bits(), b.active, b.icon))
            .collect(),
        rail.languette,
        s.to_bits(),
    );
    let (ox, oy) = (0.0, rail.languette.1);
    let perime = ui.rail_cache.as_ref().is_none_or(|c| c.cle != cle);
    if perime {
        let fin_x = rail.languette.0 + rail.languette.2;
        let fin_y = rail
            .panneau
            .map_or(0.0, |p| p.1 + p.3)
            .max(rail.languette.1 + rail.languette.3);
        let Some(mut tampon) = Pixmap::new(fin_x.ceil() as u32, (fin_y - oy).ceil() as u32) else {
            return;
        };
        dessiner(&mut tampon.as_mut(), &rail, theme, s, (ox, oy));
        let dessins = ui.rail_cache.as_ref().map_or(0, |c| c.dessins) + 1;
        ui.rail_cache = Some(RailCache {
            pixmap: tampon,
            cle,
            origine: (ox, oy),
            dessins,
        });
    }
    if let Some(cache) = &ui.rail_cache {
        crate::composition::poser(pixmap, &cache.pixmap, cache.origine, Melange::Composer);
    }
}

/// Le panneau, ses cases et la languette, dans un tampon dont l'origine est `origine` à l'écran.
fn dessiner(
    tampon: &mut PixmapMut,
    rail: &RailLayout,
    theme: &Theme,
    s: f32,
    (ox, oy): (f32, f32),
) {
    let boite = |(x, y, w, h): Boite| Rect::from_xywh(x - ox, y - oy, w, h);
    if let Some(p) = rail.panneau {
        if let Some(r) = boite(p) {
            fill_crisp(tampon, r, theme.bg_header);
        }
        // Le filet du bord droit : une zone se lit par sa bordure (style.md).
        if let Some(r) = boite((p.0 + p.2 - 1.0, p.1, 1.0, p.3)) {
            fill_crisp(tampon, r, theme.border_subtle);
        }
    }
    for b in &rail.boutons {
        let rect = ScaledRect {
            x: b.x - ox,
            y: b.y - oy,
            w: b.w,
            h: b.h,
            scale: s * CASE / 30.0,
        };
        let etat = ButtonState {
            active: b.active,
            hover: false,
        };
        draw_tool_button(tampon, theme, rect, b.icon, etat);
    }
    let l = rail.languette;
    if let Some(r) = boite(l) {
        fill_crisp(tampon, r, theme.bg_header);
    }
    fleche(
        tampon,
        (l.0 - ox, l.1 - oy, l.2, l.3),
        rail.panneau.is_some(),
        theme,
        s,
    );
}

/// La flèche de la languette : vers la droite pour déplier, vers la gauche pour replier.
fn fleche(tampon: &mut PixmapMut, (x, y, w, h): Boite, ouvert: bool, theme: &Theme, s: f32) {
    let (cx, cy, d) = (x + w / 2.0, y + h / 2.0, 5.0 * s);
    let sens = if ouvert { -1.0 } else { 1.0 };
    let mut pb = PathBuilder::new();
    pb.move_to(cx - sens * d * 0.6, cy - d);
    pb.line_to(cx + sens * d * 0.6, cy);
    pb.line_to(cx - sens * d * 0.6, cy + d);
    pb.close();
    let Some(chemin) = pb.finish() else {
        return;
    };
    let mut paint = Paint::default();
    paint.set_color(theme.text_primary);
    paint.anti_alias = true;
    tampon.fill_path(
        &chemin,
        &paint,
        FillRule::Winding,
        Transform::identity(),
        None,
    );
}

#[cfg(test)]
mod tests;
