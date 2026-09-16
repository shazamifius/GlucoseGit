//! Les dialogues natifs, ancrés à la fenêtre de Glucose.
//!
//! # Invariant DIAL-1 — un dialogue s'accroche toujours à la fenêtre qui l'ouvre
//!
//! Un dialogue natif ouvert **sans parent** n'appartient à aucune fenêtre. Sur Windows, le
//! gestionnaire est alors libre de le placer où il veut dans l'ordre d'empilement, et il le
//! place volontiers **derrière** la fenêtre principale — qui, elle, attend la réponse sans
//! rien afficher.
//!
//! Vu de l'utilisateur, l'application est figée : la croix ne fait rien, aucun bouton ne
//! répond, et il ne reste qu'à tuer le processus depuis le gestionnaire des tâches. Le
//! symptôme ne ressemble en rien à sa cause, ce qui est le propre de ce défaut : aucun test
//! ne le voit, aucun journal ne le dit, et le programme fait exactement ce qu'on lui a
//! demandé.
//!
//! Le remède est d'une ligne par dialogue, et il vaut pour **tous** : la question des
//! modifications non enregistrées, l'ouverture, l'enregistrement et l'import d'images. Un
//! dialogue ancré s'affiche devant son parent, le bloque proprement, et se ferme avec lui.

use winit::window::Window;

/// Une boîte de message accrochée à `parent`.
///
/// `None` n'arrive qu'avant que la fenêtre existe — au tout début, ou en test. Le dialogue
/// s'ouvre alors sans parent, ce qui est le seul comportement possible et ne bloque personne,
/// puisqu'il n'y a pas encore de fenêtre à bloquer.
pub fn message(parent: Option<&Window>) -> rfd::MessageDialog {
    let dialogue = rfd::MessageDialog::new();
    match parent {
        Some(fenetre) => dialogue.set_parent(fenetre),
        None => dialogue,
    }
}

/// Un sélecteur de fichier accroché à `parent`. Même règle que [`message`].
pub fn fichier(parent: Option<&Window>) -> rfd::FileDialog {
    let dialogue = rfd::FileDialog::new();
    match parent {
        Some(fenetre) => dialogue.set_parent(fenetre),
        None => dialogue,
    }
}
