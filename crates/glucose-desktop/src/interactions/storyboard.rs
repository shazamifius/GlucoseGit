//! **Le storyboard sur le canevas** (STORYBOARD-1, fiche 64) : la grille armée suit le curseur
//! et se pose d'un clic, comme un gabarit (PRESET-1) ; une case se prend par sa **bande** — la
//! description sous son cadre —, se glisse avec les images qu'elle porte, se décrit d'un double
//! clic, se retire par sa croix ou `Suppr`.
//!
//! # Par sa bande, et non par son cadre
//!
//! Le cadre d'une case est fait pour recevoir une image : le prendre volerait le clic à l'image
//! qu'on y pose, et au rectangle de sélection qu'on y trace. La bande, elle, n'est qu'à la case
//! — la règle des zones, et la demande la plus votée du forum de Figma pour ses sections.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use glucose_core::geometry::Rect;
use glucose_core::store::storyboard::{Grille, DESCRIPTION};

/// La case tenue : laquelle, son coin et le point du monde au départ, et les images qu'elle
/// emporte.
#[derive(Debug, Clone, PartialEq)]
pub struct Tenue {
    case: String,
    depart: (f64, f64),
    pointeur: (f64, f64),
    /// Les images qu'elle porte, qui la suivent.
    portees: Vec<String>,
}

/// Ce que l'interface sait du storyboard : la case choisie, la case tenue.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StoryboardUi {
    pub choisie: Option<String>,
    pub tenue: Option<Tenue>,
}

impl GlucoseApp {
    /// **Arme la grille** : elle suit le curseur jusqu'au clic qui la pose. Le panneau se
    /// ferme, comme celui des Presets, pour laisser voir le canevas où elle va se poser — au
    /// téléphone, il le couvrirait.
    fn armer_le_storyboard(&mut self, grille: Grille) {
        if self.dock_manager.is_open(crate::dock::TabId::Storyboard) {
            self.dock_manager.toggle_tab(crate::dock::TabId::Storyboard);
        }
        self.ui.placement.storyboard = Some(grille);
        self.mark_dirty();
    }

    /// **Ce que demande le panneau du storyboard** (STORYBOARD-1).
    pub(crate) fn suivre_le_panneau_du_storyboard(
        &mut self,
        intent: crate::dock::storyboard::StoryboardIntent,
    ) {
        use crate::dock::storyboard::StoryboardIntent as I;
        let board = self.store.project.active_board_id.clone();
        match intent {
            I::Armer(grille) => self.armer_le_storyboard(grille),
            I::Effacer => {
                self.store.effacer_le_storyboard(&board);
                self.ui.storyboard.choisie = None;
            }
            I::Ajouter(grille) => {
                self.store.ajouter_une_case(&board, grille);
            }
            I::Decrire(id) => self.decrire_la_case(&id),
            I::Retirer(id) => {
                if self.ui.storyboard.choisie.as_deref() == Some(id.as_str()) {
                    self.ui.storyboard.choisie = None;
                }
                self.store.retirer_la_case(&board, &id);
            }
            I::Renumeroter => self.store.renumeroter_les_cases(&board),
        }
        self.mark_dirty();
    }

    /// Le coin de chaque case que la grille armée poserait sous le curseur.
    fn grille_sous_le_curseur(&self) -> Option<(Grille, Vec<(f64, f64)>)> {
        let grille = self.ui.placement.storyboard?;
        let board = &self.store.project.active_board_id;
        let centre = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport());
        Some((grille, self.store.coins_a_poser(board, grille, centre)))
    }

    /// **Le fantôme de la grille armée**, une boîte par case — son cadre et sa bande —, en
    /// unités du monde.
    pub(crate) fn fantome_du_storyboard(&self) -> Vec<Rect> {
        let Some((grille, coins)) = self.grille_sous_le_curseur() else {
            return Vec::new();
        };
        let hauteur = grille.hauteur() + DESCRIPTION;
        coins
            .into_iter()
            .map(|(x, y)| Rect::new(x, y, grille.largeur, hauteur))
            .collect()
    }

    /// **Pose la grille armée là où son fantôme est**, s'il y en a une : le clic est pris. Un
    /// seul `Ctrl+Z` la retire.
    pub(crate) fn poser_le_storyboard_arme(&mut self) -> bool {
        let Some(grille) = self.ui.placement.storyboard.take() else {
            return false;
        };
        let centre = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport());
        let board = self.store.project.active_board_id.clone();
        self.store.poser_le_storyboard(&board, grille, centre);
        self.mark_dirty();
        true
    }

    /// **Renonce à la grille armée** — Échap. Rend vrai s'il y en avait une.
    pub(crate) fn renoncer_au_storyboard(&mut self) -> bool {
        if self.ui.placement.storyboard.take().is_none() {
            return false;
        }
        self.mark_dirty();
        true
    }

    /// **La croix de la case choisie** passe avant les nœuds, comme les prises d'un nœud
    /// choisi : elle retire la case.
    pub(crate) fn prendre_la_case_choisie(&mut self) -> bool {
        let Some(id) = self.ui.storyboard.choisie.clone() else {
            return false;
        };
        let board = self.store.project.active_board_id.clone();
        let Some(case) = self
            .store
            .cases(&board)
            .into_iter()
            .find(|c| c.id == id)
            .cloned()
        else {
            self.ui.storyboard.choisie = None;
            return false;
        };
        let vp = self.store.viewport();
        let boite = crate::renderer::storyboard::boite_a_l_ecran(&case, &vp);
        let bande = DESCRIPTION as f32 * vp.scale as f32;
        let (cx, cy, cw, ch) = crate::renderer::storyboard::croix(boite, bande, self.ui.scale());
        let (px, py) = (self.mouse_pos.0 as f32, self.mouse_pos.1 as f32);
        if !((cx..=cx + cw).contains(&px) && (cy..=cy + ch).contains(&py)) {
            return false;
        }
        self.retirer_la_case_choisie()
    }

    /// **Dans le vide : la bande d'une case la choisit et la prend** ; un second clic sur la
    /// même bande la décrit. Ailleurs, la case choisie ne l'est plus.
    pub(crate) fn prendre_la_bande(&mut self) -> bool {
        let board = self.store.project.active_board_id.clone();
        let point = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport());
        let case = self.store.cases(&board).into_iter().rev().find(|c| {
            let bas = c.y + c.height;
            (c.x..=c.x + c.width).contains(&point.0) && (bas..=bas + DESCRIPTION).contains(&point.1)
        });
        let Some(case) = case.cloned() else {
            if self.ui.storyboard.choisie.take().is_some() {
                self.mark_dirty();
            }
            return false;
        };
        let cle = format!("case:{}", case.id);
        let rang = self.click_count_at(&cle);
        self.remember_click(cle, rang);
        self.ui.storyboard.choisie = Some(case.id.clone());
        self.store.clear_selection();
        if rang >= 2 {
            self.decrire_la_case(&case.id);
            return true;
        }
        self.store.begin_live_edit();
        self.ui.storyboard.tenue = Some(Tenue {
            portees: self.store.images_de_la_case(&board, &case.id),
            case: case.id,
            depart: (case.x, case.y),
            pointeur: point,
        });
        self.mark_dirty();
        true
    }

    /// **La case tenue suit la main**, avec les images qu'elle porte. Rend vrai si une case est
    /// tenue.
    pub(crate) fn suivre_la_case_tenue(&mut self) -> bool {
        let Some(tenue) = self.ui.storyboard.tenue.clone() else {
            return false;
        };
        let point = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport());
        let total = (point.0 - tenue.pointeur.0, point.1 - tenue.pointeur.1);
        let board = self.store.project.active_board_id.clone();
        let case = (tenue.case.as_str(), tenue.depart);
        self.store
            .glisser_la_case(&board, case, &tenue.portees, total);
        self.mark_dirty();
        true
    }

    /// **Le geste finit** : une seule entrée du journal, ou aucune si rien n'a bougé.
    pub(crate) fn lacher_la_case(&mut self) {
        if self.ui.storyboard.tenue.take().is_some() {
            self.store.end_live_edit();
        }
    }

    /// **Retire la case choisie** — sa croix, ou la touche Suppr. Rend vrai s'il y en avait une.
    pub(crate) fn retirer_la_case_choisie(&mut self) -> bool {
        let Some(id) = self.ui.storyboard.choisie.take() else {
            return false;
        };
        let board = self.store.project.active_board_id.clone();
        self.store.retirer_la_case(&board, &id);
        self.mark_dirty();
        true
    }

    /// **Décrit une case** : une question dessinée, un champ que le clavier remplit — au
    /// téléphone aussi, par le miroir (CLAVIER-1). Tout est choisi : la première lettre tapée
    /// remplace l'ancienne description.
    pub(crate) fn decrire_la_case(&mut self, id: &str) {
        use crate::ui::question::{Champ, Question, Reponse, Suite};
        let board = self.store.project.active_board_id.clone();
        let Some(case) = self
            .store
            .cases(&board)
            .into_iter()
            .find(|c| c.id == id)
            .cloned()
        else {
            return;
        };
        let texte = case.description.clone();
        let question = Question {
            titre: format!("Case {}", case.order + 1),
            texte: "Ce qui s'y passe : l'action, le son, une note.".into(),
            champ: Some(Champ {
                selection: glucose_core::text::Selection::all(&texte),
                texte,
            }),
            choix: vec![
                ("Décrire".into(), Reponse::Oui),
                ("Annuler".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(
            question,
            Suite::DecrireLaCase {
                board,
                case: case.id,
            },
        );
    }
}

#[cfg(test)]
mod tests;
