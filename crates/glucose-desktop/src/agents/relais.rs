//! **`glucose --mcp`** : le programme qu'un client MCP lance pour parler à Glucose.
//!
//! Un client MCP (Claude Code, Protéo) lance un serveur et lui parle par ses flux standard. Le
//! serveur, ici, c'est le Glucose **déjà ouvert** sous les yeux de l'utilisateur : le relais
//! recopie les lignes entre ses flux et la porte ([`super::porte`]) ; le protocole se joue dans
//! Glucose.
//!
//! # RELAIS-1 — le relais n'ouvre Glucose qu'au premier outil appelé
//!
//! Un client lance ses serveurs au début de chacune de ses sessions — Claude Code le fait. Un
//! relais qui se brancherait tout de suite ouvrirait Glucose à chaque session, sans que rien ne
//! lui soit demandé. Il répond donc **lui-même** à la découverte, à la poignée de main et à la
//! liste des outils — par le même protocole, pur, que Glucose ([`super::protocole`]) — et ne se
//! branche qu'au premier appel d'outil. Il rejoue alors la poignée de main d'un client ancien
//! devant la porte, et tait les réponses déjà données.
//!
//! **Si Glucose n'est pas ouvert, le relais l'ouvre** à ce moment-là — sans lui confier ses
//! flux : ce que Glucose écrit sur sa sortie corromprait le protocole, qui n'y admet que des
//! messages (« The server MUST NOT write anything to its stdout that is not a valid MCP
//! message »).
//!
//! `GLUCOSE_PORTE` nomme une autre porte : un instrument, pour les épreuves, qui n'ouvre alors
//! jamais Glucose.

use super::json::Json;
use super::porte;
use super::protocole::{Connexion, Suite};
use std::io::{BufRead, BufReader, Read, Write};
use std::time::{Duration, Instant};

/// Le temps qu'on laisse à Glucose pour ouvrir sa porte après l'avoir lancé : le lancement le
/// plus lent mesuré (la reprise d'un gros document, fiche 47) tient en quelques secondes ; au
/// bout de celui-ci, quelque chose d'autre ne va pas, et le dire vaut mieux qu'attendre.
const LANCEMENT_MAX: Duration = Duration::from_secs(30);
/// L'intervalle entre deux essais de la porte, pendant ce lancement.
const ESSAI: Duration = Duration::from_millis(100);

/// **Relaie** jusqu'à ce que le client ferme l'entrée ; rend le code de sortie du processus.
pub fn relayer() -> i32 {
    let instrument = std::env::var("GLUCOSE_PORTE")
        .ok()
        .filter(|n| !n.is_empty());
    let Some(nom) = instrument.clone().or_else(porte::nom_par_defaut) else {
        eprintln!("[glucose --mcp] aucune porte pour ce compte");
        return 1;
    };
    let mut entree = BufReader::new(std::io::stdin());
    // Le client a fermé l'entrée avant tout outil : rien n'a été ouvert, rien à fermer.
    let Some((prelude, premier)) = repondre_seul(&mut entree, &mut std::io::stdout()) else {
        return 0;
    };
    let flux = match brancher(&nom, instrument.is_none()) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[glucose --mcp] {e}");
            return 1;
        }
    };
    let Ok(retour) = flux.try_clone() else {
        return 1;
    };
    // Les réponses de la poignée de main rejouée ont déjà été données : elles se taisent.
    let a_taire = prelude.iter().filter(|l| a_une_reponse(l)).count();
    let _ = std::thread::Builder::new()
        .name("glucose-relais".into())
        .spawn(move || {
            let mut retour = BufReader::new(retour);
            for _ in 0..a_taire {
                if !matches!(porte::lire_une_ligne(&mut retour), Ok(Some(_))) {
                    return;
                }
            }
            recopier(&mut retour, &mut std::io::stdout());
        });
    let mut aller = flux;
    for ligne in prelude.iter().chain([&premier]) {
        if writeln!(aller, "{ligne}").is_err() {
            return 1;
        }
    }
    recopier(&mut entree, &mut aller);
    // Le client a fermé l'entrée : c'est le signal de fin (« Servers SHOULD exit promptly when
    // their standard input is closed »).
    0
}

/// **Répond seul** jusqu'au premier appel d'outil (RELAIS-1) : rend la poignée de main à
/// rejouer devant la porte, et cet appel — ou rien, si le client ferme avant.
pub fn repondre_seul(
    entree: &mut impl BufRead,
    sortie: &mut impl Write,
) -> Option<(Vec<String>, String)> {
    let mut locale = Connexion::default();
    let mut prelude = Vec::new();
    loop {
        let ligne = porte::lire_une_ligne(entree).ok()??;
        let methode = Json::lire(&ligne)
            .ok()
            .and_then(|m| m.get("method").and_then(Json::as_str).map(str::to_string));
        let de_la_poignee = matches!(
            methode.as_deref(),
            Some("initialize" | "notifications/initialized")
        );
        match locale.traiter(&ligne) {
            Suite::Executer(_) => return Some((prelude, ligne)),
            Suite::Repondre(r) => writeln!(sortie, "{r}").and_then(|()| sortie.flush()).ok()?,
            Suite::Rien => {}
        }
        if de_la_poignee {
            prelude.push(ligne);
        }
    }
}

/// Une ligne qui attend une réponse : une requête, avec son identifiant.
fn a_une_reponse(ligne: &str) -> bool {
    Json::lire(ligne).is_ok_and(|m| m.get("id").is_some())
}

/// Se branche à la porte — et ouvre Glucose, s'il le faut et si on le peut.
fn brancher(nom: &str, ouvrir: bool) -> Result<porte::Flux, String> {
    match porte::connecter(nom) {
        Ok(f) => Ok(f),
        Err(_) if ouvrir => lancer_glucose().and_then(|()| attendre(nom)),
        Err(e) => Err(format!("la porte {nom} ne répond pas : {e}")),
    }
}

/// Recopie un flux dans l'autre, en vidant à chaque morceau : une réponse ne reste jamais dans
/// un tampon pendant que le client l'attend.
fn recopier(depuis: &mut impl Read, vers: &mut impl Write) {
    let mut tampon = [0u8; 64 * 1024];
    loop {
        match depuis.read(&mut tampon) {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                if vers
                    .write_all(&tampon[..n])
                    .and_then(|()| vers.flush())
                    .is_err()
                {
                    return;
                }
            }
        }
    }
}

/// Lance Glucose, détaché, ses flux ailleurs que les nôtres.
///
/// **Sa vie n'est pas celle de l'agent.** Un client peut ranger le relais dans un *Job* Windows
/// qui tue toute la descendance quand il s'arrête (Protéo le fait, `arbre.rs`) : le Glucose
/// ouvert ici en serait — et le travail de l'utilisateur avec. Il quitte donc le *Job*
/// (`CREATE_BREAKAWAY_FROM_JOB`) ; un *Job* qui l'interdit fait échouer ce lancement, et le
/// second, ordinaire, le dit au moins en s'ouvrant.
fn lancer_glucose() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("Glucose introuvable : {e}"))?;
    let commande = || {
        let mut c = std::process::Command::new(&exe);
        c.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        c
    };
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
        if commande()
            .creation_flags(CREATE_BREAKAWAY_FROM_JOB)
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
    }
    commande()
        .spawn()
        .map(drop)
        .map_err(|e| format!("Glucose n'a pas pu s'ouvrir : {e}"))
}

/// Attend que la porte de Glucose réponde.
fn attendre(nom: &str) -> Result<porte::Flux, String> {
    let debut = Instant::now();
    loop {
        if let Ok(f) = porte::connecter(nom) {
            return Ok(f);
        }
        if debut.elapsed() > LANCEMENT_MAX {
            return Err("Glucose s'est ouvert, mais sa porte ne répond pas".to_string());
        }
        std::thread::sleep(ESSAI);
    }
}

#[cfg(test)]
mod tests;
