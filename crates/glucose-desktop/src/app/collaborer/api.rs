//! **« L'API publique »** (fiche 128) : Collaborer ouvre les clés des IA gratuites, les teste,
//! et dit ce qui répond. La question se pose ici ; le réseau tourne sur un fil à part, et la
//! boucle reprend son résultat à chaque tour — jamais elle n'attend un fournisseur.

use crate::app::GlucoseApp;
use crate::ia::cles::{self, Cles};
use crate::ia::modeles::Fournisseur;
use crate::ui::question::{Champ, Question, Reponse, Suite};
use std::sync::mpsc::{channel, Receiver};

/// Le bouton « L'API publique… » de Collaborer, puis les réponses de son menu.
pub(crate) const MENU: Reponse = Reponse::Choix(14);
pub(crate) const TESTER: Reponse = Reponse::Choix(10);
pub(crate) const CLE_GOOGLE: Reponse = Reponse::Choix(11);
pub(crate) const CLE_GITHUB: Reponse = Reponse::Choix(12);
const RETOUR: Reponse = Reponse::Choix(13);
/// « Enregistrer » dans la question où l'on écrit une clé : la réponse Oui.
const ENREGISTRER: Reponse = Reponse::Oui;

/// Ce qu'un essai rend au fil de la boucle : le bilan des clés, ou la raison de n'en rien dire.
pub type Essai = Receiver<Result<String, String>>;

impl GlucoseApp {
    /// **« L'API publique… »** : les clés branchées, et les trois gestes.
    pub(crate) fn montrer_l_api_publique(&mut self, note: Option<String>) {
        let cles = cles::lire_depuis(&cles::chemin());
        let etat = |cle: &Option<String>| {
            if cle.is_some() {
                "branchée"
            } else {
                "pas branchée"
            }
        };
        let mut texte = format!(
            "Des IA gratuites pour Glucose. Google AI Studio : {}. GitHub : {}.\n\n\
             « Tester » cherche les modèles que tes clés ouvrent sans payer, du plus capable au \
             moins capable, et essaie les meilleurs un par un.",
            etat(&cles.google),
            etat(&cles.github),
        );
        if let Some(n) = note {
            texte = format!("{n}\n\n{texte}");
        }
        let question = Question {
            titre: "L'API publique".into(),
            texte,
            choix: vec![
                ("Tester les modèles gratuits".into(), TESTER),
                ("Donner la clé Google AI Studio".into(), CLE_GOOGLE),
                ("Donner la clé GitHub".into(), CLE_GITHUB),
                ("Fermer".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::ApiPublique);
    }

    /// Sa réponse au menu de l'API publique.
    pub(crate) fn repondre_a_l_api_publique(&mut self, reponse: Reponse) {
        match reponse {
            TESTER => self.lancer_l_essai_ia(),
            CLE_GOOGLE => self.demander_une_cle(Fournisseur::Google),
            CLE_GITHUB => self.demander_une_cle(Fournisseur::GitHub),
            RETOUR => self.montrer_l_api_publique(None),
            _ => {}
        }
    }

    /// La question où l'on écrit la clé d'un fournisseur ; « Enregistrer » la garde sur ce Glucose.
    fn demander_une_cle(&mut self, fournisseur: Fournisseur) {
        let nom = match fournisseur {
            Fournisseur::Google => "Google AI Studio",
            Fournisseur::GitHub => "GitHub",
        };
        let question = Question {
            titre: format!("Clé {nom}"),
            texte: format!(
                "Colle ta clé {nom} ici. Elle reste sur ce Glucose, à tes seuls droits de \
                 lecture, et ne part que vers {nom}."
            ),
            choix: vec![
                ("Enregistrer".into(), ENREGISTRER),
                ("Annuler".into(), Reponse::Annuler),
            ],
            champ: Some(Champ::default()),
            ..Default::default()
        };
        self.demander(question, Suite::ClefIa(fournisseur));
    }

    /// Ce que la réponse à la question de clé décide : on garde la clé écrite, s'il y en a une.
    pub(crate) fn enregistrer_la_cle(
        &mut self,
        fournisseur: Fournisseur,
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
        let mut tout = cles::lire_depuis(&cles::chemin());
        match fournisseur {
            Fournisseur::Google => tout.google = cle,
            Fournisseur::GitHub => tout.github = cle,
        }
        let note = match cles::ecrire_dans(&cles::chemin(), &tout) {
            Ok(()) => "Clé enregistrée sur ce Glucose.".to_string(),
            Err(e) => format!("La clé n'a pas pu s'enregistrer ({e})."),
        };
        self.montrer_l_api_publique(Some(note));
    }

    /// Lance l'essai sur un fil à part : la boucle continue, et reprend le bilan au tour suivant.
    fn lancer_l_essai_ia(&mut self) {
        let tout = cles::lire_depuis(&cles::chemin());
        let (tx, rx) = channel();
        self.lancement.agents.essai_ia = Some(rx);
        std::thread::spawn(move || {
            let _ = tx.send(essayer(&tout));
        });
    }

    /// Ce que l'essai a rendu, s'il est revenu : la question des modèles qui répondent.
    pub(crate) fn suivre_l_essai_ia(&mut self) {
        let Some(rx) = self.lancement.agents.essai_ia.as_ref() else {
            return;
        };
        let Ok(resultat) = rx.try_recv() else {
            return;
        };
        self.lancement.agents.essai_ia = None;
        let texte = match resultat {
            Ok(bilan) => bilan,
            Err(e) => format!("L'essai n'a pas abouti : {e}."),
        };
        let question = Question {
            titre: "Modèles gratuits".into(),
            texte,
            choix: vec![
                ("Retour".into(), RETOUR),
                ("Fermer".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::ApiPublique);
    }
}

/// Teste les clés branchées, et rend le texte du bilan. Hors de Windows seulement pour l'instant :
/// Windows n'a pas encore sa pile de réseau pour cet essai (WinHTTP, à brancher).
#[cfg(not(windows))]
fn essayer(cles: &Cles) -> Result<String, String> {
    use crate::ia::essai;
    let Some(cle) = cles.google.as_deref() else {
        return Err("aucune clé Google AI Studio n'est donnée".into());
    };
    let bilan = essai::essayer_google(cle)?;
    let mut texte = format!(
        "Google AI Studio ouvre {} modèles de texte.\n",
        bilan.modeles.len()
    );
    for m in bilan.modeles.iter().take(12) {
        let repond = if bilan.qui_repond.contains(&m.id) {
            "répond"
        } else {
            "—"
        };
        texte.push_str(&format!("\n{} : {repond}", m.id));
    }
    if bilan.qui_repond.is_empty() {
        texte.push_str("\nAucun ne répond pour l'instant : la limite est peut-être atteinte.");
    }
    Ok(texte)
}

#[cfg(windows)]
fn essayer(_cles: &Cles) -> Result<String, String> {
    Err("l'essai n'est pas encore branché sous Windows".into())
}
