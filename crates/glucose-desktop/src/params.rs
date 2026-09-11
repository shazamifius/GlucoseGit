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
use glucose_core::smart_align::SnapGuides;
use glucose_core::types::Viewport;
use std::collections::HashSet;

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
pub struct SceneOverlay<'a> {
    pub guides: &'a SnapGuides,
    /// `(x0, y0, x1, y1)` en coordonnées monde, `None` hors sélection rectangle.
    pub selection_box: Option<(f64, f64, f64, f64)>,
    pub editing: Option<&'a TextEditSession>,
}

/// Tranche visible en cours de dessin : ce que la requête spatiale a retenu
/// (loi L1) et le repère nécessaire pour la projeter à l'écran.
#[derive(Clone, Copy)]
pub struct ViewPass<'a> {
    pub vp: Viewport,
    /// Identifiants retournés par la requête spatiale — rien d'autre ne se dessine.
    pub visible_ids: &'a HashSet<&'a str>,
    /// Hauteur du bandeau : sommet de la zone de canevas, en unités logiques.
    pub header_h: f32,
}
