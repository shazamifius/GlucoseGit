//! **IDENTITE-2 — son nom et sa couleur, demandés une fois** (Q19, fiche 140).
//!
//! Sa réponse du 23/11 : *« carrément, je suis plutôt d'accord : on demande le pseudo ainsi que la
//! couleur d'affichage »*. Jusque-là, un nom tiré au sort (« Aigle Émeraude ») qu'on ne changeait
//! que dans un fichier. Désormais, **la première fois** qu'il partage ou rejoint un canevas, Glucose
//! les lui demande — un champ, et les dix pastilles —, puis fait ce qu'il demandait ; ensuite,
//! « Ton nom et ta couleur… » dans Collaborer, à tout moment. Dans un canevas partagé, le changement
//! part avec la lettre suivante : chaque lettre porte son nom et sa couleur, et chez les autres
//! chacune les relit.
//!
//! **La couleur reste l'une des dix de la palette** (nous avions écrit « sur la roue ») : elles
//! sont choisies pour rester lisibles en pastille de quelques pixels sur le fond sombre, et la
//! lettre porte son **rang** — un Glucose déjà installé (la beta.5) comprend un rang de la palette,
//! pas une couleur libre. **Deux personnes de la même couleur** : la question le dit, elle ne
//! choisit pas à sa place.

use super::identite::{nom_propre, Identite, COULEURS};
use crate::app::GlucoseApp;
use crate::ui::question::pastilles::Pastilles;
use crate::ui::question::{Champ, Question, Reponse, Suite};
use glucose_core::text::Selection;

/// **Ce qu'il demandait** quand Glucose lui a d'abord demandé son nom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Puis {
    Partager,
    Rejoindre,
}

/// Ce que la question explique, toujours.
const POURQUOI: &str = "Les autres te voient sous ce nom, de cette couleur : ta pastille, ton \
                        curseur, ton nom chez eux. Tu les changes à tout moment dans Collaborer.";

impl GlucoseApp {
    /// **La première fois**, son nom et sa couleur avant `puis` : rend `true` si la question est
    /// posée — ce qu'il demandait viendra après sa réponse.
    pub(super) fn d_abord_son_nom(&mut self, puis: Puis) -> bool {
        if Identite::la_sienne(&self.disque.identite).choisie {
            return false;
        }
        self.demander_son_nom(Some(puis), None);
        true
    }

    /// **« Ton nom et ta couleur »** : le champ porte son nom, tout choisi ; les pastilles, sa
    /// couleur. `note`, ce qui n'a pas tenu la fois d'avant (un nom vide).
    pub(crate) fn demander_son_nom(&mut self, puis: Option<Puis>, note: Option<(&str, usize)>) {
        let moi = Identite::la_sienne(&self.disque.identite);
        let couleur = note.map_or(moi.couleur, |(_, c)| c);
        let mut texte = self.texte_du_nom(couleur);
        if let Some((n, _)) = note {
            texte = format!("{n}\n\n{texte}");
        }
        let question = Question {
            titre: "Ton nom et ta couleur".into(),
            texte,
            choix: vec![
                ("Garder".into(), Reponse::Oui),
                ("Annuler".into(), Reponse::Annuler),
            ],
            champ: Some(Champ {
                selection: Selection::all(&moi.nom),
                texte: moi.nom,
            }),
            pastilles: Some(Pastilles {
                couleurs: COULEURS.to_vec(),
                choisie: couleur,
            }),
            ..Default::default()
        };
        self.demander(question, Suite::Identite(puis));
    }

    /// Le texte de la question, et qui a déjà cette couleur parmi ceux qui sont là.
    fn texte_du_nom(&self, couleur: usize) -> String {
        match self.qui_a_cette_couleur(couleur) {
            Some(nom) => format!("{POURQUOI}\n\n« {nom} » a déjà cette couleur."),
            None => POURQUOI.to_string(),
        }
    }

    /// Le nom de quelqu'un, présent dans ce canevas, qui porte déjà cette couleur.
    fn qui_a_cette_couleur(&self, couleur: usize) -> Option<String> {
        let cp = self.lancement.agents.partage.as_ref()?;
        cp.presents()
            .into_iter()
            .find(|p| usize::from(p.couleur) % COULEURS.len() == couleur)
            .map(|p| p.nom.clone())
    }

    /// **Une pastille vient d'être choisie** : la question dit qui l'a déjà, s'il y en a un.
    pub(crate) fn la_couleur_a_change(&mut self) {
        let Some((question, Suite::Identite(_))) = &self.ui.question else {
            return;
        };
        let Some(couleur) = question.pastilles.as_ref().map(|p| p.choisie) else {
            return;
        };
        let texte = self.texte_du_nom(couleur);
        if let Some((question, _)) = &mut self.ui.question {
            question.texte = texte;
        }
    }

    /// **Sa réponse** : gardée, choisie — et dans un canevas partagé, ses lettres suivantes la
    /// portent ; puis ce qu'il demandait. Un nom vide se redemande, la couleur choisie gardée ;
    /// « Annuler » revient à Collaborer, sans rien partager ni rejoindre.
    pub(crate) fn repondre_a_l_identite(
        &mut self,
        reponse: Reponse,
        (champ, pastille): (Option<String>, Option<usize>),
        puis: Option<Puis>,
    ) {
        if reponse != Reponse::Oui {
            return self.montrer_collaborer_avec(None);
        }
        let avant = Identite::la_sienne(&self.disque.identite);
        let couleur = pastille
            .filter(|c| *c < COULEURS.len())
            .unwrap_or(avant.couleur);
        let Some(nom) = champ.as_deref().and_then(nom_propre) else {
            let note = "Un nom vide ne te nomme pas : écris-en un.";
            return self.demander_son_nom(puis, Some((note, couleur)));
        };
        let moi = Identite {
            nom,
            couleur,
            choisie: true,
        };
        moi.garder(&self.disque.identite);
        if let Some(cp) = self.lancement.agents.partage.as_mut() {
            cp.nom.clone_from(&moi.nom);
            cp.couleur = moi.couleur as u8;
            cp.photos
                .signer((cp.marque.clone(), moi.nom.clone(), cp.couleur));
        }
        match puis {
            Some(Puis::Partager) => self.partager_ce_canevas(),
            Some(Puis::Rejoindre) => self.demander_le_code(),
            None => self.montrer_collaborer_avec(None),
        }
    }
}

#[cfg(test)]
mod tests;
