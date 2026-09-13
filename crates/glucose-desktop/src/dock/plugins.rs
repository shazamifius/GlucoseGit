//! Le panneau PLUGINS & IA locale — une **façade honnête** (fiche 09 § 10).
//!
//! Rien n'existe derrière : ni détection d'Ollama, ni sonde de la machine, ni moteur, ni
//! téléchargement. Le panneau montre la forme de la référence — IA locale, moteur, réglages
//! de densité et de disposition — et dit, à chaque endroit, ce qui n'est pas là : « non
//! détecté », « à venir », « pas encore disponible ». Il affichait « Ollama actif » avec une
//! pastille verte, « 32 Go RAM · 12 cœurs · GPU 6 Go » écrits en dur, et un moteur « intégré
//! v1.0 » cerclé de vert qui n'a jamais existé.

use super::WidgetRect;
use crate::params::{Pointer, ScaledRect};

pub mod paint;

/// L'état affiché d'Ollama tant qu'aucune détection n'existe. Le jour où `localhost:11434`
/// est interrogé, cette constante disparaît au profit du résultat.
pub const OLLAMA_STATUS: &str = "Ollama : non détecté";
/// Ce que dit la sonde de la machine tant qu'elle n'existe pas.
pub const MACHINE_STATUS: &str = "Détection de la machine : à venir";
/// Ce que dit le conseil de modèle tant qu'aucune sonde ne peut le fonder.
pub const MODEL_ADVICE: &str = "Modèle conseillé : avec la détection";
/// Le bouton de téléchargement, qui ne télécharge pas encore et le dit au clic.
pub const DOWNLOAD_LABEL: &str = "Télécharger un modèle";
/// Le moteur de la référence, que cette version n'embarque pas.
pub const ENGINE_NAME: &str = "Cours magistral";
pub const ENGINE_DESCRIPTION: &str = "Transforme un texte en carte de concepts avec l'IA locale.";
pub const ENGINE_STATUS: &str = "Pas encore disponible dans cette version.";

/// Les trois densités de la référence.
pub const DENSITIES: [&str; 3] = [
    "Concis — les idées maîtresses",
    "Normal — équilibré",
    "Détaillé — chaque nuance",
];
/// Les deux dispositions de la référence.
pub const DISPOSITIONS: [&str; 2] = ["Grille — lecture en blocs", "Fil — une section par ligne"];

#[derive(Debug, Clone)]
pub struct PluginsState {
    pub density_idx: usize,
    pub disposition_idx: usize,
}

impl Default for PluginsState {
    fn default() -> Self {
        Self {
            density_idx: 1,
            disposition_idx: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PluginsPanelLayout {
    pub download_button: WidgetRect,
    pub card_rect: WidgetRect,
    pub density_options: Vec<WidgetRect>,
    pub disposition_options: Vec<WidgetRect>,
}

/// La géométrie du panneau — la seule, lue par le dessin et par le clic.
pub fn layout_plugins_panel(frame: ScaledRect) -> PluginsPanelLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (px, py, pw) = (frame.x, frame.y, frame.w);
    let pad_x = 14.0 * s;
    let inner_w = pw - 28.0 * s;
    let download_button = WidgetRect::new(px + pad_x, py + 104.0 * s, inner_w, 24.0 * s);
    let card_rect = WidgetRect::new(px + pad_x, py + 152.0 * s, inner_w, 54.0 * s);

    let row =
        |y: f32, i: usize| WidgetRect::new(px + pad_x, y + i as f32 * 16.0 * s, inner_w, 16.0 * s);
    let dens_y = py + 244.0 * s;
    let density_options = (0..DENSITIES.len()).map(|i| row(dens_y, i)).collect();
    let disp_y = dens_y + DENSITIES.len() as f32 * 16.0 * s + 20.0 * s;
    let disposition_options = (0..DISPOSITIONS.len()).map(|i| row(disp_y, i)).collect();

    PluginsPanelLayout {
        download_button,
        card_rect,
        density_options,
        disposition_options,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginsClick {
    Handled,
    /// Le bouton de téléchargement — sans moteur derrière.
    DownloadRequested,
    DensitySelected(usize),
    DispositionSelected(usize),
}

pub fn click_plugins_panel(
    state: &mut PluginsState,
    layout: &PluginsPanelLayout,
    pointer: Pointer,
) -> PluginsClick {
    let hit = |r: &WidgetRect| r.contains(pointer.x, pointer.y);
    if hit(&layout.download_button) {
        return PluginsClick::DownloadRequested;
    }
    if let Some(i) = layout.density_options.iter().position(hit) {
        state.density_idx = i;
        return PluginsClick::DensitySelected(i);
    }
    if let Some(i) = layout.disposition_options.iter().position(hit) {
        state.disposition_idx = i;
        return PluginsClick::DispositionSelected(i);
    }
    PluginsClick::Handled
}
