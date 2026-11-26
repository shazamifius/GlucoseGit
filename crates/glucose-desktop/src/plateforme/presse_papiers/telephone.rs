//! **Le presse-papiers d'Android** (PRESSE-ANDROID-1, fiche 171).
//!
//! Java ne fait que ce que seul Java peut faire — écrire un `ClipData`, lire son étiquette, ouvrir
//! ses adresses — par une [`Porte`] que `glucose-android` branche au lancement. Ce qui décide est
//! ici : une épreuve branche une porte à la main, et suit le chemin du téléphone.
//!
//! * **Copier** : le lot (un `.glucose`, fiche 51 § 2) et l'image seule s'écrivent dans `partages/`,
//!   le dossier que le fournisseur de Glucose sert ; le `ClipData` porte **un seul élément**, le
//!   texte et l'adresse de l'image ensemble — une zone de texte qui colle concatène chaque élément,
//!   et y écrirait une adresse `content://` ; son étiquette porte la date du lot.
//! * **Coller** : l'étiquette d'abord, qui reconnaît un lot de Glucose sans lire les données (aucun
//!   message d'Android 12) ; sinon les adresses, qui suivent le chemin d'un partage — lues sur un
//!   fil, posées au point du geste ; sinon le texte.

use super::LotALire;
use crate::interactions::clipboard::Formes;
use crate::plateforme::partage::{self, Partage};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// **Ce que Java sait faire du presse-papiers.**
pub trait Porte: Send + Sync {
    /// Y écrit un seul élément — ce texte et l'adresse de cette image — sous cette étiquette.
    fn ecrire(
        &self,
        etiquette: &str,
        texte: Option<&str>,
        image: Option<&Path>,
    ) -> Result<(), String>;
    /// Son étiquette, sans lire ses données.
    fn etiquette(&self) -> Option<String>;
    /// Ce qu'il porte ; ses adresses ouvertes seulement si `ouvrir`.
    fn lire(&self, ouvrir: bool) -> Option<Lu>;
}

/// **Ce que porte le presse-papiers**, tel que Java l'a lu.
#[derive(Debug, Default)]
pub struct Lu {
    /// Ses adresses, ouvertes : le fichier, son début, sa longueur quand le système la dit —
    /// `None` pour une adresse que le système n'a pas laissé ouvrir.
    pub fichiers: Vec<Option<Ouvert>>,
    /// Son texte, s'il en porte un.
    pub texte: Option<String>,
    /// Le nom de chaque fichier, quand le système le dit (RECU-2).
    pub noms: Vec<Option<String>>,
}

/// Une adresse ouverte : le fichier, son début, sa longueur (celles d'un `AssetFileDescriptor`).
pub type Ouvert = (std::fs::File, u64, Option<u64>);

/// **Ce qu'un collage trouve** dans le presse-papiers d'Android.
#[derive(Debug, PartialEq, Eq)]
pub enum Colle {
    /// Des fichiers : ils sont en route, par le chemin d'un partage.
    Fichiers,
    /// Un texte, à poser comme un texte collé au bureau.
    Texte(String),
    /// Rien qu'un collage sache poser.
    Rien,
}

/// La porte et le dossier où Glucose écrit ce qu'il offre (`partages/`, celui du fournisseur).
type Branche = (Arc<dyn Porte>, PathBuf);

#[cfg(not(test))]
static BRANCHE: std::sync::Mutex<Option<Branche>> = std::sync::Mutex::new(None);

// Les épreuves tournent en parallèle : chacune a sa porte, comme sa feuille de partage.
#[cfg(test)]
thread_local! {
    static BRANCHE: std::cell::RefCell<Option<Branche>> = const { std::cell::RefCell::new(None) };
}

/// **Branche le presse-papiers d'Android** — au lancement, avec le dossier des partages.
pub fn installer(porte: Arc<dyn Porte>, partages: PathBuf) {
    #[cfg(not(test))]
    {
        *BRANCHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((porte, partages));
    }
    #[cfg(test)]
    BRANCHE.with(|b| *b.borrow_mut() = Some((porte, partages)));
}

/// La porte, si elle est branchée — prise hors du verrou : on l'appelle verrou relâché.
fn branche() -> Option<Branche> {
    #[cfg(not(test))]
    {
        BRANCHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
    #[cfg(test)]
    BRANCHE.with(|b| b.borrow().clone())
}

/// Ce que dit un presse-papiers que Java n'a pas branché.
pub const PAS_BRANCHE: &str = "le presse-papiers d'Android n'est pas branche";

/// Ce que dit un presse-papiers qui ne porte aucun texte.
const SANS_TEXTE: &str = "le presse-papiers ne porte aucun texte";

/// Le presse-papiers d'Android est-il là ?
pub fn disponible() -> Result<(), String> {
    branche().map(|_| ()).ok_or_else(|| PAS_BRANCHE.into())
}

/// L'étiquette de ce que Glucose écrit — d'un lot, elle porte sa date.
const ETIQUETTE: &str = "Glucose";

/// Le début du nom de ce que Glucose écrit dans `partages/` pour le presse-papiers.
const COPIE: &str = "presse-papiers-";

fn etiquette_du_lot(date: i64) -> String {
    format!("{ETIQUETTE} {date}")
}

/// La date du lot qu'une étiquette annonce — rien pour celle d'une autre application, ni pour
/// un texte seul copié dans Glucose.
fn date_du_lot(etiquette: &str) -> Option<i64> {
    etiquette
        .strip_prefix(ETIQUETTE)?
        .strip_prefix(' ')?
        .parse()
        .ok()
}

fn fichier_du_lot(partages: &Path, date: i64) -> PathBuf {
    partages.join(format!("{COPIE}{date}.glucose"))
}

/// **Confie la sélection au presse-papiers d'Android.** Une copie remplace la précédente : ses
/// fichiers s'effacent, comme au bureau.
pub fn ecrire_la_selection(formes: Formes) -> Result<(), String> {
    let (porte, partages) = branche().ok_or(PAS_BRANCHE)?;
    std::fs::create_dir_all(&partages).map_err(|e| e.to_string())?;
    oublier_les_copies(&partages);
    let ecrire = |chemin: PathBuf, octets: &[u8]| {
        std::fs::write(&chemin, octets)
            .map(|()| chemin)
            .map_err(|e| e.to_string())
    };
    ecrire(fichier_du_lot(&partages, formes.date), &formes.lot)?;
    let image = (formes.image.as_ref())
        .map(|i| ecrire(partages.join(format!("{COPIE}{}.png", formes.date)), &i.png))
        .transpose()?;
    let etiquette = etiquette_du_lot(formes.date);
    porte.ecrire(&etiquette, formes.texte.as_deref(), image.as_deref())
}

/// Efface les fichiers des copies précédentes.
fn oublier_les_copies(partages: &Path) {
    let Ok(entrees) = std::fs::read_dir(partages) else {
        return;
    };
    for entree in entrees.flatten() {
        if entree.file_name().to_string_lossy().starts_with(COPIE) {
            let _ = std::fs::remove_file(entree.path());
        }
    }
}

/// **Écrit un texte seul** : ce qu'on copie d'un texte qu'on écrit.
pub fn ecrire_le_texte(texte: &str) -> Result<(), String> {
    let (porte, _) = branche().ok_or(PAS_BRANCHE)?;
    porte.ecrire(ETIQUETTE, Some(texte), None)
}

/// Le texte qu'il porte — sans ouvrir ses adresses.
pub fn lire_le_texte() -> Result<String, String> {
    let (porte, _) = branche().ok_or(PAS_BRANCHE)?;
    (porte.lire(false).and_then(|lu| lu.texte)).ok_or_else(|| SANS_TEXTE.into())
}

/// **Le lot que porte le presse-papiers** : celui dont l'étiquette dit la date, et dont le
/// fichier est encore là — lu sur le fil du collage.
pub fn lire_un_lot() -> Option<LotALire> {
    let (porte, partages) = branche()?;
    let chemin = fichier_du_lot(&partages, date_du_lot(&porte.etiquette()?)?);
    chemin
        .is_file()
        .then(|| Box::new(move || std::fs::read(chemin).ok()) as LotALire)
}

/// **Colle ce que porte le presse-papiers**, au point `ou` de la fenêtre : ses fichiers
/// suivent le chemin d'un partage, lus sur un fil (rien de lourd sur celui qui dessine,
/// COLLER-1) ; son texte revient à l'appelant. Rien si la porte n'est pas branchée.
pub fn coller(ou: (f64, f64)) -> Option<Colle> {
    let (porte, _) = branche()?;
    let Some(lu) = porte.lire(true) else {
        return Some(Colle::Rien);
    };
    if lu.fichiers.is_empty() {
        return Some(lu.texte.map_or(Colle::Rien, Colle::Texte));
    }
    let (fichiers, noms) = (lu.fichiers, lu.noms);
    let lance = std::thread::Builder::new()
        .name("collage".into())
        .spawn(move || partage::recevoir(partage_du_collage(fichiers, &noms, ou)));
    if let Err(e) = lance {
        crate::dire_erreur!("[Glucose] collage : {e}");
    }
    Some(Colle::Fichiers)
}

/// **Le partage d'un collage** : ses fichiers lus, chacun sous son nom — un illisible se
/// compte —, son point, et aucun texte (une adresse qui l'accompagne n'irait pas chercher
/// d'image).
fn partage_du_collage(
    fichiers: Vec<Option<Ouvert>>,
    noms: &[Option<String>],
    ou: (f64, f64),
) -> Partage {
    let mut collage = Partage {
        ou: Some(ou),
        ..Partage::default()
    };
    for (i, ouvert) in fichiers.into_iter().enumerate() {
        collage.ajouter(ouvert, noms.get(i).cloned().flatten().as_deref());
    }
    collage
}

#[cfg(test)]
mod tests;
