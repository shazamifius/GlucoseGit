//! **Le flou du reste, autour de ce qu'on a choisi** (Q12, fiche 137) — quand il vit, et son
//! réglage. Ce qu'il floute et comment : `renderer/flou.rs`.
//!
//! # La main au repos
//!
//! Le flou vit quand le réglage est allumé et que **la main est au repos** : ni un glisser, ni un
//! redimensionnement, ni un tracé, ni un coude tenu, ni le rectangle de sélection, ni un texte
//! qu'on écrit ou dont on choisit les lettres, ni une lecture. Pendant qu'on déplace ce qu'on a
//! choisi, il faut voir où le poser — le dossier où le ranger compris ; pendant qu'on écrit, ce qui
//! entoure le texte. La caméra, elle, peut bouger : ce qu'on a choisi reste net. Le geste se
//! déduit de l'état de l'application, comme celui de la chronique (`terrain.rs`).
//!
//! # Le réglage
//!
//! *« Je suis d'accord avec le système qui permettrait à l'utilisateur de TOUJOURS CHOISIR »*
//! (15/10) : on choisit des nœuds toute la journée, un flou à chaque clic peut fatiguer. Le menu de
//! la sélection dit « Flouter le reste » ou « Ne plus flouter le reste », la palette aussi ; le
//! choix se retient d'une session à l'autre, à côté de l'échelle de l'interface.

use super::GlucoseApp;
use crate::salissure::Salissure;

/// Le nom du souvenir, à côté de celui de l'échelle de l'interface.
const SOUVENIR: &str = "flou-de-la-selection";

impl GlucoseApp {
    /// **Le flou vit-il ?** Le réglage allumé, la main au repos (voir l'en-tête).
    pub(crate) fn le_flou_vit(&self) -> bool {
        self.ui.flou_de_la_selection
            && !self.is_dragging_item
            && self.resize_session.is_none()
            && self.draw_session.is_none()
            && self.bend_session.is_none()
            && self.ui.placement.trace.is_none()
            && self.selection_box.is_none()
            && self.editing_session.is_none()
            && self.text_drag.is_none()
            && self.ui.lecture.is_none()
    }

    /// **La salissure de cette image, sous le flou** : elle est consommée (remplacée par
    /// `apres`), et rendue entière si elle n'est pas propre et que le flou le demande. Un flou
    /// couvre tout l'écran : l'image qui le pose, celle qui le garde et celle qui le retire se
    /// repeignent entières — une région repeinte seule se peindrait nette dans un écran flouté,
    /// ou flouterait des pixels déjà floutés. Une fois par image : il retient ce qu'il a dit.
    pub(super) fn salissure_sous_le_flou(&mut self, apres: Salissure) -> Salissure {
        let sale = self.salissure.replace(apres);
        let vit = self.le_flou_vit() && self.store.compte_de_la_selection() > 0;
        let entiere = vit || std::mem::replace(&mut self.flou_vivait, vit);
        if entiere && !sale.est_propre() {
            Salissure::Tout
        } else {
            sale
        }
    }

    /// **Flouter le reste autour de la sélection, ou plus** — et le retenir.
    pub(crate) fn choisir_le_flou(&mut self, oui: bool) {
        self.ui.flou_de_la_selection = oui;
        let _ = std::fs::write(
            self.souvenir_de_la_fenetre(SOUVENIR),
            if oui { "oui" } else { "non" },
        );
        self.ui.context_menu_at = None;
        self.mark_dirty();
    }

    /// **Le choix de la session précédente** — oui, sauf s'il a dit non.
    pub(crate) fn flou_retenu(&self) -> bool {
        std::fs::read_to_string(self.souvenir_de_la_fenetre(SOUVENIR))
            .map_or(true, |s| s.trim() != "non")
    }
}

#[cfg(test)]
mod tests;
