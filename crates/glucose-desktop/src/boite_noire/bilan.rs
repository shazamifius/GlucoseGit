//! **Comment la session d'avant a fini**, relu au lancement suivant — et ce que la boîte noire
//! garde sur le disque.
//!
//! # Ce qu'on peut dire, et rien de plus
//!
//! Une session qui a fini proprement l'a écrit. Une panique du fil principal aussi — le témoin
//! l'a notée avant que le programme tombe. Sans l'une ni l'autre, la session s'est **arrêtée
//! sans rien dire** : tuée, plantée hors de Rust (un pilote), gelée puis fermée de force — ou
//! l'appareil s'est éteint. Ce dernier cas se reconnaît à une chose sûre : **sa dernière trace
//! précède le démarrage de l'appareil présent**. On le dit tel quel, « l'appareil a redémarré
//! depuis », avec la dernière batterie écrite ; conclure « la batterie » serait deviner.
//!
//! **Et ce que le système a vu** : une session arrêtée sans rien dire a peut-être planté, ou gelé
//! jusqu'à ce que le système la ferme — il l'a noté ([`super::plantage`]). Elle se dit alors
//! « plantée dans tel module » ou « gelée ».

use super::plantage::{self, Plantage};
use super::sondes::EtatMachine;
use crate::plateforme::journal::Evenement;
use glucose_core::persist::tauri::json;
use std::path::{Path, PathBuf};

/// Comment une session a fini.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fin {
    /// Fermée par l'utilisateur : la dernière ligne le dit.
    Propre,
    /// Une panique du fil principal l'a fait tomber.
    Panique,
    /// Arrêtée sans rien dire.
    Interrompue,
    /// Arrêtée sans rien dire, et le système a vu un plantage.
    Plantee,
    /// Arrêtée sans rien dire, et le système l'a fermée gelée.
    Gelee,
}

impl Fin {
    pub fn nom(self) -> &'static str {
        match self {
            Fin::Propre => "propre",
            Fin::Panique => "panique",
            Fin::Interrompue => "interrompue",
            Fin::Plantee => "plantee",
            Fin::Gelee => "gelee",
        }
    }
}

/// Ce qu'on sait de la fin d'une session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bilan {
    pub fin: Fin,
    /// L'appareil a redémarré depuis la dernière trace d'une session qui n'a pas fini proprement.
    pub appareil_redemarre: bool,
    /// De la première trace à la dernière.
    pub duree_ms: u64,
    /// Le dernier état de la machine qu'elle a écrit.
    pub machine: EtatMachine,
    /// Ce que le système a vu de sa fin, si elle s'est arrêtée sans rien dire.
    pub plantage: Option<Plantage>,
}

impl Bilan {
    /// Ce qu'on en dit, en une phrase.
    pub fn dire(&self) -> String {
        let duree = self.duree_ms / 1000;
        let (min, s) = (duree / 60, duree % 60);
        let mut t = match self.fin {
            Fin::Propre => format!("fermée proprement, après {min} min {s} s"),
            Fin::Panique => format!("tombée sur une panique, après {min} min {s} s"),
            Fin::Interrompue => format!("arrêtée sans rien dire, après {min} min {s} s"),
            Fin::Plantee | Fin::Gelee => match &self.plantage {
                Some(p) => format!("{}, après {min} min {s} s", p.dire()),
                None => format!("arrêtée sans rien dire, après {min} min {s} s"),
            },
        };
        if self.appareil_redemarre {
            t.push_str(" ; l'appareil a redémarré depuis");
            if let Some(p) = self.machine.batterie_pct {
                let secteur = match self.machine.en_charge {
                    Some(true) => ", sur secteur",
                    Some(false) => ", sur batterie",
                    None => "",
                };
                t.push_str(&format!(" — dernière batterie : {p} %{secteur}"));
            }
        }
        t
    }
}

/// Le bilan d'une session d'après ses lignes, et le démarrage de l'appareil présent.
///
/// `None` si le texte n'est pas une session : aucun début lisible.
pub fn bilan(texte: &str, demarrage_appareil_ms: Option<u64>) -> Option<Bilan> {
    let mut epoque = None;
    let mut dernier = 0u64;
    let mut fin = Fin::Interrompue;
    let mut machine = EtatMachine::default();
    for ligne in texte.lines() {
        // Une ligne coupée par la fin brutale ne se lit pas : c'est justement cette fin-là
        // qu'on cherche, et les lignes d'avant suffisent à la dire.
        let Ok(v) = json::lire(ligne) else {
            continue;
        };
        let entier = |cle: &str| v.entier(cle).and_then(|n| u64::try_from(n).ok());
        dernier = dernier.max(entier("instant_ms").unwrap_or(0));
        match v.texte("type") {
            Some("debut") => epoque = entier("epoque_ms"),
            Some("machine") => {
                machine = EtatMachine {
                    batterie_pct: entier("batterie_pct").and_then(|n| u8::try_from(n).ok()),
                    en_charge: v.booleen("en_charge"),
                };
            }
            // Un autre fil peut tomber sans emporter le programme : seul le principal finit
            // la session.
            Some("panique") if v.booleen("principal") == Some(true) => fin = Fin::Panique,
            Some("fin") => fin = Fin::Propre,
            _ => {}
        }
    }
    let derniere_trace = epoque? + dernier;
    Some(Bilan {
        fin,
        appareil_redemarre: fin != Fin::Propre
            && demarrage_appareil_ms.is_some_and(|d| derniere_trace < d),
        duree_ms: dernier,
        machine,
        plantage: None,
    })
}

/// Les sessions de la boîte noire, de la plus ancienne à la plus récente.
pub fn sessions(dossier: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dossier)
        .map(|entrees| {
            entrees
                .flatten()
                .map(|e| e.path())
                .filter(|p| est_une_session(p))
                .collect()
        })
        .unwrap_or_default();
    // Le nom commence par l'heure du début, sur vingt chiffres : l'ordre des noms est celui
    // du temps.
    v.sort();
    v
}

fn est_une_session(p: &Path) -> bool {
    let nom = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
    nom.starts_with("session-") && nom.ends_with(".jsonl")
}

/// **Relit la session d'avant, et range le dossier.**
///
/// Ce qui reste sur le disque, sans aucun nombre choisi :
///
/// * **la session d'avant**, quelle que soit sa fin — c'est elle qu'on raconte ;
/// * **toute session qui a mal fini** : ce sont celles qui ont quelque chose à apprendre ;
/// * une session **encore tenue** par un autre Glucose, qui n'a pas fini du tout.
///
/// Une session plus ancienne qui a fini proprement s'efface : elle a été racontée au lancement
/// qui la suivait. Un fichier sans début lisible aussi : il ne dit rien.
///
/// `systeme` rend ce que le système a noté depuis un instant
/// ([`crate::plateforme::journal::plantages_depuis`]) : il n'est lu que pour une session d'avant
/// qui s'est arrêtée sans rien dire.
pub fn ranger(
    dossier: &Path,
    demarrage_appareil_ms: Option<u64>,
    systeme: impl FnOnce(u64) -> Vec<Evenement>,
) -> Option<Bilan> {
    let vivantes = |p: &Path| crate::persist::verrou::tenu_ailleurs(p);
    let finies: Vec<PathBuf> = sessions(dossier)
        .into_iter()
        .filter(|p| !vivantes(p))
        .collect();
    let (precedente, anciennes) = finies.split_last()?;
    for p in anciennes {
        let b = std::fs::read_to_string(p)
            .ok()
            .and_then(|t| bilan(&t, demarrage_appareil_ms));
        if b.is_none_or(|b| b.fin == Fin::Propre) {
            let _ = std::fs::remove_file(p);
        }
    }
    let texte = std::fs::read_to_string(precedente).ok()?;
    let mut b = bilan(&texte, demarrage_appareil_ms)?;
    if b.fin == Fin::Interrompue {
        if let Some((debut_ms, processus)) = debut_et_processus(precedente) {
            b.plantage = plantage::reconnaitre(&systeme(debut_ms), processus, debut_ms);
        }
        if let Some(p) = &b.plantage {
            b.fin = if p.gel { Fin::Gelee } else { Fin::Plantee };
        }
    }
    Some(b)
}

/// Le début d'une session et son processus, lus dans son nom :
/// `session-<début, en millisecondes depuis 1970>-<processus>.jsonl`.
fn debut_et_processus(p: &Path) -> Option<(u64, u32)> {
    let nom = p.file_name()?.to_str()?;
    let (debut, processus) = nom
        .strip_prefix("session-")?
        .strip_suffix(".jsonl")?
        .split_once('-')?;
    Some((debut.parse().ok()?, processus.parse().ok()?))
}
