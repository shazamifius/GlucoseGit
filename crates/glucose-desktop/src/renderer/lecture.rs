//! **L'animatique, à l'écran** (fiche 67) : la case montrée remplit l'écran ; tout le reste est
//! voilé de noir — les voisines, les autres nœuds, le canevas —, ses textes passent en
//! sous-titre au bas de son cadre, et un compteur dit où l'on en est.
//!
//! Dans la couche du dessus, après les ornements : sur les deux voies, au-dessus des images —
//! la place des prises (ORNEMENTS-1). Rien ici ne se calcule deux fois : la case est où le
//! canevas la pose, et la croix est où le clic la cherche ([`croix`], loi L4).

use super::storyboard::boite_a_l_ecran;
use super::wrap::wrap_paragraph;
use super::PaintKit;
use crate::params::ViewPass;
use crate::typography::{Face, TextStyle};
use crate::ui::UiState;
use glucose_core::store::Store;
use glucose_core::types::{Storyboard, StoryboardPanel};
use tiny_skia::{Color, PixmapMut, Rect};

/// Le corps des sous-titres et du compteur, en points — ceux d'une question (`ui::question`).
const SOUS_TITRE: f32 = 17.0;
const COMPTEUR: f32 = 13.0;
/// La marge au bord de l'écran et sous les sous-titres, en points.
const MARGE: f32 = 16.0;
/// Le côté de la croix, en points : la cible d'un doigt.
const CROIX: f32 = crate::theme::CIBLE_DU_DOIGT;

/// **La croix qui finit la lecture**, à l'écran : en haut à droite, sous la barre d'état.
pub fn croix(largeur: f32, haut: f32, densite: f32) -> (f32, f32, f32, f32) {
    let (cote, marge) = (CROIX * densite, MARGE * densite / 2.0);
    (largeur - cote - marge, haut + marge, cote, cote)
}

/// **Les sous-titres d'une case** : sa description, puis chaque champ qui dit quelque chose,
/// précédé de son nom.
pub fn sous_titres(case: &StoryboardPanel, storyboard: &Storyboard) -> Vec<String> {
    let description = Some(case.description.trim().to_string()).filter(|d| !d.is_empty());
    let champs = storyboard.champs.iter().enumerate().filter_map(|(i, nom)| {
        let texte = case.texte(i).trim();
        (!texte.is_empty()).then(|| format!("{nom} : {texte}"))
    });
    description.into_iter().chain(champs).collect()
}

/// **Voile tout ce qui n'est pas la case lue**, et pose ses sous-titres, le compteur et la
/// croix. Sans lecture, rien.
pub(super) fn voiler(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    (ui, pass): (&UiState, ViewPass<'_>),
) {
    let Some(lecture) = &ui.lecture else {
        return;
    };
    let cases = store.cases(&lecture.board, &lecture.storyboard);
    let (Some(case), Some(storyboard)) = (
        cases.get(lecture.rang),
        store.storyboard(&lecture.board, &lecture.storyboard),
    ) else {
        return;
    };
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);
    let (x, y, cw, ch) = boite_a_l_ecran(case, &pass.vp);
    let noir = Color::BLACK;
    for (rx, ry, rw, rh) in [
        (0.0, 0.0, w, y),
        (0.0, y + ch, w, h - y - ch),
        (0.0, y, x, ch),
        (x + cw, y, w - x - cw, ch),
    ] {
        if let Some(r) = Rect::from_xywh(rx, ry, rw, rh) {
            crate::renderer::scale::fill_crisp(pixmap, r, noir);
        }
    }
    let d = ui.scale();
    sous_titrer(
        kit,
        pixmap,
        (x, y, cw, ch),
        &sous_titres(case, storyboard),
        d,
    );
    let numero = format!("{} / {}", lecture.rang + 1, cases.len());
    let plan = case
        .plan
        .map(|p| format!(" · {}", p.noms().0))
        .unwrap_or_default();
    let pause = match lecture.temps {
        crate::interactions::lecture::Temps::EnPause(_) => " · en pause",
        crate::interactions::lecture::Temps::Depuis(_) => "",
    };
    let style = |size: f32, a: u8| TextStyle {
        size: size * d,
        color: Color::from_rgba8(255, 255, 255, a),
        face: Face::Regular,
    };
    // Le compteur à gauche, sur la ligne de la croix à droite.
    let (bx, by, bw, bh) = croix(w, ui.marges.haut, d);
    let compteur = format!("{numero}{plan}{pause}");
    let ligne = by + (bh - COMPTEUR * d) / 2.0;
    let gauche = ui.marges.gauche + MARGE * d;
    kit.typography
        .draw_text(pixmap, &compteur, gauche, ligne, style(COMPTEUR, 150));
    let (lw, _) = kit.typography.measure_text("×", 22.0 * d, Face::Regular);
    let at = (bx + (bw - lw) / 2.0, by + (bh - 22.0 * d) / 2.0);
    kit.typography
        .draw_text(pixmap, "×", at.0, at.1, style(22.0, 200));
}

/// Les sous-titres, du bas du cadre vers le haut, sur un bandeau sombre ; coupés à la largeur
/// du cadre, et jamais plus haut que la moitié de la case — l'image reste ce qu'on lit.
fn sous_titrer(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    (x, y, w, h): (f32, f32, f32, f32),
    textes: &[String],
    d: f32,
) {
    if textes.is_empty() {
        return;
    }
    let typo = kit.typography;
    let (corps, marge) = (SOUS_TITRE * d, MARGE * d);
    let largeur = (w - 2.0 * marge).max(0.0);
    let avance = |_: usize, c: char| typo.advance(c, corps, Face::Regular);
    let lignes: Vec<&str> = textes
        .iter()
        .flat_map(|t| {
            wrap_paragraph(t, largeur, avance)
                .into_iter()
                .map(move |(a, b)| &t[a..b])
        })
        .collect();
    let pas = corps * 1.35;
    let montrees = lignes
        .len()
        .min(((h / 2.0) / pas).floor().max(1.0) as usize);
    let hauteur = montrees as f32 * pas + marge;
    let haut = y + h - hauteur;
    if let Some(r) = Rect::from_xywh(x, haut, w, hauteur) {
        crate::renderer::scale::fill_crisp(pixmap, r, Color::from_rgba8(0, 0, 0, 150));
    }
    // Si tout ne tient pas, ce sont les premières lignes qui se lisent : la description.
    for (i, ligne) in lignes.iter().take(montrees).enumerate() {
        let (lw, _) = typo.measure_text(ligne, corps, Face::Regular);
        let at = (x + (w - lw) / 2.0, haut + marge / 2.0 + i as f32 * pas);
        let style = TextStyle {
            size: corps,
            color: Color::WHITE,
            face: Face::Regular,
        };
        typo.draw_text(pixmap, ligne, at.0, at.1, style);
    }
}

#[cfg(test)]
mod tests;
