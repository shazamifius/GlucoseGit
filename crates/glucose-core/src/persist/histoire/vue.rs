//! La **vue** : où l'on regardait — tableau actif, caméra et signets de chaque tableau.
//!
//! La navigation n'est pas un geste (UNDO-1, fiche 05 § 3.5) : se déplacer ne pollue jamais
//! l'annulation, donc jamais l'histoire non plus. Mais rouvrir un document doit remettre
//! l'utilisateur **là où il était** — Glucose Tauri le faisait, parce que sa caméra vivait dans
//! le document. Une entrée « vue » l'écrit à part, quand il le faut (en posant un jalon, en
//! fermant), et la relecture l'applique par-dessus l'état rejoué.

use super::super::bytes::{Reader, Writer};
use super::super::document::{read_bookmarks, read_viewport, write_bookmarks, write_viewport};
use crate::error::CoreResult;
use crate::types::{Project, Viewport};
use std::collections::HashMap;

/// Ce que la navigation a posé sur un document.
#[derive(Debug, Clone, PartialEq)]
pub struct Vue {
    pub tableau_actif: String,
    /// Pour chaque tableau, par son identifiant : sa caméra et ses signets.
    pub tableaux: Vec<(String, Viewport, HashMap<String, Viewport>)>,
}

impl Vue {
    /// La vue d'un projet, telle qu'elle est.
    pub fn de(projet: &Project) -> Self {
        Self {
            tableau_actif: projet.active_board_id.clone(),
            tableaux: projet
                .boards
                .iter()
                .map(|b| (b.id.clone(), b.viewport, b.bookmarks.clone()))
                .collect(),
        }
    }

    /// Pose la vue sur un projet. Un tableau qui n'existe plus est ignoré, et le tableau actif
    /// ne se pose que s'il existe : la vue suit le document, jamais l'inverse.
    pub fn poser(&self, projet: &mut Project) {
        for (id, camera, signets) in &self.tableaux {
            if let Some(b) = projet.boards.iter_mut().find(|b| b.id == *id) {
                b.viewport = camera.normalized();
                b.bookmarks = signets.clone();
            }
        }
        if projet.boards.iter().any(|b| b.id == self.tableau_actif) {
            projet.active_board_id = self.tableau_actif.clone();
        }
    }
}

pub fn ecrire(w: &mut Writer, vue: &Vue) {
    w.text(&vue.tableau_actif);
    w.seq(&vue.tableaux, |ww, (id, camera, signets)| {
        ww.text(id);
        write_viewport(ww, camera);
        write_bookmarks(ww, signets);
    });
}

pub fn lire(r: &mut Reader<'_>) -> CoreResult<Vue> {
    Ok(Vue {
        tableau_actif: r.text()?,
        tableaux: r.seq(|rr| Ok((rr.text()?, read_viewport(rr)?, read_bookmarks(rr)?)))?,
    })
}
