//! **MEMB-8 — ce qu'un agent pose « dans » une membrane lui appartient** (fiche 97).
//!
//! À sa naissance, un nœud appartient à la plus petite membrane qui contient son centre (MEMB-1) :
//! la règle de la main, qui dépose là où elle lâche. Un agent, lui, **dit** où il pose. Les deux
//! s'accordaient tant que les membranes ne se chevauchaient pas ; mais un article qui grandit
//! (MEMB-7) recouvre ses voisins sans les pousser — un lieu appris ne bouge pas en silence — et le
//! squelette de Wikipédia l'a montré : les sections d'« Eau » finissaient dans un autre article,
//! celles d'« Hydrogène » au fond d'« Eau ». VISION le disait déjà : le sens d'un tableau ne tient
//! jamais **seulement** dans la position ; ce qui compte s'écrit dans une structure explicite. Ce
//! que l'agent a nommé l'emporte donc sur la géométrie ; sans nom, la naissance décide.

use super::{Refus, Sorte};
use crate::store::Store;
use crate::types::Annotation;

impl Store {
    /// **Le nœud `id` appartient à `dans`**, s'il est nommé. Refuse ce qui n'est pas une membrane,
    /// et une membrane qu'on poserait dans sa propre descendance. Rien ne s'écrit si le nœud y est
    /// déjà.
    pub(super) fn rattacher(
        &mut self,
        board_id: &str,
        id: &str,
        dans: Option<&str>,
    ) -> Result<(), Refus> {
        let Some(m) = dans else {
            return Ok(());
        };
        match self.project.annotation(board_id, m) {
            Some(Annotation::Membrane { .. }) => {}
            Some(a) => {
                let est = Sorte::de(a).nom();
                return Err(Refus::PasUneMembrane {
                    id: m.to_string(),
                    est,
                });
            }
            None => return Err(Refus::NoeudInconnu(m.to_string())),
        }
        // Remonter de `dans` vers la racine : y rencontrer le nœud ferait un cycle. Un cycle déjà
        // présent dans le document ne fait pas tourner sans fin.
        let mut vues: Vec<String> = Vec::new();
        let mut courant = Some(m.to_string());
        while let Some(c) = courant.filter(|c| !vues.contains(c)) {
            if c == id {
                return Err(Refus::Emboitement {
                    membrane: id.to_string(),
                    dans: m.to_string(),
                });
            }
            courant = self
                .project
                .annotation(board_id, &c)
                .and_then(Annotation::membrane_id)
                .map(str::to_string);
            vues.push(c);
        }
        let voulu = Some(m.to_string());
        if let Some(a) = self.project.annotation(board_id, id) {
            if a.membrane_id() != Some(m) {
                self.update_annotation(board_id, id, |a| a.set_membrane_id(voulu));
            }
        } else if let Some(i) = self.image(board_id, id) {
            if i.membrane_id.as_deref() != Some(m) {
                self.update_image(board_id, id, |i| i.membrane_id = voulu);
            }
        }
        Ok(())
    }
}
