//! Ce qu'une image porte **en plus** de ses pixels : son cadre de sélection, ses poignées, et
//! le carré qui la remplace tant que ses octets ne sont pas là.
//!
//! # Pourquoi c'est à part, et pourquoi cela compte pour les tuiles
//!
//! Aucun de ces ornements n'appartient au **document**. La sélection est un état de
//! l'interface ; l'absence d'octets est un état du magasin. Une tuile, elle, ne connaît que
//! le document — c'est ce qui la rend réutilisable. Y peindre un cadre de sélection ferait
//! qu'une image resterait cerclée après avoir été désélectionnée, tant que sa tuile survit.
//!
//! Les ornements se dessinent donc **par-dessus** les tuiles, à chaque image, en direct.

use super::super::super::domain::draw_domain_gauge;
use super::super::super::handles::draw_rotated_handles;
use super::super::super::scale::fill_crisp;
use super::super::super::scale::WorldScale;
use super::super::super::PaintKit;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::resize::Handle;
use glucose_core::store::Store;
use tiny_skia::{BlendMode, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Comment reporter une image sur le canevas : en remplaçant, ou en composant.
///
/// Le remplacement est plus rapide, mais il écrit **tous** les pixels de la zone couverte. Il
/// n'est donc licite qu'à deux conditions réunies :
///
/// * l'image est opaque — sinon le fond devrait transparaître à travers elle ;
/// * elle n'est pas tournée — sinon la zone couverte est un parallélogramme, et les coins du
///   rectangle qui l'entoure seraient remplacés par du vide, laissant quatre trous.
///
/// Les deux se constatent, aucune ne s'estime.
pub(super) fn mode_de_report(opaque: bool, rotation: f64) -> BlendMode {
    if opaque && rotation == 0.0 {
        BlendMode::Source
    } else {
        BlendMode::SourceOver
    }
}

/// Une image dont les octets ne sont pas (encore) là : un cadre gris de la chrome, son
/// identifiant dedans. Monochrome — ce n'est pas du contenu, c'est son absence.
pub(in crate::renderer) fn draw_missing_image(
    typography: &Typography,
    theme: &Theme,
    pixmap: &mut PixmapMut,
    at: (f32, f32),
    size: (f32, f32),
    id: &str,
    rotation: f64,
) {
    let Some(rect) = Rect::from_xywh(at.0, at.1, size.0, size.1) else {
        return;
    };
    // Le carré de remplacement tourne comme tournerait la texture : sans cela, une image
    // introuvable et penchée se dessinerait droite dans un cadre incliné.
    let ts = rotation_at(rotation, at, size);
    let path = PathBuilder::from_rect(rect);
    let mut fill = Paint {
        anti_alias: true,
        ..Default::default()
    };
    fill.set_color(theme.bg_hover);
    pixmap.fill_path(&path, &fill, tiny_skia::FillRule::Winding, ts, None);

    let mut border = Paint {
        anti_alias: true,
        ..Default::default()
    };
    border.set_color(theme.border_accent);
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &border, &stroke, ts, None);

    typography.draw_text(
        pixmap,
        &format!("Image [{id}]"),
        at.0 + 10.0,
        at.1 + size.1 / 2.0 - 6.0,
        TextStyle {
            size: 12.0,
            color: theme.text_muted,
            face: Face::Regular,
        },
    );
}

/// La transformation qui fait tourner une boîte écran autour de son propre centre.
///
/// Un seul endroit où l'angle devient une matrice : la texture, son carré de remplacement et
/// son cadre de sélection tournent donc exactement pareil, par construction.
fn rotation_at(rotation: f64, at: (f32, f32), size: (f32, f32)) -> Transform {
    if rotation == 0.0 {
        return Transform::identity();
    }
    Transform::from_rotate_at(
        rotation.to_degrees() as f32,
        at.0 + size.0 / 2.0,
        at.1 + size.1 / 2.0,
    )
}

/// Débord du cadre de sélection autour de la texture, en pixels écran (fiche 06 § 4.2).
const IMAGE_SELECTION_INSET: f32 = 3.0;
/// Épaisseur du cadre de sélection, en pixels écran (fiche 06 § 4.2).
const IMAGE_SELECTION_STROKE: f32 = 1.25;

/// **Les cadres de sélection des images** — fiche 06 § 4.2 : un liseré blanc pur à 0,80,
/// débordant de 3 px, épais de 1,25 px à l'écran quel que soit le zoom ; celui d'une image
/// verrouillée prend l'encre de l'alerte. Aucun néon : le contour se lit sur une image claire
/// par le liseré des poignées, sur le fond noir par le blanc.
///
/// # Un cadre droit est quatre filets, et un filet ne s'anti-crénèle pas
///
/// Chaque cadre passait par le contour anti-crénelé du rastériseur : 40 µs pièce, presque tout
/// de mise en place, 9,9 ms pour 243 photos sélectionnées (`bench_ornements`). Réunis en un
/// seul tracé, ils coûtaient encore cinq à sept millisecondes : un contour qui couvre l'écran se
/// balaie ligne par ligne, quatre sous-lignes par pixel. Or un cadre **droit** n'a aucun bord
/// oblique — c'est exactement le cas de SCALE-3, où l'anti-crénelage n'apporte rien et coûte
/// tout. Il se pose donc en quatre filets sur la grille, de l'épaisseur entière la plus proche
/// de 1,25 px : un pixel, net.
///
/// Un cadre **penché** garde le contour : ses bords sont obliques. Il entre dans un seul tracé
/// par ses quatre coins tournés — une rotation ne déforme rien, donc tourner le rectangle puis
/// le border revient à border le rectangle puis le tourner.
#[derive(Default)]
pub(super) struct Cadres {
    /// Les cadres droits, en rectangles écran, et s'ils sont verrouillés.
    droits: Vec<(Rect, bool)>,
    /// Les cadres penchés, un tracé par encre : normaux, puis verrouillés.
    penches: [PathBuilder; 2],
}

impl Cadres {
    /// Ajoute le cadre d'une image posée sur la boîte écran `(sx, sy, sw, sh)`.
    pub(super) fn ajouter(
        &mut self,
        img: &glucose_core::types::BoardImage,
        (sx, sy, sw, sh): (f32, f32, f32, f32),
    ) {
        let d = IMAGE_SELECTION_INSET;
        let (at, size) = ((sx - d, sy - d), (sw + 2.0 * d, sh + 2.0 * d));
        if img.rotation == 0.0 {
            if let Some(rect) = Rect::from_xywh(at.0, at.1, size.0, size.1) {
                self.droits.push((rect, img.locked));
            }
            return;
        }
        // Le cadre épouse le nœud : il tourne avec lui, autour du même centre.
        let mut coins = [
            tiny_skia::Point::from_xy(at.0, at.1),
            tiny_skia::Point::from_xy(at.0 + size.0, at.1),
            tiny_skia::Point::from_xy(at.0 + size.0, at.1 + size.1),
            tiny_skia::Point::from_xy(at.0, at.1 + size.1),
        ];
        rotation_at(img.rotation, at, size).map_points(&mut coins);
        let trace = &mut self.penches[usize::from(img.locked)];
        trace.move_to(coins[0].x, coins[0].y);
        for c in &coins[1..] {
            trace.line_to(c.x, c.y);
        }
        trace.close();
    }

    /// Pose tous les cadres ajoutés : les droits en filets, les penchés en un tracé par encre.
    pub(super) fn poser(self, pixmap: &mut PixmapMut, theme: &Theme) {
        let encres = [theme.selection_frame, theme.alert];
        for (rect, verrouille) in self.droits {
            poser_un_cadre_droit(pixmap, rect, encres[usize::from(verrouille)]);
        }
        let stroke = Stroke {
            width: IMAGE_SELECTION_STROKE,
            ..Default::default()
        };
        for (trace, encre) in self.penches.into_iter().zip(encres) {
            let Some(chemin) = trace.finish() else {
                continue;
            };
            let mut paint = Paint {
                anti_alias: true,
                ..Default::default()
            };
            paint.set_color(encre);
            pixmap.stroke_path(&chemin, &paint, &stroke, Transform::identity(), None);
        }
    }
}

/// **Un cadre droit en quatre filets sur la grille**, centrés sur le bord du rectangle comme
/// l'était le contour : le liseré déborde d'une demi-épaisseur au-dehors et mord d'autant
/// au-dedans. Les filets ne se chevauchent pas — une encre à 0,80 posée deux fois aux coins
/// y serait plus claire.
fn poser_un_cadre_droit(pixmap: &mut PixmapMut, rect: Rect, encre: tiny_skia::Color) {
    let e = IMAGE_SELECTION_STROKE.round().max(1.0);
    let (x, y) = ((rect.x() - e / 2.0).round(), (rect.y() - e / 2.0).round());
    let (l, h) = ((rect.width() + e).round(), (rect.height() + e).round());
    if l <= 2.0 * e || h <= 2.0 * e {
        return;
    }
    for (fx, fy, fl, fh) in [
        (x, y, l, e),
        (x, y + h - e, l, e),
        (x, y + e, e, h - 2.0 * e),
        (x + l - e, y + e, e, h - 2.0 * e),
    ] {
        if let Some(filet) = Rect::from_xywh(fx, fy, fl, fh) {
            fill_crisp(pixmap, filet, encre);
        }
    }
}

/// **Une image visible, et sa boite a l'ecran** : `(x, y, largeur, hauteur)` en pixels.
pub(in crate::renderer) type Posee<'a> =
    (&'a glucose_core::types::BoardImage, (f32, f32, f32, f32));

/// Ce que les images portent en plus de leurs pixels, et qui n'appartient pas au document :
/// les cadres de selection, les poignees, les jauges de domaines -- pour toutes les images
/// visibles d'un coup, chacune avec sa boite a l'ecran.
///
/// Appelable a part de la pose, parce que les tuiles ne les contiennent pas (voir
/// [`PasseImages::en_tuile`]) : quand l'ecran se compose depuis la grille, ils se dessinent
/// ensuite, en direct, par-dessus.
///
/// # Toutes ensemble, et dans cet ordre
///
/// Les cadres d'abord, en un trace ; les poignees ensuite, qui se posent SUR le cadre de leur
/// image ; les jauges enfin. Image par image, les dix-sept appels d'une photo selectionnee
/// coutaient 21,6 ms pour 243 photos (`bench_ornements`) -- le poste qui faisait tomber le
/// tempo a 48 images par seconde sur la longue session du 23/09.
pub(in crate::renderer) fn draw_image_ornaments(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    scale: WorldScale,
    images: &[Posee<'_>],
) {
    // La selection se lit une fois : la chercher dans une liste pour chaque image faisait
    // autant de comparaisons que le produit des deux.
    let choisies: std::collections::HashSet<&str> = store
        .selected_image_ids
        .iter()
        .map(String::as_str)
        .collect();
    let selectionnees = || {
        images
            .iter()
            .filter(|(img, _)| choisies.contains(img.id.as_str()))
    };
    let mut cadres = Cadres::default();
    for (img, ecran) in selectionnees() {
        cadres.ajouter(img, *ecran);
    }
    cadres.poser(pixmap, kit.theme);
    // Une image verrouillee se signale par la couleur de son cadre et par l'absence de ses
    // poignees (fiche 06 § 4.3) : les deux disent le meme fait, l'un de loin, l'autre au
    // moment ou la main cherche une prise.
    for (img, ecran) in selectionnees().filter(|(img, _)| !img.locked) {
        draw_rotated_handles(pixmap, kit.theme, scale, *ecran, &Handle::ALL, img.rotation);
    }
    for (img, (sx, sy, _, _)) in images {
        draw_domain_gauge(
            kit.typography,
            kit.tints,
            pixmap,
            scale,
            (*sx, *sy),
            &img.domains,
        );
    }
}
