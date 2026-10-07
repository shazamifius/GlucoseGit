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
    let (depart, mut pages) = (std::time::Instant::now(), 0);
    while let Some(candidat) = file.pop_front() {
        match candidat {
            Candidat::Image(url) => {
                // Les variantes d'une même image — l'original sous ses extensions, la grande
                // copie, la reçue — se suivent dans la file : elles courent ensemble.
                let mut groupe = vec![url];
                while let Some(Candidat::Image(suivante)) = file.front() {
                    if sources::famille(suivante) != sources::famille(&groupe[0]) {
                        break;
                    }
                    groupe.push(suivante.clone());
                    file.pop_front();
                }
                match course(groupe) {
                    Ok((url, recu)) => {
                        println!(
                            "[Glucose] depot : image rapatriee depuis {url} en {} ms, {pages} page(s) lue(s)",
                            depart.elapsed().as_millis()
                        );
                        return Ok(Moisson {
                            recus: vec![recu],
                            ou,
                            ..Moisson::default()
                        });
                    }
                    Err((url, e)) => echec = (hote(&url), e),
                }
            }
            Candidat::Page(url) => {
                pages += 1;
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

/// **Les variantes d'une image courent ensemble**, et la meilleure qui répond gagne
/// (fiche 53 § 7).
///
/// Essayées l'une après l'autre, chaque variante absente — l'original en `.jpg` quand il est en
/// `.png` — coûtait un aller-retour entier avant la suivante. Toutes partent à la fois ; le rang
/// `i` gagne dès qu'il a répondu une image **et** que tous ceux qui le précèdent ont échoué :
/// l'ordre de qualité reste celui de [`sources::candidats`], seul l'attente change. Les
/// perdantes finissent seules, et leur réponse se perd.
fn course(groupe: Vec<String>) -> Result<(String, Recu), (String, String)> {
    let (envoi, recu) = std::sync::mpsc::channel();
    for (rang, url) in groupe.iter().cloned().enumerate() {
        let envoi = envoi.clone();
        std::thread::spawn(move || {
            let _ = envoi.send((rang, rapatrier_l_image(&url)));
        });
    }
    drop(envoi);
    let mut etats: Vec<Option<Result<Recu, String>>> = groupe.iter().map(|_| None).collect();
    while let Ok((rang, issue)) = recu.recv() {
        etats[rang] = Some(issue);
        match vainqueur(&etats) {
            Issue::Attendre => {}
            Issue::Gagne(i) => {
                let Some(Ok(recu)) = etats[i].take() else {
                    unreachable!("le vainqueur a répondu une image")
                };
                return Ok((groupe[i].clone(), recu));
            }
            Issue::Perdu => break,
        }
    }
    // Toutes ont échoué : la raison de la dernière, la moins exigeante — la copie reçue.
    let derniere = etats
        .iter()
        .rposition(|e| matches!(e, Some(Err(_))))
        .unwrap_or(0);
    let raison = match etats.get_mut(derniere).and_then(Option::take) {
        Some(Err(e)) => e,
        _ => "aucune reponse".into(),
    };
    Err((groupe[derniere].clone(), raison))
}

/// Ce qu'une course sait, au point où elle en est.
#[derive(Debug, PartialEq, Eq)]
enum Issue {
    /// Ce rang a répondu une image, et tous ceux qui le précèdent ont échoué.
    Gagne(usize),
    /// Tous ont échoué.
    Perdu,
    /// Un rang meilleur que toute réponse reçue n'a pas encore répondu.
    Attendre,
}

/// **Qui gagne**, à ce point de la course : le premier rang qui n'a pas échoué décide.
fn vainqueur<T, E>(etats: &[Option<Result<T, E>>]) -> Issue {
    for (rang, etat) in etats.iter().enumerate() {
        match etat {
            None => return Issue::Attendre,
            Some(Ok(_)) => return Issue::Gagne(rang),
            Some(Err(_)) => {}
        }
    }
    Issue::Perdu
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

#[cfg(test)]
mod tests {
    use super::{vainqueur, Issue};

    /// **La meilleure variante gagne dès qu'elle le peut, et jamais avant** : une copie qui
    /// répond pendant que l'original n'a rien dit attend ; l'original qui répond gagne sans
    /// attendre les autres ; un original absent cède à la suivante.
    #[test]
    fn test_la_course_rend_la_meilleure_variante_des_qu_elle_le_peut() {
        let ok = || Some(Ok::<(), ()>(()));
        let ko = || Some(Err::<(), ()>(()));
        assert_eq!(vainqueur(&[None, ok(), ok()]), Issue::Attendre);
        assert_eq!(vainqueur(&[ok(), None, None]), Issue::Gagne(0));
        assert_eq!(vainqueur(&[ko(), ko(), ok(), None]), Issue::Gagne(2));
        assert_eq!(vainqueur(&[ko(), None, ok()]), Issue::Attendre);
        assert_eq!(vainqueur(&[ko(), ko()]), Issue::Perdu);
    }
}
