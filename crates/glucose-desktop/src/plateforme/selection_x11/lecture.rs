//! **Lire le lot d'un autre Glucose** — hors du fil qui dessine, sur une connexion à soi, par
//! tranches s'il le faut (ICCCM 2.7.2, du côté de celui qui demande).
//!
//! On ne demande qu'à une fenêtre qui porte la marque de Glucose ([`super::Proprietaire::lot`]) :
//! son fil de sélection répond tout de suite. L'attente finit sur la réponse, ou quand cette
//! fenêtre disparaît ; [`PATIENCE`] ne garde que d'un Glucose figé.

use super::{Atomes, PATIENCE};
use std::os::fd::AsRawFd;
use std::time::Instant;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, CreateWindowAux, EventMask, Property,
    Window, WindowClass,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::{COPY_FROM_PARENT, CURRENT_TIME, NONE};

/// **Le lot que tient la fenêtre de sélection de cet autre Glucose**, ou rien.
pub(super) fn lire_le_lot_de(ecran: Option<&str>, proprio: Window) -> Option<Vec<u8>> {
    Lecteur::ouvrir(ecran)?.lire(proprio)
}

/// Une connexion et une fenêtre à soi : la réponse s'y dépose. Les deux disparaissent avec la
/// connexion — le serveur libère tout ce qu'un client laisse.
struct Lecteur {
    conn: RustConnection,
    fenetre: Window,
    atomes: Atomes,
}

impl Lecteur {
    fn ouvrir(ecran: Option<&str>) -> Option<Self> {
        let (conn, numero) = RustConnection::connect(ecran).ok()?;
        let atomes = Atomes::new(&conn).ok()?.reply().ok()?;
        let racine = conn.setup().roots.get(numero)?.root;
        let fenetre = conn.generate_id().ok()?;
        let attributs = CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE);
        conn.create_window(
            0,
            fenetre,
            racine,
            0,
            0,
            1,
            1,
            0,
            WindowClass::INPUT_ONLY,
            COPY_FROM_PARENT,
            &attributs,
        )
        .ok()?;
        Some(Self {
            conn,
            fenetre,
            atomes,
        })
    }

    /// Demande le lot, puis le reçoit — d'un bloc, ou tranche après tranche.
    fn lire(&self, proprio: Window) -> Option<Vec<u8>> {
        let a = &self.atomes;
        let suivre = ChangeWindowAttributesAux::new().event_mask(EventMask::STRUCTURE_NOTIFY);
        self.conn.change_window_attributes(proprio, &suivre).ok()?;
        (self.conn)
            .convert_selection(self.fenetre, a.CLIPBOARD, a.LOT, a.RECU, CURRENT_TIME)
            .ok()?;
        self.conn.flush().ok()?;
        let mut recu = Vec::new();
        let mut par_tranches = false;
        let mut echeance = Instant::now() + PATIENCE;
        loop {
            match self.attendre(echeance)? {
                Event::SelectionNotify(e) if e.requestor == self.fenetre && !par_tranches => {
                    if e.property == NONE {
                        return None;
                    }
                    // Lire efface : c'est ainsi qu'on dit au propriétaire d'envoyer la suite.
                    let (genre, octets) = self.prendre()?;
                    if genre != a.INCR {
                        return Some(octets);
                    }
                    par_tranches = true;
                }
                Event::PropertyNotify(e)
                    if par_tranches
                        && e.window == self.fenetre
                        && e.atom == a.RECU
                        && e.state == Property::NEW_VALUE =>
                {
                    let (_, tranche) = self.prendre()?;
                    if tranche.is_empty() {
                        return Some(recu);
                    }
                    recu.extend_from_slice(&tranche);
                    echeance = Instant::now() + PATIENCE;
                }
                Event::DestroyNotify(e) if e.window == proprio => return None,
                _ => {}
            }
        }
    }

    /// La propriété reçue, effacée en la lisant : son type, et ses octets.
    fn prendre(&self) -> Option<(u32, Vec<u8>)> {
        let r = (self.conn)
            .get_property(
                true,
                self.fenetre,
                self.atomes.RECU,
                AtomEnum::ANY,
                0,
                u32::MAX / 4,
            )
            .ok()?
            .reply()
            .ok()?;
        self.conn.flush().ok()?;
        Some((r.type_, r.value))
    }

    /// **Le prochain évènement, au plus tard à l'échéance** : ceux déjà lus d'abord, puis on
    /// attend que la connexion ait à lire (`poll`) — sans tourner à vide.
    fn attendre(&self, echeance: Instant) -> Option<Event> {
        loop {
            if let Some(evenement) = self.conn.poll_for_event().ok()? {
                return Some(evenement);
            }
            let reste = echeance.checked_duration_since(Instant::now())?;
            let mut veille = libc::pollfd {
                fd: self.conn.stream().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let ms = i32::try_from(reste.as_millis()).unwrap_or(i32::MAX).max(1);
            // Sûr : un seul descripteur, celui de notre connexion, vivant le temps de l'appel.
            if unsafe { libc::poll(&mut veille, 1, ms) } < 0
                && std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted
            {
                return None;
            }
        }
    }
}
