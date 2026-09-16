//! Le dock : des panneaux posés en haut à gauche ou en bas à gauche de l'écran, qu'on glisse
//! pour les réordonner et qu'on tire vers leur bord pour les fermer.
//!
//! # Un panneau, un module
//!
//! | Module | Panneau | État |
//! |---|---|---|
//! | [`organize`] | ORDONNER | agit : il réarrange les images du tableau |
//! | [`pomodoro`] | POMODORO | agit : le décompte est réel |
//! | [`domains`] | DOMAINES | agit : une vue du catalogue du document (DOM-UI-1) |
//! | [`storyboard`] | STORYBOARD | façade honnête (fiche 09 § 8) |
//! | [`plugins`] | PLUGINS | façade honnête (fiche 09 § 10) |
//! | [`preset`] | PRESETS | les gabarits s'affichent, aucun ne se pose encore |
//!
//! Chaque module porte **son** état, **sa** géométrie et **son** clic ; son sous-module
//! `paint` porte son dessin. Ce fichier n'a plus que ce qui leur est commun : le dock
//! lui-même, ses cadres, et l'aiguillage.
//!
//! Ils vivaient tous dans un seul fichier de 3 000 lignes, avec cinq fonctions de rendu de
//! 230 à 370 lignes qui redessinaient chacune le même bouton à leur façon — la dette que la
//! fiche 11 nomme en premier, et l'endroit exact de R-37 : deux formules pour la largeur d'un
//! même bouton, dont une comptait les octets de « Sombre → Clair ».
//!
//! # Une seule géométrie (loi L4)
//!
//! Pour chaque panneau, `layout_*` produit la liste de ses rectangles ; le dessin la lit, le
//! clic la lit. Aucune coordonnée n'est calculée deux fois.

pub mod cache;
pub mod domains;
pub mod organize;
pub mod paint;
pub mod plugins;
pub mod pomodoro;
pub mod preset;
pub mod render;
pub mod storyboard;

use crate::params::{Pointer, ScaledRect, ScreenFrame};
use crate::typography::Typography;
use glucose_core::store::Store;

pub use cache::DockCache;
pub use organize::{apply_organize_layout, LayoutMode, LayoutResult, OrganizeState, SortType};
pub use plugins::OLLAMA_STATUS;
pub use pomodoro::PomodoroState;
pub use preset::PresetsState;
pub use render::{render_docks, DockPass};
pub use storyboard::StoryboardState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabId {
    Organize,
    Pomodoro,
    Storyboard,
    Plugins,
    Preset,
    Domains,
}

impl TabId {
    pub fn anchor(&self) -> DockAnchor {
        match self {
            Self::Organize | Self::Pomodoro | Self::Storyboard => DockAnchor::BottomLeft,
            Self::Plugins | Self::Preset | Self::Domains => DockAnchor::TopLeft,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Organize => "ORDONNER",
            Self::Pomodoro => "POMODORO",
            Self::Storyboard => "STORYBOARD",
            Self::Plugins => "PLUGINS",
            Self::Preset => "PRESETS",
            Self::Domains => "DOMAINES",
        }
    }

    /// Largeurs de la fiche 10 § 4 : Domaines 320, Presets 280, Plugins 340 en haut ;
    /// Ordonner 250, Storyboard 260, Pomodoro 160 au minimum en bas.
    pub fn default_width(&self) -> f32 {
        match self {
            Self::Organize => 250.0,
            Self::Pomodoro => 170.0,
            Self::Storyboard => 260.0,
            Self::Plugins => 340.0,
            Self::Preset => 280.0,
            Self::Domains => 320.0,
        }
    }

    pub fn default_height(&self) -> f32 {
        match self {
            Self::Organize => 410.0,
            Self::Pomodoro => 210.0,
            Self::Storyboard => 340.0,
            Self::Plugins => 460.0,
            Self::Preset => 460.0,
            Self::Domains => 430.0,
        }
    }
}

/// Glissement d'un panneau vers sa sortie au-delà duquel il se ferme (fiche 10 § 4 : 80 px).
pub const DISMISS_DRAG_PX: f32 = 80.0;

/// Marge entre le bord de l'écran et le premier panneau, et entre deux panneaux.
const DOCK_MARGIN: f32 = 12.0;
const DOCK_GAP: f32 = 10.0;
/// Hauteur de la bande de préhension d'un panneau.
const GRIP_HEIGHT: f32 = 16.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockAnchor {
    TopLeft,
    BottomLeft,
}

#[derive(Debug, Clone)]
pub struct DragSession {
    pub tab: TabId,
    pub start_x: f32,
    pub start_y: f32,
    pub current_x: f32,
    pub current_y: f32,
}

#[derive(Debug, Clone)]
pub struct DockManager {
    pub top_tabs: Vec<TabId>,
    pub bottom_tabs: Vec<TabId>,
    pub drag: Option<DragSession>,
    pub organize: OrganizeState,
    pub pomodoro: PomodoroState,
    pub storyboard: StoryboardState,
    pub plugins: plugins::PluginsState,
    pub presets: PresetsState,
    /// L'état d'**interaction** du panneau DOMAINES : ce qui attend une confirmation, ce qui
    /// est en cours de frappe. Aucune donnée de domaine — celles-ci vivent dans le document
    /// (DOM-UI-1, `dock::domains`).
    pub domains: domains::DomainsUi,
}

/// `new()` n'est pas dérivable : l'état initial ouvre deux onglets bas.
impl Default for DockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DockManager {
    pub fn new() -> Self {
        Self {
            top_tabs: Vec::new(),
            bottom_tabs: vec![TabId::Organize, TabId::Pomodoro],
            drag: None,
            organize: OrganizeState::default(),
            pomodoro: PomodoroState::default(),
            storyboard: StoryboardState::default(),
            plugins: plugins::PluginsState::default(),
            presets: PresetsState::default(),
            domains: domains::DomainsUi::default(),
        }
    }

    fn tabs(&self, anchor: DockAnchor) -> &Vec<TabId> {
        match anchor {
            DockAnchor::TopLeft => &self.top_tabs,
            DockAnchor::BottomLeft => &self.bottom_tabs,
        }
    }

    fn tabs_mut(&mut self, anchor: DockAnchor) -> &mut Vec<TabId> {
        match anchor {
            DockAnchor::TopLeft => &mut self.top_tabs,
            DockAnchor::BottomLeft => &mut self.bottom_tabs,
        }
    }

    pub fn is_open(&self, tab: TabId) -> bool {
        self.tabs(tab.anchor()).contains(&tab)
    }

    pub fn toggle_tab(&mut self, tab: TabId) {
        let list = self.tabs_mut(tab.anchor());
        match list.iter().position(|&t| t == tab) {
            Some(pos) => {
                list.remove(pos);
            }
            None => list.push(tab),
        }
    }

    pub fn dismiss_tab(&mut self, tab: TabId) {
        self.tabs_mut(tab.anchor()).retain(|&t| t != tab);
    }

    /// Suit le glissement d'un panneau et l'échange avec son voisin quand son centre dépasse
    /// celui de l'autre. Rend `true` si un échange a eu lieu.
    pub fn update_drag(&mut self, mouse_x: f32, mouse_y: f32) -> bool {
        let Some(drag) = self.drag.as_mut() else {
            return false;
        };
        let (tab, start_x) = (drag.tab, drag.start_x);
        drag.current_x = mouse_x;
        drag.current_y = mouse_y;

        let list = self.tabs_mut(tab.anchor());
        let Some(index) = list.iter().position(|&t| t == tab) else {
            return false;
        };

        // Les centres nominaux, dans l'ordre courant : c'est contre eux que le panneau tiré
        // se compare.
        let mut centers = Vec::with_capacity(list.len());
        let mut x = DOCK_MARGIN;
        for &t in list.iter() {
            let w = t.default_width();
            centers.push((x + w / 2.0, w));
            x += w + DOCK_GAP;
        }
        let dragged = centers[index].0 + (mouse_x - start_x);

        let neighbour = if index > 0 && dragged < centers[index - 1].0 {
            Some(index - 1)
        } else if index + 1 < centers.len() && dragged > centers[index + 1].0 {
            Some(index + 1)
        } else {
            None
        };
        let Some(neighbour) = neighbour else {
            return false;
        };

        list.swap(index, neighbour);
        // Le panneau a changé de place : son origine suit, sinon il repartirait d'un bond.
        let shift = centers[neighbour].1 + DOCK_GAP;
        if let Some(drag) = self.drag.as_mut() {
            drag.start_x += if neighbour < index { -shift } else { shift };
        }
        true
    }

    /// Un panneau glissé de plus de [`DISMISS_DRAG_PX`] vers sa sortie — le haut pour le
    /// dock du haut, le bas pour celui du bas — se ferme (fiche 10 § 4).
    pub fn finish_drag(&mut self) -> Option<TabId> {
        let drag = self.drag.take()?;
        let dy = drag.current_y - drag.start_y;
        let dismissed = match drag.tab.anchor() {
            DockAnchor::TopLeft => dy < -DISMISS_DRAG_PX,
            DockAnchor::BottomLeft => dy > DISMISS_DRAG_PX,
        };
        dismissed.then(|| {
            self.dismiss_tab(drag.tab);
            drag.tab
        })
    }

    /// Fait avancer le minuteur. Rend `true` si l'affichage doit changer.
    pub fn tick_pomodoro(&mut self) -> bool {
        self.pomodoro.tick()
    }
}

// ── La géométrie des cadres ─────────────────────────────────────────────────

/// Un rectangle d'interface, en pixels d'écran.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WidgetRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl WidgetRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

/// Le cadre d'un panneau ouvert : sa place nominale, sa poignée, et son décalage s'il est tiré.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelLayoutBox {
    pub tab: TabId,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub grip_y: f32,
    pub grip_height: f32,
    pub is_dragged: bool,
    pub visual_offset_x: f32,
    pub visual_offset_y: f32,
}

impl PanelLayoutBox {
    /// Le cadre tel qu'il est **vu** : sa place nominale plus son décalage de glissement.
    fn seen(&self) -> WidgetRect {
        WidgetRect::new(
            self.x + self.visual_offset_x,
            self.y + self.visual_offset_y,
            self.width,
            self.height,
        )
    }

    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        self.seen().contains(px, py)
    }

    pub fn grip_contains_point(&self, px: f32, py: f32) -> bool {
        let seen = self.seen();
        WidgetRect::new(
            seen.x,
            self.grip_y + self.visual_offset_y,
            seen.w,
            self.grip_height,
        )
        .contains(px, py)
    }

    /// L'ombre portée du panneau, décalée vers le bas et débordant de chaque côté.
    ///
    /// Elle est ici, et non dans [`draw_frame`] qui la dessine, parce que [`Self::extent`] en
    /// a besoin aussi : une boîte englobante calculée à côté de l'ombre finirait par ne plus
    /// la contenir le jour où l'une des deux bouge. Les deux dérivent donc de cette seule
    /// définition.
    fn shadow(&self, s: f32) -> WidgetRect {
        let seen = self.seen();
        WidgetRect::new(
            seen.x - 2.0 * s,
            seen.y + 3.0 * s,
            seen.w + 4.0 * s,
            seen.h + 4.0 * s,
        )
    }

    /// Tout ce que le panneau peut noircir : son cadre, son ombre, et son trait de bordure.
    ///
    /// C'est la taille du tampon qu'un panneau mis en cache doit se réserver (DOCK-CACHE-1).
    /// Elle est **exacte et dérivée**, jamais une marge choisie au jugé : un tampon trop
    /// petit rognerait l'ombre, un tampon trop grand paierait des pixels vides à chaque
    /// composition.
    fn extent(&self, s: f32) -> WidgetRect {
        let seen = self.seen();
        let shadow = self.shadow(s);
        // Le contour est tracé *sur* le bord : il déborde d'une demi-épaisseur.
        let demi_trait = 0.5 * s;
        let x0 = (seen.x - demi_trait).min(shadow.x);
        let y0 = (seen.y - demi_trait).min(shadow.y);
        let x1 = (seen.x + seen.w + demi_trait).max(shadow.x + shadow.w);
        let y1 = (seen.y + seen.h + demi_trait).max(shadow.y + shadow.h);
        WidgetRect::new(x0, y0, x1 - x0, y1 - y0)
    }
}

/// La place de chaque panneau ouvert, dans les deux docks.
pub fn compute_panel_layouts(
    dock: &DockManager,
    _screen_w: f32,
    screen_h: f32,
    header_h: f32,
    scale: f32,
) -> Vec<PanelLayoutBox> {
    let s = crate::theme::clamp_ui_scale(scale);
    let mut layouts = Vec::new();
    for anchor in [DockAnchor::TopLeft, DockAnchor::BottomLeft] {
        let mut x = DOCK_MARGIN * s;
        for &tab in dock.tabs(anchor) {
            let width = tab.default_width() * s;
            let (y, height) = place(anchor, tab, screen_h, header_h, s);
            let grip_height = GRIP_HEIGHT * s;
            // La poignée est du côté de la sortie : en bas pour le dock du haut, en haut
            // pour celui du bas (fiche 10 § 4).
            let grip_y = match anchor {
                DockAnchor::TopLeft => y + height - grip_height,
                DockAnchor::BottomLeft => y,
            };
            let dragging = dock.drag.as_ref().filter(|d| d.tab == tab);
            layouts.push(PanelLayoutBox {
                tab,
                x,
                y,
                width,
                height,
                grip_y,
                grip_height,
                is_dragged: dragging.is_some(),
                visual_offset_x: dragging.map_or(0.0, |d| d.current_x - d.start_x),
                visual_offset_y: dragging.map_or(0.0, |d| d.current_y - d.start_y),
            });
            x += width + DOCK_GAP * s;
        }
    }
    layouts
}

/// L'ordonnée et la hauteur d'un panneau, selon son ancrage — jamais plus haut que l'écran.
fn place(anchor: DockAnchor, tab: TabId, screen_h: f32, header_h: f32, s: f32) -> (f32, f32) {
    let margin = DOCK_MARGIN * s;
    match anchor {
        DockAnchor::TopLeft => {
            let y = header_h + 8.0 * s;
            let height = (tab.default_height() * s).min(screen_h - y - 2.0 * margin);
            (y, height)
        }
        DockAnchor::BottomLeft => {
            let height = (tab.default_height() * s).min(screen_h - header_h - 2.0 * margin);
            (screen_h - height - margin, height)
        }
    }
}

// ── Le rendu ────────────────────────────────────────────────────────────────

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
}

/// Le clic va au panneau qui le contient, et à personne d'autre.
pub fn handle_dock_click(
    dock: &mut DockManager,
    store: &Store,
    typo: &Typography,
    screen: ScreenFrame,
    pointer: Pointer,
) -> Option<PanelClickResult> {
    let s = crate::theme::clamp_ui_scale(screen.scale);
    let layouts = compute_panel_layouts(dock, screen.width, screen.height, screen.header_h, s);
    let panel = layouts
        .into_iter()
        .find(|b| b.contains_point(pointer.x, pointer.y))?;
    let seen = panel.seen();
    let frame = ScaledRect {
        x: seen.x,
        y: seen.y,
        w: seen.w,
        h: seen.h,
        scale: s,
    };
    Some(click_panel(dock, store, typo, panel.tab, frame, pointer))
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
        // Cliquer un gabarit ne le pose pas encore (fiche 12, chantier 3.G) ; le clic est
        // tout de même consommé, pour ne pas traverser le panneau jusqu'au canevas.
        TabId::Preset => PanelClickResult::Handled,
        TabId::Domains => {
            let layout = domains::layout_domains_panel(frame, store, &dock.domains);
            let has_selection =
                !store.selected_annotation_ids.is_empty() || !store.selected_image_ids.is_empty();
            match domains::hit_domains_panel(&layout, store, pointer, has_selection) {
                Some(intent) => PanelClickResult::Domain(intent),
                None => PanelClickResult::Handled,
            }
        }
    }
}

#[cfg(test)]
mod tests;
