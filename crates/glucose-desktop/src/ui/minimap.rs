//! La minimap : la carte du document en bas à droite, et ce qu'on y clique.
//!
//! # Deux dessins, et un seul est cher
//!
//! Le **fond** — le cadre, les nœuds, les dossiers — ne dépend que du document et du cadrage
//! de la carte : il est gardé d'une image à l'autre ([`MinimapCache`]). Le **rectangle de
//! caméra**, lui, bouge à chaque déplacement, mais c'est un rectangle, pas un par nœud.
//!
//! Avant ce partage, la minimap dessinait un rectangle par nœud à chaque image : 5,1 ms sur
//! les 9,4 d'une image à dix mille nœuds, pour redessiner à l'identique ce qui était déjà là.

use super::TOTAL_HEADER_HEIGHT;
use crate::theme::Theme;
use glucose_core::store::Store;
use glucose_core::types::Annotation;
use tiny_skia::{Color, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Le fond de la minimap, gardé d'une image à l'autre.
///
/// # Pourquoi
///
/// La minimap dessinait **un rectangle par nœud du tableau, à chaque image**, dans une vignette
/// de 180 × 120 pixels où la plupart tombent les uns sur les autres. Le banc a chiffré ce que
/// cela coûte : **5,1 ms sur les 9,4 d'une image** à dix mille nœuds, soit plus de la moitié du
/// budget, pour redessiner à l'identique ce qui était déjà là.
///
/// Or ce fond ne dépend que de deux choses : le document, et le cadrage de la carte. Le
/// rectangle de caméra, lui, bouge à chaque déplacement — mais c'est **un** rectangle, et il se
/// dessine par-dessus.
///
/// # Pourquoi la clé tient malgré le pan
///
/// Le cadrage de la minimap englobe le contenu **et** la caméra : c'est ce qui permet de voir
/// où l'on est quand on s'éloigne du document. Tant qu'on travaille à l'intérieur du contenu —
/// le cas normal — la caméra ne change rien aux bornes, et la clé reste identique d'une image à
/// l'autre. Ce n'est qu'en sortant du document que le cadrage bouge, et le fond se refait alors
/// le temps du déplacement.
///
/// # Ce que la composition coûte vraiment, et qui n'était pas ce que je croyais
///
/// « Source par-dessus » est associatif **en réels** : composer le fond puis la scène donne le
/// même résultat que tout composer d'un coup. Il ne l'est pas **en entiers de huit bits**, où
/// chaque étape arrondit. Passer par un pixmap intermédiaire ajoute donc un arrondi, et l'écart
/// atteint un niveau de quantification sur les pixels semi-transparents.
///
/// L'empreinte de la scène témoin a changé pour cette raison, et la capture a été regardée :
/// la minimap y est identique à l'œil. Un niveau sur 255 est en dessous de ce qu'un écran
/// distingue — mais il fallait le mesurer, pas le supposer, et surtout pas écrire « au bit
/// près » comme je l'avais fait.
pub struct MinimapCache {
    pixmap: tiny_skia::Pixmap,
    key: MinimapKey,
}

/// Ce qui, s'il change, oblige à refaire le fond de la minimap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MinimapKey {
    /// La version du document : toute modification la fait avancer.
    version: u64,
    /// La taille de la vignette en pixels entiers — elle change avec l'échelle de l'interface.
    size: (u32, u32),
    /// Les bornes du cadrage, en bits : deux `f64` égaux ont les mêmes bits, et c'est la seule
    /// comparaison qui ait un sens ici (on ne veut pas d'un seuil arbitraire).
    bounds: [u64; 4],
}

/// Le mot d'accueil, posé par l'application au démarrage — pas par le constructeur.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MinimapBounds {
    pub mm_x: f32,
    pub mm_y: f32,
    pub mm_w: f32,
    pub mm_h: f32,
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub span_x: f64,
    pub span_y: f64,
    pub scale: f32,
    pub cam_left: f64,
    pub cam_top: f64,
    pub vp_w: f64,
    pub vp_h: f64,
}

pub fn layout_minimap(
    store: &Store,
    screen_w: f32,
    screen_h: f32,
    scale: f32,
) -> Option<MinimapBounds> {
    let board = store.active_board()?;
    let s = crate::theme::clamp_ui_scale(scale);
    let mm_w = 180.0f32 * s;
    let mm_h = 120.0f32 * s;
    // Fiche 06 § 9 : `bottom: 12px`, `right: 12px`.
    let mm_x = screen_w - mm_w - 12.0 * s;
    let mm_y = screen_h - mm_h - 12.0 * s;
    let header_h = TOTAL_HEADER_HEIGHT * s;

    // Les bornes du contenu sont une question qu'on pose au document ; la minimap n'a pas à
    // les recalculer avec ses propres tailles. Un tableau vide n'en a pas : la caméra seule
    // fait alors la carte.
    let contenu = store.content_bounds(&board.id);
    let (mut min_x, mut min_y, mut max_x, mut max_y) = match contenu {
        Some(r) => (r.left, r.top, r.right(), r.bottom()),
        None => (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ),
    };

    let vp = &board.viewport;
    let vp_w = screen_w as f64 / vp.scale;
    let vp_h = (screen_h as f64 - header_h as f64) / vp.scale;
    let cam_left = -vp.x / vp.scale;
    let cam_top = -vp.y / vp.scale;

    min_x = min_x.min(cam_left) - 200.0;
    min_y = min_y.min(cam_top) - 200.0;
    max_x = max_x.max(cam_left + vp_w) + 200.0;
    max_y = max_y.max(cam_top + vp_h) + 200.0;

    // Garde-fou : un viewport corrompu (échelle nulle, NaN) ou un board vide
    // laisserait des bornes infinies, puis des coordonnées NaN transmises au
    // rasterizer. On renonce alors à la minimap plutôt que de dessiner du bruit.
    let finite = [min_x, min_y, max_x, max_y, cam_left, cam_top, vp_w, vp_h];
    if finite.iter().any(|v| !v.is_finite()) {
        return None;
    }

    let span_x = (max_x - min_x).max(1.0);
    let span_y = (max_y - min_y).max(1.0);

    let scale_x = (mm_w - 12.0 * s) / span_x as f32;
    let scale_y = (mm_h - 12.0 * s) / span_y as f32;
    let minimap_scale = scale_x.min(scale_y);

    Some(MinimapBounds {
        mm_x,
        mm_y,
        mm_w,
        mm_h,
        min_x,
        min_y,
        max_x,
        max_y,
        span_x,
        span_y,
        scale: minimap_scale,
        cam_left,
        cam_top,
        vp_w,
        vp_h,
    })
}

pub fn render_minimap(
    pixmap: &mut PixmapMut,
    store: &Store,
    theme: &Theme,
    w: f32,
    h: f32,
    scale: f32,
    cache: &mut Option<MinimapCache>,
) {
    let s = crate::theme::clamp_ui_scale(scale);
    let mb = match layout_minimap(store, w, h, s) {
        Some(m) => m,
        None => return,
    };
    if store.active_board().is_none() {
        return;
    }

    let cle = MinimapKey {
        version: store.version,
        size: (mb.mm_w.ceil() as u32, mb.mm_h.ceil() as u32),
        bounds: [
            mb.min_x.to_bits(),
            mb.min_y.to_bits(),
            mb.max_x.to_bits(),
            mb.max_y.to_bits(),
        ],
    };
    if cache.as_ref().map(|c| c.key) != Some(cle) {
        *cache = dessine_fond(store, theme, &mb, s).map(|pixmap| MinimapCache { pixmap, key: cle });
    }
    if let Some(c) = cache.as_ref() {
        // Par REPORT-1, comme les panneaux du dock : le fond de la minimap est un tampon posé
        // à une position entière, exactement le cas où le chemin exact s'applique.
        crate::composition::poser(
            pixmap,
            &c.pixmap,
            (mb.mm_x, mb.mm_y),
            glucose_core::report::Melange::Composer,
        );
    }

    dessine_camera(pixmap, theme, &mb, s);
}

/// Dessine le fond de la minimap — tout sauf le rectangle de caméra — dans un pixmap à part.
///
/// Les coordonnées y sont relatives au coin de la vignette, puisqu'elle sera composée à sa
/// place : c'est la seule différence avec le dessin direct d'avant.
/// Peint un rectangle **opaque** en écrivant les pixels, sans construire de chemin.
///
/// # Pourquoi cette fonction existe
///
/// La minimap dessinait un `fill_rect` par nœud. Sur un document d'un million, cela fait un
/// million de chemins construits, alloués et rastérisés — pour une vignette de 180 × 120,
/// soit **trente-cinq rectangles par pixel**. Mesuré : 1 700 ms à chaque mutation du
/// document, le premier poste de l'application.
///
/// Un nœud y occupe deux à quatre pixels. Construire un chemin pour cela est hors de
/// proportion : le rectangle se réduit à quelques écritures directes.
///
/// # Ce que cela change à l'image, et pourquoi c'est admis
///
/// L'anti-crénelage disparaît sur ces rectangles : un pixel est peint si son **centre** tombe
/// dans le rectangle, sans demi-teinte sur les bords. C'est la règle que `tiny-skia` applique
/// lui-même quand on lui désactive le lissage, et sur des marques de deux pixels le résultat
/// est plus net plutôt que moins fidèle (R-46). Le cadre de la minimap et les dossiers, eux,
/// gardent leur tracé : ce sont des traits fins, où le lissage compte vraiment.
///
/// La couleur doit être opaque — c'est le cas de toutes celles qui passent ici — sans quoi il
/// faudrait composer au lieu d'écrire.
fn remplir_net(pixmap: &mut tiny_skia::PixmapMut, x: f32, y: f32, w: f32, h: f32, color: Color) {
    let (largeur, hauteur) = (pixmap.width() as i32, pixmap.height() as i32);
    // Un pixel est peint quand son centre — en `i + 0,5` — tombe dans le rectangle.
    let borne = |de: f32, a: f32, max: i32| {
        let d = (de - 0.5).ceil().max(0.0) as i32;
        let f = (a - 0.5).ceil().clamp(0.0, max as f32) as i32;
        (d.min(max), f)
    };
    let (x0, x1) = borne(x, x + w, largeur);
    let (y0, y1) = borne(y, y + h, hauteur);
    if x0 >= x1 || y0 >= y1 {
        return;
    }
    let pixel = tiny_skia::PremultipliedColorU8::from_rgba(
        (color.red() * 255.0).round() as u8,
        (color.green() * 255.0).round() as u8,
        (color.blue() * 255.0).round() as u8,
        255,
    )
    .unwrap_or_else(|| tiny_skia::PremultipliedColorU8::from_rgba(0, 0, 0, 255).expect("noir"));
    let pixels = pixmap.pixels_mut();
    for ligne in y0..y1 {
        let debut = (ligne * largeur + x0) as usize;
        let fin = (ligne * largeur + x1) as usize;
        pixels[debut..fin].fill(pixel);
    }
}

/// Le cadre de la minimap : son fond plein et sa bordure.
fn dessine_le_cadre(pixmap: &mut PixmapMut, theme: &Theme, mb: &MinimapBounds, s: f32) {
    // Le fond est un rectangle aligné sur les axes : l'anti-aliasing ne lui apporte rien et
    // lui coûte deux demi-teintes sur chaque bord (SCALE-3).
    if let Some(rect) = Rect::from_xywh(mb.mm_x, mb.mm_y, mb.mm_w, mb.mm_h) {
        crate::renderer::scale::fill_crisp(pixmap, rect, theme.minimap_bg);
    }
    let mut border_paint = Paint::default();
    border_paint.set_color(theme.minimap_border);
    let stroke = Stroke {
        width: 1.0 * s,
        ..Default::default()
    };
    let mut pb = PathBuilder::new();
    pb.move_to(mb.mm_x, mb.mm_y);
    pb.line_to(mb.mm_x + mb.mm_w, mb.mm_y);
    pb.line_to(mb.mm_x + mb.mm_w, mb.mm_y + mb.mm_h);
    pb.line_to(mb.mm_x, mb.mm_y + mb.mm_h);
    pb.close();
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &border_paint, &stroke, Transform::identity(), None);
    }
}

/// Où une boîte du monde tombe dans la vignette, et de quelle taille — jamais sous
/// `plancher`, sans quoi un nœud disparaîtrait de la carte.
fn place_dans_la_carte(
    mb: &MinimapBounds,
    boite: glucose_core::geometry::Rect,
    pad: f32,
    plancher: f32,
) -> (f32, f32, f32, f32) {
    (
        mb.mm_x + pad + ((boite.left - mb.min_x) as f32 * mb.scale),
        mb.mm_y + pad + ((boite.top - mb.min_y) as f32 * mb.scale),
        (boite.width as f32 * mb.scale).max(plancher),
        (boite.height as f32 * mb.scale).max(plancher),
    )
}

/// Les photos et les annotations : des marques de quelques pixels, écrites sans chemin.
fn dessine_le_contenu(
    pixmap: &mut PixmapMut,
    board: &glucose_core::types::Board,
    theme: &Theme,
    mb: &MinimapBounds,
    (pad, s): (f32, f32),
) {
    for img in &board.images {
        let (x, y, w, h) = place_dans_la_carte(mb, img.rect(), pad, 2.0 * s);
        remplir_net(pixmap, x, y, w, h, theme.minimap_element);
    }
    // Une flèche n'a pas de boîte et ne se dessine pas ici ; les autres nœuds ont la leur,
    // taille de naissance comprise, et un plancher de quelques pixels pour rester visibles.
    // Une membrane est du contenu, mais sa couleur propre n'est pas lue ici : dans la
    // minimap elle se dessine comme les autres nœuds, en gris.
    for ann in &board.annotations {
        let Some(boite) = ann.rect() else {
            continue;
        };
        let (couleur, plancher) = match ann {
            Annotation::Membrane { .. } => (theme.minimap_element, 4.0 * s),
            _ => (theme.text_muted, 2.0 * s),
        };
        let (x, y, w, h) = place_dans_la_carte(mb, boite, pad, plancher);
        remplir_net(pixmap, x, y, w, h, couleur);
    }
}

/// Les dossiers — fiche 06 § 9 : en pointillés, à LEUR couleur, et non en gris comme le reste
/// du contenu. C'est ce qui les distingue d'une membrane à l'œil, sur une carte de 180 px.
fn dessine_les_dossiers(
    pixmap: &mut PixmapMut,
    board: &glucose_core::types::Board,
    mb: &MinimapBounds,
    (pad, s): (f32, f32),
) {
    let folder_stroke = Stroke {
        width: 1.0 * s,
        dash: tiny_skia::StrokeDash::new(vec![2.0 * s, 2.0 * s], 0.0),
        ..Default::default()
    };
    for f in &board.folders {
        let boite = glucose_core::geometry::Rect::new(f.x, f.y, f.width, f.height);
        let (fx, fy, fw, fh) = place_dans_la_carte(mb, boite, pad, 4.0 * s);
        let (r, g, b) = crate::renderer::parse_hex_color(&f.color, 136, 136, 136);
        let mut folder_paint = Paint {
            anti_alias: true,
            ..Default::default()
        };
        folder_paint.set_color(Color::from_rgba8(r, g, b, 179));
        let mut fpb = PathBuilder::new();
        fpb.move_to(fx, fy);
        fpb.line_to(fx + fw, fy);
        fpb.line_to(fx + fw, fy + fh);
        fpb.line_to(fx, fy + fh);
        fpb.close();
        if let Some(path) = fpb.finish() {
            pixmap.stroke_path(
                &path,
                &folder_paint,
                &folder_stroke,
                Transform::identity(),
                None,
            );
        }
    }
}

fn dessine_fond(
    store: &Store,
    theme: &Theme,
    mb: &MinimapBounds,
    s: f32,
) -> Option<tiny_skia::Pixmap> {
    let board = store.active_board()?;
    let mut vignette = tiny_skia::Pixmap::new(mb.mm_w.ceil() as u32, mb.mm_h.ceil() as u32)?;
    let pixmap = &mut vignette.as_mut();
    // Le fond se dessine à l'origine de sa propre vignette.
    let mb = &MinimapBounds {
        mm_x: 0.0,
        mm_y: 0.0,
        ..mb.clone()
    };
    let pad = 6.0 * s;
    dessine_le_cadre(pixmap, theme, mb, s);
    dessine_le_contenu(pixmap, board, theme, mb, (pad, s));
    dessine_les_dossiers(pixmap, board, mb, (pad, s));
    Some(vignette)
}

/// Dessine le rectangle de caméra. Il bouge à chaque déplacement, donc il n'est jamais mis en
/// cache — mais c'est **un** rectangle, pas un par nœud.
fn dessine_camera(pixmap: &mut PixmapMut, theme: &Theme, mb: &MinimapBounds, s: f32) {
    let pad = 6.0 * s;
    let cx = mb.mm_x + pad + ((mb.cam_left - mb.min_x) as f32 * mb.scale);
    let cy = mb.mm_y + pad + ((mb.cam_top - mb.min_y) as f32 * mb.scale);
    let cw = (mb.vp_w as f32 * mb.scale).max(4.0 * s);
    let ch = (mb.vp_h as f32 * mb.scale).max(4.0 * s);

    let mut cam_paint = Paint::default();
    cam_paint.set_color(theme.minimap_viewport);
    let cam_stroke = Stroke {
        width: 1.5 * s,
        ..Default::default()
    };
    let mut cam_pb = PathBuilder::new();
    cam_pb.move_to(cx, cy);
    cam_pb.line_to(cx + cw, cy);
    cam_pb.line_to(cx + cw, cy + ch);
    cam_pb.line_to(cx, cy + ch);
    cam_pb.close();
    if let Some(path) = cam_pb.finish() {
        pixmap.stroke_path(&path, &cam_paint, &cam_stroke, Transform::identity(), None);
    }
}

/// Le point du monde que la minimap désigne sous `(x, y)`, ou rien si le curseur est ailleurs.
///
/// Extraite du traitement du clic parce qu'elle sert **deux fois** : à l'appui, et à chaque
/// mouvement tant que le bouton tient. Sans cela, suivre le curseur sur la minimap aurait
/// demandé de recopier la conversion — donc deux formules à tenir d'accord, et une minimap
/// qui viserait à côté le jour où l'une des deux bougerait.
pub fn point_minimap(
    store: &Store,
    x: f32,
    y: f32,
    screen_w: f32,
    screen_h: f32,
    s: f32,
) -> Option<(f64, f64)> {
    let mb = layout_minimap(store, screen_w, screen_h, s)?;
    let pad = 6.0 * s;
    if x < mb.mm_x || x > mb.mm_x + mb.mm_w || y < mb.mm_y || y > mb.mm_y + mb.mm_h {
        return None;
    }
    let rel_x = ((x - mb.mm_x - pad) / (mb.mm_w - 2.0 * pad).max(1.0)).clamp(0.0, 1.0) as f64;
    let rel_y = ((y - mb.mm_y - pad) / (mb.mm_h - 2.0 * pad).max(1.0)).clamp(0.0, 1.0) as f64;
    Some((mb.min_x + rel_x * mb.span_x, mb.min_y + rel_y * mb.span_y))
}
