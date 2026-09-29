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
//!
//! # Un brouillon d'abord (BROUILLON-1)
//!
//! Un document nommé est en sûreté sur le disque, et `Ctrl+O` le rouvre. Un brouillon n'a que
//! le lancement pour reparaître : si un document plus récent passait devant lui, il attendait
//! dans son dossier sans que rien ne le dise, lancement après lancement. Le plus récent des
//! brouillons se rouvre donc **avant** tout document, et le compte rendu dit combien d'autres
//! attendent encore ; chacun reparaît à son tour.

use super::frappe::Attente;
use super::verrou::tenu_ailleurs;
use crate::app::GlucoseApp;
use glucose_core::persist::histoire::{self, Saisie};
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
    /// nom, ou le document d'un texte en cours de frappe. Puis le texte en cours dont le
    /// document est introuvable revient dans une carte neuve (FRAPPE-1). Tout se dit d'un seul
    /// compte rendu : le lancement est un geste.
    pub fn retrouver_le_travail(&mut self) {
        let attentes = ce_qui_attend(&self.disque.brouillons);
        let mut dit = match attentes.premier {
            None => Vec::new(),
            Some(Interrompu::Document(chemin)) => vec![self.ouvrir_et_dire(chemin)],
            Some(Interrompu::Brouillon(chemin)) => {
                vec![self.ouvrir_un_brouillon(&chemin).unwrap_or_else(|e| {
                    format!(
                        "Un brouillon n'a pas pu se rouvrir : {e} — il reste dans {}",
                        chemin.display()
                    )
                })]
            }
        };
        match attentes.autres_brouillons {
            0 => {}
            1 => {
                dit.push("un autre brouillon attend : il se rouvrira au prochain lancement".into())
            }
            n => dit.push(format!(
                "{n} autres brouillons attendent : ils se rouvriront aux prochains lancements"
            )),
        }
        for (rang, (fichier, saisie)) in attentes.orphelines.into_iter().enumerate() {
            self.rendre_dans_une_carte_neuve(&saisie.texte, rang);
            let nom = Path::new(&saisie.document).file_name().map_or_else(
                || saisie.document.clone(),
                |n| n.to_string_lossy().into_owned(),
            );
            let range = super::recuperation::ranger(&fichier, &self.disque.brouillons)
                .unwrap_or_else(|_| fichier.clone());
            dit.push(format!(
                "Le texte que tu tapais dans « {nom} », introuvable, est revenu dans une carte \
                 neuve — une copie reste dans {}",
                range.display()
            ));
        }
        if !dit.is_empty() {
            self.ui.show_toast(dit.join(" · "));
        }
    }

    /// Rouvre un brouillon, et rend ce que le lancement en dit.
    fn ouvrir_un_brouillon(&mut self, chemin: &Path) -> Result<String, String> {
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
        let mut message = format!(
            "Travail non enregistré retrouvé ({} geste(s){})",
            ouvert.gestes.len(),
            if adoption.texte_rendu {
                ", et le texte que tu tapais"
            } else {
                ""
            }
        );
        if ouvert.fin_ignoree > 0 {
            message.push_str(&super::commands::dire_la_fin(
                ouvert.fin_ignoree,
                adoption.mise_de_cote.as_deref(),
            ));
        }
        Ok(message + " — Ctrl+S pour lui donner un nom")
    }
}

/// Ce qui attend au lancement.
struct Attentes {
    /// Ce qui se rouvre : le plus récent des brouillons, sinon le plus récent des documents.
    premier: Option<Interrompu>,
    /// Les autres brouillons qu'aucune fenêtre ne tient : ils reparaîtront à leur tour.
    autres_brouillons: usize,
    /// Les textes en cours dont le document est introuvable (FRAPPE-1).
    orphelines: Vec<(PathBuf, Saisie)>,
}

/// Ce qui attend dans le dossier des brouillons : les brouillons qu'aucun processus ne tient,
/// le document d'un texte en cours, le dernier document retenu, et les textes en cours dont
/// le document est introuvable.
fn ce_qui_attend(dossier: &Path) -> Attentes {
    let mut brouillons: Vec<(SystemTime, PathBuf)> = Vec::new();
    let mut documents: Vec<(SystemTime, PathBuf)> = Vec::new();
    let mut orphelines = Vec::new();
    let entrees = std::fs::read_dir(dossier).into_iter().flatten().flatten();
    for p in entrees.map(|e| e.path()) {
        let Ok(date) = std::fs::metadata(&p).and_then(|m| m.modified()) else {
            continue;
        };
        let extension = p.extension().and_then(|e| e.to_str());
        if extension == Some(glucose_core::persist::FILE_EXTENSION) && !tenu_ailleurs(&p) {
            brouillons.push((date, p));
        } else if extension == Some("saisie") {
            match super::frappe::ce_qu_attend_une_saisie(&p, dossier) {
                Attente::Document(d) => documents.push((date, d)),
                Attente::Orpheline(s) => orphelines.push((p, s)),
                Attente::Rien => {}
            }
        }
    }
    documents.extend(dernier_document(dossier));
    let autres_brouillons = brouillons.len().saturating_sub(1);
    let plus_recent =
        |l: Vec<(SystemTime, PathBuf)>| l.into_iter().max_by_key(|(date, _)| *date).map(|(_, p)| p);
    let premier = match plus_recent(brouillons) {
        Some(b) => Some(Interrompu::Brouillon(b)),
        None => plus_recent(documents).map(Interrompu::Document),
    };
    Attentes {
        premier,
        autres_brouillons,
        orphelines,
    }
}

/// Le dernier document retenu, daté de sa dernière écriture — s'il existe encore et
/// qu'aucune autre fenêtre ne le tient.
fn dernier_document(dossier: &Path) -> Option<(SystemTime, PathBuf)> {
    let texte = std::fs::read_to_string(dossier.join(DERNIER_DOCUMENT)).ok()?;
    let chemin = PathBuf::from(texte.trim());
    let date = std::fs::metadata(&chemin).ok()?.modified().ok()?;
    (chemin.is_file() && !chemin.starts_with(dossier) && !tenu_ailleurs(&chemin))
        .then_some((date, chemin))
}

#[cfg(test)]
mod tests;
