//! Les passes de scène qui ne sont pas des annotations : grille, membranes, images,
//! guides d'alignement et boîte de sélection élastique.
//!
//! Les membranes suivent la même règle que les cartes (SCALE-1) : leur rayon, leurs halos,
//! leur cadre, leur pointillé et leur titre viennent d'une seule mise à l'échelle. Avant
//! R-45, le rayon était `clamp(4, 60)`, le décalage du titre `clamp(8, 20)` et
//! `clamp(12, 24)`, et sa police `clamp(12, 22)` — quatre seuils pour un seul cadre, plus
//! un pointillé de 10 px qui ne suivait pas le zoom du tout.
//!
//! Les guides, les poignées et les points de la grille, eux, gardent une **taille écran
//! constante** : c'est l'exception de SCALE-1. Les poignées elles-mêmes sont dessinées par
//! [`super::handles`], aux positions que le test de clic utilise (RESIZE-1).

pub mod grid;
pub(super) mod image;

use super::domain::{draw_domain_gauge, gauge_width};
use super::handles::draw_resize_handles;
use super::pass::{Clip, SELECTION_RING};
use super::scale::WorldScale;
use super::{parse_hex_color, PaintKit};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle};
use glucose_core::membrane_forme::{Arrondi, Bord, Membrane};
use glucose_core::quadtree::Visibles;
use glucose_core::resize::Handle;
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Viewport};
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Rayon des coins d'une membrane, en unités monde.
const MEMBRANE_RADIUS: f32 = 60.0;
/// Débords des deux couches de halo d'une membrane, en unités monde.
const MEMBRANE_GLOW: [(f32, u8); 2] = [(20.0, 8), (10.0, 14)];
/// Décalage horizontal du titre depuis le bord gauche, en unités monde.
const MEMBRANE_LABEL_DX: f32 = 16.0;
/// Élévation du titre au-dessus du bord haut, en unités monde.
const MEMBRANE_LABEL_DY: f32 = 18.0;
/// Corps du titre d'une membrane, en unités monde.
const MEMBRANE_LABEL_FONT: f32 = 16.0;
/// Épaisseur du cadre d'une membrane, en unités monde.
const MEMBRANE_BORDER: f32 = 2.0;
/// Longueur d'un tiret du pointillé, en unités monde.
const MEMBRANE_DASH: f32 = 10.0;

// ── Membranes ───────────────────────────────────────────────────────────────

/// Mise en page d'une membrane. **En unités monde tant que `scaled` n'a pas été appelée.**
#[derive(Clone, Copy)]
struct MembraneLayout {
    width: f32,
    height: f32,
    radius: f32,
    label_dx: f32,
    label_dy: f32,
    label_font: f32,
    border: f32,
    dash: f32,
}

impl MembraneLayout {
    fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            radius: MEMBRANE_RADIUS,
            label_dx: MEMBRANE_LABEL_DX,
            label_dy: MEMBRANE_LABEL_DY,
            label_font: MEMBRANE_LABEL_FONT,
            border: MEMBRANE_BORDER,
            dash: MEMBRANE_DASH,
        }
    }

    /// L'UNIQUE transformation d'échelle de la membrane (SCALE-1).
    fn scaled(self, s: WorldScale) -> Self {
        Self {
            width: s.world(self.width),
            height: s.world(self.height),
            radius: s.world(self.radius),
            label_dx: s.world(self.label_dx),
            label_dy: s.world(self.label_dy),
            label_font: s.world(self.label_font),
            border: s.world(self.border),
            dash: s.world(self.dash),
        }
    }
}

/// La couleur d'une membrane qui n'en a pas : `#60a5fa`, comme chez Glucose Tauri. Le fond du
/// mode Focus la reprend ([`super::focus`]).
pub(crate) const MEMBRANE_SANS_COULEUR: (u8, u8, u8) = (96, 165, 250);

/// L'opacité du fond translucide d'une membrane, sur 255.
const MEMBRANE_FILL_ALPHA: u8 = 8;
/// L'opacité de son contour au repos, sur 255.
const MEMBRANE_BORDER_ALPHA: u8 = 115;
/// L'opacité de son contour quand elle est sélectionnée, sur 255.
const MEMBRANE_SELECTED_ALPHA: u8 = 235;

/// La teinte d'une membrane : sa couleur, ou celle d'une membrane qui n'en a pas.
fn teinte_de_membrane(couleur: Option<&str>) -> (u8, u8, u8) {
    let (r, g, b) = MEMBRANE_SANS_COULEUR;
    couleur.map_or(MEMBRANE_SANS_COULEUR, |c| parse_hex_color(c, r, g, b))
}

/// Une membrane que l'écran montre : ce que le document en dit, et où elle se pose.
struct MembraneVue<'a> {
    text: Option<&'a str>,
    domains: &'a [glucose_core::types::DomainAssignment],
    /// Son coin haut-gauche, à l'écran.
    coin: (f32, f32),
    layout: MembraneLayout,
    teinte: (u8, u8, u8),
    selectionnee: bool,
    scale: WorldScale,
}

/// **Chaque membrane visible, une fois.** Le culling et la mise à l'échelle ne s'écrivent
/// qu'ici : la forme et les ornements les lisent, et ne peuvent donc pas diverger.
fn pour_chaque_membrane(
    store: &Store,
    pass: ViewPass<'_>,
    taille: (f32, f32),
    mut faire: impl FnMut(MembraneVue<'_>),
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let scale = pass.echelle();
    let clip = Clip {
        width: taille.0,
        height: taille.1,
        top: pass.header_h,
    };
    for ann in Visibles::nouvelles(pass.visibles, board).annotations() {
        let Annotation::Membrane {
            id,
            x,
            y,
            width,
            height,
            text,
            color,
            domains,
            ..
        } = ann
        else {
            continue;
        };
        let layout = MembraneLayout::new(*width as f32, *height as f32).scaled(scale);
        let (wx, wy) = world_to_screen(*x, *y, &pass.vp);
        let coin = (wx as f32, wy as f32);
        if clip.rejects(coin.0, coin.1, layout.width, layout.height) {
            continue;
        }
        faire(MembraneVue {
            text: text.as_deref(),
            domains,
            coin,
            layout,
            teinte: teinte_de_membrane(color.as_deref()),
            selectionnee: store.selected_annotation_ids.contains(id),
            scale,
        });
    }
}

impl MembraneVue<'_> {
    /// **La membrane telle que la loi la décrit** (MEMB-FORME-1) : deux halos, le fond
    /// translucide, le contour — une seule teinte, donc un seul champ d'opacité.
    fn forme(&self) -> Membrane {
        let (l, (x, y)) = (&self.layout, self.coin);
        let fond = Arrondi::nouveau(x, y, l.width, l.height, l.radius);
        // Le halo appartient au cadre, donc il le suit ; vu de trop loin pour qu'on lise un
        // titre, il n'y a plus de halo à voir non plus.
        let halo = |(pad_monde, alpha): (f32, u8)| {
            let pad = self.scale.world(pad_monde);
            let forme = Arrondi::nouveau(
                x - pad,
                y - pad,
                l.width + pad * 2.0,
                l.height + pad * 2.0,
                l.radius + pad * 0.5,
            );
            let alpha = if self.scale.draws_detail() {
                f32::from(alpha) / 255.0
            } else {
                0.0
            };
            (forme, alpha)
        };
        let (r, g, b) = self.teinte;
        Membrane {
            remplissages: [
                halo(MEMBRANE_GLOW[0]),
                halo(MEMBRANE_GLOW[1]),
                (fond, f32::from(MEMBRANE_FILL_ALPHA) / 255.0),
            ],
            bord: self.bord(fond),
            teinte: [
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
            ],
        }
    }

    /// Le contour d'une membrane : pointillé au repos, plein quand elle est prise.
    ///
    /// # Le pointillé s'arrête où le pixel s'arrête
    ///
    /// Un tiret plus fin qu'un pixel ne se voit pas comme un tiret : l'œil n'y lit qu'un trait
    /// continu, à moitié moins dense puisque la moitié du parcours est vide. On dessine donc
    /// exactement cela — un trait plein, d'opacité moitié. Le seuil n'est pas choisi : c'est
    /// le pixel, la plus petite chose qu'un écran sache montrer.
    fn bord(&self, forme: Arrondi) -> Bord {
        let dash = self.layout.dash;
        let pointille = !self.selectionnee && dash >= 1.0;
        let (largeur, opacite) = if self.selectionnee {
            (self.scale.screen(SELECTION_RING), MEMBRANE_SELECTED_ALPHA)
        } else if pointille {
            (self.layout.border, MEMBRANE_BORDER_ALPHA)
        } else {
            (self.layout.border, MEMBRANE_BORDER_ALPHA / 2)
        };
        Bord {
            forme,
            demi_largeur: largeur / 2.0,
            pointille: pointille.then(|| forme.pointille(dash)),
            alpha: f32::from(opacite) / 255.0,
        }
    }
}

/// **Les formes des membranes que l'écran montre**, pour la carte (MEMB-FORME-1).
pub(super) fn formes_des_membranes(
    store: &Store,
    pass: ViewPass<'_>,
    taille: (u32, u32),
) -> Vec<Membrane> {
    let mut formes = Vec::new();
    pour_chaque_membrane(store, pass, (taille.0 as f32, taille.1 as f32), |m| {
        formes.push(m.forme());
    });
    formes
}

/// **Les membranes entières, au processeur** : leurs formes, puis leurs ornements. Rend vrai
/// si au moins une membrane a reçu de l'encre.
///
/// Toutes les formes d'abord, les ornements ensuite — l'ordre de la voie graphique, où la
/// carte pose les formes sous la couche qui porte les ornements. Un titre passe donc au-dessus
/// du voile d'une membrane voisine sur les deux voies, au lieu de dépendre de la voie.
pub(super) fn draw_membranes(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) -> bool {
    let (l, h) = (pixmap.width(), pixmap.height());
    let mut encre = false;
    {
        let (pixels, _) = pixmap.data_mut().as_chunks_mut::<4>();
        pour_chaque_membrane(store, pass, (l as f32, h as f32), |m| {
            encre |= glucose_core::membrane_forme::peindre(&m.forme(), pixels, l, h);
        });
    }
    draw_membrane_ornaments(kit, pixmap, store, pass) || encre
}

/// **Ce qui reste au processeur sur les deux voies** : le titre, les poignées, la réglette des
/// domaines. Rend vrai si quelque chose a été posé — c'est ce qui décide si la couche du
/// dessous doit être relevée et envoyée.
pub(super) fn draw_membrane_ornaments(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) -> bool {
    let taille = (pixmap.width() as f32, pixmap.height() as f32);
    let mut encre = false;
    pour_chaque_membrane(store, pass, taille, |m| {
        encre |= poser_les_ornements(kit, pixmap, &m);
    });
    encre
}

fn poser_les_ornements(kit: PaintKit<'_>, pixmap: &mut PixmapMut, m: &MembraneVue<'_>) -> bool {
    let (sx, sy) = m.coin;
    let (layout, (r, g, b)) = (m.layout, m.teinte);
    let mut encre = !m.domains.is_empty();
    if let Some(label) = m.text.filter(|l| !l.is_empty() && m.scale.draws_detail()) {
        kit.typography.draw_text_with_outline(
            pixmap,
            label,
            sx + layout.label_dx,
            sy - layout.label_dy,
            TextStyle {
                size: layout.label_font,
                color: Color::from_rgba8(r, g, b, 255),
                face: Face::Bold,
            },
            kit.theme.bg_canvas,
        );
        encre = true;
    }
    if m.selectionnee {
        draw_resize_handles(
            pixmap,
            kit.theme,
            m.scale,
            (sx, sy, layout.width, layout.height),
            &Handle::ALL,
        );
        encre = true;
    }
    // La réglette d'une membrane s'aligne à DROITE de son bord haut : le coin haut-gauche est
    // déjà occupé par le titre protecteur, et deux textes superposés ne se lisent ni l'un ni
    // l'autre.
    let gauge_x = sx + layout.width - gauge_width(m.scale, m.domains.len());
    draw_domain_gauge(
        kit.typography,
        kit.tints,
        pixmap,
        m.scale,
        (gauge_x, sy),
        m.domains,
    );
    encre
}

pub(super) fn draw_guides(
    theme: &Theme,
    pixmap: &mut PixmapMut,
    guides: &SnapGuides,
    vp: &Viewport,
    size: (u32, u32),
    header_h: f32,
) {
    let mut paint = Paint::default();
    paint.set_color(theme.snap_guide);
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    let (w, h) = size;

    for &gx in guides.x.iter().flatten() {
        let (sx, _) = world_to_screen(gx, 0.0, vp);
        let mut pb = PathBuilder::new();
        pb.move_to(sx as f32, header_h);
        pb.line_to(sx as f32, h as f32);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    for &gy in guides.y.iter().flatten() {
        let (_, sy) = world_to_screen(0.0, gy, vp);
        if sy < header_h as f64 {
            continue;
        }
        let mut pb = PathBuilder::new();
        pb.move_to(0.0, sy as f32);
        pb.line_to(w as f32, sy as f32);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

/// Fiche 07 § 7.3 — la sélection élastique : contour blanc à 0,50 de 1 px, intérieur blanc
/// à 0,03. Monochrome, comme toute la chrome.
pub(super) fn draw_selection_box(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    a: (f64, f64),
    b: (f64, f64),
) {
    let rect = Rect::from_xywh(
        a.0.min(b.0) as f32,
        a.1.min(b.1) as f32,
        (a.0 - b.0).abs() as f32,
        (a.1 - b.1).abs() as f32,
    );
    let Some(rect) = rect else {
        return;
    };
    let mut fill = Paint::default();
    fill.set_color(theme.rubberband_fill);
    pixmap.fill_rect(rect, &fill, Transform::identity(), None);

    let mut border = Paint::default();
    border.set_color(theme.rubberband_stroke);
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    pixmap.stroke_path(
        &PathBuilder::from_rect(rect),
        &border,
        &stroke,
        Transform::identity(),
        None,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_1_a_membrane_is_self_similar_at_every_zoom() {
        let world = MembraneLayout::new(800.0, 600.0);
        for zoom in [0.25_f64, 0.5, 1.0, 2.0, 4.0] {
            let screen = world.scaled(WorldScale::new(zoom, 1.0));
            for (on_screen, in_world) in [
                (screen.radius, world.radius),
                (screen.label_dx, world.label_dx),
                (screen.label_dy, world.label_dy),
                (screen.label_font, world.label_font),
                (screen.border, world.border),
                (screen.dash, world.dash),
            ] {
                let expected = in_world / world.width;
                let observed = on_screen / screen.width;
                assert!(
                    (observed - expected).abs() < 1e-6,
                    "zoom {zoom} : rapport {observed} au lieu de {expected}"
                );
            }
        }
    }
}
