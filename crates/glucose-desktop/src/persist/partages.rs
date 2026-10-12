//! **PARTAGE-2 — ce que Glucose offre aux autres applications** là où il n'y a pas de
//! sélecteur de fichiers à qui demander « où » : au téléphone (fiche 63).
//!
//! Exporter et « Enregistrer l'image sous… » y ouvraient un sélecteur qui n'existe pas, et ne
//! faisaient rien. Ils écrivent désormais dans `partages/`, le seul dossier que la feuille de
//! partage d'Android a le droit d'offrir (`fichiers_partages.xml`), puis l'ouvrent.
//!
//! # Le lieu décide, et rien d'autre
//!
//! Aucun drapeau ne voyage avec l'export : **un fichier qui vient de s'écrire dans `partages/`
//! s'offre**, et c'est tout. L'image enregistrée, l'export en texte, et l'export en image à la
//! fin de son fil passent par la même question — [`GlucoseApp::offrir_si_partage`]. Le dossier
//! se vide à chaque nouveau partage : il ne garde que le dernier fichier, et ne grossit jamais.

use super::export::Format;
use crate::app::GlucoseApp;
use crate::ui::question::{Question, Reponse, Suite};
use std::path::{Path, PathBuf};

impl GlucoseApp {
    /// **Offre ce fichier**, s'il vient de s'écrire dans les partages. Ailleurs, rien.
    ///
    /// Le seul refus possible est l'absence de feuille de partage ; or rien ne s'écrit ici sans
    /// elle ([`crate::plateforme::partager::disponible`]) — il n'y a donc rien à dire de plus.
    pub(crate) fn offrir_si_partage(&self, chemin: &Path) {
        if chemin.parent() == Some(self.disque.partages.as_path()) {
            let _ = crate::plateforme::partager::partager(chemin);
        }
    }

    /// **Où écrire ce qu'on va partager** sous ce nom : le dossier des partages, vidé de ce
    /// qu'il gardait du partage d'avant.
    pub(crate) fn a_partager(&self, nom: &str) -> PathBuf {
        let _ = std::fs::remove_dir_all(&self.disque.partages);
        // Un dossier qui ne se crée pas, l'écriture le dira : c'est elle qui rend compte.
        let _ = std::fs::create_dir_all(&self.disque.partages);
        self.disque.partages.join(nom)
    }

    /// **Exporter, sans sélecteur** : le format se demande — aucun nom de fichier ne le dit
    /// par son extension —, dans l'ordre et sous les noms des filtres du bureau.
    pub(crate) fn demander_le_format_a_partager(&mut self) {
        let mut choix: Vec<(String, Reponse)> = Format::TOUS
            .iter()
            .enumerate()
            .map(|(i, f)| (f.nom().to_string(), Reponse::Choix(i)))
            .collect();
        choix.push(("Annuler".into(), Reponse::Annuler));
        let question = Question {
            titre: "Exporter".into(),
            texte: "Le tableau, puis la feuille de partage : Fichiers, la galerie, une \
                    conversation…"
                .into(),
            choix,
            ..Default::default()
        };
        self.demander(question, Suite::Exporter);
    }

    /// La réponse : le tableau s'écrit dans les partages, au format choisi, et s'offre.
    pub(crate) fn exporter_au_format(&mut self, reponse: Reponse) {
        let Reponse::Choix(i) = reponse else {
            return;
        };
        let Some(format) = Format::TOUS.get(i) else {
            return;
        };
        let nom = format!("{}.{}", self.document_label(), format.extension());
        let chemin = self.a_partager(&nom);
        self.exporter_vers(chemin);
    }
}

#[cfg(test)]
mod tests;
