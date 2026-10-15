//! **Ce que le menu propose**, selon l'endroit où il s'ouvre : un onglet, une sélection, le
//! vide. Sorti de [`super`] quand la mise en page en colonnes (ACCES-1) l'a fait passer les
//! six cents lignes : ce que le menu **offre** et la façon dont il **se pose** ne changent
//! pas pour les mêmes raisons.

use super::MenuAction;
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

pub(super) fn entrees(store: &Store, onglet: Option<&str>) -> Option<Vec<Def>> {
    if onglet.is_some() {
        return Some(entrees_d_onglet(store));
    }
    let mut defs = entrees_de_l_historique(store);
    if store.compte_de_la_selection() > 0 {
        defs.extend(entrees_de_selection(store)?);
    } else {
        defs.extend(entrees_du_vide(store));
    }
    Some(defs)
}

/// **Annuler et rétablir**, en tête de tout menu du canevas (ACCES-1) : au clavier, `Ctrl+Z`
/// et `Ctrl+Y` ; au doigt, deux et trois doigts touchés (GESTES-1) — rien ne se devinait, ni à
/// la souris sans clavier. Chacun n'existe que s'il y a quelque chose à défaire ou à refaire.
fn entrees_de_l_historique(store: &Store) -> Vec<Def> {
    let mut defs = Vec::new();
    if store.can_undo() {
        defs.push(Some((MenuAction::Annuler, "Annuler", "")));
    }
    if store.can_redo() {
        defs.push(Some((MenuAction::Retablir, "Rétablir", "")));
    }
    if !defs.is_empty() {
        defs.push(None);
    }
    defs
}

/// Les entrées d'un menu ouvert sur une sélection : ce qu'on fait de ce qu'on a choisi.
fn entrees_de_selection(store: &Store) -> Option<Vec<Def>> {
    // **Un dossier seul** (DOSSIER-1) : copier, dupliquer et l'ordre ne portent pas encore les
    // dossiers — les proposer mentirait. La suppression, elle, les prend.
    if store.selected_image_ids.is_empty() && store.selected_annotation_ids.is_empty() {
        return Some(vec![Some((MenuAction::Delete, "Supprimer", ""))]);
    }
    let images = store.selected_image_ids.len();
    // « Copier » emporte tout : le lot, le texte, les images elles-mêmes — « Copier l'image »
    // n'avait plus de raison d'être un geste à part (COPIER-1, fiche 59).
    let mut defs: Vec<Def> = vec![
        Some((MenuAction::Copier, "Copier", "")),
        Some((MenuAction::Couper, "Couper", "")),
        Some((MenuAction::Duplicate, "Dupliquer", "")),
        Some((MenuAction::CadrerLaSelection, "Cadrer la sélection", "")),
    ];
    if images == 1 && store.selected_annotation_ids.is_empty() {
        defs.push(None);
        defs.push(Some((
            MenuAction::EnregistrerLImage,
            "Enregistrer l'image sous…",
            "",
        )));
    }
    // Un dépôt replié en lien se rattrape là où il est (DEPOT-WEB-6).
    if !crate::interactions::depot_web::liens_choisis(store).is_empty() {
        defs.push(None);
        defs.push(Some((
            MenuAction::RemplacerParLImage,
            "Remplacer par l'image",
            "",
        )));
    }
    if images > 0 {
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
        defs.push(Some((MenuAction::TrimBorders, "Retirer les bordures", "")));
        // Une case par image choisie (P2, proposition 2, fiche 66).
        defs.push(Some((
            MenuAction::StoryboardDeLaSelection,
            "Storyboard de la sélection",
            "",
        )));
    }
    defs.push(None);
    defs.push(Some((MenuAction::ToFront, "Au premier plan", "")));
    defs.push(Some((MenuAction::ToBack, "À l'arrière-plan", "")));
    defs.push(None);
    defs.push(Some((MenuAction::Delete, "Supprimer", "")));
    Some(defs)
}

/// Les entrées d'un menu ouvert sur le vide : ce qu'on y apporte, et le document.
fn entrees_du_vide(store: &Store) -> Vec<Def> {
    let mut defs: Vec<Def> = vec![
        Some((MenuAction::AjouterDesImages, "Ajouter des images…", "")),
        None,
        Some((MenuAction::Paste, "Coller", "")),
        Some((MenuAction::SelectAll, "Tout sélectionner", "")),
        Some((MenuAction::ToutCadrer, "Tout cadrer", "")),
        Some((MenuAction::Signets, "Signets…", "1…9")),
        // La palette (P4, fiche 72) : au doigt, il n'y a pas de `Ctrl+Maj+P`.
        Some((MenuAction::Palette, "Palette de commandes…", "")),
    ];
    // L'animatique, quand le tableau a des cases à lire (fiche 67).
    if !store
        .toutes_les_cases(&store.project.active_board_id)
        .is_empty()
    {
        defs.push(Some((
            MenuAction::LireLeStoryboard,
            "Lire le storyboard",
            "",
        )));
        // Ses planches, PDF ou images (P2.7, fiche 68).
        defs.push(Some((
            MenuAction::ExporterLesPlanches,
            "Exporter en planches…",
            "",
        )));
    }
    defs.extend([
        None,
        Some((MenuAction::NouveauDocument, "Nouveau document", "")),
        Some((MenuAction::OuvrirUnDocument, "Ouvrir un document…", "")),
        Some((MenuAction::Enregistrer, "Enregistrer", "")),
        Some((MenuAction::EnregistrerSous, "Enregistrer sous…", "")),
        Some((MenuAction::Exporter, "Exporter…", "")),
    ]);
    // Une fenêtre sans cadre, au-dessus des autres : un téléphone n'en a pas (fiche 56).
    if !cfg!(target_os = "android") {
        defs.push(Some((MenuAction::ModeReference, "Mode référence", "")));
    }
    // Sans serveur, ces entrées n'ont pas d'objet : elles n'existent pas.
    if crate::telemetrie::serveur().is_some() {
        defs.push(None);
        defs.push(Some((
            MenuAction::JournalTechnique,
            "Journal technique…",
            "",
        )));
        // Un dossier à ouvrir dans l'explorateur : le téléphone n'en a pas (fiche 56).
        if !cfg!(target_os = "android") {
            defs.push(Some((MenuAction::VoirCeQuiPart, "Voir ce qui part", "")));
        }
    }
    defs
}
