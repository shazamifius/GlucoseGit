//! **DEFILE-1 — le canevas défile quand on tient un nœud près du bord** (fiche 59).
//!
//! Sa demande, le 09/10 : glisser une image pour la poser loin, hors de ce que l'écran montre,
//! et que le canevas suive le mouvement — « comme dans les suites (Word) : le canevas fait
//! dérouler les choses ». Il pensait à la boucle de Blender, et a vu lui-même qu'elle
//! empêcherait de passer d'un Glucose à l'autre : on ne pourrait plus quitter la fenêtre.
//!
//! # Pourquoi une bande, et pas la poussée contre le bord
//!
//! La recherche donnait une forme plus juste sur le papier : *Push-Edge* (Malacria, Aceituno,
//! Casiez, Roussel, CHI 2015), où la vue avance de ce que la main **pousse** contre le bord de
//! l'écran — jusqu'à 13 % plus rapide que le défilement classique, à la souris. Elle ne tient
//! pas chez lui, pour trois raisons lues dans son usage et dans le système :
//!
//! * il glisse surtout **au pavé**, bouton tenu : sans pouvoir lever le doigt en route, la
//!   poussée ne mène qu'à la longueur d'un passage du doigt ;
//! * sous Windows, le bord du bas d'une fenêtre plein écran est la **barre des tâches**, pas le
//!   bord de l'écran : le curseur y sort, et le glisser partirait vers une autre fenêtre ;
//! * « dérouler », son mot, c'est un défilement qui **continue** tant qu'on tient le nœud là.
//!
//! D'où la forme de Word, de l'explorateur, de Figma et de tldraw : une **bande** au bord du
//! canevas, où la vue défile d'autant plus vite qu'on s'y enfonce. Elle sert la souris, le pavé
//! et le doigt par le même chemin ; et sortir de la fenêtre fait toujours partir le glisser vers
//! une autre (`interactions::mouse`, fiche 51 § 2).
//!
//! # Ce qui se choisit, et ce qui ne se choisit pas
//!
//! La largeur de la bande ne se choisit pas : c'est la cible d'un doigt
//! ([`crate::theme::CIBLE_DU_DOIGT`]), la mesure qu'un doigt sait viser — et qui suffit à une
//! souris. La vitesse, elle, est un **nombre de ressenti**, à juger à son écran : au plus profond
//! de la bande, la vue parcourt **une étendue du canevas par seconde**, et la vitesse croît comme
//! le **carré** de l'enfoncement — l'entrée de la bande reste douce, et poser un nœud près du
//! bord ne fait presque rien bouger.
//!
//! Journal des valeurs essayées : 09/10, une étendue par seconde au carré (première valeur).

use crate::app::GlucoseApp;
use crate::canvas::{screen_to_world, world_to_screen};
use glucose_core::types::Viewport;

/// Le canevas à l'écran, en pixels : là où la bande se mesure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cadre {
    pub gauche: f64,
    pub haut: f64,
    pub droite: f64,
    pub bas: f64,
}

/// **La vitesse du défilement**, en pixels d'écran par seconde, pour un point tenu dans ce
/// cadre : nulle hors de la bande, croissante avec l'enfoncement. Positive, le contenu glisse
/// vers la droite ou le bas — ce qui est au-delà du bord gauche ou du haut entre dans la vue.
pub fn vitesse(point: (f64, f64), cadre: Cadre, bande: f64) -> (f64, f64) {
    (
        sur_un_axe(point.0, (cadre.gauche, cadre.droite), bande),
        sur_un_axe(point.1, (cadre.haut, cadre.bas), bande),
    )
}

/// La vitesse sur un axe : l'enfoncement dans la bande du début moins celui dans la bande de la
/// fin, au carré, fois l'étendue. Un cadre plus étroit que deux bandes les partage en deux.
fn sur_un_axe(x: f64, (debut, fin): (f64, f64), bande: f64) -> f64 {
    let etendue = fin - debut;
    let bande = bande.min(etendue / 2.0);
    if bande <= 0.0 {
        return 0.0;
    }
    let vers_le_debut = ((debut + bande - x) / bande).clamp(0.0, 1.0);
    let vers_la_fin = ((x - (fin - bande)) / bande).clamp(0.0, 1.0);
    (vers_le_debut.powi(2) - vers_la_fin.powi(2)) * etendue
}

impl GlucoseApp {
    /// Un geste qui suit le curseur est-il en cours : glisser des nœuds, tracer un rectangle de
    /// sélection, redimensionner ?
    fn un_geste_suit_le_curseur(&self) -> bool {
        self.is_dragging_item || self.selection_box.is_some() || self.resize_session.is_some()
    }

    /// **La vitesse du bord, maintenant** : nulle sans geste qui suive le curseur. Le cadre est
    /// le canevas — sous la bande du haut, au-dessus de ce qui recouvre le bas (BORD-1 du
    /// téléphone : la navigation, le clavier).
    pub(crate) fn vitesse_au_bord(&self, (largeur, hauteur): (f32, f32)) -> (f64, f64) {
        if !self.un_geste_suit_le_curseur() {
            return (0.0, 0.0);
        }
        let (_, bas) = self.ui.ecran_visible((largeur, hauteur));
        let cadre = Cadre {
            gauche: 0.0,
            haut: f64::from(self.ui.header_height()),
            droite: f64::from(largeur),
            bas: f64::from(bas),
        };
        let bande = f64::from(crate::theme::CIBLE_DU_DOIGT) * self.densite();
        vitesse(self.mouse_pos, cadre, bande)
    }

    /// **Le bord fait avancer la vue** de ce que ce pas d'image demande : par la porte de la
    /// souris, qui montre tout à l'image même — rien ne glisse après.
    pub(crate) fn defiler_au_bord(&mut self, taille: (f32, f32), pas: f64) {
        let (vx, vy) = self.vitesse_au_bord(taille);
        if (vx, vy) != (0.0, 0.0) {
            self.elan.placer_pan(vx * pas, vy * pas);
        }
    }

    /// **Ce que la main tient reste sous elle quand la vue bouge** : les nœuds glissés, le coin
    /// tiré, l'autre coin du rectangle de sélection, qui reste où il était dans le monde. Vrai
    /// pour le bord, et pour toute autre source — une molette, un pavé pendant un glisser.
    pub(crate) fn les_gestes_suivent_la_camera(&mut self, avant: Viewport) {
        let (x, y) = self.mouse_pos;
        if self.is_dragging_item {
            self.handle_item_drag_move(x, y);
        }
        if self.resize_session.is_some() {
            self.handle_resize_move(x, y);
        }
        if let Some((x1, y1, _, _)) = self.selection_box {
            let (mx, my) = screen_to_world(x1, y1, &avant);
            let (nx, ny) = world_to_screen(mx, my, &self.store.viewport());
            self.selection_box = Some((nx, ny, x, y));
            self.mark_dirty();
        }
    }
}

#[cfg(test)]
mod tests;
