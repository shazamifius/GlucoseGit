//! Composants graphiques d'interface de Glucose (TopBar, BoardTabs, Minimap, Toasts).

use crate::icons::IconType;
use crate::params::Pointer;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::store::Store;
use tiny_skia::PixmapMut;

pub const TOPBAR_HEIGHT: f32 = 44.0;
pub mod action_bar;
pub mod ancrage;
pub mod bande;
pub mod boutons;
pub mod breadcrumb;
pub mod context_menu;
pub mod minimap;
pub mod onglets;
pub mod options_de_fleche;
pub mod toast;

pub use boutons::{
    draw_action_button, draw_tool_button, layout_topbar, TopbarButtonDef, TopbarLayout,
};
pub use minimap::{layout_minimap, point_minimap, MinimapBounds, MinimapCache};
pub use onglets::{layout_tabs, TabButtonLayout};
pub use toast::{Toast, ToastRepaint};

pub const TABS_HEIGHT: f32 = 34.0;
pub const TOTAL_HEADER_HEIGHT: f32 = TOPBAR_HEIGHT + TABS_HEIGHT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTool {
    Select,
    Pan,
    Text,
    Sticky,
    Arrow,
    Folder,
    Membrane,
}

impl ActiveTool {
    /// Ce que cet outil vient de poser, dit à l'utilisateur — ou `None` pour ceux qui ne
    /// posent rien.
    ///
    /// C'est l'outil qui le sait, donc c'est lui qui le dit. Chaque fabrique portait son
    /// propre message : quatre sites pour un seul événement, donc quatre formulations à
    /// tenir d'accord, et une de plus à chaque outil ajouté.
    /// L'icône qui le désigne dans la barre.
    ///
    /// C'est l'outil qui la connaît, comme il connaît son message de création : la table qui
    /// les appariait dans la mise en page devait être tenue d'accord avec cette énumération,
    /// et un outil ajouté sans y penser aurait pris l'icône du voisin.
    pub fn icone(self) -> IconType {
        match self {
            Self::Select => IconType::Select,
            Self::Pan => IconType::Pan,
            Self::Text => IconType::Text,
            Self::Sticky => IconType::Sticky,
            Self::Arrow => IconType::Arrow,
            Self::Folder => IconType::Folder,
            Self::Membrane => IconType::Membrane,
        }
    }

    pub fn creation_label(self) -> Option<&'static str> {
        match self {
            Self::Select | Self::Pan => None,
            Self::Text => Some("Édition du texte"),
            Self::Sticky => Some("Édition du sticky"),
            Self::Arrow => Some("Flèche ajoutée"),
            Self::Folder => Some("Dossier créé"),
            Self::Membrane => Some("Membrane créée"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiAction {
    SelectTool(ActiveTool),
    AddImages,
    Organize,
    ToggleTimer,
    /// Ouvre ou ferme la Time Machine — ce que fait `Ctrl+H`.
    ToggleTimeMachine,
    ToggleStoryboard,
    ToggleMagnet,
    TransDomain,
    ToggleCollab,
    ExportMenu,
    TogglePlugins,
    TogglePreset,
    ToggleDomains,
    SelectBoard(String),
    /// La croix d'un onglet : le supprimer (BOARDS-1).
    CloseBoard(String),
    AddBoard,
    MinimapPan(f64, f64),
}

pub struct UiState {
    pub active_tool: ActiveTool,
    pub smart_align: bool,
    /// Ce que le pointeur survole dans la bande, tel que la souris l'a vu en dernier : c'est
    /// ce qui lui dit si un mouvement change quelque chose à redessiner.
    pub survol: bande::Survol,
    /// La flèche que la souris survole : ce qu'elle ancre brille dans ses cartes (FLECHE-4).
    pub fleche_survolee: Option<String>,
    /// Sous l'outil Flèche armé, le nœud dont une flèche partirait si l'on appuyait ici : sa
    /// lueur s'avive avant même le premier geste (LUEUR-1).
    pub noeud_pressenti: Option<String>,
    /// L'éditeur d'ancres d'une flèche, quand il est ouvert (FLECHE-4).
    pub ancrage: Option<ancrage::Ancrage>,
    pub current_toast: Option<Toast>,
    /// Le menu contextuel ouvert, et le point où il l'a été. `None` quand il est fermé.
    ///
    /// Dans l'état d'interface et non dans l'application : un menu ouvert n'est pas un geste
    /// en cours, c'est quelque chose qui est **affiché**, au même titre qu'un toast.
    pub context_menu_at: Option<(f32, f32)>,
    pub scale_factor: f32,
    /// Le fond de la minimap, déjà dessiné (voir [`MinimapCache`]).
    pub minimap_cache: Option<MinimapCache>,
    /// La barre et les onglets, déjà dessinés (voir [`bande::BandeCache`]).
    pub bande_cache: Option<bande::BandeCache>,
    /// Ce qu'on est en train de faire aux onglets : les renommer, les glisser (BOARDS-1).
    pub onglets: onglets::EtatDesOnglets,
}

pub const WELCOME_TOAST: &str = "Bienvenue dans Glucose !";

/// `new` ne prend aucun argument : `Default` est donc exactement le même constructeur.
/// Le déclarer évite qu'un appelant générique ait à connaître le nom `new`.
impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

impl UiState {
    /// Un état d'interface **neutre** : aucun message affiché, aucune horloge en marche.
    ///
    /// # Pourquoi le toast de bienvenue n'est plus ici
    ///
    /// Il y était, et il rendait le rendu **non déterministe** : un toast porte un `Instant` de
    /// naissance et une opacité qui en dépend, donc deux frames de la même scène ne donnaient
    /// pas les mêmes pixels — environ sept mille par frame. Rien ne le voyait, parce que rien
    /// ne comparait jamais deux frames ; c'est le banc de capture qui l'a trouvé, le jour même
    /// où il a existé.
    ///
    /// Au-delà de la mesure, un constructeur d'état ne devrait pas déclencher une notification :
    /// « quel est l'état de l'interface » et « que vient-il de se passer » sont deux questions
    /// différentes. Le mot d'accueil est donc posé par [`crate::app::GlucoseApp::new`], là où le
    /// démarrage a lieu.
    pub fn new() -> Self {
        Self {
            active_tool: ActiveTool::Select,
            smart_align: true,
            survol: bande::Survol::default(),
            fleche_survolee: None,
            noeud_pressenti: None,
            ancrage: None,
            current_toast: None,
            context_menu_at: None,
            scale_factor: 1.0,
            minimap_cache: None,
            bande_cache: None,
            onglets: onglets::EtatDesOnglets::default(),
        }
    }

    #[inline]
    pub fn scale(&self) -> f32 {
        crate::theme::clamp_ui_scale(self.scale_factor)
    }

    #[inline]
    pub fn topbar_height(&self) -> f32 {
        TOPBAR_HEIGHT * self.scale()
    }

    #[inline]
    pub fn tabs_height(&self) -> f32 {
        TABS_HEIGHT * self.scale()
    }

    #[inline]
    pub fn header_height(&self) -> f32 {
        TOTAL_HEADER_HEIGHT * self.scale()
    }

    pub fn show_toast(&mut self, msg: impl Into<String>) {
        self.current_toast = Some(Toast::new(msg));
    }

    /// Le message du toast affiché, s'il y en a un — ce que les tests lisent.
    #[cfg(test)]
    pub fn toast_message(&self) -> Option<&str> {
        self.current_toast.as_ref().map(|t| t.message.as_str())
    }
}

/// Ce que dit le bouton Collab tant que la collaboration (fiche 09 § 9) n'existe pas.
pub const NOT_YET_COLLAB: &str = "Collaboration : pas encore disponible";

pub fn render_ui(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &mut UiState,
    typo: &Typography,
    theme: &Theme,
    pointer: Pointer,
) {
    let w = pixmap.width() as f32;
    let h = pixmap.height() as f32;

    // Nettoyage toast expiré
    if let Some(ref t) = ui.current_toast {
        if t.is_expired() {
            ui.current_toast = None;
        }
    }

    // 1 et 2. La barre et les onglets : la bande du haut, rendue une fois par changement.
    //
    // **Refaite, ou seulement recomposee ?** Le poste `bande` vaut 0,22 ms en median et
    // 6,95 ms sur une image de zoom du terrain : un ecart de trente qui ne peut venir que
    // d'un redessin. Reste a savoir combien souvent -- et la duree seule ne le dit pas.
    let dessins_avant = ui.bande_cache.as_ref().map_or(0, |c| c.dessins);
    bande::render_bande(pixmap, store, ui, typo, theme, w, pointer);
    let dessins_apres = ui.bande_cache.as_ref().map_or(0, |c| c.dessins);
    crate::perf::compteur(
        "bande_refaite",
        f64::from(u8::from(dessins_apres != dessins_avant)),
    );
    crate::perf::stage("bande");

    // 2 bis. Fil d'Ariane des dossiers — seulement quand on est entré quelque part.
    breadcrumb::draw_breadcrumb(
        pixmap,
        store,
        typo,
        theme,
        TOTAL_HEADER_HEIGHT * ui.scale(),
        ui.scale_factor,
    );
    // **Le fil d'Ariane a sa marque, et il ne l'avait pas.** `minimap` mesurait depuis
    // `bande`, donc le fil d'Ariane tombait dedans -- QUATRIEME marque de ce dépôt à
    // absorber ce qui la précède, après `occlusion` (fiche 19 § 4.4), `recolte` (fiche 22
    // § 5.4) et `blit` (fiche 23 § 2). Les trois premières ont chacune désigné le mauvais
    // coupable pendant plusieurs sessions.
    crate::perf::stage("ariane");

    // 3. Minimap (en bas à droite)
    let echelle_ui = ui.scale_factor;
    minimap::render_minimap(
        pixmap,
        store,
        theme,
        w,
        h,
        echelle_ui,
        &mut ui.minimap_cache,
    );

    crate::perf::stage("minimap");

    // 4, 5 et 6. Ce qui ne paraît que sur décision : la barre d'action, le toast, le menu.
    poser_ce_qui_attend_une_decision(pixmap, store, ui, typo, theme, (w, h), pointer);
    // **Ce que `ui` nomme désormais** : la barre d'action, le toast et le menu contextuel —
    // ce qui ne paraît que sur décision de l'utilisateur. La marque se posait auparavant chez
    // l'appelant, donc après le retour, et couvrait la bande, le fil d'Ariane et la minimap
    // en plus : un poste qui nomme quatre choses ne désigne rien.
    crate::perf::stage("ui");
}

/// La barre d'action, le toast et le menu contextuel — ce qui attend une décision.
///
/// Extraite de [`render_ui`], qui posait la chrome permanente et celle-ci dans la même
/// fonction : la première paraît toujours, la seconde presque jamais, et les mêler faisait
/// passer le cliquet des quatre-vingts lignes.
///
/// L'ordre entre les trois n'est pas libre : le toast doit rester lisible par-dessus la barre
/// d'action, et le menu par-dessus tout, puisqu'il attend qu'on choisisse.
fn poser_ce_qui_attend_une_decision(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &UiState,
    typo: &Typography,
    theme: &Theme,
    (w, h): (f32, f32),
    pointer: Pointer,
) {
    action_bar::draw_action_bar(pixmap, store, typo, theme, (w, h), ui.scale_factor);
    // Pendant l'édition des ancres, sa fenêtre couvre tout : le moteur de rendu la pose
    // par-dessus l'interface (`poser_la_fenetre_d_ancrage`).
    if ui.ancrage.is_none() {
        options_de_fleche::draw_options_de_fleche(
            pixmap,
            store,
            typo,
            theme,
            ((w, h), ui.scale_factor),
        );
    }
    if let Some(ref toast) = ui.current_toast {
        toast::render_toast(pixmap, toast, typo, theme, w, h, ui.scale_factor);
    }
    let Some(at) = ui.context_menu_at else {
        return;
    };
    let Some(menu) = context_menu::layout_context_menu(
        store,
        typo,
        (at, ui.onglets.menu.as_deref()),
        (w, h),
        ui.scale_factor,
    ) else {
        return;
    };
    context_menu::draw_context_menu(
        pixmap,
        &menu,
        typo,
        theme,
        (pointer.x, pointer.y),
        ui.scale_factor,
    );
}

/// Détecte si un clic souris se situe sur l'interface et retourne l'action associée
pub fn handle_ui_click(
    x: f32,
    y: f32,
    screen_w: f32,
    screen_h: f32,
    store: &Store,
    ui: &mut UiState,
    typo: &Typography,
) -> Option<UiAction> {
    let topbar_h = ui.topbar_height();
    let header_h = ui.header_height();
    let s = ui.scale();

    if y < topbar_h {
        let img_count = store.nombre_d_images();
        let layout = layout_topbar(screen_w, ui, typo, img_count);
        for btn in layout.buttons {
            if x >= btn.x && x < btn.x + btn.w && y >= btn.y && y < btn.y + btn.h {
                match btn.action {
                    UiAction::ToggleMagnet => {
                        ui.smart_align = !ui.smart_align;
                        ui.show_toast(if ui.smart_align {
                            "Aimant activé"
                        } else {
                            "Aimant désactivé"
                        });
                    }
                    UiAction::ToggleCollab => {
                        // Fiche 09 § 9 : aucun réseau n'existe. Le bouton ne « connecte »
                        // rien, et ne doit pas le prétendre.
                        ui.show_toast(NOT_YET_COLLAB);
                    }
                    _ => {}
                }
                return Some(btn.action);
            }
        }
    } else if y >= topbar_h && y < header_h {
        // Les onglets : la même géométrie que le dessin (loi L4).
        return onglets::cible(&layout_tabs(store, ui, typo), x, y).map(|c| match c {
            onglets::CibleOnglet::Onglet(id) => UiAction::SelectBoard(id),
            onglets::CibleOnglet::Fermer(id) => UiAction::CloseBoard(id),
            onglets::CibleOnglet::Plus => UiAction::AddBoard,
        });
    } else if let Some((wx, wy)) = point_minimap(store, x, y, screen_w, screen_h, s) {
        return Some(UiAction::MinimapPan(wx, wy));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typography::Typography;
    use crate::ui::boutons::{ACTION_LABEL_FONT, ACTION_LABEL_PAD_RIGHT, ACTION_LABEL_X};
    use std::time::{Duration, Instant};

    /// Fiche 06 § 9 et § 10 — minimap 180 × 120 à 12 px des bords ; barre d'outils 44 px,
    /// onglets 34 px, boutons d'outil 30 × 30.
    #[test]
    fn test_the_chrome_metrics_are_those_of_the_spec() {
        assert_eq!(TOPBAR_HEIGHT, 44.0);
        assert_eq!(TABS_HEIGHT, 34.0);
        let ui = UiState::new();
        let typo = Typography::new();
        let layout = layout_topbar(1440.0, &ui, &typo, 0);
        let tool = layout
            .buttons
            .iter()
            .find(|b| matches!(b.action, UiAction::SelectTool(_)))
            .expect("un outil");
        assert_eq!((tool.w, tool.h), (30.0, 30.0));
        let store = Store::new("m");
        let mm = layout_minimap(&store, 1440.0, 900.0, 1.0).expect("la minimap");
        assert_eq!((mm.mm_w, mm.mm_h), (180.0, 120.0));
        assert_eq!(
            (mm.mm_x + mm.mm_w, mm.mm_y + mm.mm_h),
            (1440.0 - 12.0, 900.0 - 12.0)
        );
    }

    #[test]
    fn test_topbar_no_overlap_across_all_resolutions() {
        let ui = UiState::new();
        let typo = Typography::new();
        let test_widths = [640.0, 800.0, 1024.0, 1280.0, 1440.0, 1920.0, 2560.0, 3840.0];

        for width in test_widths {
            let layout = layout_topbar(width, &ui, &typo, 5);
            assert!(
                !layout.buttons.is_empty(),
                "Buttons should not be empty for width {}",
                width
            );

            // Vérifier que chaque bouton a une largeur et hauteur positive
            for btn in &layout.buttons {
                assert!(
                    btn.w > 0.0,
                    "Button width must be positive for width {}",
                    width
                );
                assert!(
                    btn.h > 0.0,
                    "Button height must be positive for width {}",
                    width
                );
            }

            // Vérifier qu'aucun bouton ne se chevauche
            for i in 0..layout.buttons.len() {
                for j in (i + 1)..layout.buttons.len() {
                    let b1 = &layout.buttons[i];
                    let b2 = &layout.buttons[j];
                    let overlap_x = b1.x < (b2.x + b2.w) && (b1.x + b1.w) > b2.x;
                    let overlap_y = b1.y < (b2.y + b2.h) && (b1.y + b1.h) > b2.y;
                    assert!(
                        !(overlap_x && overlap_y),
                        "Collision detected at screen width {} between button {} and button {} (b1: [{}, {}], b2: [{}, {}])",
                        width, i, j, b1.x, b1.x + b1.w, b2.x, b2.x + b2.w
                    );
                }
            }
        }
    }

    /// R-51 — le libellé d'un bouton d'action tient dans son cadre, en gras comme en maigre.
    #[test]
    fn test_topbar_labels_fit_inside_their_buttons_whatever_the_weight() {
        let ui = UiState::new();
        let typo = Typography::new();
        for width in [1440.0, 1920.0] {
            let layout = layout_topbar(width, &ui, &typo, 0);
            for btn in layout.buttons.iter().filter(|b| !b.label.is_empty()) {
                // La topbar n'écrit qu'en maigre et en gras : l'italique et la chasse fixe
                // appartiennent au contenu, pas à la chrome.
                for face in [
                    crate::typography::Face::Regular,
                    crate::typography::Face::Bold,
                ] {
                    let (text_w, _) =
                        typo.measure_text(btn.label, ACTION_LABEL_FONT * ui.scale(), face);
                    let right_edge = ACTION_LABEL_X * ui.scale() + text_w;
                    assert!(
                        right_edge <= btn.w - ACTION_LABEL_PAD_RIGHT * ui.scale() + 0.01,
                        "« {} » ({face:?}) finit a {right_edge:.1} px dans un bouton de {:.1} px",
                        btn.label,
                        btn.w
                    );
                }
            }
        }
    }

    #[test]
    fn test_topbar_responsive_collapse() {
        let ui = UiState::new();
        let typo = Typography::new();

        // Mode ultra-compact (< 1050px) : les boutons d'action doivent être réduits à 30px
        let layout_ultra = layout_topbar(900.0, &ui, &typo, 0);
        for btn in &layout_ultra.buttons {
            if btn.label.is_empty() {
                assert_eq!(btn.w, 30.0);
            }
        }

        // Mode complet (> 1320px) : les boutons secondaires ont leurs labels
        let layout_full = layout_topbar(1600.0, &ui, &typo, 2);
        let plugins_btn = layout_full
            .buttons
            .iter()
            .find(|b| b.action == UiAction::TogglePlugins)
            .unwrap();
        assert_eq!(plugins_btn.label, "Plugins");
        assert!(plugins_btn.w > 30.0);
    }

    #[test]
    fn test_click_tabs_selects_correct_board() {
        let mut store = Store::new("Tabs Test");
        let b1 = store.project.active_board_id.clone();
        let b2 = store.add_board("Very Long Custom Board Name That Could Shift Layout");
        let b3 = store.add_board("Short");
        let b4 = store.add_board("Board 4");

        let mut ui = UiState::new();
        let typo = Typography::new();

        // Tester en activant tour à tour chaque onglet pour prouver l'absence de dérive (R-07)
        for target_id in [&b1, &b2, &b3, &b4] {
            store.set_active_board_id(target_id);

            let tabs = layout_tabs(&store, &ui, &typo);
            assert_eq!(tabs.len(), 5); // 4 boards + 1 bouton '+'

            for tab in &tabs {
                let click_x = tab.x + tab.width / 2.0;
                let click_y = tab.y + tab.height / 2.0;

                let action =
                    handle_ui_click(click_x, click_y, 1440.0, 900.0, &store, &mut ui, &typo);
                if tab.is_plus {
                    assert_eq!(action, Some(UiAction::AddBoard));
                } else {
                    assert_eq!(action, Some(UiAction::SelectBoard(tab.board_id.clone())));
                }
            }
        }
    }

    #[test]
    fn test_minimap_click_returns_minimap_pan() {
        let mut store = Store::new("Minimap Test");
        let bid = store.project.active_board_id.clone();
        let img = glucose_core::types::BoardImage::new("img1", 500.0, 300.0, 400.0, 300.0);
        store.add_image(&bid, img);

        let mut ui = UiState::new();
        let typo = Typography::new();

        let mb = layout_minimap(&store, 1440.0, 900.0, ui.scale())
            .expect("Minimap should have valid layout");

        // Clic au centre de la minimap
        let click_x = mb.mm_x + mb.mm_w / 2.0;
        let click_y = mb.mm_y + mb.mm_h / 2.0;

        let action = handle_ui_click(click_x, click_y, 1440.0, 900.0, &store, &mut ui, &typo);
        assert!(
            matches!(action, Some(UiAction::MinimapPan(..))),
            "Minimap click must produce MinimapPan action"
        );
    }

    #[test]
    fn test_ui_dpi_scaling_and_hit_testing_at_150_percent() {
        let mut store = Store::new("DPI Test");
        let b1 = store.project.active_board_id.clone();
        let b2 = store.add_board("Scaled Board");
        store.set_active_board_id(&b2);

        let mut ui = UiState::new();
        ui.scale_factor = 1.5;
        let typo = Typography::new();

        // 1. Topbar à 150 %
        assert_eq!(ui.topbar_height(), 44.0 * 1.5);
        assert_eq!(ui.tabs_height(), 34.0 * 1.5);
        assert_eq!(ui.header_height(), 78.0 * 1.5);

        let layout = layout_topbar(1920.0, &ui, &typo, 1);
        let first_tool = &layout.buttons[0];
        assert_eq!(first_tool.w, 30.0 * 1.5);
        assert_eq!(first_tool.h, 30.0 * 1.5);

        // Clic sur l'outil Pan à 150 %
        let pan_btn = &layout.buttons[1];
        let click_pan_x = pan_btn.x + pan_btn.w / 2.0;
        let click_pan_y = pan_btn.y + pan_btn.h / 2.0;
        let act = handle_ui_click(
            click_pan_x,
            click_pan_y,
            1920.0,
            1080.0,
            &store,
            &mut ui,
            &typo,
        );
        assert_eq!(act, Some(UiAction::SelectTool(ActiveTool::Pan)));

        // 2. Tabs à 150 %
        let tabs = layout_tabs(&store, &ui, &typo);
        assert_eq!(tabs.len(), 3); // b1, b2, plus
        let first_tab = &tabs[0];
        assert_eq!(first_tab.height, 34.0 * 1.5);
        assert_eq!(first_tab.y, 44.0 * 1.5);

        // Clic sur le premier onglet (b1)
        let click_tab_x = first_tab.x + first_tab.width / 2.0;
        let click_tab_y = first_tab.y + first_tab.height / 2.0;
        let tab_act = handle_ui_click(
            click_tab_x,
            click_tab_y,
            1920.0,
            1080.0,
            &store,
            &mut ui,
            &typo,
        );
        assert_eq!(tab_act, Some(UiAction::SelectBoard(b1)));

        // 3. Minimap à 150 %
        let mb = layout_minimap(&store, 1920.0, 1080.0, ui.scale()).expect("Minimap valid layout");
        assert_eq!(mb.mm_w, 180.0 * 1.5);
        assert_eq!(mb.mm_h, 120.0 * 1.5);

        let mm_click_x = mb.mm_x + mb.mm_w / 2.0;
        let mm_click_y = mb.mm_y + mb.mm_h / 2.0;
        let mm_act = handle_ui_click(
            mm_click_x, mm_click_y, 1920.0, 1080.0, &store, &mut ui, &typo,
        );
        assert!(matches!(mm_act, Some(UiAction::MinimapPan(..))));
    }

    #[test]
    fn test_toast_alpha_phases_and_plateau() {
        let mut toast = Toast::new("Test Toast");
        let base_instant = Instant::now();

        // 1. Début du fondu entrant (0 ms)
        toast.created_at = base_instant;
        assert_eq!(toast.alpha(), 0.0);

        // Fiche 07 § 1 : 2 400 ms de vie, 180 ms d'apparition.
        assert_eq!(Toast::DURATION_MS, 2400);
        assert_eq!(Toast::FADE_IN_MS, 180.0);

        // 2. Mi-course du fondu entrant (90 ms / 180 ms)
        toast.created_at = base_instant - Duration::from_millis(90);
        assert!((toast.alpha() - 0.5).abs() < 0.05);

        // 3. Fin du fondu entrant (180 ms)
        toast.created_at = base_instant - Duration::from_millis(180);
        assert_eq!(toast.alpha(), 1.0);

        // 4. Plateau statique où aucun repaint n'est requis (1000 ms) : le toast demande
        // à dormir exactement jusqu'au début du fondu sortant.
        toast.created_at = base_instant - Duration::from_millis(1000);
        assert_eq!(toast.alpha(), 1.0);
        assert_eq!(toast.repaint_need(), ToastRepaint::Sleep(1000));
        toast.created_at = base_instant - Duration::from_millis(90);
        assert_eq!(
            toast.repaint_need(),
            ToastRepaint::Redraw,
            "en plein fondu entrant"
        );
        toast.created_at = base_instant - Duration::from_millis(2200);
        assert_eq!(
            toast.repaint_need(),
            ToastRepaint::Redraw,
            "en plein fondu sortant"
        );
        toast.created_at = base_instant - Duration::from_millis(2400);
        assert_eq!(toast.repaint_need(), ToastRepaint::Gone);

        // 5. Début du fondu sortant (2000 ms = 2400 - 400)
        toast.created_at = base_instant - Duration::from_millis(2000);
        assert_eq!(toast.alpha(), 1.0);

        // 6. Mi-course du fondu sortant (2200 ms = 2400 - 200)
        toast.created_at = base_instant - Duration::from_millis(2200);
        assert!((toast.alpha() - 0.5).abs() < 0.05);

        // 7. Expiration (>= 2400 ms)
        toast.created_at = base_instant - Duration::from_millis(2400);
        assert_eq!(toast.alpha(), 0.0);
        assert!(toast.is_expired());
    }
}
