//! **REJOINDRE-1 — rejoindre un canevas par le code qu'on écrit** (sa note du 23/11, `text-kpll-346`,
//! sa capture du téléphone : *« ce serait cool d'avoir juste un emplacement texte pour pouvoir
//! écrire à la main »*).
//!
//! « Rejoindre » ne lisait que le presse-papiers. Sous Android, Glucose n'a que le sien
//! ([`crate::interactions::presse_papiers`], fiche 54) : un code copié d'un message n'y arrivait
//! jamais, et Glucose répondait « aucun code » — le co-working du téléphone était fermé. La
//! question porte désormais un champ, que le clavier du téléphone remplit (sa suggestion du
//! presse-papiers passe par la saisie, pas par Glucose) ; **déjà rempli du code copié** quand le
//! presse-papiers en porte un : au bureau, rien n'est perdu.

use crate::app::GlucoseApp;
use crate::persist::close::Ensuite;
use crate::ui::question::{Champ, Question, Reponse, Suite};
use glucose_core::salle::code::Code;
use glucose_core::text::Selection;

/// Ce que la question dit quand le presse-papiers portait un code.
pub(crate) const DEJA_COPIE: &str = "Le code copié est déjà là.";
/// Ce qu'elle dit d'un texte qui n'est pas un code — gardé dans le champ, pour le corriger.
pub(crate) const PAS_UN_CODE: &str = "Ce n'est pas un code de Glucose.";

impl GlucoseApp {
    /// **La question du code** : un champ, déjà rempli du code copié s'il y en a un — et seulement
    /// d'un code : ce que le presse-papiers porte d'autre ne s'affiche pas.
    pub(super) fn demander_le_code(&mut self) {
        let copie = crate::interactions::presse_papiers::ouvrir()
            .and_then(|mut a| a.texte())
            .ok()
            .filter(|t| Code::lire_le_texte(t).is_some())
            .map(|t| t.trim().to_string());
        let entete = copie.as_ref().map(|_| DEJA_COPIE);
        self.poser_la_question_du_code(copie.unwrap_or_default(), entete);
    }

    /// La question, son champ portant `texte`, tout choisi : ce qu'on tape le remplace.
    fn poser_la_question_du_code(&mut self, texte: String, entete: Option<&str>) {
        let explication = "Colle ici le code qu'on t'a donné, ou tape-le. Il commence par \
                           glucose-ecrire- (pour y travailler) ou glucose-lire- (pour regarder).";
        let question = Question {
            titre: "Rejoindre un canevas".into(),
            texte: entete.map_or_else(
                || explication.to_string(),
                |e| format!("{e}\n\n{explication}"),
            ),
            choix: vec![
                ("Rejoindre".into(), Reponse::Oui),
                ("Annuler".into(), Reponse::Annuler),
            ],
            champ: Some(Champ {
                selection: Selection::all(&texte),
                texte,
            }),
            ..Default::default()
        };
        self.demander(question, Suite::Rejoindre);
    }

    /// **La réponse** : un code lisible rejoint sa salle — le document qu'on quitte passe par la
    /// porte habituelle (BROUILLON-1) ; un texte qui n'en est pas un se redemande, tel qu'on l'a
    /// écrit ; « Annuler » revient à Collaborer.
    pub(crate) fn rejoindre_par_le_code(&mut self, reponse: Reponse, texte: Option<String>) {
        if reponse != Reponse::Oui {
            return self.montrer_collaborer_avec(None);
        }
        let texte = texte.unwrap_or_default();
        match Code::lire_le_texte(&texte) {
            Some(code) => self.laisser_puis(Ensuite::Rejoindre(code)),
            None => self.poser_la_question_du_code(texte, Some(PAS_UN_CODE)),
        }
    }
}

#[cfg(test)]
mod tests;
