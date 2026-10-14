//! **La lecture, dans le panneau** (schéma 4, fiche 67) : « ▶ Lire » et la durée d'une case
//! pour tout le storyboard ; puis, quand une case est choisie, **ce qu'elle dit** — sa durée,
//! son plan, sa description et chacun des champs du storyboard, qu'on touche pour l'écrire, dont
//! on touche le nom pour le renommer ou le retirer, et « + Champ ».
//!
//! Storyboarder montre ainsi la case courante à côté de son image (durée, dialogue, action,
//! notes) ; Krita, chaque case avec ses commentaires. Ici, le canevas montre les cases et le
//! panneau ce que dit celle qu'on a choisie.

use super::{StoryboardIntent, WidgetRect};
use crate::params::{Pointer, ScaledRect};
use glucose_core::store::storyboard::{Plan, Texte};
use glucose_core::store::Store;

/// **Ce que le storyboard dit pour la lecture**, recopié du document : ses champs, la durée
/// d'une case qui n'a pas la sienne, et la case choisie.
#[derive(Debug, Clone, PartialEq)]
pub struct Lecture {
    /// Le storyboard que le panneau montre — le courant (fiche 68).
    pub storyboard: String,
    pub champs: Vec<String>,
    pub duree_ms: u32,
    pub case: Option<Choisie>,
}

/// La case choisie, telle que le panneau la montre.
#[derive(Debug, Clone, PartialEq)]
pub struct Choisie {
    pub id: String,
    pub numero: i32,
    pub description: String,
    pub textes: Vec<String>,
    pub plan: Option<Plan>,
    /// Sa durée à elle, si elle en a une.
    pub duree_ms: Option<u32>,
}

impl Lecture {
    /// Ce que le document dit de ce storyboard du tableau actif, et de la case `choisie` — si
    /// elle est à lui.
    pub fn du(store: &Store, sb: &str, choisie: Option<&str>) -> Option<Self> {
        let board = &store.project.active_board_id;
        let storyboard = store.storyboard(board, sb)?;
        let case = choisie.and_then(|id| {
            let c = store.cases(board, sb).into_iter().find(|c| c.id == id)?;
            Some(Choisie {
                id: c.id.clone(),
                numero: c.order + 1,
                description: c.description.clone(),
                textes: (0..storyboard.champs.len())
                    .map(|i| c.texte(i).to_string())
                    .collect(),
                plan: c.plan,
                duree_ms: c.duree_ms,
            })
        });
        Some(Self {
            storyboard: storyboard.id.clone(),
            champs: storyboard.champs.clone(),
            duree_ms: storyboard.duree_ms,
            case,
        })
    }
}

/// **Un texte de la case choisie, dans le panneau** : son nom — qu'on touche pour renommer le
/// champ, sauf la description —, et ce qu'elle y dit, qu'on touche pour l'écrire.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeeDeTexte {
    pub texte: Texte,
    pub nom: WidgetRect,
    pub ecrit: WidgetRect,
}

/// La géométrie de la case choisie.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseLayout {
    /// Où s'écrit « CASE n ».
    pub titre: (f32, f32),
    /// Le champ de sa durée (CHAMP-1).
    pub duree: WidgetRect,
    /// « ↺ » : rendre la case à la durée du storyboard — quand elle a la sienne.
    pub rendre: Option<WidgetRect>,
    /// Le sélecteur du plan, qui tourne.
    pub plan: WidgetRect,
    pub textes: Vec<RangeeDeTexte>,
    pub ajouter_un_champ: WidgetRect,
}

/// La géométrie de la lecture : « ▶ Lire », la durée par case, et la case choisie.
#[derive(Debug, Clone, PartialEq)]
pub struct LectureLayout {
    pub lire: WidgetRect,
    pub duree: WidgetRect,
    /// « Exporter en planches… » (P2.7, fiche 68).
    pub planches: WidgetRect,
    pub case: Option<CaseLayout>,
}

/// La hauteur d'une rangée de texte, écart compris, et la largeur de la colonne des noms.
const RANGEE: f32 = 22.0;
const NOMS: f32 = 74.0;

/// **Pose la lecture à partir de `haut`** ; rend sa géométrie et le bas de ce qu'elle occupe.
pub fn poser(frame: ScaledRect, haut: f32, lecture: &Lecture) -> (LectureLayout, f32) {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (x, largeur) = (frame.x + 14.0 * s, frame.w - 28.0 * s);
    // Le libellé du champ s'écrit douze points au-dessus de lui (CHAMP-1).
    let y = haut + 14.0 * s;
    let moitie = (largeur - 6.0 * s) / 2.0;
    let lire = WidgetRect::new(x, y - 2.0 * s, moitie, 24.0 * s);
    let duree = WidgetRect::new(x + moitie + 6.0 * s, y, moitie, 20.0 * s);
    let planches = WidgetRect::new(x, y + 28.0 * s, largeur, 24.0 * s);
    let mut bas = planches.y + planches.h;
    let case = lecture.case.as_ref().map(|c| {
        let (layout, sous) = poser_la_case(frame, bas + 10.0 * s, lecture, c);
        bas = sous;
        layout
    });
    let layout = LectureLayout {
        lire,
        duree,
        planches,
        case,
    };
    (layout, bas)
}

/// La case choisie : son titre, sa durée, « ↺ », son plan, puis une rangée par texte.
fn poser_la_case(
    frame: ScaledRect,
    haut: f32,
    lecture: &Lecture,
    c: &Choisie,
) -> (CaseLayout, f32) {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (x, largeur) = (frame.x + 14.0 * s, frame.w - 28.0 * s);
    let titre = (x, haut);
    let y = haut + 14.0 * s + 14.0 * s;
    let tiers = (largeur - 12.0 * s) / 3.0;
    let duree = WidgetRect::new(x, y, tiers, 20.0 * s);
    let rendre = c
        .duree_ms
        .is_some()
        .then(|| WidgetRect::new(x + tiers + 4.0 * s, y, 20.0 * s, 20.0 * s));
    let plan_x = x + tiers + 28.0 * s;
    let plan = WidgetRect::new(plan_x, y, x + largeur - plan_x, 20.0 * s);
    let mut haut_rangee = y + 28.0 * s;
    let tous =
        std::iter::once(Texte::Description).chain((0..lecture.champs.len()).map(Texte::Champ));
    let textes = tous
        .map(|texte| {
            let h = (RANGEE - 4.0) * s;
            let nom = WidgetRect::new(x, haut_rangee, NOMS * s, h);
            let ecrit = WidgetRect::new(
                x + (NOMS + 4.0) * s,
                haut_rangee,
                largeur - (NOMS + 4.0) * s,
                h,
            );
            haut_rangee += RANGEE * s;
            RangeeDeTexte { texte, nom, ecrit }
        })
        .collect();
    let ajouter_un_champ = WidgetRect::new(x, haut_rangee + 2.0 * s, 72.0 * s, 18.0 * s);
    let bas = ajouter_un_champ.y + ajouter_un_champ.h;
    let layout = CaseLayout {
        titre,
        duree,
        rendre,
        plan,
        textes,
        ajouter_un_champ,
    };
    (layout, bas)
}

/// **Le clic dans la lecture** — ce qu'il demande, s'il touche quelque chose. Les champs de
/// durée se prennent ailleurs, comme tout champ de nombre (CHAMP-1).
pub fn cliquer(
    layout: &LectureLayout,
    lecture: &Lecture,
    pointer: Pointer,
) -> Option<StoryboardIntent> {
    let hit = |r: WidgetRect| r.contains(pointer.x, pointer.y);
    if hit(layout.lire) {
        let depuis = lecture.case.as_ref().map(|c| c.id.clone());
        return Some(StoryboardIntent::Lire(depuis));
    }
    if hit(layout.planches) {
        return Some(StoryboardIntent::Planches);
    }
    let (case, c) = (layout.case.as_ref()?, lecture.case.as_ref()?);
    if case.rendre.is_some_and(hit) {
        return Some(StoryboardIntent::RendreLaDuree(c.id.clone()));
    }
    if hit(case.plan) {
        return Some(StoryboardIntent::Plan(c.id.clone(), Plan::suivant(c.plan)));
    }
    if hit(case.ajouter_un_champ) {
        return Some(StoryboardIntent::NommerLeChamp(None));
    }
    let rangee = case.textes.iter().find(|r| hit(r.nom) || hit(r.ecrit))?;
    match rangee.texte {
        Texte::Champ(rang) if hit(rangee.nom) => Some(StoryboardIntent::NommerLeChamp(Some(rang))),
        texte => Some(StoryboardIntent::Ecrire(c.id.clone(), texte)),
    }
}

#[cfg(test)]
mod tests;
