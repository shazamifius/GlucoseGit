//! **« L'API publique »** (fiches 128 et 129) : Collaborer ouvre la discussion avec une IA
//! gratuite, l'essai des modèles, et la clé. Le réseau tourne sur un fil à part
//! ([`crate::ia::discussion`]) ; ce que l'essai trouve se lit dans le tiroir de la discussion,
//! ligne à ligne.

use crate::app::GlucoseApp;
use crate::ia::cles;
use crate::ia::modeles::Fournisseur;
use crate::ui::question::{Champ, Question, Reponse, Suite};

/// Le bouton « L'API publique… » de Collaborer, puis les réponses de son menu.
pub(crate) const MENU: Reponse = Reponse::Choix(14);
const TESTER: Reponse = Reponse::Choix(10);
const CLE_GOOGLE: Reponse = Reponse::Choix(11);
const DISCUTER: Reponse = Reponse::Choix(12);
/// « Enregistrer » dans la question où l'on écrit une clé : la réponse Oui.
const ENREGISTRER: Reponse = Reponse::Oui;

impl GlucoseApp {
    /// **« L'API publique… »** : la clé branchée ou non, et les trois gestes.
    pub(crate) fn montrer_l_api_publique(&mut self, note: Option<String>) {
        let branchee = self.cle_google().is_some();
        let etat = if branchee {
            "Ta clé Google AI Studio est branchée."
        } else {
            "Aucune clé n'est branchée : donne d'abord ta clé Google AI Studio (gratuite, sur \
             aistudio.google.com)."
        };
        let mut texte = format!(
            "Des IA gratuites dans Glucose. {etat}\n\nLa discussion s'ouvre dans un tiroir : \
             l'IA lit et écrit ce document sous tes yeux, chaque geste s'annule par Ctrl+Z. \
             Quand un modèle atteint sa limite, le suivant prend le relais, et Glucose te le dit ; \
             le meilleur reprend la main dès qu'il revient."
        );
        if let Some(n) = note {
            texte = format!("{n}\n\n{texte}");
        }
        let question = Question {
            titre: "L'API publique".into(),
            texte,
            choix: vec![
                ("Discuter avec une IA".into(), DISCUTER),
                ("Tester les modèles gratuits".into(), TESTER),
                ("Donner la clé Google AI Studio".into(), CLE_GOOGLE),
                ("Fermer".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::ApiPublique);
    }

    /// Sa réponse au menu de l'API publique.
    pub(crate) fn repondre_a_l_api_publique(&mut self, reponse: Reponse) {
        match reponse {
            DISCUTER => self.ouvrir_la_discussion(),
            TESTER => self.tester_les_modeles(),
            CLE_GOOGLE => self.demander_la_cle_google(),
            _ => {}
        }
    }

    /// La question où l'on écrit la clé ; « Enregistrer » la garde sur ce Glucose.
    pub(crate) fn demander_la_cle_google(&mut self) {
        let question = Question {
            titre: "Clé Google AI Studio".into(),
            texte: "Colle ta clé Google AI Studio ici. Elle reste sur ce Glucose, à tes seuls \
                    droits de lecture, et ne part que vers Google."
                .into(),
            choix: vec![
                ("Enregistrer".into(), ENREGISTRER),
                ("Annuler".into(), Reponse::Annuler),
            ],
            champ: Some(Champ::default()),
            ..Default::default()
        };
        self.demander(question, Suite::ClefIa(Fournisseur::Google));
    }

    /// Ce que la réponse à la question de clé décide : on garde la clé écrite, s'il y en a une ;
    /// une clé neuve commence une conversation neuve.
    pub(crate) fn enregistrer_la_cle(
        &mut self,
        _fournisseur: Fournisseur,
        reponse: Reponse,
        texte: Option<String>,
    ) {
        if reponse != ENREGISTRER {
            self.montrer_l_api_publique(None);
            return;
        }
        let cle = texte
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty());
        let ecrit = match self.lancement.agents.cles.as_deref() {
            Some(chemin) => cles::ecrire_dans(chemin, &cles::Cles { google: cle }),
            None => Err("aucun dossier pour la garder".into()),
        };
        let note = match ecrit {
            Ok(()) => "Clé enregistrée sur ce Glucose.".to_string(),
            Err(e) => format!("La clé n'a pas pu s'enregistrer ({e})."),
        };
        self.lancement.agents.conversation = None;
        self.montrer_l_api_publique(Some(note));
    }
}
