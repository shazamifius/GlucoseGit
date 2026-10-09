//! **COPIER-1 — la sélection, prête à partir sous toutes ses formes** (fiche 59).
//!
//! Sa demande, le 09/10 : « copier l'image » et « copier » étaient deux gestes, et ne devraient
//! en être qu'un ; un `Ctrl+V` dans Discord doit envoyer l'image, plusieurs images les envoyer
//! toutes — et partout, pas seulement dans Discord ; dans Glucose, rien ne change : un bloc
//! recolle un bloc. Et le glisser de même : « une image reste une image ».
//!
//! `Ctrl+C` emporte donc tout ce qu'une autre application peut vouloir : le **lot** de Glucose,
//! le **texte** des nœuds de texte, et les **images** elles-mêmes — l'image seule en PNG et en
//! pixels, chaque image en fichier, octets d'origine. Ce module les rassemble ; ce que chaque
//! système en fait, et quand il les fabrique, vit dans `plateforme` (`selection_windows`).
//!
//! Les images se lisent **une fois**, et servent au lot comme aux fichiers.

use super::menu_image::{extension_de, preparer, ImagePosee};
use crate::persist::objets::Objets;
use glucose_core::types::{AssetStore, Project};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

/// Une image telle qu'elle partira en fichier : son nom, et ses octets d'origine.
pub struct Fichier {
    pub nom: String,
    pub octets: Arc<Vec<u8>>,
}

/// La sélection sous toutes ses formes.
pub struct Formes {
    /// Le lot de nœuds : un `.glucose`, images comprises (fiche 51 § 2).
    pub lot: Vec<u8>,
    /// Le texte des nœuds de texte, s'il y en a.
    pub texte: Option<String>,
    /// **L'image seule**, en PNG et en pixels : ce que pose « Copier l'image » d'un navigateur.
    pub image: Option<ImagePosee>,
    /// Les images du lot, une par fichier, dans l'ordre du document — une image posée deux fois
    /// n'est qu'un fichier.
    pub fichiers: Vec<Fichier>,
    /// La date du lot, à la nanoseconde : le nom du dossier où ses fichiers s'écriront.
    pub date: i64,
}

/// **Rassemble les formes d'un lot.** L'image seule ne se prépare que si on la demande — le
/// presse-papiers la veut, le glisser non. Lourd (une lecture par image, parfois un décodage) :
/// jamais sur le fil qui dessine, sauf pour le glisser, bloquant de toute façon.
pub fn rassembler(
    lot: &Project,
    objets: &Objets,
    (texte, avec_l_image_seule): (Option<String>, bool),
) -> Formes {
    let mut actifs = AssetStore::new();
    let mut fichiers = Vec::new();
    let mut vues = HashSet::new();
    for img in lot.toutes_les_images() {
        let Some(cle) = img.src.as_deref() else {
            continue;
        };
        if !vues.insert(cle) {
            continue;
        }
        let Some(octets) = objets.lire_en_attendant(cle) else {
            continue;
        };
        let octets = Arc::new(octets);
        actifs.insert(cle, octets.as_ref().clone());
        fichiers.push(Fichier {
            nom: nom_du_fichier(cle, &octets),
            octets,
        });
    }
    let image = match (avec_l_image_seule, fichiers.as_slice()) {
        (true, [seul]) => preparer(&seul.octets).ok(),
        _ => None,
    };
    Formes {
        lot: glucose_core::persist::encode(lot, &actifs, 0),
        texte,
        image,
        fichiers,
        date: lot.created_at,
    }
}

/// **Le nom d'une image en fichier** : celui du fichier d'où elle vient quand on le connaît, sinon
/// « image » ; et l'extension que la signature de ses octets dit — un PNG nommé `.jpg` serait
/// refusé par plus d'une application.
fn nom_du_fichier(cle: &str, octets: &[u8]) -> String {
    let base = Path::new(cle)
        .is_absolute()
        .then(|| Path::new(cle).file_stem().and_then(|s| s.to_str()))
        .flatten()
        .unwrap_or("image");
    format!("{base}.{}", extension_de(octets).0)
}

#[cfg(test)]
mod tests;
