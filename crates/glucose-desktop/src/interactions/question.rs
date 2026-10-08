//! **Poser une question, et suivre sa réponse** (QUESTION-1, fiche 56 ; POPUP-1, fiche 58).
//!
//! Glucose dessine la question ([`crate::ui::question`]), **partout** : les boîtes du système
//! tenaient sa boucle tant qu'elles étaient ouvertes, et laissaient sa fenêtre noire (fiche
//! 58). Elle prend tout ce qui la touche — le doigt, la souris, le clavier — jusqu'à sa
//! réponse ; ce qui est derrière attend. Sa suite part à ce moment-là.
//!
//! # Une question remplacée n'a rien décidé
//!
//! Une question en chasse une autre (la croix pendant que le journal technique attend, un
//! menu rouvert) : celle qui part n'a reçu aucune réponse, donc n'a rien fait. Chaque suite est
//! écrite pour cela — ne rien recevoir vaut « Annuler » : le travail reste, rien ne s'envoie,
//! la question reviendra.

use crate::app::GlucoseApp;
use crate::params::{Pointer, ScreenFrame};
use crate::persist::close::Ensuite;
use crate::ui::question::{
    champ_sous, deplacer_le_focus, echappatoire, placer, reponse_en_evidence, reponse_sous,
    Question, Reponse, Suite,
};
use winit::keyboard::{Key, NamedKey};

impl GlucoseApp {
    /// **Pose cette question** ; sa réponse déclenchera `suite`.
    pub(crate) fn demander(&mut self, question: Question, suite: Suite) {
        self.ui.question = Some((question, suite));
        self.mark_dirty();
    }

    /// **La question prend tout ce qui la touche** tant qu'elle est posée : ce qui est
    /// derrière attend. L'appui retient la réponse pressée ; elle part au relâchement
    /// ([`Self::relacher_la_question`]).
    pub(crate) fn cliquer_la_question(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let ecran = self.ui.ecran_visible((screen.width, screen.height));
        let Some((question, _)) = &mut self.ui.question else {
            return false;
        };
        let placee = placer(question, &self.renderer.typography, ecran, screen.scale);
        question.sous_le_doigt = reponse_sous(&placee, pointer.x, pointer.y);
        // Toucher le champ ressort le clavier, que le geste retour a pu rentrer.
        if champ_sous(&placee, pointer.x, pointer.y) {
            self.lancement.clavier.redemander();
        }
        self.mark_dirty();
        true
    }

    /// **Le bouton se relève** : la réponse part s'il se relève sur celle qu'il a pressée
    /// (APPUI-1). Rend `true` si l'appui était à elle. Sinon, le relâchement termine le geste
    /// que l'appui avait commencé — un canevas qui glissait sous le doigt quand la question a
    /// paru ne doit pas continuer de le suivre.
    pub(crate) fn relacher_la_question(&mut self) -> bool {
        let (pointer, (largeur, hauteur)) = (self.pointer(), self.taille_de_la_fenetre());
        let screen = self.screen_frame(largeur, hauteur);
        let ecran = self.ui.ecran_visible((screen.width, screen.height));
        let Some((question, _)) = &mut self.ui.question else {
            return false;
        };
        let Some(pressee) = question.sous_le_doigt.take() else {
            return false;
        };
        let placee = placer(question, &self.renderer.typography, ecran, screen.scale);
        if reponse_sous(&placee, pointer.x, pointer.y) == Some(pressee) {
            self.repondre_a_la_question(Some(pressee));
        }
        self.mark_dirty();
        true
    }

    /// **Au clavier** (POPUP-1) : Entrée donne la réponse en évidence ; Tab, Maj + Tab et les
    /// flèches la déplacent ; Échap — et le retour d'Android — donne « Annuler », ou retire la
    /// question sans réponse. Rend `true` tant qu'une question est posée : aucune touche ne
    /// passe derrière elle, un `Suppr` n'efface rien sous le voile.
    pub(crate) fn touche_de_la_question(&mut self, touche: &Key, appuyee: bool) -> bool {
        let arriere = self.modifiers.shift_key();
        let Some((question, _)) = &mut self.ui.question else {
            return false;
        };
        if !appuyee {
            return true;
        }
        match touche {
            Key::Named(NamedKey::Tab) => deplacer_le_focus(question, if arriere { -1 } else { 1 }),
            Key::Named(NamedKey::ArrowDown | NamedKey::ArrowRight) => {
                deplacer_le_focus(question, 1);
            }
            Key::Named(NamedKey::ArrowUp | NamedKey::ArrowLeft) => {
                deplacer_le_focus(question, -1);
            }
            Key::Named(NamedKey::Enter) => {
                let reponse = reponse_en_evidence(question);
                self.repondre_a_la_question(reponse);
            }
            Key::Named(NamedKey::Escape | NamedKey::BrowserBack) => {
                let reponse = echappatoire(question);
                self.repondre_a_la_question(reponse);
            }
            _ => {}
        }
        self.mark_dirty();
        true
    }

    /// **Un appui long sur une question** n'y répond rien : la réponse pressée s'oublie — sauf
    /// sur un document de la liste, qui demande alors ce qu'on en fait (DOCUMENTS-2). Rend
    /// `true` s'il y avait une question.
    pub(crate) fn appui_long_sur_la_question(&mut self) -> bool {
        let Some((question, suite)) = &mut self.ui.question else {
            return false;
        };
        let document = match (suite, question.sous_le_doigt.take()) {
            (Suite::Ouvrir(chemins), Some(Reponse::Choix(i))) => chemins.get(i).cloned(),
            _ => None,
        };
        if let Some(chemin) = document {
            self.ui.question = None;
            self.gerer_un_document(chemin);
        }
        self.mark_dirty();
        true
    }

    /// La question se retire ; sa suite part s'il y a une réponse — sans réponse, rien ne
    /// part (VUE-1).
    fn repondre_a_la_question(&mut self, reponse: Option<Reponse>) {
        let Some((question, suite)) = self.ui.question.take() else {
            return;
        };
        if let Some(reponse) = reponse {
            self.suivre(suite, reponse, question.champ.map(|c| c.texte));
        }
    }

    /// Ce que la réponse déclenche ; `champ`, ce qu'on avait écrit dans la question.
    fn suivre(&mut self, suite: Suite, reponse: Reponse, champ: Option<String>) {
        let oui = reponse == Reponse::Oui;
        match suite {
            Suite::NouveauDocument if oui => self.laisser_puis(Ensuite::Vierge),
            Suite::Telemetrie => self.repondre_a_la_telemetrie(oui),
            Suite::Ouvrir(chemins) => {
                if let Some(chemin) = reponse_choisie(reponse).and_then(|i| chemins.get(i)) {
                    self.laisser_puis(Ensuite::Ouvrir(chemin.clone()));
                }
            }
            Suite::Gerer(chemin) => self.suivre_la_gestion(chemin, reponse),
            Suite::Renommer(chemin) => match (reponse, champ) {
                (Reponse::Oui, Some(nom)) => self.renommer_le_document(&chemin, &nom),
                _ => self.choisir_un_document(),
            },
            Suite::Supprimer(chemin) if oui => self.supprimer_le_document(&chemin),
            Suite::Supprimer(_) => self.choisir_un_document(),
            Suite::Fermer(apres) => self.repondre_a_la_fermeture(reponse, apres),
            Suite::Laisser(ensuite) => self.repondre_au_depart(reponse, ensuite),
            Suite::MiseAJour(proposition) if oui => self.preparer_la_mise_a_jour(proposition),
            Suite::NouveauDocument | Suite::MiseAJour(_) => {}
        }
    }
}

/// Le rang choisi dans une liste, si la réponse en est un.
fn reponse_choisie(reponse: Reponse) -> Option<usize> {
    match reponse {
        Reponse::Choix(i) => Some(i),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) mod tests;
