//! Ce que Glucose montre au premier lancement.
//!
//! Un sujet à part entière, et pas une ligne du constructeur : ce que l'on trouve en ouvrant
//! le logiciel est une décision sur le **document**, qui se lit et se change seule.

use super::{text_card, Renderer, Store, WELCOME_TEXT};

/// Le document qu'on trouve au lancement : une carte d'accueil, et rien à défaire.
///
/// Extrait de `new` parce que c'est une décision sur le **document**, pas sur l'application :
/// elle se lit seule, et le constructeur n'a plus qu'à assembler des champs.
pub(super) fn document_d_accueil(renderer: &Renderer) -> Store {
    let mut store = Store::new("Glucose Native");
    let active_bid = store.project.active_board_id.clone();
    // La carte d'accueil naît par la même fabrique qu'une carte posée d'un clic : même
    // largeur de naissance, même hauteur suivie (TEXT-FIT-1).
    let welcome = text_card(
        &renderer.typography,
        &renderer.math,
        "welcome-card",
        0.0,
        0.0,
        WELCOME_TEXT,
    );
    store.add_annotation(&active_bid, welcome);
    // La carte d'accueil n'est pas une modification de l'utilisateur : le document part
    // propre, sans marqueur dans le titre — et sans rien à défaire. Tant que seul le
    // marqueur était traité, Ctrl+Z était actif dès le lancement et retirait une carte
    // que personne n'avait posée.
    store.journal.clear();
    store
}
