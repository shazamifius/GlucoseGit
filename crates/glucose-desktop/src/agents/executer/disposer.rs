//! **`glucose_disposer`** (COLONNES-1, fiche 92) : ranger en colonnes ce que porte une membrane —
//! le geste même du menu « Disposer en colonnes », pour un agent, et donc `glucose disposer …` au
//! terminal (TERMINAL-1). Un agent qui vient de poser un long contenu le rend lisible.

use super::{boite_en_json, Args};
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::store::agents::Refus;
use glucose_core::types::Annotation;

impl GlucoseApp {
    /// **Range la membrane `dans`**, en un geste (AGENT-1) ; la proportion de l'écran par défaut.
    pub(super) fn agent_disposer(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let membrane = a.requis("dans")?;
        let proportion = a
            .nombre("proportion")
            .unwrap_or_else(|| self.proportion_de_l_ecran());
        if !matches!(
            self.store.project.annotation(board, membrane),
            Some(Annotation::Membrane { .. })
        ) {
            return Err(format!(
                "« {membrane} » n'est pas une membrane de ce tableau"
            ));
        }
        let n = self
            .store
            .geste_d_agent(|s| Ok::<_, Refus>(s.disposer_en_colonnes(board, membrane, proportion)))
            .map_err(|r| r.to_string())?;
        let boite = self
            .store
            .project
            .annotation(board, membrane)
            .map(Annotation::bounds)
            .ok_or("la membrane a disparu")?;
        Ok(Json::objet([
            ("ranges", Json::Nombre(n as f64)),
            ("boite", boite_en_json(boite)),
        ]))
    }
}
