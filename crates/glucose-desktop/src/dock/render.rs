//! Le dessin du dock : la passe, le cadre d'un panneau, et l'aiguillage vers son contenu.
//!
//! Ce module ne connaît ni les ancrages ni le glissement — il reçoit une géométrie déjà
//! calculée par [`super::compute_panel_layouts`] et la met en pixels. C'est la frontière que
//! la loi L4 demande : **une seule géométrie**, lue ici et par le clic, jamais recalculée.

use super::paint::Brush;
use super::{
    cache, compute_panel_layouts, domains, organize, plugins, pomodoro, preset, storyboard,
    DockCache, DockManager, PanelLayoutBox, TabId, WidgetRect,
};
use crate::params::{Pointer, ScaledRect, ScreenFrame};
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::store::Store;
use tiny_skia::PixmapMut;

/// Ce que le rendu du dock a besoin de savoir, en dehors du document lui-même.
///
/// Ces cinq valeurs voyageaient une à une, et la liste avait fini par dépasser ce qu'une
/// signature supporte. Elles vont ensemble : c'est l'état de la fenêtre au moment de
/// l'image — sa typographie, son thème, son cadre, sa souris, et les tampons qu'on garde
/// d'une image à l'autre.
pub struct DockPass<'a> {
    pub typo: &'a Typography,
    pub theme: &'a Theme,
    pub screen: ScreenFrame,
    pub pointer: Pointer,
    /// Les tampons des panneaux, ou `None` pour dessiner directement (DOCK-CACHE-1).
    ///
    /// Le rendu direct n'est pas un vestige : c'est lui qui sert de référence aux tests, qui
    /// vérifient que le cache rend la même image.
    pub cache: Option<&'a DockCache>,
}

/// Le rendu du dock : ses panneaux ouverts, dans l'ordre de leurs ancrages.
pub fn render_docks(
    pixmap: &mut PixmapMut,
    dock: &DockManager,
    store: &Store,
    pass: &DockPass<'_>,
) {
    let s = crate::theme::clamp_ui_scale(pass.screen.scale);
    let layouts = compute_panel_layouts(
        dock,
        store,
        pass.screen.width,
        pass.screen.height,
        pass.screen.header_h,
        s,
    );
    // Sans cache, un cache neuf qui ne garde rien : **le même chemin** — un tampon par
    // panneau, posé de la même façon. Peint directement sur l'image, un panneau s'arrondit
    // autrement (une composition de moins par couche transparente), et l'écart, deux niveaux
    // aux coins où des ombres se croisent, cachait ce qu'il fallait voir : un panneau périmé.
    let neuf = cache::DockCache::new();
    let cache = pass.cache.unwrap_or(&neuf);
    for panel in layouts {
        cache::draw_panel_cached(pixmap, cache, dock, store, pass, &panel, s);
    }
}

/// **Les panneaux, confiés à la carte graphique** (PANNEAUX-1) : chacun tenu à jour dans son
/// tampon, et rendu comme une texture qu'elle pose par-dessus la couche du dessus.
///
/// Composés dans la couche du dessus, ils en faisaient effacer, relever et renvoyer toutes les
/// lignes à chaque image — un panneau de toute la hauteur, toute la couche — alors qu'ils ne
/// changent que sous la main. Posés par la carte, un panneau immobile ne coûte plus rien, et un
/// panneau qui change ne renvoie que son rectangle. L'ordre ne change pas : la chrome passait
/// déjà après tout ce que la couche porte.
///
/// Un panneau dont le tampon est refusé — une taille impossible — se dessine dans `pixmap`,
/// comme avant : rien ne disparaît.
pub fn confier_les_docks(
    pixmap: &mut PixmapMut,
    dock: &DockManager,
    store: &Store,
    pass: &DockPass<'_>,
) -> Vec<crate::renderer::voies::APoser> {
    use crate::present::scene_gpu::Pose;
    let s = crate::theme::clamp_ui_scale(pass.screen.scale);
    let layouts = compute_panel_layouts(
        dock,
        store,
        pass.screen.width,
        pass.screen.height,
        pass.screen.header_h,
        s,
    );
    let neuf = cache::DockCache::new();
    let cache = pass.cache.unwrap_or(&neuf);
    let mut confies = Vec::with_capacity(layouts.len());
    for panel in layouts {
        let Some(tenu) = cache::tenir_a_jour(cache, dock, store, pass, &panel, s) else {
            cache::draw_panel_cached(pixmap, cache, dock, store, pass, &panel, s);
            continue;
        };
        confies.push(crate::renderer::voies::APoser {
            identite: cache::identite_du_panneau(panel.tab).into(),
            cle: cache::cle_du_panneau(panel.tab, tenu.generation).into(),
            pose: Pose {
                x: tenu.origine.0,
                y: tenu.origine.1,
                largeur: tenu.taille.0 as f32,
                hauteur: tenu.taille.1 as f32,
                opacite: 1.0,
                angle: 0.0,
                fenetre: Pose::TOUT,
                bornes: Pose::PARTOUT,
            },
            repli: None,
        });
    }
    confies
}

/// La profondeur du voile du liseré, en points (fiche 10 § 5.7 : `inset 0 0 60px`).
const VOILE: f32 = 60.0;
/// L'épaisseur de son trait, en points (`3px solid`).
const TRAIT: f32 = 3.0;

/// **Le liseré du passé tel que la carte le peint** (voir [`crate::present::lisere_gpu`]) : les
/// mêmes nombres que [`lisere_du_passe`], lus au même endroit.
pub fn lisere(theme: &Theme, s: f32) -> crate::present::lisere_gpu::Lisere {
    let rgba = |c: tiny_skia::Color| [c.red(), c.green(), c.blue(), c.alpha()];
    crate::present::lisere_gpu::Lisere {
        voile: VOILE * s,
        trait_: TRAIT * s,
        teinte_du_voile: rgba(theme.temps.voile),
        ambre: rgba(theme.temps.ambre),
    }
}

/// **Le liseré du passé** : quand la Time Machine montre un état passé, un trait ambre borde
/// la fenêtre et un voile ambré glisse vers l'intérieur (fiche 10 § 5.7 : `3px solid`,
/// `inset 0 0 60px`). On ne peut pas oublier qu'on ne regarde pas le présent.
///
/// Le voile est un dégradé linéaire par bord, de la teinte au transparent : aucun nombre de
/// pas à choisir.
///
/// Sur la voie processeur seulement : sur la voie graphique, la carte le peint
/// ([`crate::present::lisere_gpu`]) — au processeur, il coûtait 11,7 ms par image et faisait
/// repartir toute la couche du dessus.
pub fn lisere_du_passe(pixmap: &mut PixmapMut, pass: &DockPass<'_>, s: f32) {
    use tiny_skia::{
        Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Point, Rect, SpreadMode,
        Transform,
    };
    let (w, h) = (pass.screen.width, pass.screen.height);
    let theme = pass.theme;
    let voile = VOILE * s;
    let transparent = Color::from_rgba(
        theme.temps.voile.red(),
        theme.temps.voile.green(),
        theme.temps.voile.blue(),
        0.0,
    )
    .unwrap_or(Color::TRANSPARENT);
    // (rectangle, départ du dégradé, arrivée) pour chaque bord.
    let bords = [
        ((0.0, 0.0, w, voile), (0.0, 0.0), (0.0, voile)),
        ((0.0, h - voile, w, voile), (0.0, h), (0.0, h - voile)),
        ((0.0, 0.0, voile, h), (0.0, 0.0), (voile, 0.0)),
        ((w - voile, 0.0, voile, h), (w, 0.0), (w - voile, 0.0)),
    ];
    for ((x, y, bw, bh), de, a) in bords {
        let (Some(rect), Some(degrade)) = (
            Rect::from_xywh(x, y, bw, bh),
            LinearGradient::new(
                Point::from_xy(de.0, de.1),
                Point::from_xy(a.0, a.1),
                vec![
                    GradientStop::new(0.0, theme.temps.voile),
                    GradientStop::new(1.0, transparent),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ),
        ) else {
            continue;
        };
        let paint = Paint {
            shader: degrade,
            anti_alias: false,
            ..Paint::default()
        };
        let chemin = PathBuilder::from_rect(rect);
        pixmap.fill_path(
            &chemin,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
    let brush = Brush::nouveau((pass.typo, theme), s, pass.pointer, (0.0, 0.0));
    let trait_ = TRAIT * s;
    let cadre = WidgetRect::new(trait_ / 2.0, trait_ / 2.0, w - trait_, h - trait_);
    brush.stroke(pixmap, cadre, 0.0, theme.temps.ambre, trait_);
}

/// Dessine un panneau — son cadre puis son contenu — là où le pinceau vise.
pub(super) fn draw_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush<'_>,
    dock: &DockManager,
    store: &Store,
    panel: &PanelLayoutBox,
    s: f32,
) {
    draw_frame(pixmap, brush, panel, panel.seen());
    let (cadre, vu) = (panel.contenu(s), panel.contenu_visible(s));
    if cadre == vu {
        // Le contenu ne passe jamais sur la poignée (TIROIR-1).
        draw_content(pixmap, brush, dock, store, panel.tab, cadre);
    } else {
        draw_content_defile(pixmap, brush, (dock, store), panel, (cadre, vu));
    }
}

/// **TIROIR-6 — un contenu qui défile se dessine dans un tampon de sa seule partie visible** : ce
/// qui dépasse tombe hors du tampon, et la poignée reste intacte (TIROIR-1), comme le tiroir du
/// dessous. L'origine du tampon est entière (DOCK-CACHE-1) ; son bas s'arrête avant la poignée.
///
/// **Le contenu défile sous le filet du cadre** : le tampon s'arrête au bord intérieur du filet
/// (un pixel à cheval sur le bord du tiroir), en haut et sur les côtés — le filet reste dessus,
/// comme la poignée en bas.
///
/// **Le tampon part de ce que le tiroir montre déjà là** (son fond), et non du transparent : le
/// contenu s'y dessine exactement comme sur le tiroir, puis le tampon reprend sa place, recopié
/// (`Remplacer`) — un seul arrondi, celui du dessin direct, au lieu de deux.
fn draw_content_defile(
    pixmap: &mut PixmapMut,
    brush: &Brush<'_>,
    (dock, store): (&DockManager, &Store),
    panel: &PanelLayoutBox,
    (cadre, vu): (ScaledRect, ScaledRect),
) {
    let filet = brush.px(1.0) / 2.0;
    let (x0, y0) = ((vu.x + filet).ceil(), (vu.y + filet).ceil());
    let (x1, y1) = ((vu.x + vu.w - filet).floor(), (vu.y + vu.h).floor());
    // La région dans le tampon visé, bornée à lui : un tiroir peut déborder le cache du panneau.
    let tampon = tiny_skia::IntRect::from_xywh(0, 0, pixmap.width(), pixmap.height());
    let Some(region) = tiny_skia::IntRect::from_xywh(
        (x0 - brush.origin.0) as i32,
        (y0 - brush.origin.1) as i32,
        (x1 - x0) as u32,
        (y1 - y0) as u32,
    )
    .zip(tampon)
    .and_then(|(r, tampon)| r.intersect(&tampon)) else {
        return;
    };
    let Some(mut visible) = pixmap.as_ref().clone_rect(region) else {
        return;
    };
    let coin = (
        region.x() as f32 + brush.origin.0,
        region.y() as f32 + brush.origin.1,
    );
    brush.vers(coin, |b| {
        draw_content(&mut visible.as_mut(), b, dock, store, panel.tab, cadre);
    });
    crate::composition::poser(
        pixmap,
        &visible,
        (region.x() as f32, region.y() as f32),
        glucose_core::report::Melange::Remplacer,
    );
    draw_la_part_montree(pixmap, brush, panel, vu);
}

/// **Ce qui reste à voir** : le filet du bord intérieur du tiroir — celui qui regarde le canevas —
/// prend l'accent sur la part montrée, à sa place dans le tout (TIROIR-6). Aucun trait de plus :
/// le filet du cadre dit déjà où finit le tiroir, il dit aussi ce qu'il cache.
fn draw_la_part_montree(
    pixmap: &mut PixmapMut,
    brush: &Brush<'_>,
    panel: &PanelLayoutBox,
    vu: ScaledRect,
) {
    let tout = (panel.naturelle - panel.grip_height).max(vu.h);
    let bord = match panel.tab.cote() {
        super::Cote::Gauche => vu.x + vu.w,
        super::Cote::Droite => vu.x,
    };
    let part = WidgetRect::new(
        bord - brush.px(0.5),
        vu.y + vu.h * panel.defile / tout,
        brush.px(1.0),
        vu.h * vu.h / tout,
    );
    brush.fill(pixmap, part, 0.0, brush.theme.border_accent);
}

/// Le fond, la bordure et la poignée — ce que tout panneau a en commun. **Plat** (TIROIR-4) :
/// ni ombre ni coin arrondi, un filet d'un pixel — la charte (`style.md` : « pas d'ombre portée
/// molle », « coins peu ou pas arrondis sur la chrome ») ; ses « bulles » du 17/11 étaient
/// l'ombre de huit pixels et les coins de six. Un tiroir tiré se dit par son filet d'accent.
fn draw_frame(pixmap: &mut PixmapMut, brush: &Brush, panel: &PanelLayoutBox, seen: WidgetRect) {
    let theme = brush.theme;
    brush.fill(pixmap, seen, 0.0, theme.bg_panel);
    let border = if panel.is_dragged {
        theme.border_accent
    } else {
        theme.border_subtle
    };
    brush.stroke(pixmap, seen, 0.0, border, brush.px(1.0));

    let grip_center = (
        seen.x + seen.w / 2.0,
        panel.grip_y + panel.visual_offset_y + panel.grip_height / 2.0,
    );
    let active = panel.is_dragged || brush.hovered(panel.grip_rect());
    brush.grip(pixmap, grip_center, active);
}

fn draw_content(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    dock: &DockManager,
    store: &Store,
    tab: TabId,
    frame: ScaledRect,
) {
    match tab {
        TabId::Organize => {
            organize::paint::render_organize_panel(pixmap, brush, frame, dock, store);
        }
        TabId::Pomodoro => {
            pomodoro::paint::render_pomodoro_panel(pixmap, brush, frame, &dock.pomodoro);
        }
        TabId::Storyboard => {
            storyboard::paint::render_storyboard_panel(pixmap, brush, frame, dock);
        }
        TabId::Plugins => {
            plugins::paint::render_plugins_panel(pixmap, brush, frame, &dock.plugins);
        }
        TabId::Preset => preset::paint::render_presets_panel(pixmap, brush, frame, store),
        TabId::Domains => {
            domains::paint::render_domains_panel(pixmap, store, &dock.domains, brush, frame)
        }
        TabId::Temps => super::temps::paint::render_temps_panel(pixmap, brush, frame, &dock.temps),
        TabId::Discussion => super::discussion::paint::render_discussion_panel(
            pixmap,
            brush,
            frame,
            &dock.discussion,
        ),
    }
}

// ── Le clic ─────────────────────────────────────────────────────────────────
