//! **Les fichiers du journal technique, lisibles par n'importe qui** (fiche 58).
//!
//! Sa remarque, le 09/10, devant le dossier : *« tu l'as appelé boîte noire, il faudrait le
//! renommer, ça peut inquiéter des gens ; et le format doit être lisible, avec un lisez-moi qui
//! raconte les détails exacts de ce qui est envoyé, en colonnes »*. Le dossier s'appelait
//! `boite-noire`, et ses fichiers `session-00000001791477194257-16656.jsonl` : un nombre, et une
//! extension qu'aucun système n'ouvre d'un double clic.
//!
//! * **Le dossier porte le nom que l'utilisateur lit déjà** : `journal-technique`, comme la
//!   question et l'entrée du menu. L'ancien se range dans le nouveau au premier lancement.
//! * **Un fichier porte l'heure de sa session**, à l'heure locale : `2026-10-08 20h51m12
//!   (16656).txt` — et le numéro que le système a donné à Glucose, qui distingue deux Glucose
//!   ouverts la même seconde. Du texte, qu'un double clic ouvre.
//! * **Un `LISEZ-MOI.txt`** dit, colonne par colonne, ce que dit chaque champ de chaque ligne ;
//!   une épreuve vérifie qu'il n'en oublie aucun.
//!
//! Les lignes, elles, ne changent pas : ce sont **exactement** celles qui partent, et le
//! serveur les lit telles quelles.
//!
//! # Une session a une clé, qui ne dépend pas du nom de son fichier
//!
//! Le serveur reconnaît une session par une empreinte, et la liste de ce qui est parti la
//! retient. Elles se tiraient du nom du fichier : renommer les fichiers aurait fait repartir
//! chaque session déjà envoyée sous un autre identifiant. La clé se tire désormais du **début**
//! de la session (sa première ligne) et de son **processus** — et elle vaut, mot pour mot,
//! l'ancien nom ([`cle`]) : rien de ce qui est parti ne repart, et aucune liste n'est à refaire.

use std::path::{Path, PathBuf};

/// Le dossier du journal, dans celui de l'application.
pub const DOSSIER: &str = "journal-technique";
/// Son ancien nom.
const ANCIEN_DOSSIER: &str = "boite-noire";
/// Ce qui explique chaque ligne, posé dans le dossier.
pub const LISEZ_MOI: &str = "LISEZ-MOI.txt";
/// Son texte, qui voyage avec Glucose.
pub const TEXTE_DU_LISEZ_MOI: &str = include_str!("../../assets/journal-technique/LISEZ-MOI.txt");

/// **Le nom du fichier d'une session** : son début à l'heure locale, puis son processus. Là où
/// le système ne dit pas son fuseau, le temps universel, et il le dit.
pub fn nom_de_session(debut_ms: u64, processus: u32) -> String {
    let secondes = (debut_ms / 1000) % 60;
    let date = match crate::plateforme::heure::heure_locale(debut_ms as i64) {
        Some(h) => format!(
            "{:04}-{:02}-{:02} {:02}h{:02}m{secondes:02}",
            h.annee, h.mois, h.jour, h.heure, h.minute
        ),
        None => {
            let (annee, mois, jour) = date_universelle(debut_ms / 86_400_000);
            let minutes = (debut_ms / 60_000) % 1440;
            format!(
                "{annee:04}-{mois:02}-{jour:02} {:02}h{:02}m{secondes:02} UTC",
                minutes / 60,
                minutes % 60
            )
        }
    };
    format!("{date} ({processus}).txt")
}

/// **Le processus d'une session, lu dans le nom de son fichier** — et la preuve que c'en est
/// une : un nom qui commence par une date et finit par `(<nombre>).txt`.
pub fn processus_de(nom: &str) -> Option<u32> {
    let (date, reste) = nom.strip_suffix(").txt")?.rsplit_once(" (")?;
    let commence_par_une_annee =
        date.len() >= 10 && date.as_bytes()[..4].iter().all(u8::is_ascii_digit);
    commence_par_une_annee.then(|| reste.parse().ok()).flatten()
}

/// Ce nom est-il celui d'une session ?
pub fn est_une_session(nom: &str) -> bool {
    processus_de(nom).is_some()
}

/// **Le début d'une session**, lu dans sa première ligne.
pub fn debut_de(texte: &str) -> Option<u64> {
    use glucose_core::persist::tauri::json;
    let premiere = json::lire(texte.lines().next()?).ok()?;
    if premiere.texte("type") != Some("debut") {
        return None;
    }
    premiere
        .entier("epoque_ms")
        .and_then(|n| u64::try_from(n).ok())
}

/// Le début de la session de ce fichier, lu dans sa première ligne seulement.
pub fn debut_du_fichier(p: &Path) -> Option<u64> {
    use std::io::BufRead;
    let mut premiere = String::new();
    std::io::BufReader::new(std::fs::File::open(p).ok()?)
        .read_line(&mut premiere)
        .ok()?;
    debut_de(&premiere)
}

/// **La clé d'une session** : ce dont le serveur tire son identifiant, et ce que la liste de ce
/// qui est parti retient. L'ancien nom du fichier, mot pour mot.
pub fn cle(debut_ms: u64, processus: u32) -> String {
    format!("session-{debut_ms:020}-{processus}.jsonl")
}

/// Le début et le processus d'une session sous son ancien nom.
fn ancien_nom(nom: &str) -> Option<(u64, u32)> {
    let (debut, processus) = nom
        .strip_prefix("session-")?
        .strip_suffix(".jsonl")?
        .split_once('-')?;
    Some((debut.parse().ok()?, processus.parse().ok()?))
}

/// **Prépare le dossier du journal** dans celui de l'application, et le rend : l'ancien dossier
/// s'y range, les anciens noms deviennent lisibles, et le lisez-moi est à jour. Rien ne
/// s'écrase jamais : un nom déjà pris laisse le fichier où il est.
pub fn preparer(dossier: &Path) -> std::io::Result<PathBuf> {
    let journal = dossier.join(DOSSIER);
    ranger_l_ancien_dossier(&dossier.join(ANCIEN_DOSSIER), &journal);
    std::fs::create_dir_all(&journal)?;
    for entree in std::fs::read_dir(&journal)?.flatten() {
        let nom = entree.file_name().to_string_lossy().into_owned();
        if let Some((debut, processus)) = ancien_nom(&nom) {
            let lisible = journal.join(nom_de_session(debut, processus));
            let _ = crate::persist::atomic::renommer_sans_ecraser(&entree.path(), &lisible);
        }
    }
    let lisez_moi = journal.join(LISEZ_MOI);
    if std::fs::read_to_string(&lisez_moi).ok().as_deref() != Some(TEXTE_DU_LISEZ_MOI) {
        crate::persist::atomic::ecrire_d_un_bloc(&lisez_moi, TEXTE_DU_LISEZ_MOI.as_bytes())?;
    }
    Ok(journal)
}

/// L'ancien dossier `boite-noire` devient le nouveau, ou s'y vide s'il existe déjà.
fn ranger_l_ancien_dossier(ancien: &Path, journal: &Path) {
    if !ancien.is_dir() {
        return;
    }
    if crate::persist::atomic::renommer_sans_ecraser(ancien, journal).is_ok() {
        return;
    }
    if let Ok(entrees) = std::fs::read_dir(ancien) {
        for entree in entrees.flatten() {
            let _ = std::fs::create_dir_all(journal);
            let cible = journal.join(entree.file_name());
            let _ = crate::persist::atomic::renommer_sans_ecraser(&entree.path(), &cible);
        }
    }
    // Vide, il part ; s'il garde quelque chose, c'est qu'un nom était déjà pris : il reste.
    let _ = std::fs::remove_dir(ancien);
}

/// **La date du jour `jours` depuis le 1er janvier 1970**, au calendrier grégorien — l'algorithme
/// de Howard Hinnant (`civil_from_days`), exact sur toute l'étendue d'un `u64` de jours utile.
fn date_universelle(jours: u64) -> (u64, u64, u64) {
    let z = jours + 719_468;
    let ere = z / 146_097;
    let jour_de_l_ere = z - ere * 146_097;
    let annee_de_l_ere = (jour_de_l_ere - jour_de_l_ere / 1460 + jour_de_l_ere / 36_524
        - jour_de_l_ere / 146_096)
        / 365;
    let jour_de_l_annee =
        jour_de_l_ere - (365 * annee_de_l_ere + annee_de_l_ere / 4 - annee_de_l_ere / 100);
    let m = (5 * jour_de_l_annee + 2) / 153;
    let jour = jour_de_l_annee - (153 * m + 2) / 5 + 1;
    let mois = if m < 10 { m + 3 } else { m - 9 };
    let annee = annee_de_l_ere + ere * 400 + u64::from(mois <= 2);
    (annee, mois, jour)
}

#[cfg(test)]
mod tests;
