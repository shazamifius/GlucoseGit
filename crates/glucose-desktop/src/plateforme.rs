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

#[cfg(target_os = "linux")]
pub mod administrateur;
pub mod empreinte;
pub mod graphique;
pub mod heure;
pub mod identite;
pub mod moisson;
pub mod offre;
pub mod priorite;
pub mod rapatrier;
pub mod sources;
pub mod telechargements;

#[cfg(windows)]
mod depot_windows;
#[cfg(not(windows))]
mod telechargement;
#[cfg(windows)]
mod telechargement_windows;

use moisson::Depot;
use std::sync::mpsc::{Receiver, Sender};

/// **Les octets qu'une adresse web rend**, au plus `limite`, ou la raison de n'en pas rendre
/// (DEPOT-WEB-4).
///
/// Seules les adresses `http` et `https` passent ; [`sources::decouper`] refuse le reste. Une
/// porte, une voie par système : WinHTTP sous Windows, `ureq` ailleurs (decisions/02 et 05).
pub fn telecharger(url: &str, limite: usize) -> Result<Vec<u8>, String> {
    let adresse = sources::decouper(url).ok_or_else(|| format!("adresse refusee : {url}"))?;
    #[cfg(windows)]
    let voie = telechargement_windows::telecharger;
    #[cfg(not(windows))]
    let voie = telechargement::telecharger;
    voie(&adresse, limite)
}

/// **Le nom sous lequel Glucose se présente.**
///
/// Celui d'un navigateur courant : plusieurs serveurs d'images servent une page d'erreur, ou
/// rien, à un client qu'ils ne reconnaissent pas. On ne cache rien de ce qu'on demande — on
/// demande ce que l'utilisateur a glissé depuis son propre navigateur, et rien d'autre.
const NAVIGATEUR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                          (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36 Glucose";

/// **Combien de temps on attend chaque étape** : résoudre le nom, se connecter, envoyer,
/// recevoir.
///
/// Le téléchargement se fait hors de la boucle d'images, donc aucune image n'attend ; mais
/// un dépôt qui ne se pose jamais doit finir par retomber sur son repli. Quinze secondes, c'est
/// ce qu'un navigateur accorde avant d'afficher qu'une page ne répond pas.
const DELAI: std::time::Duration = std::time::Duration::from_secs(15);

/// Par où les dépôts du système rejoignent la boucle d'images.
///
/// Un canal, et non un appel direct, parce que le système appelle **quand il veut** : au
/// milieu de sa propre boucle de messages, pendant que la boucle d'images lit le document. Le
/// canal est ce qui rend l'instant du dépôt indépendant de l'instant où on le pose.
pub struct Depots {
    /// Le bout que la boucle d'images draine.
    recevoir: Receiver<Depot>,
}

impl Depots {
    /// **Tout ce qui est arrivé depuis le dernier passage**, et rien de bloquant.
    ///
    /// Un lot par dépôt : glisser huit images d'une page est un geste, et huit gestes en sont
    /// huit. C'est `interactions::drop` qui décide qu'un lot tient dans une entrée d'annulation.
    pub fn recolter(&self) -> Vec<Depot> {
        self.recevoir.try_iter().collect()
    }
}

/// **Installe les ponts natifs de cette fenêtre**, et rend de quoi les écouter.
///
/// Rend `None` quand cette plateforme n'a pas de pont, ou quand le système l'a refusé.
/// L'application marche alors exactement comme avant : `winit` garde son glisser-déposer de
/// fichiers, et seul le dépôt depuis un navigateur manque.
pub fn installer(fenetre: &std::sync::Arc<winit::window::Window>) -> Option<Depots> {
    let (envoyer, recevoir) = std::sync::mpsc::channel();
    // **Un depot peut arriver pendant que la boucle dort** (DEPOT-WEB-4) : l'image rapatriee
    // sur un fil a part tombe une seconde apres le geste, et sans ce reveil elle attendrait le
    // prochain mouvement de souris pour paraitre. `request_redraw` se demande de n'importe quel
    // fil, et la boucle pose ce qui est arrive des qu'elle a fini de le traiter.
    let fenetre_pour_le_reveil = std::sync::Arc::clone(fenetre);
    let reveil: Reveil = std::sync::Arc::new(move || fenetre_pour_le_reveil.request_redraw());
    poser_le_pont(fenetre, envoyer, reveil).then_some(Depots { recevoir })
}

/// **De quoi reveiller la boucle d'images** depuis un autre fil.
pub type Reveil = std::sync::Arc<dyn Fn() + Send + Sync>;

/// Le pont de cette plateforme, s'il y en a un.
#[cfg(windows)]
fn poser_le_pont(fenetre: &winit::window::Window, vers: Sender<Depot>, reveil: Reveil) -> bool {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Ok(poignee) = fenetre.window_handle() else {
        return false;
    };
    let RawWindowHandle::Win32(w) = poignee.as_raw() else {
        return false;
    };
    depot_windows::installer(w.hwnd.get(), vers, reveil)
}

/// Sur une plateforme sans pont, il n'y a rien à poser et rien à dire.
#[cfg(not(windows))]
fn poser_le_pont(_fenetre: &winit::window::Window, _vers: Sender<Depot>, _reveil: Reveil) -> bool {
    false
}
