//! **LIEN-3 — la fenêtre des liens, à la main** (fiche 102) : l'ouvrir, la regarder, en partir.
//!
//! La fenêtre elle-même — les copies, leur provenance, la disposition de Bring & Go — est au noyau
//! ([`glucose_core::store::fenetre`]). Ici, ce que la main y fait.
//!
//! * **L'ouvrir** : « Montrer ses liens » — `B` (comme *Bring*, le Bring & Go de Moscovich et al.,
//!   CHI 2009), le menu d'un nœud choisi qui a des liens, la palette. Sur une flèche choisie, elle
//!   s'ouvre sur son bout le plus proche du centre de l'écran : celui qu'on regarde.
//! * **La regarder** : zoomer, déplacer la vue — à la molette, au doigt, et en glissant n'importe
//!   où, puisqu'on n'y déplace rien.
//! * **En partir** : un clic sur un nœud — un appui relâché sans bouger, comme un bouton — ferme la
//!   fenêtre et vole vers sa vraie place (le « Go »), `Alt+Z` ramène ; `Maj`+clic la refait autour
//!   de lui (le Bring & Go qui s'enchaîne) ; au doigt et au clic droit, le menu offre les deux ;
//!   `Échap` la ferme.
//! * **Elle est passagère** : tout autre geste — une commande, un clic dans l'interface — la ferme
//!   d'abord. Rien de ce qui change le document ne s'exécute pendant qu'on la regarde.

use crate::app::GlucoseApp;
use crate::commandes::Commande;

/// **Un appui dans la fenêtre, pas encore relâché** : où il est tombé, et ce qu'il visait.
#[derive(Debug, Clone, PartialEq)]
pub struct Appui {
    pub ou: (f64, f64),
    pub vise: Option<String>,
}

impl Commande {
    /// **Cette commande regarde-t-elle la fenêtre des liens**, ou la ferme-t-elle d'abord ?
    pub(crate) fn regarde_la_fenetre(self) -> bool {
        matches!(
            self,
            Commande::Echap
                | Commande::MontrerLesLiens
                | Commande::AllerAuNoeud
                | Commande::Naviguer
                | Commande::Palette
        )
    }
}

impl GlucoseApp {
    /// **« Montrer ses liens »** : la fenêtre du nœud choisi — ou, fenêtre ouverte, refaite autour
    /// de celui qu'on y a choisi. Rend faux si rien de choisi n'a de lien.
    pub(crate) fn montrer_les_liens(&mut self) -> bool {
        let Some(id) = self.noeud_a_montrer() else {
            return false;
        };
        if !self.store.montrer_les_liens(&id) {
            return false;
        }
        self.mark_dirty();
        true
    }

    /// Le nœud dont on montrerait les liens : le seul choisi ; une flèche seule choisie, son bout
    /// le plus proche du centre de l'écran.
    fn noeud_a_montrer(&self) -> Option<String> {
        let mut choisis = self
            .store
            .selected_annotation_ids
            .iter()
            .chain(&self.store.selected_image_ids);
        let (seul, autre) = (choisis.next()?.clone(), choisis.next());
        if autre.is_some() {
            return None;
        }
        let board = self.store.active_board()?;
        let Some(glucose_core::types::Annotation::Arrow {
            source_id,
            target_id,
            ..
        }) = self.store.project.annotation(&board.id, &seul)
        else {
            return Some(seul);
        };
        let (l, h) = self.taille_de_la_fenetre();
        let vp = self.store.viewport();
        let ici = crate::canvas::screen_to_world(f64::from(l) / 2.0, f64::from(h) / 2.0, &vp);
        let pres = |id: &String| {
            let b = glucose_core::arrow::emprise(board, id)?;
            Some((b.left + b.width / 2.0 - ici.0).hypot(b.top + b.height / 2.0 - ici.1))
        };
        [source_id, target_id]
            .into_iter()
            .flatten()
            .filter_map(|id| Some((id.clone(), pres(id)?)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(id, _)| id)
    }

    /// **« Aller à ce nœud »** : la copie choisie dans la fenêtre, à sa vraie place.
    pub(crate) fn aller_au_noeud_montre(&mut self) -> bool {
        let choisi = self
            .store
            .selected_annotation_ids
            .iter()
            .chain(&self.store.selected_image_ids)
            .next()
            .cloned();
        choisi.is_some_and(|id| self.aller_depuis_la_fenetre(&id))
    }

    /// Ferme la fenêtre et vole vers le nœud du document que désigne cette identité de la fenêtre.
    fn aller_depuis_la_fenetre(&mut self, id: &str) -> bool {
        let Some(original) = self
            .store
            .fenetre_des_liens()
            .and_then(|f| f.original(id))
            .map(str::to_string)
        else {
            return false;
        };
        self.store.fermer_la_fenetre_des_liens();
        self.voler_vers_le_noeud(&original);
        self.mark_dirty();
        true
    }

    /// **Ferme la fenêtre, s'il y en a une** : `Échap`, et tout geste qui n'est pas de la regarder.
    pub(crate) fn fermer_la_fenetre_des_liens(&mut self) -> bool {
        self.ui.appui_dans_la_fenetre = None;
        if !self.store.fermer_la_fenetre_des_liens() {
            return false;
        }
        self.mark_dirty();
        true
    }

    /// **Un appui sur le canevas, fenêtre ouverte** : il retient ce qu'il vise, et la main déplace
    /// la vue jusqu'au relâchement. Rend faux sans fenêtre.
    pub(crate) fn appuyer_dans_la_fenetre(&mut self, (wx, wy): (f64, f64)) -> bool {
        if self.store.fenetre_des_liens().is_none() {
            return false;
        }
        // L'index des nœuds doit être celui de la fenêtre avant qu'on le questionne.
        self.renderer.sync_spatial_index(&self.store);
        let vise = self.pick_candidate_at(wx, wy).map(|c| c.id);
        self.ui.appui_dans_la_fenetre = Some(Appui {
            ou: self.mouse_pos,
            vise,
        });
        self.is_panning = true;
        true
    }

    /// **Le relâchement d'un appui dans la fenêtre** : sans avoir bougé, sur un nœud, il y mène —
    /// ou, avec `Maj`, refait la fenêtre autour de lui. Rend vrai s'il a été pris.
    pub(crate) fn relacher_dans_la_fenetre(&mut self) -> bool {
        let Some(appui) = self.ui.appui_dans_la_fenetre.take() else {
            return false;
        };
        let bouge = (appui.ou.0 - self.mouse_pos.0).hypot(appui.ou.1 - self.mouse_pos.1);
        let Some(vise) = appui.vise.filter(|_| bouge <= self.tremblement()) else {
            return false;
        };
        if self.modifiers.shift_key() {
            self.store.montrer_les_liens(&vise);
            self.mark_dirty();
            return true;
        }
        self.aller_depuis_la_fenetre(&vise)
    }

    /// **Le clic droit et l'appui long, fenêtre ouverte, choisissent ce qu'ils visent** — une copie
    /// (sans danger : le document ne la connaît pas) —, pour que le menu offre d'y aller ou d'en
    /// montrer les liens.
    pub(crate) fn choisir_dans_la_fenetre(&mut self) {
        if self.store.fenetre_des_liens().is_none() {
            return;
        }
        self.ui.appui_dans_la_fenetre = None;
        let vp = self.store.viewport();
        let (wx, wy) = crate::canvas::screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        self.store.clear_selection();
        self.renderer.sync_spatial_index(&self.store);
        if let Some(c) = self.pick_candidate_at(wx, wy) {
            self.select_node(&c);
        }
    }
}

#[cfg(test)]
#[path = "fenetre/tests.rs"]
mod tests;
