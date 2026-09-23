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

use super::moisson::{self, Depot, Moisson, OCTETS_MAX};
use super::sources::{self, Candidat};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;

/// Le numéro du prochain rapatriement : ce qui relie une livraison à son annonce.
static PROCHAIN: AtomicU64 = AtomicU64::new(1);

/// **Annonce le rapatriement, puis cherche l'image de ces adresses sur un fil à part**, et
/// envoie ce qu'on a trouvé — ou le repli — par le canal des dépôts.
///
/// L'annonce part **avant** le fil, depuis l'instant du lâcher : c'est elle qui fait paraître
/// le marqueur au point de dépôt pendant la seconde qu'il faut — mesurée à 0,8 à 1,4 s sur
/// trois épingles réelles, dont l'essentiel avant le premier octet de l'image.
pub fn rapatrier(
    adresses: Vec<String>,
    repli: Moisson,
    (vers, reveil): (Sender<Depot>, super::Reveil),
) {
    let numero = PROCHAIN.fetch_add(1, Ordering::Relaxed);
    let hote = adresses
        .iter()
        .find_map(|a| sources::decouper(a))
        .map(|a| a.hote)
        .unwrap_or_default();
    let ou = repli.ou;
    if vers.send(Depot::EnChemin { numero, ou, hote }).is_err() {
        return;
    }
    std::thread::spawn(move || {
        let moisson = chercher(&adresses, ou).unwrap_or(repli);
        // Toujours, même vide : c'est ce qui retire l'annonce. La fenêtre a pu se fermer
        // pendant le téléchargement, et ce n'est pas une panne.
        let pose = Depot::Pose {
            numero: Some(numero),
            moisson,
        };
        if vers.send(pose).is_ok() {
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
