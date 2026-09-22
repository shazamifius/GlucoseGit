//! Ce que le système d'exploitation fait et que personne d'autre ne peut faire.
//!
//! # Pourquoi ce module existe, et pourquoi il est seul
//!
//! La fiche 05 § 8.4 demande que le `unsafe` vive dans **un** endroit nommé. Ce module l'est :
//! tout ce qui appelle Windows directement y est, et le reste de l'application ne connaît de
//! lui qu'un canal et un type de message.
//!
//! Aujourd'hui il ne porte qu'une chose, le dépôt depuis un navigateur ([`depot_windows`]).
//! Le pont du pincement (`interactions::pincement`) est un crochet `winit` sans `unsafe` et
//! reste là où il est ; il n'a rien à faire ici.
//!
//! # Ce qu'une plateforme sans pont donne
//!
//! Rien, et sans le dire. Sur macOS, Linux et Android, [`installer`] ne fait rien et
//! l'application garde le glisser-déposer de `winit` — c'est-à-dire l'état d'aujourd'hui, pas
//! une régression. La charte interdit d'exclure une machine ; elle n'interdit pas qu'un pont
//! natif arrive d'abord là où la mesure l'a réclamé.

pub mod empreinte;
pub mod moisson;

#[cfg(windows)]
mod depot_windows;

use moisson::Moisson;
use std::sync::mpsc::{Receiver, Sender};

/// Par où les dépôts du système rejoignent la boucle d'images.
///
/// Un canal, et non un appel direct, parce que le système appelle **quand il veut** : au
/// milieu de sa propre boucle de messages, pendant que la boucle d'images lit le document. Le
/// canal est ce qui rend l'instant du dépôt indépendant de l'instant où on le pose.
pub struct Depots {
    /// Le bout que la boucle d'images draine.
    recevoir: Receiver<Moisson>,
}

impl Depots {
    /// **Tout ce qui est arrivé depuis le dernier passage**, et rien de bloquant.
    ///
    /// Un lot par dépôt : glisser huit images d'une page est un geste, et huit gestes en sont
    /// huit. C'est `interactions::drop` qui décide qu'un lot tient dans une entrée d'annulation.
    pub fn recolter(&self) -> Vec<Moisson> {
        self.recevoir.try_iter().collect()
    }
}

/// **Installe les ponts natifs de cette fenêtre**, et rend de quoi les écouter.
///
/// Rend `None` quand cette plateforme n'a pas de pont, ou quand le système l'a refusé.
/// L'application marche alors exactement comme avant : `winit` garde son glisser-déposer de
/// fichiers, et seul le dépôt depuis un navigateur manque.
pub fn installer(fenetre: &winit::window::Window) -> Option<Depots> {
    let (envoyer, recevoir) = std::sync::mpsc::channel();
    poser_le_pont(fenetre, envoyer).then_some(Depots { recevoir })
}

/// Le pont de cette plateforme, s'il y en a un.
#[cfg(windows)]
fn poser_le_pont(fenetre: &winit::window::Window, vers: Sender<Moisson>) -> bool {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Ok(poignee) = fenetre.window_handle() else {
        return false;
    };
    let RawWindowHandle::Win32(w) = poignee.as_raw() else {
        return false;
    };
    depot_windows::installer(w.hwnd.get(), vers)
}

/// Sur une plateforme sans pont, il n'y a rien à poser et rien à dire.
#[cfg(not(windows))]
fn poser_le_pont(_fenetre: &winit::window::Window, _vers: Sender<Moisson>) -> bool {
    false
}
