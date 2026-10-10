//! **Le fil qui sert la sélection** (ICCCM, chapitre 2) : il prend la propriété à l'heure du
//! serveur, répond à chaque demande — `TARGETS`, `TIMESTAMP`, `MULTIPLE`, chaque forme —,
//! envoie par tranches (`INCR`) ce qu'une requête ne porte pas, et la remet au gestionnaire du
//! presse-papiers à la fermeture.
//!
//! # Par tranches, et jusqu'où
//!
//! Une propriété s'écrit en une requête, et le serveur borne la taille d'une requête (avec
//! BIG-REQUESTS, seize mégaoctets chez Xorg). Au-delà, la propriété dit `INCR` et la taille ; le
//! demandeur l'efface pour dire qu'il est prêt, puis efface chaque tranche lue, et une tranche
//! vide clôt l'envoi (ICCCM 2.7.2). **La tranche est la plus grande requête que ce serveur
//! accepte**, moins l'en-tête : aucun autre seuil. Chromium s'arrête au mégaoctet et abandonne un
//! envoi muet depuis dix secondes ; ici, un envoi finit quand son demandeur disparaît (sa
//! fenêtre détruite, `DestroyNotify`) — l'évènement, pas une horloge.

use super::offre::{Offre, Rendu};
use super::{Atomes, PATIENCE};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use x11rb::connection::{Connection, RequestConnection as _};
use x11rb::protocol::xproto::{
    Atom, AtomEnum, ChangeWindowAttributesAux, ConnectionExt as _, CreateWindowAux, EventMask,
    PropMode, Property, SelectionClearEvent, SelectionNotifyEvent, SelectionRequestEvent,
    Timestamp, Window, WindowClass, SELECTION_NOTIFY_EVENT,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;
use x11rb::{COPY_FROM_PARENT, CURRENT_TIME, NONE};

/// L'en-tête d'un `ChangeProperty` : vingt-quatre octets, et quatre de plus pour la longueur
/// étendue de BIG-REQUESTS — le protocole X, pas un choix.
const EN_TETE: usize = 28;

/// La connexion du propriétaire, sa fenêtre, et ce qu'il sert.
pub(super) struct Serveur {
    /// L'écran X, s'il n'est pas celui de `DISPLAY` — les épreuves ont le leur.
    pub ecran: Option<String>,
    pub conn: RustConnection,
    pub fenetre: Window,
    pub atomes: Atomes,
    /// La plus grande tranche qu'une requête porte : au-delà, `INCR`.
    pub tranche: usize,
    etat: Mutex<Etat>,
}

#[derive(Default)]
struct Etat {
    /// Le fil qui sert s'est arrêté : plus rien ne se prend.
    arrete: bool,
    /// L'offre qui attend d'être prise.
    a_prendre: Option<APrendre>,
    /// L'offre servie, et l'heure du serveur où elle a été prise.
    servie: Option<(Arc<Offre>, Timestamp)>,
    /// Les envois par tranches en cours.
    envois: Vec<Envoi>,
    /// La remise au gestionnaire en cours : à qui dire qu'elle est finie.
    remise: Option<Sender<()>>,
}

/// Une offre qui attend d'être prise, et à qui dire si elle l'a été.
struct APrendre {
    offre: Arc<Offre>,
    reponse: Sender<Result<(), String>>,
}

/// Un envoi par tranches : à quelle fenêtre, sous quelle propriété, quoi, et jusqu'où.
struct Envoi {
    fenetre: Window,
    propriete: Atom,
    cible: Atom,
    offre: Arc<Offre>,
    rendu: Rendu,
    envoye: usize,
}

fn texte(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// **Une demande faite à cette heure vise-t-elle la sélection prise à celle-là ?** `CurrentTime`
/// toujours ; sinon, pas une heure d'avant la prise (ICCCM 2.2) — l'horloge du serveur compte
/// en millisecondes sur 32 bits, et boucle tous les 49 jours.
pub(super) fn pas_avant(demande: Timestamp, prise: Timestamp) -> bool {
    demande == CURRENT_TIME || demande.wrapping_sub(prise) < 1 << 31
}

impl Serveur {
    /// Une connexion à soi, et la fenêtre de sélection — jamais montrée, marquée pour qu'un
    /// autre Glucose la reconnaisse.
    pub fn ouvrir(ecran: Option<&str>) -> Result<Self, String> {
        let (conn, numero) =
            RustConnection::connect(ecran).map_err(|e| format!("aucun serveur X : {e}"))?;
        let atomes = Atomes::new(&conn).map_err(texte)?.reply().map_err(texte)?;
        let racine = conn.setup().roots.get(numero).ok_or("aucun écran X")?.root;
        let fenetre = conn.generate_id().map_err(texte)?;
        let attributs = CreateWindowAux::new()
            .event_mask(EventMask::PROPERTY_CHANGE | EventMask::STRUCTURE_NOTIFY);
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
        .map_err(texte)?;
        conn.change_property32(
            PropMode::REPLACE,
            fenetre,
            atomes.MARQUE,
            AtomEnum::CARDINAL,
            &[1],
        )
        .map_err(texte)?;
        conn.flush().map_err(texte)?;
        let tranche = conn.maximum_request_bytes().saturating_sub(EN_TETE) & !3;
        Ok(Self {
            ecran: ecran.map(str::to_string),
            conn,
            fenetre,
            atomes,
            tranche,
            etat: Mutex::default(),
        })
    }

    fn etat(&self) -> MutexGuard<'_, Etat> {
        self.etat.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// **Le fil qui sert** : chaque évènement, jusqu'à la destruction de la fenêtre — ou la
    /// connexion perdue. À l'arrêt, qui attendait le sait.
    pub fn servir(&self) {
        while let Ok(evenement) = self.conn.wait_for_event() {
            match evenement {
                Event::DestroyNotify(e) if e.window == self.fenetre => break,
                Event::DestroyNotify(e) => self.etat().envois.retain(|t| t.fenetre != e.window),
                Event::PropertyNotify(e) if e.window == self.fenetre => {
                    if e.atom == self.atomes.PRENDRE && e.state == Property::NEW_VALUE {
                        self.prendre(e.time);
                    }
                }
                Event::PropertyNotify(e) if e.state == Property::DELETE => {
                    self.poursuivre(e.window, e.atom);
                }
                Event::SelectionRequest(e) => self.repondre(e),
                Event::SelectionClear(e) => self.perdue(e),
                _ => {}
            }
            let _ = self.conn.flush();
        }
        let mut etat = self.etat();
        etat.arrete = true;
        etat.a_prendre = None;
        etat.remise = None;
    }

    /// Range l'offre qui attend d'être prise, et à qui répondre.
    pub fn attendre_la_prise(
        &self,
        offre: Arc<Offre>,
        reponse: Sender<Result<(), String>>,
    ) -> Result<(), String> {
        let mut etat = self.etat();
        if etat.arrete {
            return Err("le fil du presse-papiers s'est arrêté".into());
        }
        etat.a_prendre = Some(APrendre { offre, reponse });
        Ok(())
    }

    /// **Prend la sélection à l'heure du serveur**, et dit si le serveur l'a donnée.
    fn prendre(&self, temps: Timestamp) {
        let Some(APrendre { offre, reponse }) = self.etat().a_prendre.take() else {
            return;
        };
        let prise = self.devenir_proprietaire(temps);
        if prise.is_ok() {
            self.etat().servie = Some((offre, temps));
        }
        let _ = reponse.send(prise);
    }

    fn devenir_proprietaire(&self, temps: Timestamp) -> Result<(), String> {
        let clip = self.atomes.CLIPBOARD;
        self.conn
            .set_selection_owner(self.fenetre, clip, temps)
            .map_err(texte)?;
        let proprio = self
            .conn
            .get_selection_owner(clip)
            .map_err(texte)?
            .reply()
            .map_err(texte)?
            .owner;
        if proprio == self.fenetre {
            Ok(())
        } else {
            Err("un autre programme a pris le presse-papiers au même instant".into())
        }
    }

    /// Le lot servi, s'il en est un.
    pub fn lot_servi(&self) -> Option<Vec<u8>> {
        self.etat().servie.as_ref().map(|(o, _)| o.lot().to_vec())
    }

    /// **Une demande** : convertie si la sélection est la nôtre et la demande pas d'avant sa
    /// prise, refusée sinon — toujours une réponse.
    fn repondre(&self, e: SelectionRequestEvent) {
        // Un demandeur d'avant ICCCM 2 ne nomme pas de propriété : la cible en sert.
        let propriete = if e.property == NONE {
            e.target
        } else {
            e.property
        };
        let servie = self.etat().servie.clone();
        let acceptee = match servie {
            Some((offre, depuis))
                if e.selection == self.atomes.CLIPBOARD && pas_avant(e.time, depuis) =>
            {
                if e.target == self.atomes.MULTIPLE {
                    self.convertir_plusieurs(&offre, depuis, e.requestor, propriete)
                } else {
                    self.convertir(&offre, depuis, e.target, (e.requestor, propriete))
                }
            }
            _ => false,
        };
        let reponse = SelectionNotifyEvent {
            response_type: SELECTION_NOTIFY_EVENT,
            sequence: 0,
            time: e.time,
            requestor: e.requestor,
            selection: e.selection,
            target: e.target,
            property: if acceptee { propriete } else { NONE },
        };
        let _ = self
            .conn
            .send_event(false, e.requestor, EventMask::NO_EVENT, reponse);
    }

    /// **Une cible, écrite dans la propriété du demandeur.**
    fn convertir(
        &self,
        offre: &Arc<Offre>,
        depuis: Timestamp,
        cible: Atom,
        (demandeur, propriete): (Window, Atom),
    ) -> bool {
        let a = &self.atomes;
        if cible == a.TARGETS {
            let cibles: Vec<Atom> = [a.TARGETS, a.TIMESTAMP, a.MULTIPLE, a.SAVE_TARGETS]
                .into_iter()
                .chain(offre.cibles())
                .collect();
            let atome = AtomEnum::ATOM;
            return (self.conn)
                .change_property32(PropMode::REPLACE, demandeur, propriete, atome, &cibles)
                .is_ok();
        }
        if cible == a.TIMESTAMP {
            let entier = AtomEnum::INTEGER;
            return (self.conn)
                .change_property32(PropMode::REPLACE, demandeur, propriete, entier, &[depuis])
                .is_ok();
        }
        offre
            .rendu(cible)
            .is_some_and(|rendu| self.envoyer(offre, rendu, cible, (demandeur, propriete)))
    }

    /// **`MULTIPLE`** : la propriété du demandeur porte des paires (cible, propriété) ; chacune se
    /// convertit, et celles qui ne le peuvent pas voient leur propriété remplacée par `None`.
    fn convertir_plusieurs(
        &self,
        offre: &Arc<Offre>,
        depuis: Timestamp,
        demandeur: Window,
        propriete: Atom,
    ) -> bool {
        let a = &self.atomes;
        let Some(paires) = self
            .conn
            .get_property(false, demandeur, propriete, AtomEnum::ANY, 0, u32::MAX / 4)
            .ok()
            .and_then(|c| c.reply().ok())
            .and_then(|r| r.value32().map(Iterator::collect::<Vec<Atom>>))
            .filter(|p| p.len() % 2 == 0)
        else {
            return false;
        };
        let resultats: Vec<Atom> = paires
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|&[cible, prop]| {
                let faite = cible != a.MULTIPLE
                    && prop != NONE
                    && self.convertir(offre, depuis, cible, (demandeur, prop));
                [cible, if faite { prop } else { NONE }]
            })
            .collect();
        (self.conn)
            .change_property32(
                PropMode::REPLACE,
                demandeur,
                propriete,
                a.ATOM_PAIR,
                &resultats,
            )
            .is_ok()
    }

    /// **Les octets d'une forme, d'un bloc ou par tranches.**
    fn envoyer(
        &self,
        offre: &Arc<Offre>,
        rendu: Rendu,
        cible: Atom,
        (demandeur, propriete): (Window, Atom),
    ) -> bool {
        let Ok(octets) = offre.octets(rendu) else {
            return false;
        };
        if octets.len() <= self.tranche {
            return (self.conn)
                .change_property8(PropMode::REPLACE, demandeur, propriete, cible, &octets)
                .is_ok();
        }
        // On suit la fenêtre du demandeur avant de lui dire `INCR` : ses effacements, et sa fin.
        let suivre = ChangeWindowAttributesAux::new()
            .event_mask(EventMask::PROPERTY_CHANGE | EventMask::STRUCTURE_NOTIFY);
        let taille = u32::try_from(octets.len()).unwrap_or(u32::MAX);
        let lance = self
            .conn
            .change_window_attributes(demandeur, &suivre)
            .is_ok()
            && (self.conn)
                .change_property32(
                    PropMode::REPLACE,
                    demandeur,
                    propriete,
                    self.atomes.INCR,
                    &[taille],
                )
                .is_ok();
        if lance {
            self.etat().envois.push(Envoi {
                fenetre: demandeur,
                propriete,
                cible,
                offre: Arc::clone(offre),
                rendu,
                envoye: 0,
            });
        }
        lance
    }

    /// **Le demandeur a effacé une tranche** : la suivante, ou la tranche vide qui clôt l'envoi.
    fn poursuivre(&self, fenetre: Window, propriete: Atom) {
        let quel = |t: &Envoi| t.fenetre == fenetre && t.propriete == propriete;
        let Some((offre, rendu, cible, debut)) = self
            .etat()
            .envois
            .iter()
            .find(|t| quel(t))
            .map(|t| (Arc::clone(&t.offre), t.rendu, t.cible, t.envoye))
        else {
            return;
        };
        let octets = offre.octets(rendu).unwrap_or_default();
        let debut = debut.min(octets.len());
        let fin = (debut + self.tranche).min(octets.len());
        let _ = (self.conn).change_property8(
            PropMode::REPLACE,
            fenetre,
            propriete,
            cible,
            &octets[debut..fin],
        );
        let mut etat = self.etat();
        if fin > debut {
            if let Some(t) = etat.envois.iter_mut().find(|t| quel(t)) {
                t.envoye = fin;
            }
            return;
        }
        etat.envois.retain(|t| !quel(t));
        if !etat.envois.iter().any(|t| t.fenetre == fenetre) {
            let rien = ChangeWindowAttributesAux::new().event_mask(EventMask::NO_EVENT);
            let _ = self.conn.change_window_attributes(fenetre, &rien);
        }
    }

    /// **Un autre programme a pris la sélection** — pendant une remise, c'est le gestionnaire :
    /// il l'a gardée, et la remise est finie.
    ///
    /// C'est **le seul** signe de fin. La spécification fait prendre la sélection au gestionnaire
    /// une fois les formes reçues, puis répondre à `SAVE_TARGETS` ; KDE, lui, répond sans
    /// propriété **avant** de demander les formes (`arboard` l'a relevé) — sa réponse ne dit
    /// donc rien, et la sélection qu'il prend dit tout, chez l'un comme chez l'autre.
    fn perdue(&self, e: SelectionClearEvent) {
        if e.selection != self.atomes.CLIPBOARD {
            return;
        }
        let mut etat = self.etat();
        etat.servie = None;
        if let Some(finie) = etat.remise.take() {
            let _ = finie.send(());
        }
    }

    /// **À la fermeture : la sélection va au gestionnaire du presse-papiers** (la spécification
    /// de freedesktop), avec la liste des cibles à garder — s'il y en a un, et si la sélection est
    /// encore la nôtre. On attend qu'il l'ait prise ([`Self::perdue`]), ou qu'il ne réponde plus.
    pub fn remettre_au_gestionnaire(&self) {
        let a = &self.atomes;
        let Some((offre, _)) = self.etat().servie.clone() else {
            return;
        };
        let proprio = |sel| Some(self.conn.get_selection_owner(sel).ok()?.reply().ok()?.owner);
        if proprio(a.CLIPBOARD) != Some(self.fenetre)
            || proprio(a.CLIPBOARD_MANAGER).is_none_or(|g| g == NONE)
        {
            return;
        }
        let cibles: Vec<Atom> = offre.cibles().collect();
        let (finie, attendre) = mpsc::channel();
        self.etat().remise = Some(finie);
        let atome = AtomEnum::ATOM;
        let demandee = (self.conn)
            .change_property32(PropMode::REPLACE, self.fenetre, a.GARDER, atome, &cibles)
            .is_ok()
            && (self.conn)
                .convert_selection(
                    self.fenetre,
                    a.CLIPBOARD_MANAGER,
                    a.SAVE_TARGETS,
                    a.GARDER,
                    CURRENT_TIME,
                )
                .is_ok()
            && self.conn.flush().is_ok();
        if demandee {
            let _ = attendre.recv_timeout(PATIENCE);
        }
        self.etat().remise = None;
    }
}
