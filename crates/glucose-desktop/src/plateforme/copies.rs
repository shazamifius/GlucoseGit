//! **Les images d'une copie, en fichiers, écrites quand une application les demande**
//! (COPIER-1, fiche 59) — sous Windows (`CF_HDROP`) comme sous Linux (`text/uri-list`).
//!
//! # Rien ne s'écrit avant qu'on le demande
//!
//! Les fichiers sont les originaux des images, octet pour octet. Les écrire à chaque `Ctrl+C` —
//! trente images, des dizaines de mégaoctets — pour recoller dans Glucose, qui ne les lit
//! jamais, serait du travail perdu. Ils s'écrivent donc au premier collage qui les veut, une
//! fois ([`EnFichiers::ecrits`]).
//!
//! # Où, et combien de temps
//!
//! Dans `glucose-copies/<la date du lot>/`, sous le dossier temporaire du système, au nom de
//! chaque image. Le dossier d'une sélection partie s'efface quand la suivante part **de la même
//! façon** : le presse-papiers ne portait plus que la nouvelle. Un glisser n'efface pas ce qu'un
//! collage lira encore. Ce que la fermeture laisse, le nettoyage du système le reprend.

use super::super::interactions::clipboard::{Fichier, Pour};
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

thread_local! {
    /// Le dossier du dernier lot parti, au presse-papiers et en glisser : celui-là seul peut
    /// encore servir. Les deux ne se remplacent pas. Propre au fil qui fait partir les lots —
    /// celui de la fenêtre ; chaque fil d'épreuve a donc les siens.
    static DERNIERS_DOSSIERS: RefCell<[Option<PathBuf>; 2]> = const { RefCell::new([None, None]) };
}

/// Le dossier où les copies de Glucose posent leurs fichiers.
pub fn dossier_des_copies() -> PathBuf {
    std::env::temp_dir().join("glucose-copies")
}

/// Les fichiers d'une sélection partie : où ils s'écriront, et ce qui l'a déjà été.
pub struct EnFichiers {
    dossier: PathBuf,
    /// Les chemins écrits — une fois, au premier collage qui les veut.
    ecrits: OnceLock<Result<Vec<PathBuf>, String>>,
}

impl EnFichiers {
    /// **Le dossier d'un lot qui part** ; celui du lot parti avant lui de la même façon s'efface.
    pub fn pour(date: i64, pour: Pour) -> Self {
        let dossier = dossier_des_copies().join(date.to_string());
        DERNIERS_DOSSIERS.with(|d| {
            let precedent = d.borrow_mut()[pour as usize].replace(dossier.clone());
            if let Some(ancien) = precedent.filter(|a| *a != dossier) {
                let _ = std::fs::remove_dir_all(ancien);
            }
        });
        Self {
            dossier,
            ecrits: OnceLock::new(),
        }
    }

    /// **Les fichiers, écrits au premier collage qui les veut**, et jamais deux fois.
    pub fn ecrits(&self, fichiers: &[Fichier]) -> Result<Vec<PathBuf>, String> {
        self.ecrits
            .get_or_init(|| ecrire_les_fichiers(fichiers, &self.dossier))
            .clone()
    }
}

/// Écrit chaque image sous un nom libre du dossier, ses octets d'origine tels quels.
fn ecrire_les_fichiers(fichiers: &[Fichier], dossier: &Path) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(dossier).map_err(|e| e.to_string())?;
    let mut pris = HashSet::new();
    let mut chemins = Vec::with_capacity(fichiers.len());
    for f in fichiers {
        let chemin = dossier.join(nom_libre(&f.nom, &mut pris));
        std::fs::write(&chemin, f.octets.as_slice()).map_err(|e| e.to_string())?;
        chemins.push(chemin);
    }
    Ok(chemins)
}

/// **Un nom que le dossier n'a pas encore** : « photo.png », puis « photo 2.png »… Windows et
/// macOS ne distinguent pas les majuscules : la comparaison non plus, partout — deux noms qui ne
/// diffèrent que par elles se confondraient chez l'un d'eux.
fn nom_libre(nom: &str, pris: &mut HashSet<String>) -> String {
    let (base, extension) = nom.rsplit_once('.').unwrap_or((nom, ""));
    let mut candidat = nom.to_string();
    let mut n = 1;
    while !pris.insert(candidat.to_lowercase()) {
        n += 1;
        candidat = if extension.is_empty() {
            format!("{base} {n}")
        } else {
            format!("{base} {n}.{extension}")
        };
    }
    candidat
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn fichier(nom: &str, octet: u8) -> Fichier {
        Fichier {
            nom: nom.into(),
            octets: Arc::new(vec![octet; 64]),
        }
    }

    /// Une date neuve à chaque appel : chaque épreuve a son dossier, même en parallèle.
    fn date() -> i64 {
        static N: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
        let base = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos() as i64);
        base + N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// **Un nom libre ne regarde pas les majuscules** : Windows et macOS non plus.
    #[test]
    fn test_un_nom_libre_ignore_les_majuscules() {
        let mut pris = HashSet::new();
        assert_eq!(nom_libre("Photo.PNG", &mut pris), "Photo.PNG");
        assert_eq!(nom_libre("photo.png", &mut pris), "photo 2.png");
        assert_eq!(nom_libre("sans", &mut pris), "sans");
        assert_eq!(nom_libre("sans", &mut pris), "sans 2");
    }

    /// **Les fichiers s'écrivent une fois, au premier qui les veut** : rien avant ; puis les
    /// originaux sous des noms libres ; une retouche faite par une application n'est jamais
    /// réécrite par un second collage.
    #[test]
    fn test_les_fichiers_s_ecrivent_une_fois_a_la_demande() {
        let fichiers = [fichier("a.png", 1), fichier("a.png", 2)];
        let d = date();
        let en_fichiers = EnFichiers::pour(d, Pour::Glisser);
        let dossier = dossier_des_copies().join(d.to_string());
        assert!(!dossier.exists(), "rien d'écrit avant qu'on le demande");
        let premiers = en_fichiers.ecrits(&fichiers).expect("écrits");
        let noms: Vec<_> = premiers
            .iter()
            .filter_map(|p| p.file_name()?.to_str())
            .collect();
        assert_eq!(noms, ["a.png", "a 2.png"], "deux noms libres");
        assert_eq!(std::fs::read(&premiers[1]).expect("lu"), vec![2u8; 64]);
        std::fs::write(&premiers[1], b"retouche").expect("modifié");
        assert_eq!(en_fichiers.ecrits(&fichiers).expect("relus"), premiers);
        assert_eq!(std::fs::read(&premiers[1]).expect("lu"), b"retouche");
        let _ = std::fs::remove_dir_all(dossier);
    }
}
