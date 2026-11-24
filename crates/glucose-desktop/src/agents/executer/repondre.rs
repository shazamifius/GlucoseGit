//! **`glucose_repondre`** (MCP-REPONDRE, sa liste du 24/11, `relais-rep-mcp` point 4) : répondre à
//! un nœud en un seul geste — le texte posé à la place libre la plus proche
//! ([`glucose_core::store::Store::place_pour_une_reponse`]), dans la membrane de ce nœud, et la
//! flèche « réponse » qui va de lui à ce nœud. Un seul `Ctrl+Z` défait les deux. Le même `id`
//! récrit la réponse là où elle est.

use super::{partir, pose, Args};
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use crate::renderer::card::text_card_fit_height;
use glucose_core::geometry::Rect;
use glucose_core::store::agents::Lien;
use glucose_core::types::{Annotation, DEFAULT_TEXT_CARD_WIDTH};

impl GlucoseApp {
    pub(super) fn agent_repondre(&mut self, board: &str, a: &Args<'_>) -> Result<Json, String> {
        let (cible, texte) = (a.requis("a")?, a.requis("texte")?);
        let id = self.id_d_agent(a, "reponse");
        let fleche = format!("{id}-fleche");
        let avant = self.boite_existante(board, &id);
        let largeur = a
            .nombre("largeur")
            .or(avant.map(|b| b.width))
            .unwrap_or(DEFAULT_TEXT_CARD_WIDTH);
        let (typo, math) = (&self.renderer.typography, &self.renderer.math);
        let hauteur = text_card_fit_height(typo, math, texte, largeur);
        let boite = self
            .store
            .geste_d_agent(|s| {
                let (x, y, dans) = match avant {
                    Some(b) => (b.left, b.top, None),
                    None => {
                        let p = s.place_pour_une_reponse(board, cible, (largeur, hauteur))?;
                        (p.x, p.y, p.membrane)
                    }
                };
                let boite = Rect::new(x, y, largeur, hauteur);
                if let Some(m) = dans.as_deref() {
                    s.agrandir_la_membrane_pour(board, m, boite)?;
                }
                let ann = partir::un_texte(s, board, Annotation::text(&id, x, y, texte), boite);
                s.poser_pour_un_agent(board, ann, dans.as_deref())?;
                s.relier_pour_un_agent(
                    board,
                    Lien {
                        id: &fleche,
                        source: &id,
                        cible,
                        passage_source: None,
                        passage_cible: None,
                        relation: None,
                        texte: Some("réponse"),
                    },
                )?;
                Ok(boite)
            })
            .map_err(|r| r.to_string())?;
        let mut dit = pose(&id, boite);
        if let Json::Objet(champs) = &mut dit {
            champs.push(("fleche".to_string(), Json::from(fleche)));
        }
        Ok(dit)
    }
}
