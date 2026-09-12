//! Le conteneur `.glucose` v2 : en-tête versionné, table de sections, somme de contrôle par
//! section.
//!
//! # Disposition exacte des octets
//!
//! ```text
//! EN-TÊTE — 16 octets, toujours en tête du fichier
//!    0..8   magie          b"GLUCOSE\x1a"
//!    8..10  u16 LE         version du conteneur (2 aujourd'hui)
//!   10..12  u16 LE         fanions, réservés, valent 0
//!   12..16  u32 LE         nombre de sections N
//!
//! TABLE DES SECTIONS — N entrées de 56 octets, contiguës après l'en-tête
//!    0..1   u8             nature : 1 manifeste, 2 document, 3 actif, 4 journal (réservé)
//!    1..8   7 octets       réservés, valent 0
//!    8..16  u64 LE         offset du contenu depuis le début du fichier
//!   16..24  u64 LE         longueur du contenu
//!   24..56  [u8; 32]       sha256(contenu)
//!
//! CONTENUS — concaténés, dans l'ordre de la table
//! ```
//!
//! # Pourquoi la table est de taille fixe
//!
//! Elle se lit d'un bloc, avant tout contenu : on connaît la liste complète des sections et
//! leurs sommes de contrôle après 16 + 56 N octets. Un fichier tronqué est donc détecté sans
//! avoir rien décodé, et une version future peut ajouter une nature de section (le journal,
//! nature 4) que cette version **saute** au lieu de refuser le fichier.
//!
//! # Pourquoi la somme de contrôle est un sha256 et pas un CRC
//!
//! Pour un actif, `sha256(contenu)` **est** son adresse de contenu (§ 8, « assets
//! content-addressed »). Le même champ sert donc de vérification d'intégrité et
//! d'identité : deux occurrences de la même image donnent la même entrée, donc un seul
//! contenu écrit. Le sha256 est déjà dans le dépôt — [`crate::bundle::sha256`] — et ne coûte
//! aucune dépendance.

use crate::bundle::sha256;
use crate::error::{CoreError, CoreResult};

/// Magie du conteneur. `0x1a` final = `Ctrl-Z`, qui stoppe l'affichage d'un `type` sous
/// Windows : un `.glucose` ouvert par erreur dans un terminal ne le remplit pas de binaire.
pub const MAGIC: [u8; 8] = *b"GLUCOSE\x1a";
/// Version du conteneur produite par cette build.
pub const CONTAINER_VERSION: u16 = 2;
/// Plus ancienne version de conteneur que cette build sait lire.
pub const MIN_READABLE_VERSION: u16 = 2;

pub const HEADER_LEN: usize = 16;
pub const ENTRY_LEN: usize = 56;

pub const KIND_MANIFEST: u8 = 1;
pub const KIND_DOCUMENT: u8 = 2;
pub const KIND_ASSET: u8 = 3;
/// Réservé au journal incrémental du § 8. Cette version l'ignore au lieu de refuser le
/// fichier : un `.glucose` écrit par une build à journal reste ouvrable ici.
pub const KIND_JOURNAL: u8 = 4;

/// Une section prête à être écrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub kind: u8,
    pub payload: Vec<u8>,
}

/// Une section lue et vérifiée, empruntée au tampon du fichier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedSection<'a> {
    pub kind: u8,
    pub digest: [u8; 32],
    pub payload: &'a [u8],
}

fn kind_name(kind: u8) -> &'static str {
    match kind {
        KIND_MANIFEST => "le manifeste",
        KIND_DOCUMENT => "le document",
        KIND_ASSET => "un actif",
        KIND_JOURNAL => "le journal",
        _ => "une section inconnue",
    }
}

/// Assemble un conteneur complet. Infaillible : tout est déjà en mémoire.
pub fn assemble(sections: &[Section]) -> Vec<u8> {
    let table_len = ENTRY_LEN * sections.len();
    let body_len: usize = sections.iter().map(|s| s.payload.len()).sum();
    let mut out = Vec::with_capacity(HEADER_LEN + table_len + body_len);

    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&CONTAINER_VERSION.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(sections.len() as u32).to_le_bytes());

    let mut offset = (HEADER_LEN + table_len) as u64;
    for section in sections {
        out.push(section.kind);
        out.extend_from_slice(&[0u8; 7]);
        out.extend_from_slice(&offset.to_le_bytes());
        out.extend_from_slice(&(section.payload.len() as u64).to_le_bytes());
        out.extend_from_slice(&sha256(&section.payload));
        offset += section.payload.len() as u64;
    }

    for section in sections {
        out.extend_from_slice(&section.payload);
    }
    out
}

fn truncated(what: &str) -> CoreError {
    CoreError::DeserializationError(format!(
        "fichier .glucose tronqué : {what} manque — la sauvegarde a été interrompue, \
         rouvre la copie précédente du projet"
    ))
}

/// Lit l'en-tête et rend le nombre de sections annoncé.
fn read_header(bytes: &[u8]) -> CoreResult<usize> {
    if bytes.len() < HEADER_LEN {
        return Err(truncated("l'en-tête"));
    }
    if bytes[0..8] != MAGIC {
        return Err(CoreError::DeserializationError(
            "ce fichier n'est pas un projet Glucose : sa signature ne correspond pas — \
             vérifie que tu ouvres bien un fichier .glucose"
                .to_string(),
        ));
    }

    let version = u16::from_le_bytes([bytes[8], bytes[9]]);
    if version > CONTAINER_VERSION {
        return Err(CoreError::DeserializationError(format!(
            "projet écrit au format .glucose v{version}, cette version de Glucose lit jusqu'à \
             la v{CONTAINER_VERSION} — mets Glucose à jour pour l'ouvrir"
        )));
    }
    if version < MIN_READABLE_VERSION {
        return Err(CoreError::DeserializationError(format!(
            "projet écrit au format .glucose v{version}, antérieur au format binaire v{MIN_READABLE_VERSION} — \
             l'importeur des projets v1 (TypeScript) n'est pas encore livré, garde ce fichier tel quel"
        )));
    }

    let flags = u16::from_le_bytes([bytes[10], bytes[11]]);
    if flags != 0 {
        return Err(CoreError::DeserializationError(format!(
            "fanions d'en-tête inconnus ({flags:#06x}) : ce projet utilise une extension du \
             format que cette version ignore — mets Glucose à jour"
        )));
    }

    let count = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
    usize::try_from(count).map_err(|_| {
        CoreError::DeserializationError(
            "nombre de sections hors des capacités de cette machine — fichier corrompu".to_string(),
        )
    })
}

/// Lit une entrée de la table et vérifie les bornes puis la somme de contrôle du contenu.
fn read_entry<'a>(bytes: &'a [u8], entry: &[u8]) -> CoreResult<ParsedSection<'a>> {
    let kind = entry[0];
    if entry[1..8] != [0u8; 7] {
        return Err(CoreError::DeserializationError(format!(
            "octets réservés non nuls dans la table de sections pour {} — fichier corrompu",
            kind_name(kind)
        )));
    }

    let offset = u64::from_le_bytes([
        entry[8], entry[9], entry[10], entry[11], entry[12], entry[13], entry[14], entry[15],
    ]);
    let length = u64::from_le_bytes([
        entry[16], entry[17], entry[18], entry[19], entry[20], entry[21], entry[22], entry[23],
    ]);
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&entry[24..56]);

    let start = usize::try_from(offset).map_err(|_| truncated("un contenu de section"))?;
    let len = usize::try_from(length).map_err(|_| truncated("un contenu de section"))?;
    let end = start
        .checked_add(len)
        .ok_or_else(|| truncated("un contenu de section"))?;
    if end > bytes.len() {
        return Err(CoreError::DeserializationError(format!(
            "fichier .glucose tronqué : {} annonce {} octets à l'offset {}, le fichier n'en compte que {} — \
             rouvre la copie précédente du projet",
            kind_name(kind),
            len,
            start,
            bytes.len()
        )));
    }

    let payload = &bytes[start..end];
    if sha256(payload) != digest {
        return Err(CoreError::DeserializationError(format!(
            "somme de contrôle fausse pour {} : le fichier a été altéré ou l'écriture a été \
             interrompue — rouvre la copie précédente du projet",
            kind_name(kind)
        )));
    }

    Ok(ParsedSection {
        kind,
        digest,
        payload,
    })
}

/// Vérifie l'en-tête, la table et **toutes** les sommes de contrôle, puis rend les sections.
///
/// Aucun contenu n'est décodé ici : le conteneur est prouvé intègre avant que le moindre
/// champ du document ne soit interprété.
pub fn parse(bytes: &[u8]) -> CoreResult<Vec<ParsedSection<'_>>> {
    let count = read_header(bytes)?;
    let table_end = HEADER_LEN
        .checked_add(
            ENTRY_LEN
                .checked_mul(count)
                .ok_or_else(|| truncated("la table des sections"))?,
        )
        .ok_or_else(|| truncated("la table des sections"))?;
    if bytes.len() < table_end {
        return Err(truncated("la table des sections"));
    }

    let mut sections = Vec::with_capacity(count.min(64));
    for index in 0..count {
        let start = HEADER_LEN + index * ENTRY_LEN;
        sections.push(read_entry(bytes, &bytes[start..start + ENTRY_LEN])?);
    }
    Ok(sections)
}

/// Rend l'unique section de la nature demandée, ou une erreur qui dit laquelle manque.
pub fn require<'a>(sections: &[ParsedSection<'a>], kind: u8) -> CoreResult<ParsedSection<'a>> {
    sections
        .iter()
        .find(|s| s.kind == kind)
        .copied()
        .ok_or_else(|| {
            CoreError::DeserializationError(format!(
                "{} est absent du fichier .glucose — le projet est incomplet, rouvre la copie précédente",
                kind_name(kind)
            ))
        })
}
