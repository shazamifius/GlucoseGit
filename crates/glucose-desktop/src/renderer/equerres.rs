//! **Les équerres de la sélection** (Q12, fiche 137) — sa réponse du 23/11/2026, sous l'image D :
//! *« les carrés avec le flou »* ; et sous l'image B : *« ça j'aime bien au niveau design »*.
//!
//! # Un seul langage, sur un seul calque
//!
//! La sélection se disait de quatre façons selon la sorte du nœud (fiche 133 § 5) : un anneau
//! blanc épais autour d'un texte, un fil presque invisible autour d'un post-it, un cadre fin
//! décollé autour d'une photo, un trait plein au lieu du pointillé sur une membrane, un halo et
//! un voile plus dense sur un dossier. Elle se dit désormais d'une seule : **quatre coins de
//! viseur** autour de chaque nœud choisi, quelle que soit sa sorte, posés ensemble au-dessus de
//! tout (comme tldraw pose les siens sur un calque à part). Rien ne court le long des bords : le
//! nœud reste tel quel. Une flèche n'en porte pas : ses deux bouts sont déjà ses prises.
//!
//! # Ce que l'équerre regarde
//!
//! Le nœud **tel que l'écran le montre** : la boîte montrée d'un texte (celle qui grandit
//! pendant qu'on l'écrit, [`super::card::boite_montree`]), la boîte tournée d'une photo (ses
//! quatre coins par [`glucose_core::rotate::place`], la formule de ses poignées), la mise en
//! page d'un dossier (son minimum compris). Une photo verrouillée prend l'encre de l'alerte,
//! comme son cadre la prenait (fiche 06 § 4.3).

use super::scale::{fill_crisp, WorldScale};
use super::PaintKit;
use crate::canvas::world_to_screen;
use crate::params::ViewPass;
use crate::renderer::TextEditSession;
use crate::theme::Theme;
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::Annotation;
use tiny_skia::{Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// **La distance de l'équerre au nœud, la longueur d'un bras, l'épaisseur du trait**, en pixels
/// logiques. Des nombres de ressenti : ceux de l'image B qu'il a choisie (5 et 14) ; le trait,
/// de 2,5 anti-crénelé dans le prototype, devient **2** posé sur la grille — un trait droit n'a
/// aucun bord oblique, et 2 px est le minimum que le W3C demande à un indicateur (WCAG 2.2,
/// 2.4.13). Tous suivent la place que le nœud leur laisse à l'écran (POIGNEE-1).
pub(crate) const ECART: f32 = 5.0;
const BRAS: f32 = 14.0;
const TRAIT: f32 = 2.0;

/// **Un nœud choisi, tel que l'écran le montre** : ses quatre coins dans l'ordre du tour (le
/// haut-gauche d'abord quand il est droit), s'il est droit, et s'il est verrouillé.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Vise {
    pub coins: [(f32, f32); 4],
    pub droit: bool,
    pub verrouille: bool,
}

impl Vise {
    /// Une boîte écran droite `(x, y, w, h)`.
    fn droit((x, y, w, h): (f32, f32, f32, f32), verrouille: bool) -> Self {
        Self {
            coins: [(x, y), (x + w, y), (x + w, y + h), (x, y + h)],
            droit: true,
            verrouille,
        }
    }

    /// La longueur de ses deux côtés, à l'écran.
    pub(crate) fn cotes(&self) -> (f32, f32) {
        let d = |a: (f32, f32), b: (f32, f32)| (b.0 - a.0).hypot(b.1 - a.1);
        (
            d(self.coins[0], self.coins[1]),
            d(self.coins[1], self.coins[2]),
        )
    }
}

/// Une boîte du monde, à l'écran.
fn a_l_ecran(r: glucose_core::geometry::Rect, pass: &ViewPass<'_>) -> (f32, f32, f32, f32) {
    let (x, y) = world_to_screen(r.left, r.top, &pass.vp);
    let k = pass.vp.scale;
    (
        x as f32,
        y as f32,
        (r.width * k) as f32,
        (r.height * k) as f32,
    )
}

/// **Chaque nœud choisi du tableau actif qui est à l'écran**, tel que l'écran le montre.
pub(crate) fn les_vises(
    kit: PaintKit<'_>,
    store: &Store,
    pass: &ViewPass<'_>,
    editing: Option<&TextEditSession>,
) -> Vec<Vise> {
    let Some(board) = store.active_board() else {
        return Vec::new();
    };
    let visibles = || Visibles::nouvelles(pass.visibles, board);
    let mesure = (kit.typography, kit.math, kit.mises_en_page);
    let mut vises = Vec::new();
    // Le texte ou le post-it qu'on écrit est visé aussi : son anneau le montrait, choisi ou non.
    // Le titre d'une membrane qu'on écrit ne la vise pas : son bord ne changeait pas non plus.
    let ecrit = |ann: &Annotation| {
        matches!(ann, Annotation::Text { .. } | Annotation::Sticky { .. })
            && editing.is_some_and(|e| e.ann_id == ann.id())
    };
    for ann in visibles().annotations() {
        if !store.selected_annotation_ids.contains(ann.id()) && !ecrit(ann) {
            continue;
        }
        let boite = match ann {
            Annotation::Arrow { .. } => continue,
            Annotation::Text { .. } => super::card::boite_montree(mesure, ann, editing),
            _ => ann.rect(),
        };
        if let Some(r) = boite {
            vises.push(Vise::droit(a_l_ecran(r, pass), false));
        }
    }
    for img in visibles().images() {
        if store.selected_image_ids.contains(&img.id) {
            vises.push(vise_d_une_photo(img, pass));
        }
    }
    for f in visibles().dossiers() {
        if store.selected_folder_ids.contains(&f.id) {
            let (x, y) = world_to_screen(f.x, f.y, &pass.vp);
            let (w, h) = super::folder::taille_a_l_ecran(f, pass.echelle());
            vises.push(Vise::droit((x as f32, y as f32, w, h), false));
        }
    }
    vises
}

/// Une photo : sa boîte droite, ou ses quatre coins tournés autour de son centre.
fn vise_d_une_photo(img: &glucose_core::types::BoardImage, pass: &ViewPass<'_>) -> Vise {
    let (x, y, w, h) = a_l_ecran(img.rect(), pass);
    if img.rotation == 0.0 {
        return Vise::droit((x, y, w, h), img.locked);
    }
    let centre = (f64::from(x + w / 2.0), f64::from(y + h / 2.0));
    let (dx, dy) = (f64::from(w / 2.0), f64::from(h / 2.0));
    let coin = |sx: f64, sy: f64| {
        let (cx, cy) = glucose_core::rotate::place(centre, (sx * dx, sy * dy), img.rotation);
        (cx as f32, cy as f32)
    };
    Vise {
        coins: [
            coin(-1.0, -1.0),
            coin(1.0, -1.0),
            coin(1.0, 1.0),
            coin(-1.0, 1.0),
        ],
        droit: false,
        verrouille: img.locked,
    }
}

/// Les mesures d'une équerre autour de ce nœud, en pixels du tampon : l'écart, le bras, le
/// trait — chacun réduit d'autant que les poignées quand le nœud est petit à l'écran.
fn mesures(scale: WorldScale, vise: &Vise) -> (f32, f32, f32) {
    let part = super::handles::part_des_ornements(scale, vise.cotes());
    (
        scale.screen(ECART) * part,
        scale.screen(BRAS) * part,
        scale.screen(TRAIT) * part,
    )
}

/// **Pose les équerres de tous les nœuds choisis**, au-dessus de la scène.
///
/// **Plus fin qu'un pixel, le trait s'estompe au lieu de rester entier** (POIGNEE-1, la loi du
/// cadre des photos qu'elles remplacent) : un trait de 0,2 px couvre le cinquième d'un pixel, il
/// en reçoit le cinquième de l'encre. Sinon, des milliers de vignettes choisies de loin feraient
/// la grille blanche de son retour du 07/10.
pub(crate) fn poser(pixmap: &mut PixmapMut, theme: &Theme, scale: WorldScale, vises: &[Vise]) {
    for v in vises {
        let (ecart, bras, trait_) = mesures(scale, v);
        let mut encre = if v.verrouille {
            theme.alert
        } else {
            theme.selection_frame
        };
        if trait_ < 1.0 {
            encre.apply_opacity(trait_);
        }
        if v.droit {
            poser_droites(pixmap, v, (ecart, bras, trait_.round().max(1.0)), encre);
        } else {
            poser_penchees(pixmap, v, (ecart, bras, trait_.max(1.0)), encre);
        }
    }
}

/// **Les quatre équerres d'un nœud droit, en filets sur la grille** : chaque coin, un bras
/// horizontal qui porte l'angle, puis un bras vertical qui commence sous lui — aucun pixel posé
/// deux fois, ce qui compterait double sous une encre translucide.
fn poser_droites(
    pixmap: &mut PixmapMut,
    v: &Vise,
    (ecart, bras, trait_): (f32, f32, f32),
    encre: tiny_skia::Color,
) {
    let (x0, y0) = (
        (v.coins[0].0 - ecart).round(),
        (v.coins[0].1 - ecart).round(),
    );
    let (x1, y1) = (
        (v.coins[2].0 + ecart).round(),
        (v.coins[2].1 + ecart).round(),
    );
    // Le bras ne passe jamais le quart du côté : deux équerres ne se touchent pas.
    let bras = bras
        .min((x1 - x0) / 4.0)
        .min((y1 - y0) / 4.0)
        .round()
        .max(trait_);
    let filet = |pixmap: &mut PixmapMut, x: f32, y: f32, w: f32, h: f32| {
        if let Some(r) = Rect::from_xywh(x, y, w, h) {
            fill_crisp(pixmap, r, encre);
        }
    };
    let verticale = bras - trait_;
    for (cx, sx) in [(x0, 0.0), (x1, 1.0)] {
        for (cy, sy) in [(y0, 0.0), (y1, 1.0)] {
            filet(pixmap, cx - sx * bras, cy - sy * trait_, bras, trait_);
            if verticale > 0.0 {
                let haut = if sy == 0.0 { cy + trait_ } else { cy - bras };
                filet(pixmap, cx - sx * trait_, haut, trait_, verticale);
            }
        }
    }
}

/// **Les quatre équerres d'un nœud penché**, en un tracé : chaque coin écarté le long de ses deux
/// côtés, ses bras le long d'eux. Le trait est centré sur le tracé : le tracé passe à un
/// demi-trait en deçà de l'écart, pour que le bord extérieur tombe où tombe celui d'une équerre
/// droite.
fn poser_penchees(
    pixmap: &mut PixmapMut,
    v: &Vise,
    (ecart, bras, trait_): (f32, f32, f32),
    encre: tiny_skia::Color,
) {
    let mut pb = PathBuilder::new();
    let (cote_a, cote_b) = v.cotes();
    let bras = bras
        .min((cote_a + 2.0 * ecart) / 4.0)
        .min((cote_b + 2.0 * ecart) / 4.0);
    let recul = ecart - trait_ / 2.0;
    for i in 0..4 {
        let p = v.coins[i];
        let unitaire = |q: (f32, f32)| {
            let (dx, dy) = (q.0 - p.0, q.1 - p.1);
            let n = dx.hypot(dy).max(f32::EPSILON);
            (dx / n, dy / n)
        };
        let u = unitaire(v.coins[(i + 1) % 4]);
        let w = unitaire(v.coins[(i + 3) % 4]);
        let angle = (p.0 - recul * (u.0 + w.0), p.1 - recul * (u.1 + w.1));
        pb.move_to(angle.0 + bras * u.0, angle.1 + bras * u.1);
        pb.line_to(angle.0, angle.1);
        pb.line_to(angle.0 + bras * w.0, angle.1 + bras * w.1);
    }
    let Some(chemin) = pb.finish() else {
        return;
    };
    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    paint.set_color(encre);
    let stroke = Stroke {
        width: trait_,
        ..Stroke::default()
    };
    pixmap.stroke_path(&chemin, &paint, &stroke, Transform::identity(), None);
}

#[cfg(test)]
mod tests;
