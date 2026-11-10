//! **Le fil du presse-papiers de Wayland** : ce que le compositeur annonce, rangé pour les autres
//! fils — la sélection et ses types, le glisser qui passe sur la fenêtre, les demandes faites à nos
//! sources, et le dernier évènement de la main.

use super::{depot, tuyau, Courant, Mimes, Partage, Servie};
use crate::plateforme::moisson::Depot;
use std::sync::atomic::Ordering;
use std::sync::{mpsc::Sender, Arc};
use wayland_client::globals::{registry_queue_init, GlobalListContents};
use wayland_client::protocol::wl_data_device_manager::{DndAction, WlDataDeviceManager};
use wayland_client::protocol::{
    wl_data_device, wl_data_offer, wl_data_source, wl_keyboard, wl_pointer, wl_registry, wl_seat,
};
use wayland_client::{event_created_child, Connection, Dispatch, Proxy, QueueHandle, WEnum};

/// Un glisser qui passe sur la fenêtre.
struct Glisser {
    offre: wl_data_offer::WlDataOffer,
    mimes: Arc<Mimes>,
    /// Où il est, en coordonnées logiques de la surface.
    position: (f64, f64),
}

/// L'état du fil.
pub(super) struct Etat {
    partage: Arc<Partage>,
    connexion: Connection,
    clavier: Option<wl_keyboard::WlKeyboard>,
    pointeur: Option<wl_pointer::WlPointer>,
    glisser: Option<Glisser>,
    /// Le facteur d'échelle de l'écran : Wayland compte en points, la fenêtre en pixels.
    facteur: Box<dyn Fn() -> f64 + Send>,
    depots: (Sender<Depot>, crate::plateforme::Reveil),
}

/// **Lance le fil** sur une file à lui, et rend de quoi poser une sélection.
pub(super) fn lancer(
    connexion: &Connection,
    partage: &Arc<Partage>,
    facteur: Box<dyn Fn() -> f64 + Send>,
    depots: (Sender<Depot>, crate::plateforme::Reveil),
) -> Result<
    (
        WlDataDeviceManager,
        wl_data_device::WlDataDevice,
        QueueHandle<Etat>,
    ),
    String,
> {
    let (globaux, mut file) = registry_queue_init::<Etat>(connexion)
        .map_err(|e| format!("la liste du compositeur ne se lit pas ({e})"))?;
    let qh = file.handle();
    let gestionnaire: WlDataDeviceManager = globaux
        .bind(&qh, 1..=3, ())
        .map_err(|e| format!("le compositeur ne propose pas de presse-papiers ({e})"))?;
    let siege: wl_seat::WlSeat = globaux
        .bind(&qh, 1..=7, ())
        .map_err(|e| format!("aucun siège ({e})"))?;
    let appareil = gestionnaire.get_data_device(&siege, &qh, ());
    let mut etat = Etat {
        partage: Arc::clone(partage),
        connexion: connexion.clone(),
        clavier: None,
        pointeur: None,
        glisser: None,
        facteur,
        depots,
    };
    std::thread::Builder::new()
        .name("glucose-presse-papiers".into())
        .spawn(move || while file.blocking_dispatch(&mut etat).is_ok() {})
        .map_err(|e| format!("le fil du presse-papiers ne naît pas ({e})"))?;
    Ok((gestionnaire, appareil, qh))
}

impl Etat {
    /// Retient le numéro d'un évènement de la main : c'est lui que `set_selection` demande.
    fn retenir(&self, serial: u32) {
        self.partage.courant().serial = serial;
    }

    /// **Un glisser est lâché** : sa moisson se lit sur un fil à part — l'application d'où l'on
    /// glisse peut être lente, et ce fil-ci doit continuer de servir.
    fn recolter(&mut self) {
        let Some(g) = self.glisser.take() else {
            return;
        };
        let k = (self.facteur)();
        let ou = (g.position.0 * k, g.position.1 * k);
        let connexion = self.connexion.clone();
        let (vers, reveil) = (self.depots.0.clone(), Arc::clone(&self.depots.1));
        std::thread::spawn(move || {
            let types = g.mimes.types();
            let (mut moisson, adresses) =
                depot::recolter(&types, |t| super::recevoir(&connexion, &g.offre, t).ok());
            // `finish` n'est permis qu'après une action convenue ; sinon, une faute de protocole
            // fermerait la connexion — celle de la fenêtre.
            if g.offre.version() >= 3 && g.mimes.convenue.load(Ordering::Relaxed) {
                g.offre.finish();
            }
            g.offre.destroy();
            let _ = connexion.flush();
            moisson.ou = Some(ou);
            if !adresses.is_empty() {
                crate::plateforme::rapatrier::rapatrier(adresses, moisson, (vers, reveil));
            } else if !moisson.est_vide()
                && vers
                    .send(Depot::Pose {
                        numero: None,
                        moisson,
                    })
                    .is_ok()
            {
                reveil();
            }
        });
    }

    /// **Un glisser entre sur la fenêtre** : on dit ce qu'on prendra, et on ne convient que de la
    /// copie — un déplacement ferait effacer l'original par l'application d'où l'on glisse.
    fn entrer(
        &mut self,
        serial: u32,
        position: (f64, f64),
        offre: Option<wl_data_offer::WlDataOffer>,
    ) {
        if let Some(ancien) = self.glisser.take() {
            ancien.offre.destroy();
        }
        let Some(offre) = offre else {
            return;
        };
        let Some(mimes) = offre.data::<Arc<Mimes>>().cloned() else {
            return;
        };
        let types = mimes.types();
        let accepte = depot::type_accepte(&types)
            .filter(|_| !super::vient_d_ici(&types))
            .cloned();
        let rien = accepte.is_none();
        offre.accept(serial, accepte);
        // `set_actions` n'est permis que sur l'offre d'un glisser, version 3, avec une préférence
        // comprise dans le masque : la copie, seule.
        if offre.version() >= 3 && !rien {
            offre.set_actions(DndAction::Copy, DndAction::Copy);
        }
        self.glisser = Some(Glisser {
            offre,
            mimes,
            position,
        });
    }
}

impl Courant {
    /// La sélection change : l'ancienne offre ne servira plus.
    fn remplacer(&mut self, selection: Option<(wl_data_offer::WlDataOffer, Arc<Mimes>)>) {
        if let Some((ancienne, _)) = std::mem::replace(&mut self.selection, selection) {
            ancienne.destroy();
        }
    }
}

impl Dispatch<wl_data_device::WlDataDevice, ()> for Etat {
    event_created_child!(Etat, wl_data_device::WlDataDevice, [
        wl_data_device::EVT_DATA_OFFER_OPCODE => (wl_data_offer::WlDataOffer, Arc::new(Mimes::default()))
    ]);

    fn event(
        etat: &mut Self,
        _: &wl_data_device::WlDataDevice,
        event: wl_data_device::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        use wl_data_device::Event;
        match event {
            Event::Selection { id } => {
                let selection = id.and_then(|o| Some((o.clone(), o.data::<Arc<Mimes>>()?.clone())));
                etat.partage.courant().remplacer(selection);
                etat.partage.annonce.notify_all();
            }
            Event::Enter {
                serial, x, y, id, ..
            } => etat.entrer(serial, (x, y), id),
            Event::Motion { x, y, .. } => {
                if let Some(g) = etat.glisser.as_mut() {
                    g.position = (x, y);
                }
            }
            Event::Leave => {
                if let Some(g) = etat.glisser.take() {
                    g.offre.destroy();
                }
            }
            Event::Drop => etat.recolter(),
            _ => {}
        }
    }
}

impl Dispatch<wl_data_offer::WlDataOffer, Arc<Mimes>> for Etat {
    fn event(
        _: &mut Self,
        _: &wl_data_offer::WlDataOffer,
        event: wl_data_offer::Event,
        mimes: &Arc<Mimes>,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_data_offer::Event::Offer { mime_type } => mimes
                .types
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(mime_type),
            wl_data_offer::Event::Action {
                dnd_action: WEnum::Value(action),
            } => mimes
                .convenue
                .store(action.contains(DndAction::Copy), Ordering::Relaxed),
            _ => {}
        }
    }
}

impl Dispatch<wl_data_source::WlDataSource, Servie> for Etat {
    fn event(
        etat: &mut Self,
        source: &wl_data_source::WlDataSource,
        event: wl_data_source::Event,
        servie: &Servie,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            // **Une application colle** : les octets se fabriquent et s'écrivent sur un fil à part
            // — des fichiers peuvent s'écrire, l'autre application peut lire lentement.
            wl_data_source::Event::Send { mime_type, fd } => {
                let offre = Arc::clone(&servie.offre);
                let rendu = offre.rendu(&mime_type);
                std::thread::spawn(move || {
                    let octets = rendu.and_then(|r| offre.octets(r).ok());
                    let _ = tuyau::ecrire_tout(fd, octets.as_deref().unwrap_or_default());
                });
            }
            // **Un glisser est fini** : la source ne servira plus.
            wl_data_source::Event::DndFinished => source.destroy(),
            // **Une autre application a pris la sélection**, ou le glisser est abandonné : celle-ci
            // ne servira plus.
            wl_data_source::Event::Cancelled => {
                let mut courant = etat.partage.courant();
                if courant
                    .notre
                    .as_ref()
                    .is_some_and(|(marque, _)| *marque == servie.marque)
                {
                    courant.notre = None;
                }
                drop(courant);
                source.destroy();
            }
            _ => {}
        }
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
        if c.contains(wl_seat::Capability::Keyboard) && etat.clavier.is_none() {
            etat.clavier = Some(siege.get_keyboard(qh, ()));
        }
        if c.contains(wl_seat::Capability::Pointer) && etat.pointeur.is_none() {
            etat.pointeur = Some(siege.get_pointer(qh, ()));
        }
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for Etat {
    fn event(
        etat: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Enter { serial, .. } | wl_keyboard::Event::Key { serial, .. } => {
                etat.retenir(serial);
            }
            _ => {}
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
        if let wl_pointer::Event::Button { serial, state, .. } = event {
            etat.retenir(serial);
            if state == WEnum::Value(wl_pointer::ButtonState::Pressed) {
                etat.partage.courant().bouton = serial;
            }
        }
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

impl Dispatch<WlDataDeviceManager, ()> for Etat {
    fn event(
        _: &mut Self,
        _: &WlDataDeviceManager,
        _: <WlDataDeviceManager as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
