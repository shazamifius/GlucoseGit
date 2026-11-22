//! **La discussion** (fiche 129), sa demande du 21/11 : *« n'oublie pas de créer une fenêtre de
//! chat qui demande [à] utiliser le MCP »*. Le fil qui parle à Gemini, sur la porte réseau commune
//! — WinHTTP sous Windows, `ureq` ailleurs : elle répond aussi sur son PC de l'école.
//!
//! # DISCUSSION-1 — une IA du nuage comme les autres
//!
//! Gemini lit et écrit par **les outils de la porte**, à la portée d'une IA du nuage : le document
//! ouvert seulement (C1, fiche 116) — ce qu'il lit part chez Google ; ni les autres fichiers ni le
//! disque n'y vont. Chaque appel entre par **la boîte des agents**, comme ceux de Claude.ai, de
//! Mistral et du terminal : la boucle l'exécute en un geste signé du nom du modèle, annulable par
//! `Ctrl+Z`, et le témoin dit l'onglet. Rien de l'exécution n'est réécrit ici.
//!
//! # DISCUSSION-2 — l'histoire reste cohérente, quoi qu'il arrive
//!
//! Gemini refuse une histoire où un appel d'outil n'a pas sa réponse. Une question s'arrête
//! (arrêt demandé, refus, réseau) seulement en haut de la boucle, quand chaque appel a la sienne ;
//! la question suivante rejoint alors le dernier message de l'utilisateur au lieu d'en ouvrir un
//! second.

use super::bascule::{Bascule, Issue};
use super::gemini::{self, AppelDuModele, Refus, Tour};
use super::modeles::{self, Modele};
use crate::agents::json::Json;
use crate::agents::outils::{self, Portee};
use crate::agents::{protocole, Boite, Demande};
use crate::plateforme::{echanger, Requete};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

/// **Combien attendre une réponse** — un nombre de ressenti : Gemma a mis 27 s pour un seul geste
/// le 22/11, un modèle qui pense sur un long tableau en met davantage. Au-delà de deux minutes,
/// l'utilisateur a déjà arrêté la question.
const ATTENTE: Duration = Duration::from_secs(120);
/// La plus grosse réponse lue : une conversation tient en quelques dizaines de kilo-octets.
const LIMITE: usize = 8 * 1024 * 1024;

/// Ce que la discussion dit à la boucle, au fil de la question.
#[derive(Debug)]
pub enum Evenement {
    /// La requête part à ce modèle : il « réfléchit ».
    Demande(String),
    /// Un geste sur le canevas : l'outil, et son échec s'il en a un.
    Geste {
        outil: String,
        echec: Option<String>,
    },
    /// Ce que le modèle dit, et qui l'a dit.
    Texte { modele: String, texte: String },
    /// Une bascule, ou une ligne du bilan de l'essai.
    Avertissement(String),
    /// La question est finie : la conversation revient à la boucle, avec l'erreur s'il y en a une.
    Fin {
        conversation: Conversation,
        erreur: Option<String>,
    },
}

/// **Ce que la discussion garde d'une question à l'autre** : la clé, l'histoire, et ce que la
/// bascule a appris — la liste des modèles n'est lue qu'une fois.
#[derive(Clone)]
pub struct Conversation {
    pub cle: String,
    pub histoire: Vec<Json>,
    pub bascule: Option<Bascule>,
}

/// Une conversation se décrit sans sa clé : elle ne doit paraître dans aucun journal.
impl std::fmt::Debug for Conversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Conversation")
            .field("histoire", &self.histoire.len())
            .field("bascule", &self.bascule.as_ref().map(|b| b.modeles().len()))
            .finish_non_exhaustive()
    }
}

impl Conversation {
    pub fn nouvelle(cle: String) -> Self {
        Self {
            cle,
            histoire: Vec::new(),
            bascule: None,
        }
    }
}

/// Une question en route : ce qui en arrive, et l'interrupteur.
pub struct EnRoute {
    pub evenements: Receiver<Evenement>,
    pub arret: Arc<AtomicBool>,
}

/// **Pose une question**, sur un fil à part.
pub fn poser(conversation: Conversation, question: String, boite: Boite) -> EnRoute {
    lancer(conversation, boite, move |c, dire, arret, boite| {
        repondre(c, question, dire, arret, boite)
    })
}

/// **L'essai** (« Tester les modèles gratuits ») : la liste que la clé ouvre, et le premier qui
/// répond vraiment — par la même bascule que la discussion, qui garde ce qu'elle a appris. Une
/// seule question au modèle qui répond : un essai ne doit pas manger le quota du jour.
pub fn essayer(conversation: Conversation, boite: Boite) -> EnRoute {
    lancer(conversation, boite, |c, dire, _, _| essai(c, dire))
}

type Dire<'a> = &'a dyn Fn(Evenement);

fn lancer(
    mut conversation: Conversation,
    boite: Boite,
    travail: impl FnOnce(&mut Conversation, Dire, &AtomicBool, &Boite) -> Result<(), String>
        + Send
        + 'static,
) -> EnRoute {
    let (tx, evenements) = channel();
    let arret = Arc::new(AtomicBool::new(false));
    let tenu = arret.clone();
    // Un fil qui ne part pas, ou qui panique, laisse la réception orpheline : la boucle le lit
    // comme une fin, et repart d'une conversation neuve.
    let _ = std::thread::Builder::new()
        .name("discussion".into())
        .spawn(move || {
            let dire = |e: Evenement| {
                let _ = tx.send(e);
                boite.reveiller();
            };
            let erreur = travail(&mut conversation, &dire, &tenu, &boite).err();
            dire(Evenement::Fin {
                conversation,
                erreur,
            });
        });
    EnRoute { evenements, arret }
}

/// La consigne : qui elle est, où elle parle — puis ce que la porte dit de Glucose à tout agent.
fn consigne() -> String {
    format!(
        "Tu es l'IA de la discussion de Glucose : l'utilisateur te parle depuis un tiroir à côté \
         de son canevas, et te regarde travailler. Réponds dans sa langue, simplement et \
         brièvement. Quand il te demande d'agir sur le canevas, fais-le par les outils, puis dis \
         en une phrase ce que tu as fait.\n\n{}",
        outils::INSTRUCTIONS
    )
}

/// L'heure du mur, en millisecondes depuis 1970 (BASCULE-1).
fn heure() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis().try_into().unwrap_or(u64::MAX))
}

/// **Une question, jusqu'à la réponse** : chaque tour qui demande des outils les reçoit, puis
/// le modèle reprend.
fn repondre(
    c: &mut Conversation,
    question: String,
    dire: Dire,
    arret: &AtomicBool,
    boite: &Boite,
) -> Result<(), String> {
    ajouter_la_question(&mut c.histoire, &question);
    let bascule = preparer(&c.cle, &mut c.bascule)?;
    let outils = gemini::outils(&outils::liste_pour(Portee::publique(true)));
    let consigne = consigne();
    loop {
        if arret.load(Ordering::Relaxed) {
            return Err("arrêtée".into());
        }
        let corps = gemini::corps(&consigne, &c.histoire, &outils);
        let (modele, tour) = demander(&c.cle, bascule, &corps, dire)?;
        c.histoire.push(tour.contenu.clone());
        if !tour.texte.trim().is_empty() {
            dire(Evenement::Texte {
                modele: modele.nom.clone(),
                texte: tour.texte.trim().to_string(),
            });
        }
        if tour.appels.is_empty() {
            return Ok(());
        }
        let reponses = tour
            .appels
            .iter()
            .map(|appel| {
                let issue = executer(boite, &modele.nom, appel);
                dire(Evenement::Geste {
                    outil: appel.nom.clone(),
                    echec: issue.as_ref().err().cloned(),
                });
                gemini::reponse_d_outil(appel, issue)
            })
            .collect();
        c.histoire.push(gemini::de_l_utilisateur(reponses));
    }
}

/// **DISCUSSION-2** : la question rejoint le dernier message s'il est de l'utilisateur.
pub fn ajouter_la_question(histoire: &mut Vec<Json>, question: &str) {
    let part = gemini::texte(question);
    if let Some(Json::Objet(champs)) = histoire.last_mut() {
        let de_lui = champs
            .iter()
            .any(|(k, v)| k == "role" && v.as_str() == Some("user"));
        if de_lui {
            if let Some((_, Json::Liste(parts))) = champs.iter_mut().find(|(k, _)| k == "parts") {
                parts.push(part);
                return;
            }
        }
    }
    histoire.push(gemini::de_l_utilisateur(vec![part]));
}

/// La liste des modèles, lue une fois ; puis une question neuve pour la bascule.
fn preparer<'a>(cle: &str, bascule: &'a mut Option<Bascule>) -> Result<&'a mut Bascule, String> {
    if bascule.is_none() {
        *bascule = Some(Bascule::nouvelle(lister(cle)?));
    }
    let bascule = bascule.as_mut().ok_or("aucune liste")?;
    bascule.nouvelle_question();
    Ok(bascule)
}

/// **Le premier éveillé répond** — ou la bascule passe au suivant, et le dit quand il répond.
fn demander(
    cle: &str,
    bascule: &mut Bascule,
    corps: &str,
    dire: Dire,
) -> Result<(Modele, Tour), String> {
    loop {
        let Some(i) = bascule.premier(heure()) else {
            return Err(bascule.personne(heure()));
        };
        let modele = bascule.modele(i).clone();
        dire(Evenement::Demande(modele.nom.clone()));
        match generer(cle, &modele.id, corps)? {
            Ok(tour) => {
                if let Some(annonce) = bascule.apprendre(i, &Issue::Repondu, heure())? {
                    dire(Evenement::Avertissement(annonce));
                }
                return Ok((modele, tour));
            }
            Err(refus) => {
                bascule.apprendre(i, &Issue::Refuse(refus), heure())?;
            }
        }
    }
}

/// **Une requête à un modèle** : son tour, ou son refus ; l'erreur du dehors quand le réseau
/// lui-même manque — aucun autre modèle n'y changerait rien.
fn generer(cle: &str, id: &str, corps: &str) -> Result<Result<Tour, Refus>, String> {
    let url = format!("{}/models/{id}:generateContent", gemini::ADRESSE);
    let reponse = echanger(
        &Requete {
            verbe: "POST",
            url: &url,
            en_tetes: &[
                ("x-goog-api-key", cle),
                ("Content-Type", "application/json"),
            ],
            corps: corps.as_bytes(),
            attente: ATTENTE,
        },
        LIMITE,
    )?;
    let texte = String::from_utf8_lossy(&reponse.corps);
    if reponse.statut != 200 {
        return Ok(Err(gemini::lire_le_refus(reponse.statut, &texte)));
    }
    // Un 200 illisible, ou vide, est un refus de ce modèle : le suivant peut mieux faire.
    Ok(Json::lire(&texte)
        .and_then(|j| gemini::lire_le_tour(&j))
        .map_err(|message| Refus {
            statut: 200,
            message,
            revenir_dans: None,
        }))
}

/// **Les modèles que la clé ouvre**, du premier à essayer au dernier.
fn lister(cle: &str) -> Result<Vec<Modele>, String> {
    let url = format!("{}/models?pageSize=200", gemini::ADRESSE);
    let reponse = echanger(
        &Requete {
            verbe: "GET",
            url: &url,
            en_tetes: &[("x-goog-api-key", cle)],
            corps: &[],
            attente: ATTENTE,
        },
        LIMITE,
    )?;
    let texte = String::from_utf8_lossy(&reponse.corps);
    if reponse.statut != 200 {
        let refus = gemini::lire_le_refus(reponse.statut, &texte);
        return Err(format!("Google refuse la clé ({})", refus.message));
    }
    Ok(modeles::gemini(&Json::lire(&texte)?))
}

/// **Un appel d'outil, par la boîte des agents** (DISCUSSION-1) : la boucle l'exécute, puis
/// répond par la ligne que la porte écrit à tout agent.
fn executer(boite: &Boite, client: &str, appel: &AppelDuModele) -> Result<Json, String> {
    let (tx, rx) = channel();
    boite.deposer(Demande {
        appel: protocole::Appel {
            id: Json::Nombre(1.0),
            nom: appel.nom.clone(),
            arguments: appel.arguments.clone(),
            client: client.to_string(),
            moderne: true,
            portee: Portee::publique(true),
        },
        reponse: tx,
    });
    let ligne = rx.recv().map_err(|_| "Glucose s'est fermé".to_string())?;
    issue_de_la_ligne(&ligne)
}

/// La ligne de la porte (`protocole::reponse_d_outil`), relue : le résultat de l'outil, ou son
/// erreur.
pub fn issue_de_la_ligne(ligne: &str) -> Result<Json, String> {
    let json = Json::lire(ligne)?;
    let resultat = json.get("result").ok_or("une réponse sans résultat")?;
    let texte = resultat
        .get("content")
        .and_then(Json::as_liste)
        .and_then(<[Json]>::first)
        .and_then(|c| c.get("text"))
        .and_then(Json::as_str)
        .unwrap_or("");
    if resultat.get("isError").and_then(Json::as_bool) == Some(true) {
        return Err(texte.to_string());
    }
    Ok(Json::lire(texte).unwrap_or_else(|_| Json::from(texte)))
}

/// **L'essai** : la liste, puis le premier qui répond — chaque modèle écarté dit pourquoi.
fn essai(c: &mut Conversation, dire: Dire) -> Result<(), String> {
    c.bascule = None;
    let bascule = preparer(&c.cle, &mut c.bascule)?;
    dire(Evenement::Avertissement(format!(
        "Ta clé ouvre {} modèles de conversation. J'essaie le plus capable, puis les suivants.",
        bascule.modeles().len()
    )));
    let histoire = [gemini::de_l_utilisateur(vec![gemini::texte(
        "Réponds par un seul mot : bonjour.",
    )])];
    let corps = gemini::corps("Réponds en un mot.", &histoire, &Json::Liste(Vec::new()));
    loop {
        let Some(i) = bascule.premier(heure()) else {
            return Err(bascule.personne(heure()));
        };
        let modele = bascule.modele(i).clone();
        match generer(&c.cle, &modele.id, &corps)? {
            Ok(_) => {
                bascule.apprendre(i, &Issue::Repondu, heure())?;
                dire(Evenement::Avertissement(format!(
                    "{} répond : c'est lui qui te répondra.",
                    modele.nom
                )));
                return Ok(());
            }
            Err(refus) => {
                bascule.apprendre(i, &Issue::Refuse(refus), heure())?;
                let raison = bascule.raison(i, heure());
                dire(Evenement::Avertissement(format!(
                    "{} : {raison}.",
                    modele.nom
                )));
            }
        }
    }
}
