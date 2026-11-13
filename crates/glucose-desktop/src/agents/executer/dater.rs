//! **DATE-1 — dater des nœuds, pour un agent** (fiche 113) : le geste de `Maj+T`, sur des
//! identifiants plutôt qu'une sélection. La date se dit dans la langue de la question de
//! l'humain ([`crate::interactions::dater::AIDE`]) ; une date que Glucose ne sait pas lire est
//! refusée avec ce qu'on avait écrit, jamais devinée.

use super::Args;
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::timeline::{parse_anchor, DEFAULT_ERAS};

impl GlucoseApp {
    /// **Date ces nœuds**, ou retire leur date si `date` manque ou est vide — un geste signé de
    /// l'agent, qu'un `Ctrl+Z` défait. Rend combien ont changé.
    pub(super) fn agent_dater(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let ids: Vec<String> =
            a.0.get("ids")
                .and_then(Json::as_liste)
                .ok_or("l'argument « ids » est une liste d'identifiants")?
                .iter()
                .filter_map(Json::as_str)
                .map(str::to_string)
                .collect();
        let ecrit = a.texte("date").unwrap_or("").trim();
        let ancre = match ecrit {
            "" => None,
            _ => Some(parse_anchor(ecrit, DEFAULT_ERAS).ok_or_else(|| {
                format!(
                    "Glucose ne sait pas lire « {ecrit} ». {}",
                    crate::interactions::dater::AIDE
                )
            })?),
        };
        let n = self
            .store
            .geste_d_agent(|s| s.dater_pour_un_agent(board, &ids, ancre.as_ref()))
            .map_err(|r| r.to_string())?;
        Ok(Json::objet([("dates", Json::Nombre(n as f64))]))
    }
}
