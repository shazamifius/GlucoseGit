//! **Ce que seul le vrai lancement donne à l'application** : de quoi réveiller la boucle depuis
//! un autre fil, et la veille des mises à jour. `main.rs` les pose ; une application d'épreuve
//! n'en a aucun — elle ne cherche jamais de mise à jour et ne réveille personne.
//!
//! # La mise à jour, vue de l'application (fiche 48)
//!
//! Au lancement, un fil lit le fichier des versions ; s'il propose plus récent, Glucose le
//! demande par la question qu'il dessine (POPUP-1) — le « popup habituel » de Glucose Tauri.
//! Oui : un autre fil télécharge l'installeur et vérifie sa signature. Prêt : le document
//! s'écrit une dernière fois — la question de la fermeture s'il porte du travail sans nom —,
//! l'installeur démarre, et Glucose se ferme pour qu'il prenne sa place. Rien de cela n'attend
//! sur la boucle.
//!
//! **Après une session qui a mal fini**, la recherche se fait **avant tout** — avant la carte
//! graphique, avant le document ([`mettre_a_jour_avant_tout`]) : une version qui tombe au
//! démarrage doit pouvoir recevoir sa correction. C'est la façon la plus courante de perdre un
//! utilisateur pour toujours (fiche 44 § 4). Un lancement ordinaire, lui, n'attend personne.

use crate::app::GlucoseApp;
use crate::mise_a_jour::cycle::{self, Nouvelle, Veille};
use crate::mise_a_jour::installation::Installation;
use crate::mise_a_jour::Proposition;
use crate::persist::close::Apres;
use crate::ui::question::Reponse;
use std::path::Path;

/// Ce que `main` donne à l'application, et rien d'autre.
#[derive(Default)]
pub struct Lancement {
    /// De quoi réveiller la boucle depuis un autre fil : le veilleur du budget de la carte
    /// (ETAGES-2). `main` seul tient la boucle avant qu'elle tourne.
    pub reveil: Option<winit::event_loop::EventLoopProxy<()>>,
    /// La veille des mises à jour.
    pub mise_a_jour: Option<Veille>,
    /// La boîte noire qui voyage, et l'accord de cette machine (fiche 54).
    pub telemetrie: crate::telemetrie::Telemetrie,
    /// Le clavier du système, miroir de la saisie en cours (CLAVIER-1) — sous Android.
    pub clavier: crate::interactions::text_edit::miroir::Miroir,
}

impl GlucoseApp {
    /// **Ce que la veille a dit depuis la dernière fois**, traité ici — là où le document
    /// n'est lu par personne.
    pub(crate) fn suivre_la_mise_a_jour(&mut self) {
        let nouvelles = self
            .lancement
            .mise_a_jour
            .as_ref()
            .map(Veille::recolter)
            .unwrap_or_default();
        for nouvelle in nouvelles {
            match nouvelle {
                Nouvelle::Proposee(p) => self.proposer_la_mise_a_jour(p),
                Nouvelle::Prete(installeur) => self.installer(&installeur),
                Nouvelle::Echec(e) => self.dire_la_mise_a_jour(format!("impossible : {e}")),
            }
        }
    }

    /// Une version plus récente existe : on la propose. « Plus tard » : elle sera reproposée
    /// au prochain lancement.
    fn proposer_la_mise_a_jour(&mut self, p: Proposition) {
        let question = crate::ui::question::Question {
            titre: TITRE.into(),
            texte: question(&p),
            choix: vec![
                ("Installer".into(), Reponse::Oui),
                ("Plus tard".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, crate::ui::question::Suite::MiseAJour(p));
    }

    /// Oui : un autre fil la télécharge et vérifie sa signature.
    pub(crate) fn preparer_la_mise_a_jour(&mut self, p: Proposition) {
        let version = p.version.to_string();
        if let Some(veille) = &self.lancement.mise_a_jour {
            veille.preparer(p);
        }
        self.dire_la_mise_a_jour(format!("téléchargement de Glucose {version}…"));
    }

    /// L'installeur est vérifié et posé : le document se ferme — la question du travail non
    /// enregistré, s'il en porte —, puis ce qui relance démarre.
    fn installer(&mut self, installeur: &Path) {
        if self.lancement.mise_a_jour.is_some() {
            self.fermer_puis(Apres::Installer(installeur.to_path_buf()));
        }
    }

    /// Le document est fermé : la relance démarre — l'installeur, ou Glucose déjà remplacé —,
    /// et Glucose se ferme quand même si elle échoue : un document fermé ne s'écrirait plus,
    /// et le travail, lui, est sur le disque.
    pub(crate) fn relancer_pour_installer(&mut self, installeur: &Path) {
        let relance = self
            .lancement
            .mise_a_jour
            .as_ref()
            .map(|v| v.installation().relance(installeur));
        if let Some((programme, arguments)) = relance {
            if let Err(e) = cycle::lancer(&programme, arguments) {
                eprintln!("[Glucose] mise à jour : {e}");
            }
        }
        self.ui.fermer_la_fenetre = true;
    }

    /// Le document n'a pas été fermé — « Annuler », ou un enregistrement raté : l'installeur
    /// attend le prochain lancement.
    pub(crate) fn reporter_la_mise_a_jour(&mut self) {
        self.dire_la_mise_a_jour("reportée : elle sera reproposée au prochain lancement".into());
    }

    /// **La seule voix de la mise à jour** : un toast, pour tout ce qu'elle a à dire.
    fn dire_la_mise_a_jour(&mut self, message: String) {
        self.ui.show_toast(format!("Mise à jour : {message}"));
    }
}

/// Ce qu'on demande : la version, ce qu'elle apporte, et ce qui va se passer.
fn question(p: &Proposition) -> String {
    let notes = if p.notes.trim().is_empty() {
        String::new()
    } else {
        format!("\n\n{}", p.notes.trim())
    };
    format!(
        "Glucose {} est disponible.{notes}\n\nL'installer maintenant ? Ton travail s'enregistre, \
         Glucose se ferme, s'installe, puis se rouvre.",
        p.version
    )
}

/// Le titre de la question.
const TITRE: &str = "Mise à jour de Glucose";

impl GlucoseApp {
    /// **La recherche avant tout**, après une session qui a mal fini : avant la carte
    /// graphique et avant le document. Rend `true` si la relance a démarré : Glucose doit
    /// alors se fermer sans rien ouvrir. Tout échec laisse le lancement continuer : une mise à
    /// jour qui ne se fait pas n'empêche jamais d'ouvrir Glucose.
    ///
    /// # La seule boîte du système qui reste (POPUP-1, DIAL-5)
    ///
    /// La fenêtre n'existe pas encore, et c'est voulu : ce qui tombe au démarrage — la carte
    /// graphique, le document — ne doit pas empêcher de recevoir la correction. Glucose n'a
    /// donc rien où dessiner la question, et la pose par le système. Aucune fenêtre de Glucose
    /// n'existe à noircir ni à geler : le défaut de POPUP-1 ne peut pas s'y produire.
    pub fn mettre_a_jour_avant_tout(
        &mut self,
        installeurs: &Path,
        installation: &Installation,
    ) -> bool {
        let courante = crate::mise_a_jour::version::Version::courante();
        let Ok(Some(p)) = cycle::chercher(cycle::ADRESSE, &courante, installation) else {
            return false;
        };
        let question = format!(
            "La session précédente s'est arrêtée brutalement.\n\n{}",
            question(&p)
        );
        let Some(sans_fenetre) = self.sans_fenetre() else {
            return false;
        };
        if !crate::dialogue::oui_ou_non(sans_fenetre, TITRE, &question) {
            return false;
        }
        let relance = cycle::preparer(&p, cycle::CLE, installeurs, installation)
            .and_then(|f| installation.poser(&f).map(|()| installation.relance(&f)))
            .and_then(|(programme, arguments)| cycle::lancer(&programme, arguments));
        if let Err(e) = &relance {
            eprintln!("[Glucose] mise à jour avant tout : {e}");
        }
        relance.is_ok()
    }
}

#[cfg(test)]
mod tests;
