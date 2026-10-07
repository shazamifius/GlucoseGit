//! **DOCUMENTS-1 — les documents du téléphone** (fiche 56).
//!
//! Au bureau, un travail sans nom vit dans un brouillon, et le quitter pose la question
//! habituelle — enregistrer (où ?), ne pas enregistrer, annuler (BROUILLON-1). Au téléphone,
//! il n'y a pas de sélecteur de fichiers à qui demander « où », et cette question ne mène nulle
//! part : sous Android, elle répondait « annuler », et un document sans nom ne pouvait jamais
//! être quitté — « Nouveau document » ne faisait rien.
//!
//! Le téléphone fait donc comme les applications de dessin qu'on y trouve (Infinite Painter,
//! Concepts, Procreate) : **rien ne se demande, tout se garde**. Un travail sans nom qu'on quitte
//! se range dans le dossier des documents de Glucose, sous le premier nom libre — « Canevas 1 »,
//! « Canevas 2 »… —, et il s'y écrit désormais geste après geste comme tout document nommé.

use crate::app::GlucoseApp;
use glucose_core::persist::FILE_EXTENSION;
use std::path::{Path, PathBuf};

/// Le nom des documents que le téléphone range d'office, suivi de leur numéro.
const NOM: &str = "Canevas";

/// **Le premier nom libre** de ce dossier : `Canevas 1.glucose`, puis 2, puis 3…
pub(crate) fn nom_libre(dossier: &Path) -> PathBuf {
    (1u32..)
        .map(|n| dossier.join(format!("{NOM} {n}.{FILE_EXTENSION}")))
        .find(|chemin| !chemin.exists())
        .unwrap_or_else(|| dossier.join(format!("{NOM}.{FILE_EXTENSION}")))
}

/// Combien de documents la liste montre au plus : ce qu'un téléphone tient debout, en
/// rangées de 48 points, sous son titre.
const MONTRES: usize = 10;

/// **Les documents de ce dossier**, du plus récemment touché au plus ancien.
pub(crate) fn les_documents(dossier: &Path) -> Vec<PathBuf> {
    let Ok(entrees) = std::fs::read_dir(dossier) else {
        return Vec::new();
    };
    let mut documents: Vec<(std::time::SystemTime, PathBuf)> = entrees
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case(FILE_EXTENSION))
        })
        .map(|p| {
            let quand = std::fs::metadata(&p)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            (quand, p)
        })
        .collect();
    documents.sort_by_key(|(quand, _)| std::cmp::Reverse(*quand));
    documents.into_iter().map(|(_, p)| p).collect()
}

impl GlucoseApp {
    /// **« Ouvrir un document… »** : le sélecteur de fichiers au bureau ; au téléphone, la
    /// liste des documents rangés, du plus récent au plus ancien (DOCUMENTS-1).
    pub(crate) fn choisir_un_document(&mut self) {
        if !self.ui.questions_dessinees {
            self.open_project();
            return;
        }
        use crate::ui::question::{Question, Reponse, Suite};
        let mut documents = les_documents(&self.disque.documents);
        let texte = match documents.len() {
            0 => "Aucun document rangé pour l'instant.".to_string(),
            n if n > MONTRES => format!("Les {MONTRES} plus récents, sur {n}."),
            _ => String::new(),
        };
        documents.truncate(MONTRES);
        let mut choix: Vec<(String, Reponse)> = documents
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let nom = p
                    .file_stem()
                    .map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                (nom, Reponse::Choix(i))
            })
            .collect();
        choix.push(("Annuler".into(), Reponse::Annuler));
        let question = Question {
            titre: "Documents".into(),
            texte,
            choix,
        };
        self.demander(question, Suite::Ouvrir(documents));
    }

    /// **Range le travail sans nom dans les documents**, sous le premier nom libre, et rend
    /// `true` s'il y est en sûreté — sinon le document reste celui qu'on regarde, et
    /// l'échec s'est dit.
    pub(crate) fn ranger_dans_les_documents(&mut self) -> bool {
        let dossier = self.disque.documents.clone();
        // Un dossier qui ne se crée pas, l'écriture le dira : c'est elle qui rend compte.
        let _ = std::fs::create_dir_all(&dossier);
        self.save_to(nom_libre(&dossier));
        !self.is_dirty()
    }
}

#[cfg(test)]
mod tests;
