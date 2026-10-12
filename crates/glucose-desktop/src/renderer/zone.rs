//! **Les zones d'un gabarit**, sous tout le reste (PRESET-1, fiche 62).
//!
//! Le dessin de Glucose Tauri (`ZoneRenderer.ts`), à la mesure près : un fond sombre
//! translucide, un bord en pointillés de la couleur de la catégorie — huit unités, six de
//! vide —, un bandeau de trente-six unités teinté, un filet de deux en haut, le nom de la
//! catégorie en capitales et sa phrase au milieu. Tout en unités du monde : une zone grandit
//! avec le zoom, comme son contenu.
//!
//! # Sous les membranes, sur les deux voies
//!
//! Dans Tauri, les zones sont le premier rang du monde : une membrane posée dans une zone passe
//! par-dessus. Or la voie graphique peint elle-même la forme des membranes, **avant** la couche
//! du dessous ; une zone peinte dans cette couche y passait par-dessus les membranes — l'épreuve
//! des deux voies l'a vu, 123 niveaux d'écart.
//!
//! Le corps d'une zone n'est fait que de rectangles unis : le socle en dresse **une seule
//! liste**, déjà sur la grille de pixels ([`rectangles`]) ; le processeur les remplit, la carte
//! les pose (`present::zones_gpu`), l'un et l'autre juste après le fond. Les deux voies posent
//! ainsi les mêmes rectangles au même rang, par construction. Les textes — le nom, la phrase —
//! restent dans la couche du dessous, comme les titres des membranes.
//!
//! # Quand un pointillé devient plus fin qu'un pixel
//!
//! De loin, huit unités de trait et six de vide tiennent dans un pixel : dessiner chaque tiret
//! coûterait des centaines de rectangles pour un pixel gris. Sous un pixel la période, le bord
//! devient un trait continu de l'**opacité moyenne** du pointillé — huit quatorzièmes de la
//! sienne : ce que l'œil voit déjà, au bit près de la moyenne.

use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::renderer::pass::Clip;
use crate::renderer::scale::{on_pixel_grid, WorldScale};
use crate::renderer::wrap::wrap_paragraph;
use crate::renderer::PaintKit;
use crate::typography::{Face, TextStyle};
use glucose_core::gabarits::{ce_que_montre, Montre};
use glucose_core::store::Store;
use glucose_core::types::BoardZone;
use tiny_skia::{Color, PixmapMut, Rect};

/// Le tiret et le vide du bord, en unités du monde.
const TIRET: (f32, f32) = (8.0, 6.0);
/// La hauteur du bandeau, l'épaisseur du filet, la marge du texte, en unités du monde.
const BANDEAU: f32 = 36.0;
const FILET: f32 = 2.0;
const MARGE: f32 = 12.0;
/// Les corps du nom et de la phrase.
const CORPS_DU_NOM: f32 = 13.0;
const CORPS_DE_LA_PHRASE: f32 = 11.0;
/// Les opacités de Tauri, sur 255 : le fond (0,5), le bord (0,4), le bandeau (0,15), le filet
/// (0,8).
const FOND: (u32, u8) = (0x111111, 128);
const BORD: u8 = 102;
const TEINTE_DU_BANDEAU: u8 = 38;
const TEINTE_DU_FILET: u8 = 204;
/// La couleur de la phrase.
const PHRASE: u32 = 0x555555;

/// **Un rectangle du corps d'une zone**, sur la grille de pixels, et sa couleur : ce que les
/// deux voies posent, l'une en le remplissant, l'autre par la carte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub rect: Rect,
    pub couleur: Color,
}

/// Une boîte à l'écran : gauche, haut, largeur, hauteur, en pixels.
type Boite = (f32, f32, f32, f32);

/// Les zones du tableau actif qui tombent dans l'écran, avec leur boîte à l'écran.
fn visibles<'a>(
    store: &'a Store,
    pass: ViewPass<'_>,
    (largeur, hauteur): (f32, f32),
) -> Vec<(&'a BoardZone, Boite)> {
    let Some(board) = store.active_board() else {
        return Vec::new();
    };
    let scale = pass.echelle();
    let clip = Clip {
        width: largeur,
        height: hauteur,
        top: pass.header_h,
    };
    board
        .zones
        .iter()
        .filter_map(|zone| {
            let (sx, sy) = world_to_screen(zone.x, zone.y, &pass.vp);
            let boite = (
                sx as f32,
                sy as f32,
                scale.world(zone.width as f32),
                scale.world(zone.height as f32),
            );
            (!clip.rejects(boite.0, boite.1, boite.2, boite.3)).then_some((zone, boite))
        })
        .collect()
}

/// **Les rectangles du corps des zones visibles**, dans l'ordre où ils se peignent : par zone,
/// le fond, le bord, le bandeau, le filet.
pub(crate) fn rectangles(store: &Store, pass: ViewPass<'_>, ecran: (f32, f32)) -> Vec<Rectangle> {
    let scale = pass.echelle();
    let mut liste = Vec::new();
    for (zone, boite) in visibles(store, pass, ecran) {
        let couleur = ce_que_montre(zone, &store.project).couleur;
        let un = |l: f32| scale.world(l);
        let mut poser = |(x, y, w, h): (f32, f32, f32, f32), c: Color| {
            if let Some(rect) = Rect::from_xywh(x, y, w, h).and_then(on_pixel_grid) {
                liste.push(Rectangle { rect, couleur: c });
            }
        };
        poser(boite, rgba(FOND.0, FOND.1));
        bord(boite, couleur, scale, &mut poser);
        poser(
            (boite.0, boite.1, boite.2, un(BANDEAU)),
            rgba(couleur, TEINTE_DU_BANDEAU),
        );
        poser(
            (boite.0, boite.1, boite.2, un(FILET)),
            rgba(couleur, TEINTE_DU_FILET),
        );
    }
    liste
}

/// **Remplit les rectangles** — la voie processeur.
pub(super) fn remplir(pixmap: &mut PixmapMut, rectangles: &[Rectangle]) -> bool {
    for r in rectangles {
        crate::renderer::scale::fill_crisp(pixmap, r.rect, r.couleur);
    }
    !rectangles.is_empty()
}

/// **Les textes des zones visibles** : le nom en capitales dans le bandeau, la phrase au milieu.
/// Rend vrai si une zone en a reçu.
pub(super) fn ecrire_les_textes(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) -> bool {
    let scale = pass.echelle();
    if !scale.draws_detail() {
        return false;
    }
    let ecran = (pixmap.width() as f32, pixmap.height() as f32);
    let zones = visibles(store, pass, ecran);
    for (zone, boite) in &zones {
        ecrire(
            kit,
            pixmap,
            *boite,
            &ce_que_montre(zone, &store.project),
            scale,
        );
    }
    !zones.is_empty()
}

/// Le nom d'une zone dans son bandeau, sa phrase au milieu, coupée à sa largeur.
fn ecrire(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    (x, y, w, h): (f32, f32, f32, f32),
    montre: &Montre,
    scale: WorldScale,
) {
    let un = |l: f32| scale.world(l);
    let typo = kit.typography;
    let nom = TextStyle {
        size: un(CORPS_DU_NOM),
        color: rgba(montre.couleur, 255),
        face: Face::Bold,
    };
    typo.draw_text(
        pixmap,
        &montre.nom.to_uppercase(),
        x + un(MARGE),
        y + un(10.0),
        nom,
    );
    let corps = un(CORPS_DE_LA_PHRASE);
    let phrase = TextStyle {
        size: corps,
        color: rgba(PHRASE, 255),
        face: Face::Regular,
    };
    let largeur = (w - 2.0 * un(MARGE)).max(0.0);
    let avance = |_: usize, c: char| typo.advance(c, corps, Face::Regular);
    let mut ligne_y = y + h / 2.0 - un(10.0);
    for (debut, fin) in wrap_paragraph(&montre.description, largeur, avance) {
        typo.draw_text(
            pixmap,
            &montre.description[debut..fin],
            x + un(MARGE),
            ligne_y,
            phrase,
        );
        ligne_y += corps * 1.3;
    }
}

/// Le bord en pointillés, chaque côté depuis son coin — ou un trait continu à l'opacité
/// moyenne, quand une période tient dans un pixel.
fn bord(
    (x, y, w, h): (f32, f32, f32, f32),
    couleur: u32,
    scale: WorldScale,
    poser: &mut impl FnMut((f32, f32, f32, f32), Color),
) {
    let e = scale.world(1.0).max(1.0);
    let (tiret, vide) = (scale.world(TIRET.0), scale.world(TIRET.1));
    let periode = tiret + vide;
    if periode <= 1.0 {
        let moyen = (f32::from(BORD) * tiret / periode).round() as u8;
        let c = rgba(couleur, moyen);
        for cote in [
            (x, y, w, e),
            (x, y + h - e, w, e),
            (x, y, e, h),
            (x + w - e, y, e, h),
        ] {
            poser(cote, c);
        }
        return;
    }
    let c = rgba(couleur, BORD);
    let mut t = 0.0;
    while t < w {
        let l = tiret.min(w - t);
        poser((x + t, y, l, e), c);
        poser((x + t, y + h - e, l, e), c);
        t += periode;
    }
    let mut t = 0.0;
    while t < h {
        let l = tiret.min(h - t);
        poser((x, y + t, e, l), c);
        poser((x + w - e, y + t, e, l), c);
        t += periode;
    }
}

fn rgba(rgb: u32, alpha: u8) -> Color {
    Color::from_rgba8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, alpha)
}

#[cfg(test)]
mod tests;
