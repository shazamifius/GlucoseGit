//! **La boîte noire qui voyage** (fiches 49 et 54) : avec l'accord de chacun, les sessions closes
//! de la boîte noire partent au lancement suivant vers le serveur de l'utilisateur — un Worker
//! de Cloudflare dans son compte (`outils/telemetrie/`).
//!
//! # Les règles de la fiche 49 § 2, et où chacune se tient
//!
//! 1. **Éteinte par défaut** : rien ne part sans un « oui » ([`accord`]), demandé une fois, en
//!    français simple ; « non » ne change rien au programme, et le choix se reprend au menu.
//! 2. **Voir ce qui part** : ce sont **exactement** les lignes des fichiers de la boîte noire,
//!    que le menu ouvre.
//! 3. **Aucun mot de l'utilisateur** : vrai par le type des lignes
//!    ([`crate::boite_noire::enregistrement`]), et le serveur refuse toute autre valeur.
//! 4. **Des lots, au lancement suivant**, sur un fil à part : jamais pendant qu'on dessine.
//! 5. **Un identifiant tiré au hasard**, renouvelé quand on efface ce qui est parti.
//! 6. **Aucune adresse IP gardée**, et une page publique qui dit tout : `docs/TELEMETRIE.md`.

pub mod accord;
pub mod envoi;

use accord::Accord;
use std::path::PathBuf;

/// **L'adresse du serveur**, dans son compte Cloudflare — `None` tant qu'il n'est pas déployé :
/// rien ne se demande, rien ne part, et l'entrée du menu n'existe pas.
pub const ADRESSE: Option<&str> = None;

/// Ce que la télémétrie sait de cette machine.
#[derive(Debug, Default)]
pub struct Telemetrie {
    /// Le dossier de l'application — `None` dans les épreuves, qui n'en ont pas : rien ne s'y
    /// lit, rien n'en part.
    dossier: Option<PathBuf>,
    accord: Option<Accord>,
    /// La session en cours, qui ne part qu'au lancement suivant.
    courante: Option<PathBuf>,
}

impl Telemetrie {
    /// **Au lancement** : le dossier de l'application et la session qui commence. L'accord se
    /// relit ; un accord donné fait partir les sessions closes, sur un fil à part.
    pub fn habiter(dossier: PathBuf, courante: Option<PathBuf>) -> Self {
        let accord = Accord::charger(&dossier);
        let t = Self {
            dossier: Some(dossier),
            accord: Some(accord),
            courante,
        };
        t.envoyer_si_accorde();
        t
    }

    /// Faut-il poser la question ? Une fois, s'il y a un serveur et que personne n'a répondu.
    pub fn a_demander(&self) -> bool {
        ADRESSE.is_some() && self.accord.as_ref().is_some_and(|a| a.envoyer.is_none())
    }

    /// L'accord en cours : `Some(true)` si les sessions partent.
    pub fn accorde(&self) -> Option<bool> {
        self.accord.as_ref().and_then(|a| a.envoyer)
    }

    /// **La réponse de l'utilisateur**, retenue sur la machine. « Oui » fait partir tout de suite
    /// ce qui attend ; « non » après un « oui » efface ce qui est parti, et tire un nouvel
    /// identifiant — ce qui partira un jour ne se reliera plus à ce qui a été effacé, et ce qui
    /// est déjà parti ne repartira pas ([`envoi::ENVOYEES`] reste).
    pub fn repondre(&mut self, oui: bool) -> std::io::Result<()> {
        let (Some(dossier), Some(accord)) = (&self.dossier, &mut self.accord) else {
            return Ok(());
        };
        if !oui && accord.envoyer == Some(true) {
            if let Some(adresse) = ADRESSE {
                envoi::effacer_en_fond(adresse.to_string(), accord.installation.clone());
            }
            accord.installation = accord::nouvel_identifiant();
        }
        accord.envoyer = Some(oui);
        accord.retenir(dossier)?;
        self.envoyer_si_accorde();
        Ok(())
    }

    /// Le dossier de la boîte noire, que « voir ce qui part » ouvre.
    pub fn dossier_de_la_boite_noire(&self) -> Option<PathBuf> {
        self.dossier
            .as_ref()
            .map(|d| d.join(crate::boite_noire::DOSSIER))
    }

    fn envoyer_si_accorde(&self) {
        let (Some(adresse), Some(dossier), Some(accord)) = (ADRESSE, &self.dossier, &self.accord)
        else {
            return;
        };
        if accord.envoyer == Some(true) {
            envoi::envoyer_en_fond(
                adresse.to_string(),
                dossier.clone(),
                accord.installation.clone(),
                self.courante.clone(),
            );
        }
    }
}

/// **La question**, telle qu'elle s'affiche : ce qui part, pourquoi, et ce qui n'en part jamais.
pub const QUESTION: &str = "Aider à améliorer Glucose ?

    À chaque lancement, Glucose peut envoyer le journal technique de la session précédente :     le temps que chaque image a pris, le système et le processeur, la façon dont la session a     fini. Jamais le contenu de tes documents, ni un nom, ni un fichier, ni ton adresse.

    Tu peux voir exactement ce qui part (clic droit sur le canevas, puis Journal technique),     et changer d'avis à tout moment : ce qui est parti s'efface alors.";

/// **La question, quand on y revient par le menu** et que le journal part déjà.
pub const ARRETER: &str = "Glucose envoie le journal technique de chaque session.

    Continuer à l'envoyer ? « Non » l'arrête, et efface ce qui est déjà parti.";

impl crate::app::GlucoseApp {
    /// **La question, une seule fois** : à l'ouverture de la fenêtre, s'il y a un serveur et que
    /// personne n'a encore répondu.
    pub(crate) fn demander_la_telemetrie(&mut self) {
        if !self.lancement.telemetrie.a_demander() {
            return;
        }
        let oui = self.sous_un_dialogue(|ancre| {
            crate::dialogue::oui_ou_non(ancre, "Journal technique", QUESTION)
        });
        self.repondre_a_la_telemetrie(oui);
    }

    /// **Y revenir par le menu** : la question entière si rien ne part, « continuer ? » sinon.
    pub(crate) fn revoir_la_telemetrie(&mut self) {
        let question = if self.lancement.telemetrie.accorde() == Some(true) {
            ARRETER
        } else {
            QUESTION
        };
        let oui = self.sous_un_dialogue(|ancre| {
            crate::dialogue::oui_ou_non(ancre, "Journal technique", question)
        });
        self.repondre_a_la_telemetrie(oui);
    }

    /// **Voir ce qui part** : le dossier de la boîte noire, dans l'explorateur du système.
    pub(crate) fn voir_ce_qui_part(&mut self) {
        if let Some(dossier) = self.lancement.telemetrie.dossier_de_la_boite_noire() {
            crate::interactions::links::ouvrir_un_dossier(&dossier);
        }
    }

    fn repondre_a_la_telemetrie(&mut self, oui: bool) {
        if let Err(e) = self.lancement.telemetrie.repondre(oui) {
            eprintln!("[Glucose] journal technique : la reponse n'a pas pu s'ecrire ({e})");
        }
    }
}

#[cfg(test)]
mod tests;
