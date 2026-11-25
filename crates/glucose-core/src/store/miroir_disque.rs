//! **Remplir un dossier miroir du disque** (DISQUE-1, fiche 164) : un niveau lu par l'application
//! devient le contenu du tableau du dossier, disposé, et le dossier se dit lu — un seul geste.
//!
//! **Un geste comme les autres**, et non une lecture hors du journal comme `expand_folder` (l'héritage
//! de Tauri) : le document est une histoire de gestes. Ce qui n'y entre pas ne s'enregistre pas,
//! ne se partage pas, ne s'annule pas. Un `Ctrl+Z` juste après l'entrée vide le dossier, qui
//! redevient « à lire » ; y revenir le relit.
//!
//! **La disposition est celle de COLONNES-1** ([`crate::layout::colonnes`]) : les sous-dossiers,
//! les photos, les textes, puis les tuiles, chacun en bloc, dans l'ordre du tri, en colonnes qui
//! approchent la proportion de l'écran. Aucune constante à elle : la grille en croix de Glucose
//! Tauri (des cellules de 220, des écarts de 160, une rangée de 1 500) a disparu dans la loi qui
//! dispose déjà les membranes.

use super::journal::{Edit, Slot};
use super::Store;
use crate::geometry::Rect;
use crate::layout::colonnes::{disposer, Bloc, Marges};
use crate::types::{Annotation, BoardImage, CanvasFolder, FolderMirrorSource};
use std::collections::HashMap;

/// **Un niveau lu**, dans l'ordre du tri : ses sous-dossiers (chacun une source en attente), ses
/// photos, ses textes et ses tuiles. Les positions qu'ils portent ne comptent pas : le niveau se
/// dispose ici.
#[derive(Debug, Default, Clone)]
pub struct Niveau {
    pub dossiers: Vec<CanvasFolder>,
    pub photos: Vec<BoardImage>,
    pub textes: Vec<Annotation>,
    pub tuiles: Vec<Annotation>,
}

impl Store {
    /// **Le dossier miroir où l'on vient d'entrer, s'il attend d'être lu** : son tableau parent,
    /// son identifiant et sa source. Le haut de la pile des dossiers suffit — quelle que soit la
    /// porte par laquelle on est entré (le double-clic, le fil d'Ariane, un retour en arrière).
    pub fn miroir_a_lire(&self) -> Option<(String, String, FolderMirrorSource)> {
        let (parent, id) = self.folder_stack.last()?;
        let source = self
            .project
            .boards
            .iter()
            .find(|b| &b.id == parent)?
            .folders
            .iter()
            .find(|f| &f.id == id)?
            .mirror_source
            .as_ref()
            .filter(|s| s.pending_scan)?;
        Some((parent.clone(), id.clone(), source.clone()))
    }

    /// **Remplit le tableau de ce dossier miroir avec ce niveau**, disposé pour approcher
    /// `proportion` (largeur sur hauteur), et le dit lu à l'heure `maintenant` (secondes depuis
    /// 1970). Un seul geste, ou une part du geste déjà ouvert (le dépôt d'un dossier). Rend
    /// `false` si le dossier n'existe pas ou n'attend aucune lecture : rien ne change.
    pub fn remplir_le_miroir(
        &mut self,
        parent: &str,
        dossier: &str,
        niveau: Niveau,
        proportion: f64,
        maintenant: i64,
    ) -> bool {
        let Some(enfant) = self
            .project
            .boards
            .iter()
            .find(|b| b.id == parent)
            .and_then(|b| b.folders.iter().find(|f| f.id == dossier))
            .filter(|f| f.mirror_source.as_ref().is_some_and(|s| s.pending_scan))
            .map(|f| f.child_board_id.clone())
        else {
            return false;
        };
        let niveau = disposer_le_niveau(niveau, proportion);
        let ouvert = self.journal.is_open();
        if !ouvert {
            self.journal.begin();
        }
        for f in niveau.dossiers {
            self.create_folder_with_content(&enfant, f, Vec::new());
        }
        let noeuds = niveau.textes.into_iter().chain(niveau.tuiles);
        let edits = self.poser_dans(&enfant, niveau.photos, noeuds.collect());
        self.record_as_one_gesture(edits);
        self.update_folder(parent, dossier, |f| {
            if let Some(s) = f.mirror_source.as_mut() {
                s.pending_scan = false;
                s.last_scanned_at = maintenant;
            }
        });
        if !ouvert && self.journal.end() {
            self.bump_version();
        }
        true
    }

    /// Pose ces photos et ces nœuds dans ce tableau, sans toucher à la sélection, et rend ce qui
    /// s'est fait.
    fn poser_dans(
        &mut self,
        board: &str,
        photos: Vec<BoardImage>,
        noeuds: Vec<Annotation>,
    ) -> Vec<Edit> {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board) else {
            return Vec::new();
        };
        let mut edits = Vec::with_capacity(photos.len() + noeuds.len());
        for img in photos {
            edits.push(Edit::Image {
                board: board.to_string(),
                slot: Slot::inserted(b.images.len(), img.clone()),
            });
            b.images.push(img);
        }
        for ann in noeuds {
            edits.push(Edit::Annotation {
                board: board.to_string(),
                slot: Slot::inserted(b.annotations.len(), ann.clone()),
            });
            b.annotations.push(ann);
        }
        edits
    }
}

/// **Dispose un niveau** : un bloc par sorte, non vide, dans l'ordre du niveau ; chaque nœud reçoit
/// sa place, l'origine au coin haut-gauche du tableau.
pub fn disposer_le_niveau(mut niveau: Niveau, proportion: f64) -> Niveau {
    let feuille = |id: &str, largeur: f64, hauteur: f64| Bloc {
        id: id.to_string(),
        largeur,
        hauteur,
        enfants: Vec::new(),
        titree: false,
    };
    let groupes: [Vec<Bloc>; 4] = [
        niveau
            .dossiers
            .iter()
            .map(|f| feuille(&f.id, f.width, f.height))
            .collect(),
        niveau
            .photos
            .iter()
            .map(|i| feuille(&i.id, i.width, i.height))
            .collect(),
        blocs_de(&niveau.textes, feuille),
        blocs_de(&niveau.tuiles, feuille),
    ];
    let racine = Bloc {
        enfants: groupes
            .into_iter()
            .enumerate()
            .filter(|(_, g)| !g.is_empty())
            // Un identifiant qu'aucun nœud ne porte : un caractère nul ne s'écrit dans aucun.
            .map(|(k, enfants)| Bloc {
                enfants,
                ..feuille(&format!("\0{k}"), 0.0, 0.0)
            })
            .collect(),
        ..feuille("\0", 0.0, 0.0)
    };
    if racine.enfants.is_empty() {
        return niveau;
    }
    let proportion = if proportion.is_finite() && proportion > 0.0 {
        proportion
    } else {
        1.0
    };
    // Les marges d'une membrane sans titre : l'écart de Glucose partout.
    let ecart = super::zones::ECART;
    let places: HashMap<String, Rect> = disposer(
        &racine,
        proportion,
        Marges {
            ecart,
            titre: ecart,
        },
    )
    .into_iter()
    .collect();
    for f in &mut niveau.dossiers {
        if let Some(r) = places.get(&f.id) {
            (f.x, f.y) = (r.left, r.top);
        }
    }
    // Une photo est ancrée en son centre (R-11).
    for i in &mut niveau.photos {
        if let Some(r) = places.get(&i.id) {
            (i.x, i.y) = (r.left + r.width / 2.0, r.top + r.height / 2.0);
        }
    }
    for a in niveau.textes.iter_mut().chain(niveau.tuiles.iter_mut()) {
        if let (Some(r), Some(ici)) = (places.get(a.id()), a.rect()) {
            a.translate(r.left - ici.left, r.top - ici.top);
        }
    }
    niveau
}

fn blocs_de(noeuds: &[Annotation], feuille: impl Fn(&str, f64, f64) -> Bloc) -> Vec<Bloc> {
    noeuds
        .iter()
        .filter_map(|a| {
            let r = a.rect()?;
            Some(feuille(a.id(), r.width, r.height))
        })
        .collect()
}

#[cfg(test)]
mod tests;
