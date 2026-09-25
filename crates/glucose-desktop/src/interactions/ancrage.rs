//! **Les gestes de l'éditeur d'ancres** (FLECHE-4) : l'ouvrir, choisir un passage à la souris,
//! passer d'un côté à l'autre, terminer — un seul geste pour les deux côtés — ou annuler.
//!
//! # Tant qu'il est ouvert, il prend la souris
//!
//! Un clic hors de la carte de l'étape ne désélectionne rien et ne déplace rien : on est en
//! train de désigner un passage, et un faux clic ne doit pas tout défaire. `Échap` annule,
//! `Entrée` valide l'étape.
//!
//! # Un clic prend un mot, un glisser prend ce qu'il couvre
//!
//! Glisser sur le texte choisit exactement ce qui est couvert, au caractère près. Un clic sans
//! glisser prend le mot sous la souris : c'est le geste le plus fréquent — « ce mot-là » —, et
//! il ne demande pas de viser ses deux bouts.

use crate::app::GlucoseApp;
use crate::params::Eclairage;
use crate::renderer::richtext::TextMode;
use crate::ui::ancrage::{ActionDAncrage, Ancrage, Etape};
use glucose_core::text::selection::{expand, Granularity};
use glucose_core::text_anchors::{add_anchor, create_anchor, normalize_text_sel, resolve_anchors};
use glucose_core::types::{Annotation, TextAnchor, TextSelection};

impl GlucoseApp {
    /// **Ouvre l'éditeur d'ancres** de la flèche sélectionnée, si elle touche une carte de
    /// texte, et vole jusqu'à la carte de sa première étape.
    pub(crate) fn commencer_l_ancrage(&mut self) {
        self.commit_editing();
        let Some(Annotation::Arrow {
            id,
            source_id,
            target_id,
            source_text_sel,
            target_text_sel,
            ..
        }) = self.store.selected_arrows().first().copied().cloned()
        else {
            return;
        };
        let carte = |noeud: &Option<String>| {
            let noeud = noeud.as_ref()?;
            let board = self.store.active_board()?;
            matches!(
                self.store.project.annotation(&board.id, noeud),
                Some(Annotation::Text { .. })
            )
            .then(|| noeud.clone())
        };
        let cartes = (carte(&source_id), carte(&target_id));
        let etape = match cartes {
            (Some(_), _) => Etape::Source,
            (None, Some(_)) => Etape::Cible,
            (None, None) => return,
        };
        let source = self.ancres_rafraichies(cartes.0.as_deref(), source_text_sel.as_ref());
        let cible = self.ancres_rafraichies(cartes.1.as_deref(), target_text_sel.as_ref());
        self.ui.ancrage = Some(Ancrage {
            fleche: id,
            etape,
            cartes,
            source,
            cible,
            glisse: None,
        });
        self.voler_vers_la_carte_de_l_etape();
        self.mark_dirty();
    }

    /// Les ancres gardées d'un côté, remises aux positions du texte d'aujourd'hui : ce qu'on
    /// édite est ce qu'on voit briller.
    fn ancres_rafraichies(
        &self,
        carte: Option<&str>,
        passage: Option<&TextSelection>,
    ) -> Vec<TextAnchor> {
        let Some(texte) = carte.and_then(|c| self.texte_de(c)) else {
            return Vec::new();
        };
        resolve_anchors(&texte, &normalize_text_sel(passage))
            .into_iter()
            .filter_map(|r| create_anchor(&texte, r.start, r.end))
            .collect()
    }

    /// Le texte d'une annotation du tableau actif.
    fn texte_de(&self, id: &str) -> Option<String> {
        let board = self.store.active_board()?;
        self.store.project.annotation(&board.id, id)?.own_text()
    }

    /// La caméra vole jusqu'à la carte de l'étape, à sa taille de lecture.
    fn voler_vers_la_carte_de_l_etape(&mut self) {
        let Some(carte) = self
            .ui
            .ancrage
            .as_ref()
            .and_then(|a| a.carte().map(str::to_string))
        else {
            return;
        };
        let Some(boite) = self
            .store
            .active_board()
            .and_then(|b| glucose_core::arrow::node_rect(b, &carte))
        else {
            return;
        };
        let (l, h) = self.taille_de_la_fenetre();
        let ecran = glucose_core::membrane_focus::ScreenSize {
            width: f64::from(l),
            height: f64::from(h),
        };
        let bandeau = f64::from(self.ui.header_height());
        let vue = crate::interactions::vol::vue_sur(boite, ecran, bandeau);
        self.vol.voler_vers(vue);
    }

    /// **Un clic pendant l'édition des ancres** : un bouton du panneau, ou le début d'un choix
    /// dans la carte de l'étape. Tout autre clic est pris sans rien faire.
    pub(crate) fn click_ancrage(&mut self, pointer: crate::params::Pointer, scale: f32) -> bool {
        let Some(ancrage) = &self.ui.ancrage else {
            return false;
        };
        let panneau = crate::ui::ancrage::layout_ancrage(
            ancrage,
            &self.renderer.typography,
            self.taille_de_la_fenetre(),
            scale,
        );
        if crate::ui::ancrage::couvre(&panneau, pointer.x, pointer.y) {
            if let Some(action) = crate::ui::ancrage::action_sous(&panneau, pointer.x, pointer.y) {
                self.agir_sur_l_ancrage(action);
            }
            return true;
        }
        if let Some(o) = self
            .sur_la_carte()
            .then(|| self.octet_sous_la_souris())
            .flatten()
        {
            let ajoute = self.modifiers.control_key();
            if let Some(a) = self.ui.ancrage.as_mut() {
                a.glisse = Some((o, o, ajoute));
            }
            self.mark_dirty();
        }
        true
    }

    /// L'octet du texte de la carte de l'étape sous la souris, dans sa mise en page au repos —
    /// **seulement si la souris est sur la carte**.
    ///
    /// `offset_in_card` ramène tout point au caractère le plus proche : c'est ce qu'il faut à
    /// l'édition, où cliquer dans la marge pose le curseur au plus près. Ici ce serait un
    /// défaut — un clic n'importe où sur le canevas choisirait le mot le plus proche de la
    /// carte. Un glisser commencé sur la carte, lui, continue d'en suivre le texte quand la
    /// souris en sort.
    fn octet_sous_la_souris(&self) -> Option<usize> {
        let carte = self.ui.ancrage.as_ref()?.carte()?;
        let texte = self.texte_de(carte)?;
        self.offset_in_card(carte, &texte, self.mouse_pos, TextMode::Rendered)
    }

    /// La souris est-elle sur la carte de l'étape ?
    fn sur_la_carte(&self) -> bool {
        let Some(carte) = self.ui.ancrage.as_ref().and_then(|a| a.carte()) else {
            return false;
        };
        let vp = self.store.viewport();
        let (wx, wy) = crate::canvas::screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);
        self.store
            .active_board()
            .and_then(|b| glucose_core::arrow::node_rect(b, carte))
            .is_some_and(|r| r.contains_point(glucose_core::geometry::Point::new(wx, wy)))
    }

    /// **Le glisser suit la souris** — rend vrai s'il y en a un en cours.
    pub(crate) fn glisser_l_ancrage(&mut self) -> bool {
        if self.ui.ancrage.as_ref().and_then(|a| a.glisse).is_none() {
            return false;
        }
        if let Some(o) = self.octet_sous_la_souris() {
            if let Some((_, fin, _)) = self.ui.ancrage.as_mut().and_then(|a| a.glisse.as_mut()) {
                *fin = o;
            }
            self.mark_dirty();
        }
        true
    }

    /// **Le relâchement pose le choix** : ce que le glisser couvre, ou le mot sous un clic.
    /// `Ctrl` ajoute au choix ; sans lui, le choix est remplacé.
    pub(crate) fn lacher_l_ancrage(&mut self) -> bool {
        let Some((debut, fin, ajoute)) = self.ui.ancrage.as_mut().and_then(|a| a.glisse.take())
        else {
            return false;
        };
        let Some(texte) = self
            .ui
            .ancrage
            .as_ref()
            .and_then(|a| a.carte())
            .and_then(|c| self.texte_de(c))
        else {
            return true;
        };
        let plage = if debut == fin {
            let mot = expand(&texte, debut, fin, Granularity::Word);
            mot.range()
        } else {
            (debut.min(fin), debut.max(fin))
        };
        if let (Some(ancre), Some(a)) = (
            create_anchor(&texte, plage.0, plage.1),
            self.ui.ancrage.as_mut(),
        ) {
            let ancres = a.ancres_mut();
            *ancres = if ajoute {
                add_anchor(std::mem::take(ancres), ancre)
            } else {
                vec![ancre]
            };
        }
        self.mark_dirty();
        true
    }

    /// Ce qu'un bouton du panneau — ou `Entrée`, `Échap` — demande.
    pub(crate) fn agir_sur_l_ancrage(&mut self, action: ActionDAncrage) {
        match action {
            ActionDAncrage::Effacer => {
                if let Some(a) = self.ui.ancrage.as_mut() {
                    a.ancres_mut().clear();
                }
            }
            ActionDAncrage::Suivant => {
                if let Some(a) = self.ui.ancrage.as_mut() {
                    a.etape = Etape::Cible;
                }
                self.voler_vers_la_carte_de_l_etape();
            }
            ActionDAncrage::Terminer => self.terminer_l_ancrage(),
            ActionDAncrage::Annuler => self.ui.ancrage = None,
        }
        self.mark_dirty();
    }

    /// **Écrit les deux côtés, en un seul geste** — qu'un `Ctrl+Z` défait d'un coup.
    fn terminer_l_ancrage(&mut self) {
        let Some(a) = self.ui.ancrage.take() else {
            return;
        };
        let board = self.store.project.active_board_id.clone();
        self.store.begin_live_edit();
        self.store
            .ancrer_la_fleche(&board, &a.fleche, (a.source, a.cible));
        self.store.end_live_edit();
    }

    /// `Échap` annule, `Entrée` valide l'étape. Rend vrai si la touche était pour l'éditeur.
    pub(crate) fn touche_de_l_ancrage(&mut self, touche: &winit::keyboard::Key) -> bool {
        use winit::keyboard::{Key, NamedKey};
        let Some(a) = &self.ui.ancrage else {
            return false;
        };
        let action = match touche {
            Key::Named(NamedKey::Escape) => ActionDAncrage::Annuler,
            Key::Named(NamedKey::Enter) if a.a_une_suite() => ActionDAncrage::Suivant,
            Key::Named(NamedKey::Enter) => ActionDAncrage::Terminer,
            _ => return false,
        };
        self.agir_sur_l_ancrage(action);
        true
    }

    /// **Ce que l'éditeur fait briller** : le choix de l'étape, et ce que le glisser couvre.
    pub(crate) fn eclairages_de_l_ancrage(&self) -> Option<Vec<Eclairage>> {
        let a = self.ui.ancrage.as_ref()?;
        let carte = a.carte()?;
        let texte = self.texte_de(carte)?;
        let mut plages: Vec<(usize, usize)> = resolve_anchors(&texte, a.ancres())
            .into_iter()
            .map(|r| (r.start, r.end))
            .collect();
        if let Some((debut, fin, _)) = a.glisse.filter(|g| g.0 != g.1) {
            plages.push((debut.min(fin), debut.max(fin)));
        }
        Some(vec![Eclairage {
            carte: carte.to_string(),
            plages,
            teinte: None,
        }])
    }
}

#[cfg(test)]
mod tests;
