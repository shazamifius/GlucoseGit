//! **Prendre une zone** (PRESET-1, fiche 62) : la choisir, la glisser, la redimensionner, la
//! retirer.
//!
//! # Par son bandeau, et non par sa surface
//!
//! Dans Glucose Tauri, une zone prenait tout clic sur son vide : un premier la choisissait, un
//! second la glissait — et sous un gabarit qui couvre le tableau, on ne pouvait plus tracer de
//! rectangle de sélection. Figma fait de même avec ses sections, et c'est l'une des plaintes
//! les plus demandées de son forum : qu'une section ne se prenne que par son titre. Ici, une
//! zone se prend **par son bandeau** — on la choisit et on la glisse d'un même geste — ;
//! choisie, elle montre ses quatre coins et sa croix. Son intérieur reste le canevas.
//!
//! Les nœuds passent devant : un bandeau couvert par une photo laisse la photo se prendre. Un
//! geste est une entrée du journal, et la zone suit la main pendant le geste.

use crate::app::GlucoseApp;
use crate::canvas::{screen_to_world, world_to_screen};
use glucose_core::resize::{resize_rect, Handle, ResizeRule};
use glucose_core::smart_align::AlignRect;

/// Comment la zone tenue a été prise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prise {
    Bandeau,
    Coin(Handle),
}

/// La zone tenue : laquelle, par où, sa boîte et le point du monde au départ du geste.
#[derive(Debug, Clone, PartialEq)]
pub struct Tenue {
    zone: String,
    prise: Prise,
    depart: AlignRect,
    pointeur: (f64, f64),
}

/// Ce que l'interface sait des zones : celle qui est choisie, celle qui est tenue.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ZonesUi {
    pub choisie: Option<String>,
    pub tenue: Option<Tenue>,
}

/// La boîte d'une zone, dans le monde.
fn boite(z: &glucose_core::types::BoardZone) -> AlignRect {
    AlignRect::new(z.x, z.y, z.width, z.height)
}

impl GlucoseApp {
    /// Le point du monde sous le curseur.
    fn sous_le_curseur(&self) -> (f64, f64) {
        screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport())
    }

    /// **Les prises de la zone choisie** — sa croix, ses coins — passent avant les nœuds, comme
    /// les poignées d'un nœud choisi.
    pub(crate) fn prendre_la_zone_choisie(&mut self) -> bool {
        let Some(id) = self.ui.zones.choisie.clone() else {
            return false;
        };
        let board = self.store.project.active_board_id.clone();
        let Some(zone) = self
            .store
            .zones(&board)
            .iter()
            .find(|z| z.slot_id == id)
            .cloned()
        else {
            self.ui.zones.choisie = None;
            return false;
        };
        let vp = self.store.viewport();
        let (sx, sy) = world_to_screen(zone.x, zone.y, &vp);
        let ecran = (
            sx as f32,
            sy as f32,
            (zone.width * vp.scale) as f32,
            (zone.height * vp.scale) as f32,
        );
        let (cx, cy, cw, ch) = crate::renderer::zone::croix(ecran, self.ui.scale());
        let (px, py) = (self.mouse_pos.0 as f32, self.mouse_pos.1 as f32);
        if (cx..=cx + cw).contains(&px) && (cy..=cy + ch).contains(&py) {
            self.store.retirer_la_zone(&board, &id);
            self.ui.zones.choisie = None;
            self.mark_dirty();
            return true;
        }
        let point = self.sous_le_curseur();
        let Some(prise) =
            glucose_core::hit_priority::handle_slop_world(vp.scale, zone.width, zone.height)
        else {
            return false;
        };
        let coin = Handle::ALL.into_iter().filter(|h| h.is_corner()).find(|h| {
            let (x, y) = h.position_on(boite(&zone));
            (point.0 - x).abs() <= prise && (point.1 - y).abs() <= prise
        });
        coin.is_some_and(|h| {
            self.tenir(&zone, Prise::Coin(h), point);
            true
        })
    }

    /// **Dans le vide : le bandeau d'une zone la choisit et la prend.** Ailleurs, la zone
    /// choisie ne l'est plus, et le vide reste au canevas.
    pub(crate) fn prendre_le_bandeau(&mut self) -> bool {
        let board = self.store.project.active_board_id.clone();
        let point = self.sous_le_curseur();
        let bandeau = crate::renderer::zone::BANDEAU as f64;
        let zone = self
            .store
            .zones(&board)
            .iter()
            .rev()
            .find(|z| {
                (z.x..=z.x + z.width).contains(&point.0)
                    && (z.y..=z.y + bandeau.min(z.height)).contains(&point.1)
            })
            .cloned();
        let Some(zone) = zone else {
            if self.ui.zones.choisie.take().is_some() {
                self.mark_dirty();
            }
            return false;
        };
        self.ui.zones.choisie = Some(zone.slot_id.clone());
        self.store.clear_selection();
        self.tenir(&zone, Prise::Bandeau, point);
        true
    }

    /// Commence un geste sur cette zone : une entrée du journal, close au relâchement.
    fn tenir(&mut self, zone: &glucose_core::types::BoardZone, prise: Prise, pointeur: (f64, f64)) {
        self.store.begin_live_edit();
        self.ui.zones.tenue = Some(Tenue {
            zone: zone.slot_id.clone(),
            prise,
            depart: boite(zone),
            pointeur,
        });
        self.mark_dirty();
    }

    /// **La zone tenue suit la main** : déplacée par son bandeau, tirée par un coin jusqu'à sa
    /// taille minimale. Rend vrai si une zone est tenue.
    pub(crate) fn suivre_la_zone_tenue(&mut self) -> bool {
        let Some(tenue) = self.ui.zones.tenue.clone() else {
            return false;
        };
        let point = self.sous_le_curseur();
        let delta = (point.0 - tenue.pointeur.0, point.1 - tenue.pointeur.1);
        let r = match tenue.prise {
            Prise::Bandeau => AlignRect::new(
                tenue.depart.left + delta.0,
                tenue.depart.top + delta.1,
                tenue.depart.width,
                tenue.depart.height,
            ),
            Prise::Coin(h) => resize_rect(tenue.depart, h, delta, ResizeRule::zone()),
        };
        let board = self.store.project.active_board_id.clone();
        self.store
            .placer_la_zone(&board, &tenue.zone, (r.left, r.top, r.width, r.height));
        self.mark_dirty();
        true
    }

    /// **Le geste finit** : une seule entrée du journal, ou aucune si rien n'a bougé.
    pub(crate) fn lacher_la_zone(&mut self) {
        if self.ui.zones.tenue.take().is_some() {
            self.store.end_live_edit();
        }
    }

    /// **Retire la zone choisie** — la touche Suppr. Rend vrai s'il y en avait une.
    pub(crate) fn retirer_la_zone_choisie(&mut self) -> bool {
        let Some(id) = self.ui.zones.choisie.take() else {
            return false;
        };
        let board = self.store.project.active_board_id.clone();
        self.store.retirer_la_zone(&board, &id);
        self.mark_dirty();
        true
    }
}

#[cfg(test)]
mod tests;
