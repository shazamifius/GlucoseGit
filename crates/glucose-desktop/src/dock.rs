//! Le dock : les tiroirs, chacun **du côté de son bouton**, collés au bord de l'écran sous la
//! bande (TIROIR-4) ; on les glisse pour les réordonner, et on les tire hors de l'écran, vers
//! leur bord, pour les fermer.
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
pub mod discussion;
pub mod domains;
pub mod organize;
pub mod paint;
pub mod plugins;
pub mod pomodoro;
pub mod preset;
mod rangement;
pub mod render;
pub mod storyboard;
pub mod temps;

use crate::params::{Pointer, ScaledRect, ScreenFrame};
use glucose_core::store::Store;

pub use cache::{DockCache, Raison as RaisonDuPanneau};
pub use clic::{handle_dock_click, PanelClickResult};
pub use organize::{apply_organize_layout, LayoutMode, LayoutResult, OrganizeState, SortType};
pub use plugins::OLLAMA_STATUS;
pub use pomodoro::PomodoroState;
pub use rangement::compute_panel_layouts;
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
    /// La Time Machine (fiche 10 § 5.7).
    Temps,
    /// La discussion avec une IA gratuite (fiche 129).
    Discussion,
}

impl TabId {
    /// **TIROIR-4 — un tiroir s'ouvre du côté de son bouton** (fiche 121 § 9 ; sa parole du
    /// 19/11 : *« tout ça, tu le fais exactement comme proposé »*, fiche 122). Sa capture du
    /// 17/11 ne voyait *« aucune structure qui permettrait d'identifier pourquoi ces tiroirs
    /// s'ouvrent comme ça »* : la Time Machine, dont le bouton est à gauche de la bande, s'ouvrait
    /// à droite, et Domaines, à droite, s'ouvrait à gauche. Ordonner, Timer, Time Machine et
    /// Storyboard sont dans le groupe de gauche de la bande ; Plugins, Mood board et Domaines dans
    /// celui de droite (`ui::boutons`) — l'épreuve le lit dans la vraie bande.
    pub fn cote(&self) -> Cote {
        match self {
            Self::Organize | Self::Pomodoro | Self::Temps | Self::Storyboard => Cote::Gauche,
            Self::Plugins | Self::Preset | Self::Domains | Self::Discussion => Cote::Droite,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Organize => "ORDONNER",
            Self::Pomodoro => "POMODORO",
            Self::Storyboard => "STORYBOARD",
            Self::Plugins => "PLUGINS",
            Self::Preset => "MOOD BOARD",
            Self::Domains => "DOMAINES",
            Self::Temps => "TIME MACHINE",
            Self::Discussion => "DISCUSSION",
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
            Self::Discussion => discussion::LARGEUR,
        }
    }
}

/// **TIROIR-2 — un panneau a la hauteur de ce qu'il montre**, en pixels d'écran, poignée
/// comprise ; [`colonnes`] la borne à l'écran. Sa capture du 17/11 : le Storyboard et la Time
/// Machine prenaient toute la hauteur pour quelques lignes, Domaines 430 pixels pour deux
/// domaines — *« des bulles souvent trop grosses »*. Chacun la mesure sur sa propre mise en page.
/// Ordonner et Pomodoro n'ont ni liste ni texte qui varie : leur hauteur est celle de leur
/// contenu, une fois pour toutes (`test_tiroir_2_ordonner_et_pomodoro_sont_pleins`).
fn hauteur_naturelle(dock: &DockManager, store: &Store, tab: TabId, s: f32) -> f32 {
    let largeur = tab.default_width() * s;
    // Tout tiroir a sa poignée sous son contenu (TIROIR-1) : elle s'ajoute.
    let contenu = match tab {
        TabId::Organize => 410.0 * s,
        TabId::Pomodoro => 184.0 * s,
        TabId::Storyboard => storyboard::hauteur_naturelle(&dock.storyboard, largeur, s),
        TabId::Temps => temps::hauteur_naturelle(&dock.temps, largeur, s),
        TabId::Plugins => plugins::hauteur_naturelle(largeur, s),
        TabId::Discussion => discussion::hauteur_naturelle(&dock.discussion, largeur, s),
        TabId::Preset => preset::hauteur_de(store) * s,
        TabId::Domains => domains::hauteur_naturelle(store, &dock.domains, largeur, s),
    };
    contenu + GRIP_HEIGHT * s
}

/// Une hauteur sans borne, où un panneau pose tout ce qu'il a pour se mesurer (TIROIR-2) : bien
/// plus que tout écran, en pixels logiques.
pub(crate) const HAUTEUR_DE_MESURE: f32 = 1.0e6;

/// Ce qu'un tiroir doit passer **au-delà du bord de l'écran**, de son côté, pour se fermer
/// (fiche 10 § 4 : 80 px).
pub const DISMISS_DRAG_PX: f32 = 80.0;

/// Hauteur de la bande de préhension d'un panneau.
const GRIP_HEIGHT: f32 = 16.0;

/// Le côté de l'écran où un tiroir s'ouvre : celui de son bouton (TIROIR-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cote {
    Gauche,
    Droite,
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
    /// Les tiroirs ouverts de chaque côté, dans l'ordre où ils s'empilent : un tiroir qu'on
    /// ouvre se pose sous les autres, sans les faire sauter.
    pub gauche: Vec<TabId>,
    pub droite: Vec<TabId>,
    pub drag: Option<DragSession>,
    pub organize: OrganizeState,
    pub pomodoro: PomodoroState,
    pub storyboard: StoryboardState,
    pub plugins: plugins::PluginsState,
    /// L'état d'**interaction** du panneau DOMAINES : ce qui attend une confirmation, ce qui
    /// est en cours de frappe. Aucune donnée de domaine — celles-ci vivent dans le document
    /// (DOM-UI-1, `dock::domains`).
    pub domains: domains::DomainsUi,
    /// Ce que la Time Machine montre : les gestes, les jalons, le point regardé.
    pub temps: temps::TempsUi,
    /// Ce qui s'est dit avec l'IA, et le champ (fiche 129).
    pub discussion: discussion::DiscussionUi,
    /// Le champ de nombre qu'on tient ou qu'on tape (CHAMP-1).
    pub champ: Option<champ::Saisie>,
    /// De combien chaque tiroir a défilé (TIROIR-6) ; le rangement le borne à ce qu'il cache.
    pub defilement: std::collections::HashMap<TabId, f32>,
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
            gauche: Vec::new(),
            droite: Vec::new(),
            drag: None,
            organize: OrganizeState::default(),
            pomodoro: PomodoroState::default(),
            storyboard: StoryboardState::default(),
            plugins: plugins::PluginsState::default(),
            domains: domains::DomainsUi::default(),
            temps: temps::TempsUi::default(),
            discussion: discussion::DiscussionUi::default(),
            champ: None,
            defilement: Default::default(),
        }
    }

    /// De combien ce tiroir a défilé (TIROIR-6).
    pub fn defile(&self, tab: TabId) -> f32 {
        self.defilement.get(&tab).copied().unwrap_or(0.0)
    }

    fn tabs(&self, cote: Cote) -> &Vec<TabId> {
        match cote {
            Cote::Gauche => &self.gauche,
            Cote::Droite => &self.droite,
        }
    }

    fn tabs_mut(&mut self, cote: Cote) -> &mut Vec<TabId> {
        match cote {
            Cote::Gauche => &mut self.gauche,
            Cote::Droite => &mut self.droite,
        }
    }

    /// Aucun tiroir ouvert, d'aucun côté.
    pub fn tout_fermer(&mut self) {
        self.gauche.clear();
        self.droite.clear();
    }

    pub fn is_open(&self, tab: TabId) -> bool {
        self.tabs(tab.cote()).contains(&tab)
    }

    pub fn toggle_tab(&mut self, tab: TabId) {
        let list = self.tabs_mut(tab.cote());
        match list.iter().position(|&t| t == tab) {
            Some(pos) => {
                list.remove(pos);
            }
            None => list.push(tab),
        }
    }

    pub fn dismiss_tab(&mut self, tab: TabId) {
        self.tabs_mut(tab.cote()).retain(|&t| t != tab);
    }

    /// **Suit le glissement d'un tiroir, et le range où il est vu** : à la place du tiroir de son
    /// côté dont le centre est le plus proche du sien — dans sa colonne ou dans la voisine.
    /// `places` donne la géométrie que le dessin et le clic lisent (loi L4). Le tiroir tiré ne
    /// saute pas : son origine suit sa nouvelle place. Rend `true` s'il a changé de rang.
    pub fn update_drag(
        &mut self,
        (mouse_x, mouse_y): (f32, f32),
        places: impl Fn(&DockManager) -> Vec<PanelLayoutBox>,
    ) -> bool {
        let Some(drag) = self.drag.as_mut() else {
            return false;
        };
        drag.current_x = mouse_x;
        drag.current_y = mouse_y;
        let (tab, decalage) = (drag.tab, (mouse_x - drag.start_x, mouse_y - drag.start_y));
        let avant = places(self);
        let centre = |b: &PanelLayoutBox| (b.x + b.width / 2.0, b.y + b.height / 2.0);
        let Some(lui) = avant.iter().find(|b| b.tab == tab) else {
            return false;
        };
        let (cx, cy) = centre(lui);
        let vu = (cx + decalage.0, cy + decalage.1);
        let distance = |b: &&PanelLayoutBox| {
            let (x, y) = centre(b);
            (x - vu.0).powi(2) + (y - vu.1).powi(2)
        };
        let plus_proche = avant
            .iter()
            .filter(|b| b.tab.cote() == tab.cote())
            .min_by(|a, b| distance(a).total_cmp(&distance(b)))
            .map(|b| b.tab);
        let liste = self.tabs_mut(tab.cote());
        let (Some(cible), Some(rang)) = (plus_proche, liste.iter().position(|&t| t == tab)) else {
            return false;
        };
        let Some(rang_cible) = liste
            .iter()
            .position(|&t| t == cible)
            .filter(|&r| r != rang)
        else {
            return false;
        };
        liste.remove(rang);
        liste.insert(rang_cible, tab);
        let (x0, y0) = (lui.x, lui.y);
        if let Some(apres) = places(self).into_iter().find(|b| b.tab == tab) {
            if let Some(drag) = self.drag.as_mut() {
                drag.start_x += apres.x - x0;
                drag.start_y += apres.y - y0;
            }
        }
        true
    }

    /// **Un tiroir tiré au-delà du bord de l'écran, de son côté, de plus de
    /// [`DISMISS_DRAG_PX`], se ferme** (fiche 10 § 4) : la sortie est le bord où il est collé.
    /// Un tiroir de la colonne voisine traverse la première sans se fermer — c'est un
    /// rangement, pas une sortie. `places` : la géométrie que le dessin lit.
    pub fn finish_drag(
        &mut self,
        largeur_d_ecran: f32,
        places: impl Fn(&DockManager) -> Vec<PanelLayoutBox>,
    ) -> Option<TabId> {
        let tab = self.drag.as_ref()?.tab;
        // Vu où le dessin le montre — le glisser compris —, puis lâché quoi qu'il arrive : un
        // tiroir fermé pendant qu'on le tenait ne laisse pas une main qui tient le vide.
        let vu = places(self)
            .into_iter()
            .find(|b| b.tab == tab)
            .map(|b| b.seen());
        self.drag = None;
        let vu = vu?;
        let sorti = match tab.cote() {
            Cote::Gauche => vu.x < -DISMISS_DRAG_PX,
            Cote::Droite => vu.x + vu.w > largeur_d_ecran + DISMISS_DRAG_PX,
        };
        sorti.then(|| {
            self.dismiss_tab(tab);
            tab
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
    /// La hauteur qu'il aurait si l'écran la lui laissait, poignée comprise (TIROIR-5) : plus
    /// haute que `height`, ce qu'il ne montre pas défile (TIROIR-6).
    pub naturelle: f32,
    /// De combien son contenu a défilé vers le haut — zéro s'il se montre entier.
    pub defile: f32,
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

    /// **TIROIR-1 — le cadre de son contenu** : le panneau sans la bande de sa poignée, en bas.
    /// Un panneau posait son contenu sur toute sa hauteur, poignée comprise : « + Nouveau
    /// domaine » couvrait la poignée qui referme le panneau (sa capture du 17/11).
    ///
    /// **TIROIR-6 — défilé, c'est le cadre de tout son contenu** : à sa hauteur naturelle, remonté
    /// de son défilement. Le panneau s'y range comme sur un écran assez haut, et le dessin comme
    /// le clic lisent ce même cadre (L4) ; seul ce qui tombe dans [`Self::contenu_visible`] se
    /// voit. Montré entier, les deux cadres sont le même.
    pub fn contenu(&self, s: f32) -> ScaledRect {
        let vu = self.contenu_visible(s);
        ScaledRect {
            y: vu.y - self.defile,
            h: (self.naturelle - self.grip_height).max(vu.h),
            ..vu
        }
    }

    /// **Le cadre de son contenu tel qu'il se voit** : le panneau sans la bande de sa poignée.
    pub fn contenu_visible(&self, s: f32) -> ScaledRect {
        let vu = self.seen();
        ScaledRect {
            x: vu.x,
            y: vu.y,
            w: vu.w,
            h: vu.h - self.grip_height,
            scale: s,
        }
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

    /// Tout ce que le panneau peut noircir : son cadre et son trait de bordure — plat, sans
    /// ombre (TIROIR-4 : la charte, `style.md` : « pas d'ombre portée molle »).
    ///
    /// C'est la taille du tampon qu'un panneau mis en cache doit se réserver (DOCK-CACHE-1).
    /// Elle est **exacte et dérivée**, jamais une marge choisie au jugé : un tampon trop
    /// petit rognerait le trait, un tampon trop grand paierait des pixels vides à chaque
    /// composition.
    fn extent(&self, s: f32) -> WidgetRect {
        let seen = self.seen();
        // Le contour est tracé *sur* le bord : il déborde d'une demi-épaisseur.
        let demi_trait = 0.5 * s;
        WidgetRect::new(
            seen.x - demi_trait,
            seen.y - demi_trait,
            seen.w + 2.0 * demi_trait,
            seen.h + 2.0 * demi_trait,
        )
    }
}

// ── Le rendu ────────────────────────────────────────────────────────────────

/// **Le cadre du contenu d'un panneau à l'écran**, tel que son dessin et son clic le lisent
/// (TIROIR-1) — `None` s'il est fermé. Le panneau sous un point est celui que [`panneau_sous`]
/// désigne.
pub fn cadre_du_panneau(
    dock: &DockManager,
    store: &Store,
    screen: ScreenFrame,
    tab: TabId,
) -> Option<ScaledRect> {
    cadres(dock, store, screen)
        .find(|(t, _, _)| *t == tab)
        .map(|(_, _, contenu)| contenu)
}

/// **Le panneau sous ce point**, et le cadre de son contenu : le clic va à lui, et à personne
/// d'autre. Le panneau entier le prend, poignée comprise ; là où deux se croisent (un écran trop
/// étroit pour leurs colonnes), celui qui est dessiné dessus, le dernier.
pub fn panneau_sous(
    dock: &DockManager,
    store: &Store,
    screen: ScreenFrame,
    p: Pointer,
) -> Option<(TabId, ScaledRect)> {
    let cadres: Vec<_> = cadres(dock, store, screen).collect();
    cadres
        .into_iter()
        .rev()
        .find(|(_, vu, _)| vu.contains(p.x, p.y))
        .map(|(tab, _, contenu)| (tab, contenu))
}

/// Les panneaux ouverts et les cadres de leurs contenus : ce que le dessin et le clic lisent.
#[cfg(test)]
pub fn cadres_ouverts(
    dock: &DockManager,
    store: &Store,
    screen: ScreenFrame,
) -> Vec<(TabId, ScaledRect)> {
    cadres(dock, store, screen)
        .map(|(tab, _, contenu)| (tab, contenu))
        .collect()
}

/// **La place des tiroirs sur cet écran** — celle que le dessin, le clic et le glisser lisent
/// (loi L4).
pub fn places(dock: &DockManager, store: &Store, screen: ScreenFrame) -> Vec<PanelLayoutBox> {
    compute_panel_layouts(
        dock,
        store,
        screen.width,
        screen.height,
        screen.header_h,
        crate::theme::clamp_ui_scale(screen.scale),
    )
}

/// Les panneaux ouverts : ce qu'ils couvrent à l'écran, et le cadre de leur contenu, à
/// l'échelle bornée.
fn cadres(
    dock: &DockManager,
    store: &Store,
    screen: ScreenFrame,
) -> impl Iterator<Item = (TabId, WidgetRect, ScaledRect)> {
    let s = crate::theme::clamp_ui_scale(screen.scale);
    places(dock, store, screen)
        .into_iter()
        .map(move |b| (b.tab, b.seen(), b.contenu(s)))
}

/// **La réglette de la Time Machine, là où elle est à l'écran** — la même géométrie que le
/// dessin et le clic. `None` si le panneau est fermé.
pub fn reglette_a_l_ecran(
    dock: &DockManager,
    store: &Store,
    screen: ScreenFrame,
) -> Option<WidgetRect> {
    let cadre = cadre_du_panneau(dock, store, screen, TabId::Temps)?;
    Some(temps::layout_temps_panel(cadre, &dock.temps).reglette)
}

/// Le point de la réglette sous l'abscisse `x` — ce que le glisser lit à chaque mouvement.
pub fn point_de_la_reglette(
    dock: &DockManager,
    store: &Store,
    screen: ScreenFrame,
    x: f32,
) -> Option<usize> {
    let reglette = reglette_a_l_ecran(dock, store, screen)?;
    Some(temps::point_sous(reglette, dock.temps.gestes.len(), x))
}

#[cfg(test)]
mod tests;
