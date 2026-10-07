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
    /// **Un document nommé se quitte sans question, mais pas sans un mot.** Chaque geste y est
    /// déjà écrit : demander « enregistrer ? » mentirait. Mais rien ne le disait, et son essai du
    /// 07/10 l'a lu comme un travail abandonné — « il crée instantanément une nouvelle session
    /// sans même demander d'enregistrer notre travail ». Le message dit ce qui vient d'avoir
    /// lieu : le document est enregistré, et un autre commence.
    pub fn nouveau_document(&mut self) {
        let quitte = self.project_path.as_ref().map(|_| self.document_label());
        if !self.laisser_le_document() {
            return;
        }
        self.adopter_un_document_vierge();
        if let Some(nom) = quitte {
            self.ui
                .show_toast(format!("« {nom} » est enregistré — nouveau document"));
        }
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
