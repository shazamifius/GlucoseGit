//! **Ce qu'une flèche demande au rendu** (FLECHE-4) : la boîte d'un nœud, retrouvée par
//! l'index, et la hauteur d'un passage de texte, mesurée par la vraie mise en page de la carte.
//!
//! Le dessin, les poignées et le clic passent tous par lui : une flèche ancrée au second
//! « bonjours » d'une carte part de ce mot, et se vise là où elle part (loi L4).

use crate::renderer::card::{card_text_layout, text_box, TEXT_ORIGIN};
use crate::renderer::math::MathRenderer;
use crate::renderer::richtext::{TextLayout, TextMode};
use crate::typography::Typography;
use glucose_core::arrow::{node_rect, node_rect_indexe, Noeuds};
use glucose_core::geometry::Rect;
use glucose_core::quadtree::{noeud_au_rang, Noeud, SpatialHash};
use glucose_core::text_anchors::resolve_anchors;
use glucose_core::types::{Annotation, Board, TextAnchor};

/// Le tableau tel que le rendu le connaît : ses nœuds, son index, ses polices.
#[derive(Clone, Copy)]
pub(crate) struct NoeudsDuRendu<'a> {
    pub board: &'a Board,
    /// L'index spatial synchronisé sur ce tableau, s'il y en a un : sans lui, les nœuds se
    /// cherchent dans le tableau.
    pub index: Option<&'a SpatialHash>,
    pub typographie: &'a Typography,
    pub math: &'a MathRenderer,
}

impl NoeudsDuRendu<'_> {
    /// L'annotation `id`, par l'index quand on l'a.
    fn annotation(&self, id: &str) -> Option<&Annotation> {
        let par_l_index = self
            .index
            .and_then(|i| noeud_au_rang(self.board, i.rang_de(id)?))
            .and_then(|n| match n {
                Noeud::Annotation(a) if a.id() == id => Some(a),
                _ => None,
            });
        par_l_index.or_else(|| self.board.annotations.iter().find(|a| a.id() == id))
    }
}

/// La ligne visuelle qui porte l'octet `b` : la dernière qui commence avant lui. Le début
/// d'une ligne repliée est sur cette ligne, pas sur celle d'avant.
fn ligne_qui_porte(mise_en_page: &TextLayout, b: usize) -> usize {
    mise_en_page
        .lines
        .iter()
        .rposition(|l| l.start <= b)
        .unwrap_or(0)
}

impl Noeuds for NoeudsDuRendu<'_> {
    fn boite(&self, id: &str) -> Option<Rect> {
        match self.index {
            Some(index) => node_rect_indexe(self.board, index, id),
            None => node_rect(self.board, id),
        }
    }

    /// Le milieu des lignes que le passage occupe, depuis le haut de la carte — la moyenne de
    /// leurs hauts et de leurs bas quand il y a plusieurs ancres, comme Tauri
    /// (`findTextSelPosition`). Une carte de texte seulement : c'est la seule qu'on met en page
    /// ici ; les autres nœuds sont visés par leur milieu.
    fn hauteur_du_passage(&self, id: &str, ancres: &[TextAnchor]) -> Option<f64> {
        let ann = self.annotation(id)?;
        let Annotation::Text { text, .. } = ann else {
            return None;
        };
        let plages = resolve_anchors(text, ancres);
        if plages.is_empty() {
            return None;
        }
        let (largeur, _) = ann.size()?;
        let largeur = largeur as f32;
        let mise_en_page = card_text_layout(
            self.typographie,
            self.math,
            text,
            largeur,
            TextMode::Rendered,
        );
        let ligne = f64::from(text_box(largeur).line_height);
        let (haut, bas) = plages.iter().fold((0.0, 0.0), |(haut, bas), p| {
            let premiere = ligne_qui_porte(&mise_en_page, p.start);
            let derniere = ligne_qui_porte(&mise_en_page, p.end.saturating_sub(1).max(p.start));
            (
                haut + premiere as f64 * ligne,
                bas + (derniere + 1) as f64 * ligne,
            )
        });
        let n = plages.len() as f64;
        Some(f64::from(TEXT_ORIGIN.1) + (haut + bas) / (2.0 * n))
    }
}
