//! **L'aiguillage du clic** : le clic va au panneau qui le contient, et ce panneau dit ce
//! qu'il demande à l'application.

use super::{
    domains, organize, panneau_sous, plugins, pomodoro, preset, storyboard, temps, DockManager,
    OrganizeState, TabId,
};
use crate::params::{Pointer, ScaledRect, ScreenFrame};
use crate::typography::Typography;
use glucose_core::store::Store;

/// Ce qu'un clic dans un panneau demande à l'application. Les gestes qu'un panneau règle
/// lui-même — choisir un tri, cocher une densité, lancer le minuteur — rendent [`Handled`].
///
/// [`Handled`]: PanelClickResult::Handled
#[derive(Debug, Clone, PartialEq)]
pub enum PanelClickResult {
    Handled,
    /// `Appliquer` du panneau ORDONNER : réarranger les images du tableau.
    ApplyLayout(OrganizeState),
    /// Le storyboard n'a pas d'effet sur le canevas (fiche 09 § 8).
    StoryboardNotReady,
    /// Le bouton « Télécharger » du panneau PLUGINS, sans moteur derrière (fiche 09 § 10.3).
    DownloadModel,
    /// Un geste du panneau DOMAINES, à traduire en commande par `interactions::domains`.
    Domain(domains::DomainIntent),
    /// Un geste de la Time Machine, à traduire par `interactions::temps`.
    Temps(temps::TempsIntent),
    /// Armer un gabarit, ou retirer celui du tableau (PRESET-1).
    Gabarit(preset::GabaritIntent),
}

pub fn handle_dock_click(
    dock: &mut DockManager,
    store: &Store,
    typo: &Typography,
    screen: ScreenFrame,
    pointer: Pointer,
) -> Option<PanelClickResult> {
    let (tab, frame) = panneau_sous(dock, screen, pointer)?;
    Some(click_panel(dock, store, typo, tab, frame, pointer))
}

fn click_panel(
    dock: &mut DockManager,
    store: &Store,
    typo: &Typography,
    tab: TabId,
    frame: ScaledRect,
    pointer: Pointer,
) -> PanelClickResult {
    match tab {
        TabId::Organize => {
            let layout = organize::layout_organize_panel(frame, typo);
            match organize::click_organize_panel(&mut dock.organize, &layout, pointer) {
                organize::OrganizeClick::Apply(state) => PanelClickResult::ApplyLayout(state),
                organize::OrganizeClick::Handled => PanelClickResult::Handled,
            }
        }
        TabId::Pomodoro => {
            let layout = pomodoro::layout_pomodoro_panel(frame, typo);
            pomodoro::click_pomodoro_panel(&mut dock.pomodoro, &layout, pointer);
            PanelClickResult::Handled
        }
        TabId::Storyboard => {
            let layout = storyboard::layout_storyboard_panel(frame);
            match storyboard::click_storyboard_panel(&mut dock.storyboard, &layout, pointer) {
                storyboard::StoryboardClick::Handled => PanelClickResult::Handled,
                // Ni le format ni l'activation ne touchent au canevas : le panneau ne doit
                // pas laisser croire le contraire, et son état revient où il était.
                storyboard::StoryboardClick::FormatSelected(_)
                | storyboard::StoryboardClick::ToggleRequested => {
                    dock.storyboard.active = false;
                    PanelClickResult::StoryboardNotReady
                }
            }
        }
        TabId::Plugins => {
            let layout = plugins::layout_plugins_panel(frame);
            match plugins::click_plugins_panel(&mut dock.plugins, &layout, pointer) {
                plugins::PluginsClick::DownloadRequested => PanelClickResult::DownloadModel,
                _ => PanelClickResult::Handled,
            }
        }
        // Un clic arme un gabarit, ou retire celui que le tableau suit ; ailleurs, il est
        // consommé, pour ne pas traverser le panneau jusqu'au canevas.
        TabId::Preset => {
            let contenu = preset::Contenu::du(store);
            let layout = preset::layout_presets_panel(frame, &contenu);
            preset::click_presets_panel(&layout, &contenu, (pointer.x, pointer.y))
                .map_or(PanelClickResult::Handled, PanelClickResult::Gabarit)
        }
        TabId::Domains => {
            let layout = domains::layout_domains_panel(frame, store, &dock.domains);
            let has_selection =
                !store.selected_annotation_ids.is_empty() || !store.selected_image_ids.is_empty();
            match domains::hit_domains_panel(&layout, store, pointer, has_selection) {
                Some(intent) => PanelClickResult::Domain(intent),
                None => PanelClickResult::Handled,
            }
        }
        TabId::Temps => {
            let layout = temps::layout_temps_panel(frame, &dock.temps);
            match temps::click_temps_panel(&dock.temps, &layout, pointer) {
                Some(intent) => PanelClickResult::Temps(intent),
                None => PanelClickResult::Handled,
            }
        }
    }
}
