//! **Ce que le compositeur fait de chaque image** (PARU-1, fiche 112) — `wp_presentation`.
//!
//! # Pourquoi
//!
//! Sous Wayland, en `mailbox`, la présentation rend la main dès que l'image est confiée : rien
//! dans Glucose ne savait quand elle paraissait, ni si elle paraissait. La chronique a donc pris
//! le compteur du tempo pour la vérité de l'écran (fiche 111). Le protocole `wp_presentation`,
//! que niri, GNOME, KDE et sway proposent, le dit : pour chaque image confiée, **montrée** à tel
//! instant, au battement numéro tant — ou **jetée**.
//!
//! # Comment
//!
//! Comme le pincement ([`super::pave_wayland`], fiche 93) : sur la connexion même de la fenêtre
//! (`Backend::from_foreign_display`), une file d'événements à elle, lue par un fil à elle. La
//! surface est celle de `winit` (`ObjectId::from_ptr`) : la demande `feedback` vaut pour son
//! prochain `commit`, celui que la carte graphique fait en présentant — d'où l'appel juste avant
//! de présenter. Chaque demande porte l'instant de l'envoi, lu sur **l'horloge que le compositeur
//! annonce** (`clock_id`) : les deux instants se comparent sans rien supposer.
//!
//! Aucune caisse de plus : `wayland-protocols` est déjà là, et `presentation-time` est un protocole
//! stable (`decisions/14`).

use crate::chronique::ecran::{Retour, Sonde, Vue};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use wayland_client::backend::ObjectId;
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle};
use wayland_protocols::wp::presentation_time::client::{
    wp_presentation as presentation, wp_presentation_feedback as retour,
};

/// Ce que le fil des verdicts partage avec la boucle.
#[derive(Default)]
struct Partage {
    retours: Mutex<Vec<Retour>>,
    /// L'horloge du compositeur : la monotone, tant qu'il n'en a pas annoncé une autre — le
    /// protocole l'annonce dès la liaison.
    horloge: AtomicI32,
}

/// **La sonde Wayland** : demander, relever.
pub struct SondeWayland {
    presentation: presentation::WpPresentation,
    surface: wl_surface::WlSurface,
    qh: QueueHandle<Etat>,
    partage: Arc<Partage>,
}

impl Sonde for SondeWayland {
    fn envoyer(&self) -> u64 {
        let envoi = maintenant_us(self.partage.horloge.load(Ordering::Relaxed));
        let _ = self.presentation.feedback(&self.surface, &self.qh, envoi);
        envoi
    }

    fn relever(&self) -> Vec<Retour> {
        std::mem::take(
            &mut self
                .partage
                .retours
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        )
    }
}

/// L'état du fil : ce qu'il partage.
struct Etat(Arc<Partage>);

/// **Branche la sonde** sur la surface de la fenêtre — ou dit pourquoi pas : le compositeur ne
/// propose pas le protocole, ou la connexion ne se laisse pas partager.
///
/// # Sûreté
///
/// `affichage` et `surface` sont ceux de `winit`, vivants autant que la fenêtre — donc autant que
/// Glucose sur un bureau.
pub fn installer(
    affichage: std::ptr::NonNull<std::ffi::c_void>,
    surface: std::ptr::NonNull<std::ffi::c_void>,
) -> Result<SondeWayland, String> {
    // SAFETY : voir ci-dessus.
    let backend = unsafe {
        wayland_client::backend::Backend::from_foreign_display(affichage.as_ptr().cast())
    };
    let connexion = Connection::from_backend(backend);
    let (globaux, mut file) = registry_queue_init::<Etat>(&connexion)
        .map_err(|e| format!("la liste du compositeur ne se lit pas ({e})"))?;
    let qh = file.handle();
    let presentation: presentation::WpPresentation = globaux
        .bind(&qh, 1..=1, ())
        .map_err(|e| format!("le compositeur ne propose pas wp_presentation ({e})"))?;
    // SAFETY : la surface de `winit`, vivante autant que la fenêtre.
    let id =
        unsafe { ObjectId::from_ptr(wl_surface::WlSurface::interface(), surface.as_ptr().cast()) }
            .map_err(|_| "la surface de la fenêtre ne se laisse pas prendre".to_string())?;
    let surface = wl_surface::WlSurface::from_id(&connexion, id)
        .map_err(|_| "la surface de la fenêtre ne se laisse pas nommer".to_string())?;
    let partage = Arc::new(Partage {
        horloge: AtomicI32::new(libc::CLOCK_MONOTONIC),
        ..Partage::default()
    });
    let mut etat = Etat(Arc::clone(&partage));
    std::thread::Builder::new()
        .name("glucose-ecran".into())
        .spawn(move || while file.blocking_dispatch(&mut etat).is_ok() {})
        .map_err(|e| format!("le fil des verdicts ne naît pas ({e})"))?;
    Ok(SondeWayland {
        presentation,
        surface,
        qh,
        partage,
    })
}

/// L'instant présent sur cette horloge, en microsecondes.
fn maintenant_us(horloge: i32) -> u64 {
    let mut t = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY : une structure à nous, une horloge que le compositeur a nommée.
    unsafe { libc::clock_gettime(horloge, &mut t) };
    u64::try_from(t.tv_sec).unwrap_or(0) * 1_000_000 + u64::try_from(t.tv_nsec).unwrap_or(0) / 1_000
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for Etat {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<presentation::WpPresentation, ()> for Etat {
    fn event(
        etat: &mut Self,
        _: &presentation::WpPresentation,
        event: presentation::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let presentation::Event::ClockId { clk_id } = event {
            if let Ok(horloge) = i32::try_from(clk_id) {
                etat.0.horloge.store(horloge, Ordering::Relaxed);
            }
        }
    }
}

impl Dispatch<retour::WpPresentationFeedback, u64> for Etat {
    fn event(
        etat: &mut Self,
        _: &retour::WpPresentationFeedback,
        event: retour::Event,
        envoi: &u64,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let vue = match event {
            retour::Event::Presented {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                refresh,
                seq_hi,
                seq_lo,
                ..
            } => Some(Vue {
                us: ((u64::from(tv_sec_hi) << 32) | u64::from(tv_sec_lo)) * 1_000_000
                    + u64::from(tv_nsec) / 1_000,
                battement: (u64::from(seq_hi) << 32) | u64::from(seq_lo),
                periode_ns: refresh,
            }),
            retour::Event::Discarded => None,
            // `sync_output` dit sur quel écran : un seul verdict compte.
            _ => return,
        };
        etat.0
            .retours
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Retour {
                envoi_us: *envoi,
                vue,
            });
    }
}
