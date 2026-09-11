//! Le redimensionnement interactif : une poignée tirée, un nœud qui suit (RESIZE-1).
//!
//! C'est le branchement de trois briques du noyau qui existaient sans appelant :
//! `hit_priority` (la poignée sous le curseur), `resize` (l'ancrage, le minimum, le rapport)
//! et `smart_align::snap_resize` (le magnétisme). Le motif est celui du déplacement
//! (`interactions/drag.rs`) : `begin_live_edit` à l'appui, des écritures dans le document
//! à chaque mouvement, `end_live_edit` au relâchement — **une** entrée d'undo par geste
//! (§ 3.6). `Échap` pendant le geste rend la taille de départ et n'en laisse aucune.
//!
//! Deux nœuds ont une règle à eux, documentée dans le renderer : la hauteur d'une carte de
//! texte suit son texte (TEXT-FIT-1, `renderer/card.rs`) — le geste ne tire que sa
//! largeur et écrit la hauteur reflué ; une image conserve son rapport sur les coins,
//! `Shift` le libère (RESIZE-3, `glucose_core::resize`).

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use crate::renderer::card::text_card_fit_height;
use crate::renderer::halo::DEFAULT_TEXT_CARD_WIDTH;
use glucose_core::hit_priority::{handle_cursor, PickCandidate, PickKind, PickOwner};
use glucose_core::resize::{resize_rect, snap_resized_rect, Handle, ResizeRule};
use glucose_core::smart_align::{
    collect_align_targets, rect_of_annotation, rect_of_folder, rect_of_image, AlignRect, AlignTarget, SnapGuides,
    SnapOptions,
};
use glucose_core::types::Annotation;
use std::collections::HashSet;
use winit::window::CursorIcon;

#[cfg(test)]
mod proof;
#[cfg(test)]
mod tests;

/// Le nœud que le geste redimensionne.
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeTarget {
    Image { id: String, rotation: f64 },
    TextCard { id: String },
    Annotation { id: String, rule: ResizeRule },
    Folder { id: String },
}

impl ResizeTarget {
    pub fn id(&self) -> &str {
        match self {
            Self::Image { id, .. } | Self::TextCard { id } | Self::Annotation { id, .. } | Self::Folder { id } => id,
        }
    }
}

/// Un geste en cours : ce qu'il faut retenir de l'appui pour interpréter chaque mouvement.
///
/// Ce n'est pas une copie du document mais l'état **de départ** du geste, sans lequel un
/// déplacement du pointeur ne se traduit pas en rectangle (même rôle que
/// `drag_selection_base` pour le déplacement).
#[derive(Debug, Clone)]
pub struct ResizeSession {
    pub target: ResizeTarget,
    pub handle: Handle,
    /// Boîte de départ, monde, origine haut-gauche. Pour une image tournée : sa boîte non
    /// tournée autour du même centre.
    pub start: AlignRect,
    pub pointer_start: (f64, f64),
    pub snap_targets: Vec<AlignTarget>,
    /// Le pointeur a-t-il bougé ? Un simple clic sur une poignée n'est pas un geste et ne
    /// laisse pas d'entrée d'undo.
    pub moved: bool,
}

/// Le curseur winit d'une poignée, via le nom CSS que le noyau connaît.
pub fn cursor_for(handle: Handle) -> CursorIcon {
    handle_cursor(handle.as_str()).parse().unwrap_or(CursorIcon::Default)
}

impl GlucoseApp {
    /// S'il y a une poignée sous `(wx, wy)`, ouvre le geste et rend `true`.
    pub fn begin_resize_at(&mut self, wx: f64, wy: f64) -> bool {
        let Some(candidate) = self.pick_candidate_at(wx, wy).filter(|c| c.kind == PickKind::Handle) else {
            return false;
        };
        let Some(handle) = candidate.corner.as_deref().and_then(Handle::parse) else {
            return false;
        };
        let Some((target, start)) = self.resize_target_of(&candidate) else {
            return false;
        };
        if self.editing_session.as_ref().is_some_and(|s| s.ann_id != target.id()) {
            self.commit_editing();
        }

        let mut exclude = HashSet::new();
        exclude.insert(target.id().to_string());
        let snap_targets = self.store.active_board().map(|b| collect_align_targets(b, &exclude)).unwrap_or_default();

        self.store.begin_live_edit();
        self.resize_session = Some(ResizeSession {
            target,
            handle,
            start,
            pointer_start: (wx, wy),
            snap_targets,
            moved: false,
        });
        true
    }

    /// Le nœud désigné par une poignée, et sa boîte de départ.
    fn resize_target_of(&self, candidate: &PickCandidate) -> Option<(ResizeTarget, AlignRect)> {
        let board = self.store.active_board()?;
        let id = candidate.id.clone();
        match candidate.owner {
            PickOwner::Image => {
                let img = board.images.iter().find(|i| i.id == id)?;
                Some((ResizeTarget::Image { id, rotation: img.rotation }, rect_of_image(img)))
            }
            PickOwner::Annotation | PickOwner::Membrane => {
                let ann = board.annotations.iter().find(|a| a.id() == id)?;
                let rect = rect_of_annotation(ann)?;
                let target = match ann {
                    Annotation::Text { .. } => ResizeTarget::TextCard { id },
                    Annotation::Sticky { .. } => ResizeTarget::Annotation { id, rule: ResizeRule::sticky() },
                    Annotation::Membrane { .. } => ResizeTarget::Annotation { id, rule: ResizeRule::membrane() },
                    Annotation::Arrow { .. } => return None,
                };
                Some((target, rect))
            }
            PickOwner::Folder => {
                let folder = board.folders.iter().find(|f| f.id == id)?;
                Some((ResizeTarget::Folder { id }, rect_of_folder(folder)))
            }
            PickOwner::Arrow => None,
        }
    }

    /// Le pointeur a bougé pendant le geste : recalcule la boîte et l'écrit dans le document.
    pub fn handle_resize_move(&mut self, screen_x: f64, screen_y: f64) {
        let Some(session) = self.resize_session.as_mut() else {
            return;
        };
        let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
        let (wx, wy) = screen_to_world(screen_x, screen_y, &vp);
        let delta = (wx - session.pointer_start.0, wy - session.pointer_start.1);
        if delta.0 == 0.0 && delta.1 == 0.0 {
            return;
        }
        session.moved = true;
        let session = session.clone();
        let rule = self.rule_of(&session.target);
        let (rect, guides) = self.resized_box(&session, rule, delta, vp.scale);
        self.write_resized_box(&session.target, rect);
        self.active_guides = guides;
        self.mark_dirty();
    }

    /// La règle du geste à cet instant : `Shift` libère le rapport d'une image.
    fn rule_of(&self, target: &ResizeTarget) -> ResizeRule {
        match target {
            ResizeTarget::Image { .. } => ResizeRule::image(self.modifiers.shift_key()),
            ResizeTarget::TextCard { .. } => ResizeRule::text_card(),
            ResizeTarget::Annotation { rule, .. } => *rule,
            ResizeTarget::Folder { .. } => ResizeRule::folder(),
        }
    }

    /// La boîte que le pointeur demande, ancrée, bornée, aimantée, puis adaptée au nœud.
    fn resized_box(&self, session: &ResizeSession, rule: ResizeRule, delta: (f64, f64), scale: f64) -> (AlignRect, SnapGuides) {
        let rotation = match session.target {
            ResizeTarget::Image { rotation, .. } => rotation,
            _ => 0.0,
        };
        // Une image tournée se redimensionne dans son propre repère : le pointeur y est ramené.
        let delta = rotate(delta, -rotation);
        // Une carte de texte ne se tire qu'en largeur (TEXT-FIT-1).
        let delta = if matches!(session.target, ResizeTarget::TextCard { .. }) { (delta.0, 0.0) } else { delta };

        let free = resize_rect(session.start, session.handle, delta, rule);
        let (mut rect, guides) = if self.ui.smart_align && rotation == 0.0 {
            let opts = SnapOptions { scale, ..Default::default() };
            let snapped = snap_resized_rect(session.start, session.handle, free, &session.snap_targets, opts, rule);
            (snapped.rect, snapped.guides)
        } else {
            (free, SnapGuides::default())
        };

        if let ResizeTarget::TextCard { id } = &session.target {
            rect.top = session.start.top;
            rect.height = self.fitted_height_of(id, rect.width);
        }
        if rotation != 0.0 {
            rect = recenter_rotated(session.start, rect, rotation);
        }
        (rect, guides)
    }

    fn write_resized_box(&mut self, target: &ResizeTarget, rect: AlignRect) {
        let board = self.store.project.active_board_id.clone();
        match target {
            ResizeTarget::Image { id, .. } => self.store.set_image_rect(&board, id, rect),
            ResizeTarget::TextCard { id } | ResizeTarget::Annotation { id, .. } => {
                self.store.set_annotation_rect(&board, id, rect)
            }
            ResizeTarget::Folder { id } => self.store.set_folder_rect(&board, id, rect),
        };
    }

    /// TEXT-FIT-1 — la hauteur qu'une carte doit avoir à `width` pour son texte (celui en
    /// cours de saisie si elle est en édition).
    fn fitted_height_of(&self, id: &str, width: f64) -> f64 {
        let text = match self.editing_session.as_ref().filter(|s| s.ann_id == id) {
            Some(session) => session.buffer.clone(),
            None => self
                .store
                .active_board()
                .and_then(|b| b.annotations.iter().find(|a| a.id() == id))
                .and_then(|a| match a {
                    Annotation::Text { text, .. } => Some(text.clone()),
                    _ => None,
                })
                .unwrap_or_default(),
        };
        text_card_fit_height(&self.renderer.typography, &text, width)
    }

    /// Relâchement : le geste se referme sur une seule entrée d'undo, ou sur aucune s'il
    /// n'a pas bougé.
    pub fn finish_resize(&mut self) {
        let Some(session) = self.resize_session.take() else {
            return;
        };
        if session.moved {
            self.store.end_live_edit();
        } else {
            self.store.cancel_live_edit();
        }
        self.active_guides = SnapGuides::default();
    }

    /// `Échap` : le nœud reprend sa taille de départ, la pile d'undo n'en garde rien.
    pub fn cancel_resize(&mut self) -> bool {
        if self.resize_session.take().is_none() {
            return false;
        }
        self.store.cancel_live_edit();
        self.active_guides = SnapGuides::default();
        self.mark_dirty();
        true
    }

    /// La poignée sous le pointeur hors de tout geste — pour annoncer le geste par le curseur.
    pub fn hovered_handle(&self) -> Option<Handle> {
        if self.mouse_pos.1 < self.ui.header_height() as f64 || self.selection_box.is_some() || self.is_dragging_item {
            return None;
        }
        let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
        let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        let candidate = self.pick_candidate_at(wx, wy).filter(|c| c.kind == PickKind::Handle)?;
        candidate.corner.as_deref().and_then(Handle::parse)
    }

    /// TEXT-FIT-1 — écrit dans le document la hauteur qu'une carte de texte doit avoir pour
    /// son texte à sa largeur. Appelé quand le texte change (validation d'une saisie).
    pub fn fit_text_card_height(&mut self, ann_id: &str) {
        let board = self.store.project.active_board_id.clone();
        let Some((rect, text)) = self.store.active_board().and_then(|b| {
            b.annotations.iter().find(|a| a.id() == ann_id).and_then(|a| match a {
                Annotation::Text { text, .. } => rect_of_annotation(a).map(|r| (r, text.clone())),
                _ => None,
            })
        }) else {
            return;
        };
        let height = text_card_fit_height(&self.renderer.typography, &text, rect.width);
        if (height - rect.height).abs() > 1e-6 {
            self.store.set_annotation_rect(&board, ann_id, AlignRect { height, ..rect });
        }
    }
}

impl GlucoseApp {
    /// TEXT-FIT-1 à l'ouverture d'un document : chaque carte de texte reçoit la hauteur de
    /// son texte. C'est une normalisation de chargement, pas une action de l'utilisateur —
    /// elle n'entre pas dans la pile d'undo, que `load_project` vient de vider.
    pub fn fit_all_text_cards(&mut self) {
        let typography = &self.renderer.typography;
        for board in &mut self.store.project.boards {
            for ann in &mut board.annotations {
                let Annotation::Text { width, height, text, .. } = ann else {
                    continue;
                };
                let w = width.unwrap_or(DEFAULT_TEXT_CARD_WIDTH);
                *width = Some(w);
                *height = Some(text_card_fit_height(typography, text, w));
            }
        }
    }
}

fn rotate((x, y): (f64, f64), angle: f64) -> (f64, f64) {
    if angle == 0.0 {
        return (x, y);
    }
    let (c, s) = (angle.cos(), angle.sin());
    (x * c - y * s, x * s + y * c)
}

/// Une image tournée : la boîte calculée dans le repère de l'image est reposée dans le
/// monde en faisant tourner le déplacement de son centre.
fn recenter_rotated(start: AlignRect, local: AlignRect, rotation: f64) -> AlignRect {
    let (cx0, cy0) = (start.left + start.width / 2.0, start.top + start.height / 2.0);
    let (cx, cy) = (local.left + local.width / 2.0, local.top + local.height / 2.0);
    let (dx, dy) = rotate((cx - cx0, cy - cy0), rotation);
    AlignRect::new(cx0 + dx - local.width / 2.0, cy0 + dy - local.height / 2.0, local.width, local.height)
}
