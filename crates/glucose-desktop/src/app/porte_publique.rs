//! **La porte publique, vue de la boucle** (fiche 116) : le bouton **Collaborer** l'ouvre et la
//! ferme, la question d'accord montre le code de la page de l'IA, la liste dit qui a la porte et
//! la retire, et un témoin dit quand une IA du nuage travaille.
//!
//! L'état vit à deux endroits, jamais en double : sur le disque quand la porte est fermée
//! (`porte-publique.json`), dans le guichet du lien quand elle est ouverte — et le guichet
//! l'écrit à chaque accord.

use crate::agents::publique::accords::{self, Droits, Etat};
use crate::agents::publique::lien::{self, EtatDuLien};
use crate::agents::publique::oauth::Issue;
use crate::app::GlucoseApp;
use crate::ui::question::{Question, Reponse, Suite};
use std::sync::PoisonError;
use std::time::{Duration, Instant};

/// La question d'accord : les trois réponses.
const LIRE_ET_ECRIRE: Reponse = Reponse::Choix(0);
const LIRE_SEULEMENT: Reponse = Reponse::Choix(1);
/// La question de la porte ouverte.
const COPIER: Reponse = Reponse::Choix(0);
const VOIR_LES_IA: Reponse = Reponse::Choix(1);
const FERMER_LA_PORTE: Reponse = Reponse::Choix(2);
/// **Le témoin** : une IA qui reprend après ce silence se dit une fois — pas à chaque geste.
const SILENCE: Duration = Duration::from_secs(60);

impl GlucoseApp {
    /// **Au lancement** : une porte laissée ouverte se rouvre. Seul le Glucose qui tient la porte
    /// locale le fait : deux Glucose du même compte se disputeraient la même adresse.
    pub(crate) fn rouvrir_la_porte_publique(&mut self) {
        match accords::charger(&self.disque.porte_publique) {
            Ok(etat) if etat.ouverte => self.lancer_le_lien(etat),
            Ok(_) => {}
            Err(e) => crate::dire!("[Glucose] porte publique : {e}"),
        }
    }

    fn lancer_le_lien(&mut self, etat: Etat) {
        let Some(boite) = self.lancement.agents.boite.clone() else {
            return;
        };
        let fichier = self.disque.porte_publique.clone();
        let racine = self
            .lancement
            .agents
            .relais
            .clone()
            .unwrap_or_else(crate::agents::publique::relais);
        self.lancement.agents.publique = Some(lien::lancer(&racine, etat, fichier, boite));
        self.ui.porte_ouverte = true;
    }

    /// L'état de la porte : celui du guichet si elle est ouverte, celui du disque sinon.
    fn etat_de_la_porte(&self) -> Result<Etat, String> {
        match &self.lancement.agents.publique {
            Some(l) => Ok(l
                .guichet
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .etat
                .clone()),
            None => accords::charger(&self.disque.porte_publique),
        }
    }

    /// **« Collaborer »** : la porte aux IA publiques — fermée, ce qu'elle ferait ; ouverte, son
    /// adresse, son état et qui l'a.
    pub(crate) fn montrer_la_porte_publique(&mut self) {
        let question = match &self.lancement.agents.publique {
            None => Question {
                titre: "La porte aux IA publiques".into(),
                texte: "Fermée. Ouverte, elle donne à ce Glucose une adresse que tu ajoutes à \
                        Claude.ai (Personnaliser → Connecteurs → Ajouter un connecteur \
                        personnalisé) ou à une autre IA qui parle MCP. Une IA n'y entre qu'après \
                        ton accord, ici, devant le même code que sur sa page ; elle ne voit que le \
                        document ouvert, et chacun de ses gestes est signé de son nom et s'annule \
                        par Ctrl+Z.\n\nLe co-working entre humains passera par la même porte."
                    .into(),
                choix: vec![
                    ("Ouvrir la porte".into(), Reponse::Oui),
                    ("Annuler".into(), Reponse::Annuler),
                ],
                ..Default::default()
            },
            Some(l) => {
                let etat = match l.etat() {
                    EtatDuLien::Relie => "Ouverte, et reliée.".to_string(),
                    EtatDuLien::EnRoute => "Ouverte ; elle se relie…".to_string(),
                    EtatDuLien::Hors(r) => {
                        format!("Ouverte, mais pas reliée ({r}) : elle réessaie.")
                    }
                };
                let ia = self.qui_a_la_porte();
                Question {
                    titre: "La porte aux IA publiques".into(),
                    texte: format!("{etat}\n\nTon adresse :\n{}\n\n{ia}", l.adresses.mcp()),
                    choix: vec![
                        ("Copier l'adresse".into(), COPIER),
                        ("Les IA qui ont ta porte…".into(), VOIR_LES_IA),
                        ("Fermer la porte".into(), FERMER_LA_PORTE),
                        ("Fermer".into(), Reponse::Annuler),
                    ],
                    ..Default::default()
                }
            }
        };
        self.demander(question, Suite::PortePublique);
    }

    fn qui_a_la_porte(&self) -> String {
        let accords = self
            .etat_de_la_porte()
            .map(|e| e.accords)
            .unwrap_or_default();
        if accords.is_empty() {
            return "Aucune IA n'a encore ta porte.".into();
        }
        let noms: Vec<String> = accords
            .iter()
            .map(|a| format!("{} — {}", a.signature(), mots_des_droits(a.droits)))
            .collect();
        format!("Ont ta porte : {}.", noms.join(" ; "))
    }

    /// Sa réponse à la question de la porte.
    pub(crate) fn repondre_a_la_porte(&mut self, reponse: Reponse) {
        match (self.lancement.agents.publique.is_some(), reponse) {
            (false, Reponse::Oui) => self.ouvrir_la_porte_publique(),
            (true, COPIER) => {
                if let Some(l) = &self.lancement.agents.publique {
                    let adresse = l.adresses.mcp();
                    let _ = crate::interactions::presse_papiers::ouvrir()
                        .and_then(|mut a| a.ecrire(adresse));
                }
            }
            (true, VOIR_LES_IA) => self.montrer_les_ia(),
            (true, FERMER_LA_PORTE) => self.fermer_la_porte_publique(),
            _ => {}
        }
    }

    /// **Ouvre la porte** : l'état s'écrit ouvert, le lien part, et la question revient — avec
    /// l'adresse à copier.
    fn ouvrir_la_porte_publique(&mut self) {
        let mut etat = match accords::charger(&self.disque.porte_publique) {
            Ok(e) => e,
            Err(e) => {
                crate::dire_erreur!("[Glucose] porte publique : {e}");
                return;
            }
        };
        etat.ouverte = true;
        if let Err(e) = accords::enregistrer(&self.disque.porte_publique, &etat) {
            crate::dire_erreur!("[Glucose] porte publique : {e}");
            return;
        }
        self.lancer_le_lien(etat);
        self.montrer_la_porte_publique();
    }

    /// **Ferme la porte** : le lien s'arrête, plus rien n'entre ; les IA gardent leur place, et
    /// la retrouvent à la prochaine ouverture.
    fn fermer_la_porte_publique(&mut self) {
        let Some(l) = self.lancement.agents.publique.take() else {
            return;
        };
        let mut etat = l
            .guichet
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .etat
            .clone();
        etat.ouverte = false;
        if let Err(e) = accords::enregistrer(&self.disque.porte_publique, &etat) {
            crate::dire_erreur!("[Glucose] porte publique : {e}");
        }
        self.ui.porte_ouverte = false;
    }

    /// **Les IA qui ont ta porte**, chacune à retirer d'un clic.
    fn montrer_les_ia(&mut self) {
        let accords = self
            .etat_de_la_porte()
            .map(|e| e.accords)
            .unwrap_or_default();
        let mut choix: Vec<(String, Reponse)> = accords
            .iter()
            .enumerate()
            .map(|(i, a)| (format!("Retirer {}", a.signature()), Reponse::Choix(i)))
            .collect();
        choix.push(("Fermer".into(), Reponse::Annuler));
        let texte = if accords.is_empty() {
            "Aucune IA n'a ta porte.".to_string()
        } else {
            "Une IA retirée perd sa clé à la seconde : sa prochaine demande est refusée, et elle \
             devra te redemander la porte."
                .to_string()
        };
        let question = Question {
            titre: "Les IA qui ont ta porte".into(),
            texte,
            choix,
            ..Default::default()
        };
        self.demander(question, Suite::RetirerUneIa);
    }

    /// Retire l'IA choisie, et l'écrit.
    pub(crate) fn retirer_une_ia(&mut self, reponse: Reponse) {
        let Reponse::Choix(i) = reponse else {
            return;
        };
        let Some(l) = &self.lancement.agents.publique else {
            return;
        };
        let mut g = l.guichet.lock().unwrap_or_else(PoisonError::into_inner);
        if i < g.etat.accords.len() {
            g.etat.accords.remove(i);
            if let Err(e) = accords::enregistrer(&self.disque.porte_publique, &g.etat) {
                crate::dire_erreur!("[Glucose] porte publique : {e}");
            }
        }
    }

    /// **À chaque tour de la boucle** : une IA qui demande la porte reçoit sa question — dès
    /// qu'aucune autre n'est posée, une seule fois.
    pub(crate) fn suivre_la_porte_publique(&mut self) {
        if self.ui.question.is_some() {
            return;
        }
        let Some(l) = &self.lancement.agents.publique else {
            return;
        };
        let a_poser = {
            let mut g = l.guichet.lock().unwrap_or_else(PoisonError::into_inner);
            g.autorisations
                .iter_mut()
                .find(|a| !a.posee && a.issue == Issue::Attente)
                .map(|a| {
                    a.posee = true;
                    (
                        a.demande.clone(),
                        a.code_affiche.clone(),
                        a.client.nom.clone(),
                        a.client.hote.clone(),
                    )
                })
        };
        if let Some((demande, code, nom, hote)) = a_poser {
            self.demander(
                question_d_accord(&code, &nom, &hote),
                Suite::AccordPublic(demande),
            );
        }
    }

    /// **Sa réponse à une IA** : elle part au guichet, où la page de l'IA la lira. Échap refuse.
    pub(crate) fn repondre_a_l_ia(&mut self, demande: &str, reponse: Reponse) {
        let issue = match reponse {
            LIRE_ET_ECRIRE => Issue::Accordee(Droits::Ecrire),
            LIRE_SEULEMENT => Issue::Accordee(Droits::Lire),
            _ => Issue::Refusee,
        };
        if let Some(l) = &self.lancement.agents.publique {
            let mut g = l.guichet.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(a) = g.autorisations.iter_mut().find(|a| a.demande == demande) {
                a.issue = issue;
            }
        }
    }

    /// **Le témoin** : une IA du nuage qui se met à travailler dans le canevas le dit, une fois
    /// par reprise.
    pub(crate) fn temoigner(&mut self, signature: &str) {
        let maintenant = Instant::now();
        let reprise = self
            .lancement
            .agents
            .dernier_geste_public
            .is_none_or(|t| maintenant.duration_since(t) > SILENCE);
        self.lancement.agents.dernier_geste_public = Some(maintenant);
        if reprise {
            self.ui
                .show_toast(format!("{signature} travaille dans ton canevas"));
        }
    }
}

fn mots_des_droits(d: Droits) -> &'static str {
    match d {
        Droits::Ecrire => "lire et écrire",
        Droits::Lire => "lire seulement",
    }
}

/// **La question d'accord** : qui, d'où, le code de sa page — et ce que chaque réponse donne.
fn question_d_accord(code: &str, nom: &str, hote: &str) -> Question {
    let code: String = code.chars().map(String::from).collect::<Vec<_>>().join(" ");
    Question {
        titre: "Une IA demande ta porte".into(),
        texte: format!(
            "« {nom} », depuis {hote}, demande l'accès au document ouvert : le lire, et y \
             travailler sous tes yeux.\n\nCode : {code} — le même que sur la page de ton IA ? \
             Sinon, refuse.\n\nChacun de ses gestes sera signé de son nom et s'annulera par \
             Ctrl+Z ; Collaborer la retire quand tu veux."
        ),
        choix: vec![
            ("Lire et écrire".into(), LIRE_ET_ECRIRE),
            ("Lire seulement".into(), LIRE_SEULEMENT),
            ("Refuser".into(), Reponse::Non),
        ],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests;
