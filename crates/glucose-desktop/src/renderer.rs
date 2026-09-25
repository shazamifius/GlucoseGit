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

pub mod apercu;
pub mod arrivage;
mod arrondi;
pub mod arrow;
pub mod arrow_label;
pub mod atelier;
pub mod cadrage;
pub mod card;
pub mod composants;
pub mod domain;
mod fils;
pub mod focus;
pub mod folder;
pub mod grille;
pub mod halo;
pub mod handles;
pub mod hue;
pub mod magasin;
pub mod math;
pub mod note;
pub mod pass;
pub mod passages;
pub mod photo;
pub mod predicate;
pub mod richtext;
pub mod scale;
pub mod scene;
pub mod tuiles;
pub mod vignette;
pub mod wrap;

use crate::params::{Pointer, SceneOverlay, ViewPass};
use crate::theme::Theme;
use crate::typography::Typography;
use crate::ui::{render_ui, UiState};
pub(crate) use arrondi::push_rounded_rect;
use domain::DomainTints;
use glucose_core::quadtree::SpatialHash;
use glucose_core::store::Store;
use glucose_core::text::Selection;
use hue::SymbioticHueCache;
use tiny_skia::PixmapMut;

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
    /// Quand la phase du clignotement a commencé.
    ///
    /// Sert à **décider** de la phase et à caler le réveil ([`crate::app`]) ; le rendu, lui,
    /// ne la lit pas — voir [`TextEditSession::curseur_visible`].
    pub blink_timer: std::time::Instant,
    /// Le curseur est-il dans sa demi-seconde allumée ?
    ///
    /// # BLINK-1 — un rastériseur qui lit l'horloge n'est pas reproductible
    ///
    /// La visibilité se déduisait de `blink_timer.elapsed()` **dans le dessin**, à deux
    /// endroits. Deux conséquences, et la seconde est la vraie :
    ///
    /// * le test d'aspect des cartes échouait au hasard depuis trois sessions — il posait un
    ///   `blink_timer` en phase éteinte, et le second de ses deux rendus arrivait parfois une
    ///   demi-seconde plus tard, curseur rallumé. Il est nommé « instable » dans la fiche 17
    ///   § 5 depuis, sans que la cause ait été cherchée ;
    /// * plus grave : **le même état rendait deux images différentes**. La fiche 05 § 4.4 le
    ///   dit — le renderer *lit*, il ne calcule pas. Une horloge dans une passe de dessin
    ///   rend toute épreuve d'aspect non reproductible, et interdit de comparer deux voies.
    ///
    /// La phase se décide donc là où le temps avance — la boucle de réveil, qui la calculait
    /// **déjà** pour savoir quand se réveiller — et le dessin ne fait plus que la lire.
    pub curseur_visible: bool,
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

/// Ce avec quoi une passe peint, et qui ne change pas de la frame : la police, la table des
/// teintes de domaine et le thème. Un seul paramètre au lieu de trois (R-44).
#[derive(Clone, Copy)]
pub struct PaintKit<'a> {
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
    /// Les tuiles deja peintes, memoisees par empreinte (TUILE-1, voir [`grille`]).
    ///
    /// C'est ce qui remplace le cache de vignettes pour tout ce qui bouge : ancre au monde et
    /// non a l'ecran, un deplacement de la vue ne l'invalide pas.
    pub tuiles: tuiles::Tuiles,
    /// Ce que l'écran porte en tuiles, relevé une fois par image et lu deux fois : avant le
    /// fond, pour savoir s'il se verra, et pendant la pose. Gardé ici pour n'allouer qu'une
    /// fois (fiche 05 § 4.3).
    couverture: grille::Couverture,
    pub spatial_hash: SpatialHash,
    pub spatial_version: u64,
    pub active_board_id: String,
    /// Ce que la carte laisse aux photos, et le cran qu'on en a déduit (ETAGES-3).
    pub carte: voies::cran::EtatDeLaCarte,
    /// Le mode Focus d'une membrane : ce qui se voit, et le fond (MEMB-2).
    pub focus: focus::FocusDuRendu,
    /// Ce que le geste en cours a touché, que l'index ne connaîtra qu'à sa fin (GESTE-1).
    suivi_du_geste: glucose_core::quadtree::SuiviDuGeste,
    /// Combien de pixels physiques font un pixel logique sur l'écran de cette image (DPI-1) :
    /// l'échelle de l'interface, relue à chaque image — une fenêtre qui change d'écran change
    /// de densité.
    densite: f32,
}

/// `new` ne prend aucun argument : `Default` est donc exactement le même constructeur.
/// Le déclarer évite qu'un appelant générique ait à connaître le nom `new`.
impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

pub use cadrage::{Cadrage, Couche, Regard, SceneReduite};

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
            tuiles: tuiles::Tuiles::nouveau(),
            couverture: grille::Couverture::default(),
            spatial_hash: SpatialHash::new(1000.0),
            spatial_version: 0,
            active_board_id: String::new(),
            carte: Default::default(),
            focus: Default::default(),
            suivi_du_geste: Default::default(),
            densite: 1.0,
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
    fn synchroniser_les_caches(&mut self, store: &Store, ecran: (u32, u32)) {
        // Ce que les fils de fond ont fini entre deux images entre dans les caches ici, et
        // nulle part ailleurs : le rendu voit ensuite un cache qui ne bouge pas sous ses
        // pieds. Une récolte est une remise d'accord comme les trois autres, et c'est bien
        // ici qu'elle appartient.
        self.magasin.recolter();
        self.magasin.regler_la_vue_d_ensemble(store, ecran);
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
        regard: Regard,
    ) {
        self.magasin.ouvrir();
        self.synchroniser_les_caches(store, (pixmap.width(), pixmap.height()));
        let debut = std::time::Instant::now();
        self.rendre_la_scene(pixmap, store, ui, overlay, ui.header_height(), regard);
        noter_le_cout_de_la_scene(debut);

        // 9. Interface utilisateur complete (TopBar, Tabs, Minimap, Toasts)
        render_ui(pixmap, store, ui, &self.typography, &self.theme, pointer);
        self.magasin.fermer();
    }

    /// La scene rendue **plus petite que la fenetre**, puis agrandie, puis l'interface nette.
    ///
    /// # Ce que cela achete, et ce que cela coute
    ///
    /// Les passes qui couvrent l'ecran -- le fond, la grille, les halos, l'aura et le cadre
    /// d'une carte zoomee -- coutent proportionnellement au nombre de pixels qu'elles
    /// ecrivent. Les rendre dans une image `f` fois plus petite coute donc `f²` fois moins, et
    /// c'est la seule facon connue de borner **toutes** les passes a la fois : aucune
    /// optimisation passe par passe ne repond a un cout qui vient de la surface.
    ///
    /// L'agrandissement passe par REPORT-1 au plus proche voisin, la primitive la moins chere
    /// du programme -- un texel lu, aucun melange. C'est la pixelisation assumee de la charte.
    ///
    /// **L'interface, elle, reste nette.** Elle ne suit pas la vue, elle ne coute pas la
    /// surface de l'ecran, et une barre d'outils floue se remarque bien plus qu'un canevas
    /// grossier pendant un geste.
    pub fn rendre_reduit(
        &mut self,
        plein: &mut PixmapMut,
        scene: SceneReduite<'_>,
        store: &Store,
        ui: &mut UiState,
        overlay: SceneOverlay<'_>,
        pointer: Pointer,
    ) {
        self.magasin.ouvrir();
        self.synchroniser_les_caches(store, (plein.width(), plein.height()));
        let f = scene.facteur.max(1);
        let debut = std::time::Instant::now();
        self.rendre_la_region(
            &mut scene.tampon.as_mut(),
            store,
            ui,
            overlay,
            ui.header_height() / f as f32,
            Cadrage::reduit(f),
        );
        noter_le_cout_de_la_scene(debut);
        agrandir(plein, scene.tampon, f);
        crate::perf::stage("agrandir");

        render_ui(plein, store, ui, &self.typography, &self.theme, pointer);
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
        regard: Regard,
    ) {
        let cadrage = Cadrage::plein().sous_le_regard(regard);
        self.rendre_la_region(pixmap, store, ui, overlay, header_h, cadrage);
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
    /// Ce qu'une passe de dessin lit sans le modifier : la typographie, les formules, les
    /// teintes, le thème.
    ///
    /// Publique parce que la voie graphique rend une carte **hors contexte** — à la demande,
    /// quand sa texture manque — et n'a alors que le moteur sous la main.
    pub fn kit(&self) -> PaintKit<'_> {
        PaintKit {
            typography: &self.typography,
            math: &self.math,
            tints: &self.domain_tints,
            theme: &self.theme,
        }
    }

    /// Rend **vrai** si cette couche a reçu de l'encre — la seule question dont dépend le
    /// téléversement de la couche du dessous (voir [`voies::Confie`]).
    pub fn rendre_la_region(
        &mut self,
        pixmap: &mut PixmapMut,
        store: &Store,
        ui: &UiState,
        overlay: SceneOverlay<'_>,
        header_h: f32,
        cadrage: Cadrage,
    ) -> bool {
        let width = pixmap.width();
        let height = pixmap.height();
        self.densite = ui.scale();
        let (vp, rangs, densite) = self.cadrer(store, (width, height), header_h, cadrage);
        let pass = ViewPass {
            vp,
            visibles: &rangs,
            index: &self.spatial_hash,
            header_h,
            densite,
        };
        // Par champs et non par `self.kit()` : les passes qui suivent empruntent d'autres
        // champs en ecriture, et un emprunt disjoint ne se prouve qu'a travers des champs.
        let kit = PaintKit {
            typography: &self.typography,
            math: &self.math,
            tints: &self.domain_tints,
            theme: &self.theme,
        };

        let mut encre = false;
        if cadrage.couche.porte_le_dessous() {
            encre = dessiner_sous_les_photos(
                grille::Fond {
                    couverture: &mut self.couverture,
                    tuiles: &self.tuiles,
                    theme: &self.theme,
                },
                &mut self.hue_cache,
                pixmap,
                (store, pass, kit),
                (cadrage, overlay.designees),
                header_h,
            );
        }

        if cadrage.couche.porte_les_photos() {
            poser_les_photos(
                grille::Atelier {
                    store,
                    magasin: &mut self.magasin,
                    cout: &mut self.cout,
                    tuiles: &mut self.tuiles,
                    index: &self.spatial_hash,
                    kit,
                },
                pixmap,
                (store, pass),
                (cadrage, &self.couverture),
            );
        }

        if cadrage.couche.porte_le_dessus() {
            dessiner_sur_les_photos(
                &mut self.hue_cache,
                pixmap,
                (store, pass, kit),
                (ui, overlay),
                cadrage,
            );
            // Le dessus porte toujours la chrome, donc toujours de l'encre.
            encre = true;
        }
        encre
    }
}

/// Pose les photos sur la voie **processeur** : par la grille de tuiles quand la vue le
/// permet, en direct sinon.
///
/// Une fonction **libre**, et ce n'est pas un choix de style : l'atelier emprunte quatre
/// champs du moteur à la fois, dont trois en écriture, et le compilateur autorise des
/// emprunts disjoints sur des champs distincts, jamais à travers `&mut self`. Le rendu se
/// prête en pièces — plusieurs méthodes ont déjà dû descendre ici pour cette seule raison.
fn poser_les_photos(
    atelier: grille::Atelier<'_>,
    pixmap: &mut PixmapMut,
    (store, pass): (&Store, ViewPass<'_>),
    (cadrage, couverture): (Cadrage, &grille::Couverture),
) {
    let mut atelier = atelier;
    grille::poser_les_images(&mut atelier, pixmap, store, pass, cadrage, couverture);
    crate::perf::stage("images");
}

/// Ce que la scene a coute, elle seule, en microsecondes.
///
/// Mesure a part parce que c'est la **seule** part du temps d'une image qui suive la surface :
/// l'interface, les panneaux, l'agrandissement et le televersement coutent ce qu'ils coutent,
/// que la scene soit grande ou petite. Les confondre a deja conduit a rapetisser une scene qui
/// ne coutait rien, jusqu'a l'illisible, sans rien gagner (voir [`crate::resolution`]).
fn noter_le_cout_de_la_scene(debut: std::time::Instant) {
    crate::perf::compteur("img_scene_us", debut.elapsed().as_micros() as f64);
}

/// Etale la scene reduite sur toute la fenetre, au plus proche voisin.
///
/// Le facteur est un entier, donc chaque texel devient un carre exact de `f x f` pixels :
/// aucun reechantillonnage, aucun texel invente, et le resultat est reproductible au bit
/// pres. C'est ce qui permet de dire que la seule chose perdue est la finesse, et rien d'autre.
fn agrandir(plein: &mut PixmapMut, scene: &tiny_skia::Pixmap, f: u32) {
    let (largeur, hauteur) = (plein.width(), plein.height());
    let (texels, _) = scene.data().as_chunks::<4>();
    let Some(vue) = glucose_core::report::Vue::nouvelle(texels, scene.width(), scene.height())
    else {
        return;
    };
    let (pixels, _) = plein.data_mut().as_chunks_mut::<4>();
    let Some(mut cible) = glucose_core::report::VueMut::nouvelle(pixels, largeur, hauteur) else {
        return;
    };
    let pose = glucose_core::report::Pose {
        x: 0.0,
        y: 0.0,
        largeur: (scene.width() * f) as f32,
        hauteur: (scene.height() * f) as f32,
    };
    let clip = glucose_core::occlusion::Boite::nouvelle(0.0, 0.0, largeur as f32, hauteur as f32);
    glucose_core::report::reporter(
        &mut cible,
        &vue,
        pose,
        clip,
        // `Remplacer` : la scene reduite EST l'image, elle ne se compose sur rien.
        glucose_core::report::Melange::Remplacer,
        glucose_core::report::Filtre::PlusProche,
    );
}

pub mod voies;

pub use voies::Confie;
use voies::{dessiner_sous_les_photos, dessiner_sur_les_photos};

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
        let overlay = SceneOverlay::sans_rien(&guides);
        let origin = Pointer { x: 0.0, y: 0.0 };
        renderer.render(
            &mut view,
            store,
            ui,
            overlay,
            origin,
            crate::renderer::Regard::immobile(),
        );
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
