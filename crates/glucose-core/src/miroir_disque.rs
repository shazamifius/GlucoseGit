//! **Les dossiers miroirs du disque** (DISQUE-1, fiche 164) : ce qu'un niveau d'un dossier de son
//! PC devient dans Glucose — la loi pure.
//!
//! Le noyau ne lit aucun disque : l'application lit un niveau (son nom, sa taille, sa date, ce que
//! chaque entrée est) et le donne ici. Le noyau dit ce qui se tait, dans quel ordre le reste se
//! range, et la source d'un sous-dossier qu'on lira quand on y entrera — un niveau à la fois,
//! comme Glucose Tauri (`folderMirror.ts`) : un dossier de dix mille sous-dossiers se pose aussi
//! vite qu'un dossier vide.

use crate::recherche::pli;
use crate::types::{BoardImage, FolderMirrorSource, FolderSortMode, Project};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::iter::Peekable;
use std::str::Chars;

/// **Ce qu'une entrée du disque devient.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sorte {
    /// Un sous-dossier : un dossier de Glucose, lu quand on y entre.
    Dossier,
    /// Une photo, ses dimensions lues dans son en-tête : liée par son chemin, jamais scellée dans
    /// le document — un dossier de deux mille photos ne doit pas peser deux mille photos.
    Photo { largeur: u32, hauteur: u32 },
    /// Un fichier que le noyau sait lire ([`crate::file_kind::readable`]) : un texte.
    Lisible,
    /// Le reste : une tuile qui révèle le fichier dans l'explorateur, sans jamais le lancer.
    Autre,
}

/// **Une entrée d'un niveau**, telle que l'application l'a lue.
#[derive(Debug, Clone, PartialEq)]
pub struct Entree {
    pub nom: String,
    pub chemin: String,
    pub sorte: Sorte,
    /// En octets ; zéro pour un dossier.
    pub taille: u64,
    /// La dernière modification, en secondes depuis 1970.
    pub date: i64,
}

/// Les binaires du système, qui n'ont rien à montrer (Glucose Tauri : `SCAN_HIDE_EXTS`).
const BINAIRES: &[&str] = &[
    "bin", "com", "cpl", "dll", "drv", "exe", "lnk", "msi", "msp", "ocx", "pif", "scr", "sys",
    "tmp",
];

/// Les miettes que l'explorateur de Windows laisse dans chaque dossier.
const MIETTES: &[&str] = &["Thumbs.db", "desktop.ini"];

/// **Ce que le miroir tait.**
///
/// * **Tout ce qui commence par un point** — la convention des systèmes pour « caché ». Glucose
///   Tauri montrait `.env` : c'est là que vivent les mots de passe d'un projet, et un canevas se
///   partage, et des IA le lisent. Rien de caché n'y entre.
/// * Les miettes de l'explorateur ([`MIETTES`]) et les binaires ([`BINAIRES`]) : du bruit.
pub fn masquee(nom: &str, est_un_dossier: bool) -> bool {
    if nom.starts_with('.') || MIETTES.contains(&nom) {
        return true;
    }
    !est_un_dossier && BINAIRES.contains(&crate::file_kind::extension(nom).as_str())
}

/// **Range un niveau** : les dossiers d'abord, comme l'explorateur, puis selon le tri du dossier
/// (par nom si aucun). Deux entrées égales pour ce tri se départagent par leur nom.
pub fn trier(entrees: &mut [Entree], mode: Option<FolderSortMode>) {
    let mode = mode.unwrap_or(FolderSortMode::NameAsc);
    entrees.sort_by(|a, b| {
        let fichier = |e: &Entree| e.sorte != Sorte::Dossier;
        fichier(a).cmp(&fichier(b)).then_with(|| selon(a, b, mode))
    });
}

fn selon(a: &Entree, b: &Entree, mode: FolderSortMode) -> Ordering {
    let noms = || comparer_les_noms(&a.nom, &b.nom);
    let ext = |e: &Entree| crate::file_kind::extension(&e.nom);
    match mode {
        FolderSortMode::NameAsc => noms(),
        FolderSortMode::NameDesc => noms().reverse(),
        FolderSortMode::Type => ext(a).cmp(&ext(b)).then_with(noms),
        FolderSortMode::SizeDesc => b.taille.cmp(&a.taille).then_with(noms),
        FolderSortMode::SizeAsc => a.taille.cmp(&b.taille).then_with(noms),
        FolderSortMode::ModifiedDesc => b.date.cmp(&a.date).then_with(noms),
        FolderSortMode::ModifiedAsc => a.date.cmp(&b.date).then_with(noms),
    }
}

/// **L'ordre de deux noms, comme un explorateur** : sans majuscules ni accents (le pli de la
/// recherche, [`pli`] — « Été » vient avant « zèbre »), et les nombres pour leur valeur —
/// « photo 2 » avant « photo 10 ». Deux noms égaux ainsi se départagent tels qu'ils s'écrivent :
/// l'ordre est total, un tri le rend toujours le même.
pub fn comparer_les_noms(a: &str, b: &str) -> Ordering {
    let (pa, pb) = (pli(a), pli(b));
    let (mut x, mut y) = (pa.chars().peekable(), pb.chars().peekable());
    loop {
        let o = match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return a.cmp(b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                comparer_les_nombres(&nombre(&mut x), &nombre(&mut y))
            }
            (Some(c), Some(d)) => {
                x.next();
                y.next();
                c.cmp(&d)
            }
        };
        if o != Ordering::Equal {
            return o;
        }
    }
}

/// Les chiffres qui suivent, consommés.
fn nombre(it: &mut Peekable<Chars<'_>>) -> String {
    let mut s = String::new();
    while let Some(c) = it.next_if(char::is_ascii_digit) {
        s.push(c);
    }
    s
}

/// Deux nombres écrits en chiffres, par leur valeur, quelle que soit leur longueur : sans les
/// zéros de tête, le plus long est le plus grand ; à longueur égale, l'ordre des chiffres.
fn comparer_les_nombres(m: &str, n: &str) -> Ordering {
    let (m, n) = (m.trim_start_matches('0'), n.trim_start_matches('0'));
    m.len().cmp(&n.len()).then_with(|| m.cmp(n))
}

/// **La source d'un dossier du disque qu'on n'a pas encore lu** : il le sera quand on y entrera.
/// « snapshot » : le miroir est lu à l'entrée ; le vivant (DISQUE-2) le relira.
pub fn source_en_attente(chemin: &str, tri: Option<FolderSortMode>) -> FolderMirrorSource {
    FolderMirrorSource {
        root_path: chemin.to_string(),
        mode: "snapshot".to_string(),
        last_scanned_at: 0,
        pattern: None,
        recursive: true,
        sort_by: tri,
        pending_scan: true,
    }
}

/// **Les tableaux des miroirs** : ce qu'ils montrent reste sur le disque, et ne se scelle pas dans
/// le document — un dossier de deux mille photos ne doit pas en peser deux mille. Le signe est le
/// lieu, pas la photo : une photo qui sort d'un miroir (glissée, copiée sur son canevas) devient
/// sienne, et se scelle ; `AssetRef::Link` ne le dit pas, qui désigne aussi les images d'un document
/// portable de Tauri — à sceller, elles.
pub fn tableaux_du_disque(projet: &Project) -> HashSet<&str> {
    projet
        .boards
        .iter()
        .flat_map(|b| &b.folders)
        .filter(|f| f.mirror_source.is_some())
        .map(|f| f.child_board_id.as_str())
        .collect()
}

/// **Les photos du projet qui se scellent** : toutes, sauf celles qui vivent dans le tableau d'un
/// miroir ([`tableaux_du_disque`]).
pub fn images_hors_du_disque(projet: &Project) -> impl Iterator<Item = &BoardImage> {
    let disque = tableaux_du_disque(projet);
    projet
        .boards
        .iter()
        .filter(move |b| !disque.contains(b.id.as_str()))
        .flat_map(|b| b.images.iter())
}

#[cfg(test)]
mod tests;
