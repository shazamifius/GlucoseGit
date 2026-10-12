//! **Les champs de nombre du dock, à la main** (CHAMP-1) : l'appui les prend, le glisser
//! règle, le relâcher sans bouger ouvre la frappe, un clic ailleurs la valide. Ce qu'ils
//! font est dans [`crate::dock::champ`] ; ici, l'ordre des couches seulement.

use crate::app::GlucoseApp;
use crate::dock::champ::{champ_sous, Champ, Saisie};
use crate::params::{Pointer, ScreenFrame};
use winit::keyboard::Key;
use winit::window::CursorIcon;

impl GlucoseApp {
    /// Le champ sous le pointeur, s'il y en a un.
    fn champ_sous_le_pointeur(&self, screen: ScreenFrame) -> Option<Champ> {
        let typo = &self.renderer.typography;
        champ_sous(&self.dock_manager, typo, screen, self.pointer()).map(|(c, _)| c)
    }

    /// Le champ qu'on tape, s'il y en a un.
    fn champ_en_frappe(&self) -> Option<Champ> {
        match &self.dock_manager.champ {
            Some(Saisie::Frappe { champ, .. }) => Some(*champ),
            _ => None,
        }
    }

    /// **Un appui ailleurs que dans le champ qu'on tape le valide** — comme l'onglet qu'on
    /// renomme, et le champ de Glucose Tauri qui validait en perdant le focus.
    pub(crate) fn valider_le_champ_quitte(&mut self, screen: ScreenFrame) {
        let Some(tape) = self.champ_en_frappe() else {
            return;
        };
        if self.champ_sous_le_pointeur(screen) != Some(tape) {
            self.dock_manager.valider_le_champ();
            self.mark_dirty();
        }
    }

    /// **L'appui prend le champ sous lui** ; dans le champ qu'on tape, il ne fait rien.
    pub(crate) fn prendre_un_champ(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let typo = &self.renderer.typography;
        let Some((champ, frame)) = champ_sous(&self.dock_manager, typo, screen, pointer) else {
            return false;
        };
        if self.champ_en_frappe() != Some(champ) {
            let jeu = self.tremblement() as f32;
            self.dock_manager
                .tenir_le_champ(champ, frame, (pointer.x, jeu));
        }
        self.mark_dirty();
        true
    }

    /// Le champ tenu suit la main.
    pub(crate) fn glisser_le_champ(&mut self, x: f32) -> bool {
        let tenu = self.dock_manager.glisser_le_champ(x);
        if tenu {
            self.mark_dirty();
        }
        tenu
    }

    /// Le champ tenu est lâché : rend vrai s'il y en avait un.
    pub(crate) fn lacher_le_champ(&mut self) -> bool {
        let tenu = self.dock_manager.lacher_le_champ();
        if tenu {
            self.update_cursor();
            self.mark_dirty();
        }
        tenu
    }

    /// Une touche pour le champ qu'on tape.
    pub(crate) fn frapper_dans_le_champ(&mut self, key: &Key) -> bool {
        let pris = self.dock_manager.frapper_le_champ(key);
        if pris {
            self.mark_dirty();
        }
        pris
    }

    /// **Le curseur d'un champ** : `↔` sur un champ qu'on peut glisser, et tant qu'on le tient ;
    /// le curseur de texte sur celui qu'on tape.
    pub(crate) fn curseur_du_champ(&self) -> Option<CursorIcon> {
        if let Some(Saisie::Tenu { .. }) = self.dock_manager.champ {
            return Some(CursorIcon::EwResize);
        }
        let (w, h) = self.taille_de_la_fenetre();
        let sous = self.champ_sous_le_pointeur(self.screen_frame(w, h))?;
        Some(if self.champ_en_frappe() == Some(sous) {
            CursorIcon::Text
        } else {
            CursorIcon::EwResize
        })
    }
}

#[cfg(test)]
mod tests;
