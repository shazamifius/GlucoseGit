//! **PLACEMENT-1 — l'aimant dès le premier placement** (registre de Tauri, entrée 3).
//!
//! Sa demande : *« on peut activer le snap intelligent […] mais il ne s'active qu'une fois
//! qu'on édite le placement. Pourquoi pas DÈS qu'on souhaite réellement placer une première
//! fois ? »* Un clic avec un outil de création posait l'élément sous le curseur, sans aimant :
//! il fallait le poser de travers, puis le glisser pour qu'il s'aligne.
//!
//! Sous un outil qui crée une boîte — carte, pense-bête, membrane, dossier —, un **fantôme** de
//! l'élément à naître suit désormais le curseur, à sa taille de naissance, **aimanté** comme un
//! glisser l'est, guides compris ; le clic le pose là où le fantôme est. C'est ce que fait
//! FigJam pour ses notes : l'outil choisi, un aperçu suit le curseur, un clic le dépose.
//!
//! **Une seule fonction dit où l'élément se poserait** ([`GlucoseApp::placement_aimante`]) : le
//! fantôme la lit au mouvement, le clic la lit au clic. Aucun état ne passe de l'un à l'autre,
//! donc le clic ne peut pas poser ailleurs que là où le fantôme était.

use crate::app::GlucoseApp;
use crate::interactions::tools::{text_card, NEW_CONTAINER_SIZE, NEW_TEXT};
use crate::ui::ActiveTool;
use glucose_core::smart_align::{
    collect_align_targets, snap_move, AlignRect, AlignTarget, SnapGuides, SnapOptions,
};
use glucose_core::types::{DEFAULT_STICKY_HEIGHT, DEFAULT_STICKY_WIDTH};

/// **L'élément à naître, là où il se poserait** : son rectangle dans le monde, et les guides
/// qui disent à quoi il s'aligne.
#[derive(Debug, Clone, PartialEq)]
pub struct Fantome {
    pub rect: AlignRect,
    pub guides: SnapGuides,
}

/// Ce que le placement garde d'une image à l'autre : le fantôme montré, et les cibles de
/// l'aimant, relevées une fois par état du document.
#[derive(Debug, Clone, Default)]
pub struct Placement {
    pub fantome: Option<Fantome>,
    cibles: Option<Cibles>,
}

/// Les cibles de l'aimant, et l'état du document qu'elles décrivent.
#[derive(Debug, Clone)]
struct Cibles {
    version: u64,
    tableau: String,
    liste: Vec<AlignTarget>,
}

impl GlucoseApp {
    /// **La taille de naissance** de ce que l'outil armé crée, en unités monde — `None` s'il ne
    /// crée pas de boîte (sélection, main, flèche).
    ///
    /// La même que celle des fabriques de [`crate::interactions::tools`] : une carte naît à sa
    /// largeur de naissance et à la hauteur de son texte, et c'est cette carte-là qu'on mesure.
    pub(crate) fn taille_de_naissance(&self) -> Option<(f64, f64)> {
        match self.ui.active_tool {
            ActiveTool::Text => text_card(
                &self.renderer.typography,
                &self.renderer.math,
                "fantome",
                0.0,
                0.0,
                NEW_TEXT,
            )
            .size(),
            ActiveTool::Sticky => Some((DEFAULT_STICKY_WIDTH, DEFAULT_STICKY_HEIGHT)),
            ActiveTool::Membrane | ActiveTool::Folder => Some(NEW_CONTAINER_SIZE),
            ActiveTool::Select | ActiveTool::Pan | ActiveTool::Arrow => None,
        }
    }

    /// **Là où l'élément que l'outil crée se poserait** si l'on cliquait au point `(wx, wy)` du
    /// monde : son coin sous le curseur, puis aimanté si l'Aimant est allumé.
    pub(crate) fn placement_aimante(&mut self, (wx, wy): (f64, f64)) -> Option<Fantome> {
        let (width, height) = self.taille_de_naissance()?;
        let propose = AlignRect {
            left: wx,
            top: wy,
            width,
            height,
        };
        if !self.ui.smart_align {
            return Some(Fantome {
                rect: propose,
                guides: SnapGuides::default(),
            });
        }
        // Le seuil de l'aimant est en pixels logiques, comme celui du glisser (DPI-1).
        let scale = self.store.viewport().scale / self.densite();
        let snap = snap_move(
            propose,
            self.cibles_de_placement(),
            SnapOptions {
                scale,
                ..Default::default()
            },
        );
        Some(Fantome {
            rect: AlignRect {
                left: propose.left + snap.dx,
                top: propose.top + snap.dy,
                ..propose
            },
            guides: snap.guides,
        })
    }

    /// Les cibles de l'aimant pour le tableau actif, relevées une fois par état du document.
    ///
    /// Les relever à chaque mouvement de la souris copierait le tableau entier autant de fois ;
    /// elles ne changent que quand le document change, et sa version le dit.
    fn cibles_de_placement(&mut self) -> &[AlignTarget] {
        let (version, tableau) = (self.store.version, &self.store.project.active_board_id);
        let a_jour = self
            .ui
            .placement
            .cibles
            .as_ref()
            .is_some_and(|c| c.version == version && &c.tableau == tableau);
        if !a_jour {
            let liste = self
                .store
                .active_board()
                .map(|b| collect_align_targets(b, &Default::default()))
                .unwrap_or_default();
            self.ui.placement.cibles = Some(Cibles {
                version,
                tableau: tableau.clone(),
                liste,
            });
        }
        self.ui
            .placement
            .cibles
            .as_ref()
            .map_or(&[], |c| c.liste.as_slice())
    }

    /// Le fantôme à montrer : seulement sous un outil qui crée une boîte. Un outil changé au
    /// clavier, sans que la souris bouge, ne laisse donc pas traîner celui d'avant.
    pub(crate) fn fantome_montre(&self) -> Option<Fantome> {
        let cree = !matches!(
            self.ui.active_tool,
            ActiveTool::Select | ActiveTool::Pan | ActiveTool::Arrow
        );
        self.ui.placement.fantome.clone().filter(|_| cree)
    }

    /// **Le fantôme suit le curseur** sous un outil de création armé, et disparaît sous les
    /// autres. L'image ne se redessine que là où il était et là où il est — sauf quand des
    /// guides traversent l'écran, comme pour un glisser.
    pub(crate) fn suivre_le_fantome(&mut self) {
        let vp = self.store.viewport();
        let point = crate::canvas::screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        let fantome = self.placement_aimante(point);
        if fantome == self.ui.placement.fantome {
            return;
        }
        let guides = |f: &Option<Fantome>| {
            f.as_ref()
                .is_some_and(|f| f.guides.x.is_some() || f.guides.y.is_some())
        };
        if guides(&fantome) || guides(&self.ui.placement.fantome) {
            self.mark_dirty();
        } else {
            for f in [&fantome, &self.ui.placement.fantome].into_iter().flatten() {
                self.salir(f.rect);
            }
        }
        self.ui.placement.fantome = fantome;
    }
}

#[cfg(test)]
mod tests;
