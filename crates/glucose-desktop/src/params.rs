//! Structures de paramètres nommés partagées par le rendu et l'interface.
//!
//! **R-44 — pourquoi ce module existe.** `render_docks` a été appelée un jour
//! avec `scale` et `mouse_x` intervertis. Les deux étant des `f32` voisins dans
//! une liste de onze paramètres, le compilateur n'a rien pu dire : `scale` a
//! reçu 170, les polices ont été rastérisées à 2 040 px, la frame est montée à
//! 10 697 ms et la fenêtre est restée gelée au démarrage. `clippy::too_many_arguments`
//! l'aurait signalée, mais le lint était désactivé au niveau du crate.
//!
//! La parade n'est pas de découper les fonctions au hasard : c'est de donner un
//! **type nommé** aux scalaires qui vont ensemble, et de les construire avec
//! leurs champs explicites au point d'appel. Passer une position là où on
//! attend une échelle redevient alors une erreur de compilation.
//!
//! Corollaire à respecter : ces structures ne portent **aucune** méthode de
//! calcul. Elles nomment des paramètres, elles ne font rien (§ 2.2).

use crate::renderer::TextEditSession;
use glucose_core::quadtree::SpatialHash;
use glucose_core::smart_align::SnapGuides;
use glucose_core::types::Viewport;

/// Position du curseur, en unités logiques d'écran (§ 5.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pointer {
    pub x: f32,
    pub y: f32,
}

/// Cadre de la fenêtre : dimensions utiles, hauteur du bandeau, échelle UI.
///
/// `scale` voisine ici trois longueurs, mais il est nommé : c'est précisément
/// ce qui manquait à la signature de `render_docks`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenFrame {
    pub width: f32,
    pub height: f32,
    pub header_h: f32,
    pub scale: f32,
}

/// Rectangle en unités logiques accompagné de l'échelle UI qui le gouverne.
///
/// Utilisé pour le contenu d'un panneau de dock et pour les boutons de la
/// barre d'outils : mêmes cinq scalaires, même risque d'inversion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaledRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub scale: f32,
}

/// Où l'on pose du texte : sa **ligne de base** à l'écran, et son corps en pixels.
///
/// Trois `f32` de plus qui se ressemblent — une abscisse, une ordonnée, une taille — et que
/// rien ne distinguerait dans une liste de paramètres. Nommés, les intervertir ne compile plus.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pen {
    pub x: f32,
    pub y: f32,
    pub font_size: f32,
}

/// État visuel d'un bouton. Deux `bool` consécutifs ne se distinguent pas
/// davantage que deux `f32` : ils sont nommés eux aussi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonState {
    pub active: bool,
    pub hover: bool,
}

/// Éléments transitoires superposés à la scène pour une frame : guides
/// d'alignement, rectangle de sélection en cours, session d'édition de texte.
///
/// Ils ne font pas partie du document : ils vivent le temps d'un geste.
///
/// `Copy` parce que tous ses champs le sont, et parce qu'une scene qui se rend en DEUX
/// couches doit le donner deux fois (voir `Renderer::rendre_les_couches`).
#[derive(Clone, Copy)]
pub struct SceneOverlay<'a> {
    pub guides: &'a SnapGuides,
    /// `(x0, y0, x1, y1)` en coordonnées monde, `None` hors sélection rectangle.
    pub selection_box: Option<(f64, f64, f64, f64)>,
    pub editing: Option<&'a TextEditSession>,
    /// Les images en train d'arriver d'un dépôt web, là où elles se poseront.
    pub arrivages: &'a [Arrivage],
    /// Les passages de cartes à faire briller : ce qu'une flèche survolée désigne, ou ce que
    /// l'éditeur d'ancres sélectionne (FLECHE-4).
    pub eclairages: &'a [Eclairage],
    /// **Les cartes désignées, et leur vivacité** : la cible qu'une flèche en train de naître
    /// vise, les bouts d'une flèche survolée qui ne désigne pas de passage. Leur lueur s'avive —
    /// l'indice qui dit à quoi une flèche va se lier (LUEUR-1, l'`isHighlightBox` de Tauri) —,
    /// en glissant de l'un à l'autre (LUEUR-2).
    pub designees: &'a [(String, f32)],
    /// Sous un outil de création armé, l'élément à naître là où il se poserait, en unités
    /// monde (PLACEMENT-1).
    pub fantome: Option<glucose_core::geometry::Rect>,
}

impl<'a> SceneOverlay<'a> {
    /// **La scène seule, rien par-dessus** : ce que rendent un banc, un témoin, une épreuve.
    ///
    /// Trente et un littéraux recopiaient les mêmes cinq champs vides ; un champ de plus les
    /// aurait tous fait changer.
    pub fn sans_rien(guides: &'a SnapGuides) -> Self {
        Self {
            guides,
            selection_box: None,
            editing: None,
            arrivages: &[],
            eclairages: &[],
            designees: &[],
            fantome: None,
        }
    }
}

/// **Un passage à faire briller** dans une carte de texte (FLECHE-4).
#[derive(Debug, Clone, PartialEq)]
pub struct Eclairage {
    /// La carte.
    pub carte: String,
    /// Ses plages, en octets de sa source.
    pub plages: Vec<(usize, usize)>,
    /// Sa couleur — celle de la carte, si `None`.
    pub teinte: Option<(u8, u8, u8)>,
}

/// **Une image qui arrive** : un dépôt web annoncé, pas encore livré.
///
/// Elle vit hors du document — rien n'est posé tant que rien n'est arrivé, et un `Ctrl+Z`
/// n'a rien à défaire —, mais elle a déjà sa place dans le monde : celle où le curseur a
/// lâché, figée à l'annonce. La vue peut bouger pendant la seconde d'attente ; l'image
/// arrivera quand même là où on l'a déposée.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrivage {
    /// Le numéro de l'annonce, que la livraison porte aussi.
    pub numero: u64,
    /// Le point du monde où l'image se posera.
    pub monde: (f64, f64),
    /// Le site d'où elle vient : ce que le marqueur dit, et rien qu'il ne sache pas.
    pub hote: String,
}

/// Tranche visible en cours de dessin : ce que la requête spatiale a retenu
/// (loi L1) et le repère nécessaire pour la projeter à l'écran.
#[derive(Clone, Copy)]
pub struct ViewPass<'a> {
    pub vp: Viewport,
    /// Ce que la requête spatiale a retenu, par tranches — rien d'autre ne se dessine.
    ///
    /// Une passe y va droit, au lieu de filtrer le document entier. Le coût cesse alors de
    /// dépendre de la taille du document pour ne plus dépendre que de ce qu'on regarde
    /// (CULL-1). L'ensemble de noms qui vivait ici auparavant obligeait chaque passe à faire
    /// l'inverse du culling : parcourir tout le tableau pour demander de chaque nœud s'il
    /// était dedans.
    pub visibles: &'a [u32],
    /// L'index spatial d'où sortent ces rangs, synchronisé sur le tableau actif.
    ///
    /// Le culling n'en a pas besoin — il a déjà sa réponse — mais le calcul d'une teinte, si :
    /// elle dépend des cartes voisines, et les chercher dans le tableau coûterait un parcours
    /// par carte (HALO-4).
    pub index: &'a SpatialHash,
    /// Hauteur du bandeau : sommet de la zone de canevas, en unités logiques.
    pub header_h: f32,
    /// Combien de pixels du tampon font un pixel logique (DPI-1) : la densité de l'écran,
    /// divisée par la réduction quand la scène se rend plus petite que la fenêtre.
    pub densite: f32,
}

impl ViewPass<'_> {
    /// **L'échelle de cette passe** : le zoom de sa vue, et la densité de son tampon.
    pub fn echelle(&self) -> crate::renderer::scale::WorldScale {
        crate::renderer::scale::WorldScale::new(self.vp.scale, self.densite)
    }
}
