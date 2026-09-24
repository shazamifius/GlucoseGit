//! **Le filet de la frappe** : ce qu'une carte en édition contient ne se perd pas dans un
//! arrêt brutal.
//!
//! Une saisie entière est un seul geste, écrit quand elle se valide. Avant cela, son texte ne
//! vivait qu'en mémoire : dix minutes de note disparaissaient avec la fenêtre. À chaque image
//! où il a changé, il est maintenant confié au scribe, qui le garde à côté du document
//! ([`glucose_core::persist::histoire::saisie`]) ; rouvrir le document le rend, par le même
//! chemin qu'une saisie validée — un geste, que `Ctrl+Z` retire.

use crate::app::GlucoseApp;
use glucose_core::persist::histoire::{saisie, Saisie};
use glucose_core::types::{Annotation, Project};
use std::path::{Path, PathBuf};

impl GlucoseApp {
    /// Confie au scribe le texte de la carte en édition, s'il a changé depuis l'image
    /// précédente — ou l'oubli de ce texte, quand la saisie s'est refermée.
    ///
    /// Un document sans fichier en reçoit un dès que la frappe y change quelque chose : un
    /// texte tapé est du travail, comme un geste.
    pub(crate) fn garder_la_saisie(&mut self) {
        if self.disque.ecriture.is_none() {
            if !self.la_frappe_a_change() {
                return;
            }
            if let Err(e) = self.s_assurer_d_un_fichier() {
                self.dire_l_echec(&e);
                return;
            }
        }
        let Some(e) = self.disque.ecriture.as_mut() else {
            return;
        };
        let en_cours = self
            .editing_session
            .as_ref()
            .map(|s| (s.ann_id.as_str(), s.buffer.as_str()));
        e.saisir(&self.store.project.active_board_id, en_cours);
    }

    /// La carte en édition dit-elle autre chose que le document ?
    fn la_frappe_a_change(&self) -> bool {
        self.editing_session.as_ref().is_some_and(|s| {
            let p = &self.store.project;
            texte_porte(p, &p.active_board_id, &s.ann_id) != Some(s.buffer.as_str())
        })
    }

    /// **Rend au document le texte qu'un arrêt a laissé en cours de frappe** : la saisie se
    /// rouvre sur sa carte et se valide, exactement comme si l'utilisateur avait cliqué
    /// ailleurs. Rend `false` si le document le portait déjà.
    pub(crate) fn rendre_la_saisie(&mut self, s: Saisie) -> bool {
        let porte = texte_porte(&self.store.project, &s.tableau, &s.annotation).map(str::to_owned);
        match porte {
            Some(t) if t == s.texte => return false,
            Some(_) => {
                // On revient là où l'on tapait : c'est là que le texte doit se voir.
                let _ = self.store.try_set_active_board_id(s.tableau);
                self.start_text_edit(s.annotation, s.texte);
                self.commit_editing();
            }
            None => {
                // La carte a disparu — ce qui ne devrait pas arriver : rien ne change le
                // document pendant qu'on tape. Le texte ne se perd pas pour autant : il revient
                // dans une carte neuve, au centre de ce qu'on regarde.
                let (x, y) = self.drop_origin(None);
                let id = self.store.generate_id("text");
                let carte = crate::interactions::tools::text_card(
                    &self.renderer.typography,
                    &self.renderer.math,
                    &id,
                    x,
                    y,
                    &s.texte,
                );
                let tableau = self.store.project.active_board_id.clone();
                self.store.add_annotation(&tableau, carte);
            }
        }
        true
    }
}

/// Le texte qu'une annotation porte dans le document, tel qu'une saisie l'écrit.
fn texte_porte<'a>(p: &'a Project, tableau: &str, annotation: &str) -> Option<&'a str> {
    Some(match p.annotation(tableau, annotation)? {
        Annotation::Text { text, .. } | Annotation::Sticky { text, .. } => text,
        Annotation::Membrane { text, .. } | Annotation::Arrow { text, .. } => {
            text.as_deref().unwrap_or("")
        }
    })
}

/// Le document nommé qu'une saisie attend, s'il faut le rouvrir au lancement.
///
/// Un brouillon se rouvre de lui-même, et un document tenu par une autre fenêtre est en
/// train d'être tapé. Une saisie illisible, ou dont le document n'existe plus, s'efface : il
/// n'y a plus rien où la rendre.
pub(super) fn document_d_une_saisie(fichier: &Path, brouillons: &Path) -> Option<PathBuf> {
    let lue = std::fs::read(fichier).ok().and_then(|o| saisie::lire(&o));
    match lue.map(|(_, s)| PathBuf::from(s.document)) {
        Some(d) if d.is_file() => {
            (!d.starts_with(brouillons) && !super::verrou::tenu_ailleurs(&d)).then_some(d)
        }
        _ => {
            let _ = std::fs::remove_file(fichier);
            None
        }
    }
}

#[cfg(test)]
mod tests;
