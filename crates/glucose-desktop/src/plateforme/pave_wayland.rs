//! **Le pincement du pavé sous Wayland** (fiche 93) — que `winit` 0.30 ne transmet pas.
//!
//! # Le défaut
//!
//! Sous Linux, pincer le pavé ne zoomait pas du tout. `winit` ne connaît le pincement que sous
//! macOS et iOS (`PinchGesture`) ; sous Wayland, le geste passe par un protocole à part,
//! `zwp_pointer_gestures_v1`, qu'il n'écoute pas. Le compositeur le donnait (niri, GNOME, KDE,
//! sway le proposent tous) ; personne ne le prenait.
//!
//! # Ce que ce module fait
//!
//! Il se branche sur **la connexion même de la fenêtre** (`Backend::from_foreign_display` : le
//! pointeur de l'affichage que `winit` a ouvert), sur une file d'événements à lui, lue par un fil à
//! lui — libwayland laisse plusieurs lecteurs se partager la connexion. Il demande au siège un
//! pointeur, puis le pincement de ce pointeur. Chaque pas du geste dit l'échelle **depuis le début
//! du geste** et le glissement du centre des doigts : une similitude exacte
//! ([`Mouvement::pincement`]), versée dans le chemin de *Direct Manipulation* sous Windows
//! ([`Pave`]) — la vue la montre en entier à l'image suivante, rien n'est classé, aucun seuil.
//!
//! Le centre des doigts : le pointeur ne bouge pas pendant un geste ; il est le centre au début, et
//! chaque pas le fait glisser. Wayland le dit en coordonnées logiques de la surface ; la fenêtre
//! les compte en pixels, au facteur d'échelle de l'écran.
//!
//! Sous X11, ou sur un compositeur sans ce protocole, rien ne change : le pavé reste ce que
//! `winit` en dit.

use super::Reveil;
use crate::interactions::pave::{Mouvement, Pave};
use std::sync::{Arc, Mutex, PoisonError};
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::{wl_pointer, wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle, WEnum};
use wayland_protocols::wp::pointer_gestures::zv1::client::{
    zwp_pointer_gesture_pinch_v1 as pinch, zwp_pointer_gestures_v1 as gestes,
};

/// Ce que le fil des gestes laisse à la boucle de Glucose.
#[derive(Default)]
struct Partage {
    en_geste: bool,
    mouvements: Vec<Mouvement>,
}

/// **Le pavé Wayland**, tel que la boucle le voit : ce que les doigts ont fait, à prendre.
pub struct PaveWayland {
    partage: Arc<Mutex<Partage>>,
}

impl Pave for PaveWayland {
    fn en_geste(&self) -> bool {
        self.partage
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .en_geste
    }

    fn avancer(&mut self) -> Vec<Mouvement> {
        std::mem::take(
            &mut self
                .partage
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .mouvements,
        )
    }
}

/// **Un pincement en cours**, pas à pas — pur, et éprouvé sans compositeur.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pincement {
    /// L'échelle du pas précédent, depuis le début du geste.
    echelle: f64,
    /// Le centre des doigts, en coordonnées logiques de la surface.
    centre: (f64, f64),
}

impl Pincement {
    /// Le geste commence là où est le pointeur.
    pub fn commencer(pointeur: (f64, f64)) -> Self {
        Self {
            echelle: 1.0,
            centre: pointeur,
        }
    }

    /// **Un pas** : l'échelle depuis le début du geste, le glissement du centre depuis le pas
    /// précédent (logiques), le facteur d'échelle de l'écran. Rend la similitude en pixels.
    pub fn pas(&mut self, echelle: f64, glissement: (f64, f64), k: f64) -> Option<Mouvement> {
        if !(echelle > 0.0 && echelle.is_finite() && self.echelle > 0.0) {
            return None;
        }
        let r = echelle / self.echelle;
        let centre = (self.centre.0 * k, self.centre.1 * k);
        let m = Mouvement::pincement(r, centre, (glissement.0 * k, glissement.1 * k));
        self.echelle = echelle;
        self.centre = (self.centre.0 + glissement.0, self.centre.1 + glissement.1);
        Some(m)
    }
}

/// L'état du fil : ce qu'il a lié, où est le pointeur, le geste en cours.
struct Etat {
    partage: Arc<Mutex<Partage>>,
    reveil: Reveil,
    /// Le facteur d'échelle de l'écran, lu à chaque pas : il change quand la fenêtre change d'écran.
    facteur: Box<dyn Fn() -> f64 + Send>,
    gestes: gestes::ZwpPointerGesturesV1,
    pointeur: Option<wl_pointer::WlPointer>,
    position: (f64, f64),
    pincement: Option<Pincement>,
}

/// **Écoute le pincement** de la fenêtre dont `affichage` est l'affichage Wayland — ou dit pourquoi
/// pas : le compositeur ne propose pas le protocole, ou la connexion ne se laisse pas partager.
///
/// # Sûreté
///
/// `affichage` est le `wl_display` vivant de `winit`, qui vit autant que la boucle — donc autant
/// que Glucose.
pub fn installer(
    affichage: std::ptr::NonNull<std::ffi::c_void>,
    facteur: Box<dyn Fn() -> f64 + Send>,
    reveil: Reveil,
) -> Result<PaveWayland, String> {
    // SAFETY : voir ci-dessus — l'affichage de `winit` survit à ce fil, qui meurt avec Glucose.
    let backend = unsafe {
        wayland_client::backend::Backend::from_foreign_display(affichage.as_ptr().cast())
    };
    let connexion = Connection::from_backend(backend);
    let (globaux, mut file) = registry_queue_init::<Etat>(&connexion)
        .map_err(|e| format!("la liste du compositeur ne se lit pas ({e})"))?;
    let qh = file.handle();
    let gestes: gestes::ZwpPointerGesturesV1 = globaux
        .bind(&qh, 1..=1, ())
        .map_err(|e| format!("le compositeur ne propose pas pointer-gestures ({e})"))?;
    let _siege: wl_seat::WlSeat = globaux
        .bind(&qh, 1..=5, ())
        .map_err(|e| format!("aucun siège ({e})"))?;
    let partage = Arc::new(Mutex::new(Partage::default()));
    let mut etat = Etat {
        partage: Arc::clone(&partage),
        reveil,
        facteur,
        gestes,
        pointeur: None,
        position: (0.0, 0.0),
        pincement: None,
    };
    std::thread::Builder::new()
        .name("glucose-gestes".into())
        .spawn(move || while file.blocking_dispatch(&mut etat).is_ok() {})
        .map_err(|e| format!("le fil des gestes ne naît pas ({e})"))?;
    Ok(PaveWayland { partage })
}

impl Etat {
    fn verser(&mut self, m: Option<Mouvement>, en_geste: bool) {
        let mut p = self.partage.lock().unwrap_or_else(PoisonError::into_inner);
        p.en_geste = en_geste;
        p.mouvements.extend(m);
        drop(p);
        (self.reveil)();
    }
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

impl Dispatch<wl_seat::WlSeat, ()> for Etat {
    fn event(
        etat: &mut Self,
        siege: &wl_seat::WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(c),
        } = event
        else {
            return;
        };
        if c.contains(wl_seat::Capability::Pointer) && etat.pointeur.is_none() {
            let pointeur = siege.get_pointer(qh, ());
            etat.gestes.get_pinch_gesture(&pointeur, qh, ());
            etat.pointeur = Some(pointeur);
        }
    }
}

impl Dispatch<wl_pointer::WlPointer, ()> for Etat {
    fn event(
        etat: &mut Self,
        _: &wl_pointer::WlPointer,
        event: wl_pointer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter {
                surface_x,
                surface_y,
                ..
            }
            | wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => etat.position = (surface_x, surface_y),
            _ => {}
        }
    }
}

impl Dispatch<gestes::ZwpPointerGesturesV1, ()> for Etat {
    fn event(
        _: &mut Self,
        _: &gestes::ZwpPointerGesturesV1,
        _: gestes::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<pinch::ZwpPointerGesturePinchV1, ()> for Etat {
    fn event(
        etat: &mut Self,
        _: &pinch::ZwpPointerGesturePinchV1,
        event: pinch::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            pinch::Event::Begin { .. } => {
                etat.pincement = Some(Pincement::commencer(etat.position));
                etat.verser(None, true);
            }
            pinch::Event::Update { dx, dy, scale, .. } => {
                let k = (etat.facteur)();
                let m = etat
                    .pincement
                    .as_mut()
                    .and_then(|p| p.pas(scale, (dx, dy), k));
                etat.verser(m, true);
            }
            pinch::Event::End { .. } => {
                etat.pincement = None;
                etat.verser(None, false);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
