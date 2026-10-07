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
pub mod ecran;
pub mod empreinte;
pub mod graphique;
pub mod heure;
pub mod identite;
pub mod journal;
pub mod moisson;
pub mod offre;
pub mod partage;
pub mod presse_papiers;
pub mod priorite;
pub mod rapatrier;
pub mod sources;
pub mod telechargements;

#[cfg(windows)]
mod depot_windows;
#[cfg(windows)]
mod glisser_windows;
#[cfg(windows)]
mod pave_windows;
#[cfg(not(windows))]
mod telechargement;
#[cfg(windows)]
mod telechargement_windows;

use moisson::Depot;
use std::sync::mpsc::Receiver;

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

/// Ce qu'un envoi fait à l'adresse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbe {
    /// Déposer un corps JSON (`POST`).
    Deposer,
    /// Effacer ce que l'adresse désigne (`DELETE`).
    Effacer,
}

/// **Envoie à une adresse web**, et rend le code de sa réponse (fiche 54) : la même porte que
/// [`telecharger`], dans l'autre sens.
pub fn envoyer(url: &str, verbe: Verbe, corps: &[u8]) -> Result<u16, String> {
    let adresse = sources::decouper(url).ok_or_else(|| format!("adresse refusee : {url}"))?;
    let verbe = match verbe {
        Verbe::Deposer => "POST",
        Verbe::Effacer => "DELETE",
    };
    #[cfg(windows)]
    let voie = telechargement_windows::envoyer;
    #[cfg(not(windows))]
    let voie = telechargement::envoyer;
    voie(&adresse, verbe, corps)
}

/// **Glisse un lot de nœuds hors de cette fenêtre** (fiche 51 § 2), jusqu'au lâcher : rend
/// vrai s'il a été déposé quelque part. La fenêtre vient d'être peinte : rien ne s'y redessine
/// pendant le geste. Sous Windows seulement, comme la cible : ailleurs, le glisser de `winit`
/// ne sait porter que des fichiers, et rien ne part.
pub fn glisser_un_lot(
    fenetre: &winit::window::Window,
    texte: Option<&str>,
    lot: &[u8],
) -> Result<bool, String> {
    #[cfg(windows)]
    {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let poignee = fenetre.window_handle().map_err(|e| e.to_string())?;
        let RawWindowHandle::Win32(w) = poignee.as_raw() else {
            return Err("une fenetre Windows sans poignee Win32".into());
        };
        glisser_windows::glisser(w.hwnd.get() as *mut core::ffi::c_void, texte, lot)
    }
    #[cfg(not(windows))]
    {
        let _ = (fenetre, texte, lot);
        Ok(false)
    }
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
    /// Sous Android, la boîte des partages branchée sur ce pont (PARTAGE-1) : elle se
    /// débranche quand il s'en va.
    _partages: Option<partage::Branchement>,
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
    #[cfg(target_os = "android")]
    {
        let _ = fenetre;
        let partages = Some(partage::brancher(envoyer, reveil));
        Some(Depots {
            recevoir,
            _partages: partages,
        })
    }
    #[cfg(not(target_os = "android"))]
    poser_le_pont(fenetre, envoyer, reveil).then_some(Depots {
        recevoir,
        _partages: None,
    })
}

/// **De quoi reveiller la boucle d'images** depuis un autre fil.
pub type Reveil = std::sync::Arc<dyn Fn() + Send + Sync>;

/// Le pont de cette plateforme, s'il y en a un.
#[cfg(windows)]
fn poser_le_pont(
    fenetre: &winit::window::Window,
    vers: std::sync::mpsc::Sender<Depot>,
    reveil: Reveil,
) -> bool {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let Ok(poignee) = fenetre.window_handle() else {
        return false;
    };
    let RawWindowHandle::Win32(w) = poignee.as_raw() else {
        return false;
    };
    depot_windows::installer(w.hwnd.get(), vers, reveil)
}

/// **Le pavé de précision tel que le doigt le fait** (fiche 53) : *Direct Manipulation* sous
/// Windows. Rend `None` ailleurs, ou si Windows refuse — le pavé reste alors ce qu'il était,
/// des défilements de molette.
#[cfg(windows)]
pub fn installer_le_pave(
    fenetre: &std::sync::Arc<winit::window::Window>,
) -> Option<Box<dyn crate::interactions::pave::Pave>> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let RawWindowHandle::Win32(w) = fenetre.window_handle().ok()?.as_raw() else {
        return None;
    };
    let pour_le_reveil = std::sync::Arc::clone(fenetre);
    let reveil: Reveil = std::sync::Arc::new(move || pour_le_reveil.request_redraw());
    let taille = fenetre.inner_size();
    let pave = pave_windows::installer(w.hwnd.get(), (taille.width, taille.height), reveil)?;
    Some(Box::new(pave))
}

/// Ailleurs, le pavé arrive comme le système le donne : `winit` le dit déjà en pixels.
#[cfg(not(windows))]
pub fn installer_le_pave(
    _fenetre: &std::sync::Arc<winit::window::Window>,
) -> Option<Box<dyn crate::interactions::pave::Pave>> {
    None
}

/// Sur une plateforme sans pont, il n'y a rien à poser et rien à dire. Android a le sien :
/// la boîte des partages ([`partage`]).
#[cfg(not(any(windows, target_os = "android")))]
fn poser_le_pont(
    _fenetre: &winit::window::Window,
    _vers: std::sync::mpsc::Sender<Depot>,
    _reveil: Reveil,
) -> bool {
    false
}
