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
use std::sync::{Arc, RwLock};

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
}

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
            None => std::fs::read(cle).ok(),
        }
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
