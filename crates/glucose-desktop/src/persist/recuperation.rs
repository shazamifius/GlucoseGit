//! **Ce qui ne se détruit jamais** : ce qu'on va recouvrir est d'abord mis de côté (FIN-1).
//!
//! # Le défaut
//!
//! À l'ouverture, la lecture d'un document s'arrête à la première entrée qui ne suit pas la
//! chaîne ([`glucose_core::persist::histoire`]) ; le scribe tronque ensuite le fichier à cet
//! endroit, pour que l'histoire reprenne à la suite de ce qui est intact. Pour une fin déchirée
//! par un plantage — quelques kilo-octets —, c'est juste. Mais une seule entrée abîmée **au
//! milieu** d'un document de deux cents mégaoctets — un secteur du disque, un défaut d'écriture —
//! rendrait « ignoré » tout ce qui la suit, et la troncature le détruisait : des heures de travail,
//! pour un toast qui disait « la fin d'un enregistrement interrompu ignorée ».
//!
//! # La règle
//!
//! Aucune constante ne sépare « une fin déchirée » d'« une histoire coupée » : on ne tranche pas,
//! on garde. Les octets qu'on va recouvrir se copient d'abord ici, se poussent sur le disque, et
//! alors seulement le fichier se tronque. Le fichier d'avant se reconstitue exactement : ses
//! `debut` premiers octets — que l'ajout seul ne touche plus jamais —, puis ce qui a été mis de
//! côté. Une copie qui échoue refuse la troncature : le document s'ouvre sans être modifié, et ce
//! qu'on y change part dans un brouillon.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Le dossier de ce qui a été mis de côté, dans celui des brouillons : le travail à reprendre.
pub const DOSSIER: &str = "recuperation";

/// **Met de côté les octets `debut..fin` de ce document** avant qu'on les recouvre, et rend où.
///
/// Le nom porte le document, l'endroit de la coupure et l'instant : `abime-81234-1790….fin` se
/// recolle derrière les 81 234 premiers octets d'`abime.glucose`.
pub fn mettre_de_cote(
    fichier: &mut File,
    document: &Path,
    (debut, fin): (u64, u64),
    brouillons: &Path,
) -> std::io::Result<PathBuf> {
    let dossier = brouillons.join(DOSSIER);
    std::fs::create_dir_all(&dossier)?;
    let nom = document
        .file_stem()
        .map_or_else(|| "document".into(), |n| n.to_string_lossy());
    let chemin = dossier.join(format!("{nom}-{debut}-{}.fin", super::now_millis()));
    let mut cote = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&chemin)?;
    let copie = copier(fichier, (debut, fin), &mut cote);
    if copie.is_err() {
        // Un morceau mis de côté ferait croire à tout : il ne reste pas.
        drop(cote);
        let _ = std::fs::remove_file(&chemin);
    }
    copie.map(|()| chemin)
}

/// **Range un fichier parmi ce qui a été mis de côté** — une saisie qui ne peut plus être
/// rendue là où elle attendait, ou qui ne se relit plus —, sous un nom neuf. Il est copié entier
/// et poussé sur le disque avant que l'original ne parte : jamais effacé sans avoir été rangé.
pub fn ranger(fichier: &Path, brouillons: &Path) -> std::io::Result<PathBuf> {
    let dossier = brouillons.join(DOSSIER);
    std::fs::create_dir_all(&dossier)?;
    let nom = fichier
        .file_name()
        .map_or_else(|| "fichier".into(), |n| n.to_string_lossy());
    let chemin = dossier.join(format!("{}-{nom}", super::now_millis()));
    super::atomic::copier_d_un_bloc(fichier, &chemin)?;
    std::fs::remove_file(fichier)?;
    Ok(chemin)
}

/// Copie `debut..fin`, en entier, jusqu'au disque. Un fichier plus court que promis — qu'un
/// autre programme aurait tronqué entre-temps, ce qu'aucun verrou n'empêche hors de Windows —
/// est une erreur : mettre de côté la moitié d'une fin, puis tronquer, en perdrait l'autre.
fn copier(fichier: &mut File, (debut, fin): (u64, u64), cote: &mut File) -> std::io::Result<()> {
    fichier.seek(SeekFrom::Start(debut))?;
    let copies = std::io::copy(&mut fichier.by_ref().take(fin - debut), cote)?;
    if copies != fin - debut {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            format!("{copies} octets lus sur {}", fin - debut),
        ));
    }
    cote.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Une fin plus courte que promis ne se met pas de côté à moitié** : l'échec se dit, et
    /// aucun morceau ne reste dans le dossier.
    #[test]
    fn test_une_fin_plus_courte_que_promis_n_est_pas_mise_de_cote_a_moitie() {
        let d = std::env::temp_dir().join(format!("glucose-recuperation-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("dossier d'épreuve");
        let doc = d.join("court.glucose");
        std::fs::write(&doc, b"0123456789").expect("document d'épreuve");
        let mut f = File::open(&doc).expect("ouverture");
        assert!(mettre_de_cote(&mut f, &doc, (5, 20), &d).is_err());
        let restes = std::fs::read_dir(d.join(DOSSIER))
            .expect("le dossier existe")
            .count();
        assert_eq!(restes, 0, "aucun morceau ne reste");
        let cote = mettre_de_cote(&mut f, &doc, (5, 10), &d).expect("une fin entière");
        assert_eq!(std::fs::read(cote).expect("relecture"), b"56789");
        let _ = std::fs::remove_dir_all(&d);
    }
}
