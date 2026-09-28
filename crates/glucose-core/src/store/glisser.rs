//! **GLISSER-1 — un glisser s'écrit en une seule translation**, et le fichier rejoue l'écran au
//! bit près.
//!
//! # Ce qu'un glisser écrivait
//!
//! Chaque mouvement de la main déplaçait la sélection **d'un pas**, et chaque pas entrait dans
//! le geste : cinq secondes de glisser à deux cent quarante mouvements par seconde, un millier
//! de translations dans la pile d'annulation et dans l'histoire du fichier (registre de Tauri,
//! 12). Et chaque pas reparcourait le tableau entier pour retrouver ce que la sélection
//! emporte.
//!
//! # Pourquoi on ne les additionne pas après coup
//!
//! Une translation porte un **pas**, pas une valeur. Additionner des pas en virgule flottante
//! ne redonne pas la position qu'ils ont laissée : `(x + a) + b` n'est pas toujours `x + (a +
//! b)`. Fondre les pas d'un glisser en une translation de leur somme aurait écrit un fichier
//! qui, relu, pose les nœuds un ulp à côté de l'écran — et l'épreuve fondatrice de l'histoire
//! (« le fichier relu redonne exactement le document ») serait tombée.
//!
//! # La règle : départ plus déplacement total
//!
//! Le glisser relève **une fois**, au premier mouvement, ce que la sélection emporte et d'où
//! chaque chose part. À chaque mouvement, chaque position devient son départ plus le
//! déplacement **total** du geste — l'addition même que la translation du journal fera en
//! rejouant. À la fermeture du geste, ses pas se remplacent par **une** translation, du départ
//! à l'arrivée : rejouée, elle redonne l'écran exactement. Un glisser revenu à son point de
//! départ ne laisse rien, et `Échap` rend chaque départ exact.
//!
//! Les pas restent écrits **pendant** le geste : ce qui suit le geste en cours — l'écran, les
//! itinéraires des flèches (GESTE-1, FLECHE-5) — les lit pour savoir ce qui a bougé.

use super::journal::{Bouts, Edit};
use super::Store;
use crate::types::{Annotation, Board};

/// Un glisser en cours : ce qu'il emporte, d'où chaque chose part, et où il en est.
#[derive(Debug, Clone)]
pub struct Glisser {
    /// Le geste du journal où il vit, et son tableau.
    geste: u64,
    board: String,
    /// Ce qu'il emporte, par rangs (JRN-4).
    images: Vec<u32>,
    annotations: Vec<u32>,
    folders: Vec<u32>,
    bouts: Vec<(u32, Bouts)>,
    /// D'où chaque chose part, dans l'ordre de ses rangs.
    depart_images: Vec<(f64, f64)>,
    depart_annotations: Vec<Annotation>,
    depart_dossiers: Vec<(f64, f64)>,
    depart_bouts: Vec<[f64; 4]>,
    /// Le déplacement total, depuis le départ.
    total: (f64, f64),
    /// Le rang de son premier pas dans le geste ouvert, et combien il en a écrit.
    premier: usize,
    pas: usize,
}

impl Glisser {
    /// Relève ce que la sélection emporte et d'où chaque chose part — le seul parcours du
    /// tableau de tout le geste.
    fn prendre(store: &Store, board_id: &str, geste: u64, premier: usize) -> Option<Self> {
        let Edit::Translation {
            images,
            annotations,
            folders,
            bouts,
            ..
        } = store.translation_de_la_selection(board_id, (0.0, 0.0))?
        else {
            return None;
        };
        let b = store.project.boards.iter().find(|b| b.id == board_id)?;
        let depart_bouts = bouts
            .iter()
            .map(|&(i, _)| match &b.annotations[i as usize] {
                Annotation::Arrow { x, y, x2, y2, .. } => [*x, *y, *x2, *y2],
                _ => [0.0; 4],
            })
            .collect();
        Some(Self {
            geste,
            board: board_id.to_string(),
            depart_images: images
                .iter()
                .map(|&i| (b.images[i as usize].x, b.images[i as usize].y))
                .collect(),
            depart_annotations: annotations
                .iter()
                .map(|&i| b.annotations[i as usize].clone())
                .collect(),
            depart_dossiers: folders
                .iter()
                .map(|&i| (b.folders[i as usize].x, b.folders[i as usize].y))
                .collect(),
            depart_bouts,
            images,
            annotations,
            folders,
            bouts,
            total: (0.0, 0.0),
            premier,
            pas: 0,
        })
    }

    /// La translation de ce qu'il emporte, de ce vecteur.
    fn translation(&self, delta: (f64, f64)) -> Edit {
        Edit::Translation {
            board: self.board.clone(),
            delta,
            images: self.images.clone(),
            annotations: self.annotations.clone(),
            folders: self.folders.clone(),
            bouts: self.bouts.clone(),
        }
    }

    /// **Pose chaque chose à son départ plus `(dx, dy)`** — l'addition même de la translation.
    fn poser(&self, board: &mut Board, (dx, dy): (f64, f64)) {
        for (&i, &(x, y)) in self.images.iter().zip(&self.depart_images) {
            let img = &mut board.images[i as usize];
            img.x = x + dx;
            img.y = y + dy;
        }
        for (&i, depart) in self.annotations.iter().zip(&self.depart_annotations) {
            board.annotations[i as usize].placer_depuis(depart, (dx, dy));
        }
        for (&i, &(x, y)) in self.folders.iter().zip(&self.depart_dossiers) {
            let f = &mut board.folders[i as usize];
            f.x = x + dx;
            f.y = y + dy;
        }
        for (&(i, quels), d) in self.bouts.iter().zip(&self.depart_bouts) {
            if let Annotation::Arrow { x, y, x2, y2, .. } = &mut board.annotations[i as usize] {
                if quels.origine {
                    (*x, *y) = (d[0] + dx, d[1] + dy);
                }
                if quels.cible {
                    (*x2, *y2) = (d[2] + dx, d[3] + dy);
                }
            }
        }
    }

    /// Ses pas sont-ils encore, tels quels, dans ce geste ?
    fn vit_dans(&self, geste: u64, ecrits: usize) -> bool {
        self.geste == geste && self.pas > 0 && self.premier + self.pas <= ecrits
    }
}

impl Store {
    /// **Glisse la sélection à `total` de là où le geste l'a prise** (GLISSER-1).
    ///
    /// Hors geste, c'est un déplacement ordinaire, qui est un geste à lui seul.
    pub fn glisser_la_selection(&mut self, board_id: &str, total: (f64, f64)) {
        let Some((geste, ecrits)) = self.journal.en_cours().map(|(n, e)| (n, e.len())) else {
            self.move_selected(board_id, total.0, total.1);
            return;
        };
        let suit = self.glisser.as_ref().is_some_and(|g| {
            g.geste == geste && g.board == board_id && g.premier + g.pas == ecrits
        });
        if !suit {
            // Un autre glisser dans le même geste se ferme d'abord : ses pas deviennent sa
            // translation, et le nouveau part de là où l'ancien a laissé les choses.
            self.clore_le_glisser();
            let ecrits = self.journal.en_cours().map_or(0, |(_, e)| e.len());
            self.glisser = Glisser::prendre(self, board_id, geste, ecrits);
        }
        let (Some(g), Some(board)) = (
            self.glisser.as_mut(),
            self.project.boards.iter_mut().find(|b| b.id == board_id),
        ) else {
            return;
        };
        let pas = (total.0 - g.total.0, total.1 - g.total.1);
        let pas_edit = g.translation(pas);
        if pas_edit.is_noop() {
            return;
        }
        g.poser(board, total);
        g.total = total;
        g.pas += 1;
        self.journal.record(pas_edit);
    }

    /// **Ferme le glisser en cours** : ses pas deviennent une seule translation, du départ à
    /// l'arrivée — ou rien, s'il est revenu à son point de départ.
    pub(super) fn clore_le_glisser(&mut self) {
        let Some(g) = self.glisser.take() else {
            return;
        };
        let Some((geste, ecrits)) = self.journal.en_cours().map(|(n, e)| (n, e.len())) else {
            return;
        };
        if !g.vit_dans(geste, ecrits) {
            return;
        }
        let totale = g.translation(g.total);
        let par = (!totale.is_noop()).then_some(totale);
        self.journal
            .remplacer_dans_le_geste(g.premier..g.premier + g.pas, par);
    }

    /// **Abandonne le glisser en cours** (`Échap`) : chaque chose retourne **exactement** d'où
    /// elle est partie, et ses pas quittent le geste — que le journal défait ensuite.
    ///
    /// Quand d'autres éditions suivent ses pas, elles doivent se défaire avant lui, dans l'ordre
    /// du journal : ses pas deviennent alors sa translation, et le journal défait le tout.
    pub(super) fn abandonner_le_glisser(&mut self) {
        let Some(g) = self.glisser.take() else {
            return;
        };
        let Some((geste, ecrits)) = self.journal.en_cours().map(|(n, e)| (n, e.len())) else {
            return;
        };
        if !g.vit_dans(geste, ecrits) {
            return;
        }
        let rangs = g.premier..g.premier + g.pas;
        if rangs.end < ecrits {
            let totale = g.translation(g.total);
            self.journal.remplacer_dans_le_geste(rangs, Some(totale));
            return;
        }
        if let Some(board) = self.project.boards.iter_mut().find(|b| b.id == g.board) {
            g.poser(board, (0.0, 0.0));
        }
        self.journal.remplacer_dans_le_geste(rangs, None);
    }
}

#[cfg(test)]
#[path = "glisser_tests.rs"]
mod tests;
