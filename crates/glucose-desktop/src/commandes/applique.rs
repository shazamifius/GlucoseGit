//! **APPLIQUE-1 — une commande s'applique-t-elle ?** (fiche 113) Une seule loi, que trois chemins
//! lisent : la palette (`Ctrl+Maj+P`) n'offre, le menu du clic droit ne montre, et l'exécution ne
//! fait que ce qu'elle dit. Avant elle, chacun avait sa règle — le menu la sienne, écrite à part ;
//! les gestes la leur, qui rendait `false` en silence ; la palette aucune —, et la palette
//! proposait « Couleur » sur une flèche, qui ne fait rien (fiche 111 § 4) : un bouton qui ment.
//!
//! * **Absente, pas grisée** : la loi du menu (« griser, c'est montrer une porte fermée ; retirer,
//!   c'est ne pas parler de porte du tout ») vaut pour la palette — celle de VS Code tait de même
//!   une commande dont la condition est fausse. Pour **changer un raccourci**, la palette montre
//!   tout : on règle le clavier d'une commande qui ne s'applique pas encore.
//! * **Sans joker** : chaque commande est rangée — toujours, selon le document, selon le choix ;
//!   une commande ajoutée sans son rang ne compile pas.
//! * **La fenêtre des liens se ferme d'abord** (LIEN-3) : une commande qui ne la regarde pas la
//!   ferme avant d'agir, et ce qu'on y avait choisi avec elle ; ce qui dépend du choix ne s'y
//!   applique donc pas.
//! * **Le clavier n'y perd rien** : un accord essaie ses commandes dans l'ordre, la première qui
//!   s'applique l'emporte — la loi des chiffres (une relation, sinon un signet) reste la même.

use super::Commande;
use crate::app::GlucoseApp;
use crate::interactions::depot_web::liens_choisis;
use crate::interactions::fenetre::{candidats_a_montrer, noeud_montre_choisi};
use crate::interactions::suivre::une_fleche_seule;
use crate::ui::UiState;
use glucose_core::store::Store;
use glucose_core::types::Annotation;

impl GlucoseApp {
    /// **APPLIQUE-1** : ce que la palette offre, ce que l'exécution fait. La loi du document
    /// ([`s_applique`]), et ce que Glucose seul sait : le texte qu'on écrit — son `Ctrl+Z` défait
    /// la dernière frappe —, et le focus d'une membrane.
    pub(crate) fn s_applique(&self, commande: Commande) -> bool {
        match commande {
            Commande::SortirDeLaMembrane => self.focus.membrane().is_some(),
            Commande::Annuler | Commande::Retablir if self.editing_session.is_some() => true,
            _ => s_applique(&self.store, &self.ui, commande),
        }
    }
}

/// **APPLIQUE-1, vue du document et de l'interface** : ce que le menu montre. Le focus d'une
/// membrane et le texte qu'on écrit n'y sont pas — le menu n'offre ni l'un ni l'autre ;
/// [`GlucoseApp::s_applique`] les ajoute.
pub(crate) fn s_applique(store: &Store, ui: &UiState, commande: Commande) -> bool {
    use Commande as C;
    match commande {
        C::Copier
        | C::Couper
        | C::Dupliquer
        | C::PremierPlan
        | C::ArrierePlan
        | C::Supprimer
        | C::Pousser(..)
        | C::Grouper
        | C::RetirerLesBordures
        | C::Verrouiller
        | C::StoryboardDeLaSelection
        | C::EnregistrerLImage
        | C::RemplacerParLImage
        | C::Dater
        | C::SuivreLeLien
        | C::MontrerLesLiens
        | C::AllerAuNoeud
        | C::DisposerEnColonnes
        | C::EntrerDansLaMembrane
        | C::Relation(_)
        | C::Operateur(_)
        | C::Couleur(_)
        | C::RoueDeCouleur => {
            let fermee_d_abord =
                store.fenetre_des_liens().is_some() && !commande.regarde_la_fenetre();
            !fermee_d_abord && selon_le_choix(store, ui, commande)
        }
        C::Annuler
        | C::Retablir
        | C::VuePrecedente
        | C::VueSuivante
        | C::LireLeStoryboard
        | C::ExporterLesPlanches
        | C::SortirDeLaMembrane
        | C::ModeReference
        | C::JournalTechnique
        | C::VoirCeQuiPart
        | C::NePlusRegarder => selon_le_document(store, commande),
        C::NouveauDocument
        | C::OuvrirUnDocument
        | C::Enregistrer
        | C::EnregistrerSous
        | C::Exporter
        | C::ImporterDesImages
        | C::TimeMachine
        | C::Frise
        | C::Coller
        | C::ToutChoisir
        | C::ToutCadrer
        | C::CadrerLaSelection
        | C::Naviguer
        | C::Echap
        | C::PoserLeSignet(_)
        | C::AllerAuSignet(_)
        | C::Outil(_)
        | C::Signets
        | C::NouvelOnglet
        | C::ImporterUnDocument
        | C::Chercher
        | C::ResultatSuivant
        | C::ResultatPrecedent
        | C::Palette
        | C::ChangerUnRaccourci
        | C::RetablirLesRaccourcis
        | C::OublierLesAgents => true,
    }
}

/// **Ce que le choix décide**, pour les commandes qui portent sur lui.
fn selon_le_choix(store: &Store, ui: &UiState, commande: Commande) -> bool {
    use Commande as C;
    let images = !store.selected_image_ids.is_empty();
    let noeuds = images || !store.selected_annotation_ids.is_empty();
    match commande {
        // Un dossier seul ne se copie, ne se duplique ni ne s'empile encore (DOSSIER-1).
        C::Copier | C::Couper | C::Dupliquer | C::PremierPlan | C::ArrierePlan => noeuds,
        // Une zone de gabarit ou une case choisie se retire seule.
        C::Supprimer => {
            store.compte_de_la_selection() > 0
                || ui.zones.choisie.is_some()
                || ui.storyboard.choisie.is_some()
        }
        C::Pousser(..) => store.compte_de_la_selection() > 0,
        C::Grouper => store.peut_grouper_la_selection(),
        C::RetirerLesBordures | C::Verrouiller | C::StoryboardDeLaSelection => images,
        C::EnregistrerLImage => {
            store.selected_image_ids.len() == 1 && store.selected_annotation_ids.is_empty()
        }
        C::RemplacerParLImage => !liens_choisis(store).is_empty(),
        C::Dater => noeuds,
        C::SuivreLeLien => une_fleche_seule(store),
        C::MontrerLesLiens => candidats_a_montrer(store)
            .iter()
            .any(|id| store.peut_montrer_les_liens(id)),
        C::AllerAuNoeud => noeud_montre_choisi(store).is_some(),
        C::DisposerEnColonnes | C::EntrerDansLaMembrane => {
            !store.membranes_de_la_selection().is_empty()
        }
        // Même déjà portée, une relation reste offerte : sinon le chiffre qui la pose volerait au
        // signet du même chiffre, la commande suivante de son accord.
        C::Relation(_) => !store.selected_arrows().is_empty(),
        // Un opérateur que tous les post-it choisis portent déjà est sans objet.
        C::Operateur(voulu) => store
            .selected_stickies()
            .iter()
            .any(|a| matches!(a, Annotation::Sticky { operator, .. } if *operator != voulu)),
        // Une couleur que tout ce qui est choisi porte déjà est sans objet, comme un opérateur.
        C::Couleur(rang) => {
            let voulue = match rang.map(crate::ui::couleurs::teinte) {
                Some(None) => return false,
                Some(teinte) => teinte,
                None => None,
            };
            store.couleurs_choisies().iter().any(|c| *c != voulue)
        }
        C::RoueDeCouleur => !store.couleurs_choisies().is_empty(),
        // Rangées ailleurs par [`s_applique`], qui ne les envoie jamais ici.
        _ => false,
    }
}

/// **Ce que le document décide**, pour les commandes qui ne regardent pas le choix.
fn selon_le_document(store: &Store, commande: Commande) -> bool {
    use Commande as C;
    match commande {
        C::Annuler => store.can_undo(),
        C::Retablir => store.can_redo(),
        C::VuePrecedente => store.histoire_de_la_vue.peut_reculer(),
        C::VueSuivante => store.histoire_de_la_vue.peut_avancer(),
        C::LireLeStoryboard | C::ExporterLesPlanches => !store
            .toutes_les_cases(&store.project.active_board_id)
            .is_empty(),
        // Une fenêtre sans cadre, au-dessus des autres : un téléphone n'en a pas (fiche 56).
        C::ModeReference => !cfg!(target_os = "android"),
        // Sans serveur, rien ne part, et il n'y a rien à voir partir ; le téléphone n'a pas
        // d'explorateur où ouvrir le dossier (fiche 56).
        C::JournalTechnique => crate::telemetrie::serveur().is_some(),
        C::VoirCeQuiPart => crate::telemetrie::serveur().is_some() && !cfg!(target_os = "android"),
        C::NePlusRegarder => store.domaine_regarde.is_some(),
        // Le focus d'une membrane n'est pas au document : seul [`GlucoseApp::s_applique`] le
        // voit. Les autres sont rangées ailleurs par [`s_applique`].
        _ => false,
    }
}

#[cfg(test)]
mod tests;
