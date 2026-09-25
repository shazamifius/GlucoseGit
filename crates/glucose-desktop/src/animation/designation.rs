//! **La lueur d'une carte désignée s'avive, puis s'éteint** (LUEUR-2).
//!
//! Chez Tauri, la carte qu'une flèche vise passe de sa lueur de repos à une lueur vive en
//! 0,2 s (`transition: box-shadow 0.2s`). Ici, la désignation sautait d'un état à l'autre : il
//! demandait des indices visuels *et des animations* pendant qu'on tire une flèche.
//!
//! Une carte n'a donc plus deux états mais une **vivacité**, de 0 au repos à 1 désignée, qui
//! glisse en deux cents millisecondes sur l'amorti universel de Glucose. Une carte qui cesse
//! d'être désignée repart de là où elle en était : une souris qui passe vite d'une carte à
//! l'autre ne fait pas clignoter la première.
//!
//! La vivacité est une **fonction du temps écoulé**, sans horloge : ce module se prouve sans
//! attendre, comme les courbes du noyau (`glucose_core::anim`).

use glucose_core::anim::{timing, Curve};
use std::collections::HashMap;

/// Une carte en transition : d'où sa vivacité part, où elle va, et depuis quand.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Transition {
    depart: f32,
    cible: f32,
    debut_ms: f64,
}

impl Transition {
    fn vivacite(&self, ms: f64) -> f32 {
        let t = (ms - self.debut_ms) / f64::from(timing::DESIGNATION_MS);
        let avance = Curve::EaseOutCubic.at(t) as f32;
        self.depart + (self.cible - self.depart) * avance
    }

    fn finie(&self, ms: f64) -> bool {
        ms - self.debut_ms >= f64::from(timing::DESIGNATION_MS)
    }
}

/// Les cartes désignées, et celles qui s'éteignent encore.
#[derive(Debug, Default)]
pub struct Designation {
    cartes: HashMap<String, Transition>,
}

impl Designation {
    /// **Suit l'ensemble désigné à l'instant `ms`** : une carte qui entre s'avive depuis sa
    /// vivacité du moment, une carte qui sort s'éteint depuis la sienne, et celle qui s'est
    /// éteinte tout à fait est oubliée.
    pub fn suivre(&mut self, designees: &[String], ms: f64) {
        for id in designees {
            let deja = self.cartes.get(id);
            if deja.is_some_and(|t| t.cible == 1.0) {
                continue;
            }
            let depart = deja.map_or(0.0, |t| t.vivacite(ms));
            self.cartes.insert(
                id.clone(),
                Transition {
                    depart,
                    cible: 1.0,
                    debut_ms: ms,
                },
            );
        }
        for (id, t) in &mut self.cartes {
            if t.cible == 1.0 && !designees.contains(id) {
                *t = Transition {
                    depart: t.vivacite(ms),
                    cible: 0.0,
                    debut_ms: ms,
                };
            }
        }
        self.cartes.retain(|_, t| t.cible > 0.0 || !t.finie(ms));
    }

    /// **La vivacité de chaque carte encore allumée**, à l'instant `ms`.
    pub fn vivacites(&self, ms: f64) -> Vec<(String, f32)> {
        self.cartes
            .iter()
            .map(|(id, t)| (id.clone(), t.vivacite(ms)))
            .filter(|(_, v)| *v > 0.0)
            .collect()
    }

    /// Une transition court-elle encore à l'instant `ms` ?
    pub fn en_cours(&self, ms: f64) -> bool {
        self.cartes.values().any(|t| !t.finie(ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vivacite(d: &Designation, id: &str, ms: f64) -> f32 {
        d.vivacites(ms)
            .into_iter()
            .find(|(c, _)| c == id)
            .map_or(0.0, |(_, v)| v)
    }

    /// **Une carte désignée s'avive en deux cents millisecondes**, vive d'abord, posée à la fin.
    #[test]
    fn test_lueur_2_une_carte_designee_s_avive() {
        let mut d = Designation::default();
        d.suivre(&["a".into()], 1_000.0);
        assert_eq!(vivacite(&d, "a", 1_000.0), 0.0);
        let mi = vivacite(&d, "a", 1_100.0);
        assert!(
            mi > 0.8 && mi < 1.0,
            "à mi-course, l'amorti a déjà fait l'essentiel : {mi}"
        );
        assert_eq!(vivacite(&d, "a", 1_200.0), 1.0);
        assert!(d.en_cours(1_150.0) && !d.en_cours(1_200.0));
    }

    /// **Une carte qui cesse d'être désignée s'éteint depuis où elle en était**, puis s'oublie.
    #[test]
    fn test_lueur_2_une_carte_quittee_s_eteint_sans_sauter() {
        let mut d = Designation::default();
        d.suivre(&["a".into()], 0.0);
        let avant = vivacite(&d, "a", 50.0);
        d.suivre(&[], 50.0);
        assert_eq!(
            vivacite(&d, "a", 50.0),
            avant,
            "pas de saut à l'instant du départ"
        );
        assert!(vivacite(&d, "a", 150.0) < avant);
        d.suivre(&[], 300.0);
        assert!(d.vivacites(300.0).is_empty() && !d.en_cours(300.0));
        assert!(
            d.cartes.is_empty(),
            "oubliée : la table ne grandit pas de chaque carte jamais survolée"
        );
    }

    /// Suivre deux fois le même ensemble ne redémarre rien : la souris qui bouge sur la carte
    /// visée ne la fait pas clignoter.
    #[test]
    fn test_lueur_2_le_meme_ensemble_ne_redemarre_rien() {
        let mut d = Designation::default();
        d.suivre(&["a".into()], 0.0);
        d.suivre(&["a".into()], 100.0);
        assert_eq!(vivacite(&d, "a", 200.0), 1.0);
    }
}
