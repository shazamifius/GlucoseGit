//! **La flèche sous la souris, et ce qu'elle désigne** (FLECHE-4).
//!
//! Tauri faisait briller, au survol d'une flèche, les passages qu'elle ancre dans ses deux
//! cartes (`glucose:hover-arrow`). Ici la souris suit la flèche qu'elle survole ; l'image
//! demande ensuite quels passages éclairer — calculés au dessin, jamais gardés : une édition
//! de la carte déplace le passage, et rien n'est à invalider.

use crate::app::GlucoseApp;
use crate::params::Eclairage;
use crate::ui::ActiveTool;
use glucose_core::hit_priority::PickKind;
use glucose_core::text_anchors::resolve_text_sel;
use glucose_core::types::Annotation;

impl GlucoseApp {
    /// **Suit ce que la souris survole** — la flèche (ce qu'elle ancre brille, FLECHE-4) et le
    /// nœud (ses liens se montrent pleins, LIEN-1) : l'arbitre de clic est seul juge de ce
    /// qu'elle vise, pour que ce qui s'allume soit ce qu'un clic sélectionnerait. Une seule
    /// visée pour les deux.
    pub(crate) fn suivre_le_survol(&mut self) {
        let vp = self.store.viewport();
        let (wx, wy) = crate::canvas::screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        let candidat = self.pick_candidate_at(wx, wy);
        let fleche = candidat
            .as_ref()
            .filter(|c| c.kind == PickKind::Arrow)
            .map(|c| c.id.clone());
        if fleche != self.ui.fleche_survolee {
            self.ui.fleche_survolee = fleche;
            self.mark_dirty();
        }
        let noeud = candidat.map(|c| c.id);
        if noeud != self.ui.noeud_survole {
            let avant = std::mem::replace(&mut self.ui.noeud_survole, noeud);
            let maintenant = self.ui.noeud_survole.as_deref();
            if self.le_survol_change_un_lien([avant.as_deref(), maintenant]) {
                self.mark_dirty();
            }
        }
    }

    /// **Le survol change-t-il l'aspect d'un lien ?** Seulement si les liens ne sont pas déjà
    /// pleins à cette distance, et si l'un des deux nœuds — quitté, atteint — en touche un
    /// (LIEN-1) : survoler un nœud sans lien ne redessine rien.
    fn le_survol_change_un_lien(&self, noeuds: [Option<&str>; 2]) -> bool {
        let echelle = crate::renderer::scale::WorldScale::new(
            self.store.viewport().scale,
            self.densite() as f32,
        );
        let Some(board) = self.store.active_board() else {
            return false;
        };
        echelle.lecture() < 1.0
            && noeuds
                .into_iter()
                .flatten()
                .any(|n| glucose_core::arrow::touche_un_lien(board, n))
    }

    /// **Suit le nœud dont une flèche partirait**, sous l'outil Flèche armé : l'image ne se
    /// redessine que quand il change.
    pub(crate) fn suivre_le_noeud_pressenti(&mut self) {
        let pressenti = if self.ui.active_tool == ActiveTool::Arrow {
            let vp = self.store.viewport();
            let point = crate::canvas::screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
            self.snap_for_arrow(point, &[]).node
        } else {
            None
        };
        if pressenti != self.ui.noeud_pressenti {
            self.ui.noeud_pressenti = pressenti;
            self.mark_dirty();
        }
    }

    /// **Suit les cartes désignées, et rend leur vivacité à cet instant** (LUEUR-2) : ce que
    /// l'image montre, et ce qui dit au réveil qu'une lueur glisse encore.
    pub(crate) fn suivre_la_designation(&mut self) -> Vec<(String, f32)> {
        let maintenant = self.now_ms() as f64;
        let designees = self.cartes_designees();
        self.vivacites.cartes.suivre(&designees, maintenant);
        self.vivacites.cartes.vivacites(maintenant)
    }

    /// **Suit la pastille de relation sous la souris, et rend l'effacement de chacune**
    /// (BADGE-1) : la survolée s'efface en deux cents millisecondes, celle qu'on quitte
    /// revient de même.
    pub(crate) fn suivre_les_badges(&mut self) -> Vec<(String, f32)> {
        let maintenant = self.now_ms() as f64;
        let survolee: Vec<String> = self.badge_sous_la_souris().into_iter().collect();
        self.vivacites.badges.suivre(&survolee, maintenant);
        self.vivacites.badges.vivacites(maintenant)
    }

    /// **La flèche dont la pastille de relation est sous la souris** — par la géométrie même du
    /// dessin ([`crate::renderer::predicate::centre_du_badge`]), et parmi les seules flèches que
    /// l'index spatial trouve autour du point : aucun parcours du tableau.
    fn badge_sous_la_souris(&self) -> Option<String> {
        use crate::renderer::predicate::{centre_du_badge, BADGE_RAYON};
        let board = self.store.active_board()?;
        let vp = self.store.viewport();
        let (wx, wy) = crate::canvas::screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        let rayon = f64::from(BADGE_RAYON);
        let noeuds = crate::renderer::arrow::NoeudsDuRendu {
            board,
            index: Some(&self.renderer.spatial_hash),
            typographie: &self.renderer.typography,
            math: &self.renderer.math,
            contournement: Some(self.renderer.contournement()),
        };
        let rangs = self
            .renderer
            .spatial_hash
            .query_rect_ranks(wx, wy, wx, wy, rayon);
        rangs.into_iter().find_map(|rang| {
            let Some(glucose_core::quadtree::Noeud::Annotation(fleche)) =
                glucose_core::quadtree::noeud_au_rang(board, rang)
            else {
                return None;
            };
            let morceaux = glucose_core::arrow::morceaux_with(fleche, noeuds)?;
            let milieu = glucose_core::arrow::milieu_du_trace(&morceaux)?;
            let editing = self
                .editing_session
                .as_ref()
                .filter(|s| s.ann_id == fleche.id());
            let etiquette = crate::renderer::arrow_label::montre_une_etiquette(fleche, editing);
            let (cx, cy) = centre_du_badge(fleche, milieu, etiquette)?;
            ((cx - wx).hypot(cy - wy) <= rayon).then(|| fleche.id().to_string())
        })
    }

    /// **Les cartes dont la lueur s'avive dans cette image** (LUEUR-1) — ce à quoi une flèche
    /// va se lier, ou se lie :
    ///
    /// * la flèche qui naît sous la main : ce que sa pointe vise ;
    /// * l'outil Flèche armé : le nœud dont elle partirait ;
    /// * une flèche survolée : ceux de ses bouts qui ne désignent pas de passage — un passage
    ///   ancré brille à sa place, lui seul.
    pub(crate) fn cartes_designees(&self) -> Vec<String> {
        let Some(board) = self.store.active_board() else {
            return Vec::new();
        };
        if let Some(session) = &self.draw_session {
            let visee = match self.store.project.annotation(&board.id, &session.id) {
                Some(Annotation::Arrow { target_id, .. }) => target_id.clone(),
                _ => None,
            };
            return visee.into_iter().collect();
        }
        // L'outil rendu, ce qu'il pressentait ne vaut plus — même avant que la souris bouge.
        if self.ui.active_tool == ActiveTool::Arrow {
            return self.ui.noeud_pressenti.iter().cloned().collect();
        }
        let Some(Annotation::Arrow {
            source_id,
            target_id,
            source_text_sel,
            target_text_sel,
            ..
        }) = self
            .ui
            .fleche_survolee
            .as_ref()
            .and_then(|id| self.store.project.annotation(&board.id, id))
        else {
            return Vec::new();
        };
        [(source_id, source_text_sel), (target_id, target_text_sel)]
            .into_iter()
            .filter(|(_, passage)| {
                !glucose_core::text_anchors::has_text_selection(passage.as_ref())
            })
            .filter_map(|(bout, _)| bout.clone())
            .collect()
    }

    /// **Les passages à faire briller dans cette image** : ceux que la flèche survolée ancre,
    /// dans sa source et dans sa cible, chacun à la couleur de sa carte.
    pub(crate) fn eclairages(&self) -> Vec<Eclairage> {
        // L'éditeur d'ancres ouvert, c'est ce qu'on choisit qui brille.
        if let Some(choisi) = self.eclairages_de_l_ancrage() {
            return choisi;
        }
        let (Some(id), Some(board)) = (&self.ui.fleche_survolee, self.store.active_board()) else {
            return Vec::new();
        };
        let Some(Annotation::Arrow {
            source_id,
            target_id,
            source_text_sel,
            target_text_sel,
            ..
        }) = self.store.project.annotation(&board.id, id)
        else {
            return Vec::new();
        };
        [(source_id, source_text_sel), (target_id, target_text_sel)]
            .into_iter()
            .filter_map(|(carte, passage)| {
                let carte = carte.as_ref()?;
                let texte = self
                    .store
                    .project
                    .annotation(&board.id, carte)?
                    .own_text()?;
                let plages: Vec<(usize, usize)> = resolve_text_sel(&texte, passage.as_ref())
                    .into_iter()
                    .map(|p| (p.start, p.end))
                    .collect();
                (!plages.is_empty()).then(|| Eclairage {
                    carte: carte.clone(),
                    plages,
                    teinte: None,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
