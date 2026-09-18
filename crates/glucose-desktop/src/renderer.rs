//! Moteur de rendu 2D haute fidélité pour Glucose Desktop (PureRef-style).
//! Utilise tiny-skia pour le rendu vectoriel anti-aliasé et fontdue pour la typographie.
//!
//! Ce fichier n'est plus que l'**ordonnanceur** d'une frame : il tient les caches, fait la
//! requête spatiale (loi L1) et appelle les passes dans l'ordre. Chaque passe vit dans son
//! module, avec ses mesures et ses tests :
//!
//! | Module | Passe |
//! |---|---|
//! | [`scale`] | l'unique mise à l'échelle monde → écran (standard § 4.4) |
//! | [`hue`] | les teintes symbiotiques et leur invalidation |
//! | [`domain`] | la réglette de domaines et la table de teintes qui l'alimente |
//! | [`scene`] | grille, membranes, images, guides, boîte de sélection |
//! | [`halo`] | les halos d'ambiance |
//! | [`card`] | les cartes de texte |
//! | [`note`] | les pense-bêtes et les flèches |
//! | [`handles`] | les poignées de redimensionnement, là où le test de clic les cherche |
//! | [`wrap`] | le découpage d'un paragraphe en lignes (WRAP-1) |

pub mod arrow;
pub mod arrow_label;
pub mod atelier;
pub mod card;
pub mod domain;
pub mod folder;
pub mod halo;
pub mod handles;
pub mod hue;
pub mod magasin;
pub mod math;
pub mod note;
pub mod pass;
pub mod photo;
pub mod predicate;
pub mod richtext;
pub mod scale;
pub mod scene;
pub mod vignette;
pub mod wrap;

use crate::canvas::screen_to_world;
use crate::params::{Pointer, SceneOverlay, ViewPass};
use crate::theme::Theme;
use crate::typography::Typography;
use crate::ui::{render_ui, UiState};
use domain::DomainTints;
use glucose_core::quadtree::SpatialHash;
use glucose_core::store::Store;
use glucose_core::text::Selection;
use hue::SymbioticHueCache;
use tiny_skia::{PathBuilder, PixmapMut};

#[derive(Debug, Clone)]
pub struct TextEditSession {
    pub ann_id: String,
    pub buffer: String,
    /// Ce qui est sélectionné, et où le curseur clignote : sa tête (SEL-1). Une sélection
    /// vide **est** un curseur — il n'y a donc qu'un état à tenir, pas deux.
    pub selection: Selection,
    /// L'abscisse que `↑` et `↓` cherchent à retrouver, en unités monde.
    ///
    /// # COLUMN-1 — la colonne se mémorise, elle ne se recalcule pas
    ///
    /// Descendre d'une ligne pose le curseur sur la frontière de caractère la plus proche de
    /// l'abscisse visée, jamais exactement dessus. Recalculer l'abscisse depuis cette nouvelle
    /// position à chaque pas fait **dériver** le curseur vers la gauche, un peu à chaque ligne
    /// — le défaut le plus connu des éditeurs qui ne gardent pas cette valeur.
    ///
    /// Elle est posée au premier mouvement vertical et oubliée dès qu'autre chose bouge le
    /// curseur, pour que la colonne suive alors la nouvelle position.
    pub goal_x: Option<f32>,
    pub blink_timer: std::time::Instant,
}

/// Décode une couleur hexadécimale `#rrggbb` ou `#rgb` ; `None` si ce n'en est pas une.
///
/// Le document range ses couleurs en texte, et un texte peut être faux : l'appelant décide
/// alors de la couleur de repli — en général un jeton du thème — plutôt que d'en recevoir une
/// choisie ici.
pub(crate) fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let s = hex.trim_start_matches('#');
    let channel = |from: usize, to: usize| u8::from_str_radix(s.get(from..to)?, 16).ok();
    match s.len() {
        6 => Some((channel(0, 2)?, channel(2, 4)?, channel(4, 6)?)),
        3 => Some((
            channel(0, 1)? * 17,
            channel(1, 2)? * 17,
            channel(2, 3)? * 17,
        )),
        _ => None,
    }
}

/// [`parse_hex_rgb`], avec une couleur de repli.
pub(crate) fn parse_hex_color(
    hex: &str,
    default_r: u8,
    default_g: u8,
    default_b: u8,
) -> (u8, u8, u8) {
    parse_hex_rgb(hex).unwrap_or((default_r, default_g, default_b))
}

/// Ajoute un rectangle à coins arrondis dans un PathBuilder.
///
/// Le rayon est ramené à la moitié du plus petit côté : c'est une contrainte **géométrique**
/// — un coin ne peut pas être plus rond que la forme — et non une borne sur une valeur
/// dérivée du zoom, puisque rayon et côtés subissent la même mise à l'échelle.
pub(crate) fn push_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}

/// Ce avec quoi une passe peint, et qui ne change pas de la frame : la police, la table des
/// teintes de domaine et le thème. Un seul paramètre au lieu de trois (R-44).
#[derive(Clone, Copy)]
pub(crate) struct PaintKit<'a> {
    pub typography: &'a Typography,
    pub math: &'a math::MathRenderer,
    pub tints: &'a DomainTints,
    pub theme: &'a Theme,
}

pub struct Renderer {
    pub theme: Theme,
    /// Tout ce qui sert à poser une image : le cache, les vignettes, les échecs, et les fils
    /// qui décodent pendant que la scène continue de se dessiner (DECODE-1).
    pub magasin: magasin::Magasin,
    pub typography: Typography,
    pub math: math::MathRenderer,
    pub hue_cache: SymbioticHueCache,
    /// `domain_id → teinte`, reconstruite une fois par version du document (DOMAIN-TINT-1).
    pub domain_tints: DomainTints,
    /// Ce que cette machine coûte, appris de ce qu'elle vient de faire (COUT-1).
    ///
    /// Il vit ici, et non dans le magasin : il ne décrit pas les images mais **la machine**,
    /// et il servira à toutes les passes le jour où elles sauront dire ce qu'elles vont
    /// écrire. Aujourd'hui il ne connaît que les pixels de photos, qui sont le gros poste.
    pub cout: glucose_core::cout::Cout,
    pub spatial_hash: SpatialHash,
    pub spatial_version: u64,
    pub active_board_id: String,
}

/// `new` ne prend aucun argument : `Default` est donc exactement le même constructeur.
/// Le déclarer évite qu'un appelant générique ait à connaître le nom `new`.
impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            theme: Theme::dark(),
            magasin: magasin::Magasin::nouveau(),
            typography: Typography::new(),
            math: math::MathRenderer::new(),
            hue_cache: SymbioticHueCache::new(),
            domain_tints: DomainTints::new(),
            cout: glucose_core::cout::Cout::nouveau(),
            spatial_hash: SpatialHash::new(1000.0),
            spatial_version: 0,
            active_board_id: String::new(),
        }
    }

    /// Rendu complet de la scène Glucose et de son interface
    /// Met l'index spatial à jour si le document a changé depuis la dernière fois.
    ///
    /// Appelée par le rendu **et par le picking**. Le clic en dépendait autrefois par effet de
    /// bord : l'index n'était synchronisé que dans `render`, si bien qu'un nœud créé puis
    /// cliqué avant la frame suivante était introuvable. Un geste ne doit pas dépendre de ce
    /// qu'une autre passe a bien voulu faire avant lui.
    ///
    /// Ne coûte rien quand rien n'a changé : la comparaison de version précède le balayage.
    pub fn sync_spatial_index(&mut self, store: &Store) {
        let Some(board) = store.active_board() else {
            return;
        };
        if self.spatial_version != store.version || self.active_board_id != board.id {
            self.spatial_hash.index_board(board);
            self.spatial_version = store.version;
            self.active_board_id = board.id.clone();
        }
    }

    /// Remet les caches du rendu d'accord avec le document, avant de dessiner quoi que ce soit.
    ///
    /// # Ce que cette méthode a coûté d'être anonyme
    ///
    /// Ces trois lignes vivaient au début de `render`, dans le silence, et le chronomètre les
    /// comptait sous l'étiquette `cull`. Le nom mentait : le culling est la requête qui suit,
    /// et elle répond en une fraction de milliseconde. Ce qui coûtait, c'était **ceci** — deux
    /// passes sur le document entier et une reconstruction d'index — et personne ne le
    /// cherchait là, puisque le poste semblait être du culling.
    ///
    /// Mesuré sur un million de nœuds : 315 ms par image sans qu'aucune mutation n'ait eu
    /// lieu, et jusqu'à 2 100 ms après une. Un poste de mesure qui porte le nom d'autre chose
    /// est pire qu'un poste absent : il envoie chercher au mauvais endroit.
    /// Les trois postes sont chronométrés séparément : ils reparcourent tous le document, mais
    /// pas pour les mêmes raisons ni au même prix, et un poste agrégé les rendrait
    /// indiscernables. Mesuré à un million de nœuds, la première image après une mutation :
    /// l'index pèse 1 100 ms quand une image est ajoutée et 4 ms quand c'est une note.
    fn synchroniser_les_caches(&mut self, store: &Store) {
        // Ce que les fils de fond ont fini entre deux images entre dans les caches ici, et
        // nulle part ailleurs : le rendu voit ensuite un cache qui ne bouge pas sous ses
        // pieds. Une récolte est une remise d'accord comme les trois autres, et c'est bien
        // ici qu'elle appartient.
        self.magasin.recolter();
        crate::perf::stage("recolte");
        self.domain_tints.refresh(store, &self.theme);
        crate::perf::stage("teintes");
        if let Some(board) = store.active_board() {
            self.hue_cache.suivre(store.version, &board.id);
        }
        crate::perf::stage("hues");
        self.sync_spatial_index(store);
        crate::perf::stage("index");
    }

    pub fn render(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        ui: &mut UiState,
        overlay: SceneOverlay<'_>,
        pointer: Pointer,
    ) {
        self.magasin.ouvrir();
        self.synchroniser_les_caches(store);
        self.rendre_la_scene(pixmap, store, ui, overlay, ui.header_height());

        // 9. Interface utilisateur complete (TopBar, Tabs, Minimap, Toasts)
        render_ui(pixmap, store, ui, &self.typography, &self.theme, pointer);
        crate::perf::stage("ui");
        self.magasin.fermer();
    }

    /// Ce qui suit la vue : le fond, la grille, les halos, les conteneurs, les images, les
    /// annotations et les repères de geste. Tout ce que `world_to_screen` place.
    ///
    /// # Pourquoi c'est separe de la chrome (prealable a A.1)
    ///
    /// Un banc a pose la question qui decide de toute la vague A : **rendre une region
    /// donne-t-il les memes pixels que rendre tout ?** Sur l'ecran entier, oui, au bit pres.
    /// Sur une region, non : 11 a 46 % d'ecart, des la premiere ligne.
    ///
    /// La cause n'etait pas la scene mais l'**interface**, qui se place sur la taille du
    /// pixmap et non sur la vue : dans une sous-fenetre, elle se redessine au mauvais endroit.
    /// La scene, elle, se decale exactement avec `vp` -- decaler la vue de `-x0` revient a
    /// deplacer l'origine de l'ecran en `x0`, puisque `world_to_screen` vaut
    /// `monde x echelle + vp`.
    ///
    /// Les deux n'ont donc pas la meme loi et ne peuvent pas partager une salissure : la scene
    /// se salit en coordonnees **monde**, la chrome en coordonnees **ecran**. Les separer
    /// n'est pas un rangement, c'est la condition pour que l'une puisse se rendre par region
    /// pendant que l'autre ne bouge pas.
    ///
    /// `header_h` est passe plutot que lu sur l'interface : dans une region qui commence en
    /// `y0`, le bandeau se trouve `y0` pixels plus haut, et peut etre entierement au-dessus.
    pub fn rendre_la_scene(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        ui: &UiState,
        overlay: SceneOverlay<'_>,
        header_h: f32,
    ) {
        self.rendre_la_region(pixmap, store, ui, overlay, header_h, (0.0, 0.0));
    }

    /// La scène, rendue comme si l'origine de l'écran était `origine` (A.1).
    ///
    /// `world_to_screen` vaut `monde x echelle + vp` : décaler la vue de `-origine` déplace
    /// donc l'origine de l'écran d'autant, exactement. Rendre la région `(x0, y0, w, h)` dans
    /// une image de `w x h` revient à rendre la scène entière avec `vp.x -= x0`.
    ///
    /// Aucune passe n'a besoin de le savoir, et c'est tout l'intérêt : la scène ignore
    /// qu'elle est partielle. Le culling, lui, se resserre tout seul -- il part des bords du
    /// pixmap, qui sont ceux de la région.
    ///
    /// `bench_zone` mesure que le résultat est identique au bit près à un rendu complet, à
    /// condition de déborder de la portée du flou des halos.
    pub fn rendre_la_region(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        ui: &UiState,
        overlay: SceneOverlay<'_>,
        header_h: f32,
        origine: (f32, f32),
    ) {
        // L'index spatial se remet d'accord ici, et non chez l'appelant. Il l'etait dans
        // `render`, si bien qu'un appelant de `rendre_la_scene` -- un banc, un temoin --
        // dessinait un ecran VIDE sans que rien ne le dise. C'est arrive, et le banc annoncait
        // alors un gain nul en toute bonne foi.
        //
        // Ne coute rien quand rien n'a change : la comparaison de version precede le balayage.
        self.sync_spatial_index(store);
        let width = pixmap.width();
        let height = pixmap.height();
        let mut vp = store.viewport();
        vp.x -= f64::from(origine.0);
        vp.y -= f64::from(origine.1);
        let (min_wx, min_wy) = screen_to_world(0.0, header_h as f64, &vp);
        let (max_wx, max_wy) = screen_to_world(width as f64, height as f64, &vp);
        let rangs = self
            .spatial_hash
            .query_rect_ranks(min_wx, min_wy, max_wx, max_wy, 200.0);
        crate::perf::stage("cull");
        let pass = ViewPass {
            vp,
            visibles: &rangs,
            index: &self.spatial_hash,
            header_h,
        };
        let kit = PaintKit {
            typography: &self.typography,
            math: &self.math,
            tints: &self.domain_tints,
            theme: &self.theme,
        };

        // 1. Le fond du canevas
        pixmap.fill(self.theme.bg_canvas);
        crate::perf::stage("clear");

        // 2. Grille de points infinie
        scene::grid::draw_grid(pixmap, &vp, width, height, header_h);
        crate::perf::stage("grid");

        // 3. Halos symbiotiques d'ambiance (Biome 2D + composition par anneaux)
        halo::draw_halos(&mut self.hue_cache, pixmap, store, pass);
        crate::perf::stage("halos");

        // 4. Membranes (pointillés, titre protecteur en haut à gauche)
        scene::draw_membranes(kit, pixmap, store, pass);
        crate::perf::stage("membranes");

        // 4 bis. Dossiers — des portails vers un autre tableau, donc dessinés AVEC les autres
        // conteneurs et sous leur contenu. Ils n'avaient aucun pixel jusqu'ici.
        folder::draw_folders(kit, pixmap, store, pass);
        crate::perf::stage("folders");

        // 5. Images — le magasin pour les poser, le modele de cout pour apprendre leur prix.
        scene::draw_images(&mut self.magasin, &mut self.cout, kit, pixmap, store, pass);
        crate::perf::stage("images");

        // 6. Annotations (cartes de texte, pense-bêtes, flèches + édition live in-place)
        pass::draw_annotations(
            &mut self.hue_cache,
            kit,
            pixmap,
            store,
            overlay.editing,
            pass,
        );
        crate::perf::stage("annotations");

        self.dessiner_les_reperes_du_geste(pixmap, ui, overlay, vp, (width, height), header_h);
    }

    /// Les repères du geste en cours : les guides d'alignement et la boîte de sélection.
    ///
    /// Ils appartiennent à la scène parce qu'ils suivent la vue, mais pas au contenu : ils
    /// n'existent que pendant un geste, ne sont dans aucun document, et disparaîtront sans
    /// laisser de trace. C'est aussi ce qui les distingue pour A.1 — ils salissent l'écran à
    /// chaque mouvement de la main, et rien d'autre ne le fait pour eux.
    fn dessiner_les_reperes_du_geste(
        &self,
        pixmap: &mut PixmapMut,
        ui: &UiState,
        overlay: SceneOverlay<'_>,
        vp: glucose_core::types::Viewport,
        taille: (u32, u32),
        header_h: f32,
    ) {
        // 7. Guides d'alignement intelligents (SNAP-1)
        if ui.smart_align {
            scene::draw_guides(&self.theme, pixmap, overlay.guides, &vp, taille, header_h);
        }

        // 8. Boîte de sélection élastique (Marquee)
        if let Some((x1, y1, x2, y2)) = overlay.selection_box {
            scene::draw_selection_box(pixmap, &self.theme, (x1, y1), (x2, y2));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ScreenFrame;
    use glucose_core::smart_align::SnapGuides;
    use tiny_skia::Pixmap;

    /// Budget de temps d'une frame complète (scène + docks) en 1440x900, en debug.
    ///
    /// Garde-fou de démarrage : une frame qui dépasse ce budget affame la pompe
    /// de messages de l'OS et produit une fenêtre blanche « Ne répond pas ».
    const FULL_FRAME_BUDGET_MS: u128 = 2_000;

    fn render_one_frame(
        renderer: &mut Renderer,
        pixmap: &mut Pixmap,
        store: &Store,
        ui: &mut UiState,
        dock: &crate::dock::DockManager,
    ) {
        let guides = SnapGuides::default();
        let mut view = pixmap.as_mut();
        let overlay = SceneOverlay {
            guides: &guides,
            selection_box: None,
            editing: None,
        };
        let origin = Pointer { x: 0.0, y: 0.0 };
        renderer.render(&mut view, store, ui, overlay, origin);
        crate::dock::render_docks(
            &mut view,
            dock,
            store,
            &crate::dock::DockPass {
                typo: &renderer.typography,
                theme: &renderer.theme,
                screen: ScreenFrame {
                    width: 1440.0,
                    height: 900.0,
                    header_h: ui.header_height(),
                    scale: ui.scale_factor,
                },
                pointer: origin,
                cache: None,
            },
        );
    }

    #[test]
    fn test_full_frame_render_stays_within_time_budget() {
        let mut pixmap = Pixmap::new(1440, 900).expect("pixmap 1440x900");
        let mut store = Store::new("Budget");
        let board_id = store.project.active_board_id.clone();
        store.add_annotation(&board_id, card::tests::probe_card("budget-1", 0.0, 0.0));

        let mut renderer = Renderer::new();
        let mut ui = UiState::new();
        let dock = crate::dock::DockManager::new();

        // Frame de chauffe : remplit le cache de glyphes et l'index spatial.
        render_one_frame(&mut renderer, &mut pixmap, &store, &mut ui, &dock);

        let started = std::time::Instant::now();
        render_one_frame(&mut renderer, &mut pixmap, &store, &mut ui, &dock);
        let elapsed = started.elapsed().as_millis();

        assert!(
            elapsed < FULL_FRAME_BUDGET_MS,
            "frame complète : {elapsed} ms (budget {FULL_FRAME_BUDGET_MS} ms)"
        );
        // Ce que la frame coûte au cache de glyphes, phases sous-pixel comprises (GLYPH-1).
        println!(
            "[perf] frame complète : {elapsed} ms, {} variantes de glyphes en cache",
            renderer.typography.cached_glyph_count()
        );
    }
}
