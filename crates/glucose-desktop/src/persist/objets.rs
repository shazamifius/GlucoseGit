//! **Les objets du document** : où sont les octets de chaque image.
//!
//! # Pourquoi un registre, et pas un nouveau nom d'image
//!
//! Tout le rendu — l'atelier qui décode, le magasin des pyramides, les textures de la carte,
//! les aperçus sur le disque — connaît une image par une **clé** : son `src`. Changer cette clé
//! au moment où l'image entre dans le document obligerait à renommer une entrée dans chacun
//! de ces caches, à l'instant précis où ils la tiennent. Ici, la clé ne change jamais ; c'est
//! **sa source** qui change :
//!
//! * d'abord le fichier d'où elle vient, pour qu'elle s'affiche tout de suite ;
//! * puis, dès que le fil d'écriture l'a **scellée** — lue, hachée, ajoutée au document si ses
//!   octets n'y étaient pas —, la tranche du `.glucose` qui porte ses octets.
//!
//! Une image scellée ne dépend plus d'aucun fichier extérieur : le dossier temporaire de
//! Windows peut faire son ménage (fiche 33 § 5.1).
//!
//! # L'intégrité se vérifie à la lecture
//!
//! Ouvrir un document ne hache pas ses images (fiche 32, 180 Mo). Chaque tranche annonce
//! l'empreinte de ses octets ; [`Objets::lire`] la vérifie au moment où l'atelier en a besoin.
//! Une tranche abîmée rend `None` — l'image se dessine comme introuvable, jamais avec les
//! octets d'une autre.

use glucose_core::hash::sha256;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, RwLock};

/// D'où lire les octets d'une image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// Un fichier extérieur : l'image n'est pas encore scellée dans le document.
    Fichier(PathBuf),
    /// Des octets que le document portait lui-même hors d'un fichier — une image en base64
    /// d'un vieux document Tauri —, le temps que le scribe les scelle.
    Memoire(Arc<Vec<u8>>),
    /// Une tranche du document, et l'empreinte que ses octets doivent avoir.
    Tranche {
        empreinte: [u8; 32],
        offset: u64,
        longueur: u64,
    },
    /// Une tranche d'**un autre** document : une image qu'un import vient d'apporter
    /// (BOARDS-2), le temps que le scribe la copie dans celui-ci.
    Ailleurs {
        fichier: PathBuf,
        empreinte: [u8; 32],
        offset: u64,
        longueur: u64,
    },
    /// Des octets **promis** : une image collée, que l'atelier encode (COLLER-3).
    Promise(Arc<Promesse>),
}

/// **Des octets promis** — ceux d'une image collée, que l'atelier est en train d'encoder
/// (COLLER-3).
///
/// # Pourquoi plus de fichier
///
/// Une image collée n'a pas de fichier : on lui en écrivait un dans le dossier temporaire du
/// système, que le scribe relisait pour la sceller. Deux fils se passaient ainsi un fichier, et
/// le 28/09 le scribe l'a lu au moment où il venait d'être créé, vide : trois images scellées
/// vides (COLLER-2). Et ces fichiers restaient pour toujours — 1 638 chez lui, 2,9 Go — dans un
/// dossier que Windows peut vider.
///
/// Les octets passent désormais de l'atelier au scribe **sans disque**, par cette promesse. Le
/// scribe l'attend à sa place dans sa file, qui est aussi celle des gestes : l'image entre dans
/// l'histoire **avant** le geste qui la pose, jamais sans ses octets. Un arrêt pendant
/// l'encodage ne laisse ni l'image ni ce qui la suit — un document cohérent, et l'image encore
/// dans le presse-papiers.
pub struct Promesse {
    etat: Mutex<Etat>,
    tenue: Condvar,
}

enum Etat {
    Attendue,
    Tenue(Arc<Vec<u8>>),
    /// Celui qui avait promis a disparu sans tenir : on n'attend pas pour rien.
    Abandonnee,
}

/// Ce que tient celui qui a promis : la tenir — ou, s'il disparaît sans l'avoir tenue,
/// l'abandonner, pour que personne ne l'attende sans fin.
pub struct Parole(Arc<Promesse>);

impl Promesse {
    /// Une promesse, et la parole de celui qui la tiendra.
    pub fn nouvelle() -> (Arc<Self>, Parole) {
        let p = Arc::new(Self {
            etat: Mutex::new(Etat::Attendue),
            tenue: Condvar::new(),
        });
        (Arc::clone(&p), Parole(p))
    }

    /// Les octets s'ils sont déjà là, sans attendre.
    pub fn deja(&self) -> Option<Arc<Vec<u8>>> {
        match &*self.etat.lock().ok()? {
            Etat::Tenue(o) => Some(Arc::clone(o)),
            Etat::Attendue | Etat::Abandonnee => None,
        }
    }

    /// Attend les octets. `None` si la promesse a été abandonnée.
    pub fn attendre(&self) -> Option<Arc<Vec<u8>>> {
        let mut etat = self.etat.lock().ok()?;
        loop {
            match &*etat {
                Etat::Tenue(o) => return Some(Arc::clone(o)),
                Etat::Abandonnee => return None,
                Etat::Attendue => etat = self.tenue.wait(etat).ok()?,
            }
        }
    }

    fn poser(&self, nouveau: Etat) {
        if let Ok(mut etat) = self.etat.lock() {
            if matches!(*etat, Etat::Attendue) {
                *etat = nouveau;
            }
        }
        self.tenue.notify_all();
    }
}

impl Parole {
    /// Tient la promesse : ces octets sont ceux de l'image.
    pub fn tenir(self, octets: Vec<u8>) {
        self.0.poser(Etat::Tenue(Arc::new(octets)));
    }
}

impl Drop for Parole {
    fn drop(&mut self) {
        // Tenue, elle ne change plus ; sinon, celui qui attend l'apprend maintenant.
        self.0.poser(Etat::Abandonnee);
    }
}

impl std::fmt::Debug for Promesse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Promesse")
    }
}

/// Deux promesses sont la même si c'est la même : leurs octets ne se comparent pas.
impl PartialEq for Promesse {
    fn eq(&self, autre: &Self) -> bool {
        std::ptr::eq(self, autre)
    }
}

impl Eq for Promesse {}

/// Le registre, partagé entre le fil qui dessine, les ouvriers de l'atelier et le fil
/// d'écriture.
#[derive(Debug, Default)]
pub struct Objets {
    table: RwLock<HashMap<String, Source>>,
    /// Le fichier qui porte les tranches : le document ouvert, ou son brouillon.
    document: RwLock<Option<PathBuf>>,
}

impl Objets {
    pub fn nouveau() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Oublie tout : un autre document s'ouvre.
    pub fn vider(&self) {
        if let Ok(mut t) = self.table.write() {
            t.clear();
        }
        self.porter(None);
    }

    /// Le fichier dont les tranches sont lues désormais. Un « Enregistrer sous » copie le
    /// fichier octet pour octet : les tranches restent justes, seul le chemin change.
    pub fn porter(&self, document: Option<PathBuf>) {
        if let Ok(mut d) = self.document.write() {
            *d = document;
        }
    }

    pub fn document(&self) -> Option<PathBuf> {
        self.document.read().ok().and_then(|d| d.clone())
    }

    pub fn poser(&self, cle: &str, source: Source) {
        if let Ok(mut t) = self.table.write() {
            t.insert(cle.to_string(), source);
        }
    }

    pub fn source(&self, cle: &str) -> Option<Source> {
        self.table.read().ok()?.get(cle).cloned()
    }

    /// Vrai si les octets de cette clé sont dans le document.
    pub fn est_scellee(&self, cle: &str) -> bool {
        matches!(self.source(cle), Some(Source::Tranche { .. }))
    }

    /// **L'empreinte des octets de cette clé**, si un document les porte — celui-ci, ou celui
    /// d'où un ajout les apporte. C'est ce qui nomme son aperçu (APERCU-5) : une identité qui
    /// ne change jamais, et qu'aucun fichier extérieur ne porte.
    pub fn empreinte(&self, cle: &str) -> Option<[u8; 32]> {
        match self.source(cle)? {
            Source::Tranche { empreinte, .. } | Source::Ailleurs { empreinte, .. } => {
                Some(empreinte)
            }
            _ => None,
        }
    }

    /// Les octets d'une image, d'où qu'ils viennent. Une clé inconnue se lit comme un chemin :
    /// c'est ce qu'était toute clé avant ce registre.
    pub fn lire(&self, cle: &str) -> Option<Vec<u8>> {
        match self.source(cle) {
            Some(Source::Tranche {
                empreinte,
                offset,
                longueur,
            }) => self.lire_la_tranche(&empreinte, offset, longueur),
            Some(Source::Ailleurs {
                fichier,
                empreinte,
                offset,
                longueur,
            }) => lire_une_tranche(&fichier, &empreinte, offset, longueur),
            Some(Source::Fichier(chemin)) => std::fs::read(chemin).ok(),
            Some(Source::Memoire(octets)) => Some(octets.as_ref().clone()),
            // Jamais d'attente ici : l'atelier, qui lit, est aussi celui qui tient la promesse.
            // Avant qu'elle soit tenue, personne ne redemande l'image — elle est en chantier.
            Some(Source::Promise(p)) => p.deja().map(|o| o.as_ref().clone()),
            None => std::fs::read(cle).ok(),
        }
    }

    /// Les octets d'une image, **en attendant** ceux qu'un ouvrier a promis. Pour un fil qui a
    /// le droit d'attendre — la copie d'un lot (fiche 51 § 2) —, jamais pour l'atelier, qui est
    /// celui qui tient la promesse.
    pub fn lire_en_attendant(&self, cle: &str) -> Option<Vec<u8>> {
        if let Some(Source::Promise(p)) = self.source(cle) {
            return p.attendre().map(|o| o.as_ref().clone());
        }
        self.lire(cle)
    }

    fn lire_la_tranche(&self, empreinte: &[u8; 32], offset: u64, longueur: u64) -> Option<Vec<u8>> {
        lire_une_tranche(&self.document()?, empreinte, offset, longueur)
    }
}

/// **Les octets d'une tranche de ce fichier**, s'ils ont l'empreinte annoncée — `None` sinon :
/// une tranche abîmée se dessine introuvable, jamais avec les octets d'une autre image.
pub fn lire_une_tranche(
    fichier: &std::path::Path,
    empreinte: &[u8; 32],
    offset: u64,
    longueur: u64,
) -> Option<Vec<u8>> {
    let mut f = std::fs::File::open(fichier).ok()?;
    f.seek(SeekFrom::Start(offset)).ok()?;
    let mut octets = vec![0u8; usize::try_from(longueur).ok()?];
    f.read_exact(&mut octets).ok()?;
    (sha256(&octets) == *empreinte).then_some(octets)
}
