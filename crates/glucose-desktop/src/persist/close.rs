//! Fermer la fenêtre sans perdre le travail (R-48, INVARIANT SAVE-3).
//!
//! Un document propre se ferme sans un mot. Un document modifié pose la question, et la
//! réponse passe par [`GlucoseApp::close_with`], qui relit l'état « modifié » **après**
//! l'enregistrement : tant que le document l'est encore, rien n'a été écrit, donc la
//! fenêtre reste ouverte. Fermer après un échec d'écriture serait une version pire du
//! défaut qu'on répare.
//!
//! L'état « modifié » n'est pas réinventé ici : c'est `store.version != saved_version`,
//! lu par `is_dirty()` (SAVE-2). Deux mécanismes finiraient par diverger.
//!
//! # La réponse arrive plus tard (POPUP-1, fiche 58)
//!
//! La question se dessine dans Glucose, qui continue de se dessiner pendant qu'elle attend :
//! la fermeture ne tient plus la boucle jusqu'à une réponse, elle a une **suite** — quitter
//! ([`Apres`]), ouvrir un autre document ([`Ensuite`]) — qui part quand la réponse arrive.
//! « Annuler », ou un enregistrement raté, ne fait rien partir. Et chaque réponse dit ce
//! qu'elle fait : les boîtes du système ne savaient pas nommer leurs boutons, il fallait
//! écrire « Oui — enregistrer puis fermer » dans le texte.

use crate::app::GlucoseApp;
use crate::ui::question::{Question, Reponse, Suite};
use std::path::PathBuf;

/// Ce que l'utilisateur répond quand on ferme une fenêtre au document modifié.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseChoice {
    /// Enregistrer, puis fermer — et **ne pas** fermer si l'enregistrement échoue.
    Save,
    /// Fermer en abandonnant les modifications.
    Discard,
    /// Ne pas fermer.
    Cancel,
}

impl CloseChoice {
    /// La réponse à la question, traduite : « Annuler », comme tout ce qui n'est pas un oui
    /// ou un non, ne ferme rien.
    fn de(reponse: Reponse) -> Self {
        match reponse {
            Reponse::Oui => Self::Save,
            Reponse::Non => Self::Discard,
            _ => Self::Cancel,
        }
    }
}

/// **Ce qui suit la fermeture du document.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Apres {
    /// La croix : la fenêtre part, et la boucle s'arrête.
    Quitter,
    /// La mise à jour prête : ce qui relance — l'installeur posé ici, ou Glucose déjà
    /// remplacé — démarre, puis Glucose se ferme (fiche 48).
    Installer(PathBuf),
}

/// **Ce qui suit un enregistrement** qu'une question a demandé : rien (`Ctrl+S`), fermer, ou
/// laisser le document pour un autre. Un travail sans nom choisit d'abord où s'écrire — un
/// sélecteur qui ne tient plus la boucle (DIAL-2) : la suite attend son choix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Puis {
    Rien,
    Fermer(Apres),
    Laisser(Ensuite),
}

/// **Ce qu'on ouvre à la place du document qu'on quitte** (BROUILLON-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ensuite {
    /// Ce document-ci.
    Ouvrir(PathBuf),
    /// Un document vierge (`Ctrl+N`).
    Vierge,
}

/// **La question du travail non enregistré** : « Enregistrer » d'abord, la réponse qui ne
/// perd rien, et que donne Entrée.
fn question_du_travail(label: &str) -> Question {
    Question {
        titre: "Modifications non enregistrées".into(),
        texte: format!("« {label} » contient des modifications non enregistrées."),
        choix: vec![
            ("Enregistrer".into(), Reponse::Oui),
            ("Ne pas enregistrer".into(), Reponse::Non),
            ("Annuler".into(), Reponse::Annuler),
        ],
        ..Default::default()
    }
}

impl GlucoseApp {
    /// **La croix, ou la mise à jour prête** : le document se ferme, puis `apres`.
    ///
    /// INVARIANT SAVE-3 — la fermeture ne perd jamais de travail (R-48). Un document propre
    /// se ferme sans un mot ; un document modifié pose la question, et la réponse est
    /// traitée par [`GlucoseApp::close_with`], qui refuse de fermer sur un échec.
    pub fn fermer_puis(&mut self, apres: Apres) {
        // Ce que le dernier geste a changé s'écrit d'abord — la carte en édition comprise : un
        // document qui a un nom est alors enregistré, et se ferme sans question (HISTOIRE-1).
        self.terminer_les_gestes_en_cours();
        self.consigner();
        if self.is_dirty() {
            let question = question_du_travail(&self.document_label());
            self.demander(question, Suite::Fermer(apres));
            return;
        }
        let ferme = self.fermer_le_document();
        self.apres_la_fermeture(ferme, apres);
    }

    /// La réponse à la question de la fermeture. « Enregistrer » sur un travail sans nom
    /// choisit d'abord où : la fermeture attend ce choix ([`Self::apres_l_enregistrement`]).
    pub(crate) fn repondre_a_la_fermeture(&mut self, reponse: Reponse, apres: Apres) {
        if reponse == Reponse::Oui && self.project_path.is_none() {
            self.enregistrer_puis(Puis::Fermer(apres));
            return;
        }
        let ferme = self.close_with(CloseChoice::de(reponse));
        self.apres_la_fermeture(ferme, apres);
    }

    /// **L'enregistrement fait, ce qui suit** — si le document est écrit. Sinon, rien ne part :
    /// l'échec s'est dit, et le travail reste où il est (SAVE-3) ; une mise à jour qui
    /// attendait est reportée.
    pub(crate) fn apres_l_enregistrement(&mut self, puis: Puis) {
        let ecrit = !self.is_dirty();
        match puis {
            Puis::Rien => {}
            Puis::Fermer(apres) => {
                let ferme = ecrit && self.fermer_le_document();
                self.apres_la_fermeture(ferme, apres);
            }
            Puis::Laisser(ensuite) if ecrit => self.poursuivre(ensuite),
            Puis::Laisser(_) => {}
        }
    }

    /// Le document fermé — ou non : rien ne part, et une mise à jour le dit.
    fn apres_la_fermeture(&mut self, ferme: bool, apres: Apres) {
        match (ferme, apres) {
            (true, Apres::Quitter) => self.ui.fermer_la_fenetre = true,
            (true, Apres::Installer(installeur)) => self.relancer_pour_installer(&installeur),
            (false, Apres::Installer(_)) => self.reporter_la_mise_a_jour(),
            (false, Apres::Quitter) => self.mark_dirty(),
        }
    }

    /// Applique une réponse à la question des modifications non enregistrées : `true` si le
    /// document est fermé.
    ///
    /// Séparée de la question pour que la règle — et surtout le refus de fermer après un
    /// enregistrement raté — soit vérifiable sans elle.
    pub fn close_with(&mut self, choice: CloseChoice) -> bool {
        match choice {
            CloseChoice::Cancel => false,
            CloseChoice::Discard => {
                // Le document sans nom qu'on ne garde pas : son brouillon s'efface, et le
                // prochain lancement ne le rouvrira pas.
                self.abandonner_le_brouillon();
                true
            }
            CloseChoice::Save => {
                self.save_project();
                // `save_project` a déjà dit pourquoi si l'écriture a échoué (toast). La
                // seule question qui reste est celle du document lui-même : tant qu'il est
                // modifié, rien n'a été écrit, et fermer perdrait exactement ce que R-48
                // décrit. On relit donc l'état « modifié », jamais la réponse.
                !self.is_dirty() && self.fermer_le_document()
            }
        }
    }

    /// **Du travail sans nom ne se quitte pas en silence** (BROUILLON-1) : avant d'ouvrir un
    /// autre document, celui qu'on quitte pose la question de la fermeture s'il a du travail
    /// sans nom ; `ensuite` part quand il est laissé. Sans elle, son brouillon restait sur le
    /// disque sans que rien ne le dise, et le lancement suivant ne rouvrait que le plus récent
    /// de ce qui attend : ce travail pouvait ne jamais reparaître.
    ///
    /// Un document nommé s'écrit geste après geste (HISTOIRE-1) : il n'est pas « modifié » ici,
    /// et se quitte sans un mot.
    pub(crate) fn laisser_puis(&mut self, ensuite: Ensuite) {
        self.terminer_les_gestes_en_cours();
        self.consigner();
        let laisse = if !self.is_dirty() {
            true
        } else if self.ui.documents_ranges {
            // Au téléphone, rien ne se demande : le travail sans nom se range dans les
            // documents, sous un nom libre (DOCUMENTS-1).
            self.ranger_dans_les_documents()
        } else {
            let question = question_du_travail(&self.document_label());
            self.demander(question, Suite::Laisser(ensuite));
            return;
        };
        if laisse {
            self.poursuivre(ensuite);
        }
    }

    /// La réponse à la question du départ : le document laissé, `ensuite` part — après le
    /// choix de l'endroit, pour un travail sans nom qu'on enregistre.
    pub(crate) fn repondre_au_depart(&mut self, reponse: Reponse, ensuite: Ensuite) {
        if reponse == Reponse::Oui && self.project_path.is_none() {
            self.enregistrer_puis(Puis::Laisser(ensuite));
            return;
        }
        if self.laisser_avec(CloseChoice::de(reponse)) {
            self.poursuivre(ensuite);
        }
    }

    fn poursuivre(&mut self, ensuite: Ensuite) {
        match ensuite {
            Ensuite::Ouvrir(chemin) => self.open_from(chemin),
            Ensuite::Vierge => self.adopter_un_document_vierge(),
        }
    }

    /// La réponse appliquée avant d'ouvrir l'autre document — séparée de la question pour se
    /// vérifier sans elle. Rien n'est fermé ici : si l'ouverture échoue ensuite, le document
    /// reste celui qu'on regarde, et continue de s'écrire.
    pub(crate) fn laisser_avec(&mut self, choix: CloseChoice) -> bool {
        match choix {
            CloseChoice::Cancel => false,
            CloseChoice::Discard => {
                self.abandonner_le_brouillon();
                true
            }
            CloseChoice::Save => {
                self.save_project();
                !self.is_dirty()
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;
