//! **TRACE-1 — tracer un rectangle** (fiche 65) : sous l'outil Membrane ou Dossier, et sous la
//! grille armée du Storyboard, l'appui ne pose rien encore.
//!
//! Sa demande : *« quand on ajoute un élément, on voit sa prévisualisation suivre la souris
//! jusqu'à ce qu'on pose, ou qu'on crée un rectangle en maintenant le clic — les membranes, ça
//! ne fonctionne pas, il faut juste cliquer »*. C'est la loi de tldraw
//! (`BaseBoxShapeTool`, son état `Pointing`) et de Sketch : **un clic pose à la taille de
//! naissance, un glisser trace la taille**. Ce qui les sépare n'est pas un délai mais la main :
//! tant qu'elle reste dans le tremblement d'un double-clic, c'est un clic.
//!
//! **Une seule fonction dit ce que le relâchement poserait** : le fantôme la lit pendant le
//! tracé, le relâchement la lit au relâchement — il se pose là où on l'a vu.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use crate::interactions::placement::Fantome;
use crate::ui::ActiveTool;
use glucose_core::smart_align::{AlignRect, SnapGuides};
use glucose_core::store::storyboard::Disposition;

/// Un rectangle que la main trace : d'où elle est partie, dans le monde et à l'écran, et où
/// est le coin qu'elle tire — `None` tant qu'elle n'a pas quitté le tremblement, c'est-à-dire
/// tant que le geste est un clic. `decalage` va de la main au coin : nul quand elle trace,
/// l'écart à la poignée quand elle tire celle du storyboard (STORYBOARD-2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trace {
    pub depart: (f64, f64),
    pub depart_ecran: (f64, f64),
    pub point: Option<(f64, f64)>,
    pub decalage: (f64, f64),
}

impl Trace {
    /// Le rectangle tracé dans le monde, s'il y en a un : `(x, y, largeur, hauteur)`.
    pub fn rectangle(&self) -> Option<(f64, f64, f64, f64)> {
        let (p, d) = (self.point?, self.depart);
        Some((
            p.0.min(d.0),
            p.1.min(d.1),
            (p.0 - d.0).abs(),
            (p.1 - d.1).abs(),
        ))
    }
}

impl GlucoseApp {
    /// **L'appui commence un tracé**, sous l'outil Membrane ou Dossier, ou sous la grille armée
    /// du storyboard : le clic est pris, rien n'est encore posé.
    pub(crate) fn commencer_un_trace(&mut self) -> bool {
        let trace = matches!(
            self.ui.active_tool,
            ActiveTool::Membrane | ActiveTool::Folder
        ) || self.ui.placement.storyboard.is_some();
        if !trace {
            return false;
        }
        let depart = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport());
        self.ui.placement.trace = Some(Trace {
            depart,
            depart_ecran: self.mouse_pos,
            point: None,
            decalage: (0.0, 0.0),
        });
        true
    }

    /// **La main bouge pendant un tracé** : hors du tremblement, le rectangle la suit, et le
    /// fantôme avec lui. Rend vrai si un tracé est en cours.
    pub(crate) fn suivre_le_trace(&mut self) -> bool {
        let (souris, tremblement) = (self.mouse_pos, self.tremblement());
        let vp = self.store.viewport();
        let Some(trace) = self.ui.placement.trace.as_mut() else {
            return false;
        };
        let ecart = (souris.0 - trace.depart_ecran.0).hypot(souris.1 - trace.depart_ecran.1);
        if trace.point.is_none() && ecart <= tremblement {
            return true;
        }
        let (x, y) = screen_to_world(souris.0, souris.1, &vp);
        trace.point = Some((x + trace.decalage.0, y + trace.decalage.1));
        let trace = *trace;
        // Sous une membrane ou un dossier, le fantôme est le rectangle lui-même.
        if let Some((left, top, width, height)) = trace.rectangle() {
            if self.ui.placement.storyboard.is_none() {
                self.ui.placement.fantome = Some(Fantome {
                    rect: AlignRect {
                        left,
                        top,
                        width,
                        height,
                    },
                    guides: SnapGuides::default(),
                });
            }
        }
        self.mark_dirty();
        true
    }

    /// **Le relâchement pose** : un clic comme un clic l'a toujours fait — le storyboard centré
    /// sous lui, la membrane à sa taille de naissance —, un tracé à la taille tracée.
    pub(crate) fn finir_le_trace(&mut self) {
        let Some(trace) = self.ui.placement.trace.take() else {
            return;
        };
        if let Some(d) = self.disposition_du_storyboard_trace(&trace) {
            self.ui.placement.storyboard = None;
            let board = self.store.project.active_board_id.clone();
            self.store.disposer(&board, d);
        } else if let Some(rectangle) = trace.rectangle() {
            self.poser_le_contenant(rectangle);
        } else {
            self.place_with_tool(trace.depart.0, trace.depart.1);
        }
        self.mark_dirty();
    }

    /// **Ce que le relâchement poserait sous la grille armée** : la disposition du clic, ou
    /// celle du rectangle tracé. `None` sans grille armée.
    pub(crate) fn disposition_du_storyboard_trace(&self, trace: &Trace) -> Option<Disposition> {
        let grille = self.ui.placement.storyboard?;
        let board = &self.store.project.active_board_id;
        Some(match trace.point {
            Some(point) => self
                .store
                .disposition_du_trace(board, grille, (trace.depart, point)),
            None => self.store.disposition_au_clic(board, grille, trace.depart),
        })
    }

    /// **Le rectangle que la main trace sous la grille armée, à l'écran** — dessiné comme le
    /// rectangle de sélection, pour voir ce que les cases remplissent.
    pub(crate) fn rectangle_trace_a_l_ecran(&self) -> Option<(f64, f64, f64, f64)> {
        let trace = self.ui.placement.trace.as_ref()?;
        self.ui.placement.storyboard?;
        let vp = self.store.viewport();
        let (x1, y1) = crate::canvas::world_to_screen(trace.depart.0, trace.depart.1, &vp);
        let point = trace.point?;
        let (x2, y2) = crate::canvas::world_to_screen(point.0, point.1, &vp);
        Some((x1, y1, x2, y2))
    }

    /// **Renonce au tracé** — Échap. Rend vrai s'il y en avait un ; l'outil reste armé.
    pub(crate) fn renoncer_au_trace(&mut self) -> bool {
        if self.ui.placement.trace.take().is_none() {
            return false;
        }
        self.ui.placement.fantome = None;
        self.mark_dirty();
        true
    }
}

#[cfg(test)]
mod tests;
