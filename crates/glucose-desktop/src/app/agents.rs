//! **Les agents, vus de la boucle** (fiche 75) : la porte ouverte au lancement, l'accord de
//! l'utilisateur (AGENTS-1), et les appels qui attendent leur tour.
//!
//! Un appel attend tant que :
//!
//! * l'utilisateur n'a pas répondu à la question — elle se pose dès qu'aucune autre ne l'est ;
//!   on ne chasse jamais la sienne ;
//! * il est au milieu d'un geste : ce que l'agent écrirait entrerait dans son geste à lui
//!   (AGENT-1). Son relâchement réveille la boucle, et l'appel passe.

use crate::agents::json::Json;
use crate::agents::porte::{self, Porte};
use crate::agents::{protocole, Boite, Demande};
use crate::app::GlucoseApp;
use crate::plateforme::Reveil;
use crate::ui::question::{Question, Reponse, Suite};
use std::collections::{HashMap, VecDeque};

/// Ce que la boucle sait des agents.
#[derive(Default)]
pub struct Agents {
    boite: Option<Boite>,
    /// Tenue pour que la prise Unix vive autant que Glucose ; sous Windows, le tuyau vit avec
    /// son fil.
    _porte: Option<Porte>,
    /// L'accord de l'utilisateur, par agent, jusqu'à la fermeture de Glucose.
    accords: HashMap<String, bool>,
    attente: VecDeque<Demande>,
}

impl Agents {
    /// **Retire l'accord de chacun** : leur prochaine demande reposera la question.
    pub(crate) fn oublier(&mut self) {
        self.accords.clear();
    }

    /// Les agents qui travaillent avec son accord, par ordre de nom.
    fn autorises(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self
            .accords
            .iter()
            .filter(|(_, oui)| **oui)
            .map(|(c, _)| c.as_str())
            .collect();
        v.sort_unstable();
        v
    }

    /// Une boîte sans porte : ce qu'une épreuve y dépose arrive comme d'un agent.
    pub fn avec_la_boite(boite: Boite) -> Self {
        Self {
            boite: Some(boite),
            ..Self::default()
        }
    }
}

/// Le titre de la question d'accord.
const TITRE: &str = "Un agent veut travailler ici";

fn question(client: &str) -> Question {
    Question {
        titre: TITRE.into(),
        texte: format!(
            "Un programme qui se présente comme « {client} » demande à lire ce tableau et à y \
             travailler sous tes yeux : poser des textes, des pages web, des flèches. Chacun de \
             ses gestes s'annule par Ctrl+Z.\n\nTon choix vaut jusqu'à la fermeture de Glucose."
        ),
        choix: vec![
            ("Autoriser".into(), Reponse::Oui),
            ("Refuser".into(), Reponse::Non),
            ("Plus tard".into(), Reponse::Annuler),
        ],
        ..Default::default()
    }
}

impl GlucoseApp {
    /// **Ouvre la porte** (au vrai lancement seulement : une épreuve n'en ouvre aucune).
    pub(crate) fn ouvrir_la_porte_des_agents(&mut self, reveil: Reveil) {
        let boite = Boite::nouvelle(reveil);
        let nom = porte::nom_par_defaut();
        let porte = nom
            .as_deref()
            .and_then(|nom| Porte::ouvrir(nom, boite.clone()));
        // Le téléphone n'a pas de porte : le dire tel, et non comme un refus (fiche 83).
        match (&nom, &porte) {
            (None, _) => println!("[Glucose] porte des agents : aucune sur ce système"),
            (Some(_), None) => println!(
                "[Glucose] porte des agents : un autre Glucose la tient, ou le système la refuse"
            ),
            _ => {}
        }
        self.lancement.agents = Agents {
            boite: Some(boite),
            _porte: porte,
            ..Agents::default()
        };
    }

    /// **Ce que les agents demandent**, dans l'ordre où ils l'ont demandé.
    pub(crate) fn suivre_les_agents(&mut self) {
        let agents = &mut self.lancement.agents;
        if let Some(boite) = &agents.boite {
            agents.attente.extend(boite.relever());
        }
        while let Some(client) = self
            .lancement
            .agents
            .attente
            .front()
            .map(|d| d.appel.client.clone())
        {
            match self.lancement.agents.accords.get(&client) {
                Some(true) if self.store.in_live_edit() => return,
                Some(true) => {
                    let Some(d) = self.lancement.agents.attente.pop_front() else {
                        return;
                    };
                    let issue = self.executer_pour_un_agent(&d.appel.nom, &d.appel.arguments);
                    repondre(d, issue);
                }
                Some(false) => {
                    self.refuser_le_premier(&client, "l'utilisateur ne l'a pas autorisé")
                }
                None => {
                    if self.ui.question.is_none() {
                        self.demander(question(&client), Suite::Agent(client));
                    }
                    return;
                }
            }
        }
    }

    /// **Sa réponse** : oui ou non vaut jusqu'à la fermeture ; « Plus tard » refuse ce seul
    /// appel, et la question reviendra au suivant.
    pub(crate) fn repondre_a_l_agent(&mut self, client: String, reponse: Reponse) {
        match reponse {
            Reponse::Oui => {
                self.lancement.agents.accords.insert(client, true);
            }
            Reponse::Non => {
                self.lancement.agents.accords.insert(client, false);
            }
            _ => self.refuser_le_premier(&client, "l'utilisateur a remis sa réponse à plus tard"),
        }
        self.suivre_les_agents();
    }

    /// **« Retirer l'accord des agents… »** : la question dit qui l'a, et le retire sur sa parole.
    pub(crate) fn demander_a_oublier_les_agents(&mut self) {
        let autorises = self.lancement.agents.autorises();
        let (texte, choix) = if autorises.is_empty() {
            (
                "Aucun agent ne travaille dans Glucose avec ton accord.".to_string(),
                vec![("Fermer".into(), Reponse::Annuler)],
            )
        } else {
            let noms: Vec<String> = autorises.iter().map(|c| format!("« {c} »")).collect();
            (
                format!(
                    "Ont ton accord : {}.\n\nLe retirer ? Leur prochaine demande te sera reposée.",
                    noms.join(", ")
                ),
                vec![
                    ("Retirer l'accord".into(), Reponse::Oui),
                    ("Annuler".into(), Reponse::Annuler),
                ],
            )
        };
        let question = Question {
            titre: "Les agents".into(),
            texte,
            choix,
            ..Default::default()
        };
        self.demander(question, Suite::OublierLesAgents);
    }

    fn refuser_le_premier(&mut self, client: &str, pourquoi: &str) {
        if let Some(d) = self.lancement.agents.attente.pop_front() {
            repondre(
                d,
                Err(format!(
                    "« {client} » ne peut pas travailler dans Glucose : {pourquoi}"
                )),
            );
        }
    }
}

fn repondre(d: Demande, issue: Result<Json, String>) {
    // L'agent est peut-être parti : sa réponse n'a plus personne à qui aller.
    let _ = d.reponse.send(protocole::reponse_d_outil(&d.appel, issue));
}

#[cfg(test)]
mod tests;
