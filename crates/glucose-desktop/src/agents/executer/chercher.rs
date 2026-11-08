//! **`glucose_chercher`** (fiche 90) : ce que `Ctrl+F` trouve, rendu à un agent.
//!
//! # AGENT-7 — l'agent cherche comme l'humain
//!
//! Sa règle la plus forte (VISION, 22/10) : *« un Glucose doit pouvoir être lu par un humain ET par
//! une IA, de quasi la même façon »*. Depuis la fiche 89, l'humain cherche dans **tout le document**,
//! sans majuscules ni accents, les débuts de mots d'abord ; l'agent, lui, ne cherchait que dans le
//! tableau qu'on regarde (`glucose_lire`, `cherche`). Cet outil lui donne **la même recherche** : le
//! même index ([`GlucoseApp::chercher_dans_le_document`]), donc les mêmes résultats, dans le même
//! ordre, et la même page de dix rangées que la palette — avec `depuis` pour la suite, comme
//! l'humain fait défiler. Seule la phrase n'est pas coupée : la coupe est une affaire d'écran.

use super::Args;
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use crate::persist::documents::MONTRES;
use glucose_core::recherche::passage_du;

impl GlucoseApp {
    /// **Cherche dans tout le document**, comme `Ctrl+F`, et rend une page de résultats.
    pub(super) fn agent_chercher(&mut self, a: &Args<'_>) -> Result<Json, String> {
        let requete = a.requis("cherche")?;
        let trouves = self.chercher_dans_le_document(requete);
        let entier = |cle: &str, defaut: usize| {
            a.nombre(cle)
                .filter(|n| *n >= 0.0)
                .map_or(defaut, |n| n as usize)
        };
        let depuis = entier("depuis", 0);
        let combien = entier("combien", MONTRES).max(1);
        let resultats = trouves
            .iter()
            .skip(depuis)
            .take(combien)
            .map(|t| {
                Json::objet([
                    ("tableau", Json::from(t.tableau.as_str())),
                    ("onglet", Json::from(self.store.nom_de_l_onglet(&t.tableau))),
                    ("id", Json::from(t.id.as_deref())),
                    ("sorte", Json::from(t.genre.nom())),
                    (
                        "phrase",
                        Json::from(passage_du(&self.store.project, t, requete)),
                    ),
                ])
            })
            .collect();
        Ok(Json::objet([
            ("cherche", Json::from(requete)),
            ("total", Json::from(trouves.len() as f64)),
            ("depuis", Json::from(depuis as f64)),
            ("resultats", Json::Liste(resultats)),
        ]))
    }
}

#[cfg(test)]
mod tests;
