//! **Collaborer en trois portes** (sa proposition du 24/11, `relais-rep-321` ; fiche 154) : le
//! bouton ouvre trois portes — **les humains** (partager, rejoindre, les codes, son nom), **les IA**
//! (la porte MCP du nuage, l'API publique, chacune avec sa mise en garde), **la sécurité** (qui a
//! accès, et le lui retirer) — puis chacune ce qu'elle tient. Une seule question sur deux niveaux :
//! toutes ses réponses passent par [`GlucoseApp::repondre_a_collaborer`], « Retour » remonte.

use super::{photos, CanevasPartage, COPIER_ECRIRE, COPIER_LIRE, NOMMER, PORTE, QUITTER};
use crate::app::GlucoseApp;
use crate::ui::question::{Question, Reponse, Suite};

const PORTE_HUMAIN: Reponse = Reponse::Choix(7);
const PORTE_IA: Reponse = Reponse::Choix(8);
const SECURITE: Reponse = Reponse::Choix(9);
const RETOUR: Reponse = Reponse::Choix(15);
const LES_IA_DU_NUAGE: Reponse = Reponse::Choix(16);
const LES_PROGRAMMES: Reponse = Reponse::Choix(17);

type Choix = (String, Reponse);

fn choix(libelle: &str, reponse: Reponse) -> Choix {
    (libelle.into(), reponse)
}

impl GlucoseApp {
    /// **« Collaborer »** : les trois portes, et d'un coup d'œil où en est chacune.
    pub(crate) fn montrer_collaborer(&mut self) {
        let humains = match &self.lancement.agents.partage {
            None => "Porte humain : ce canevas n'est pas partagé.".to_string(),
            Some(cp) => format!("Porte humain : {}", self.etat_du_partage(cp)),
        };
        let texte = format!(
            "{humains}\n\nPorte IA : {}.\n\nSécurité : qui a accès à ce canevas, et le lui retirer.",
            self.etat_des_ia()
        );
        let question = Question {
            titre: "Collaborer".into(),
            texte,
            choix: vec![
                choix("Porte humain…", PORTE_HUMAIN),
                choix("Porte IA…", PORTE_IA),
                choix("Sécurité…", SECURITE),
                choix("Fermer", Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::Collaborer);
    }

    /// **La porte humain** : seul, partager ou rejoindre ; partagé, les codes, qui est là,
    /// quitter — avec ce qui vient d'avoir lieu en tête (un code copié, un partage qui n'a pas pu
    /// se faire) : ce qu'on a fait se dit là où on l'a fait.
    pub(crate) fn montrer_la_porte_humain(&mut self, note: Option<String>) {
        let nommer = choix("Ton nom et ta couleur…", NOMMER);
        let retour = choix("Retour", RETOUR);
        let mut question = match &self.lancement.agents.partage {
            None => self.question_seul(nommer, retour),
            Some(cp) => {
                let mut liste = Vec::new();
                if cp.code.peut_ecrire() {
                    liste.push(choix("Copier le code pour y travailler", COPIER_ECRIRE));
                }
                liste.push(choix("Copier le code pour regarder", COPIER_LIRE));
                liste.push(choix("Quitter le partage", QUITTER));
                liste.extend([nommer, retour, choix("Fermer", Reponse::Annuler)]);
                Question {
                    titre: "Porte humain".into(),
                    texte: self.etat_du_partage(cp),
                    choix: liste,
                    ..Default::default()
                }
            }
        };
        if let Some(n) = note {
            question.texte = format!("{n}\n\n{}", question.texte);
        }
        self.demander(question, Suite::Collaborer);
    }

    /// **La porte IA** : la porte MCP du nuage et l'API publique, chacune ouvrant sa fenêtre — et
    /// sa mise en garde.
    fn montrer_la_porte_ia(&mut self) {
        let texte = format!(
            "{}.\n\nLa porte aux IA publiques : Claude.ai, ou une autre IA qui parle MCP, y entre \
             après ton accord ; elle ne voit que le document ouvert.\n\nL'API publique : des IA \
             gratuites (Google AI Studio) discutent avec toi dans un tiroir, et travaillent dans \
             ce document.\n\nChaque geste d'une IA est signé de son nom et s'annule par Ctrl+Z.",
            majuscule(&self.etat_des_ia())
        );
        let question = Question {
            titre: "Porte IA".into(),
            texte,
            choix: vec![
                choix("La porte aux IA publiques…", PORTE),
                choix("L'API publique…", super::api::MENU),
                choix("Retour", RETOUR),
                choix("Fermer", Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::Collaborer);
    }

    /// **Sécurité** : qui a accès à ce canevas — les humains par leur code, les IA du nuage par la
    /// porte, les programmes de cette machine par ton accord —, et de quoi le leur retirer.
    fn montrer_la_securite(&mut self) {
        let humains = match &self.lancement.agents.partage {
            None => "Humains : ce canevas n'est pas partagé ; personne n'y entre.".to_string(),
            Some(cp) => self.les_humains(cp),
        };
        let porte = match self.lancement.agents.publique {
            Some(_) => "ouverte",
            None => "fermée",
        };
        let programmes = self.qui_a_ton_accord();
        let programmes = if programmes.is_empty() {
            "aucun n'a ton accord.".to_string()
        } else {
            format!("ont ton accord : {}.", programmes.join(", "))
        };
        let question = Question {
            titre: "Sécurité".into(),
            texte: format!(
                "{humains}\n\nIA du nuage — la porte est {porte}. {}\n\nProgrammes de cette \
                 machine (Claude Code, par glucose --mcp) : {programmes}",
                self.qui_a_la_porte()
            ),
            choix: vec![
                choix("Les IA du nuage…", LES_IA_DU_NUAGE),
                choix("Les programmes de cette machine…", LES_PROGRAMMES),
                choix("Retour", RETOUR),
                choix("Fermer", Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::Collaborer);
    }

    /// Les réponses des trois portes ; celles de chacune, [`GlucoseApp::repondre_a_collaborer`]
    /// les a déjà prises.
    pub(super) fn repondre_aux_portes(&mut self, reponse: Reponse) {
        match reponse {
            PORTE_HUMAIN => self.montrer_la_porte_humain(None),
            PORTE_IA => self.montrer_la_porte_ia(),
            SECURITE => self.montrer_la_securite(),
            RETOUR => self.montrer_collaborer(),
            LES_IA_DU_NUAGE => self.montrer_les_ia(),
            LES_PROGRAMMES => self.demander_a_oublier_les_agents(),
            _ => {}
        }
    }

    /// Le canevas partagé : le lien, son nom, qui est là, les photos qui attendent, ce que son
    /// code permet.
    fn etat_du_partage(&self, cp: &CanevasPartage) -> String {
        use crate::agents::publique::lien::EtatDuLien;
        let lien = match cp.lien.etat() {
            EtatDuLien::Relie if cp.entree.attend() => "Relié ; le canevas arrive…".into(),
            EtatDuLien::Relie => "Relié.".to_string(),
            EtatDuLien::EnRoute => "Il se relie…".to_string(),
            EtatDuLien::Hors(r) => format!("Pas relié ({r}) : il réessaie."),
        };
        let pouvoir = if cp.code.peut_ecrire() {
            ""
        } else {
            "\n\nTon code ne fait que regarder : tes gestes ne partent pas."
        };
        let photos = self.photos_en_attente();
        let avec = avec(cp);
        format!("{lien} Tu es « {} ».\n\n{avec}{photos}{pouvoir}", cp.nom)
    }

    /// Dans Sécurité : les codes qui ouvrent la salle, qui y est, les photos qui attendent.
    fn les_humains(&self, cp: &CanevasPartage) -> String {
        let codes = if cp.code.peut_ecrire() {
            "deux codes ouvrent ta salle, l'un pour y travailler, l'autre pour regarder ; ils se \
             copient par la porte humain."
        } else {
            "tu y es entré par le code qui regarde."
        };
        let photos = self.photos_en_attente();
        format!("Humains : {codes}\n{}{photos}", avec(cp))
    }

    fn photos_en_attente(&self) -> String {
        photos::en_attente(&self.store.project, &self.disque.objets)
            .map_or_else(String::new, |p| format!("\n\n{p}"))
    }

    /// La porte du nuage et la clé de l'API publique, en une ligne.
    fn etat_des_ia(&self) -> String {
        let porte = match self.lancement.agents.publique {
            Some(_) => "la porte aux IA publiques est ouverte",
            None => "la porte aux IA publiques est fermée",
        };
        let cle = match self.cle_google() {
            Some(_) => "ta clé de l'API publique est branchée",
            None => "aucune clé pour l'API publique",
        };
        format!("{porte} ; {cle}")
    }
}

/// Qui est là avec soi.
fn avec(cp: &CanevasPartage) -> String {
    let noms: Vec<&str> = cp.presents().iter().map(|p| p.nom.as_str()).collect();
    if noms.is_empty() {
        "Personne d'autre pour l'instant.".to_string()
    } else {
        format!("Avec : {}.", noms.join(", "))
    }
}

fn majuscule(texte: &str) -> String {
    let mut lettres = texte.chars();
    lettres.next().map_or_else(String::new, |p| {
        p.to_uppercase().chain(lettres).collect::<String>()
    })
}

#[cfg(test)]
mod tests;
