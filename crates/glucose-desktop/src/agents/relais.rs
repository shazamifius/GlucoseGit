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

mod lien;

use super::json::Json;
use super::porte;
use super::protocole::{Connexion, Suite};
pub use lien::Sortie;
use lien::{Envoi, Lien};
use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};

/// Le temps qu'on laisse à Glucose pour ouvrir sa porte après l'avoir lancé : le lancement le
/// plus lent mesuré (la reprise d'un gros document, fiche 47) tient en quelques secondes ; au
/// bout de celui-ci, quelque chose d'autre ne va pas, et le dire vaut mieux qu'attendre.
const LANCEMENT_MAX: Duration = Duration::from_secs(30);
/// L'intervalle entre deux essais de la porte, pendant ce lancement.
const ESSAI: Duration = Duration::from_millis(100);

/// **Relaie** jusqu'à ce que le client ferme l'entrée ; rend le code de sortie du processus.
pub fn relayer() -> i32 {
    let Some((nom, ouvrir)) = porte_a_joindre() else {
        crate::dire_erreur!("[glucose --mcp] aucune porte pour ce compte");
        return 1;
    };
    let sortie: Sortie = std::sync::Arc::new(std::sync::Mutex::new(Box::new(std::io::stdout())));
    relayer_par(&mut BufReader::new(std::io::stdin()), &sortie, &nom, ouvrir)
}

/// **Le relais, ses flux donnés** : le client d'un côté, la porte nommée de l'autre. Une porte
/// qui se ferme se rebranche à la ligne suivante (RELAIS-2, [`lien`]).
pub fn relayer_par(entree: &mut impl BufRead, sortie: &Sortie, nom: &str, ouvrir: bool) -> i32 {
    // Le client a fermé l'entrée avant tout outil : rien n'a été ouvert, rien à fermer.
    let Some((prelude, premier)) = repondre_seul(entree, &mut Partagee(sortie)) else {
        return 0;
    };
    let mut lien = match brancher(nom, ouvrir).and_then(|f| Lien::sur(f, &prelude, sortie)) {
        Ok(l) => l,
        Err(e) => {
            crate::dire_erreur!("[glucose --mcp] {e}");
            return 1;
        }
    };
    let mut suivante = Some(premier);
    while let Some(ligne) = suivante {
        if lien.envoyer(&ligne) == Envoi::Coupee {
            match revenir(nom, ouvrir).and_then(|f| Lien::sur(f, &prelude, sortie)) {
                Ok(neuf) => {
                    lien = neuf;
                    if lien.envoyer(&ligne) == Envoi::Coupee {
                        repondre_en_erreur(sortie, &ligne, lien::FERMEE);
                    }
                }
                Err(e) => repondre_en_erreur(sortie, &ligne, &e),
            }
        }
        suivante = porte::lire_une_ligne(entree).ok().flatten();
    }
    // Le client a fermé l'entrée : c'est le signal de fin (« Servers SHOULD exit promptly when
    // their standard input is closed »).
    0
}

/// Une requête qui n'a pas pu partir reçoit son erreur ; une notification se perd en silence,
/// comme le protocole le veut (elle n'attend aucune réponse).
fn repondre_en_erreur(sortie: &Sortie, ligne: &str, pourquoi: &str) {
    if let Some(id) = Json::lire(ligne).ok().and_then(|m| m.get("id").cloned()) {
        lien::dire(
            sortie,
            &super::protocole::erreur_de_fermeture(&id, pourquoi),
        );
    }
}

/// La sortie partagée, vue comme un flux : pour la poignée de main que le relais rend seul.
struct Partagee<'a>(&'a Sortie);

impl Write for Partagee<'_> {
    fn write(&mut self, octets: &[u8]) -> std::io::Result<usize> {
        let mut s = self.0.lock().map_err(|_| std::io::ErrorKind::BrokenPipe)?;
        s.write(octets)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut s = self.0.lock().map_err(|_| std::io::ErrorKind::BrokenPipe)?;
        s.flush()
    }
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

/// **La porte à joindre**, et s'il faut ouvrir Glucose quand elle ne répond pas : celle de ce
/// compte, ou celle que nomme `GLUCOSE_PORTE` — un instrument d'épreuve, qui n'ouvre jamais Glucose.
/// Le relais et les commandes du terminal (fiche 90) la joignent pareil.
pub(super) fn porte_a_joindre() -> Option<(String, bool)> {
    let instrument = std::env::var("GLUCOSE_PORTE")
        .ok()
        .filter(|n| !n.is_empty());
    match instrument {
        Some(nom) => Some((nom, false)),
        None => porte::nom_par_defaut().map(|nom| (nom, true)),
    }
}

/// Se branche à la porte — et ouvre Glucose, s'il le faut et si on le peut.
pub(super) fn brancher(nom: &str, ouvrir: bool) -> Result<porte::Flux, String> {
    match porte::connecter(nom) {
        Ok(f) => Ok(f),
        Err(_) if ouvrir => lancer_glucose().and_then(|()| {
            attendre(nom).ok_or_else(|| "Glucose s'est ouvert, mais sa porte ne répond pas".into())
        }),
        Err(e) => Err(format!("la porte {nom} ne répond pas : {e}")),
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
    #[cfg(target_os = "linux")]
    if par_son_service() {
        return Ok(());
    }
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

/// **Son service d'abord** (fiche 144 § 4). Sous Linux, un service utilisateur `glucose.service`
/// peut tenir Glucose ouvert jour et nuit (le relais, H24) et le relancer quand il tombe. C'est
/// alors à lui de l'ouvrir : pendant qu'il le relance, la porte se tait une seconde, et un relais
/// MCP ou une commande qui lançait Glucose à côté faisait **une seconde fenêtre** sur le même
/// document (le 24/11 à 14 h 02). Demander au service ne double jamais : `start` ne fait rien sur
/// un service qui tourne, et rejoint celui qui redémarre. Sans service activé : le lancer soi-même.
#[cfg(target_os = "linux")]
fn par_son_service() -> bool {
    const SERVICE: &str = "glucose.service";
    let systemctl = |args: &[&str]| {
        std::process::Command::new("systemctl")
            .arg("--user")
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    systemctl(&["is-enabled", "--quiet", SERVICE]) && systemctl(&["start", SERVICE])
}

/// Attend que la porte de Glucose réponde, le temps d'un lancement.
fn attendre(nom: &str) -> Option<porte::Flux> {
    let debut = Instant::now();
    loop {
        if let Ok(f) = porte::connecter(nom) {
            return Some(f);
        }
        if debut.elapsed() > LANCEMENT_MAX {
            return None;
        }
        std::thread::sleep(ESSAI);
    }
}

/// **Rebrancher** (RELAIS-2) : Glucose était là un instant plus tôt — son service, une mise à jour
/// ou l'utilisateur le relancent peut-être déjà. On lui laisse le temps d'un lancement avant de
/// l'ouvrir soi-même : jamais deux Glucose.
fn revenir(nom: &str, ouvrir: bool) -> Result<porte::Flux, String> {
    match attendre(nom) {
        Some(f) => Ok(f),
        None if ouvrir => brancher(nom, true),
        None => Err(format!("la porte {nom} ne répond plus")),
    }
}

#[cfg(test)]
mod tests;
