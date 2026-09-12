//! Rendre une frame sans fenêtre, et la mesurer — le travail A.5 du plan de marche.
//!
//! # Pourquoi ceci peut exister
//!
//! `Renderer::render` écrit dans un `PixmapMut`, c'est-à-dire dans de la mémoire. Il n'a jamais
//! eu besoin d'un écran : c'est la boucle d'événements qui en avait un. En séparant le binaire
//! de la bibliothèque, le rendu devient donc **mesurable et capturable en test**, exactement
//! comme une fonction pure — à ceci près qu'il produit deux millions de pixels.
//!
//! # Les deux usages, et pourquoi ils sont dans le même module
//!
//! * **Mesurer** ([`measure`]) : combien de millisecondes coûte une frame, sur un document
//!   donné, à une définition donnée. Le budget est de 10 ms — cent images par seconde.
//! * **Capturer** ([`capture`]) : la même frame, écrite en PNG, pour être **regardée** et
//!   comparée. C'est le filet que les tests unitaires ne tendent pas : aucun d'eux ne voit une
//!   carte posée un pixel trop bas.
//!
//! Les deux partagent le même chemin de rendu, et c'est la seule raison qui compte : une
//! mesure prise sur un autre chemin que celui qu'on capture ne mesure pas ce qu'on regarde.

use crate::params::{Pointer, SceneOverlay};
use crate::renderer::Renderer;
use crate::ui::UiState;
use glucose_core::smart_align::SnapGuides;
use glucose_core::store::Store;
use glucose_core::types::Viewport;
use std::time::Instant;
use tiny_skia::Pixmap;

/// Définitions de référence du banc. La 4K est dans la liste parce que la charte l'exige
/// explicitement — « aucun problème sur n'importe quel écran du monde ».
pub const DEFINITIONS: &[(&str, u32, u32)] =
    &[("1080p", 1920, 1080), ("1440p", 2560, 1440), ("4K", 3840, 2160)];

/// Le budget d'une frame, en millisecondes : cent images par seconde.
pub const BUDGET_MS: f64 = 10.0;

/// Pose le cadrage d'un document : origine au centre du contenu, à l'échelle voulue.
///
/// Un banc qui laisserait le viewport par défaut mesurerait surtout le culling — presque rien
/// ne serait visible. Ici, la caméra regarde le milieu du document, ce qui met à l'écran la
/// part que le zoom demandé permet d'y mettre.
pub fn frame_document(store: &mut Store, scale: f64, width: u32, height: u32) {
    let board_id = store.project.active_board_id.clone();
    let Some(boite) = store.content_bounds(&board_id) else { return };
    let (cx, cy) = (boite.left + boite.width / 2.0, boite.top + boite.height / 2.0);

    store.set_viewport(
        &board_id,
        Viewport {
            x: width as f64 / 2.0 - cx * scale,
            y: height as f64 / 2.0 - cy * scale,
            scale,
        },
    );
}

/// Rend une frame complète du document dans un pixmap neuf.
///
/// Le rendu est celui de l'application, interface comprise : mesurer la scène seule flatterait
/// le chiffre d'une part que l'utilisateur paie quand même.
pub fn render_frame(renderer: &mut Renderer, ui: &mut UiState, store: &Store, width: u32, height: u32) -> Pixmap {
    let mut pixmap = Pixmap::new(width, height).expect("un pixmap de cette taille");
    render_into(renderer, ui, store, &mut pixmap);
    pixmap
}

/// Rend dans un pixmap existant — ce que [`measure`] appelle en boucle, pour ne pas mesurer
/// l'allocation de huit mégaoctets à chaque tour.
pub fn render_into(renderer: &mut Renderer, ui: &mut UiState, store: &Store, pixmap: &mut Pixmap) {
    let guides = SnapGuides::default();
    let overlay = SceneOverlay { guides: &guides, selection_box: None, editing: None };
    // Le pointeur est posé hors de la fenêtre : aucun survol, donc aucun état de l'interface
    // qui dépendrait de la position de la souris. Une capture doit être la même partout.
    let pointer = Pointer { x: -1.0, y: -1.0 };
    renderer.render(&mut pixmap.as_mut(), store, ui, overlay, pointer);
}

/// Ce qu'une série de frames a coûté.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub frames: usize,
    pub min_ms: f64,
    pub median_ms: f64,
    pub max_ms: f64,
}

impl Stats {
    /// La médiane tient-elle le budget de cent images par seconde ?
    pub fn within_budget(&self) -> bool {
        self.median_ms <= BUDGET_MS
    }

    /// Images par seconde, d'après la médiane.
    pub fn fps(&self) -> f64 {
        if self.median_ms <= 0.0 {
            f64::INFINITY
        } else {
            1000.0 / self.median_ms
        }
    }
}

/// Mesure `repeats` frames du même document et rend leurs statistiques.
///
/// C'est la **médiane** qui juge, pas la moyenne : une frame isolée peut être arbitrairement
/// lente sans que cela dise quoi que ce soit du rendu — un ordonnanceur qui préempte, un cache
/// processeur qu'un autre programme vient de vider. La moyenne en garde la trace, la médiane
/// non. Le maximum est rendu quand même : c'est lui qui dit s'il y a des à-coups.
///
/// La première frame n'est pas comptée. Elle remplit les caches — glyphes, teintes, index
/// spatial — et mesure donc un démarrage, pas un régime.
pub fn measure(store: &Store, width: u32, height: u32, repeats: usize) -> Stats {
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let mut pixmap = Pixmap::new(width, height).expect("un pixmap de cette taille");

    render_into(&mut renderer, &mut ui, store, &mut pixmap);

    let mut temps = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let t = Instant::now();
        render_into(&mut renderer, &mut ui, store, &mut pixmap);
        temps.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    temps.sort_by(f64::total_cmp);

    Stats {
        frames: temps.len(),
        min_ms: temps.first().copied().unwrap_or(0.0),
        median_ms: temps.get(temps.len() / 2).copied().unwrap_or(0.0),
        max_ms: temps.last().copied().unwrap_or(0.0),
    }
}

/// Rend un document et écrit le PNG.
pub fn capture(store: &Store, width: u32, height: u32) -> Vec<u8> {
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let pixmap = render_frame(&mut renderer, &mut ui, store, width, height);
    pixmap.encode_png().expect("encoder un PNG")
}

#[cfg(test)]
mod tests;
