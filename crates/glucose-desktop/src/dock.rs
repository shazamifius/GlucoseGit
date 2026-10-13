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
//! | [`preset`] | PRESETS | agit : il arme un gabarit, que le clic pose sur le tableau (PRESET-1) |
//! | [`temps`] | TIME MACHINE | agit : l'histoire du document, l'aperçu du passé, restaurer, les jalons |
//!
//! Chaque module porte **son** état, **sa** géométrie et **son** clic ; son sous-module
//! `paint` porte son dessin. Ce fichier n'a plus que ce qui leur est commun : le dock
//! lui-même et ses cadres ; [`clic`] aiguille le clic, [`champ`] porte les champs de nombre.
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
pub mod champ;
pub mod clic;
pub mod domains;
pub mod organize;
pub mod paint;
pub mod plugins;
pub mod pomodoro;
pub mod preset;
pub mod render;
pub mod storyboard;
pub mod temps;

use crate::params::{Pointer, ScaledRect, ScreenFrame};

pub use cache::{DockCache, Raison as RaisonDuPanneau};
pub use clic::{handle_dock_click, PanelClickResult};
pub use organize::{apply_organize_layout, LayoutMode, LayoutResult, OrganizeState, SortType};
pub use plugins::OLLAMA_STATUS;
pub use pomodoro::PomodoroState;
pub use render::{confier_les_docks, lisere, lisere_du_passe, render_docks, DockPass};
pub use storyboard::StoryboardState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabId {
    Organize,
    Pomodoro,
    Storyboard,
    Plugins,
    Preset,
    Domains,
    /// La Time Machine (fiche 10 § 5.7) : seule à droite, sur toute la hauteur.
    Temps,
}

impl TabId {
    pub fn anchor(&self) -> DockAnchor {
        match self {
            Self::Organize | Self::Pomodoro | Self::Storyboard => DockAnchor::BottomLeft,
            Self::Plugins | Self::Preset | Self::Domains => DockAnchor::TopLeft,
            Self::Temps => DockAnchor::Right,
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
            Self::Temps => "TIME MACHINE",
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
            Self::Temps => 324.0,
        }
    }

    pub fn default_height(&self) -> f32 {
        match self {
            Self::Organize => 410.0,
            Self::Pomodoro => 210.0,
            // Toute la hauteur disponible : ses cases s'y listent (STORYBOARD-1).
            Self::Storyboard => f32::MAX,
            Self::Plugins => 460.0,
            // Celle de ce qu'il peut montrer de plus grand, mesurée sur sa mise en page.
            Self::Preset => preset::hauteur_naturelle(),
            Self::Domains => 430.0,
            // Toute la hauteur disponible : `place` la borne à l'écran.
            Self::Temps => f32::MAX,
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
    /// Le bord droit, sur toute la hauteur : la Time Machine, comme chez Glucose Tauri.
    Right,
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
    /// L'état d'**interaction** du panneau DOMAINES : ce qui attend une confirmation, ce qui
    /// est en cours de frappe. Aucune donnée de domaine — celles-ci vivent dans le document
    /// (DOM-UI-1, `dock::domains`).
    pub domains: domains::DomainsUi,
    /// Le panneau de droite : la Time Machine.
    pub right_tabs: Vec<TabId>,
    /// Ce que la Time Machine montre : les gestes, les jalons, le point regardé.
    pub temps: temps::TempsUi,
    /// Le champ de nombre qu'on tient ou qu'on tape (CHAMP-1).
    pub champ: Option<champ::Saisie>,
}

impl Default for DockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DockManager {
    /// **Aucun panneau ouvert au lancement** (fiche 59) : Glucose Tauri ouvrait Ordonner et
    /// Pomodoro, et sur son téléphone ils couvraient la question du journal technique —
    /// « commencer Glucose avec deux menus déroulés, c'est un peu chiant ». Un panneau s'ouvre
    /// quand on le demande.
    pub fn new() -> Self {
        Self {
            top_tabs: Vec::new(),
            bottom_tabs: Vec::new(),
            drag: None,
            organize: OrganizeState::default(),
            pomodoro: PomodoroState::default(),
            storyboard: StoryboardState::default(),
            plugins: plugins::PluginsState::default(),
            domains: domains::DomainsUi::default(),
            right_tabs: Vec::new(),
            temps: temps::TempsUi::default(),
            champ: None,
        }
    }

    fn tabs(&self, anchor: DockAnchor) -> &Vec<TabId> {
        match anchor {
            DockAnchor::TopLeft => &self.top_tabs,
            DockAnchor::BottomLeft => &self.bottom_tabs,
            DockAnchor::Right => &self.right_tabs,
        }
    }

    fn tabs_mut(&mut self, anchor: DockAnchor) -> &mut Vec<TabId> {
        match anchor {
            DockAnchor::TopLeft => &mut self.top_tabs,
            DockAnchor::BottomLeft => &mut self.bottom_tabs,
            DockAnchor::Right => &mut self.right_tabs,
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
        let dx = drag.current_x - drag.start_x;
        let dismissed = match drag.tab.anchor() {
            DockAnchor::TopLeft => dy < -DISMISS_DRAG_PX,
            DockAnchor::BottomLeft => dy > DISMISS_DRAG_PX,
            DockAnchor::Right => dx > DISMISS_DRAG_PX,
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
        self.grip_rect().contains(px, py)
    }

    /// Le rectangle de la poignée, là où l'œil la voit.
    pub fn grip_rect(&self) -> WidgetRect {
        let seen = self.seen();
        WidgetRect::new(
            seen.x,
            self.grip_y + self.visual_offset_y,
            seen.w,
            self.grip_height,
        )
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
    screen_w: f32,
    screen_h: f32,
    header_h: f32,
    scale: f32,
) -> Vec<PanelLayoutBox> {
    let s = crate::theme::clamp_ui_scale(scale);
    let mut layouts = Vec::new();
    for anchor in [
        DockAnchor::TopLeft,
        DockAnchor::BottomLeft,
        DockAnchor::Right,
    ] {
        let mut x = DOCK_MARGIN * s;
        for &tab in dock.tabs(anchor) {
            let width = tab.default_width() * s;
            if anchor == DockAnchor::Right {
                x = screen_w - DOCK_MARGIN * s - width;
            }
            let (y, height) = place(anchor, tab, screen_h, header_h, s);
            let grip_height = GRIP_HEIGHT * s;
            // La poignée est du côté de la sortie : en bas pour le dock du haut, en haut
            // pour celui du bas (fiche 10 § 4) — et en haut à droite, où l'on tire vers le
            // bord.
            let grip_y = match anchor {
                DockAnchor::TopLeft => y + height - grip_height,
                DockAnchor::BottomLeft | DockAnchor::Right => y,
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
        DockAnchor::Right => {
            let y = header_h + 8.0 * s;
            (y, (tab.default_height() * s).min(screen_h - y - margin))
        }
    }
}

// ── Le rendu ────────────────────────────────────────────────────────────────

/// **Le cadre d'un panneau à l'écran**, tel que son dessin et son clic le lisent — `None` s'il
/// est fermé. Le panneau sous un point est celui que [`panneau_sous`] désigne.
pub fn cadre_du_panneau(dock: &DockManager, screen: ScreenFrame, tab: TabId) -> Option<ScaledRect> {
    cadres(dock, screen)
        .find(|(t, _)| *t == tab)
        .map(|(_, f)| f)
}

/// **Le panneau sous ce point**, et son cadre : le clic va à lui, et à personne d'autre.
pub fn panneau_sous(
    dock: &DockManager,
    screen: ScreenFrame,
    p: Pointer,
) -> Option<(TabId, ScaledRect)> {
    cadres(dock, screen).find(|(_, f)| WidgetRect::new(f.x, f.y, f.w, f.h).contains(p.x, p.y))
}

/// Les panneaux ouverts et leurs cadres, à l'échelle bornée.
fn cadres(dock: &DockManager, screen: ScreenFrame) -> impl Iterator<Item = (TabId, ScaledRect)> {
    let s = crate::theme::clamp_ui_scale(screen.scale);
    let boites = compute_panel_layouts(dock, screen.width, screen.height, screen.header_h, s);
    boites.into_iter().map(move |b| {
        let vu = b.seen();
        let cadre = ScaledRect {
            x: vu.x,
            y: vu.y,
            w: vu.w,
            h: vu.h,
            scale: s,
        };
        (b.tab, cadre)
    })
}

/// **La réglette de la Time Machine, là où elle est à l'écran** — la même géométrie que le
/// dessin et le clic. `None` si le panneau est fermé.
pub fn reglette_a_l_ecran(dock: &DockManager, screen: ScreenFrame) -> Option<WidgetRect> {
    let cadre = cadre_du_panneau(dock, screen, TabId::Temps)?;
    Some(temps::layout_temps_panel(cadre, &dock.temps).reglette)
}

/// Le point de la réglette sous l'abscisse `x` — ce que le glisser lit à chaque mouvement.
pub fn point_de_la_reglette(dock: &DockManager, screen: ScreenFrame, x: f32) -> Option<usize> {
    let reglette = reglette_a_l_ecran(dock, screen)?;
    Some(temps::point_sous(reglette, dock.temps.gestes.len(), x))
}

#[cfg(test)]
mod tests;
