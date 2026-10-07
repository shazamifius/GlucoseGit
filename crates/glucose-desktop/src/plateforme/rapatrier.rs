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
        let livrer = |depot: Depot| {
            if vers.send(depot).is_ok() {
                reveil();
            }
        };
        // **La première copie qui arrive se montre tout de suite** (fiche 53 § 9) : l'original
        // d'une épingle peut mettre douze secondes quand une copie en met une demie.
        let mut montrer = |recu: Recu| {
            livrer(Depot::Pose {
                numero: Some(numero),
                moisson: Moisson {
                    recus: vec![recu],
                    ou,
                    apercu: true,
                    ..Moisson::default()
                },
            });
        };
        // Toujours une livraison, même vide : c'est ce qui retire l'annonce. La fenêtre a pu
        // se fermer pendant le téléchargement, et ce n'est pas une panne.
        match chercher_par_etapes(&adresses, Some(&mut montrer)) {
            Ok(Arrivee::Nouvelle(recu)) => livrer(Depot::Pose {
                numero: Some(numero),
                moisson: Moisson {
                    recus: vec![recu],
                    ou,
                    ..Moisson::default()
                },
            }),
            Ok(Arrivee::Meilleure(recu)) => livrer(Depot::Ameliore {
                numero,
                recu: Some(recu),
            }),
            Ok(Arrivee::DejaMontree) => livrer(Depot::Ameliore { numero, recu: None }),
            Err(raison) => livrer(Depot::Pose {
                numero: Some(numero),
                moisson: Moisson {
                    echec: Some(raison),
                    ..repli
                },
            }),
        }
    });
}

/// **Ce qu'une recherche par étapes a rapporté.**
pub enum Arrivee {
    /// L'image, et rien n'en avait été montré.
    Nouvelle(Recu),
    /// Une copie a déjà été montrée ; voici l'original, qui prend sa place.
    Meilleure(Recu),
    /// La copie montrée était déjà la meilleure qui existe.
    DejaMontree,
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

/// **La meilleure image de ces adresses**, d'un seul tenant — sans rien montrer avant : un lien
/// qu'on relance, un banc.
pub fn chercher(adresses: &[String], ou: Option<(f64, f64)>) -> Result<Moisson, String> {
    match chercher_par_etapes(adresses, None)? {
        Arrivee::Nouvelle(recu) | Arrivee::Meilleure(recu) => Ok(Moisson {
            recus: vec![recu],
            ou,
            ..Moisson::default()
        }),
        Arrivee::DejaMontree => unreachable!("rien n'a été montré"),
    }
}

/// **Essaie les candidats dans l'ordre, et rend la meilleure image rapatriée** — ou la raison
/// du dernier échec, précédée du site : c'est elle que l'utilisateur lira. Avec `montrer`, la
/// première copie qui arrive pendant que l'original se fait attendre lui est confiée.
///
/// Une page ne se lit qu'une fois, et les images qu'elle annonce passent **devant** les
/// autres pages ; les pages qu'elle cite ne se suivent pas — la recherche est donc finie.
pub fn chercher_par_etapes<'m>(
    adresses: &[String],
    mut montrer: Option<&mut (dyn FnMut(Recu) + 'm)>,
) -> Result<Arrivee, String> {
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
                match course(groupe, montrer.as_deref_mut()) {
                    Ok((url, arrivee)) => {
                        println!(
                            "[Glucose] depot : image rapatriee depuis {url} en {} ms, {pages} page(s) lue(s)",
                            depart.elapsed().as_millis()
                        );
                        return Ok(arrivee);
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
/// `.png` — coûtait un aller-retour entier avant la suivante. Toutes partent à la fois ; ce que
/// [`Course`] décide à chaque réponse fait le reste. Les perdantes finissent seules, et leur
/// réponse se perd.
fn course<'m>(
    groupe: Vec<String>,
    mut montrer: Option<&mut (dyn FnMut(Recu) + 'm)>,
) -> Result<(String, Arrivee), (String, String)> {
    let (envoi, recu) = std::sync::mpsc::channel();
    let partir = |rangs: std::ops::Range<usize>| {
        for rang in rangs {
            let (envoi, url) = (envoi.clone(), groupe[rang].clone());
            std::thread::spawn(move || {
                let _ = envoi.send((rang, rapatrier_l_image(&url)));
            });
        }
    };
    // **Quand une copie peut se montrer, elle part seule d'abord** (fiche 53 § 9) : la copie
    // reçue — la dernière variante, la plus légère — mettait 1,9 à 3,6 s à arriver quand
    // l'original de 2 Mo lui disputait la bande passante, contre 0,4 s seule. Les autres
    // partent dès qu'elle a répondu.
    let tout = 0..groupe.len();
    let mut en_attente = match montrer {
        Some(_) if groupe.len() > 1 => {
            partir(groupe.len() - 1..groupe.len());
            Some(0..groupe.len() - 1)
        }
        _ => {
            partir(tout);
            None
        }
    };
    let mut course = Course::nouvelle(groupe.len(), montrer.is_some());
    while let Ok((rang, issue)) = recu.recv() {
        if let Some(reste) = en_attente.take() {
            partir(reste);
        }
        match course.recevoir(rang, issue) {
            Decision::Attendre => {}
            Decision::Montrer(copie) => {
                if let Some(montrer) = montrer.as_deref_mut() {
                    montrer(copie);
                }
            }
            Decision::Gagne(i, arrivee) => return Ok((groupe[i].clone(), arrivee)),
            Decision::Perdu(i, raison) => return Err((groupe[i].clone(), raison)),
        }
    }
    drop(envoi);
    Err((groupe[0].clone(), "aucune reponse".into()))
}

/// **Ce que sait une course, réponse après réponse** — pure, éprouvée sans réseau.
struct Course {
    etats: Vec<Option<Result<Recu, String>>>,
    /// Peut-on montrer une copie en attendant ? Et laquelle l'a été.
    montrer: bool,
    montree: Option<usize>,
}

/// Ce que la course décide après une réponse.
enum Decision {
    /// Un rang meilleur que toute réponse reçue n'a pas encore répondu.
    Attendre,
    /// Une copie est arrivée, des meilleures se font attendre : la montrer.
    Montrer(Recu),
    /// Ce rang a gagné : tous ceux qui le précèdent ont échoué.
    Gagne(usize, Arrivee),
    /// Tous ont échoué ; ce rang a la raison la plus parlante, celle de la copie reçue.
    Perdu(usize, String),
}

impl Course {
    fn nouvelle(rangs: usize, montrer: bool) -> Self {
        Self {
            etats: (0..rangs).map(|_| None).collect(),
            montrer,
            montree: None,
        }
    }

    fn recevoir(&mut self, rang: usize, issue: Result<Recu, String>) -> Decision {
        self.etats[rang] = Some(issue);
        match vainqueur(&self.etats) {
            Issue::Gagne(i) if self.montree == Some(i) => Decision::Gagne(i, Arrivee::DejaMontree),
            Issue::Gagne(i) => {
                let Some(Ok(recu)) = self.etats[i].take() else {
                    unreachable!("le vainqueur a répondu une image")
                };
                let arrivee = if self.montree.is_some() {
                    Arrivee::Meilleure(recu)
                } else {
                    Arrivee::Nouvelle(recu)
                };
                Decision::Gagne(i, arrivee)
            }
            Issue::Attendre if self.montrer && self.montree.is_none() => {
                match self.etats.iter().position(|e| matches!(e, Some(Ok(_)))) {
                    Some(copie) => {
                        self.montree = Some(copie);
                        match &self.etats[copie] {
                            Some(Ok(recu)) => Decision::Montrer(recu.clone()),
                            _ => Decision::Attendre,
                        }
                    }
                    None => Decision::Attendre,
                }
            }
            Issue::Attendre => Decision::Attendre,
            Issue::Perdu => {
                let derniere = self
                    .etats
                    .iter()
                    .rposition(|e| matches!(e, Some(Err(_))))
                    .unwrap_or(0);
                let raison = match self.etats[derniere].take() {
                    Some(Err(e)) => e,
                    _ => "aucune reponse".into(),
                };
                Decision::Perdu(derniere, raison)
            }
        }
    }
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
    use super::{vainqueur, Arrivee, Course, Decision, Issue, Recu};

    fn image(nom: &str) -> Result<Recu, String> {
        Recu::nouveau(nom, vec![1, 2, 3]).ok_or_else(|| "vide".into())
    }

    fn nom(decision: &Decision) -> String {
        match decision {
            Decision::Attendre => "attendre".into(),
            Decision::Montrer(r) => format!("montrer {}", r.nom),
            Decision::Gagne(i, Arrivee::Nouvelle(r)) => format!("gagne {i} nouvelle {}", r.nom),
            Decision::Gagne(i, Arrivee::Meilleure(r)) => format!("gagne {i} meilleure {}", r.nom),
            Decision::Gagne(i, Arrivee::DejaMontree) => format!("gagne {i} deja montree"),
            Decision::Perdu(i, raison) => format!("perdu {i} {raison}"),
        }
    }

    /// **La copie se montre tout de suite, et l'original la remplace** (fiche 53 § 9) : la
    /// copie reçue arrive la première pendant que l'original se fait attendre — elle se
    /// montre ; l'original absent sous `.jpg` échoue, celui en `.png` arrive — il gagne,
    /// comme « meilleure ».
    #[test]
    fn test_la_copie_se_montre_puis_l_original_la_remplace() {
        let mut c = Course::nouvelle(3, true);
        assert_eq!(nom(&c.recevoir(2, image("236x.jpg"))), "montrer 236x.jpg");
        assert_eq!(nom(&c.recevoir(0, Err("403".into()))), "attendre");
        assert_eq!(
            nom(&c.recevoir(1, image("o.png"))),
            "gagne 1 meilleure o.png"
        );
    }

    /// **Quand la copie montrée était la meilleure, rien ne la remplace** ; et sans droit de
    /// montrer, aucune copie ne se montre — le vainqueur arrive d'un tenant.
    #[test]
    fn test_la_copie_deja_meilleure_reste_et_sans_droit_rien_ne_se_montre() {
        let mut c = Course::nouvelle(2, true);
        assert_eq!(nom(&c.recevoir(1, image("736x.jpg"))), "montrer 736x.jpg");
        assert_eq!(
            nom(&c.recevoir(0, Err("403".into()))),
            "gagne 1 deja montree"
        );

        let mut c = Course::nouvelle(2, false);
        assert_eq!(nom(&c.recevoir(1, image("736x.jpg"))), "attendre");
        assert_eq!(
            nom(&c.recevoir(0, Err("403".into()))),
            "gagne 1 nouvelle 736x.jpg"
        );
    }

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
