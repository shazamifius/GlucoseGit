//! **La discussion, vue de la boucle** (fiche 129) : ouvrir le tiroir, envoyer une question,
//! recevoir ce que le fil rapporte, le clavier du champ. Le réseau et la bascule vivent dans
//! [`crate::ia::discussion`] ; ici, rien que les relier — et le dire.

use crate::app::GlucoseApp;
use crate::dock::discussion::{DiscussionIntent, Message};
use crate::dock::TabId;
use crate::ia::cles;
use crate::ia::discussion::{self, Conversation, Evenement};
use crate::interactions::question::frappe::OuEcrire;
use std::sync::atomic::Ordering;
use std::sync::mpsc::TryRecvError;
use winit::keyboard::{Key, NamedKey};

/// Ce que l'arrêt demandé rend : pas une erreur à montrer.
const ARRETEE: &str = "arrêtée";

impl GlucoseApp {
    /// **« Discuter avec une IA »** : le tiroir s'ouvre, le clavier va au champ. Sans clé, la
    /// question de la clé d'abord.
    pub(crate) fn ouvrir_la_discussion(&mut self) {
        if !self.dock_manager.is_open(TabId::Discussion) {
            self.dock_manager.toggle_tab(TabId::Discussion);
        }
        self.dock_manager.discussion.tape = true;
        if self.cle_google().is_none() {
            self.demander_la_cle_google();
        }
        self.descendre_la_discussion();
    }

    /// **« Tester les modèles gratuits »** : dans le tiroir, chaque modèle écarté dit pourquoi,
    /// et le premier qui répond se nomme.
    pub(crate) fn tester_les_modeles(&mut self) {
        self.ouvrir_la_discussion();
        let Some((conversation, boite)) = self.preparer_une_question() else {
            return;
        };
        let ui = &mut self.dock_manager.discussion;
        ui.dire(Message::Note("Essai des modèles gratuits…".into()));
        ui.en_cours = Some("L'essai".into());
        self.lancement.agents.discussion = Some(discussion::essayer(conversation, boite));
        self.descendre_la_discussion();
    }

    /// **Envoie ce qui est écrit dans le champ.**
    fn envoyer_dans_la_discussion(&mut self) {
        let question = self.dock_manager.discussion.entree.texte.trim().to_string();
        if question.is_empty() || self.lancement.agents.discussion.is_some() {
            return;
        }
        let Some((conversation, boite)) = self.preparer_une_question() else {
            return;
        };
        let ui = &mut self.dock_manager.discussion;
        ui.entree = Default::default();
        ui.dire(Message::Moi(question.clone()));
        ui.en_cours = Some("L'IA".into());
        self.lancement.agents.discussion = Some(discussion::poser(conversation, question, boite));
        self.descendre_la_discussion();
    }

    /// La conversation au repos (ou une neuve, avec la clé), et la boîte des agents — ou le dire.
    fn preparer_une_question(&mut self) -> Option<(Conversation, crate::agents::Boite)> {
        if self.lancement.agents.discussion.is_some() {
            return None;
        }
        let Some(cle) = self.cle_google() else {
            self.dire_dans_la_discussion(Message::Erreur(
                "Il faut d'abord ta clé Google AI Studio : Collaborer, puis « L'API publique »."
                    .into(),
            ));
            return None;
        };
        let Some(boite) = self.lancement.agents.boite.clone() else {
            self.dire_dans_la_discussion(Message::Erreur(
                "La porte des agents n'est pas ouverte : l'IA ne pourrait rien faire ici.".into(),
            ));
            return None;
        };
        let conversation = self
            .lancement
            .agents
            .conversation
            .take()
            .filter(|c| c.cle == cle)
            .unwrap_or_else(|| Conversation::nouvelle(cle));
        Some((conversation, boite))
    }

    /// La clé Google AI Studio que l'utilisateur a donnée, s'il en a donné une.
    pub(crate) fn cle_google(&self) -> Option<String> {
        let chemin = self.lancement.agents.cles.as_deref()?;
        cles::lire_depuis(chemin).google
    }

    /// Ce qu'un clic dans le tiroir demande.
    pub(crate) fn agir_dans_la_discussion(&mut self, intent: DiscussionIntent) {
        match intent {
            DiscussionIntent::Saisir => self.dock_manager.discussion.tape = true,
            DiscussionIntent::Envoyer => self.envoyer_dans_la_discussion(),
            DiscussionIntent::Arreter => {
                if let Some(en_route) = &self.lancement.agents.discussion {
                    en_route.arret.store(true, Ordering::Relaxed);
                    self.dire_dans_la_discussion(Message::Note(
                        "Arrêt demandé : la réponse en cours finit d'abord.".into(),
                    ));
                }
            }
        }
        self.mark_dirty();
    }

    /// **À chaque tour de la boucle** : ce que le fil a rapporté, puis les lignes recoupées si le
    /// fil ou l'échelle ont changé.
    pub(crate) fn suivre_la_discussion(&mut self) {
        let mut arrive = Vec::new();
        let mut orphelin = false;
        if let Some(en_route) = &self.lancement.agents.discussion {
            loop {
                match en_route.evenements.try_recv() {
                    Ok(e) => arrive.push(e),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        orphelin = true;
                        break;
                    }
                }
            }
        }
        let rien = arrive.is_empty() && !orphelin;
        for e in arrive {
            self.recevoir(e);
        }
        if orphelin && self.lancement.agents.discussion.take().is_some() {
            self.dock_manager.discussion.en_cours = None;
            self.dire_dans_la_discussion(Message::Erreur(
                "La discussion s'est interrompue : la suivante repartira de zéro.".into(),
            ));
        }
        let s = crate::theme::clamp_ui_scale(self.ui.scale_factor);
        let typo = &self.renderer.typography;
        if self.dock_manager.discussion.couper(typo, s) || !rien {
            self.descendre_la_discussion();
        }
    }

    /// Un événement du fil, dans le tiroir.
    fn recevoir(&mut self, e: Evenement) {
        let ui = &mut self.dock_manager.discussion;
        match e {
            Evenement::Demande(modele) => ui.en_cours = Some(modele),
            Evenement::Geste { outil, echec } => {
                ui.dire(Message::Geste(geste_en_mots(&outil, echec)))
            }
            Evenement::Texte { modele, texte } => ui.dire(Message::Ia { modele, texte }),
            Evenement::Avertissement(t) => ui.dire(Message::Note(t)),
            Evenement::Fin {
                conversation,
                erreur,
            } => {
                ui.en_cours = None;
                if let Some(e) = erreur.filter(|e| e != ARRETEE) {
                    ui.dire(Message::Erreur(e));
                }
                self.lancement.agents.discussion = None;
                self.lancement.agents.conversation = Some(conversation);
            }
        }
    }

    fn dire_dans_la_discussion(&mut self, message: Message) {
        self.dock_manager.discussion.dire(message);
        self.descendre_la_discussion();
    }

    /// **Le dernier message se voit** : le tiroir défile jusqu'en bas — le rangement borne le
    /// défilement à ce qu'il cache (TIROIR-6).
    fn descendre_la_discussion(&mut self) {
        self.dock_manager
            .defilement
            .insert(TabId::Discussion, f32::MAX);
        self.mark_dirty();
    }

    /// **Une touche pendant qu'on écrit à l'IA.** Rend `true` si elle est consommée.
    pub(crate) fn frapper_dans_la_discussion(&mut self, key: &Key, texte: Option<&str>) -> bool {
        let ui = &self.dock_manager.discussion;
        if !ui.tape || !self.dock_manager.is_open(TabId::Discussion) {
            return false;
        }
        match key {
            Key::Named(NamedKey::Enter) => self.envoyer_dans_la_discussion(),
            Key::Named(NamedKey::Escape) => self.dock_manager.discussion.tape = false,
            // Le reste par la loi de l'édition d'un nœud : accents des touches mortes, sélection,
            // copier, coller (CHAMP-QUESTION-1).
            _ => return self.frapper_le_champ(OuEcrire::Discussion, key, texte),
        }
        self.mark_dirty();
        true
    }
}

/// **Un geste de l'IA, en mots** — ce que l'utilisateur lit dans le fil ; sa trace exacte est
/// dans la Time Machine, signée.
pub fn geste_en_mots(outil: &str, echec: Option<String>) -> String {
    let fait = match outil {
        "glucose_lire" => "a lu le tableau",
        "glucose_chercher" => "a cherché dans le document",
        "glucose_poser_texte" => "a posé un texte",
        "glucose_poser_membrane" => "a posé une membrane",
        "glucose_ecrire" => "a écrit dans un nœud",
        "glucose_relier" => "a tendu une flèche",
        "glucose_repondre" => "a répondu à un nœud",
        "glucose_poser_page" => "a posé une page web",
        "glucose_deplacer" => "a déplacé un nœud",
        "glucose_ouvrir_tableau" => "a ouvert un onglet",
        "glucose_poser_domaine" => "a posé un domaine",
        "glucose_grouper" => "a groupé des nœuds",
        "glucose_disposer" => "a rangé une membrane",
        "glucose_retirer" => "a retiré un nœud",
        "glucose_dater" => "a daté un nœud",
        "glucose_noter" => "a écrit une note",
        "glucose_histoire" => "a lu l'histoire du document",
        "glucose_quoi_de_neuf" => "a lu ce qui a changé",
        autre => return geste_inconnu(autre, echec),
    };
    match echec {
        None => fait.to_string(),
        Some(e) => format!("un geste refusé ({outil}) : {e}"),
    }
}

fn geste_inconnu(outil: &str, echec: Option<String>) -> String {
    match echec {
        None => format!("a appelé {outil}"),
        Some(e) => format!("un geste refusé ({outil}) : {e}"),
    }
}

#[cfg(test)]
mod tests;
