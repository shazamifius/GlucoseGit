//! **Rapatrier l'image d'un dépôt qui n'a apporté que des adresses** (DEPOT-WEB-4).
//!
//! Le pont de dépôt récolte ce que le navigateur donne. Quand ce ne sont que des adresses —
//! le cas de Pinterest, dix fois sur dix —, ce module essaie les candidats de
//! [`super::sources`] dans l'ordre, de l'original à la copie, **sur un fil à part** : un
//! téléchargement prend des centaines de millisecondes, et la charte interdit qu'une image
//! l'attende. Ce qu'il trouve rejoint la boucle d'images par le **même canal** que tout
//! dépôt, et s'y pose exactement comme un fichier glissé depuis l'explorateur.
//!
//! S'il ne trouve rien, le dépôt retombe sur ce qu'il portait : la vignette que la page avait
//! posée, ou le lien qu'on peut suivre. Un repli visible vaut mieux qu'un geste sans effet.

use super::moisson::{self, Moisson, OCTETS_MAX};
use super::sources::{self, Candidat};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

/// **Cherche l'image de ces adresses sur un fil à part**, et envoie ce qu'on a trouvé — ou le
/// repli — par le canal des dépôts.
pub fn rapatrier(
    adresses: Vec<String>,
    ou: Option<(f64, f64)>,
    repli: Moisson,
    (vers, reveil): (Sender<Moisson>, super::Reveil),
) {
    std::thread::spawn(move || {
        let moisson = chercher(&adresses, ou).unwrap_or(repli);
        if moisson.est_vide() {
            return;
        }
        // La fenêtre a pu se fermer pendant le téléchargement : ce n'est pas une panne.
        if vers.send(moisson).is_ok() {
            reveil();
        }
    });
}

/// Essaie les candidats dans l'ordre, et rend la première image rapatriée.
///
/// Une page ne se lit qu'une fois, et les images qu'elle annonce passent **devant** les
/// autres pages ; les pages qu'elle cite ne se suivent pas — la recherche est donc finie.
pub fn chercher(adresses: &[String], ou: Option<(f64, f64)>) -> Option<Moisson> {
    let mut file: VecDeque<Candidat> = sources::candidats(adresses).into();
    while let Some(candidat) = file.pop_front() {
        match candidat {
            Candidat::Image(url) => {
                if let Some(chemin) = rapatrier_l_image(&url) {
                    println!("[Glucose] depot : image rapatriee depuis {url}");
                    return Some(Moisson {
                        chemins: vec![chemin],
                        liens: Vec::new(),
                        ou,
                    });
                }
            }
            Candidat::Page(url) => {
                let html = match super::telecharger(&url, OCTETS_MAX) {
                    Ok(octets) => String::from_utf8_lossy(&octets).into_owned(),
                    Err(e) => {
                        dire(&url, &e);
                        continue;
                    }
                };
                let annoncees = sources::candidats(&sources::images_de_la_page(&html));
                for image in annoncees.into_iter().rev() {
                    if matches!(image, Candidat::Image(_)) {
                        file.push_front(image);
                    }
                }
            }
        }
    }
    None
}

/// Télécharge cette adresse, et l'écrit si ce sont bien les octets d'une image.
fn rapatrier_l_image(url: &str) -> Option<PathBuf> {
    let octets = match super::telecharger(url, OCTETS_MAX) {
        Ok(o) => o,
        Err(e) => {
            dire(url, &e);
            return None;
        }
    };
    if !sources::est_une_image(&octets) {
        dire(url, "la reponse n'est pas une image");
        return None;
    }
    let dossier = moisson::dossier().ok()?;
    moisson::poser(&dossier, &sources::nom_pour(url), 0, &octets)
}

/// Ce qu'un essai a donné, quand `GLUCOSE_DEPOT` le demande : l'original de Pinterest manque
/// souvent, et savoir lequel a répondu est ce qui dira si l'ordre des candidats est juste.
fn dire(url: &str, raison: &str) {
    if std::env::var_os("GLUCOSE_DEPOT").is_some() {
        eprintln!("[Glucose] depot : {url} -- {raison}");
    }
}
