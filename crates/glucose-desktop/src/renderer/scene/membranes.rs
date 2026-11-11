//! **Les membranes à l'écran** : leurs champs, leurs titres, leurs poignées, leurs réglettes.
//!
//! Extrait de [`super`] quand la loi des régions y est entrée (fiche 105) : la scène approchait des
//! six cents lignes que la fiche 05 admet. La loi elle-même : [`champs`], et
//! [`glucose_core::membrane_forme::Regions`].
//!
//! Les membranes suivent la même règle que les cartes (SCALE-1) : leur rayon, leurs halos,
//! leur cadre, leur pointillé et leur titre viennent d'une seule mise à l'échelle. Avant
//! R-45, le rayon était `clamp(4, 60)`, le décalage du titre `clamp(8, 20)` et
//! `clamp(12, 24)`, et sa police `clamp(12, 22)` — quatre seuils pour un seul cadre, plus
//! un pointillé de 10 px qui ne suivait pas le zoom du tout.
//!
//! # REGION-1 — le sol, ce qui se pose entre, les frontières
//!
//! Chaque membrane se peint comme une région de carte : la couleur en un point est celle de la
//! plus petite région qui le contient. Les deux voies posent la même liste dans le même ordre —
//! le sol de toutes les régions, puis la grille et les zones, puis toutes les frontières, puis
//! les titres —, le processeur par [`draw_membranes`], la carte graphique par ce que
//! [`formes_des_membranes`] lui confie.

mod champs;
mod saisie;

pub(crate) use champs::{sans_couleur, teinte_de_membrane, BLEU_DE_TAURI};

use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::renderer::domain::{draw_domain_gauge, gauge_width};
use crate::renderer::handles::draw_resize_handles;
use crate::renderer::pass::Clip;
use crate::renderer::scale::WorldScale;
use crate::renderer::{PaintKit, TextEditSession};
use crate::theme::Theme;
use crate::typography::{Face, TextStyle};
use glucose_core::membrane_forme::{Arrondi, Membrane, Regions};
use glucose_core::membrane_titre::porte_un_titre;
use glucose_core::quadtree::Visibles;
use glucose_core::resize::Handle;
use glucose_core::store::Store;
use glucose_core::types::Annotation;
use tiny_skia::{Color, PixmapMut};

/// Rayon des coins d'une membrane, en unités monde.
const MEMBRANE_RADIUS: f32 = 60.0;
/// Débords des deux couches de halo d'une membrane, en unités monde.
const MEMBRANE_GLOW: [(f32, u8); 2] = [(20.0, 8), (10.0, 14)];
/// Épaisseur du cadre d'une membrane, en unités monde.
const MEMBRANE_BORDER: f32 = 2.0;
/// Longueur d'un tiret du pointillé, en unités monde.
const MEMBRANE_DASH: f32 = 10.0;

// ── La mise en page ─────────────────────────────────────────────────────────

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
    /// Le titre suit la loi du noyau (TITRE-1) : chacune de ses mesures grandit comme la racine de
    /// la surface de la membrane, à partir de celles d'une membrane neuve.
    fn new(width: f32, height: f32) -> Self {
        use glucose_core::membrane_titre as titre;
        let s = titre::echelle(f64::from(width), f64::from(height));
        Self {
            width,
            height,
            radius: MEMBRANE_RADIUS,
            label_dx: (titre::RETRAIT * s) as f32,
            label_dy: (titre::ELEVATION * s) as f32,
            label_font: (titre::CORPS * s) as f32,
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

/// Une membrane que l'écran montre : ce que le document en dit, et où elle se pose.
struct MembraneVue<'a> {
    /// Son titre : celui du document, ou **ce qu'on y tape** quand on l'écrit (fiche 99).
    text: Option<&'a str>,
    /// La saisie de son titre, s'il s'écrit : sa sélection et son curseur se dessinent.
    saisie: Option<&'a TextEditSession>,
    domains: &'a [glucose_core::types::DomainAssignment],
    /// Son coin haut-gauche, à l'écran.
    coin: (f32, f32),
    layout: MembraneLayout,
    teinte: (u8, u8, u8),
    selectionnee: bool,
    /// Choisie seule, ou avec d'autres en un groupe qui porte les poignées (fiche 53 § 10).
    poignees: bool,
    scale: WorldScale,
}

/// **Chaque membrane visible, une fois, de la plus grande à la plus petite.** Le culling et la mise à l'échelle ne s'écrivent
/// qu'ici : la forme et les ornements les lisent, et ne peuvent donc pas diverger.
///
/// La saisie en cours y remplace le titre du document, **à la source** : le titre, sa mesure, son
/// ajustement à la largeur ne connaissent qu'un texte — celui qu'on voit.
fn pour_chaque_membrane(
    (store, saisie): (&Store, Option<&TextEditSession>),
    pass: ViewPass<'_>,
    taille: (f32, f32),
    mut faire: impl FnMut(MembraneVue<'_>),
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let en_groupe = store.emprise_du_groupe(&board.id).is_some();
    let scale = pass.echelle();
    let clip = Clip {
        width: taille.0,
        height: taille.1,
        top: pass.header_h,
    };
    // La plus grande d'abord, la plus petite par-dessus (REGION-1) : la loi des cartes, que
    // l'ordre du modèle ne donne pas — une fille peut y précéder sa mère. À surface égale, l'ordre
    // du modèle.
    let mut membranes: Vec<(&Annotation, f64)> = Visibles::nouvelles(pass.visibles, board)
        .annotations()
        .filter_map(|ann| match ann {
            Annotation::Membrane { width, height, .. } => Some((ann, width * height)),
            _ => None,
        })
        .collect();
    membranes.sort_by(|a, b| b.1.total_cmp(&a.1));
    for (ann, _) in membranes {
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
        // Le titre au-dessus du bord compte (TITRE-1) : une membrane encore sous l'écran peut y
        // montrer son nom.
        let titre = layout.label_dy;
        if clip.rejects(coin.0, coin.1 - titre, layout.width, layout.height + titre) {
            continue;
        }
        let saisie = saisie.filter(|s| s.ann_id == *id);
        faire(MembraneVue {
            text: saisie.map_or(text.as_deref(), |s| Some(s.buffer.as_str())),
            saisie,
            domains,
            coin,
            layout,
            teinte: champs::teinte_de_membrane(color.as_deref()),
            selectionnee: store.selected_annotation_ids.contains(id),
            poignees: !en_groupe,
            scale,
        });
    }
}

impl<'a> MembraneVue<'a> {
    /// **Le titre tel qu'il se pose** : son texte, son corps, son retrait et son élévation à
    /// l'écran — ou rien pour une membrane sans titre. **Aucun ne se cache**, à aucune distance (TITRE-1, sa parole
    /// du 8/11/2026). Le corps est celui de la loi, ramené s'il le faut au plus grand qui tient
    /// dans la largeur de la membrane (un retrait de chaque côté) — un nom ne déborde jamais sur
    /// la région voisine, la règle des cartes. Il ne fait que rétrécir : ce que la disposition et
    /// l'encre réservent au titre de la loi lui suffit toujours. Toutes ses mesures rétrécissent
    /// ensemble — le corps, le retrait de chaque côté, l'élévation —, comme une membrane plus
    /// petite : sa ligne de base garde sa proportion.
    ///
    /// Rend le texte, puis le corps, le retrait et l'élévation, en pixels.
    fn titre(&self, kit: PaintKit<'_>) -> Option<(&'a str, [f32; 3])> {
        // Un titre vide qu'on écrit a sa place : son curseur s'y pose avant la première lettre.
        let label = self
            .text
            .filter(|l| self.saisie.is_some() || porte_un_titre(Some(l)))?;
        let l = &self.layout;
        let large = kit
            .typography
            .measure_text(label, l.label_font, Face::Bold)
            .0;
        let ajuste = (l.width / (large + 2.0 * l.label_dx)).min(1.0);
        let [corps, retrait, elevation] =
            [l.label_font, l.label_dx, l.label_dy].map(|v| v * ajuste);
        Some((label, [corps, retrait, elevation]))
    }
}

/// **Les champs des régions que l'écran montre** (REGION-1), dans l'ordre où ils se posent :
/// chacune pose son sol, puis sa frontière, sur le fond du canevas du thème — celui que le mode
/// Focus teinte. Ce qui ne se voit pas n'y est pas.
pub(in crate::renderer) fn formes_des_membranes(
    (store, theme): (&Store, &Theme),
    pass: ViewPass<'_>,
    taille: (u32, u32),
) -> Regions {
    let ecran = (taille.0 as f32, taille.1 as f32);
    let c = theme.bg_canvas;
    let canevas = [c.red(), c.green(), c.blue()];
    let mut regions = Regions::default();
    pour_chaque_membrane((store, None), pass, ecran, |m| {
        let (sol, frontiere) = m.champs(canevas);
        regions.poser(sol, frontiere, ecran);
    });
    regions
}

/// **Des champs de membrane en SVG** (P5, fiche 69) : leurs remplissages, puis leur trait. Les
/// couches d'un champ portent la même teinte : la composition du lecteur rend exactement l'opacité
/// que la loi calcule (MEMB-FORME-1). Le bord part, comme le `<rect>` du SVG, du haut juste après
/// le coin haut gauche, dans le sens des aiguilles d'une montre : le pointillé tombe au même
/// endroit.
pub(in crate::renderer) fn champs_en_svg(svg: &mut crate::renderer::svg::Svg, champs: &[Membrane]) {
    use crate::renderer::svg::n;
    for forme in champs {
        let [r, g, b] = forme.teinte.map(|c| (c * 255.0).round() as u8);
        let teinte = crate::renderer::svg::hex((r, g, b));
        let boite = |a: &Arrondi| (a.gauche, a.haut, a.droite - a.gauche, a.bas - a.haut);
        for (arrondi, alpha) in forme.remplissages.iter().filter(|(_, a)| *a > 0.0) {
            svg.rectangle(boite(arrondi), arrondi.rayon, (&teinte, *alpha));
        }
        let bord = forme.bord;
        if bord.alpha <= 0.0 {
            continue;
        }
        let (x, y, l, h) = boite(&bord.forme);
        let tirets = bord.pointille.map_or(String::new(), |p| {
            let t = n(p.tiret);
            format!(r#" stroke-dasharray="{t} {t}" stroke-linecap="round""#)
        });
        svg.poser(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="none" stroke="{teinte}" stroke-opacity="{}" stroke-width="{}"{tirets}/>"#,
            n(x), n(y), n(l), n(h), n(bord.forme.rayon), n(bord.alpha), n(bord.demi_largeur * 2.0)
        ));
    }
}

/// **Les titres et les réglettes des membranes en SVG**, au-dessus de toutes les formes, comme au
/// canevas : le titre détouré du fond, puis la réglette, alignée à droite du bord haut.
pub(in crate::renderer) fn ornements_des_membranes_en_svg(
    (kit, store, pass): (PaintKit<'_>, &Store, ViewPass<'_>),
    taille: (u32, u32),
    svg: &mut crate::renderer::svg::Svg,
) {
    let ecran = (taille.0 as f32, taille.1 as f32);
    pour_chaque_membrane((store, None), pass, ecran, |m| {
        ornements_en_svg(kit, svg, &m);
    });
}

/// Le titre et la réglette d'une membrane — la sœur de `poser_les_ornements`.
fn ornements_en_svg(kit: PaintKit<'_>, svg: &mut crate::renderer::svg::Svg, m: &MembraneVue<'_>) {
    let (sx, sy) = m.coin;
    let (layout, (r, g, b)) = (m.layout, m.teinte);
    if let Some((label, [corps, retrait, elevation])) = m.titre(kit) {
        let style = TextStyle {
            size: corps,
            color: Color::from_rgba8(r, g, b, 255),
            face: Face::Bold,
        };
        let at = (sx + retrait, sy - elevation);
        svg.texte_detoure(kit.typography, at, label, (style, kit.theme.bg_canvas));
    }
    let gauge_x = sx + layout.width - gauge_width(m.scale, m.domains.len());
    let mut vecteurs = crate::renderer::surface::Vecteurs {
        svg,
        typographie: kit.typography,
    };
    let coin = (gauge_x, sy);
    crate::renderer::domain::reglette(&mut vecteurs, kit.tints, m.scale, coin, m.domains);
}

/// **Les membranes entières, au processeur** : le sol des régions, puis ce que `entre` pose sur
/// lui — la grille, les zones (REGION-1) —, puis leurs frontières, puis leurs ornements. Rend vrai
/// si au moins une membrane a reçu de l'encre.
///
/// Toutes les formes d'abord, les ornements ensuite — l'ordre de la voie graphique, où la
/// carte pose les formes sous la couche qui porte les ornements. Un titre passe donc au-dessus
/// du voile d'une membrane voisine sur les deux voies, au lieu de dépendre de la voie.
pub(in crate::renderer) fn draw_membranes(
    (kit, pixmap): (PaintKit<'_>, &mut PixmapMut),
    (store, saisie): (&Store, Option<&TextEditSession>),
    pass: ViewPass<'_>,
    entre: impl FnOnce(&mut PixmapMut),
) -> bool {
    let (l, h) = (pixmap.width(), pixmap.height());
    let regions = formes_des_membranes((store, kit.theme), pass, (l, h));
    let mut encre = peindre(pixmap, regions.sol());
    entre(pixmap);
    encre |= peindre(pixmap, regions.frontieres());
    draw_membrane_ornaments(kit, pixmap, (store, saisie), pass) || encre
}

/// Peint des champs de membrane, dans l'ordre. Rend vrai si l'un d'eux a posé de l'encre.
fn peindre(pixmap: &mut PixmapMut, champs: &[Membrane]) -> bool {
    let (l, h) = (pixmap.width(), pixmap.height());
    let (pixels, _) = pixmap.data_mut().as_chunks_mut::<4>();
    let mut encre = false;
    for champ in champs {
        encre |= glucose_core::membrane_forme::peindre(champ, pixels, l, h);
    }
    encre
}

/// **Ce qui reste au processeur sur les deux voies** : le titre, les poignées, la réglette des
/// domaines. Rend vrai si quelque chose a été posé — c'est ce qui décide si la couche du
/// dessous doit être relevée et envoyée.
pub(in crate::renderer) fn draw_membrane_ornaments(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store_et_saisie: (&Store, Option<&TextEditSession>),
    pass: ViewPass<'_>,
) -> bool {
    let taille = (pixmap.width() as f32, pixmap.height() as f32);
    let mut encre = false;
    pour_chaque_membrane(store_et_saisie, pass, taille, |m| {
        encre |= poser_les_ornements(kit, pixmap, &m);
    });
    encre
}

fn poser_les_ornements(kit: PaintKit<'_>, pixmap: &mut PixmapMut, m: &MembraneVue<'_>) -> bool {
    let (sx, sy) = m.coin;
    let (layout, (r, g, b)) = (m.layout, m.teinte);
    let mut encre = !m.domains.is_empty();
    if let Some((label, [corps, retrait, elevation])) = m.titre(kit) {
        let titre = saisie::Titre {
            texte: label,
            at: (sx + retrait, sy - elevation),
            corps,
        };
        let couleur = Color::from_rgba8(r, g, b, 255);
        if let Some(s) = m.saisie {
            saisie::surligner(kit, pixmap, titre, s);
        }
        let style = TextStyle {
            size: corps,
            color: couleur,
            face: Face::Bold,
        };
        let (x, y) = titre.at;
        kit.typography
            .draw_text_with_outline(pixmap, label, x, y, style, kit.theme.bg_canvas);
        if let Some(s) = m.saisie {
            saisie::poser_le_curseur(kit, pixmap, titre, (s, m.scale, couleur));
        }
        encre = true;
    }
    if m.selectionnee && m.poignees {
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

#[cfg(test)]
mod titres;

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
