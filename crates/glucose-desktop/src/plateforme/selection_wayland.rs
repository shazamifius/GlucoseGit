//! **Le presse-papiers et le glisser-déposer de Wayland, servis par Glucose lui-même** (fiche 100).
//!
//! # Le défaut
//!
//! Son essai du 9/11 sous niri : *« IMPOSSIBLE de copier-coller une image dans Glucose, ainsi que
//! TOUT le système de glisser-déposer »*, et copier un texte de Glucose vers Discord, impossible.
//! Glucose est une fenêtre Wayland (c'est ce que `winit` ouvre dès que `WAYLAND_DISPLAY` existe),
//! mais il ne parlait au presse-papiers que par X11 ([`super::selection_x11`], `arboard`), donc par
//! XWayland. Sous GNOME et KDE, le compositeur relaie lui-même la sélection de X11 à Wayland ; sous
//! niri, XWayland vit dans `xwayland-satellite`, un client comme un autre, qui ne voit le
//! presse-papiers de Wayland qu'avec le clavier — jamais quand c'est Glucose qui l'a. Et `winit`
//! 0.30 ne transmet **aucun** glisser-déposer sous Wayland (`DroppedFile` n'existe que sous X11) :
//! rien de ce qu'on lâchait sur la fenêtre n'arrivait.
//!
//! # Ce que ce module fait
//!
//! Comme le pincement ([`super::pave_wayland`], fiche 93), il se branche sur **la connexion même de
//! la fenêtre**, sur une file et un fil à lui, et parle le protocole de base, `wl_data_device` —
//! celui de GTK, de Qt, de Chromium : aucun protocole réservé aux gestionnaires de presse-papiers,
//! donc tous les bureaux Wayland, GNOME compris.
//!
//! * **Copier** : une source qui offre les types de [`super::selection_unix`] — la même règle que
//!   sous X11 et Windows —, et une **marque** à elle, qui dit, quand la sélection revient, que c'est
//!   la nôtre (un `Ctrl+V` se sert alors sans tuyau). La copie n'est dite réussie que lorsque le
//!   compositeur annonce la sélection marquée : un refus se dit, il ne se tait pas.
//! * **Coller** : la sélection annoncée et ses types ; le texte, l'image (PNG d'abord), les
//!   fichiers, le lot, lus par un tuyau, la patience bornée ([`tuyau`]).
//! * **Déposer** : ce qu'on lâche sur la fenêtre devient la même moisson que sous Windows
//!   ([`depot`]), posée là où le curseur a lâché, ou rapatriée si ce n'est qu'une adresse.
//! * **Glisser hors de la fenêtre** : des nœuds tenus qui la quittent partent vers l'application où
//!   on les lâche (`start_drag`), sous les formes d'un glisser — le lot, le texte, les images en
//!   fichiers —, comme sous Windows (fiche 51 § 2). Un glisser parti d'ici ne se dépose pas ici.
//!
//! Le compositeur exige, pour poser la sélection, le numéro d'un évènement de la main
//! (`serial`) : ce fil tient son propre clavier et son propre pointeur, qui reçoivent les mêmes
//! évènements que ceux de `winit`, et retient le dernier.
//!
//! # Prudence
//!
//! La connexion est celle de la fenêtre : une faute de protocole la fermerait, et Glucose avec.
//! Chaque requête qui peut en être une (`set_actions`, `finish`) n'est envoyée que dans l'état où
//! la spécification la permet, et c'est écrit à côté de chacune.

mod depot;
mod etat;
#[cfg(test)]
mod tests;
mod tuyau;

use super::moisson::Depot;
use super::selection_unix::{Offre, Rendu, LISTE_D_URI, LOT, PNG, TEXTES};
use crate::interactions::clipboard::{Formes, Pour};
use crate::interactions::presse_papiers::Pixels;
use etat::Etat;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc::Sender, Arc, Condvar, Mutex, OnceLock, PoisonError};
use std::time::Duration;
use wayland_client::protocol::{
    wl_data_device::WlDataDevice, wl_data_device_manager::WlDataDeviceManager,
    wl_data_offer::WlDataOffer, wl_data_source::WlDataSource,
};
use wayland_client::{Connection, QueueHandle};

/// **Le type qui marque notre sélection**, suivi du processus et du numéro de la copie : une autre
/// application ne le demande jamais.
const MARQUE: &str = "application/x-glucose-marque";

/// Ce qu'on attend que le compositeur annonce la sélection qu'on vient de poser : il répond dans la
/// milliseconde ; au-delà, il l'a refusée (Glucose n'avait pas le clavier).
const ANNONCE: Duration = Duration::from_millis(500);

/// Le plus gros transfert qu'on accepte : la borne d'un dépôt ([`super::moisson::OCTETS_MAX`]).
const PLAFOND: usize = super::moisson::OCTETS_MAX;

/// Le numéro de la prochaine copie.
static PROCHAINE: AtomicU64 = AtomicU64::new(1);

/// **Les types qu'une offre annonce**, et l'action qu'un glisser a convenue.
#[derive(Debug, Default)]
pub(super) struct Mimes {
    types: Mutex<Vec<String>>,
    /// Une action a été convenue (`wl_data_offer.action`, version 3) : sans elle, `finish` est une
    /// faute de protocole.
    convenue: std::sync::atomic::AtomicBool,
}

impl Mimes {
    fn types(&self) -> Vec<String> {
        self.types
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn porte(&self, type_: &str) -> bool {
        self.types
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .any(|t| t == type_)
    }
}

/// **Ce que porte une source qu'on a posée** : ce qu'elle offre, et sa marque.
pub(super) struct Servie {
    offre: Arc<Offre<String>>,
    marque: String,
}

/// Ce que le fil de Wayland tient à jour, et que les autres lisent.
#[derive(Default)]
pub(super) struct Courant {
    /// Le dernier évènement de la main, clavier ou pointeur.
    serial: u32,
    /// Le dernier bouton enfoncé : un glisser ne part que du clic qui tient encore.
    bouton: u32,
    /// La sélection que le compositeur annonce, et ses types.
    selection: Option<(WlDataOffer, Arc<Mimes>)>,
    /// La nôtre : sa marque, et ce qu'elle offre.
    notre: Option<(String, Arc<Offre<String>>)>,
}

/// L'état partagé, et l'annonce qui réveille qui l'attend.
#[derive(Default)]
pub(super) struct Partage {
    courant: Mutex<Courant>,
    annonce: Condvar,
}

impl Partage {
    fn courant(&self) -> std::sync::MutexGuard<'_, Courant> {
        self.courant.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// **Le pont**, tel que les autres fils l'emploient.
struct Pont {
    connexion: Connection,
    qh: QueueHandle<Etat>,
    gestionnaire: WlDataDeviceManager,
    appareil: WlDataDevice,
    partage: Arc<Partage>,
}

/// Le pont de la fenêtre, posé une fois.
static PONT: OnceLock<Pont> = OnceLock::new();

/// **Pose le pont sur la connexion de la fenêtre** dont `affichage` est l'affichage Wayland : les
/// dépôts partent par `depots`. Rend pourquoi pas, s'il ne se pose pas.
///
/// # Sûreté
///
/// `affichage` est le `wl_display` vivant de `winit`, qui vit autant que la boucle — donc autant
/// que Glucose.
pub(super) fn installer(
    affichage: std::ptr::NonNull<std::ffi::c_void>,
    facteur: Box<dyn Fn() -> f64 + Send>,
    depots: (Sender<Depot>, super::Reveil),
) -> Result<(), String> {
    // SAFETY : voir ci-dessus — l'affichage de `winit` survit à ce fil, qui meurt avec Glucose.
    let backend = unsafe {
        wayland_client::backend::Backend::from_foreign_display(affichage.as_ptr().cast())
    };
    let connexion = Connection::from_backend(backend);
    let partage = Arc::new(Partage::default());
    let (gestionnaire, appareil, qh) = etat::lancer(&connexion, &partage, facteur, depots)?;
    PONT.set(Pont {
        connexion,
        qh,
        gestionnaire,
        appareil,
        partage,
    })
    .map_err(|_| "le pont de Wayland est déjà posé".to_string())
}

/// **Le presse-papiers passe-t-il par Wayland ?** Oui dès que le pont est posé.
pub(super) fn actif() -> bool {
    PONT.get().is_some()
}

fn pont() -> Result<&'static Pont, String> {
    PONT.get()
        .ok_or_else(|| "le presse-papiers de Wayland n'est pas branché".into())
}

/// **Confie la sélection au presse-papiers**, sous toutes ses formes.
pub(super) fn confier(formes: Formes) -> Result<(), String> {
    pont()?.poser(Offre::nouvelle(formes, Pour::PressePapiers, str::to_string))
}

/// **Glisse ces formes hors de la fenêtre** dont `surface` est la surface Wayland : le compositeur
/// prend la main, et l'application où l'on lâche les demande.
///
/// # Sûreté
///
/// `surface` est la `wl_surface` vivante de la fenêtre de `winit`.
pub(super) fn glisser(
    surface: std::ptr::NonNull<std::ffi::c_void>,
    formes: Formes,
) -> Result<(), String> {
    use wayland_client::protocol::wl_surface::WlSurface;
    use wayland_client::Proxy;
    let pont = pont()?;
    // SAFETY : la surface de `winit` vit autant que sa fenêtre, que le glisser ne survit pas.
    let id = unsafe {
        wayland_client::backend::ObjectId::from_ptr(WlSurface::interface(), surface.as_ptr().cast())
    }
    .map_err(|_| "la surface de la fenêtre ne se lit pas".to_string())?;
    let origine = WlSurface::from_id(&pont.connexion, id).map_err(|e| e.to_string())?;
    let offre = Offre::nouvelle(formes, Pour::Glisser, str::to_string);
    let (source, _) = pont.source(offre);
    // Un glisser dit ses actions avant de partir (version 3) : la copie, seule.
    if source.version() >= 3 {
        source.set_actions(wayland_client::protocol::wl_data_device_manager::DndAction::Copy);
    }
    let bouton = pont.partage.courant().bouton;
    pont.appareil
        .start_drag(Some(&source), &origine, None, bouton);
    pont.connexion.flush().map_err(|e| e.to_string())
}

/// **Un glisser parti de ce Glucose ?** Ses types portent notre marque : il ne se dépose pas ici,
/// comme sous Windows — les nœuds n'ont pas bougé, et les recoller les doublerait.
pub(super) fn vient_d_ici(types: &[String]) -> bool {
    let notre = format!("{MARQUE};n={}.", std::process::id());
    types.iter().any(|t| t.starts_with(&notre))
}

/// **Confie un texte seul** : ce qu'on copie d'un texte qu'on écrit.
pub(super) fn confier_le_texte(texte: String) -> Result<(), String> {
    pont()?.poser(Offre::du_texte(texte, str::to_string))
}

/// Le texte que porte le presse-papiers.
pub(super) fn lire_le_texte() -> Result<String, String> {
    let octets = match pont()?.selection()? {
        Selection::Notre(offre) => offre.octets(Rendu::Texte)?.into_owned(),
        Selection::Autre(s) => {
            let type_ = TEXTES.iter().find(|t| s.mimes.porte(t)).ok_or(RIEN)?;
            s.recevoir(type_)?
        }
    };
    Ok(String::from_utf8_lossy(&octets).into_owned())
}

/// L'image que porte le presse-papiers : son PNG d'abord, puis tout type d'image que le décodeur
/// sait lire.
pub(super) fn lire_l_image() -> Result<Pixels, String> {
    let octets = match pont()?.selection()? {
        Selection::Notre(offre) => offre.octets(Rendu::Png)?.into_owned(),
        Selection::Autre(s) => {
            let types = s.mimes.types();
            let type_ = std::iter::once(PNG)
                .chain(types.iter().map(String::as_str))
                .find(|t| t.starts_with("image/") && types.iter().any(|o| o == t))
                .ok_or(RIEN)?;
            s.recevoir(type_)?
        }
    };
    let image = image::load_from_memory(&octets)
        .map_err(|e| e.to_string())?
        .to_rgba8();
    Ok(Pixels {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: image.into_raw(),
    })
}

/// **Les fichiers que porte le presse-papiers** — copiés dans un gestionnaire de fichiers —, ceux
/// qui existent ici.
pub(super) fn lire_les_fichiers() -> Vec<PathBuf> {
    let Ok(Selection::Autre(s)) = pont().and_then(Pont::selection) else {
        return Vec::new();
    };
    if !s.mimes.porte(LISTE_D_URI) {
        return Vec::new();
    }
    let liste = s.recevoir(LISTE_D_URI).unwrap_or_default();
    let (chemins, _) = super::selection_unix::lire_la_liste(&String::from_utf8_lossy(&liste));
    chemins.into_iter().filter(|c| c.exists()).collect()
}

/// **Le lot que porte le presse-papiers** : le nôtre se lit ici ; celui d'un autre Glucose, sur le
/// fil du collage ; une autre application n'est jamais sollicitée.
pub(super) fn lire_un_lot() -> Option<super::presse_papiers::LotALire> {
    match pont().and_then(Pont::selection).ok()? {
        Selection::Notre(offre) => {
            let lot = offre.lot().to_vec();
            (!lot.is_empty()).then(|| super::presse_papiers::lot_deja_la(lot))
        }
        Selection::Autre(s) => s
            .mimes
            .porte(LOT)
            .then(|| -> super::presse_papiers::LotALire { Box::new(move || s.recevoir(LOT).ok()) }),
    }
}

/// Ce qu'un presse-papiers qui ne porte pas la forme demandée répond.
const RIEN: &str = "le presse-papiers ne porte rien de tel";

/// La sélection, telle qu'un collage la lit.
enum Selection {
    /// La nôtre : elle se sert sans tuyau.
    Notre(Arc<Offre<String>>),
    /// Celle d'une autre application.
    Autre(Autre),
}

/// La sélection d'une autre application, et de quoi la lire.
struct Autre {
    connexion: Connection,
    offre: WlDataOffer,
    mimes: Arc<Mimes>,
}

impl Autre {
    /// **Reçoit ce type** : le bout d'écriture d'un tuyau part à l'application qui a copié, on lit
    /// l'autre jusqu'à ce qu'elle le ferme.
    fn recevoir(&self, type_: &str) -> Result<Vec<u8>, String> {
        recevoir(&self.connexion, &self.offre, type_)
    }
}

/// Reçoit ce type d'une offre — de sélection ou de dépôt.
fn recevoir(connexion: &Connection, offre: &WlDataOffer, type_: &str) -> Result<Vec<u8>, String> {
    use std::os::fd::AsFd;
    let (lecteur, ecrivain) = std::io::pipe().map_err(|e| e.to_string())?;
    offre.receive(type_.to_string(), ecrivain.as_fd());
    connexion.flush().map_err(|e| e.to_string())?;
    // Notre bout d'écriture se ferme : seul celui de l'autre application tient le tuyau ouvert.
    drop(ecrivain);
    tuyau::lire_tout(lecteur, PLAFOND)
}

impl Pont {
    /// **Une source qui offre ceci**, et sa marque.
    fn source(&self, offre: Offre<String>) -> (WlDataSource, (String, Arc<Offre<String>>)) {
        let n = PROCHAINE.fetch_add(1, Ordering::Relaxed);
        let marque = format!("{MARQUE};n={}.{n}", std::process::id());
        let offre = Arc::new(offre);
        let servie = Servie {
            offre: Arc::clone(&offre),
            marque: marque.clone(),
        };
        let source: WlDataSource = self.gestionnaire.create_data_source(&self.qh, servie);
        for cible in offre.cibles() {
            source.offer(cible);
        }
        source.offer(marque.clone());
        (source, (marque, offre))
    }

    /// **Pose cette offre comme sélection**, et attend que le compositeur l'annonce.
    fn poser(&self, offre: Offre<String>) -> Result<(), String> {
        let (source, (marque, offre)) = self.source(offre);
        let courant = self.partage.courant();
        self.appareil.set_selection(Some(&source), courant.serial);
        self.connexion.flush().map_err(|e| e.to_string())?;
        let (mut courant, attente) = self
            .partage
            .annonce
            .wait_timeout_while(courant, ANNONCE, |c| {
                !c.selection.as_ref().is_some_and(|(_, m)| m.porte(&marque))
            })
            .unwrap_or_else(PoisonError::into_inner);
        if attente.timed_out() {
            drop(courant);
            source.destroy();
            let _ = self.connexion.flush();
            return Err("le bureau n'a pas pris la copie".into());
        }
        courant.notre = Some((marque, offre));
        Ok(())
    }

    /// La sélection que le compositeur annonce : la nôtre, ou celle d'une autre application.
    fn selection(&self) -> Result<Selection, String> {
        let courant = self.partage.courant();
        let (offre, mimes) = courant.selection.clone().ok_or(RIEN)?;
        if let Some((marque, notre)) = &courant.notre {
            if mimes.porte(marque) {
                return Ok(Selection::Notre(Arc::clone(notre)));
            }
        }
        Ok(Selection::Autre(Autre {
            connexion: self.connexion.clone(),
            offre,
            mimes,
        }))
    }
}
