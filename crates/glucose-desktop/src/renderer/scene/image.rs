//! Poser une image sur le canevas — et ce que cela coûte selon la façon de la poser.
//!
//! Le sujet a grandi jusqu'à mériter son module : une image ne se dessine pas, elle se
//! **reporte**, depuis un niveau de réduction (MIP-1) ou depuis une vignette déjà à la forme
//! voulue (MIP-2), en composant ou en remplaçant selon ce que son opacité autorise.

use super::super::domain::draw_domain_gauge;
use super::super::handles::draw_rotated_handles;
use super::super::magasin::Magasin;
use super::super::pass::Clip;
use super::super::scale::WorldScale;
use super::super::{photo, vignette, PaintKit};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::occlusion::{self, Calque};
use glucose_core::quadtree::Visibles;
use glucose_core::resize::Handle;
use glucose_core::store::Store;
use tiny_skia::{
    BlendMode, Color, FilterQuality, Paint, PathBuilder, PixmapMut, PixmapPaint, Rect, Stroke,
    Transform,
};

pub(in crate::renderer) fn draw_images(
    magasin: &mut Magasin,
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let PaintKit {
        typography,
        tints,
        theme,
        ..
    } = kit;
    let scale = WorldScale::new(pass.vp.scale);
    let clip = Clip {
        width: pixmap.width() as f32,
        height: pixmap.height() as f32,
        top: pass.header_h,
    };

    // Combien d'images sont réellement posées, et quelle surface d'écran elles couvrent au
    // total. Le rapport des deux à la surface de la fenêtre dit tout de suite si le coût vient
    // du nombre ou de la SURCOUVERTURE — trente-six images empilées repeignent trente-six fois
    // le même écran, et une durée seule ne le distingue pas de trente-six images chères.
    let mut posees = 0.0f64;
    let mut pixels = 0.0f64;
    let mut cachees = 0.0f64;

    // OCCLUSION-2 : on ne dessine pas ce qui sera recouvert.
    //
    // La chronique de terrain a mesure cent cinq photos couvrant cinquante fois la surface de
    // l'ecran. La premiere version ne savait traiter qu'un cas -- une seule photo couvrant
    // toute la fenetre -- et ne servait jamais : les photos se chevauchent PARTIELLEMENT, et
    // aucune ne cache seule ce que plusieurs cachent ensemble.
    //
    // Le calcul vit dans le noyau parce qu'il est geometrique : les memes rectangles donnent
    // la meme reponse sur un processeur, sur une carte graphique et sur un telephone.
    let visibles: Vec<&glucose_core::types::BoardImage> =
        Visibles::nouvelles(pass.visibles, board).images().collect();
    let calques: Vec<Calque> = visibles
        .iter()
        .map(|img| calque_de(img, &pass, magasin))
        .collect();
    let mut caches = occlusion::Visibles::default();
    occlusion::ce_qui_se_voit(
        &calques,
        occlusion::Boite::nouvelle(0.0, clip.top, clip.width, clip.height - clip.top),
        &mut caches,
    );
    crate::perf::stage("occlusion");

    for (rang, img) in visibles.into_iter().enumerate() {
        // Une liste de parties vide veut dire : entierement recouvert, rien a peindre.
        if caches.parts(rang).is_empty() {
            cachees += 1.0;
            continue;
        }
        let (wx, wy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &pass.vp);
        let (sx, sy) = (wx as f32, wy as f32);
        let sw = (img.width * pass.vp.scale) as f32;
        let sh = (img.height * pass.vp.scale) as f32;
        if clip.rejects(sx, sy, sw, sh) {
            continue;
        }

        posees += 1.0;
        pixels += (sw as f64) * (sh as f64);
        if !poser_ou_demander(magasin, pixmap, img, (sx, sy, sw, sh)) {
            draw_missing_image(
                typography,
                theme,
                pixmap,
                (sx, sy),
                (sw, sh),
                &img.id,
                img.rotation,
            );
        }
        if store.selected_image_ids.contains(&img.id) {
            draw_image_adornments(pixmap, theme, scale, img, (sx, sy, sw, sh));
        }
        draw_domain_gauge(typography, tints, pixmap, scale, (sx, sy), &img.domains);
    }

    let fenetre = (pixmap.width() as f64) * (pixmap.height() as f64);
    crate::perf::compteur("img_n", posees);
    crate::perf::compteur("img_ecrans", pixels / fenetre.max(1.0));
    crate::perf::compteur("img_mo", magasin.octets() as f64 / 1_048_576.0);
    crate::perf::compteur("vign_mo", magasin.vignettes.octets() as f64 / 1_048_576.0);
    // Combien d'images sont encore en chemin : c'est ce qui distingue « la scene est lente »
    // de « la scene attend », et la trace ne savait pas les separer.
    crate::perf::compteur("img_attente", magasin.en_travail() as f64);
    // Combien d'images l'occlusion a evitees : le gain d'OCCLUSION-1, mesure plutot qu'annonce.
    crate::perf::compteur("img_cachees", cachees);
    // Combien d'images le cache a rendues à la machine : si ce nombre monte pendant qu'on
    // travaille, c'est que la mémoire se tend et que la borne se contracte.
    crate::perf::compteur("img_rendues", magasin.evincees() as f64);
}

/// Ce que le noyau a besoin de savoir d'une image pour decider si elle se voit.
///
/// Trois conditions font qu'une image en cache une autre, et toutes se **constatent** :
///
/// * elle est **opaque** -- la pyramide l'a verifie pixel par pixel au decodage ;
/// * elle n'est pas **tournee** -- sinon sa boite n'est plus ce qu'elle couvre, et les coins
///   laisseraient voir dessous ;
/// * elle est **decodee** -- une image en chemin se dessine comme un cadre, a travers lequel
///   le fond se voit.
fn calque_de(
    img: &glucose_core::types::BoardImage,
    pass: &ViewPass<'_>,
    magasin: &Magasin,
) -> Calque {
    let (sx, sy, sw, sh) = boite_ecran(img, pass);
    let opaque = img.rotation == 0.0
        && img
            .src
            .as_deref()
            .filter(|s| !s.is_empty())
            .and_then(|src| magasin.cache.get(src))
            .is_some_and(|e| e.pyramide.opaque());
    Calque {
        boite: occlusion::Boite::nouvelle(sx, sy, sw, sh),
        opaque,
    }
}

/// La boite ecran d'une image : son coin haut-gauche et sa taille.
fn boite_ecran(img: &glucose_core::types::BoardImage, pass: &ViewPass<'_>) -> (f32, f32, f32, f32) {
    let (wx, wy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &pass.vp);
    (
        wx as f32,
        wy as f32,
        (img.width * pass.vp.scale) as f32,
        (img.height * pass.vp.scale) as f32,
    )
}

/// Pose cette image si elle est décodée ; sinon la demande, et le dit.
///
/// # INVARIANT DECODE-1 — le rendu n'attend jamais un décodage
///
/// Rendre `false` n'est pas un échec : c'est l'état normal d'une image qui vient d'arriver
/// sur le canevas. L'appelant dessine alors son cadre, et la photo paraîtra d'elle-même à
/// l'image où l'atelier la rendra — sans qu'aucune frame ait eu à l'attendre.
///
/// Les deux caches s'empruntent séparément : la pyramide en écriture pour construire un
/// niveau, les vignettes pour en garder une. C'est ce que des champs distincts autorisent, et
/// ce qu'une méthode du magasin interdirait.
fn poser_ou_demander(
    magasin: &mut Magasin,
    pixmap: &mut PixmapMut,
    img: &glucose_core::types::BoardImage,
    ecran: (f32, f32, f32, f32),
) -> bool {
    let Some(src) = img.src.as_deref().filter(|s| !s.is_empty()) else {
        return false;
    };
    // Réclamer marque l'image comme servie à cette passe, ce qui la met hors d'atteinte de
    // l'éviction : ce qui est à l'écran ne se rend jamais à la machine (ADAPT-1).
    if !magasin.reclamer(src) {
        return false;
    }
    let Some(entree) = magasin.cache.get(src) else {
        return false;
    };
    poser(&entree.pyramide, &mut magasin.vignettes, pixmap, img, ecran);
    true
}

/// Pose une image sur le canevas, par le chemin le plus économique qu'elle autorise.
///
/// Deux chemins, et le premier n'existe que parce que le rasteriseur n'est rapide qu'à
/// l'échelle 1 posée sur un entier (MIP-2) :
///
/// * la **vignette**, déjà à la taille et à la phase voulues : un report sans transformation ;
/// * le chemin général, qui rééchantillonne depuis le niveau de pyramide adéquat (MIP-1).
///
/// Une image tournée passe toujours par le second : une vignette est un rectangle droit, et la
/// faire tourner redemanderait la transformation qu'elle sert à éviter.
fn poser(
    pyramide: &photo::Pyramide,
    vignettes: &mut vignette::Vignettes,
    pixmap: &mut PixmapMut,
    img: &glucose_core::types::BoardImage,
    ecran: (f32, f32, f32, f32),
) {
    let (sx, sy, sw, sh) = ecran;
    let opaque = pyramide.opaque();
    let paint = PixmapPaint {
        quality: FilterQuality::Bilinear,
        blend_mode: mode_de_report(opaque, img.rotation),
        ..Default::default()
    };

    // Une vignette n'a de sens que si elle tient dans la fenetre : son role est d'eviter une
    // transformation au moment de poser, or d'une image plus grande que l'ecran on ne voit
    // qu'un morceau. En demander une en zoom proche allouait des dizaines de gigaoctets, et
    // l'application plantait.
    let tient = sw <= pixmap.width() as f32 && sh <= pixmap.height() as f32;
    if img.rotation == 0.0 && tient {
        let forme = photo::Forme::posee(sx, sy, sw, sh);
        if let Some(vignette) = vignettes.pour(&img.id, forme, pyramide) {
            // La phase est déjà dans la vignette : il ne reste qu'une position entière, ce qui
            // est le seul cas où le rasteriseur se contente de recopier.
            pixmap.draw_pixmap(
                sx.floor() as i32,
                sy.floor() as i32,
                vignette.as_ref(),
                &paint,
                Transform::identity(),
                None,
            );
            return;
        }
    }

    // MIP-1 : on part du niveau qui couvre encore la taille posée, jamais de la résolution
    // native. Le filtre lit alors des texels voisins au lieu d'en sauter neuf sur dix.
    let loaded = pyramide.niveau_pour(sw);
    let ts = Transform::from_scale(sw / loaded.width() as f32, sh / loaded.height() as f32)
        .post_translate(sx, sy)
        .post_rotate_at(
            img.rotation.to_degrees() as f32,
            sx + sw / 2.0,
            sy + sh / 2.0,
        );
    pixmap.draw_pixmap(0, 0, loaded.as_ref(), &paint, ts, None);
}

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
fn mode_de_report(opaque: bool, rotation: f64) -> BlendMode {
    if opaque && rotation == 0.0 {
        BlendMode::Source
    } else {
        BlendMode::SourceOver
    }
}

/// Ce qu'une image **sélectionnée** porte en plus : son cadre, et ses prises.
///
/// Une image verrouillée se signale par la couleur de son cadre et par l'absence de ses
/// poignées (fiche 06 § 4.3) : les deux disent le même fait, l'un de loin, l'autre au moment
/// où la main cherche une prise. Les deux suivent la rotation du nœud, comme lui.
fn draw_image_adornments(
    pixmap: &mut PixmapMut,
    theme: &Theme,
    scale: WorldScale,
    img: &glucose_core::types::BoardImage,
    screen_box: (f32, f32, f32, f32),
) {
    let (sx, sy, sw, sh) = screen_box;
    let ink = if img.locked {
        theme.alert
    } else {
        theme.selection_frame
    };
    draw_image_selection(pixmap, ink, (sx, sy), (sw, sh), img.rotation);
    if !img.locked {
        draw_rotated_handles(pixmap, theme, scale, screen_box, &Handle::ALL, img.rotation);
    }
}

/// Une image dont les octets ne sont pas (encore) là : un cadre gris de la chrome, son
/// identifiant dedans. Monochrome — ce n'est pas du contenu, c'est son absence.
fn draw_missing_image(
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

/// Fiche 06 § 4.2 — cadre hairline blanc pur à 0,80, débordant de 3 px, épais de 1,25 px à
/// l'écran quel que soit le zoom. Aucun néon : le contour se lit sur une image claire par le
/// liseré des poignées, sur le fond noir par le blanc.
fn draw_image_selection(
    pixmap: &mut PixmapMut,
    ink: Color,
    at: (f32, f32),
    size: (f32, f32),
    rotation: f64,
) {
    let d = IMAGE_SELECTION_INSET;
    let Some(rect) = Rect::from_xywh(at.0 - d, at.1 - d, size.0 + 2.0 * d, size.1 + 2.0 * d) else {
        return;
    };
    // Le cadre épouse le nœud : il tourne avec lui, autour du même centre.
    let ts = rotation_at(
        rotation,
        (at.0 - d, at.1 - d),
        (size.0 + 2.0 * d, size.1 + 2.0 * d),
    );
    let mut paint = Paint {
        anti_alias: true,
        ..Default::default()
    };
    paint.set_color(ink);
    let stroke = Stroke {
        width: IMAGE_SELECTION_STROKE,
        ..Default::default()
    };
    pixmap.stroke_path(&PathBuilder::from_rect(rect), &paint, &stroke, ts, None);
}

// ── Guides et boîte de sélection — taille écran constante ───────────────────

#[cfg(test)]
mod tests;
