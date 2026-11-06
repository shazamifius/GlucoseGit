//! **Un scénario minuté, joué par le vrai Glucose** (`GLUCOSE_SCENARIO=<fichier>`, fiche 86) : sa
//! vraie fenêtre, sa vraie carte graphique, ses vraies commandes, et sa chronique à la fin.
//!
//! Les bancs hors écran ne voient ni la présentation ni le coût de ce qui se joue entre deux
//! images ; simuler le clavier enverrait les touches à ce qui a le focus sur son bureau. Un
//! instrument, comme `GLUCOSE_CARTE`, et non un choix de production. Une ligne par geste :
//!
//! ```text
//! 3 signet.poser.1
//! 4.5 tout-cadrer
//! 8 voler 0 0 1.5
//! 12 quitter
//! ```

use super::GlucoseApp;
use crate::commandes::Commande;
use std::collections::VecDeque;
use std::time::Instant;

/// Ce qui reste à jouer.
#[derive(Debug, Default)]
pub struct Scenario {
    depart: Option<Instant>,
    gestes: VecDeque<(f64, String)>,
}

impl Scenario {
    /// Le scénario de ce texte. Une ligne qui ne se lit pas s'arrête ici, et non en plein essai.
    pub fn lire(texte: &str) -> Result<Self, String> {
        let mut gestes = VecDeque::new();
        for ligne in texte.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let (quand, quoi) = ligne
                .split_once(char::is_whitespace)
                .ok_or_else(|| format!("« {ligne} » : il faut un instant et un geste"))?;
            let quand: f64 = quand
                .parse()
                .map_err(|_| format!("« {quand} » n'est pas un instant"))?;
            let quoi = quoi.trim();
            if quoi != "quitter" && vue_de(quoi).is_none() && Commande::de_l_id(quoi).is_none() {
                return Err(format!("« {quoi} » n'est pas une commande"));
            }
            gestes.push_back((quand, quoi.to_string()));
        }
        Ok(Self {
            depart: None,
            gestes,
        })
    }

    /// Le geste dû à cet instant, s'il y en a un : le départ est la première question posée.
    fn du(&mut self, maintenant: Instant) -> Option<String> {
        let depart = *self.depart.get_or_insert(maintenant);
        let (quand, _) = self.gestes.front()?;
        (maintenant.duration_since(depart).as_secs_f64() >= *quand)
            .then(|| self.gestes.pop_front().map(|(_, quoi)| quoi))
            .flatten()
    }
}

/// `voler <x> <y> <échelle>` : le même vol que `F` et les signets, vers cette vue.
fn vue_de(geste: &str) -> Option<(f64, f64, f64)> {
    let mut mots = geste.strip_prefix("voler ")?.split_whitespace();
    let mut suivant = || mots.next()?.parse().ok();
    Some((suivant()?, suivant()?, suivant()?))
}

impl GlucoseApp {
    /// **Lit le scénario demandé**, et réveille la boucle tant qu'il dure : au repos, elle dort.
    pub fn charger_le_scenario(&mut self) {
        let Some(chemin) = std::env::var_os("GLUCOSE_SCENARIO") else {
            return;
        };
        let lu = std::fs::read_to_string(&chemin)
            .map_err(|e| e.to_string())
            .and_then(|t| Scenario::lire(&t));
        match lu {
            Ok(scenario) => self.lancement.scenario = Some(scenario),
            Err(e) => {
                eprintln!("[Glucose] scénario illisible : {e}");
                return;
            }
        }
        if let Some(reveil) = self.lancement.reveil.clone() {
            let _ = std::thread::Builder::new()
                .name("scenario".into())
                .spawn(move || {
                    while reveil.send_event(()).is_ok() {
                        std::thread::sleep(std::time::Duration::from_millis(20));
                    }
                });
        }
    }

    /// **Joue ce qui est dû** — une fois la fenêtre là. Rend `true` s'il faut quitter.
    pub(super) fn jouer_le_scenario(&mut self) -> bool {
        if self.window.is_none() {
            return false;
        }
        let Some(scenario) = self.lancement.scenario.as_mut() else {
            return false;
        };
        let Some(geste) = scenario.du(Instant::now()) else {
            return false;
        };
        if geste == "quitter" {
            return true;
        }
        let debut = Instant::now();
        if let Some((x, y, scale)) = vue_de(&geste) {
            let mut vue = self.store.viewport();
            (vue.x, vue.y, vue.scale) = (x, y, scale);
            self.vol.voler_vers(vue);
        } else if let Some(commande) = Commande::de_l_id(&geste) {
            self.executer(commande);
        }
        println!("[Glucose] scénario : {geste} en {:?}", debut.elapsed());
        self.mark_dirty();
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_scenario_une_ligne_fausse_s_arrete_a_la_lecture() {
        assert!(Scenario::lire("3 tout-cadrer\n12 quitter\n").is_ok());
        assert!(Scenario::lire("3 pas-une-commande").is_err());
        assert!(Scenario::lire("bientot tout-cadrer").is_err());
        assert!(Scenario::lire("tout-cadrer").is_err());
        assert!(Scenario::lire("3 voler 0 0 1.5").is_ok());
        assert!(
            Scenario::lire("3 voler 0 0").is_err(),
            "il faut les trois nombres"
        );
    }

    #[test]
    fn test_scenario_chaque_geste_part_a_son_heure_et_une_seule_fois() {
        let mut s = Scenario::lire("1 tout-cadrer\n2 quitter\n").expect("lu");
        let t = Instant::now();
        assert_eq!(s.du(t), None, "le départ est posé, rien n'est dû");
        assert_eq!(
            s.du(t + Duration::from_millis(1100)).as_deref(),
            Some("tout-cadrer")
        );
        assert_eq!(
            s.du(t + Duration::from_millis(1200)),
            None,
            "une seule fois"
        );
        assert_eq!(s.du(t + Duration::from_secs(3)).as_deref(), Some("quitter"));
    }
}
