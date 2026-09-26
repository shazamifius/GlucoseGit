//! **Les gestes de l'éditeur du texte lié** (FLECHE-4, ANCRE-UX) : l'ouvrir, choisir un passage
//! à la souris dans sa fenêtre, passer d'un côté à l'autre, terminer — un seul geste pour les
//! deux côtés — ou annuler.
//!
//! # Tant qu'il est ouvert, il prend la souris
//!
//! Sa fenêtre est modale : un clic hors d'elle ne désélectionne rien et ne déplace rien. On est
//! en train de désigner un passage, et un faux clic ne doit pas tout défaire. `Échap` annule,
//! `Entrée` presse le bouton principal.
//!
//! # Un clic prend un mot, un glisser prend ce qu'il couvre
//!
//! Glisser sur le texte choisit exactement ce qui est couvert, au caractère près. Un clic sans
//! glisser prend le mot sous la souris : c'est le geste le plus fréquent — « ce mot-là » —, et
//! il ne demande pas de viser ses deux bouts.

use crate::app::GlucoseApp;
use crate::params::Eclairage;
use crate::renderer::card::{card_text_layout, text_box, TEXT_ORIGIN};
use crate::renderer::richtext::hit::offset_at;
use crate::renderer::richtext::TextMode;
use crate::ui::ancrage::{ActionDAncrage, Ancrage, Etape, Fenetre};
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
            defilement: 0.0,
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

    /// **La fenêtre de l'éditeur**, telle que le dessin la pose — le clic la lit (loi L4).
    pub(crate) fn fenetre_d_ancrage(&self) -> Option<(Fenetre, String)> {
        let a = self.ui.ancrage.as_ref()?;
        let texte = self.texte_de(a.carte()?)?;
        let (l, h) = self.taille_de_la_fenetre();
        let fenetre = crate::ui::ancrage::layout_ancrage(
            a,
            &texte,
            (&self.renderer.typography, &self.renderer.math),
            (l, h),
            self.ui.scale_factor,
        );
        Some((fenetre, texte))
    }

    /// **Un clic pendant l'édition des ancres** : un bouton de la fenêtre, la croix d'une puce,
    /// ou le début d'un choix dans son texte. Tout autre clic est pris sans rien faire.
    pub(crate) fn click_ancrage(&mut self, pointer: crate::params::Pointer) -> bool {
        let Some((fenetre, _)) = self.fenetre_d_ancrage() else {
            return self.ui.ancrage.is_some();
        };
        if let Some(action) = fenetre.action_sous(pointer.x, pointer.y) {
            self.agir_sur_l_ancrage(action);
            return true;
        }
        if fenetre.dans_la_zone(pointer.x, pointer.y) {
            if let Some(o) = self.octet_sous_la_souris() {
                let ajoute = self.modifiers.control_key();
                if let Some(a) = self.ui.ancrage.as_mut() {
                    a.glisse = Some((o, o, ajoute));
                }
                self.mark_dirty();
            }
        }
        true
    }

    /// L'octet du texte de la fenêtre sous la souris, dans sa mise en page au repos.
    ///
    /// Un point hors de la zone se ramène au caractère le plus proche : un glisser commencé
    /// dans le texte continue d'en suivre les lignes quand la souris en sort. C'est pourquoi
    /// **commencer** un choix demande d'être dans la zone ([`Self::click_ancrage`]).
    fn octet_sous_la_souris(&self) -> Option<usize> {
        let (fenetre, texte) = self.fenetre_d_ancrage()?;
        let zone = fenetre.zone;
        let (wx, wy) = zone.vers_le_monde((self.mouse_pos.0 as f32, self.mouse_pos.1 as f32));
        // La mise en page même que le dessin : la place des passages choisis y est ouverte, et
        // l'on vise ce qu'on voit (PASSAGE-2).
        let choisies = self.ui.ancrage.as_ref()?.plages_choisies(&texte);
        let eclaires = crate::renderer::passages::Eclaires {
            plages: &choisies,
            teinte: (0, 0, 0),
            style: &crate::renderer::passages::DANS_L_EDITEUR,
        };
        let mise_en_page = crate::renderer::passages::mise_en_page(
            (&self.renderer.typography, &self.renderer.math),
            (&texte, zone.largeur_monde),
            TextMode::Rendered,
            Some(&eclaires),
        );
        Some(offset_at(
            &self.renderer.typography,
            &mise_en_page,
            &texte,
            (wx - TEXT_ORIGIN.0, wy - TEXT_ORIGIN.1),
            &text_box(zone.largeur_monde),
        ))
    }

    /// **La molette fait défiler le texte de la fenêtre**, quand il est plus haut qu'elle. Rend
    /// vrai si l'éditeur est ouvert : il prend la molette comme il prend la souris.
    pub(crate) fn defiler_l_ancrage(&mut self, points: f32) -> bool {
        let Some((fenetre, _)) = self.fenetre_d_ancrage() else {
            return self.ui.ancrage.is_some();
        };
        let max = fenetre.zone.defilement_max();
        if let Some(a) = self.ui.ancrage.as_mut() {
            a.defilement = (a.defilement + points).clamp(0.0, max);
        }
        self.mark_dirty();
        true
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
        let mut plage = if debut == fin {
            let mot = expand(&texte, debut, fin, Granularity::Word);
            mot.range()
        } else {
            (debut.min(fin), debut.max(fin))
        };
        if let Some((fenetre, _)) = self.fenetre_d_ancrage() {
            let mise_en_page = card_text_layout(
                &self.renderer.typography,
                &self.renderer.math,
                &texte,
                fenetre.zone.largeur_monde,
                TextMode::Rendered,
            );
            plage = etendre_aux_formules(&mise_en_page, plage);
        }
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
            ActionDAncrage::Retirer(rang) => {
                if let Some(a) = self.ui.ancrage.as_mut() {
                    let ancres = a.ancres_mut();
                    if rang < ancres.len() {
                        ancres.remove(rang);
                    }
                }
            }
            ActionDAncrage::Suivant => {
                if let Some(a) = self.ui.ancrage.as_mut() {
                    a.etape = Etape::Cible;
                    a.defilement = 0.0;
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

    /// **Ce que l'éditeur fait briller sur la carte** : le choix de l'étape. Le glisser en
    /// cours ne se montre que dans la fenêtre, où l'on choisit : sur la carte, il ouvrirait sa
    /// place à chaque mouvement de la souris.
    pub(crate) fn eclairages_de_l_ancrage(&self) -> Option<Vec<Eclairage>> {
        let a = self.ui.ancrage.as_ref()?;
        let carte = a.carte()?;
        let texte = self.texte_de(carte)?;
        Some(vec![Eclairage {
            carte: carte.to_string(),
            plages: a.plages_choisies(&texte),
            teinte: None,
        }])
    }
}

/// **Une formule est un atome** : une plage qui touche un paragraphe de formule le prend
/// entier. Au repos, on ne voit pas sa source mais son dessin, et l'on ne désigne pas la moitié
/// d'une fraction — le cadre du passage épouse d'ailleurs la formule dessinée
/// (`passages::troncons`).
fn etendre_aux_formules(
    mise_en_page: &crate::renderer::richtext::TextLayout,
    (mut debut, mut fin): (usize, usize),
) -> (usize, usize) {
    for ligne in &mise_en_page.lines {
        let formule = matches!(ligne.kind, glucose_core::text::BlockKind::Math { .. });
        let touche = debut < ligne.end && fin.max(debut + 1) > ligne.start;
        if formule && ligne.first && touche {
            (debut, fin) = (debut.min(ligne.start), fin.max(ligne.end));
        }
    }
    (debut, fin)
}

#[cfg(test)]
mod tests;
