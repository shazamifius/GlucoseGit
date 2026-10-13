//! **Le storyboard d'un tableau** (STORYBOARD-1, fiche 64) : des cases numérotées posées en
//! grille, une description sous chacune, et une image ajustée dans chacune.
//!
//! # Ce que Glucose Tauri faisait, et ce qui lui manquait
//!
//! Son panneau posait une grille de `colonnes × 2` cases, centrée là où l'on cliquait, et
//! « Mettre à jour » rangeait les cases existantes à la nouvelle grille ; chaque case portait un
//! numéro et une description. Mais une case ne pouvait **rien contenir** — son `imageId` n'était
//! jamais écrit —, ni se prendre, ni se déplacer ; et « Re-numéroter » numérotait dans l'ordre
//! de création, que rien ne montre (`StoryboardControls.tsx`, `StoryboardLayer.ts`).
//!
//! # Ce que font les outils du métier
//!
//! Boords renumérote seul quand on réordonne ; Storyboarder et Boords mettent le numéro et les
//! notes **sous** l'image ; Milanote fait glisser une image dans une colonne. Ici :
//!
//! * une image posée dans une case **s'y ajuste** — contenue dans le cadre, centrée, à ses
//!   proportions. Rien ne s'écrit sur la case : l'image appartient à la case où son centre est
//!   posé, comme à une zone (PRESET-1) ;
//! * renuméroter suit l'**ordre de lecture** — par rangée, de haut en bas, puis de gauche à
//!   droite —, celui que l'œil suit ;
//! * chaque geste est **un** geste du journal : une grille de huit cases s'annule d'un coup.

use super::journal::{Edit, Slot};
use super::Store;
use crate::types::StoryboardPanel;

/// La hauteur de la description sous chaque case, en unités du monde — celle de Glucose Tauri
/// (`StoryboardLayer.ts`, `descH`). Le dessin, le fantôme et la prise d'une case la lisent.
pub const DESCRIPTION: f64 = 40.0;

/// **Les cinq formats de Glucose Tauri**, dans l'ordre de son panneau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Cinema,
    Classique,
    Scope,
    Carre,
    Vertical,
}

impl Format {
    pub const TOUS: [Format; 5] = [
        Format::Cinema,
        Format::Classique,
        Format::Scope,
        Format::Carre,
        Format::Vertical,
    ];

    /// La largeur sur la hauteur d'une case.
    pub fn rapport(self) -> f64 {
        match self {
            Self::Cinema => 16.0 / 9.0,
            Self::Classique => 4.0 / 3.0,
            Self::Scope => 2.35,
            Self::Carre => 1.0,
            Self::Vertical => 9.0 / 16.0,
        }
    }
}

/// **La grille d'un storyboard** : le format des cases, leur largeur, les colonnes, l'écart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grille {
    pub format: Format,
    pub largeur: f64,
    pub colonnes: usize,
    pub ecart: f64,
}

impl Grille {
    /// La hauteur d'une case : sa largeur sur son rapport.
    pub fn hauteur(&self) -> f64 {
        self.largeur / self.format.rapport()
    }

    /// Combien de colonnes, au moins une.
    fn colonnes(&self) -> usize {
        self.colonnes.max(1)
    }

    /// Combien de cases une grille neuve porte : deux rangées, comme dans Tauri.
    pub fn nombre_initial(&self) -> usize {
        self.colonnes() * 2
    }

    /// **Le coin de chaque case d'une grille de `nombre` cases centrée sur `centre`**, dans
    /// l'ordre de lecture. Le fantôme qui suit le curseur et le clic qui pose la lisent tous
    /// deux : la grille se pose là où on l'a vue.
    pub fn coins(&self, nombre: usize, centre: (f64, f64)) -> Vec<(f64, f64)> {
        let colonnes = self.colonnes();
        let rangees = nombre.div_ceil(colonnes);
        let (pas_x, pas_y) = (
            self.largeur + self.ecart,
            self.hauteur() + DESCRIPTION + self.ecart,
        );
        let largeur = colonnes.min(nombre.max(1)) as f64 * pas_x - self.ecart;
        let hauteur = rangees as f64 * pas_y - self.ecart;
        let (gauche, haut) = (centre.0 - largeur / 2.0, centre.1 - hauteur / 2.0);
        (0..nombre)
            .map(|i| {
                let (colonne, rangee) = (i % colonnes, i / colonnes);
                (
                    gauche + colonne as f64 * pas_x,
                    haut + rangee as f64 * pas_y,
                )
            })
            .collect()
    }
}

/// **L'ordre de lecture** de ces cases : par rangée, de haut en bas, puis de gauche à droite.
///
/// Une rangée commence à la plus haute des cases restantes, et prend toutes celles dont le
/// milieu tombe dans sa hauteur : une case glissée un peu plus bas que ses voisines reste dans
/// leur rangée. Rend les rangs dans la liste, dans l'ordre.
pub fn ordre_de_lecture(cases: &[StoryboardPanel]) -> Vec<usize> {
    let mut restantes: Vec<usize> = (0..cases.len()).collect();
    restantes.sort_by(|&a, &b| cases[a].y.total_cmp(&cases[b].y));
    let mut ordre = Vec::with_capacity(cases.len());
    while let Some(&premiere) = restantes.first() {
        let bas = cases[premiere].y + cases[premiere].height;
        let (mut rangee, reste): (Vec<usize>, Vec<usize>) = restantes
            .into_iter()
            .partition(|&i| cases[i].y + cases[i].height / 2.0 <= bas);
        rangee.sort_by(|&a, &b| cases[a].x.total_cmp(&cases[b].x));
        ordre.extend(rangee);
        restantes = reste;
    }
    ordre
}

impl Store {
    /// Les cases de ce tableau, dans l'ordre de leurs numéros.
    pub fn cases(&self, board_id: &str) -> Vec<&StoryboardPanel> {
        let mut cases: Vec<&StoryboardPanel> = self
            .project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .map(|b| b.panels.iter().collect())
            .unwrap_or_default();
        cases.sort_by_key(|c| c.order);
        cases
    }

    /// **Pose le storyboard, centré sur `centre`** : une grille neuve de deux rangées sur un
    /// tableau qui n'en a pas ; sinon, les cases existantes rangées à cette grille, dans l'ordre
    /// de leurs numéros, renumérotées de un en un — le « Mettre à jour » de Tauri. Un geste.
    pub fn poser_le_storyboard(&mut self, board_id: &str, grille: Grille, centre: (f64, f64)) {
        let existantes: Vec<StoryboardPanel> = self.cases(board_id).into_iter().cloned().collect();
        let coins = self.coins_a_poser(board_id, grille, centre);
        let nombre = coins.len();
        let hauteur = grille.hauteur();
        let mut neuves = Vec::with_capacity(nombre);
        for (i, (x, y)) in coins.into_iter().enumerate() {
            let mut case = match existantes.get(i) {
                Some(c) => c.clone(),
                None => StoryboardPanel::new(self.generate_id("case"), 0, 0.0, 0.0, 0.0, 0.0),
            };
            (case.order, case.x, case.y) = (i as i32, x, y);
            (case.width, case.height) = (grille.largeur, hauteur);
            neuves.push(case);
        }
        self.remplacer_les_cases(board_id, neuves);
    }

    /// **Les coins des cases que poser le storyboard ici donnerait** : deux rangées sur un
    /// tableau qui n'en a pas, sinon autant que de cases. Le fantôme qui suit le curseur et le
    /// clic qui pose la lisent tous deux : la grille se pose là où on l'a vue.
    pub fn coins_a_poser(
        &self,
        board_id: &str,
        grille: Grille,
        centre: (f64, f64),
    ) -> Vec<(f64, f64)> {
        let n = self.cases(board_id).len();
        let nombre = if n == 0 { grille.nombre_initial() } else { n };
        grille.coins(nombre, centre)
    }

    /// **Ajoute une case après la dernière**, à sa place dans la grille de la première — le
    /// « + Ajouter » de Tauri. Sur un tableau sans storyboard, rien.
    pub fn ajouter_une_case(&mut self, board_id: &str, grille: Grille) -> Option<String> {
        let cases: Vec<StoryboardPanel> = self.cases(board_id).into_iter().cloned().collect();
        let premiere = cases.first()?;
        let colonnes = grille.colonnes();
        let n = cases.len();
        let (pas_x, pas_y) = (
            grille.largeur + grille.ecart,
            grille.hauteur() + DESCRIPTION + grille.ecart,
        );
        let id = self.generate_id("case");
        let case = StoryboardPanel::new(
            id.clone(),
            n as i32,
            premiere.x + (n % colonnes) as f64 * pas_x,
            premiere.y + (n / colonnes) as f64 * pas_y,
            grille.largeur,
            grille.hauteur(),
        );
        let mut toutes = cases;
        toutes.push(case);
        self.remplacer_les_cases(board_id, toutes);
        Some(id)
    }

    /// **Retire une case**, et les suivantes prennent un numéro de moins : la suite reste
    /// continue. Un geste.
    pub fn retirer_la_case(&mut self, board_id: &str, id: &str) {
        let mut cases: Vec<StoryboardPanel> = self
            .cases(board_id)
            .into_iter()
            .filter(|c| c.id != id)
            .cloned()
            .collect();
        for (i, c) in cases.iter_mut().enumerate() {
            c.order = i as i32;
        }
        self.remplacer_les_cases(board_id, cases);
    }

    /// **Efface le storyboard** : toutes les cases, en un geste. Les images restent.
    pub fn effacer_le_storyboard(&mut self, board_id: &str) {
        self.remplacer_les_cases(board_id, Vec::new());
    }

    /// **Renumérote les cases dans l'ordre de lecture** ([`ordre_de_lecture`]). Un geste.
    pub fn renumeroter_les_cases(&mut self, board_id: &str) {
        let cases: Vec<StoryboardPanel> = self.cases(board_id).into_iter().cloned().collect();
        let mut rangees: Vec<StoryboardPanel> = ordre_de_lecture(&cases)
            .into_iter()
            .map(|i| cases[i].clone())
            .collect();
        for (i, c) in rangees.iter_mut().enumerate() {
            c.order = i as i32;
        }
        self.remplacer_les_cases(board_id, rangees);
    }

    /// **Pose cette case à ce coin** — pendant un glisser, dans le geste ouvert.
    pub fn placer_la_case(&mut self, board_id: &str, id: &str, (x, y): (f64, f64)) {
        let mut cases: Vec<StoryboardPanel> = self.cases(board_id).into_iter().cloned().collect();
        if let Some(c) = cases.iter_mut().find(|c| c.id == id) {
            (c.x, c.y) = (x, y);
        }
        self.remplacer_les_cases(board_id, cases);
    }

    /// **Glisse une case et les images qu'elle porte** à `total` de leur départ, dans le geste
    /// ouvert — en deux éditions, quelle que soit la durée du geste : la case, **réécrite à sa
    /// place** dans le geste (FONDRE-1 ne fond que des éditions voisines, et les pas des images
    /// les séparent), puis ses images par le glisser (GLISSER-1), dont les pas deviennent une
    /// translation à la fin. Défaire rend les images, puis la case. Hors geste, rien.
    pub fn glisser_la_case(
        &mut self,
        board_id: &str,
        (id, depart): (&str, (f64, f64)),
        images: &[String],
        total: (f64, f64),
    ) {
        let Some((_, edits)) = self.journal.en_cours() else {
            return;
        };
        let deja = edits.iter().enumerate().find_map(|(rang, e)| match e {
            Edit::Panel { board, slot } if board == board_id => slot
                .after
                .as_ref()
                .filter(|c| c.id == id)
                .and(slot.before.as_deref().cloned())
                .map(|avant| (rang, avant)),
            _ => None,
        });
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let Some(index) = b.panels.iter().position(|c| c.id == id) else {
            return;
        };
        let actuelle = b.panels[index].clone();
        let mut neuve = actuelle.clone();
        (neuve.x, neuve.y) = (depart.0 + total.0, depart.1 + total.1);
        b.panels[index] = neuve.clone();
        let board = board_id.to_string();
        match deja {
            Some((rang, avant)) => {
                let slot = Slot::changed(index, avant, neuve);
                let edit = Edit::Panel { board, slot };
                self.journal
                    .remplacer_dans_le_geste(rang..rang + 1, Some(edit));
            }
            None => {
                let slot = Slot::changed(index, actuelle, neuve);
                self.journal.record(Edit::Panel { board, slot });
            }
        }
        self.glisser_ces_images(board_id, images, total);
    }

    /// **Les images que cette case porte** : celles dont le centre tombe dans son cadre.
    pub fn images_de_la_case(&self, board_id: &str, id: &str) -> Vec<String> {
        let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) else {
            return Vec::new();
        };
        let Some(c) = b.panels.iter().find(|c| c.id == id) else {
            return Vec::new();
        };
        let dedans = |x: f64, y: f64| {
            (c.x..=c.x + c.width).contains(&x) && (c.y..=c.y + c.height).contains(&y)
        };
        b.images
            .iter()
            .filter(|i| dedans(i.x, i.y))
            .map(|i| i.id.clone())
            .collect()
    }

    /// **La case dont le cadre contient ce point** — la plus haute, la dernière posée, si deux se
    /// recouvrent. La description, sous le cadre, n'en est pas.
    pub fn case_sous(&self, board_id: &str, (x, y): (f64, f64)) -> Option<&StoryboardPanel> {
        self.cases(board_id)
            .into_iter()
            .rev()
            .find(|c| (c.x..=c.x + c.width).contains(&x) && (c.y..=c.y + c.height).contains(&y))
    }

    /// **Les images posées dans une case s'y ajustent** : parmi `ids`, chaque image non
    /// verrouillée dont le centre tombe dans le cadre d'une case y est contenue, centrée, à ses
    /// proportions. Dans le geste ouvert, s'il y en a un.
    pub fn ajuster_dans_les_cases(&mut self, board_id: &str, ids: &[String]) {
        for id in ids {
            let Some(image) = self.image(board_id, id) else {
                continue;
            };
            if image.locked || image.width <= 0.0 || image.height <= 0.0 {
                continue;
            }
            let Some(case) = self.case_sous(board_id, (image.x, image.y)).cloned() else {
                continue;
            };
            let s = (case.width / image.width).min(case.height / image.height);
            self.update_image(board_id, id, |img| {
                (img.width, img.height) = (img.width * s, img.height * s);
                img.x = case.x + case.width / 2.0;
                img.y = case.y + case.height / 2.0;
            });
        }
    }

    /// **Remplace les cases du tableau**, et consigne chaque case qui change, entre, ou part —
    /// le tout en un geste.
    fn remplacer_les_cases(&mut self, board_id: &str, neuves: Vec<StoryboardPanel>) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let avant = std::mem::replace(&mut b.panels, neuves);
        let apres = b.panels.clone();
        let edits = diff_des_cases(board_id, &avant, &apres);
        self.record_as_one_gesture(edits);
    }
}

/// **Ce qui change d'une liste de cases à l'autre**, en éditions qui se rejouent dans l'ordre.
///
/// Quand la liste garde ses cases aux mêmes rangs — un glisser, une description, un rangement —
/// seules celles qui changent sont consignées : un glisser n'écrit pas toutes les cases à chaque
/// image. Sinon, on retire l'ancien de la fin vers le début, puis on insère le neuf.
fn diff_des_cases(board: &str, avant: &[StoryboardPanel], apres: &[StoryboardPanel]) -> Vec<Edit> {
    let memes_rangs =
        avant.len() == apres.len() && avant.iter().zip(apres).all(|(a, b)| a.id == b.id);
    if memes_rangs {
        return avant
            .iter()
            .zip(apres)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (a, b))| Edit::Panel {
                board: board.to_string(),
                slot: Slot::changed(i, a.clone(), b.clone()),
            })
            .collect();
    }
    let retraits = avant.iter().enumerate().rev().map(|(i, c)| Edit::Panel {
        board: board.to_string(),
        slot: Slot::removed(i, c.clone()),
    });
    let ajouts = apres.iter().enumerate().map(|(i, c)| Edit::Panel {
        board: board.to_string(),
        slot: Slot::inserted(i, c.clone()),
    });
    retraits.chain(ajouts).collect()
}

#[cfg(test)]
mod tests;
