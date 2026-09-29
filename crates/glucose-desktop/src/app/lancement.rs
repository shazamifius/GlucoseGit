//! **Ce que seul le vrai lancement donne à l'application** : de quoi réveiller la boucle depuis
//! un autre fil, et la veille des mises à jour. `main.rs` les pose ; une application d'épreuve
//! n'en a aucun — elle ne cherche jamais de mise à jour et ne réveille personne.
//!
//! # La mise à jour, vue de l'application (fiche 48)
//!
//! Au lancement, un fil lit le fichier des versions ; s'il propose plus récent, la boucle le
//! demande par un dialogue — le « popup habituel » de Glucose Tauri. Oui : un autre fil
//! télécharge l'installeur et vérifie sa signature. Prêt : le document s'écrit une dernière
//! fois — la question de la fermeture s'il porte du travail sans nom —, l'installeur démarre,
//! et Glucose se ferme pour qu'il prenne sa place. Rien de cela n'attend sur la boucle.
//!
//! **Après une session qui a mal fini**, la recherche se fait **avant tout** — avant la carte
//! graphique, avant le document ([`mettre_a_jour_avant_tout`]) : une version qui tombe au
//! démarrage doit pouvoir recevoir sa correction. C'est la façon la plus courante de perdre un
//! utilisateur pour toujours (fiche 44 § 4). Un lancement ordinaire, lui, n'attend personne.

use crate::app::GlucoseApp;
use crate::mise_a_jour::cycle::{self, Nouvelle, Veille};
use crate::mise_a_jour::Proposition;
use std::path::Path;
use winit::event_loop::ActiveEventLoop;

/// Ce que `main` donne à l'application, et rien d'autre.
#[derive(Default)]
pub struct Lancement {
    /// De quoi réveiller la boucle depuis un autre fil : le veilleur du budget de la carte
    /// (ETAGES-2). `main` seul tient la boucle avant qu'elle tourne.
    pub reveil: Option<winit::event_loop::EventLoopProxy<()>>,
    /// La veille des mises à jour.
    pub mise_a_jour: Option<Veille>,
}

/// Les arguments d'un installeur lancé par une mise à jour — ceux que Glucose Tauri passe :
/// sans question, relancer Glucose ensuite, c'est une mise à jour.
pub const ARGUMENTS_DE_L_INSTALLEUR: [&str; 3] = ["/P", "/R", "/UPDATE"];

impl GlucoseApp {
    /// **Ce que la veille a dit depuis la dernière fois**, traité ici — là où le document
    /// n'est lu par personne.
    pub(crate) fn suivre_la_mise_a_jour(&mut self, event_loop: &ActiveEventLoop) {
        let nouvelles = self
            .lancement
            .mise_a_jour
            .as_ref()
            .map(Veille::recolter)
            .unwrap_or_default();
        for nouvelle in nouvelles {
            match nouvelle {
                Nouvelle::Proposee(p) => self.proposer_la_mise_a_jour(p),
                Nouvelle::Prete(installeur) => self.installer(&installeur, event_loop),
                Nouvelle::Echec(e) => self.dire_la_mise_a_jour(format!("impossible : {e}")),
            }
        }
    }

    /// Une version plus récente existe : on la propose. Non : elle sera reproposée au prochain
    /// lancement.
    fn proposer_la_mise_a_jour(&mut self, p: Proposition) {
        let question = question(&p);
        let oui = self.sous_un_dialogue(|fenetre| demander(fenetre, &question));
        if !oui {
            return;
        }
        let version = p.version.to_string();
        if let Some(veille) = &self.lancement.mise_a_jour {
            veille.preparer(p);
        }
        self.dire_la_mise_a_jour(format!("téléchargement de Glucose {version}…"));
    }

    /// L'installeur est vérifié et posé : le document s'écrit une dernière fois, puis
    /// l'installeur démarre, et Glucose se ferme pour qu'il prenne sa place.
    fn installer(&mut self, installeur: &Path, event_loop: &ActiveEventLoop) {
        if !self.request_close() {
            self.dire_la_mise_a_jour(
                "reportée : elle sera reproposée au prochain lancement".into(),
            );
            return;
        }
        // Le document est fermé : l'installeur démarre, ou Glucose se ferme quand même — un
        // document fermé ne s'écrirait plus, et le travail, lui, est sur le disque.
        if let Err(e) = cycle::lancer(installeur, &ARGUMENTS_DE_L_INSTALLEUR) {
            eprintln!("[Glucose] mise à jour : {e}");
        }
        event_loop.exit();
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

/// Le dialogue natif, accroché à la fenêtre s'il y en a une : oui ou non.
fn demander(ancre: crate::dialogue::Ancre<'_>, question: &str) -> bool {
    crate::dialogue::message(ancre)
        .set_level(rfd::MessageLevel::Info)
        .set_title("Mise à jour de Glucose")
        .set_description(question)
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
        == rfd::MessageDialogResult::Yes
}

impl GlucoseApp {
    /// **La recherche avant tout**, après une session qui a mal fini : avant la carte
    /// graphique et avant le document — la fenêtre n'existe pas encore, et le dialogue s'ouvre
    /// sans parent, ce que DIAL-1 prévoit. Rend `true` si un installeur a démarré : Glucose doit
    /// alors se fermer sans rien ouvrir. Tout échec laisse le lancement continuer : une mise à
    /// jour qui ne se fait pas n'empêche jamais d'ouvrir Glucose.
    pub fn mettre_a_jour_avant_tout(&mut self, installeurs: &Path) -> bool {
        let courante = crate::mise_a_jour::version::Version::courante();
        let Ok(Some(p)) = cycle::chercher(cycle::ADRESSE, &courante) else {
            return false;
        };
        let question = format!(
            "La session précédente s'est arrêtée brutalement.\n\n{}",
            question(&p)
        );
        if !self.sous_un_dialogue(|fenetre| demander(fenetre, &question)) {
            return false;
        }
        match cycle::preparer(&p, cycle::CLE, installeurs) {
            Ok(installeur) => cycle::lancer(&installeur, &ARGUMENTS_DE_L_INSTALLEUR).is_ok(),
            Err(e) => {
                eprintln!("[Glucose] mise à jour avant tout : {e}");
                false
            }
        }
    }
}
