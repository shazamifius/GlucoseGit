//! Section `manifeste` du format `.glucose` v2.
//!
//! Le manifeste est la **table des matières** du fichier : version du schéma de document, nom du
//! projet, date d'écriture, et la table des actifs `clé → empreinte`.
//!
//! # Pourquoi une table `clé → empreinte` alors que les actifs sont adressés par contenu
//!
//! Les contenus sont stockés une seule fois, sous leur sha256 (déduplication native du § 8).
//! Mais le document, lui, désigne un actif par une **clé** lisible (le chemin d'origine de
//! l'image). Deux clés différentes qui pointent le même contenu partagent donc une seule
//! section ; c'est exactement ce que prouve le test `un actif dupliqué n'écrit qu'un blob`.

use super::bytes::{Reader, Writer};
use crate::error::CoreResult;

/// Une entrée de la table des actifs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetEntry {
    /// Clé du magasin d'actifs — telle que le document la référence.
    pub key: String,
    /// sha256 du contenu : c'est aussi l'adresse de la section qui le porte.
    pub digest: [u8; 32],
    /// Taille du contenu, en octets. Redondante avec la section, donc vérifiée au chargement.
    pub size: u64,
}

/// En-tête logique d'un projet enregistré.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Version du **schéma du document**, indépendante de celle du conteneur : le conteneur
    /// peut rester v2 pendant que le modèle de données évolue.
    pub document_version: u16,
    pub project_name: String,
    /// Millisecondes depuis l'époque Unix. Fournie par l'appelant : le noyau ne lit pas l'heure.
    pub saved_at: i64,
    pub assets: Vec<AssetEntry>,
}

pub fn encode(manifest: &Manifest) -> Vec<u8> {
    let Manifest {
        document_version,
        project_name,
        saved_at,
        assets,
    } = manifest;

    let mut w = Writer::with_capacity(64 + assets.len() * 64);
    w.u16(*document_version);
    w.text(project_name);
    w.i64(*saved_at);
    w.seq(assets, |ww, entry| {
        ww.text(&entry.key);
        ww.raw(&entry.digest);
        ww.u64(entry.size);
    });
    w.into_bytes()
}

pub fn decode(bytes: &[u8]) -> CoreResult<Manifest> {
    let mut r = Reader::new(bytes);
    let manifest = Manifest {
        document_version: r.u16()?,
        project_name: r.text()?,
        saved_at: r.i64()?,
        assets: r.seq(|rr| {
            Ok(AssetEntry {
                key: rr.text()?,
                digest: rr.digest()?,
                size: rr.u64()?,
            })
        })?,
    };
    r.finish()?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_round_trip() {
        let manifest = Manifest {
            document_version: 1,
            project_name: "Étude de cas — été 2026".into(),
            saved_at: 1_770_000_000_000,
            assets: vec![
                AssetEntry {
                    key: "C:/photos/a.png".into(),
                    digest: [7u8; 32],
                    size: 1234,
                },
                AssetEntry {
                    key: "C:/photos/b.png".into(),
                    digest: [9u8; 32],
                    size: 0,
                },
            ],
        };
        let decoded = decode(&encode(&manifest)).expect("le manifeste doit se relire");
        assert_eq!(decoded, manifest);
    }

    #[test]
    fn test_truncated_manifest_is_an_error() {
        let manifest = Manifest {
            document_version: 1,
            project_name: "P".into(),
            saved_at: 0,
            assets: Vec::new(),
        };
        let mut bytes = encode(&manifest);
        bytes.truncate(2);
        assert!(decode(&bytes).is_err());
    }
}
