//! Persistance : le format `.glucose` v2 (répare R-01).
//!
//! Tout ce module est **pur** : il transforme un [`Project`] en `Vec<u8>` et l'inverse. Aucune
//! ouverture de fichier, aucun dialogue, aucune horloge — l'I/O vit dans `glucose-desktop`,
//! ce qui rend le format testable sans fenêtre (standard § 7.1).
//!
//! # Le conteneur
//!
//! ```text
//! .glucose
//! ├── manifeste     version du schéma, nom, date, table des actifs (clé → sha256)
//! ├── document      projet, tableaux, images, annotations, dossiers, domaines, presets
//! └── actifs        un contenu par sha256 distinct, dédupliqué
//! ```
//!
//! La disposition exacte des octets de l'en-tête et de la table est documentée dans
//! [`container`]. Les conventions de champ (petit-boutiste, varints, `f64` par ses bits) le
//! sont dans [`bytes`].
//!
//! # Deux numéros de version, et pourquoi
//!
//! - [`container::CONTAINER_VERSION`] décrit l'**enveloppe** : en-tête, table, sommes de
//!   contrôle. Elle ne bougera que si la structure du fichier change.
//! - [`DOCUMENT_VERSION`] décrit le **schéma du modèle**. C'est celui qui bougera à chaque
//!   champ ajouté, et c'est sur lui que se brancheront les migrations chaînées `v1 → v2 → v3`
//!   du § 8.
//!
//! Les séparer évite de faire monter la version du conteneur — donc de casser la lecture par
//! les outils — à chaque évolution du modèle.
//!
//! # Ce que ce format laisse ouvert
//!
//! - **Le journal incrémental** : la nature de section [`container::KIND_JOURNAL`] est
//!   réservée et **sautée** à la lecture. Une build à journal peut donc écrire un fichier que
//!   cette build ouvre encore, en perdant seulement les commandes non fusionnées.
//! - **L'import v1** : un conteneur de version inférieure à
//!   [`container::MIN_READABLE_VERSION`] est refusé par un message qui nomme l'importeur
//!   manquant, au lieu d'un « fichier corrompu » trompeur.

pub mod bytes;
pub mod container;
pub mod manifest;

mod annotation;
mod document;
mod image;
mod tags;

use crate::hash::sha256;
use crate::error::{CoreError, CoreResult};
use crate::types::{AssetStore, Project};
use bytes::{Reader, Writer};
use container::{ParsedSection, Section};
use manifest::{AssetEntry, Manifest};

/// Version du schéma du document produite par cette build.
pub const DOCUMENT_VERSION: u16 = 1;
/// Plus ancien schéma de document que cette build sait lire.
pub const MIN_READABLE_DOCUMENT_VERSION: u16 = 1;
/// Extension de fichier, sans le point.
pub const FILE_EXTENSION: &str = "glucose";

/// Le contenu complet d'un fichier `.glucose` relu.
#[derive(Debug, Clone, PartialEq)]
pub struct GlucoseFile {
    pub project: Project,
    pub assets: AssetStore,
    pub manifest: Manifest,
}

// ── Document seul ───────────────────────────────────────────────────────────

/// Sérialise le document nu, sans conteneur ni somme de contrôle.
///
/// Utile pour un test d'aller-retour ciblé et, demain, pour le journal incrémental qui aura
/// besoin d'écrire un point de reprise sans réécrire les actifs.
pub fn encode_document(project: &Project) -> Vec<u8> {
    let mut w = Writer::with_capacity(4096);
    document::write_project(&mut w, project);
    w.into_bytes()
}

/// Relit un document nu produit par [`encode_document`].
pub fn decode_document(payload: &[u8]) -> CoreResult<Project> {
    let mut r = Reader::new(payload);
    let project = document::read_project(&mut r)?;
    r.finish()?;
    Ok(project)
}

// ── Fichier complet ─────────────────────────────────────────────────────────

/// Construit le fichier `.glucose` complet.
///
/// `saved_at` est fourni par l'appelant (millisecondes Unix) : le noyau ne lit jamais l'horloge,
/// sinon deux encodages du même projet ne seraient plus comparables en test.
///
/// Les actifs sont **dédupliqués par contenu** : deux clés portant les mêmes octets partagent
/// une unique section, et un actif présent deux fois n'est écrit qu'une fois.
pub fn encode(project: &Project, assets: &AssetStore, saved_at: i64) -> Vec<u8> {
    let mut entries: Vec<AssetEntry> = assets
        .blobs
        .iter()
        .map(|(key, data)| AssetEntry {
            key: key.clone(),
            digest: sha256(data),
            size: data.len() as u64,
        })
        .collect();
    // Ordre stable : deux enregistrements du même projet donnent les mêmes octets.
    entries.sort_by(|a, b| a.key.cmp(&b.key));

    let mut sections = Vec::with_capacity(2 + entries.len());
    sections.push(Section {
        kind: container::KIND_MANIFEST,
        payload: manifest::encode(&Manifest {
            document_version: DOCUMENT_VERSION,
            project_name: project.name.clone(),
            saved_at,
            assets: entries.clone(),
        }),
    });
    sections.push(Section {
        kind: container::KIND_DOCUMENT,
        payload: encode_document(project),
    });

    let mut written: Vec<[u8; 32]> = Vec::with_capacity(entries.len());
    for entry in &entries {
        if written.contains(&entry.digest) {
            continue;
        }
        let Some(data) = assets.get(&entry.key) else {
            continue;
        };
        written.push(entry.digest);
        sections.push(Section {
            kind: container::KIND_ASSET,
            payload: data.to_vec(),
        });
    }

    container::assemble(&sections)
}

/// Relit un fichier `.glucose` complet.
///
/// L'intégrité est prouvée avant tout décodage : en-tête, table, puis **chaque** somme de
/// contrôle. Un fichier tronqué ou altéré rend une [`CoreError::DeserializationError`] dont le
/// message dit quoi faire (standard § 6.5), jamais une panique ni un document à moitié juste.
pub fn decode(file: &[u8]) -> CoreResult<GlucoseFile> {
    let sections = container::parse(file)?;

    let manifest_section = container::require(&sections, container::KIND_MANIFEST)?;
    let manifest = manifest::decode(manifest_section.payload)?;
    check_document_version(manifest.document_version)?;

    let document_section = container::require(&sections, container::KIND_DOCUMENT)?;
    let project = decode_document(document_section.payload)?;
    let assets = rebuild_assets(&manifest, &sections)?;

    Ok(GlucoseFile {
        project,
        assets,
        manifest,
    })
}

fn check_document_version(version: u16) -> CoreResult<()> {
    if version > DOCUMENT_VERSION {
        return Err(CoreError::DeserializationError(format!(
            "document au schéma v{version}, cette version de Glucose lit jusqu'au schéma \
             v{DOCUMENT_VERSION} — mets Glucose à jour pour ouvrir ce projet"
        )));
    }
    if version < MIN_READABLE_DOCUMENT_VERSION {
        return Err(CoreError::DeserializationError(format!(
            "document au schéma v{version}, antérieur au plus ancien schéma lisible \
             (v{MIN_READABLE_DOCUMENT_VERSION}) — la migration de ce schéma n'est pas encore livrée"
        )));
    }
    Ok(())
}

/// Réassocie chaque clé d'actif au contenu qui porte son empreinte.
fn rebuild_assets(manifest: &Manifest, sections: &[ParsedSection<'_>]) -> CoreResult<AssetStore> {
    let mut store = AssetStore::new();
    for entry in &manifest.assets {
        let blob = sections
            .iter()
            .find(|s| s.kind == container::KIND_ASSET && s.digest == entry.digest)
            .ok_or_else(|| {
                CoreError::DeserializationError(format!(
                    "l'actif « {} » est annoncé par le manifeste mais absent du fichier — \
                     le projet a été enregistré incomplètement, rouvre la copie précédente",
                    entry.key
                ))
            })?;
        if blob.payload.len() as u64 != entry.size {
            return Err(CoreError::DeserializationError(format!(
                "l'actif « {} » fait {} octets au lieu des {} annoncés — fichier corrompu",
                entry.key,
                blob.payload.len(),
                entry.size
            )));
        }
        store.insert(entry.key.clone(), blob.payload.to_vec());
    }
    Ok(store)
}

/// Nombre de sections d'actif réellement écrites — la preuve de la déduplication.
pub fn asset_section_count(file: &[u8]) -> CoreResult<usize> {
    Ok(container::parse(file)?
        .iter()
        .filter(|s| s.kind == container::KIND_ASSET)
        .count())
}
