//! **`Ctrl+N` : un nouveau document vierge** (fiche 51 § 4).
//!
//! Glucose Rust n'avait aucun moyen d'en commencer un : ni raccourci, ni entrée. Il fallait
//! fermer et relancer — et le lancement rouvrait le dernier travail.
//!
//! # Rien ne se perd
//!
//! Le document qu'on quitte passe par la même porte qu'avant d'en ouvrir un autre
//! ([`GlucoseApp::laisser_le_document`]) : un document nommé s'écrit déjà geste après geste et
//! se quitte sans un mot ; du travail **sans nom** pose la question habituelle (BROUILLON-1),
//! et « Annuler » ne change rien.
//!
//! # Vierge
//!
//! Un tableau vide, sans carte d'accueil : l'accueil est pour le premier lancement, pas pour
//! un document qu'on vient de demander. Il n'a pas de fichier tant qu'on n'y a rien fait ; son
//! premier geste fera naître son brouillon, comme pour le document du lancement.

use crate::app::GlucoseApp;

impl GlucoseApp {
    /// `Ctrl+N`, et l'entrée « Nouveau document » du menu.
    ///
    /// # NOUVEAU-1 — `Ctrl+N` demande toujours
    ///
    /// Un document nommé se quittait sans un mot : chaque geste y est déjà écrit. Juste, mais son
    /// essai du 07/10 l'a lu comme un travail perdu, et un message après coup ne suffisait pas :
    /// *« si on fait Ctrl+N accidentellement, que ça ne swappe pas instantanément, mais qu'on
    /// voie une popup : voulez-vous créer un nouveau document ? »*. Un raccourci se tape par
    /// erreur ; la question rattrape l'erreur avant qu'elle ait lieu.
    ///
    /// Un travail sans nom à enregistrer pose déjà la sienne (BROUILLON-1) — enregistrer, ne pas
    /// enregistrer, annuler —, qui vaut confirmation : deux boîtes de suite seraient de trop.
    pub fn nouveau_document(&mut self) {
        self.terminer_les_gestes_en_cours();
        self.consigner();
        if !self.is_dirty() && !self.confirmer_le_nouveau_document() {
            return;
        }
        if !self.laisser_le_document() {
            return;
        }
        self.adopter_un_document_vierge();
    }

    /// La question de NOUVEAU-1 : oui, un nouveau document ; non, rien ne change.
    fn confirmer_le_nouveau_document(&mut self) -> bool {
        let question = match self.project_path {
            Some(_) => format!(
                "Créer un nouveau document ?\n\n« {} » est enregistré : il reste où il est, \
                 et se rouvre par Ouvrir.",
                self.document_label()
            ),
            None => "Créer un nouveau document ?".to_string(),
        };
        self.sous_un_dialogue(|ancre| {
            crate::dialogue::oui_ou_non(ancre, "Nouveau document", &question)
        })
    }

    /// Fait d'un document vierge le document courant. Séparée de la question pour se
    /// vérifier sans dialogue.
    pub(crate) fn adopter_un_document_vierge(&mut self) {
        // Le document qu'on quitte s'écrit une dernière fois. S'il n'a pas pu, son écriture
        // s'abandonne ici — le toast l'a dit —, comme à l'ouverture d'un autre.
        if !self.fermer_le_document() {
            self.disque.ecriture = None;
        }
        self.disque.objets.vider();
        self.editing_session = None;
        self.historique_du_texte.oublier();
        self.selection_box = None;
        self.dock_manager.domains.reset();
        let vierge = glucose_core::types::Project::new(crate::app::accueil::NOM_D_UN_DOCUMENT_NEUF);
        // Le chargement vide l'annulation (JRN-2) ; ce qu'il a mis dans la file de sortie
        // n'appartient à personne — rien n'est encore écrit pour ce document.
        self.store.load_project(vierge);
        self.store.journal.prendre_les_ecrits();
        self.store.bump_version();
        self.disque.depart = Some(self.store.project.clone());
        self.disque.a_sceller.clear();
        self.project_path = None;
        self.saved_version = self.store.version;
        self.suivre_le_document();
        self.sync_window_title();
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
