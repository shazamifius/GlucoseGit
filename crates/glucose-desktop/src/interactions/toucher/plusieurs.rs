//! **GESTES-1 — les gestes à plusieurs doigts** (fiche 58).
//!
//! Sa demande, le 08/10 : *« impossible de faire une multi-sélection et un clic droit ; avec 2
//! doigts on crée des raccourcis, et avec 3 aussi ; le plus simple possible, le moins de
//! boutons possible »*. La carte, acceptée le 09/10, sans un bouton de plus :
//!
//! * **Deux doigts touchés** ensemble annulent, **trois** rétablissent — Procreate, Infinite
//!   Painter, Concepts. Un toucher, c'est : tous les doigts levés avant que l'appui long ne
//!   prenne, aucun n'ayant bougé plus qu'une main qui tremble. Aucun délai n'est choisi ici :
//!   ce sont celui du système (`plateforme::doigt`, l'accessibilité d'Android) et le
//!   tremblement.
//! * **L'appui long prend** — le téléphone vibre —, **puis ce qui suit décide** : lever le
//!   doigt ouvre le menu du clic droit (APPUI-1) ; le glisser trace un rectangle de sélection
//!   (FigJam) ; un autre doigt qui touche des nœuds les ajoute à la sélection (Concepts,
//!   Affinity Designer).
//!
//! # Pourquoi le menu attend le lever du doigt
//!
//! La première carte disait « un doigt tient un nœud pendant qu'un autre en touche ». Le menu de
//! l'appui long l'aurait coupée net : il s'ouvrait à l'échéance, 0,4 s après la pose. Il s'ouvre
//! donc quand le doigt se lève sans que rien d'autre n'ait suivi — et c'est ce qui suit l'appui
//! qui choisit le geste.

use crate::app::GlucoseApp;
use std::time::Instant;
use winit::event::MouseButton;

/// **Un toucher à plusieurs doigts qui se décide** : quand le premier doigt s'est posé, combien
/// de doigts au plus, et où chacun s'est posé.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Bref {
    pub(crate) debut: Instant,
    doigts: usize,
    departs: Vec<(u64, (f64, f64))>,
}

/// **L'appui long a pris** : le doigt qui tient, où, et ce qui a suivi.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Pris {
    tenu: u64,
    depart: (f64, f64),
    suite: Suite,
    /// Les autres doigts posés depuis, où ils se sont posés, et s'ils ont bougé.
    autres: Vec<(u64, (f64, f64), bool)>,
}

/// Ce qui a suivi l'appui long.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Suite {
    /// Rien encore : le lever ouvrira le menu.
    Rien,
    /// Le doigt a glissé : un rectangle de sélection.
    Cadre,
    /// Un autre doigt a touché un nœud : il s'est ajouté à la sélection.
    Ajout,
}

impl Pris {
    pub(crate) fn nouveau(tenu: u64, depart: (f64, f64)) -> Self {
        Self {
            tenu,
            depart,
            suite: Suite::Rien,
            autres: Vec::new(),
        }
    }
}

impl GlucoseApp {
    /// **Un doigt se pose** : le premier ouvre un toucher bref ; les suivants s'y comptent tant
    /// que l'appui long n'a pas pris. Après lui, un doigt posé est un doigt qui ajoute.
    pub(crate) fn plusieurs_a_la_pose(&mut self, id: u64, ici: (f64, f64)) {
        if let Some(pris) = &mut self.toucher.pris {
            pris.autres.push((id, ici, false));
            return;
        }
        match &mut self.toucher.bref {
            Some(b) if b.debut.elapsed() < crate::plateforme::doigt::appui_long() => {
                b.departs.push((id, ici));
                b.doigts = b.doigts.max(b.departs.len());
            }
            _ => {
                self.toucher.bref = Some(Bref {
                    debut: Instant::now(),
                    doigts: 1,
                    departs: vec![(id, ici)],
                });
            }
        }
    }

    /// **Un doigt bouge** : au-delà d'une main qui tremble, ce n'était pas un toucher. Après un
    /// appui long, le doigt qui tient trace le rectangle ; rend `true` si le mouvement lui
    /// appartenait — les deux doigts ne font alors pas leur similitude.
    pub(crate) fn plusieurs_au_mouvement(&mut self, id: u64, ici: (f64, f64)) -> bool {
        let tremblement = self.tremblement();
        let loin = |depart: (f64, f64)| (ici.0 - depart.0).hypot(ici.1 - depart.1) > tremblement;
        if let Some(b) = &self.toucher.bref {
            if b.departs.iter().any(|(i, d)| *i == id && loin(*d)) {
                self.toucher.bref = None;
            }
        }
        let Some(pris) = &mut self.toucher.pris else {
            return false;
        };
        if id != pris.tenu {
            for (i, depart, bouge) in &mut pris.autres {
                *bouge |= *i == id && loin(*depart);
            }
            return true;
        }
        match pris.suite {
            Suite::Rien if loin(pris.depart) => {
                pris.suite = Suite::Cadre;
                let depart = pris.depart;
                self.start_selection_box(depart.0, depart.1);
                self.handle_cursor_moved(winit::dpi::PhysicalPosition::new(ici.0, ici.1));
            }
            Suite::Cadre => {
                self.handle_cursor_moved(winit::dpi::PhysicalPosition::new(ici.0, ici.1));
            }
            _ => {}
        }
        true
    }

    /// **Un doigt se lève** : après un appui long, le doigt qui tient finit le geste — le menu
    /// s'il ne s'est rien passé, le rectangle s'il a glissé ; un autre doigt qui n'a pas bougé
    /// ajoute le nœud qu'il touchait. Rend `true` si le lever appartenait à l'appui.
    pub(crate) fn plusieurs_au_lever(&mut self, id: u64) -> bool {
        let Some(pris) = &mut self.toucher.pris else {
            return false;
        };
        if id != pris.tenu {
            let autre = pris.autres.iter().position(|(i, _, _)| *i == id);
            if let Some((_, ici, false)) = autre.map(|k| pris.autres.remove(k)) {
                if pris.suite != Suite::Cadre {
                    pris.suite = Suite::Ajout;
                    self.ajouter_a_la_selection(ici);
                }
            }
            return true;
        }
        let Some(pris) = self.toucher.pris.take() else {
            return true;
        };
        match pris.suite {
            Suite::Rien => {
                self.right_down_at = Some(pris.depart);
                self.open_context_menu_if_still();
                self.ui.menu_au_doigt = true;
            }
            Suite::Cadre => self.finish_selection_box(),
            Suite::Ajout => {}
        }
        true
    }

    /// **Tous les doigts se sont levés** : un toucher bref à deux doigts annule, à trois
    /// rétablit — s'il n'a pas tenu jusqu'à l'appui long.
    pub(crate) fn plusieurs_tous_leves(&mut self) {
        let Some(b) = self.toucher.bref.take() else {
            return;
        };
        if b.debut.elapsed() >= crate::plateforme::doigt::appui_long() {
            return;
        }
        match b.doigts {
            2 => self.annuler(),
            3 => self.retablir(),
            _ => {}
        }
    }

    /// Le nœud sous ce point rejoint la sélection, comme un clic avec `Maj` — et le pointeur
    /// revient au doigt qui tient. Sur le vide, rien : un clic y viderait la sélection.
    fn ajouter_a_la_selection(&mut self, ici: (f64, f64)) {
        let vue = self.store.viewport();
        let (wx, wy) = crate::canvas::screen_to_world(ici.0, ici.1, &vue);
        if self.pick_candidates_at(wx, wy).is_empty() {
            return;
        }
        let (pointeur, touches) = (self.mouse_pos, self.modifiers);
        self.modifiers = winit::keyboard::ModifiersState::SHIFT;
        self.handle_cursor_moved(winit::dpi::PhysicalPosition::new(ici.0, ici.1));
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        self.handle_mouse_down(MouseButton::Left, largeur, hauteur);
        self.handle_mouse_up(MouseButton::Left);
        self.modifiers = touches;
        self.handle_cursor_moved(winit::dpi::PhysicalPosition::new(pointeur.0, pointeur.1));
    }
}

#[cfg(test)]
mod tests;
