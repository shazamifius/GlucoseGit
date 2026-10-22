//! **La main d'épreuve** du Glucose sans fenêtre : les gestes d'un utilisateur, lus dans un
//! fichier et joués par les chemins de la main — jamais par ceux d'un agent —, pour éprouver ce
//! qu'un agent voit de ce que l'humain fait (fiche 80) sans personne devant l'écran. Elle n'existe
//! que pour les bancs, comme la réponse « Autoriser » de [`super::porte`].
//!
//! Une ligne JSON par geste, ajoutée au fichier pendant que le banc tourne :
//!
//! ```text
//! {"selectionner": ["id", …]}                           sa sélection (vide : plus rien)
//! {"recrire": {"id": "…", "texte": "…", "tableau": "…"}} il récrit un texte (un autre onglet, s'il le dit)
//! {"poser_post_it": {"id": "…", "texte": "…", "x": 0, "y": 0}}
//! {"retirer": ["id", …]}
//! ```

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::types::Annotation;
use std::path::PathBuf;

/// Le fichier des gestes, et ce qui en a déjà été joué.
pub struct MainDEpreuve {
    chemin: PathBuf,
    joues: usize,
}

impl MainDEpreuve {
    pub fn nouvelle(chemin: PathBuf) -> Self {
        MainDEpreuve { chemin, joues: 0 }
    }

    /// Joue les gestes neufs (les lignes complètes qui n'ont pas encore été jouées), et dit
    /// chacun sur la sortie. Rend combien il en a joué.
    pub fn jouer(&mut self, app: &mut GlucoseApp) -> usize {
        let Ok(texte) = std::fs::read_to_string(&self.chemin) else {
            return 0;
        };
        let lignes: Vec<&str> = texte
            .split_inclusive('\n')
            .filter(|l| l.ends_with('\n'))
            .collect();
        let neufs = lignes.len().saturating_sub(self.joues);
        for ligne in &lignes[self.joues..] {
            let issue = Json::lire(ligne.trim())
                .and_then(|g| geste(app, &g))
                .unwrap_or_else(|e| format!("refusé : {e}"));
            println!("[main] {issue}");
        }
        self.joues = lignes.len();
        neufs
    }
}

fn ids(v: &Json) -> Vec<String> {
    v.as_liste()
        .unwrap_or(&[])
        .iter()
        .filter_map(Json::as_str)
        .map(str::to_string)
        .collect()
}

/// Un geste de la main : la sélection n'entre pas au journal (UNDO-1) ; le reste est un geste,
/// comme ceux qu'on fait à la souris et au clavier.
fn geste(app: &mut GlucoseApp, g: &Json) -> Result<String, String> {
    let board = app.store.project.active_board_id.clone();
    let s = &mut app.store;
    if let Some(l) = g.get("selectionner") {
        let choisis = ids(l);
        s.clear_selection();
        s.set_selected_annotation_ids(choisis.clone());
        return Ok(format!("sélectionne {choisis:?}"));
    }
    if let Some(r) = g.get("recrire") {
        let id = r
            .get("id")
            .and_then(Json::as_str)
            .ok_or("recrire sans id")?;
        let texte = r.get("texte").and_then(Json::as_str).unwrap_or("");
        // Un autre onglet, s'il le dit : il y corrige peut-être le carnet d'un agent.
        let ou = r.get("tableau").and_then(Json::as_str).unwrap_or(&board);
        s.begin_live_edit();
        s.ecrire_le_texte(ou, id, texte);
        s.end_live_edit();
        return Ok(format!("récrit {id}"));
    }
    if let Some(p) = g.get("poser_post_it") {
        let id = p
            .get("id")
            .and_then(Json::as_str)
            .ok_or("post-it sans id")?;
        let texte = p.get("texte").and_then(Json::as_str).unwrap_or("");
        let (x, y) = (
            p.get("x").and_then(Json::as_f64).unwrap_or(0.0),
            p.get("y").and_then(Json::as_f64).unwrap_or(0.0),
        );
        s.begin_live_edit();
        s.add_annotation(&board, Annotation::sticky(id, x, y, texte));
        s.end_live_edit();
        return Ok(format!("pose le post-it {id} en ({x}, {y})"));
    }
    if let Some(l) = g.get("retirer") {
        let choisis = ids(l);
        let refs: Vec<&str> = choisis.iter().map(String::as_str).collect();
        s.begin_live_edit();
        s.remove_annotations(&board, &refs);
        s.end_live_edit();
        return Ok(format!("retire {choisis:?}"));
    }
    Err("geste inconnu".into())
}
