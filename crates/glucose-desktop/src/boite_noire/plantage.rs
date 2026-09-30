//! **Ce que le système a vu d'une session qui s'est arrêtée sans rien dire** (fiche 49).
//!
//! # Lire le système, plutôt qu'un témoin
//!
//! Le plan (fiche 44 § 1.1) prévoyait un processus témoin — `crash-handler` et `minidumper` — qui
//! écrirait l'état du programme à l'instant où il tombe. Windows le fait déjà : son rapporteur
//! d'erreurs note chaque plantage dans le journal Application (« Application Error »,
//! événement 1000 : le module, le code de l'exception, le décalage, le numéro du processus et
//! l'heure de sa création) et chaque gel qu'il a fermé (« Application Hang », 1002). Le relire au
//! lancement suivant ne demande **aucune caisse**, **aucun code qui tourne dans un programme déjà
//! abîmé**, et ne lit **aucune mémoire** du programme — un minidump en contient, et avec elle ce
//! que l'utilisateur écrivait.
//!
//! # Reconnaître la session, sans seuil
//!
//! Un numéro de processus se réutilise, mais jamais par deux processus vivants à la fois. Le
//! nôtre a été créé **avant** le début de sa session — c'est lui qui l'a ouverte — et il est tombé
//! **après**. Un événement qui porte ce numéro, pour un processus créé au plus tard au début de la
//! session et tombé au plus tôt à ce début, ne peut être que lui : celui qui portait ce numéro
//! avant est tombé avant notre création ; celui qui l'a porté après a été créé après notre fin.
//!
//! # Ce qui s'en écrit
//!
//! Des nombres, et deux noms : celui du module et sa version — un nom de fichier du système ou
//! d'un pilote, jamais un chemin (qui porterait le nom de l'utilisateur). Un nom qui contient
//! autre chose que des lettres, des chiffres et `.-_+` ne se dit pas.

use crate::plateforme::journal::Evenement;

/// Ce que le système a vu de la fin d'une session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plantage {
    /// Un gel que le système a fermé, plutôt qu'un plantage.
    pub gel: bool,
    /// Le module où le plantage a eu lieu : un nom de fichier, sans chemin.
    pub module: Option<String>,
    pub version_du_module: Option<String>,
    /// Le code de l'exception (`0xC0000005` : une violation d'accès à la mémoire).
    pub code: Option<u32>,
    /// Où, dans le module.
    pub decalage: Option<u64>,
}

/// **Le plantage de la session** que ce processus a ouverte à `debut_ms`, parmi ce que le
/// système a noté — le plus tardif, s'il en a noté plusieurs.
pub fn reconnaitre(evenements: &[Evenement], processus: u32, debut_ms: u64) -> Option<Plantage> {
    evenements
        .iter()
        .filter(|e| {
            e.processus == processus && e.creation_ms <= debut_ms && debut_ms <= e.instant_ms
        })
        .max_by_key(|e| e.instant_ms)
        .map(|e| Plantage {
            gel: e.gel,
            module: e.module.as_deref().and_then(nom_sur),
            version_du_module: e.version_du_module.as_deref().and_then(nom_sur),
            code: e
                .code
                .as_deref()
                .and_then(|c| u32::from_str_radix(sans_0x(c), 16).ok()),
            decalage: e
                .decalage
                .as_deref()
                .and_then(|d| u64::from_str_radix(sans_0x(d), 16).ok()),
        })
}

fn sans_0x(t: &str) -> &str {
    t.trim().trim_start_matches("0x").trim_start_matches("0X")
}

/// Un nom qu'on peut dire sans risque : lettres, chiffres, `.-_+` — ce que portent les noms de
/// modules et leurs versions —, et rien d'autre. Pas de séparateur de chemin : jamais un dossier
/// de l'utilisateur.
fn nom_sur(t: &str) -> Option<String> {
    let t = t.trim();
    let sur = !t.is_empty()
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+'));
    sur.then(|| t.to_string())
}

impl Plantage {
    /// Ce qu'on en dit, en quelques mots.
    pub fn dire(&self) -> String {
        if self.gel {
            return "gelée, puis fermée par le système".to_string();
        }
        let mut t = "plantée".to_string();
        if let Some(m) = &self.module {
            t.push_str(&format!(" dans {m}"));
            if let Some(v) = &self.version_du_module {
                t.push_str(&format!(" {v}"));
            }
        }
        if let Some(c) = self.code {
            match nom_de_l_exception(c) {
                Some(nom) => t.push_str(&format!(" ({nom}, {c:08x})")),
                None => t.push_str(&format!(" ({c:08x})")),
            }
        }
        t
    }
}

/// Le sens des codes d'exception de Windows qu'on rencontre (`ntstatus.h`).
fn nom_de_l_exception(code: u32) -> Option<&'static str> {
    Some(match code {
        0xC000_0005 => "violation d'accès à la mémoire",
        0xC000_001D => "instruction illégale",
        0xC000_0094 => "division entière par zéro",
        0xC000_00FD => "débordement de pile",
        0xC000_0409 => "arrêt demandé par le programme",
        0x8000_0003 => "point d'arrêt",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le plantage du pilote OpenGL noté par Windows le 24/09 sur sa machine (un programme
    /// d'épreuve), tel que le journal le rend.
    fn vu(processus: u32, creation_ms: u64, instant_ms: u64) -> Evenement {
        Evenement {
            gel: false,
            processus,
            creation_ms,
            instant_ms,
            module: Some("nvoglv64.dll".into()),
            version_du_module: Some("32.0.16.1074".into()),
            code: Some("c0000005".into()),
            decalage: Some("0000000000a48b17".into()),
        }
    }

    /// **La session se reconnaît sans seuil** : le même numéro, un processus créé au plus tard
    /// au début de la session, tombé au plus tôt à ce début. Celui qui portait ce numéro avant
    /// elle, celui qui l'a porté après, un autre numéro : aucun n'est elle.
    #[test]
    fn test_la_session_se_reconnait_sans_seuil() {
        let debut = 1_000_000;
        let le_sien = vu(42, debut - 300, debut + 60_000);
        let p = reconnaitre(std::slice::from_ref(&le_sien), 42, debut).expect("le sien");
        assert_eq!(p.module.as_deref(), Some("nvoglv64.dll"));
        assert_eq!(p.version_du_module.as_deref(), Some("32.0.16.1074"));
        assert_eq!(p.code, Some(0xC000_0005));
        assert_eq!(p.decalage, Some(0x00a4_8b17));
        assert!(!p.gel);
        let avant = vu(42, debut - 9_000, debut - 1);
        let apres = vu(42, debut + 1, debut + 5_000);
        let autre = vu(43, debut - 300, debut + 60_000);
        assert_eq!(reconnaitre(&[avant, apres, autre], 42, debut), None);
        // Aux bornes mêmes : créé à l'instant du début, tombé à cet instant.
        assert!(reconnaitre(&[vu(42, debut, debut)], 42, debut).is_some());
    }

    /// **S'il en a noté plusieurs, le plus tardif** ; et un gel se dit gel.
    #[test]
    fn test_le_plus_tardif_l_emporte() {
        let debut = 1_000_000;
        let mut gel = vu(42, debut - 300, debut + 90_000);
        gel.gel = true;
        let p = reconnaitre(&[vu(42, debut - 300, debut + 60_000), gel], 42, debut).unwrap();
        assert!(p.gel);
        assert_eq!(p.dire(), "gelée, puis fermée par le système");
    }

    /// **Un nom qui pourrait porter un dossier de l'utilisateur ne se dit pas** : ni chemin, ni
    /// espace ; le reste se lit, et un code illisible reste inconnu.
    #[test]
    fn test_un_nom_qui_n_est_pas_sur_ne_se_dit_pas() {
        let mut e = vu(1, 0, 10);
        e.module = Some(r"C:\Users\camille\AppData\x.dll".into());
        e.version_du_module = Some("1.0 bêta".into());
        e.code = Some("pas un code".into());
        let p = reconnaitre(&[e], 1, 5).unwrap();
        assert_eq!(p.module, None);
        assert_eq!(p.version_du_module, None);
        assert_eq!(p.code, None);
        assert_eq!(p.decalage, Some(0x00a4_8b17));
        assert_eq!(p.dire(), "plantée");
    }

    /// **Ce qu'on en dit** : le module, sa version, le sens du code quand on le connaît.
    #[test]
    fn test_ce_qu_on_dit_d_un_plantage() {
        let p = reconnaitre(&[vu(7, 0, 10)], 7, 5).unwrap();
        assert_eq!(
            p.dire(),
            "plantée dans nvoglv64.dll 32.0.16.1074 (violation d'accès à la mémoire, c0000005)"
        );
    }
}
