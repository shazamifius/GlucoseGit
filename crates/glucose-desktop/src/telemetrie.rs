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

/// **L'adresse du serveur**, dans son compte Cloudflare (déployé le 08/10, fiche 54 § 6).
pub const ADRESSE: &str = "https://glucose-boite-noire.ferme-nilslamber.workers.dev";

/// **Le serveur du programme** — jamais celui d'une épreuve : aucune épreuve n'atteint le réseau
/// (celles qui envoient se donnent un serveur local, [`Telemetrie::habiter_avec`]).
pub fn serveur() -> Option<&'static str> {
    if cfg!(test) {
        None
    } else {
        Some(ADRESSE)
    }
}

/// Ce que la télémétrie sait de cette machine.
#[derive(Debug, Default)]
pub struct Telemetrie {
    /// Le dossier de l'application — `None` dans les épreuves, qui n'en ont pas : rien ne s'y
    /// lit, rien n'en part.
    dossier: Option<PathBuf>,
    accord: Option<Accord>,
    /// La session en cours, qui ne part qu'au lancement suivant.
    courante: Option<PathBuf>,
    /// Où tout part — sans serveur, rien ne se demande ni ne part.
    serveur: Option<String>,
}

impl Telemetrie {
    /// **Au lancement** : le dossier de l'application et la session qui commence. L'accord se
    /// relit ; un accord donné fait partir les sessions closes, sur un fil à part.
    pub fn habiter(dossier: PathBuf, courante: Option<PathBuf>) -> Self {
        Self::habiter_avec(dossier, courante, serveur().map(str::to_string))
    }

    /// [`Self::habiter`], vers ce serveur.
    pub fn habiter_avec(
        dossier: PathBuf,
        courante: Option<PathBuf>,
        serveur: Option<String>,
    ) -> Self {
        let accord = Accord::charger(&dossier);
        let t = Self {
            dossier: Some(dossier),
            accord: Some(accord),
            courante,
            serveur,
        };
        t.envoyer_si_accorde();
        t
    }

    /// Faut-il poser la question ? Une fois, s'il y a un serveur et que personne n'a répondu.
    pub fn a_demander(&self) -> bool {
        self.serveur.is_some() && self.accord.as_ref().is_some_and(|a| a.envoyer.is_none())
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
            if let Some(adresse) = &self.serveur {
                envoi::effacer_en_fond(adresse.clone(), accord.installation.clone());
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
        let (Some(adresse), Some(dossier), Some(accord)) =
            (&self.serveur, &self.dossier, &self.accord)
        else {
            return;
        };
        if accord.envoyer == Some(true) {
            envoi::envoyer_en_fond(
                adresse.clone(),
                dossier.clone(),
                accord.installation.clone(),
                self.courante.clone(),
            );
        }
    }
}

/// Le texte de la question, autour du geste qui y revient.
macro_rules! la_question {
    ($geste:expr) => {
        concat!(
            "Aider à améliorer Glucose ?\n\n",
            "À chaque lancement, Glucose peut envoyer le journal technique de la session ",
            "précédente : le temps que chaque image a pris, le système et le processeur, la façon ",
            "dont la session a fini. Jamais le contenu de tes documents, ni un nom, ni un fichier, ",
            "ni ton adresse.\n\n",
            "Tu peux changer d'avis à tout moment (",
            $geste,
            ", puis Journal technique) : ce qui est parti s'efface alors."
        )
    };
}

/// Le geste qui ouvre le menu du canevas, là où l'on est : la main n'a pas de clic droit.
#[cfg(target_os = "android")]
macro_rules! revenir {
    () => {
        "deux touchers sur le vide"
    };
}
#[cfg(not(target_os = "android"))]
macro_rules! revenir {
    () => {
        "clic droit sur le canevas"
    };
}

/// **La question**, telle qu'elle s'affiche : ce qui part, pourquoi, et ce qui n'en part jamais.
///
/// Un paragraphe par ligne : le dialogue du système et la question dessinée (QUESTION-1) coupent
/// eux-mêmes à leur largeur. Le geste qui y revient est celui de la main qu'on a : le clic
/// droit, ou deux touchers sur le vide.
pub const QUESTION: &str = la_question!(revenir!());

/// **La question, quand on y revient par le menu** et que le journal part déjà.
pub const ARRETER: &str = concat!(
    "Glucose envoie le journal technique de chaque session.\n\n",
    "Continuer à l'envoyer ? « Non » l'arrête, et efface ce qui est déjà parti."
);

/// La question du journal technique : « Oui » d'abord, la réponse qu'on lit en premier.
fn question(texte: &str) -> crate::ui::question::Question {
    use crate::ui::question::{Question, Reponse};
    Question {
        titre: "Journal technique".into(),
        texte: texte.into(),
        choix: vec![("Oui".into(), Reponse::Oui), ("Non".into(), Reponse::Non)],
        ..Default::default()
    }
}

impl crate::app::GlucoseApp {
    /// **La question, une seule fois** : à l'ouverture de la fenêtre, s'il y a un serveur et que
    /// personne n'a encore répondu.
    pub(crate) fn demander_la_telemetrie(&mut self) {
        if !self.lancement.telemetrie.a_demander() {
            return;
        }
        self.demander(question(QUESTION), crate::ui::question::Suite::Telemetrie);
    }

    /// **Y revenir par le menu** : la question entière si rien ne part, « continuer ? » sinon.
    pub(crate) fn revoir_la_telemetrie(&mut self) {
        let texte = if self.lancement.telemetrie.accorde() == Some(true) {
            ARRETER
        } else {
            QUESTION
        };
        self.demander(question(texte), crate::ui::question::Suite::Telemetrie);
    }

    /// **Voir ce qui part** : le dossier de la boîte noire, dans l'explorateur du système.
    pub(crate) fn voir_ce_qui_part(&mut self) {
        if let Some(dossier) = self.lancement.telemetrie.dossier_de_la_boite_noire() {
            crate::interactions::links::ouvrir_un_dossier(&dossier);
        }
    }

    pub(crate) fn repondre_a_la_telemetrie(&mut self, oui: bool) {
        if let Err(e) = self.lancement.telemetrie.repondre(oui) {
            eprintln!("[Glucose] journal technique : la reponse n'a pas pu s'ecrire ({e})");
        }
    }
}

#[cfg(test)]
mod tests;
