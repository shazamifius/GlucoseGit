//! **`glucose_grouper`** (GROUPER-1, fiche 98) : entourer des nœuds d'une membrane neuve — le geste
//! « Grouper dans une membrane » de l'humain (`Ctrl+G`), pour un agent, et donc `glucose grouper …`
//! au terminal (TERMINAL-1). Ce que l'humain fait, une IA le fait aussi.

use super::{pose, Args};
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::types::Annotation;

impl GlucoseApp {
    /// **Entoure les nœuds `ids`**, en un geste (AGENT-1) ; le titre et la couleur s'il les dit.
    /// Un identifiant inconnu est refusé, en le nommant ; rien sans boîte, aussi.
    pub(super) fn agent_grouper(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let ids: Vec<String> =
            a.0.get("ids")
                .and_then(Json::as_liste)
                .ok_or("l'argument « ids » est une liste d'identifiants")?
                .iter()
                .filter_map(Json::as_str)
                .map(str::to_string)
                .collect();
        let (titre, couleur) = (a.texte("titre").map(str::to_string), a.couleur()?);
        let groupe = self
            .store
            .geste_d_agent(|s| {
                let g = s.grouper_pour_un_agent(board, &ids)?;
                s.update_annotation(board, &g, |m| {
                    if let Annotation::Membrane { text, color, .. } = m {
                        (*text, *color) = (titre, couleur);
                    }
                });
                Ok(g)
            })
            .map_err(|r| r.to_string())?;
        let boite = self
            .store
            .project
            .annotation(board, &groupe)
            .map(Annotation::bounds)
            .ok_or("la membrane a disparu")?;
        Ok(pose(&groupe, boite))
    }
}
