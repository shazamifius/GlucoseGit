//! **La fin de Glucose, quelle qu'elle soit** (FIN-1, FIN-SESSION-1, fiche 82) : la croix, une
//! mise à jour qui relance, l'extinction de Windows, un signal sous Linux, AppKit qui quitte.
//!
//! Ce qui doit être vrai quand Glucose s'arrête est écrit **une fois**, ici, et chaque chemin y
//! passe : le document écrit et synchronisé, ce que Glucose a confié au presse-papiers rendu au
//! système, la chronique et la boîte noire closes. Avant, `exiting` ne fermait pas le document —
//! chaque chemin devait l'avoir fait avant lui —, et son commentaire disait couvrir « l'arrêt
//! demandé par le système », que `winit` ne transmet pas sous Windows ([`crate::plateforme::fin`]).
//!
//! Rien ici ne pose de question : quand le système finit la session, personne n'est là pour
//! répondre. Un document sans nom reste un brouillon sur le disque, que le lancement suivant
//! rouvre ; un document nommé est déjà écrit, geste après geste (HISTOIRE-1).
//!
//! Chaque étape ne fait son travail qu'une fois, par elle-même — le document se ferme en rendant
//! son écriture, la boîte noire en se rendant — : finir deux fois (l'extinction, puis `exiting`
//! quand le Restart Manager laisse vivre le processus) ne ferme rien deux fois.

use super::GlucoseApp;

impl GlucoseApp {
    /// **FIN-1 — Glucose s'arrête** : le document écrit et synchronisé, sans question ; puis le
    /// presse-papiers, la chronique et la boîte noire.
    pub(crate) fn finir(&mut self) {
        // Une réponse a déjà réglé le sort du document quand la fenêtre a le droit de partir :
        // fermé, enregistré, ou abandonné — et un brouillon qu'on vient d'abandonner ne doit
        // pas renaître ici. Sinon, c'est le système qui finit : le document se ferme lui-même.
        if !self.ui.fermer_la_fenetre {
            self.fermer_le_document();
        }
        // Ce que Glucose a confié au presse-papiers y reste après lui (COPIER-1).
        crate::plateforme::presse_papiers::rendre_avant_de_quitter();
        self.noter_le_gel_en_cours();
        self.clore_la_chronique();
        self.vider_les_signes_du_pave();
        self.chronique.clore_la_boite_noire();
    }

    /// **FIN-SESSION-1 — Windows finit la session** : Glucose finit maintenant, pendant
    /// `WM_ENDSESSION`, puis la fenêtre a le droit de partir — le Restart Manager attend que le
    /// processus s'en aille.
    pub(crate) fn finir_avec_la_session(&mut self) {
        self.finir();
        self.ui.fermer_la_fenetre = true;
    }

    /// **La fermeture que `winit` transmet vient-elle de la fin de la session ?** La fenêtre en
    /// porte alors la marque.
    pub(crate) fn la_session_finit(&self) -> bool {
        self.lancement
            .session
            .as_ref()
            .is_some_and(crate::plateforme::fin::Session::finit)
    }

    /// **Un signal de fin est-il arrivé ?** (Linux) La boucle part alors à son tour suivant, et
    /// `exiting` finit.
    pub(crate) fn un_signal_demande_la_fin(&self) -> bool {
        self.lancement
            .signaux
            .as_ref()
            .is_some_and(crate::plateforme::fin::Demande::faite)
    }
}

#[cfg(test)]
mod tests;
