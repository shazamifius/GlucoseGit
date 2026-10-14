//! **Les cases du storyboard** (STORYBOARD-1, fiche 64), au premier rang du monde, juste
//! après les zones : sous les membranes et sous les images qu'on y pose.
//!
//! Le dessin de Glucose Tauri (`StoryboardLayer.ts`), à la mesure près : un cadre `#1a1a1a`
//! bordé de `#333` à 0,8, une bande de description de quarante unités en `#111`, le numéro en
//! gras `#888` dans un badge `#222` à 0,9, la description en `#777`, coupée à la largeur.
//!
//! **Une différence, voulue** : Tauri posait le badge dans le coin du cadre. Une case y était
//! toujours vide — son image n'était jamais écrite —, et le badge ne cachait rien. Ici, une
//! image posée dans une case s'y ajuste et couvre son cadre : le numéro passe dans la bande,
//! devant la description — là où Boords et Storyboarder écrivent le numéro d'un plan.
//!
//! Comme les zones : le corps — des rectangles unis — en **une seule liste** sur la grille de
//! pixels, que le processeur remplit et que la carte pose au même rang ; les textes dans la
//! couche du dessous, sur les deux voies ; la case choisie, son contour et sa croix, dans la
//! couche du dessus — ce sont des prises, pas du contenu.

use super::zone::{rgba, Boite, Rectangle};
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::renderer::pass::Clip;
use crate::renderer::scale::{on_pixel_grid, WorldScale};
use crate::renderer::wrap::wrap_paragraph;
use crate::renderer::PaintKit;
use crate::typography::{Face, TextStyle};
use glucose_core::store::storyboard::DESCRIPTION;
use glucose_core::store::Store;
use glucose_core::types::StoryboardPanel;
use tiny_skia::{Color, PixmapMut, Rect};

/// Les couleurs de Tauri, et leurs opacités sur 255.
const CADRE: u32 = 0x1a1a1a;
const BORD: (u32, u8) = (0x333333, 204);
const BANDE: u32 = 0x111111;
const BADGE: (u32, u8) = (0x222222, 230);
const NUMERO: u32 = 0x888888;
const TEXTE: u32 = 0x777777;
/// Le badge du numéro et la marge, en unités du monde (Tauri : 24 × 18, à 4 du bord).
const BADGE_TAILLE: (f32, f32) = (24.0, 18.0);
const MARGE: f32 = 4.0;
/// Le corps du numéro et de la description.
const CORPS: f32 = 10.0;
/// Le côté de la croix d'une case choisie et sa marge, en pixels d'écran — ceux des zones.
const CROIX: (f32, f32) = (18.0, 6.0);

/// La hauteur de la bande, en unités du monde.
const BANDE_HAUTEUR: f32 = DESCRIPTION as f32;

/// **Les cases du tableau actif qui tombent dans l'écran**, avec la boîte de leur cadre.
fn visibles<'a>(
    store: &'a Store,
    pass: ViewPass<'_>,
    (largeur, hauteur): (f32, f32),
) -> Vec<(&'a StoryboardPanel, Boite)> {
    let Some(board) = store.active_board() else {
        return Vec::new();
    };
    let scale = pass.echelle();
    let clip = Clip {
        width: largeur,
        height: hauteur,
        top: pass.header_h,
    };
    store
        .cases(&board.id)
        .into_iter()
        .filter_map(|case| {
            let (sx, sy) = world_to_screen(case.x, case.y, &pass.vp);
            let boite = (
                sx as f32,
                sy as f32,
                scale.world(case.width as f32),
                scale.world(case.height as f32),
            );
            let entiere = (
                boite.0,
                boite.1,
                boite.2,
                boite.3 + scale.world(BANDE_HAUTEUR),
            );
            (!clip.rejects(entiere.0, entiere.1, entiere.2, entiere.3)).then_some((case, boite))
        })
        .collect()
}

/// **Les rectangles des cases visibles**, dans l'ordre où ils se peignent : par case, le cadre,
/// son bord, la bande, le badge.
pub(crate) fn rectangles(store: &Store, pass: ViewPass<'_>, ecran: (f32, f32)) -> Vec<Rectangle> {
    let scale = pass.echelle();
    let mut liste = Vec::new();
    let mut poser = |(x, y, w, h): Boite, couleur: Color| {
        if let Some(rect) = Rect::from_xywh(x, y, w, h).and_then(on_pixel_grid) {
            liste.push(Rectangle { rect, couleur });
        }
    };
    for (_, (x, y, w, h)) in visibles(store, pass, ecran) {
        let un = |l: f32| scale.world(l);
        poser((x, y, w, h), rgba(CADRE, 255));
        // Le bord, centré sur le cadre comme un trait de Tauri, d'un pixel au moins.
        let e = un(1.0).max(1.0);
        let bord = rgba(BORD.0, BORD.1);
        poser((x - e / 2.0, y - e / 2.0, w + e, e), bord);
        poser((x - e / 2.0, y + h - e / 2.0, w + e, e), bord);
        poser((x - e / 2.0, y + e / 2.0, e, h - e), bord);
        poser((x + w - e / 2.0, y + e / 2.0, e, h - e), bord);
        poser(
            (x, y + h + e / 2.0, w, un(BANDE_HAUTEUR) - e / 2.0),
            rgba(BANDE, 255),
        );
        let (bw, bh) = (un(BADGE_TAILLE.0), un(BADGE_TAILLE.1));
        poser(
            (x + un(MARGE), y + h + un(MARGE), bw, bh),
            rgba(BADGE.0, BADGE.1),
        );
    }
    liste
}

/// **Le numéro et la description des cases visibles**, dans leur bande. Rend vrai si une case
/// en a reçu.
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
    let cases = visibles(store, pass, ecran);
    for (case, boite) in &cases {
        ecrire(kit, pixmap, (case, *boite), scale);
    }
    !cases.is_empty()
}

/// Le numéro centré dans son badge, la description à sa droite, coupée à la largeur restante.
fn ecrire(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    (case, (x, y, w, h)): (&StoryboardPanel, Boite),
    scale: WorldScale,
) {
    let un = |l: f32| scale.world(l);
    let typo = kit.typography;
    let corps = un(CORPS);
    let numero = (case.order + 1).to_string();
    let (nw, _) = typo.measure_text(&numero, corps, Face::Bold);
    let (bx, by) = (x + un(MARGE), y + h + un(MARGE));
    let (bw, bh) = (un(BADGE_TAILLE.0), un(BADGE_TAILLE.1));
    let style = |couleur, face| TextStyle {
        size: corps,
        color: rgba(couleur, 255),
        face,
    };
    typo.draw_text(
        pixmap,
        &numero,
        bx + (bw - nw) / 2.0,
        by + (bh - corps) / 2.0,
        style(NUMERO, Face::Bold),
    );
    let gauche = bx + bw + un(MARGE * 1.5);
    let largeur = (x + w - un(MARGE) - gauche).max(0.0);
    let avance = |_: usize, c: char| typo.advance(c, corps, Face::Regular);
    let mut ligne = by + (bh - corps) / 2.0;
    let bas = y + h + un(BANDE_HAUTEUR);
    for (debut, fin) in wrap_paragraph(&case.description, largeur, avance) {
        if ligne + corps > bas {
            break;
        }
        let morceau = &case.description[debut..fin];
        typo.draw_text(pixmap, morceau, gauche, ligne, style(TEXTE, Face::Regular));
        ligne += corps * 1.3;
    }
}

/// **La croix de la case choisie, à l'écran** : au bout de sa bande. La seule géométrie, que
/// le dessin et le clic lisent (loi L4).
pub(crate) fn croix((x, y, w, h): Boite, bande: f32, densite: f32) -> Boite {
    let (cote, marge) = (CROIX.0 * densite, CROIX.1 * densite);
    (
        x + w - cote - marge,
        y + h + (bande - cote).max(0.0) / 2.0,
        cote,
        cote,
    )
}

/// **Les deux « + » de la case choisie, à l'écran** (P2, proposition 3, fiche 66) : au milieu de
/// ses bords gauche et droit — insérer une case avant elle, ou après. De la taille de sa croix ;
/// la seule géométrie, que le dessin et le clic lisent (loi L4).
pub(crate) fn plus((x, y, w, h): Boite, densite: f32) -> [Boite; 2] {
    let cote = CROIX.0 * densite;
    let haut = y + (h - cote) / 2.0;
    [
        (x - cote / 2.0, haut, cote, cote),
        (x + w - cote / 2.0, haut, cote, cote),
    ]
}

/// **La boîte d'une case à l'écran**, cadre seul — pour qui doit la toucher.
pub(crate) fn boite_a_l_ecran(case: &StoryboardPanel, vp: &glucose_core::types::Viewport) -> Boite {
    let (sx, sy) = world_to_screen(case.x, case.y, vp);
    let zoom = vp.scale as f32;
    (
        sx as f32,
        sy as f32,
        case.width as f32 * zoom,
        case.height as f32 * zoom,
    )
}

/// **La case choisie** : le contour blanc de Tauri autour de son cadre, et sa croix. Dans la
/// couche du dessus : par-dessus l'image qu'elle porte.
pub(super) fn orner_la_case_choisie(
    pixmap: &mut PixmapMut,
    theme: &crate::theme::Theme,
    (store, pass): (&Store, ViewPass<'_>),
    choisie: Option<&str>,
) {
    let Some(id) = choisie else {
        return;
    };
    let ecran = (pixmap.width() as f32, pixmap.height() as f32);
    let Some((_, b)) = visibles(store, pass, ecran)
        .into_iter()
        .find(|(c, _)| c.id == id)
    else {
        return;
    };
    let scale = pass.echelle();
    let un = |l: f32| scale.world(l);
    // Tauri : un trait blanc de 1,5 à 0,8, à deux unités du cadre.
    let (e, ecart) = (un(1.5).max(1.0), un(2.0));
    let contour = rgba(0xffffff, 204);
    let (x, y, w, h) = (
        b.0 - ecart,
        b.1 - ecart,
        b.2 + 2.0 * ecart,
        b.3 + 2.0 * ecart,
    );
    for cote in [
        (x, y, w, e),
        (x, y + h - e, w, e),
        (x, y + e, e, h - 2.0 * e),
        (x + w - e, y + e, e, h - 2.0 * e),
    ] {
        if let Some(r) = Rect::from_xywh(cote.0, cote.1, cote.2, cote.3) {
            crate::renderer::scale::fill_crisp(pixmap, r, contour);
        }
    }
    let d = scale.screen(1.0);
    let boite = croix(b, un(BANDE_HAUTEUR), d);
    super::zone::dessiner_la_croix(pixmap, theme, boite, d);
    for bouton in plus(b, d) {
        super::zone::dessiner_le_plus(pixmap, theme, bouton, d);
    }
    dessiner_la_poignee(pixmap, theme, (store, pass));
}

/// **La poignée du storyboard** (STORYBOARD-2) : au coin bas droit de sa boîte, celle des nœuds
/// — on la tire pour le retracer.
fn dessiner_la_poignee(
    pixmap: &mut PixmapMut,
    theme: &crate::theme::Theme,
    (store, pass): (&Store, ViewPass<'_>),
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let grille = glucose_core::store::storyboard::Grille::default();
    let Some(d) = store.disposition_actuelle(&board.id, grille) else {
        return;
    };
    let (x, y, w, h) = d.boite();
    let (sx, sy) = world_to_screen(x, y, &pass.vp);
    let scale = pass.echelle();
    let boite = (
        sx as f32,
        sy as f32,
        scale.world(w as f32),
        scale.world(h as f32),
    );
    let coin = [glucose_core::resize::Handle::BottomRight];
    crate::renderer::handles::draw_resize_handles(pixmap, theme, scale, boite, &coin);
}

#[cfg(test)]
mod tests;
