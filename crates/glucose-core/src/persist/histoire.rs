//! **L'histoire d'un document** : ce qui s'écrit à la suite de la base, en ajout seul.
//!
//! # Pourquoi (fiche 33 § 5, fiche 36 phase 1)
//!
//! `Ctrl+S` relisait chaque image, en refaisait l'empreinte et réécrivait **tout** le fichier :
//! 180 Mo pour 96 Ko de document, deux secondes d'écran figé, et une image sur deux lue dans
//! le dossier temporaire de Windows. Ici, enregistrer coûte **ce qu'on a changé** : chaque
//! geste s'écrit à la suite, au fil du travail, et un plantage ne perd rien de ce qui a été
//! écrit. Remonter le temps, c'est relire cette suite.
//!
//! # La forme du fichier
//!
//! ```text
//! .glucose
//! ├── la BASE    le conteneur v2 inchangé : en-tête, table, manifeste, document, images
//! └── la QUEUE   des entrées à la suite, chacune :
//!                  0..1   nature        1 objet, 2 geste, 3 instantané, 4 jalon, 5 lien, 6 vue
//!                  1..4   réservés, nuls
//!                  4..8   u32 LE        longueur du contenu
//!                  8..16  somme         chaînée à l'entrée précédente
//!                 16..    contenu
//! ```
//!
//! Un fichier v2 existant **est** un fichier de ce format, à la queue vide : rien n'est converti.
//! La première entrée ajoutée fait passer son en-tête à la version 3, pour qu'une build plus
//! ancienne refuse le fichier au lieu d'en montrer une base périmée.
//!
//! # La chaîne des sommes — la leçon du WAL de SQLite
//!
//! Une écriture interrompue laisse une fin déchirée, et une fin déchirée peut *ressembler* à
//! une entrée. Chaque somme couvre donc la précédente, la nature, la longueur et le contenu :
//! `somme(k) = SHA-256(somme(k−1) ‖ nature ‖ longueur ‖ H)[..8]`. La première lecture qui ne
//! suit pas la chaîne marque la fin — tout ce qui précède est intact, rien de ce qui suit
//! n'est cru. La graine est l'empreinte de la **table** de la base : une queue ne peut valider
//! qu'à la suite de sa propre base.
//!
//! Pour un **objet** (les octets d'une image), `H` est l'empreinte que l'entrée *annonce* :
//! ouvrir un document de 180 Mo ne hache pas 180 Mo. Les octets se vérifient contre elle
//! quand on les lit.

pub mod geste;
pub mod ouvrir;
pub mod vue;

pub use geste::Geste;
pub use ouvrir::{ouvrir, Ouvert};
pub use vue::Vue;

use super::bytes::{Reader, Writer};
use super::container::{ENTRY_LEN, HEADER_LEN};
use crate::error::{CoreError, CoreResult};
use crate::hash::sha256;
use crate::types::Project;

/// Les natures d'entrée. INVARIANT PERSIST-4 : un numéro attribué ne bouge jamais.
pub mod nature {
    /// Les octets d'une image : son empreinte, puis ses octets.
    pub const OBJET: u8 = 1;
    /// Une transaction du journal : ce qu'un geste a changé.
    pub const GESTE: u8 = 2;
    /// Le document entier, pour ne pas avoir à tout rejouer.
    pub const INSTANTANE: u8 = 3;
    /// Un point nommé de l'histoire.
    pub const JALON: u8 = 4;
    /// Une clé d'image du document, liée à l'empreinte de ses octets.
    pub const LIEN: u8 = 5;
    /// La navigation : tableau actif, caméras, signets.
    pub const VUE: u8 = 6;
}

/// Longueur de l'en-tête d'une entrée.
pub const ENTETE: usize = 16;

/// La somme qui suit `somme` pour une entrée de cette nature, de cette longueur, dont le
/// contenu a pour empreinte `h`.
fn suivante(somme: [u8; 8], nature: u8, longueur: u32, h: &[u8; 32]) -> [u8; 8] {
    let mut couvert = Vec::with_capacity(8 + 1 + 4 + 32);
    couvert.extend_from_slice(&somme);
    couvert.push(nature);
    couvert.extend_from_slice(&longueur.to_le_bytes());
    couvert.extend_from_slice(h);
    let e = sha256(&couvert);
    let mut s = [0u8; 8];
    s.copy_from_slice(&e[..8]);
    s
}

/// L'état de la chaîne : la somme de la dernière entrée écrite ou lue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chaine(pub [u8; 8]);

impl Chaine {
    /// La graine d'une base : l'empreinte de sa table des sections.
    ///
    /// `base` commence au premier octet du fichier et couvre au moins sa table.
    pub fn de_la_base(base: &[u8]) -> CoreResult<Self> {
        let n = base
            .get(12..16)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]) as usize)
            .ok_or_else(|| tronque("l'en-tête de la base"))?;
        let table = base
            .get(HEADER_LEN..HEADER_LEN + ENTRY_LEN * n)
            .ok_or_else(|| tronque("la table de la base"))?;
        let e = sha256(table);
        let mut s = [0u8; 8];
        s.copy_from_slice(&e[..8]);
        Ok(Self(s))
    }

    /// L'en-tête d'une entrée à écrire. La chaîne avance.
    pub fn entete(&mut self, nature: u8, longueur: u32, h: &[u8; 32]) -> [u8; ENTETE] {
        self.0 = suivante(self.0, nature, longueur, h);
        let mut e = [0u8; ENTETE];
        e[0] = nature;
        e[4..8].copy_from_slice(&longueur.to_le_bytes());
        e[8..16].copy_from_slice(&self.0);
        e
    }

    /// Une entrée entière — en-tête puis contenu — pour un contenu déjà en mémoire.
    ///
    /// Un objet (image) ne passe pas par ici : son en-tête se chaîne sur l'empreinte annoncée,
    /// voir [`Chaine::entete_d_objet`].
    pub fn encadrer(&mut self, nature: u8, contenu: &[u8]) -> Vec<u8> {
        let mut e = Vec::with_capacity(ENTETE + contenu.len());
        e.extend_from_slice(&self.entete(nature, contenu.len() as u32, &sha256(contenu)));
        e.extend_from_slice(contenu);
        e
    }

    /// L'en-tête d'un objet et son empreinte, à faire suivre des `longueur` octets de l'image.
    pub fn entete_d_objet(&mut self, empreinte: &[u8; 32], longueur: u64) -> CoreResult<Vec<u8>> {
        let total = u32::try_from(longueur + 32).map_err(|_| {
            CoreError::SerializationError(
                "une image de plus de 4 Go ne tient pas dans une entrée".to_string(),
            )
        })?;
        let mut e = Vec::with_capacity(ENTETE + 32);
        e.extend_from_slice(&self.entete(nature::OBJET, total, empreinte));
        e.extend_from_slice(empreinte);
        Ok(e)
    }
}

fn tronque(quoi: &str) -> CoreError {
    CoreError::DeserializationError(format!(
        "fichier .glucose tronqué : {quoi} manque — rouvre la copie précédente du projet"
    ))
}

/// Un point nommé de l'histoire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jalon {
    pub instant: i64,
    pub genre: Genre,
    pub libelle: String,
}

/// Qui a posé un jalon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    /// L'utilisateur, en le nommant.
    Nomme,
    /// `Ctrl+S` : « ici, c'était bien ».
    Enregistrement,
}

// ── Les contenus ────────────────────────────────────────────────────────────

/// Un geste, préfixé du schéma de document auquel ses éléments sont écrits.
pub fn contenu_geste(g: &Geste) -> Vec<u8> {
    let mut w = Writer::with_capacity(256);
    w.u16(super::DOCUMENT_VERSION);
    geste::ecrire(&mut w, g);
    w.into_bytes()
}

/// Le document entier, préfixé de son schéma.
pub fn contenu_instantane(p: &Project) -> Vec<u8> {
    let mut w = Writer::with_capacity(4096);
    w.u16(super::DOCUMENT_VERSION);
    w.raw(&super::encode_document(p));
    w.into_bytes()
}

pub fn contenu_jalon(j: &Jalon) -> Vec<u8> {
    let mut w = Writer::new();
    w.i64(j.instant);
    w.u8(match j.genre {
        Genre::Nomme => 0,
        Genre::Enregistrement => 1,
    });
    w.text(&j.libelle);
    w.into_bytes()
}

pub fn contenu_lien(cle: &str, empreinte: &[u8; 32]) -> Vec<u8> {
    let mut w = Writer::new();
    w.text(cle);
    w.raw(empreinte);
    w.into_bytes()
}

pub fn contenu_vue(v: &Vue) -> Vec<u8> {
    let mut w = Writer::new();
    vue::ecrire(&mut w, v);
    w.into_bytes()
}

fn lire_jalon(r: &mut Reader<'_>) -> CoreResult<Jalon> {
    Ok(Jalon {
        instant: r.i64()?,
        genre: match r.u8()? {
            0 => Genre::Nomme,
            1 => Genre::Enregistrement,
            autre => return Err(super::tags::unknown("genre de jalon", autre)),
        },
        libelle: r.text()?,
    })
}

fn lire_lien(r: &mut Reader<'_>) -> CoreResult<(String, [u8; 32])> {
    Ok((r.text()?, r.digest()?))
}

/// Relit le schéma, puis le geste.
pub fn lire_geste(contenu: &[u8]) -> CoreResult<Geste> {
    let mut r = Reader::new(contenu);
    let version = r.u16()?;
    super::check_document_version(version)?;
    let g = geste::lire(&mut r, version)?;
    r.finish()?;
    Ok(g)
}

/// Relit un instantané.
pub fn lire_instantane(contenu: &[u8]) -> CoreResult<Project> {
    let mut r = Reader::new(contenu);
    let version = r.u16()?;
    super::check_document_version(version)?;
    super::decode_document_v(r.take(r.remaining())?, version)
}
