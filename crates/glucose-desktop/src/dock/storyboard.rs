//! Le panneau STORYBOARD (STORYBOARD-1, fiche 64) — celui de Glucose Tauri, champ par champ
//! (`StoryboardControls.tsx`) : le format, la largeur, les colonnes, l'espacement, l'aperçu de
//! la grille, « Activer » ou « Mettre à jour » et sa croix ; puis les cases, chacune avec son
//! numéro, sa description et sa croix, « + Ajouter » et « Re-numéroter les panels ».
//!
//! Le panneau ne touche pas au canevas lui-même : il dit ce qu'on lui demande
//! ([`StoryboardIntent`]), et l'application le fait. « Activer » arme la grille, qui suit le
//! curseur jusqu'au clic qui la pose (`interactions::storyboard`).
//!
//! Tauri faisait défiler sa liste ; le panneau, ici, la montre **par pages**, ‹ et › — un geste
//! qu'un doigt fait aussi bien qu'une souris.

use super::WidgetRect;
use crate::params::{Pointer, ScaledRect};
use glucose_core::store::storyboard::{Format, Grille, Plan, Texte};
use glucose_core::store::Store;

pub mod lecture;
pub mod paint;

pub use lecture::Lecture;

/// Les cinq cadrages de la référence (fiche 10 § 5.3), dans l'ordre de [`Format::TOUS`].
pub const FORMATS: [&str; 5] = [
    "16:9 — Cinéma HD",
    "4:3 — Classique",
    "2.35:1 — Scope",
    "1:1 — Carré",
    "9:16 — Vertical",
];

/// L'aperçu de la grille : sa hauteur au plus, et l'écart entre deux cellules (Tauri : 120, 3).
const APERCU: (f32, f32) = (120.0, 3.0);
/// Combien de cellules l'aperçu montre au plus (Tauri : 24).
const CELLULES_MAX: usize = 24;
/// La hauteur d'une ligne de la liste, écart compris.
const LIGNE: f32 = 24.0;

#[derive(Debug, Clone, PartialEq)]
pub struct StoryboardState {
    pub format_idx: usize,
    pub panel_width: f64,
    pub cols: usize,
    pub gap: f64,
    /// La page de la liste des cases qu'on regarde.
    pub page: usize,
    /// La grille du storyboard du document, telle que les champs l'ont vue en dernier
    /// (STORYBOARD-2).
    pub vue: Option<Grille>,
    /// **Ce que le document dit**, recopié à chaque image (fiche 67) : le panneau, son clic et
    /// ses champs lisent ce miroir, et lui seul — la loi L4.
    pub contenu: Contenu,
    /// **La durée d'une case pour tout le storyboard, et celle de la case choisie**, en
    /// millisecondes : ce que leurs champs règlent (CHAMP-1), poussé au document quand on les
    /// lâche ou qu'on valide la frappe.
    pub duree_ms: f64,
    pub duree_case: f64,
    /// Ces deux durées, telles que le document les disait en dernier.
    vue_durees: Option<(u32, Option<(String, u32)>)>,
}

impl Default for StoryboardState {
    /// La grille de Tauri quand rien n'est réglé — définie une fois, dans le noyau.
    fn default() -> Self {
        let tauri = Grille::default();
        Self {
            format_idx: Format::TOUS
                .iter()
                .position(|f| *f == tauri.format)
                .unwrap_or(0),
            panel_width: tauri.largeur,
            cols: tauri.colonnes,
            gap: tauri.ecart,
            page: 0,
            vue: None,
            contenu: Contenu::default(),
            duree_ms: f64::from(glucose_core::types::DUREE_DE_NAISSANCE_MS),
            duree_case: f64::from(glucose_core::types::DUREE_DE_NAISSANCE_MS),
            vue_durees: None,
        }
    }
}

impl StoryboardState {
    pub fn format_label(&self) -> &'static str {
        FORMATS[self.format_idx.min(FORMATS.len() - 1)]
    }

    /// **Le panneau suit le document** à chaque image : ses cases, son storyboard, la case
    /// `choisie` ; et ses champs, la grille et les durées **quand le document les change**.
    pub fn suivre(&mut self, store: &Store, choisie: Option<&str>) {
        self.contenu = Contenu::du(store, choisie);
        self.suivre_la_grille(store.grille_du_storyboard(&store.project.active_board_id));
        self.suivre_les_durees();
    }

    /// **Les durées suivent le document quand il les change** — un `Ctrl+Z`, une autre case
    /// choisie ; entre deux, ce que la main règle reste, jusqu'à ce qu'elle le lâche.
    fn suivre_les_durees(&mut self) {
        let Some(l) = &self.contenu.lecture else {
            self.vue_durees = None;
            return;
        };
        let case = l
            .case
            .as_ref()
            .map(|c| (c.id.clone(), c.duree_ms.unwrap_or(l.duree_ms)));
        let vue = Some((l.duree_ms, case));
        if vue == self.vue_durees {
            return;
        }
        if let Some((defaut, case)) = &vue {
            self.duree_ms = f64::from(*defaut);
            self.duree_case = case.as_ref().map_or(self.duree_ms, |(_, d)| f64::from(*d));
        }
        self.vue_durees = vue;
    }

    /// **Les champs suivent la grille du document quand elle change** — un rectangle tracé, un
    /// autre tableau, un `Ctrl+Z` (STORYBOARD-2) ; entre deux changements, ce qu'on y règle
    /// reste, pour le prochain « Mettre à jour ».
    pub fn suivre_la_grille(&mut self, grille: Option<Grille>) {
        if grille == self.vue {
            return;
        }
        self.vue = grille;
        if let Some(g) = grille {
            self.format_idx = Format::TOUS
                .iter()
                .position(|f| *f == g.format)
                .unwrap_or(0);
            (self.panel_width, self.cols, self.gap) = (g.largeur, g.colonnes, g.ecart);
        }
    }

    /// **La grille que ces réglages décrivent.**
    pub fn grille(&self) -> Grille {
        Grille {
            format: Format::TOUS[self.format_idx.min(Format::TOUS.len() - 1)],
            largeur: self.panel_width,
            colonnes: self.cols.max(1),
            ecart: self.gap,
        }
    }
}

/// **Les cases du tableau actif**, telles que le panneau les montre : l'identifiant, le numéro,
/// la description ; et ce que le storyboard dit pour la lecture (fiche 67).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Contenu {
    pub cases: Vec<(String, i32, String)>,
    pub lecture: Option<Lecture>,
}

impl Contenu {
    pub fn du(store: &Store, choisie: Option<&str>) -> Self {
        let board = &store.project.active_board_id;
        Self {
            cases: store
                .cases(board)
                .into_iter()
                .map(|c| (c.id.clone(), c.order, c.description.clone()))
                .collect(),
            lecture: Lecture::du(store, choisie),
        }
    }
}

/// Une case de la liste : sa ligne, sa description qu'on touche pour l'écrire, sa croix, et
/// son rang dans [`Contenu::cases`].
#[derive(Debug, Clone, PartialEq)]
pub struct Ligne {
    pub ligne: WidgetRect,
    pub description: WidgetRect,
    pub retirer: WidgetRect,
    pub rang: usize,
}

#[derive(Debug, Clone)]
pub struct StoryboardPanelLayout {
    pub format_button: WidgetRect,
    pub width_input: WidgetRect,
    pub cols_input: WidgetRect,
    pub gap_input: WidgetRect,
    /// Les cellules de l'aperçu, et le numéro de chacune.
    pub cells: Vec<WidgetRect>,
    pub activate_button: WidgetRect,
    /// Où s'écrit l'aide du geste, sous le bouton : un clic pose, un glisser trace (TRACE-1).
    pub aide: (f32, f32),
    /// La croix qui efface le storyboard, quand il y en a un.
    pub effacer: Option<WidgetRect>,
    /// « ▶ Lire », la durée par case et la case choisie, quand il y a un storyboard (fiche 67).
    pub lecture: Option<lecture::LectureLayout>,
    /// Le haut de la liste et « + Ajouter », quand il y a des cases.
    pub liste: Option<(f32, WidgetRect)>,
    pub lignes: Vec<Ligne>,
    /// ‹ et ›, quand les cases ne tiennent pas sur une page ; la page montrée, et combien.
    pub pages: Option<(WidgetRect, WidgetRect, usize, usize)>,
    pub renumeroter: Option<WidgetRect>,
}

/// Les cellules de l'aperçu : la grille réglée, au format réglé, dans `zone` — celles qui
/// dépassent sa hauteur ne se montrent pas, comme chez Tauri.
fn cellules(zone: WidgetRect, state: &StoryboardState, nombre: usize, s: f32) -> Vec<WidgetRect> {
    let colonnes = state.cols.max(1);
    let ecart = APERCU.1 * s;
    let largeur = (zone.w - ecart * (colonnes - 1) as f32) / colonnes as f32;
    let hauteur = largeur / state.grille().format.rapport() as f32;
    (0..nombre.clamp(1, CELLULES_MAX))
        .map(|i| {
            let (c, r) = (i % colonnes, i / colonnes);
            WidgetRect::new(
                zone.x + c as f32 * (largeur + ecart),
                zone.y + r as f32 * (hauteur + ecart),
                largeur,
                hauteur,
            )
        })
        .take_while(|cellule| cellule.y + cellule.h <= zone.y + zone.h + 2.0 * s)
        .collect()
}

/// **La largeur, les colonnes, l'espacement** : les trois champs de nombre (CHAMP-1), qui ne
/// dépendent pas des cases — ce que les champs lisent sans connaître le document.
pub fn champs(frame: ScaledRect) -> [WidgetRect; 3] {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (px, py, pw) = (frame.x, frame.y, frame.w);
    let (pad_x, inner_w) = (14.0 * s, pw - 28.0 * s);
    let cy = py + 96.0 * s;
    let col_w = (inner_w - 12.0 * s) / 3.0;
    let column = |i: f32| px + pad_x + i * (col_w + 6.0 * s);
    [0.0, 1.0, 2.0].map(|i| WidgetRect::new(column(i), cy, col_w, 20.0 * s))
}

/// La géométrie du panneau — la seule, lue par le dessin et par le clic.
pub fn layout_storyboard_panel(
    frame: ScaledRect,
    state: &StoryboardState,
    contenu: &Contenu,
) -> StoryboardPanelLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (px, pw) = (frame.x, frame.w);
    let pad_x = 14.0 * s;
    let inner_w = pw - 28.0 * s;
    let format_button = WidgetRect::new(px + pad_x, frame.y + 52.0 * s, inner_w, 24.0 * s);
    let [width_input, cols_input, gap_input] = champs(frame);
    let cy = width_input.y;

    let n = contenu.cases.len();
    let nombre = n.max(state.cols.max(1) * 2);
    let apercu = WidgetRect::new(px + pad_x, cy + 30.0 * s, inner_w, APERCU.0 * s);
    let cells = cellules(apercu, state, nombre, s);
    let bas_apercu = cells.iter().map(|c| c.y + c.h).fold(apercu.y, f32::max);

    let boutons_y = bas_apercu + 10.0 * s;
    let croix_w = if n > 0 { 32.0 * s } else { 0.0 };
    let activate_button = WidgetRect::new(
        px + pad_x,
        boutons_y,
        inner_w - croix_w - if n > 0 { 6.0 * s } else { 0.0 },
        28.0 * s,
    );
    let effacer =
        (n > 0).then(|| WidgetRect::new(px + pw - pad_x - croix_w, boutons_y, croix_w, 28.0 * s));
    let aide = (px + pad_x, boutons_y + 34.0 * s);
    let mut layout = StoryboardPanelLayout {
        format_button,
        width_input,
        cols_input,
        gap_input,
        cells,
        activate_button,
        aide,
        effacer,
        lecture: None,
        liste: None,
        lignes: Vec::new(),
        pages: None,
        renumeroter: None,
    };
    let mut haut = aide.1 + 12.0 * s + 14.0 * s;
    if let Some(l) = &contenu.lecture {
        let (lecture, bas) = lecture::poser(frame, haut - 8.0 * s, l);
        layout.lecture = Some(lecture);
        haut = bas + 24.0 * s;
    }
    if n > 0 {
        poser_la_liste(&mut layout, (frame, haut), state, n);
    }
    layout
}

/// La liste des cases, page par page, entre son titre et « Re-numéroter les panels ».
fn poser_la_liste(
    layout: &mut StoryboardPanelLayout,
    (frame, haut): (ScaledRect, f32),
    state: &StoryboardState,
    n: usize,
) {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (px, py, pw, ph) = (frame.x, frame.y, frame.w, frame.h);
    let (pad_x, inner_w) = (14.0 * s, pw - 28.0 * s);
    let ajouter = WidgetRect::new(
        px + pw - pad_x - 64.0 * s,
        haut - 4.0 * s,
        64.0 * s,
        18.0 * s,
    );
    // « Re-numéroter » sous la dernière ligne montrée, comme chez Tauri — jamais plus bas que
    // le bas du panneau, qui borne le nombre de lignes d'une page.
    let plus_bas = py + ph - pad_x - 24.0 * s;
    let premiere = haut + 22.0 * s;
    let par_page = (((plus_bas - 8.0 * s - premiere) / (LIGNE * s)).floor() as usize).max(1);
    let pages = n.div_ceil(par_page);
    let page = state.page.min(pages - 1);
    let montrees = n.min((page + 1) * par_page) - page * par_page;
    let sous_la_liste = premiere + montrees as f32 * LIGNE * s + 6.0 * s;
    let renumeroter = WidgetRect::new(px + pad_x, sous_la_liste.min(plus_bas), inner_w, 24.0 * s);
    layout.liste = Some((haut, ajouter));
    layout.renumeroter = Some(renumeroter);
    layout.lignes = (page * par_page..(n.min((page + 1) * par_page)))
        .enumerate()
        .map(|(i, rang)| {
            let y = premiere + i as f32 * LIGNE * s;
            let ligne = WidgetRect::new(px + pad_x, y, inner_w, (LIGNE - 4.0) * s);
            let retirer = WidgetRect::new(ligne.x + ligne.w - 20.0 * s, y, 20.0 * s, ligne.h);
            let description = WidgetRect::new(
                ligne.x + 24.0 * s,
                y,
                (retirer.x - ligne.x - 28.0 * s).max(0.0),
                ligne.h,
            );
            Ligne {
                ligne,
                description,
                retirer,
                rang,
            }
        })
        .collect();
    if pages > 1 {
        let suivante = WidgetRect::new(ajouter.x - 24.0 * s, ajouter.y, 18.0 * s, 18.0 * s);
        let precedente = WidgetRect::new(suivante.x - 22.0 * s, ajouter.y, 18.0 * s, 18.0 * s);
        layout.pages = Some((precedente, suivante, page, pages));
    }
}

/// **Ce que le panneau demande** — l'application le fait.
#[derive(Debug, Clone, PartialEq)]
pub enum StoryboardIntent {
    /// Armer la grille : elle suit le curseur jusqu'au clic qui la pose, ou range les cases
    /// existantes (« Mettre à jour »).
    Armer(Grille),
    /// Effacer le storyboard : toutes les cases.
    Effacer,
    /// Ajouter une case à la suite.
    Ajouter(Grille),
    /// Écrire l'un des textes de cette case (fiche 67).
    Ecrire(String, Texte),
    /// Lire le storyboard en plein écran, depuis cette case — ou la première (fiche 67).
    Lire(Option<String>),
    /// Rendre cette case à la durée du storyboard.
    RendreLaDuree(String),
    /// Dire ce plan pour cette case.
    Plan(String, Option<Plan>),
    /// Renommer, ou retirer, le champ de ce rang — ou en ajouter un.
    NommerLeChamp(Option<usize>),
    /// Retirer cette case.
    Retirer(String),
    /// Renuméroter les cases dans l'ordre de lecture.
    Renumeroter,
}

/// **Le clic dans le panneau** : un réglage change ici, une demande part à l'application.
pub fn click_storyboard_panel(
    state: &mut StoryboardState,
    (layout, contenu): (&StoryboardPanelLayout, &Contenu),
    pointer: Pointer,
) -> Option<StoryboardIntent> {
    let hit = |r: WidgetRect| r.contains(pointer.x, pointer.y);
    if hit(layout.format_button) {
        state.format_idx = (state.format_idx + 1) % FORMATS.len();
        return None;
    }
    if hit(layout.activate_button) {
        return Some(StoryboardIntent::Armer(state.grille()));
    }
    if layout.effacer.is_some_and(hit) {
        return Some(StoryboardIntent::Effacer);
    }
    if layout.liste.is_some_and(|(_, ajouter)| hit(ajouter)) {
        return Some(StoryboardIntent::Ajouter(state.grille()));
    }
    if let Some((precedente, suivante, page, pages)) = layout.pages {
        if hit(precedente) {
            state.page = page.saturating_sub(1);
            return None;
        }
        if hit(suivante) {
            state.page = (page + 1).min(pages - 1);
            return None;
        }
    }
    if layout.renumeroter.is_some_and(hit) {
        return Some(StoryboardIntent::Renumeroter);
    }
    if let (Some(l), Some(lecture)) = (&layout.lecture, &contenu.lecture) {
        if let Some(demande) = lecture::cliquer(l, lecture, pointer) {
            return Some(demande);
        }
    }
    let ligne = layout.lignes.iter().find(|l| hit(l.ligne))?;
    let id = contenu.cases.get(ligne.rang)?.0.clone();
    if hit(ligne.retirer) {
        return Some(StoryboardIntent::Retirer(id));
    }
    Some(StoryboardIntent::Ecrire(id, Texte::Description))
}

#[cfg(test)]
mod tests;
