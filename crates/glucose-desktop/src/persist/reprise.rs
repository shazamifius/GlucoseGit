//! **Ce qui se rouvre au lancement** : on reprend là où l'on s'était arrêté.
//!
//! # Pourquoi pas seulement après un plantage
//!
//! La première version ne rouvrait que ce qu'un plantage avait laissé : un brouillon, ou un
//! texte en cours de frappe. L'utilisateur, au premier essai : il avait fermé Glucose — par
//! « Fin de tâche », qui demande poliment à une fenêtre de se fermer —, son texte s'était
//! enregistré jusqu'à la dernière lettre, et le lancement suivant l'accueillait… sur le
//! document d'accueil. *« Ce serait super qu'au lancement Glucose regarde quel fichier il a
//! dernièrement utilisé, ou qui a crashé, et l'ouvre automatiquement, tout le temps. »*
//!
//! Glucose retient donc **le dernier document qu'il a ouvert ou enregistré**, et le lancement
//! rouvre le plus récent de ce qui l'attend : ce document, un brouillon qu'un plantage a
//! laissé sans nom, ou le document d'un texte en cours de frappe. Le plus récent, parce que
//! c'est celui sur lequel on travaillait. Un document qu'une autre fenêtre de Glucose tient
//! déjà n'est jamais choisi : il est ouvert là-bas.
//!
//! Le souvenir vit dans le dossier des brouillons — celui du travail à reprendre, que les
//! épreuves remplacent par le leur.

use super::verrou::tenu_ailleurs;
use crate::app::GlucoseApp;
use glucose_core::persist::histoire;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Le fichier du souvenir, dans le dossier des brouillons.
const DERNIER_DOCUMENT: &str = "dernier-document.txt";

/// Ce qui attend d'être rouvert.
enum Interrompu {
    /// Un document sans nom, qu'un plantage a laissé dans son brouillon.
    Brouillon(PathBuf),
    /// Un document qui a un nom : le dernier utilisé, ou celui d'un texte en cours.
    Document(PathBuf),
}

impl GlucoseApp {
    /// Retient ce document comme le dernier utilisé : le lancement suivant le rouvrira. Un
    /// souvenir qui ne s'écrit pas ne coûte qu'une réouverture à la main — l'échec se tait.
    pub(crate) fn retenir_le_document(&self, chemin: &Path) {
        let dossier = &self.disque.brouillons;
        if std::fs::create_dir_all(dossier).is_ok() {
            let _ = super::atomic::write_atomic(
                &dossier.join(DERNIER_DOCUMENT),
                chemin.to_string_lossy().as_bytes(),
            );
        }
    }

    /// Rouvre le plus récent de ce qui attend : le dernier document, un brouillon laissé sans
    /// nom, ou le document d'un texte en cours de frappe.
    pub fn retrouver_le_travail(&mut self) {
        let chemin = match ce_qui_attend(&self.disque.brouillons) {
            None => return,
            Some(Interrompu::Document(chemin)) => return self.open_from(chemin),
            Some(Interrompu::Brouillon(chemin)) => chemin,
        };
        let message = match self.ouvrir_un_brouillon(&chemin) {
            Ok((gestes, texte_rendu)) => format!(
                "Travail non enregistré retrouvé ({gestes} geste(s){}) — Ctrl+S pour lui \
                 donner un nom",
                if texte_rendu {
                    ", et le texte que tu tapais"
                } else {
                    ""
                }
            ),
            Err(e) => format!(
                "Un brouillon n'a pas pu se rouvrir : {e} — il reste dans {}",
                chemin.display()
            ),
        };
        self.ui.show_toast(message);
    }

    fn ouvrir_un_brouillon(&mut self, chemin: &Path) -> Result<(usize, bool), String> {
        let f = std::fs::File::open(chemin).map_err(|e| e.to_string())?;
        let ouvert =
            histoire::ouvrir(&mut std::io::BufReader::new(f)).map_err(|e| e.to_string())?;
        let adoption = self.adopter_un_ouvert(&ouvert, chemin.to_path_buf());
        if let Some(e) = self.disque.ecriture.as_mut() {
            e.brouillon = true;
        }
        self.project_path = None;
        // Du travail sans nom : le document est « modifié » jusqu'à ce qu'il en ait un.
        self.saved_version = self.store.version.wrapping_sub(1);
        Ok((ouvert.gestes.len(), adoption.texte_rendu))
    }
}

/// Le plus récent de ce qui attend, dans le dossier des brouillons : un brouillon qu'aucun
/// processus ne tient, le document d'un texte en cours, le dernier document retenu.
fn ce_qui_attend(dossier: &Path) -> Option<Interrompu> {
    let mut trouves: Vec<(SystemTime, Interrompu)> = Vec::new();
    for p in std::fs::read_dir(dossier).ok()?.flatten().map(|e| e.path()) {
        let Ok(date) = std::fs::metadata(&p).and_then(|m| m.modified()) else {
            continue;
        };
        let extension = p.extension().and_then(|e| e.to_str());
        if extension == Some(glucose_core::persist::FILE_EXTENSION) && !tenu_ailleurs(&p) {
            trouves.push((date, Interrompu::Brouillon(p)));
        } else if extension == Some("saisie") {
            if let Some(d) = super::frappe::document_d_une_saisie(&p, dossier) {
                trouves.push((date, Interrompu::Document(d)));
            }
        }
    }
    if let Some(dernier) = dernier_document(dossier) {
        trouves.push(dernier);
    }
    trouves
        .into_iter()
        .max_by_key(|(date, _)| *date)
        .map(|(_, i)| i)
}

/// Le dernier document retenu, daté de sa dernière écriture — s'il existe encore et
/// qu'aucune autre fenêtre ne le tient.
fn dernier_document(dossier: &Path) -> Option<(SystemTime, Interrompu)> {
    let texte = std::fs::read_to_string(dossier.join(DERNIER_DOCUMENT)).ok()?;
    let chemin = PathBuf::from(texte.trim());
    let date = std::fs::metadata(&chemin).ok()?.modified().ok()?;
    (chemin.is_file() && !chemin.starts_with(dossier) && !tenu_ailleurs(&chemin))
        .then_some((date, Interrompu::Document(chemin)))
}

#[cfg(test)]
mod tests;
