//! **Les agents, vus de la boucle** (fiche 75) : la porte ouverte au lancement, l'accord de
//! l'utilisateur (AGENTS-1), et les appels qui attendent leur tour.
//!
//! Un appel attend tant que :
//!
//! * l'utilisateur n'a pas répondu à la question pour cet agent — elle se pose dès qu'aucune
//!   autre ne l'est ; on ne chasse jamais la sienne ;
//! * il **écrit** et l'utilisateur est au milieu d'un geste : ce que l'agent écrirait entrerait
//!   dans son geste à lui (AGENT-1). Son relâchement réveille la boucle, et l'appel passe. Une
//!   **lecture**, elle, passe toujours : elle ne peut rien écrire dans ce geste (LECTURE-1) ;
//! * un appel **du même agent**, plus ancien, attend encore : chaque agent est servi dans l'ordre
//!   où il a demandé (sa lecture voit ce qu'il vient d'écrire), mais **aucun n'attend derrière un
//!   autre** (LECTURE-1, fiche 130).
//!
//! Le 22/11/2026, la roue des couleurs restée ouverte chez lui a retenu treize minutes toutes les
//! demandes — la simple lecture de sa consigne par le relais comprise —, parce que la file
//! entière attendait derrière son geste (fiche 130).

use crate::agents::json::Json;
use crate::agents::outils;
use crate::agents::porte::{self, Porte};
use crate::agents::{protocole, Boite, Demande};
use crate::app::GlucoseApp;
use crate::plateforme::Reveil;
use crate::ui::question::{Question, Reponse, Suite};
use std::collections::{HashMap, VecDeque};

/// Ce que la boucle sait des agents.
#[derive(Default)]
pub struct Agents {
    pub(crate) boite: Option<Boite>,
    /// Tenue pour que la prise Unix vive autant que Glucose ; sous Windows, le tuyau vit avec
    /// son fil.
    _porte: Option<Porte>,
    /// L'accord de l'utilisateur, par agent, jusqu'à la fermeture de Glucose.
    accords: HashMap<String, bool>,
    attente: VecDeque<Demande>,
    /// **La discussion avec une IA gratuite** (fiche 129) : la question en route, s'il y en a une…
    pub(crate) discussion: Option<crate::ia::discussion::EnRoute>,
    /// … ou la conversation au repos, qui garde son histoire et ce que la bascule a appris.
    pub(crate) conversation: Option<crate::ia::discussion::Conversation>,
    /// **Le fichier de la clé des IA gratuites** : celui du dossier de Glucose, posé au vrai
    /// lancement seulement. Une épreuve n'en a aucun : elle ne lit jamais la clé de l'utilisateur,
    /// ne parle jamais à Google, n'écrit jamais dans son dossier (fiche 129).
    pub(crate) cles: Option<std::path::PathBuf>,
    /// **La porte aux IA publiques**, quand elle est ouverte (fiche 116).
    pub(crate) publique: Option<crate::agents::publique::lien::Lien>,
    /// **Le canevas partagé**, s'il l'est (fiche 117) : le fil vers sa salle, son code, les autres
    /// — Collaborer reçoit les humains comme les IA du nuage, par le même relais.
    pub(crate) partage: Option<crate::app::collaborer::CanevasPartage>,
    /// **La portée de l'appel en cours** : ce que l'outil peut atteindre — toute la machine, ou
    /// le document ouvert pour une IA du nuage (C1, fiche 116).
    pub(crate) portee: crate::agents::outils::Portee,
    /// Le dernier geste d'une IA du nuage : son témoin ne se répète pas à chaque geste.
    pub(crate) dernier_geste_public: Option<std::time::Instant>,
    /// L'onglet de son dernier geste : le témoin le redit quand elle en change (TEMOIN-2).
    pub(crate) dernier_onglet_public: Option<String>,
    /// **Le relais** de la porte publique : celui de Glucose, ou un autre — un relais local, dans
    /// les épreuves.
    pub(crate) relais: Option<String>,
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

    /// **La première demande qui peut passer** (LECTURE-1, fiche 130). Chaque agent est servi
    /// dans l'ordre où il a demandé, mais aucun n'attend derrière un autre ; pendant le geste de
    /// l'humain, seule une lecture passe (AGENT-1).
    fn prochaine(&self, geste_ouvert: bool) -> Prochaine {
        let mut retenus: Vec<&str> = Vec::new();
        let mut a_demander = None;
        for (i, d) in self.attente.iter().enumerate() {
            let client = d.appel.client.as_str();
            if retenus.contains(&client) {
                continue;
            }
            // Une IA du nuage a déjà son accord : la clé que Glucose lui a donnée après son clic
            // (PUBLIQUE-1, fiche 116).
            let accord = if d.appel.portee.machine {
                self.accords.get(client).copied()
            } else {
                Some(true)
            };
            match accord {
                Some(false) => return Prochaine::Refuser(i),
                Some(true) if !geste_ouvert || outils::lit_seulement(&d.appel.nom) => {
                    return Prochaine::Servir(i)
                }
                Some(true) => {}
                None => {
                    a_demander.get_or_insert_with(|| client.to_string());
                }
            }
            retenus.push(client);
        }
        Prochaine::Rien { a_demander }
    }
}

/// Ce que la file permet maintenant.
enum Prochaine {
    /// Servir la demande à ce rang.
    Servir(usize),
    /// La refuser : l'utilisateur a refusé son agent.
    Refuser(usize),
    /// Rien ne peut passer ; le premier agent sans réponse de l'utilisateur, s'il y en a un.
    Rien { a_demander: Option<String> },
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
            (None, _) => crate::dire!("[Glucose] porte des agents : aucune sur ce système"),
            (Some(_), None) => crate::dire!(
                "[Glucose] porte des agents : un autre Glucose la tient, ou le système la refuse"
            ),
            _ => {}
        }
        let tenue = porte.is_some();
        self.lancement.agents = Agents {
            boite: Some(boite),
            _porte: porte,
            cles: Some(crate::ia::cles::chemin()),
            ..Agents::default()
        };
        // La porte publique ne se rouvre que chez le Glucose qui tient la porte locale : un
        // second, ouvert en même temps, prendrait l'adresse au premier (fiche 116).
        if tenue {
            self.rouvrir_la_porte_publique();
        }
    }

    /// **Ce que les agents demandent**, chacun dans l'ordre où il l'a demandé (LECTURE-1) — la
    /// discussion d'abord, qui en est un (DISCUSSION-1, fiche 129).
    pub(crate) fn suivre_les_agents(&mut self) {
        self.suivre_la_discussion();
        let agents = &mut self.lancement.agents;
        if let Some(boite) = &agents.boite {
            agents.attente.extend(boite.relever());
        }
        loop {
            match self.lancement.agents.prochaine(self.store.in_live_edit()) {
                Prochaine::Servir(i) => self.servir(i),
                Prochaine::Refuser(i) => {
                    if let Some(d) = self.lancement.agents.attente.remove(i) {
                        let client = d.appel.client.clone();
                        refuser(d, &client, "l'utilisateur ne l'a pas autorisé");
                    }
                }
                Prochaine::Rien { a_demander } => {
                    if let Some(client) = a_demander.filter(|_| self.ui.question.is_none()) {
                        self.demander(question(&client), Suite::Agent(client));
                    }
                    return;
                }
            }
        }
    }

    /// **Sert la demande à ce rang** : le témoin d'une IA du nuage, puis l'outil.
    fn servir(&mut self, rang: usize) {
        let Some(d) = self.lancement.agents.attente.remove(rang) else {
            return;
        };
        if !d.appel.portee.machine {
            // L'onglet que l'appel vise : le sien, ou celui qu'il regarde.
            let onglet = d.appel.arguments.get("tableau").and_then(Json::as_str);
            let onglet = onglet.map_or_else(
                || self.store.project.active_board_id.clone(),
                str::to_string,
            );
            self.temoigner(&d.appel.client, &onglet);
        }
        let issue = self.executer_dans_sa_portee(&d.appel);
        repondre(d, issue);
    }

    /// **Exécute un appel** sous le nom de son agent (MAIN-1 : ce qu'il écrit porte son nom dans
    /// l'histoire) et dans sa portée.
    fn executer_dans_sa_portee(&mut self, appel: &protocole::Appel) -> Result<Json, String> {
        let main = glucose_core::store::journal::Main::Agent(appel.client.clone());
        let avant = std::mem::replace(&mut self.store.journal.main, main);
        let portee = std::mem::replace(&mut self.lancement.agents.portee, appel.portee);
        let issue = self.executer_pour_un_agent(&appel.nom, &appel.arguments);
        self.lancement.agents.portee = portee;
        self.store.journal.main = avant;
        issue
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

    /// Refuse **la première demande de cet agent** — pas la première de la file : un autre agent
    /// peut attendre devant lui (LECTURE-1).
    fn refuser_le_premier(&mut self, client: &str, pourquoi: &str) {
        let attente = &mut self.lancement.agents.attente;
        let rang = attente.iter().position(|d| d.appel.client == client);
        if let Some(d) = rang.and_then(|r| attente.remove(r)) {
            refuser(d, client, pourquoi);
        }
    }
}

fn refuser(d: Demande, client: &str, pourquoi: &str) {
    repondre(
        d,
        Err(format!(
            "« {client} » ne peut pas travailler dans Glucose : {pourquoi}"
        )),
    );
}

fn repondre(d: Demande, issue: Result<Json, String>) {
    // L'agent est peut-être parti : sa réponse n'a plus personne à qui aller.
    let _ = d.reponse.send(protocole::reponse_d_outil(&d.appel, issue));
}

#[cfg(test)]
mod tests;
