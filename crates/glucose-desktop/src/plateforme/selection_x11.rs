//! **COPIER-1 sous Linux — la sélection sous toutes ses formes, servie par Glucose lui-même**
//! (fiche 61). Ce que Windows a depuis la fiche 59 (`selection_windows`) : un seul « Copier »
//! qui pose à la fois le lot, le texte, et l'image seule ou les fichiers des images.
//!
//! # Pourquoi Glucose sert lui-même
//!
//! Sous X11, il n'y a pas de presse-papiers où l'on dépose : il y a un **propriétaire** de la
//! sélection `CLIPBOARD`, à qui chaque application qui colle demande la forme qu'elle veut
//! (ICCCM, chapitre 2). `arboard` sait être ce propriétaire, mais ne pose qu'**une forme à la
//! fois** par son interface, et **n'écrit jamais par `INCR`** : une image plus grosse qu'une
//! requête du serveur ne partirait pas. Glucose tient donc sa propre fenêtre de sélection, sur sa
//! propre connexion, servie par un fil à lui — la forme de Chromium (`selection_owner.cc`), de
//! GTK et de Qt.
//!
//! # Ce qui est servi, et comment
//!
//! * les cibles de [`offre`] : le lot, le texte, l'image seule **ou** les fichiers — la règle
//!   de [`crate::interactions::clipboard::Formes::offertes`], la même que sous Windows ;
//! * `TARGETS`, `TIMESTAMP`, `MULTIPLE` (que le gestionnaire de GNOME emploie pour tout prendre
//!   d'un coup), et `INCR` dès qu'une forme dépasse la plus grande requête que le serveur
//!   accepte — sans autre seuil que celui-là ([`service`]) ;
//! * **l'instant du serveur** : la propriété se prend à l'heure d'un évènement du serveur, jamais
//!   à `CurrentTime` (ICCCM 2.1) — une écriture vide sur notre fenêtre rend cette heure, et
//!   réveille le fil qui sert du même coup ;
//! * **à la fermeture**, la sélection est confiée au gestionnaire du presse-papiers s'il y en a
//!   un (`SAVE_TARGETS`, la spécification de freedesktop) : un collage après coup la trouve
//!   encore — ce que `OleFlushClipboard` fait sous Windows.
//!
//! # Coller un lot d'un autre Glucose
//!
//! La fenêtre de sélection porte une marque (`_GLUCOSE_SELECTION`). Un `Ctrl+V` demande au seul
//! **serveur** qui tient la sélection, et si la fenêtre porte la marque : s'il s'agit de la
//! nôtre, le lot se lit sans rien demander à personne ; d'un autre Glucose, il se lit hors du fil
//! qui dessine ([`lecture`]) ; d'une autre application, Glucose ne lui demande rien — le collage
//! ordinaire prend la suite.
//!
//! # Sous Wayland
//!
//! Le copier passe par XWayland, comme `arboard`. GNOME relaie le presse-papiers entre X11 et
//! Wayland dans les deux sens depuis 2015 (Mutter 3.17.2), KDE sans exiger le focus depuis
//! Plasma 6.0 (KWin, MR 5510 ; une panne en 6.5.0 et 6.5.1, corrigée en 6.5.2). Un bureau
//! Wayland sans XWayland n'a pas encore de copier — ni d'`arboard`.

mod lecture;
mod offre;
mod service;
#[cfg(test)]
mod tests;

use crate::interactions::clipboard::Formes;
use offre::Offre;
use service::Serveur;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, PropMode};
use x11rb::wrapper::ConnectionExt as _;

x11rb::atom_manager! {
    /// Les atomes que la sélection emploie, demandés une fois à la connexion.
    pub Atomes: AtomesCookies {
        CLIPBOARD,
        CLIPBOARD_MANAGER,
        TARGETS,
        TIMESTAMP,
        MULTIPLE,
        ATOM_PAIR,
        INCR,
        SAVE_TARGETS,
        UTF8_STRING,
        TEXTE_UTF8: b"text/plain;charset=utf-8",
        TEXTE: b"text/plain",
        PNG: b"image/png",
        LISTE_D_URI: b"text/uri-list",
        FICHIERS_GNOME: b"x-special/gnome-copied-files",
        LOT: b"application/x-glucose-lot",
        // La marque de la fenêtre de sélection d'un Glucose.
        MARQUE: b"_GLUCOSE_SELECTION",
        // L'écriture vide qui rend l'heure du serveur et réveille le fil qui sert.
        PRENDRE: b"_GLUCOSE_PRENDRE",
        // Les cibles à garder, confiées au gestionnaire à la fermeture.
        GARDER: b"_GLUCOSE_GARDER",
        // Où un lecteur reçoit ce qu'il demande.
        RECU: b"_GLUCOSE_RECU",
    }
}

/// **Combien de temps on attend un autre programme qui ne répond plus** : le gestionnaire du
/// presse-papiers à la fermeture, un autre Glucose qui envoie son lot. Chaque attente finit
/// d'abord sur l'évènement qui la clôt ; celle-ci ne garde que d'un programme figé. Cinq
/// secondes : la patience de Qt (`QXcbClipboard::clipboardTimeout`).
const PATIENCE: Duration = Duration::from_secs(5);

/// **Le propriétaire de la sélection** : sa connexion, sa fenêtre, et le fil qui les sert.
pub(super) struct Proprietaire {
    serveur: Arc<Serveur>,
    fil: Option<JoinHandle<()>>,
}

impl Proprietaire {
    /// Ouvre une connexion au serveur X — celui de `DISPLAY`, ou celui qu'on nomme —, crée la
    /// fenêtre marquée, et lance le fil qui sert.
    pub fn ouvrir(ecran: Option<&str>) -> Result<Self, String> {
        let serveur = Arc::new(Serveur::ouvrir(ecran)?);
        let pour_le_fil = Arc::clone(&serveur);
        let fil = std::thread::Builder::new()
            .name("glucose-selection".into())
            .spawn(move || pour_le_fil.servir())
            .map_err(|e| e.to_string())?;
        Ok(Self {
            serveur,
            fil: Some(fil),
        })
    }

    /// Le fil qui sert est-il encore là ? Il s'arrête si la connexion tombe.
    fn vivant(&self) -> bool {
        self.fil.as_ref().is_some_and(|f| !f.is_finished())
    }

    /// **Confie la sélection** : le fil qui sert la prend à l'heure du serveur, et dit si le
    /// serveur la lui a donnée.
    pub fn confier(&self, formes: Formes) -> Result<(), String> {
        let s = &self.serveur;
        let offre = Arc::new(Offre::nouvelle(formes, &s.atomes));
        let (reponse, recue) = mpsc::channel();
        s.attendre_la_prise(offre, reponse)?;
        s.conn
            .change_property8(
                PropMode::APPEND,
                s.fenetre,
                s.atomes.PRENDRE,
                AtomEnum::STRING,
                &[],
            )
            .map_err(|e| e.to_string())?;
        s.conn.flush().map_err(|e| e.to_string())?;
        recue
            .recv()
            .map_err(|_| "le fil du presse-papiers s'est arrêté".to_string())?
    }

    /// **Le lot de la sélection, s'il en est un** : le nôtre se lit ici ; celui d'un autre
    /// Glucose, plus tard, hors du fil qui dessine ; une autre application n'est jamais
    /// sollicitée.
    pub fn lot(&self) -> Option<super::presse_papiers::LotALire> {
        let s = &self.serveur;
        let proprio = s
            .conn
            .get_selection_owner(s.atomes.CLIPBOARD)
            .ok()?
            .reply()
            .ok()?
            .owner;
        if proprio == x11rb::NONE {
            return None;
        }
        if proprio == s.fenetre {
            let lot = s.lot_servi()?;
            return Some(Box::new(move || Some(lot)));
        }
        let marquee = s
            .conn
            .get_property(false, proprio, s.atomes.MARQUE, AtomEnum::ANY, 0, 1)
            .ok()?
            .reply()
            .ok()?
            .type_
            != u32::from(AtomEnum::NONE);
        let ecran = s.ecran.clone();
        marquee.then(|| -> super::presse_papiers::LotALire {
            Box::new(move || lecture::lire_le_lot_de(ecran.as_deref(), proprio))
        })
    }

    /// **À la fermeture** : la sélection, si elle est encore la nôtre, va au gestionnaire du
    /// presse-papiers ; puis la fenêtre se détruit, et le fil qui sert s'arrête.
    pub fn rendre(mut self) {
        self.serveur.remettre_au_gestionnaire();
        self.fermer();
    }

    /// Détruit la fenêtre — le fil qui sert le voit, et s'arrête. Une fois.
    fn fermer(&mut self) {
        let Some(fil) = self.fil.take() else {
            return;
        };
        let s = &self.serveur;
        let _ = s.conn.destroy_window(s.fenetre);
        let _ = s.conn.flush();
        let _ = fil.join();
    }

    /// La fenêtre de sélection, pour les épreuves.
    #[cfg(test)]
    fn fenetre(&self) -> x11rb::protocol::xproto::Window {
        self.serveur.fenetre
    }
}

impl Drop for Proprietaire {
    fn drop(&mut self) {
        self.fermer();
    }
}

/// Le propriétaire de l'application, ouvert à la première copie.
static PROPRIETAIRE: Mutex<Option<Proprietaire>> = Mutex::new(None);

/// Le propriétaire de l'application — rouvert si sa connexion est tombée.
fn avec_le_proprietaire<T>(f: impl FnOnce(&Proprietaire) -> T) -> Result<T, String> {
    let mut garde = PROPRIETAIRE
        .lock()
        .map_err(|_| "le presse-papiers est empoisonné".to_string())?;
    if !garde.as_ref().is_some_and(Proprietaire::vivant) {
        *garde = Some(Proprietaire::ouvrir(None)?);
    }
    Ok(f(garde.as_ref().expect("ouvert juste au-dessus")))
}

/// **Confie la sélection au presse-papiers du système**, sous toutes ses formes.
pub fn confier_au_presse_papiers(formes: Formes) -> Result<(), String> {
    avec_le_proprietaire(|p| p.confier(formes))?
}

/// Le lot que porte le presse-papiers, s'il en porte un.
pub fn lire_un_lot() -> Option<super::presse_papiers::LotALire> {
    avec_le_proprietaire(Proprietaire::lot).ok().flatten()
}

/// **À la fermeture** : la sélection va au gestionnaire du presse-papiers, s'il y en a un.
pub fn rendre_avant_de_quitter() {
    let proprietaire = PROPRIETAIRE.lock().ok().and_then(|mut p| p.take());
    if let Some(p) = proprietaire {
        p.rendre();
    }
}
