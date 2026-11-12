//! **Ce que le menu propose**, selon l'endroit où il s'ouvre : un onglet, une sélection, le
//! vide. Sorti de [`super`] quand la mise en page en colonnes (ACCES-1) l'a fait passer les
//! six cents lignes : ce que le menu **offre** et la façon dont il **se pose** ne changent
//! pas pour les mêmes raisons.
//!
//! **Ce qui est sans objet, la loi le dit** (APPLIQUE-1, fiche 113) : chaque endroit range ses
//! entrées, et une seule loi — celle que la palette et l'exécution lisent aussi — retire celles
//! qui ne feraient rien. Le menu avait ses propres conditions, écrites à part de celles des
//! gestes ; elles ne pouvaient que diverger.

use super::MenuAction;
use crate::commandes::applique::s_applique;
use crate::ui::UiState;
use glucose_core::store::Store;

/// Les entrées que l'état courant justifie, `None` entre deux groupes. Le troisième champ
/// n'est que l'indication d'un geste (« Double-clic », « déposer ici ») : le raccourci d'une
/// commande, le clavier le dit (P4, fiche 72).
///
/// Rien n'est grisé : une entrée sans objet **disparaît**. Griser, c'est montrer une porte
/// fermée ; retirer, c'est ne pas parler de porte du tout.
pub(super) type Def = Option<(MenuAction, &'static str, &'static str)>;

/// Les entrées d'un menu ouvert sur un onglet — celles du menu des feuilles d'un classeur.
fn entrees_d_onglet(store: &Store) -> Vec<Def> {
    let mut defs = vec![Some((
        MenuAction::RenommerOnglet,
        "Renommer",
        "Double-clic",
    ))];
    if store.onglets().nth(1).is_some() {
        defs.push(Some((MenuAction::SupprimerOnglet, "Supprimer", "×")));
    }
    defs.push(None);
    defs.push(Some((MenuAction::NouvelOnglet, "Nouveau board", "+")));
    defs.push(Some((
        MenuAction::AjouterUnDocument,
        "Importer un document…",
        "déposer ici",
    )));
    defs
}

pub(super) fn entrees(store: &Store, ui: &UiState, onglet: Option<&str>) -> Option<Vec<Def>> {
    let defs = if onglet.is_some() {
        entrees_d_onglet(store)
    } else if store.fenetre_des_liens().is_some() {
        entrees_de_la_fenetre()
    } else {
        let mut defs = entrees_de_l_historique();
        if store.compte_de_la_selection() > 0 {
            defs.extend(entrees_de_selection(store)?);
        } else {
            defs.extend(entrees_du_vide());
        }
        defs
    };
    Some(sans_ce_qui_est_sans_objet(defs, |c| {
        s_applique(store, ui, c)
    }))
}

/// **Retire ce que la loi dit sans objet** (APPLIQUE-1), puis les filets qui ne séparent plus
/// rien : en tête, en queue, ou deux de suite. Les gestes d'un onglet, qui portent son nom et non
/// une commande, restent.
fn sans_ce_qui_est_sans_objet(
    defs: Vec<Def>,
    s_applique: impl Fn(crate::commandes::Commande) -> bool,
) -> Vec<Def> {
    let mut out: Vec<Def> = Vec::with_capacity(defs.len());
    for def in defs {
        match def {
            Some((action, ..)) if !action.commande().is_none_or(&s_applique) => {}
            None if out.last().is_none_or(Option::is_none) => {}
            def => out.push(def),
        }
    }
    if out.last().is_some_and(Option::is_none) {
        out.pop();
    }
    out
}

/// **Dans la fenêtre des liens** (LIEN-3, fiche 102) : aller au nœud choisi, en montrer les liens,
/// fermer — elle ne change rien au document, et le menu n'offre rien qui le change.
fn entrees_de_la_fenetre() -> Vec<Def> {
    vec![
        Some((MenuAction::AllerAuNoeud, "Aller à ce nœud", "Clic")),
        Some((MenuAction::MontrerLesLiens, "Montrer ses liens", "Maj+clic")),
        None,
        Some((
            MenuAction::FermerLaFenetre,
            "Fermer la fenêtre des liens",
            "",
        )),
    ]
}

/// **Annuler et rétablir**, en tête de tout menu du canevas (ACCES-1) : au clavier, `Ctrl+Z`
/// et `Ctrl+Y` ; au doigt, deux et trois doigts touchés (GESTES-1) — rien ne se devinait, ni à
/// la souris sans clavier. Chacun n'existe que s'il y a quelque chose à défaire ou à refaire.
fn entrees_de_l_historique() -> Vec<Def> {
    vec![
        Some((MenuAction::Annuler, "Annuler", "")),
        Some((MenuAction::Retablir, "Rétablir", "")),
        None,
    ]
}

/// Les entrées d'un menu ouvert sur une sélection : ce qu'on fait de ce qu'on a choisi.
fn entrees_de_selection(store: &Store) -> Option<Vec<Def>> {
    // **Un dossier seul** (DOSSIER-1) : copier, dupliquer et l'ordre ne portent pas encore les
    // dossiers — les proposer mentirait. La suppression, elle, les prend.
    if store.selected_image_ids.is_empty() && store.selected_annotation_ids.is_empty() {
        return Some(vec![Some((MenuAction::Delete, "Supprimer", ""))]);
    }
    // « Copier » emporte tout : le lot, le texte, les images elles-mêmes — « Copier l'image »
    // n'avait plus de raison d'être un geste à part (COPIER-1, fiche 59).
    let mut defs: Vec<Def> = vec![
        Some((MenuAction::Copier, "Copier", "")),
        Some((MenuAction::Couper, "Couper", "")),
        Some((MenuAction::Duplicate, "Dupliquer", "")),
        Some((MenuAction::CadrerLaSelection, "Cadrer la sélection", "")),
        // Ce qui a des liens les montre, amenés près de lui (LIEN-3).
        Some((MenuAction::MontrerLesLiens, "Montrer ses liens", "")),
        // Une flèche seule se suit jusqu'à son autre bout : au doigt, c'est le seul chemin
        // (LIEN-2).
        Some((MenuAction::SuivreLeLien, "Suivre le lien", "")),
        None,
        Some((
            MenuAction::EnregistrerLImage,
            "Enregistrer l'image sous…",
            "",
        )),
        // Un dépôt replié en lien se rattrape là où il est (DEPOT-WEB-6).
        Some((MenuAction::RemplacerParLImage, "Remplacer par l'image", "")),
    ];
    defs.extend(entrees_des_regions());
    defs.extend(entrees_des_operateurs());
    defs.push(None);
    if !store.selected_image_ids.is_empty() {
        let board = store.active_board()?;
        let toutes_fermees = board
            .images
            .iter()
            .filter(|i| store.selected_image_ids.contains(&i.id))
            .all(|i| i.locked);
        let label = if toutes_fermees {
            "Déverrouiller"
        } else {
            "Verrouiller"
        };
        defs.push(Some((MenuAction::ToggleLock, label, "")));
    }
    defs.extend([
        Some((MenuAction::TrimBorders, "Retirer les bordures", "")),
        // Une case par image choisie (P2, proposition 2, fiche 66).
        Some((
            MenuAction::StoryboardDeLaSelection,
            "Storyboard de la sélection",
            "",
        )),
        None,
        Some((MenuAction::ToFront, "Au premier plan", "")),
        Some((MenuAction::ToBack, "À l'arrière-plan", "")),
        None,
        Some((MenuAction::Delete, "Supprimer", "")),
    ]);
    Some(defs)
}

/// **Les régions** : en faire une de ce qu'on a choisi — entourer d'un geste (GROUPER-1, fiche
/// 98) ; ranger celle qu'on a choisie (COLONNES-1, fiche 92) et y entrer, par un geste choisi
/// et plus jamais par le zoom (FOCUS-1, fiche 95).
fn entrees_des_regions() -> [Def; 4] {
    [
        None,
        Some((MenuAction::Grouper, "Grouper dans une membrane", "")),
        Some((MenuAction::DisposerEnColonnes, "Disposer en colonnes", "")),
        Some((
            MenuAction::EntrerDansLaMembrane,
            "Entrer dans la membrane",
            "",
        )),
    ]
}

/// **Les opérateurs des post-it choisis** (OPERATEUR-1, fiche 98) : au téléphone, sans `Alt`,
/// c'est le seul chemin. Un opérateur que tous les post-it choisis portent déjà est sans objet :
/// la loi le retire — « Post-it ordinaire » aussi, quand aucun n'en porte.
fn entrees_des_operateurs() -> [Def; 6] {
    use glucose_core::types::StickyOperator;
    [
        None,
        Some((
            MenuAction::Operateur(Some(StickyOperator::And)),
            "Opérateur ET",
            "",
        )),
        Some((
            MenuAction::Operateur(Some(StickyOperator::Or)),
            "Opérateur OU",
            "",
        )),
        Some((
            MenuAction::Operateur(Some(StickyOperator::But)),
            "Opérateur MAIS",
            "",
        )),
        Some((
            MenuAction::Operateur(Some(StickyOperator::Because)),
            "Opérateur PARCE QUE",
            "",
        )),
        Some((MenuAction::Operateur(None), "Post-it ordinaire", "")),
    ]
}

/// Les entrées d'un menu ouvert sur le vide : ce qu'on y apporte, et le document.
fn entrees_du_vide() -> Vec<Def> {
    vec![
        Some((MenuAction::AjouterDesImages, "Ajouter des images…", "")),
        None,
        Some((MenuAction::Paste, "Coller", "")),
        Some((MenuAction::SelectAll, "Tout sélectionner", "")),
        Some((MenuAction::ToutCadrer, "Tout cadrer", "")),
        // RETOUR-1 (fiche 99) : au doigt, il n'y a pas d'`Alt+Z` ; l'entrée n'existe que s'il y a
        // où revenir, comme « Annuler ».
        Some((MenuAction::VuePrecedente, "Vue précédente", "Alt+Z")),
        Some((MenuAction::Signets, "Signets…", "1…9")),
        // Chercher (fiche 89) : au doigt, il n'y a pas de `Ctrl+F`.
        Some((MenuAction::Chercher, "Chercher…", "Ctrl+F")),
        // La palette (P4, fiche 72) : au doigt, il n'y a pas de `Ctrl+Maj+P`.
        Some((MenuAction::Palette, "Palette de commandes…", "")),
        // L'animatique, quand le tableau a des cases à lire (fiche 67), et ses planches, PDF ou
        // images (P2.7, fiche 68).
        Some((MenuAction::LireLeStoryboard, "Lire le storyboard", "")),
        Some((MenuAction::ExporterLesPlanches, "Exporter en planches…", "")),
        None,
        Some((MenuAction::NouveauDocument, "Nouveau document", "")),
        Some((MenuAction::OuvrirUnDocument, "Ouvrir un document…", "")),
        Some((MenuAction::Enregistrer, "Enregistrer", "")),
        Some((MenuAction::EnregistrerSous, "Enregistrer sous…", "")),
        Some((MenuAction::Exporter, "Exporter…", "")),
        // Une fenêtre sans cadre, au-dessus des autres : un téléphone n'en a pas (fiche 56).
        Some((MenuAction::ModeReference, "Mode référence", "")),
        // Sans serveur, ces entrées n'ont pas d'objet ; le téléphone n'a pas d'explorateur.
        None,
        Some((MenuAction::JournalTechnique, "Journal technique…", "")),
        Some((MenuAction::VoirCeQuiPart, "Voir ce qui part", "")),
    ]
}
