//! **Le lien** : le fil qui tient le WebSocket de ce Glucose vers son relais, tant que la porte
//! est ouverte — il s'y relie, s'y relie encore quand le réseau revient, et sert chaque requête
//! sur un fil à elle.
//!
//! # Vivant ou pas
//!
//! Toutes les [`PING`], le lien envoie `ping` ; le relais répond `pong` sans se réveiller (rien ne
//! se paie). Sans réponse après [`PONG_MAX`], la connexion est morte — un réseau coupé ne le dit
//! pas toujours — et le lien en ouvre une autre. Entre deux essais, l'attente double, d'une
//! seconde à une minute : un relais absent n'est pas martelé.

use super::accords::Etat;
use super::http::{self, Contexte, Reponse, Requete};
use super::oauth::Guichet;
use super::Adresses;
use crate::agents::Boite;
use crate::plateforme::websocket::{self, Canal};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// L'intervalle du ping : bien sous la minute d'attente d'une réception (WinHTTP) et le délai où
/// une box oublie une connexion muette.
const PING: Duration = Duration::from_secs(20);
/// Le pong doit être revenu avant.
const PONG_MAX: Duration = Duration::from_secs(10);
/// L'attente la plus longue entre deux essais.
const PAUSE_MAX: Duration = Duration::from_secs(60);
/// Les requêtes servies en même temps ; au-delà, `503` tout de suite — une IA qui s'emballe ne
/// lance pas mille fils.
const EN_COURS_MAX: usize = 16;
/// Le plus gros document d'identité qu'on lise.
const DOCUMENT_MAX: usize = 64 << 10;

/// **Ce que le lien dit de lui**, pour l'interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EtatDuLien {
    /// Il s'y relie.
    EnRoute,
    /// Relié : les IA trouvent ce Glucose.
    Relie,
    /// Pas relié, et pourquoi ; il réessaie.
    Hors(String),
}

/// **Le lien en marche** : le lâcher l'arrête.
pub struct Lien {
    arret: Arc<AtomicBool>,
    pub guichet: Arc<Mutex<Guichet>>,
    pub etat: Arc<Mutex<EtatDuLien>>,
    pub adresses: Adresses,
}

impl Drop for Lien {
    fn drop(&mut self) {
        self.arret.store(true, Ordering::Relaxed);
    }
}

impl Lien {
    pub fn etat(&self) -> EtatDuLien {
        self.etat
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// Ce que chaque requête emporte sur son fil.
#[derive(Clone)]
struct Partage {
    adresses: Adresses,
    guichet: Arc<Mutex<Guichet>>,
    fichier: PathBuf,
    boite: Boite,
    en_cours: Arc<AtomicUsize>,
}

/// **Lance le lien** de ce Glucose, avec l'état de sa porte.
pub fn lancer(racine: &str, etat: Etat, fichier: PathBuf, boite: Boite) -> Lien {
    let adresses = Adresses::nouvelles(racine, &etat.identifiant());
    let lien = Lien {
        arret: Arc::new(AtomicBool::new(false)),
        guichet: Arc::new(Mutex::new(Guichet::nouveau(etat))),
        etat: Arc::new(Mutex::new(EtatDuLien::EnRoute)),
        adresses: adresses.clone(),
    };
    let partage = Partage {
        adresses,
        guichet: Arc::clone(&lien.guichet),
        fichier,
        boite,
        en_cours: Arc::default(),
    };
    let (arret, etat) = (Arc::clone(&lien.arret), Arc::clone(&lien.etat));
    let _ = std::thread::Builder::new()
        .name("glucose-porte-publique".into())
        .spawn(move || tourner(&partage, &arret, &etat));
    lien
}

fn dire(etat: &Mutex<EtatDuLien>, partage: &Partage, nouveau: EtatDuLien) {
    *etat.lock().unwrap_or_else(PoisonError::into_inner) = nouveau;
    partage.boite.reveiller();
}

fn tourner(partage: &Partage, arret: &AtomicBool, etat: &Mutex<EtatDuLien>) {
    let mut pause = Duration::from_secs(1);
    while !arret.load(Ordering::Relaxed) {
        let secret = partage
            .guichet
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .etat
            .secret_texte();
        let autorisation = format!("Bearer {secret}");
        match websocket::ouvrir(
            &partage.adresses.websocket(),
            &[("Authorization", &autorisation)],
        ) {
            Ok(canal) => {
                dire(etat, partage, EtatDuLien::Relie);
                pause = Duration::from_secs(1);
                let raison = servir(canal, partage, arret);
                crate::dire!("[Glucose] porte publique : {raison}");
                dire(etat, partage, EtatDuLien::Hors(raison));
            }
            Err(e) => dire(etat, partage, EtatDuLien::Hors(e)),
        }
        let debut = Instant::now();
        while debut.elapsed() < pause && !arret.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(100));
        }
        pause = (pause * 2).min(PAUSE_MAX);
    }
}

/// **Sert une connexion** jusqu'à ce qu'elle meure ou que la porte se ferme ; rend pourquoi.
fn servir(mut canal: Box<dyn Canal>, partage: &Partage, arret: &AtomicBool) -> String {
    let (envoi, reponses) = mpsc::channel::<String>();
    let mut dernier_ping = Instant::now();
    let mut pong_attendu: Option<Instant> = None;
    loop {
        if arret.load(Ordering::Relaxed) {
            return "porte fermée".into();
        }
        for r in reponses.try_iter() {
            if let Err(e) = canal.envoyer(&r) {
                return e;
            }
        }
        if pong_attendu.is_some_and(|t| t.elapsed() > PONG_MAX) {
            return "le relais ne répond plus".into();
        }
        if pong_attendu.is_none() && dernier_ping.elapsed() > PING {
            if let Err(e) = canal.envoyer("ping") {
                return e;
            }
            dernier_ping = Instant::now();
            pong_attendu = Some(dernier_ping);
        }
        match canal.recevoir(Duration::from_millis(25)) {
            Ok(Some(m)) if m == "pong" => pong_attendu = None,
            Ok(Some(m)) => {
                if let Some(requete) = Requete::lire(&m) {
                    repondre_a_part(requete, partage, &envoi);
                }
            }
            Ok(None) => {}
            Err(e) => return e,
        }
    }
}

/// Une requête, sur son fil : un appel d'outil attend la boucle, un jeton se donne tout de suite.
fn repondre_a_part(requete: Requete, partage: &Partage, envoi: &mpsc::Sender<String>) {
    if partage.en_cours.fetch_add(1, Ordering::Relaxed) >= EN_COURS_MAX {
        partage.en_cours.fetch_sub(1, Ordering::Relaxed);
        let occupe = Reponse {
            statut: 503,
            entetes: Vec::new(),
            corps: String::new(),
        };
        let _ = envoi.send(occupe.ecrire(requete.n));
        return;
    }
    let (p, envoi) = (partage.clone(), envoi.clone());
    let lance = std::thread::Builder::new()
        .name("glucose-porte-requete".into())
        .spawn(move || {
            let reponse = http::traiter(&requete, &contexte(&p, &chercher));
            p.en_cours.fetch_sub(1, Ordering::Relaxed);
            let _ = envoi.send(reponse.ecrire(requete.n));
        });
    if lance.is_err() {
        partage.en_cours.fetch_sub(1, Ordering::Relaxed);
    }
}

fn contexte<'a>(
    p: &'a Partage,
    chercher: &'a dyn Fn(&str) -> Result<String, String>,
) -> Contexte<'a> {
    Contexte {
        adresses: &p.adresses,
        guichet: &p.guichet,
        fichier: &p.fichier,
        boite: &p.boite,
        chercher,
        maintenant: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs()),
    }
}

/// Le document d'identité d'un client : une adresse `https://`, quelques kilo-octets de JSON.
fn chercher(adresse: &str) -> Result<String, String> {
    let octets = crate::plateforme::telecharger(adresse, DOCUMENT_MAX)?;
    String::from_utf8(octets).map_err(|_| "pas du texte".to_string())
}
