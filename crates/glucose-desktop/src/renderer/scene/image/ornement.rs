//! Ce qu'une image porte **en plus** de ses pixels : ses poignées, sa jauge, et le carré qui la
//! remplace tant que ses octets ne sont pas là. (Son cadre de sélection est devenu les équerres
//! de tous les nœuds, `renderer/equerres.rs`, fiche 137.)
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
///   rectangle qui l'entoure seraient remplacés par du vide, laissant quatre trous ;
/// * elle est pleine — estompée par la frise chronologique (ESTOMPE-1), elle laisse voir ce
///   qu'elle couvre.
///
/// Les trois se constatent, aucune ne s'estime.
pub(super) fn mode_de_report(opaque: bool, rotation: f64, presence: f32) -> BlendMode {
    if opaque && rotation == 0.0 && presence >= 1.0 {
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

    let (texte, (x, y), style) = libelle(theme, at, size, id);
    typography.draw_text(pixmap, &texte, x, y, style);
}

/// **Le libellé d'une photo en chemin** : son texte, son coin, son style — là où
/// [`draw_missing_image`] l'écrit, et le dessin vectoriel avec lui.
fn libelle(
    theme: &Theme,
    at: (f32, f32),
    size: (f32, f32),
    id: &str,
) -> (String, (f32, f32), TextStyle) {
    (
        format!("Image [{id}]"),
        (at.0 + 10.0, at.1 + size.1 / 2.0 - 6.0),
        TextStyle {
            size: 12.0,
            color: theme.text_muted,
            face: Face::Regular,
        },
    )
}

/// **Le cadre d'une photo en chemin, en formes** (fiche 73) : celui de [`draw_missing_image`] —
/// le même rectangle tourné, son trait d'un pixel élargi par le même traceur, son libellé —, que
/// la carte dessine de près au lieu d'en agrandir l'image.
pub(in crate::renderer) fn cadre_manquant_en_vecteurs(
    kit: PaintKit<'_>,
    v: &mut crate::renderer::vectoriel::Releve,
    (at, size): ((f32, f32), (f32, f32)),
    (id, rotation): (&str, f64),
) {
    use crate::renderer::vectoriel::Vectoriel;
    let Some(rect) = Rect::from_xywh(at.0, at.1, size.0, size.1) else {
        return;
    };
    let ts = rotation_at(rotation, at, size);
    let path = PathBuilder::from_rect(rect);
    if let Some(plein) = path.clone().transform(ts) {
        v.chemin(&plein, kit.theme.bg_hover);
    }
    let stroke = Stroke {
        width: 1.0,
        ..Default::default()
    };
    if let Some(trait_) = path.stroke(&stroke, 1.0).and_then(|t| t.transform(ts)) {
        v.chemin(&trait_, kit.theme.border_accent);
    }
    let (texte, at, style) = libelle(kit.theme, at, size, id);
    v.texte(kit.typography, at, &texte, style);
}

/// La transformation qui fait tourner une boîte écran autour de son propre centre.
///
/// Un seul endroit où l'angle devient une matrice : la texture, son carré de remplacement et
/// ses poignées tournent donc exactement pareil, par construction.
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

/// **Une image visible, et sa boite a l'ecran** : `(x, y, largeur, hauteur)` en pixels.
pub(in crate::renderer) type Posee<'a> =
    (&'a glucose_core::types::BoardImage, (f32, f32, f32, f32));

/// Ce que les images portent en plus de leurs pixels, et qui n'appartient pas au document :
/// les poignees, les jauges de domaines -- pour toutes les images visibles d'un coup, chacune
/// avec sa boite a l'ecran.
///
/// Appelable a part de la pose, parce que les tuiles ne les contiennent pas (voir
/// [`PasseImages::en_tuile`]) : quand l'ecran se compose depuis la grille, ils se dessinent
/// ensuite, en direct, par-dessus.
///
/// # Toutes ensemble, et dans cet ordre
///
/// Les poignees d'abord, les jauges ensuite (les cadres de selection, devenus les equerres, se
/// posent avec ceux de tous les noeuds). Image par image, les dix-sept appels d'une photo selectionnee
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
    // Une image verrouillee se signale par l'encre de ses equerres et par l'absence de ses
    // poignees (fiche 06 § 4.3) : les deux disent le meme fait, l'un de loin, l'autre au
    // moment ou la main cherche une prise. Ses equerres se posent avec celles des autres
    // noeuds, au-dessus de tout (Q12, fiche 137).
    // En groupe, les poignées sont celles du groupe (fiche 53 § 10).
    let seules = store
        .emprise_du_groupe(&store.project.active_board_id)
        .is_none();
    for (img, ecran) in selectionnees().filter(|(img, _)| !img.locked && seules) {
        draw_rotated_handles(pixmap, kit.theme, scale, *ecran, &Handle::ALL, img.rotation);
    }
    for (img, (sx, sy, _, _)) in images {
        draw_domain_gauge(
            kit.typography,
            kit.tints,
            pixmap,
            scale,
            ((*sx, *sy), store.presence(img.regard())),
            &img.domains,
        );
    }
}
