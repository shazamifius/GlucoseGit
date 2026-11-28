//! **Collaborer, une seule fenêtre** (sa parole du 26/11, `text-c80i-2481` : « interface
//! vraiment bordélique […] il faut que ce soit compréhensible » ; fiche 192). Les trois portes de
//! la fiche 154 faisaient un labyrinthe de trois étages, chacun avec « Retour » et « Fermer », et
//! Sécurité répétait les deux autres. Comme la fenêtre Partager de Figma, Miro ou Notion, tout ce
//! qui touche aux humains tient désormais dans une seule fenêtre : où en est le partage, qui est
//! là, les codes, son nom, l'accord des programmes de cette machine. Les IA du nuage ont leur
//! étage, un seul, dont le titre dit le chemin (« Collaborer › IA »). Toutes ses réponses passent
//! par [`GlucoseApp::repondre_a_collaborer`], « Retour » remonte.

use super::{changer, photos, CanevasPartage, COPIER_ECRIRE, COPIER_LIRE, NOMMER, PORTE, QUITTER};
use crate::app::GlucoseApp;
use crate::ui::question::{Question, Reponse, Suite};

const IA: Reponse = Reponse::Choix(8);
const RETOUR: Reponse = Reponse::Choix(15);
const LES_IA_DU_NUAGE: Reponse = Reponse::Choix(16);
const LES_PROGRAMMES: Reponse = Reponse::Choix(17);

type Choix = (String, Reponse);

fn choix(libelle: &str, reponse: Reponse) -> Choix {
    (libelle.into(), reponse)
}

impl GlucoseApp {
    /// **« Collaborer »**, sa fenêtre.
    pub(crate) fn montrer_collaborer(&mut self) {
        self.montrer_collaborer_avec(None);
    }

    /// **La fenêtre de Collaborer** : seul, partager ou rejoindre ; partagé, où en est la salle,
    /// qui est là, les codes, quitter — puis son nom, les IA, les programmes de cette machine.
    /// Avec ce qui vient d'avoir lieu en tête (un code copié, un partage qui n'a pas pu se
    /// faire) : ce qu'on a fait se dit là où on l'a fait.
    pub(crate) fn montrer_collaborer_avec(&mut self, note: Option<String>) {
        let (partage, mut liste) = match &self.lancement.agents.partage {
            None => self.seul(),
            Some(cp) => {
                let mut liste = Vec::new();
                if cp.code.peut_ecrire() {
                    liste.push(choix("Copier le code pour y travailler", COPIER_ECRIRE));
                }
                liste.push(choix("Copier le code pour regarder", COPIER_LIRE));
                if cp.code.peut_ecrire() {
                    liste.push(choix("Changer les codes…", changer::CHANGER));
                }
                liste.push(choix("Quitter le partage", QUITTER));
                (self.etat_du_partage(cp), liste)
            }
        };
        liste.extend([
            choix("Ton nom et ta couleur…", NOMMER),
            choix("IA…", IA),
            choix("Les programmes de cette machine…", LES_PROGRAMMES),
            choix("Fermer", Reponse::Annuler),
        ]);
        let mut texte = format!(
            "{partage}\n\nIA : {}.\n\nProgrammes de cette machine (Claude Code, par glucose \
             --mcp) : {}",
            self.etat_des_ia(),
            self.les_programmes()
        );
        if let Some(n) = note {
            texte = format!("{n}\n\n{texte}");
        }
        let question = Question {
            titre: "Collaborer".into(),
            texte,
            choix: liste,
            ..Default::default()
        };
        self.demander(question, Suite::Collaborer);
    }

    /// **Collaborer › IA** : la porte MCP du nuage et l'API publique, chacune ouvrant sa fenêtre
    /// — et sa mise en garde —, et qui a la porte, à qui la retirer.
    fn montrer_les_ia_de_collaborer(&mut self) {
        let texte = format!(
            "{}.\n\nLa porte aux IA publiques : Claude.ai, ou une autre IA qui parle MCP, y entre \
             après ton accord ; elle ne voit que le document ouvert. {}\n\nL'API publique : des \
             IA gratuites (Google AI Studio) discutent avec toi dans un tiroir, et travaillent \
             dans ce document.\n\nChaque geste d'une IA est signé de son nom et s'annule par \
             Ctrl+Z.",
            majuscule(&self.etat_des_ia()),
            self.qui_a_la_porte()
        );
        let question = Question {
            titre: "Collaborer › IA".into(),
            texte,
            choix: vec![
                choix("La porte aux IA publiques…", PORTE),
                choix("L'API publique…", super::api::MENU),
                choix("Les IA du nuage…", LES_IA_DU_NUAGE),
                choix("Retour", RETOUR),
                choix("Fermer", Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::Collaborer);
    }

    /// Les réponses de la fenêtre et de son étage IA ; les autres,
    /// [`GlucoseApp::repondre_a_collaborer`] les a déjà prises.
    pub(super) fn repondre_aux_portes(&mut self, reponse: Reponse) {
        match reponse {
            IA => self.montrer_les_ia_de_collaborer(),
            changer::CHANGER => self.demander_de_changer_les_codes(),
            changer::CHANGER_VRAIMENT => self.changer_les_codes(),
            RETOUR => self.montrer_collaborer(),
            LES_IA_DU_NUAGE => self.montrer_les_ia(),
            LES_PROGRAMMES => self.demander_a_oublier_les_agents(),
            _ => {}
        }
    }

    /// Les programmes de cette machine qui ont son accord, en une phrase.
    fn les_programmes(&self) -> String {
        let accord = self.qui_a_ton_accord();
        if accord.is_empty() {
            "aucun n'a ton accord.".to_string()
        } else {
            format!("ont ton accord : {}.", accord.join(", "))
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
            "\n\nTon code ne fait que regarder : tu vois tout, tu ne modifies rien."
        };
        let photos = self.photos_en_attente();
        let avec = avec(cp);
        format!("{lien} Tu es « {} ».\n\n{avec}{photos}{pouvoir}", cp.nom)
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
