//! **Le mode référence, à la PureRef** (fiche 51 § 5) : garder sa référence au-dessus de
//! Blender, et rien d'autre à l'écran qu'elle.
//!
//! # Ce que le mode fait
//!
//! * **plus aucune interface** : ni bande, ni onglets, ni minimap, ni panneaux, ni barre
//!   d'action — le canevas prend toute la fenêtre ;
//! * **plus de cadre** : la fenêtre n'a plus de barre de titre ni de bordure ;
//! * **toujours au premier plan** ;
//! * **retenu d'une session à l'autre** : relancé, Glucose reprend le mode où on l'a laissé ;
//! * **défait par le même geste** : `Ctrl+Maj+A`, ou l'entrée du menu.
//!
//! Le menu contextuel et les messages restent : sans eux, on ne saurait plus sortir du mode.
//!
//! # Les gestes sont ceux de PureRef
//!
//! Son manuel les donne ([raccourcis par défaut](https://www.pureref.com/handbook/shortcuts/all-shortcuts/)) :
//! **glisser au bouton droit déplace la fenêtre**, le bouton gauche sur un bord la redimensionne,
//! et `Ctrl+Maj+A` la met au premier plan — c'est donc le geste de tout le mode, celui que la
//! main d'un utilisateur de PureRef connaît déjà. Un clic droit sans bouger ouvre toujours le
//! menu ; le bouton du milieu déplace toujours la vue. `Alt+T`, l'ancien geste caché qui ne
//! faisait que le premier plan (et l'oubliait à la relance), fait désormais tout le mode.
//!
//! # La bordure qui redimensionne
//!
//! Huit pixels logiques : la bordure de redimensionnement que Windows donne à toute fenêtre
//! (`SM_CXSIZEFRAME` + `SM_CXPADDEDBORDER`, quatre et quatre à cent pour cent). Une fenêtre sans
//! cadre se prend donc au même endroit qu'une fenêtre ordinaire.

use crate::app::GlucoseApp;
use winit::window::{ResizeDirection, WindowLevel};

/// Le fichier qui retient le mode, dans le dossier où l'application habite.
const SOUVENIR: &str = "mode-reference";

/// La bordure de redimensionnement, en pixels logiques.
const BORD: f64 = 8.0;

/// Un déplacement de fenêtre au bouton droit : où était le curseur sur l'écran, et la fenêtre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deplacement {
    curseur: (f64, f64),
    fenetre: (i32, i32),
    /// La fenêtre a-t-elle bougé ? Alors le relâchement n'ouvre pas le menu.
    a_bouge: bool,
}

impl GlucoseApp {
    /// `Ctrl+Maj+A`, `Alt+T` et l'entrée du menu.
    pub(crate) fn basculer_le_mode_reference(&mut self) {
        self.poser_le_mode_reference(!self.ui.reference);
        let souvenir = self.souvenir_du_mode_reference();
        // Un souvenir qui ne s'écrit pas coûte la relance, jamais le mode : il ne se dit pas.
        if self.ui.reference {
            let _ = crate::persist::atomic::write_atomic(&souvenir, b"1");
        } else {
            let _ = std::fs::remove_file(&souvenir);
        }
    }

    /// Le mode, appliqué à l'interface et à la fenêtre.
    pub(crate) fn poser_le_mode_reference(&mut self, actif: bool) {
        self.ui.reference = actif;
        self.ui.context_menu_at = None;
        if let Some(fenetre) = &self.window {
            fenetre.set_decorations(!actif);
            fenetre.set_window_level(niveau(actif));
        }
        // La bande part ou revient : toute la vue change de cadre.
        self.mark_dirty();
    }

    /// Le mode était-il actif à la fin de la session précédente ?
    pub(crate) fn mode_reference_retenu(&self) -> bool {
        self.souvenir_du_mode_reference().exists()
    }

    fn souvenir_du_mode_reference(&self) -> std::path::PathBuf {
        self.disque
            .brouillons
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(SOUVENIR)
    }

    /// **Le bouton droit s'enfonce** : en mode référence, un déplacement de fenêtre peut
    /// commencer. Rend `true` s'il a commencé — la vue, alors, ne bouge pas.
    pub(crate) fn commencer_a_deplacer_la_fenetre(&mut self) -> bool {
        if !self.ui.reference {
            return false;
        }
        let Some(fenetre) = &self.window else {
            return false;
        };
        let (Ok(dedans), Ok(dehors)) = (fenetre.inner_position(), fenetre.outer_position()) else {
            return false;
        };
        self.deplacement_de_fenetre = Some(Deplacement {
            curseur: (
                f64::from(dedans.x) + self.mouse_pos.0,
                f64::from(dedans.y) + self.mouse_pos.1,
            ),
            fenetre: (dehors.x, dehors.y),
            a_bouge: false,
        });
        true
    }

    /// Le curseur bouge pendant un déplacement de fenêtre : la fenêtre suit, au pixel.
    ///
    /// Le curseur se lit **sur l'écran** — position de la fenêtre plus position dans la
    /// fenêtre —, parce que la fenêtre bouge sous lui : relu dans la fenêtre, il ne bougerait
    /// presque pas.
    pub(crate) fn deplacer_la_fenetre(&mut self) -> bool {
        let Some(d) = self.deplacement_de_fenetre.as_mut() else {
            return false;
        };
        let Some(fenetre) = &self.window else {
            return true;
        };
        let Ok(dedans) = fenetre.inner_position() else {
            return true;
        };
        let ecran = (
            f64::from(dedans.x) + self.mouse_pos.0,
            f64::from(dedans.y) + self.mouse_pos.1,
        );
        let (dx, dy) = (ecran.0 - d.curseur.0, ecran.1 - d.curseur.1);
        if dx != 0.0 || dy != 0.0 {
            d.a_bouge = true;
            fenetre.set_outer_position(winit::dpi::PhysicalPosition::new(
                d.fenetre.0 + dx.round() as i32,
                d.fenetre.1 + dy.round() as i32,
            ));
        }
        true
    }

    /// Le bouton droit se relâche : le déplacement finit. Rend `true` si la fenêtre a bougé —
    /// le menu, alors, ne s'ouvre pas.
    pub(crate) fn finir_de_deplacer_la_fenetre(&mut self) -> bool {
        self.deplacement_de_fenetre
            .take()
            .is_some_and(|d| d.a_bouge)
    }

    /// Le bord de la fenêtre sous ce point, en mode référence.
    pub(crate) fn bord_sous(&self, (x, y): (f64, f64)) -> Option<ResizeDirection> {
        if !self.ui.reference {
            return None;
        }
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        bord(
            (x, y),
            (f64::from(largeur), f64::from(hauteur)),
            BORD * self.scale_factor,
        )
    }

    /// **Le bouton gauche sur un bord** : la fenêtre se redimensionne, par le système.
    pub(crate) fn redimensionner_par_le_bord(&mut self) -> bool {
        let Some(direction) = self.bord_sous(self.mouse_pos) else {
            return false;
        };
        if let Some(fenetre) = &self.window {
            let _ = fenetre.drag_resize_window(direction);
        }
        true
    }
}

fn niveau(actif: bool) -> WindowLevel {
    if actif {
        WindowLevel::AlwaysOnTop
    } else {
        WindowLevel::Normal
    }
}

/// Le bord d'un rectangle `taille` sous ce point, à `epaisseur` près — coins compris.
pub fn bord(
    (x, y): (f64, f64),
    (largeur, hauteur): (f64, f64),
    epaisseur: f64,
) -> Option<ResizeDirection> {
    let gauche = x < epaisseur;
    let droite = x >= largeur - epaisseur;
    let haut = y < epaisseur;
    let bas = y >= hauteur - epaisseur;
    Some(match (gauche, droite, haut, bas) {
        (true, _, true, _) => ResizeDirection::NorthWest,
        (_, true, true, _) => ResizeDirection::NorthEast,
        (true, _, _, true) => ResizeDirection::SouthWest,
        (_, true, _, true) => ResizeDirection::SouthEast,
        (true, ..) => ResizeDirection::West,
        (_, true, ..) => ResizeDirection::East,
        (_, _, true, _) => ResizeDirection::North,
        (_, _, _, true) => ResizeDirection::South,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
