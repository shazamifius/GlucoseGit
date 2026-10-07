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
//!
//! # Un repli qui se dit, et qui se rattrape (DEPOT-WEB-6)
//!
//! Le 07/10, six épingles glissées depuis Pinterest sont devenues six liens, nés à seize
//! secondes d'intervalle — le délai d'une étape, épuisé, puis le suivant. Les mêmes épingles se
//! rapatriaient en une seconde une heure plus tard : c'était le réseau de ce moment-là, pas
//! Pinterest. Mais rien ne l'avait dit, et un lien posé était une impasse.
//!
//! L'échec porte donc sa raison jusqu'au compte-rendu ([`Moisson::echec`]) et jusqu'à la
//! sortie, et [`relancer`] reprend la recherche pour un lien déjà posé — le clic droit
//! « Remplacer par l'image » de [`crate::interactions::depot_web`].

use super::moisson::{Depot, Moisson, Recu, OCTETS_MAX};
use super::sources::{self, Candidat};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;

/// Le numéro du prochain rapatriement : ce qui relie une livraison à son annonce.
static PROCHAIN: AtomicU64 = AtomicU64::new(1);

/// Le numéro du prochain rapatriement, qu'une annonce et sa livraison partagent.
pub fn numero_suivant() -> u64 {
    PROCHAIN.fetch_add(1, Ordering::Relaxed)
}

/// **Annonce le rapatriement, puis cherche l'image de ces adresses sur un fil à part**, et
/// envoie ce qu'on a trouvé — ou le repli, avec la raison de s'y résoudre — par le canal des
/// dépôts.
///
/// L'annonce part **avant** le fil, depuis l'instant du lâcher : c'est elle qui fait paraître
/// le marqueur au point de dépôt pendant la seconde qu'il faut — mesurée à 0,8 à 1,4 s sur
/// trois épingles réelles, dont l'essentiel avant le premier octet de l'image.
pub fn rapatrier(
    adresses: Vec<String>,
    repli: Moisson,
    (vers, reveil): (Sender<Depot>, super::Reveil),
) {
    let numero = numero_suivant();
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
        let moisson = chercher(&adresses, ou).unwrap_or_else(|raison| Moisson {
            echec: Some(raison),
            ..repli
        });
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

/// Une recherche d'image : des adresses, et l'image rapatriée ou la raison de n'en pas avoir.
pub type Recherche = fn(&[String]) -> Result<Moisson, String>;

/// **Reprend la recherche de l'image d'un lien déjà posé**, sur un fil à part, et la livre par
/// `vers` sous ce `numero` — l'image, ou une moisson vide qui dit pourquoi (DEPOT-WEB-6).
///
/// La recherche est **reçue** plutôt qu'appelée : une épreuve passe la sienne, et aucune
/// n'atteint jamais le réseau.
pub fn relancer(
    adresse: String,
    numero: u64,
    recherche: Recherche,
    (vers, reveil): (Sender<Depot>, Option<super::Reveil>),
) {
    std::thread::spawn(move || {
        let moisson = recherche(&[adresse]).unwrap_or_else(|raison| Moisson {
            echec: Some(raison),
            ..Moisson::default()
        });
        let pose = Depot::Pose {
            numero: Some(numero),
            moisson,
        };
        if vers.send(pose).is_ok() {
            if let Some(reveil) = reveil {
                reveil();
            }
        }
    });
}

/// La recherche du réseau, sans point de dépôt : un lien relancé se remplace là où il est.
pub fn chercher_sur_le_reseau(adresses: &[String]) -> Result<Moisson, String> {
    chercher(adresses, None)
}

/// **Essaie les candidats dans l'ordre, et rend la première image rapatriée** — ou la raison du
/// dernier échec, précédée du site : c'est elle que l'utilisateur lira.
///
/// Une page ne se lit qu'une fois, et les images qu'elle annonce passent **devant** les
/// autres pages ; les pages qu'elle cite ne se suivent pas — la recherche est donc finie.
pub fn chercher(adresses: &[String], ou: Option<(f64, f64)>) -> Result<Moisson, String> {
    let mut file: VecDeque<Candidat> = sources::candidats(adresses).into();
    let mut echec = (String::new(), String::from("aucune adresse a essayer"));
    while let Some(candidat) = file.pop_front() {
        match candidat {
            Candidat::Image(url) => match rapatrier_l_image(&url) {
                Ok(recu) => {
                    println!("[Glucose] depot : image rapatriee depuis {url}");
                    return Ok(Moisson {
                        recus: vec![recu],
                        ou,
                        ..Moisson::default()
                    });
                }
                Err(e) => echec = (hote(&url), e),
            },
            Candidat::Page(url) => {
                let html = match super::telecharger(&url, OCTETS_MAX) {
                    Ok(octets) => String::from_utf8_lossy(&octets).into_owned(),
                    Err(e) => {
                        dire(&url, &e);
                        echec = (hote(&url), e);
                        continue;
                    }
                };
                let annoncees = sources::candidats(&sources::images_de_la_page(&html));
                if annoncees.is_empty() {
                    echec = (hote(&url), "la page n'annonce aucune image".into());
                }
                for image in annoncees.into_iter().rev() {
                    if matches!(image, Candidat::Image(_)) {
                        file.push_front(image);
                    }
                }
            }
        }
    }
    let raison = match echec {
        (site, raison) if site.is_empty() => raison,
        (site, raison) => format!("{site} : {raison}"),
    };
    // **Toujours dit**, et plus seulement sous l'instrument : c'est la ligne qui aurait dit, le
    // 07/10, que le réseau ne répondait pas.
    eprintln!("[Glucose] depot : aucune image rapatriee -- {raison}");
    Err(raison)
}

/// Le site d'une adresse, ce qu'un message peut nommer.
fn hote(url: &str) -> String {
    sources::decouper(url).map(|a| a.hote).unwrap_or_default()
}

/// Télécharge cette adresse, et la rend si ce sont bien les octets d'une image — en mémoire :
/// rien ne passe plus par le dossier temporaire (DEPOT-4).
fn rapatrier_l_image(url: &str) -> Result<Recu, String> {
    let octets = super::telecharger(url, OCTETS_MAX).inspect_err(|e| dire(url, e))?;
    if !sources::est_une_image(&octets) {
        dire(url, "la reponse n'est pas une image");
        return Err("la reponse n'est pas une image".into());
    }
    Recu::nouveau(&sources::nom_pour(url), octets).ok_or_else(|| "une image vide".into())
}

/// Ce qu'un essai a donné, quand `GLUCOSE_DEPOT` le demande : l'original de Pinterest manque
/// souvent, et savoir lequel a répondu est ce qui dira si l'ordre des candidats est juste.
fn dire(url: &str, raison: &str) {
    if std::env::var_os("GLUCOSE_DEPOT").is_some() {
        eprintln!("[Glucose] depot : {url} -- {raison}");
    }
}
