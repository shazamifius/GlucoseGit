//! **Le storyboard sur le canevas** (STORYBOARD-1, fiche 64 ; STORYBOARD-2, fiche 65) : la
//! grille armée suit le curseur, se pose d'un clic ou **se trace** en rectangle (TRACE-1) ; une
//! case se prend par sa **bande** — la description sous son cadre —, se glisse avec les images
//! qu'elle porte et **prend la place où on la lâche** (les autres glissent d'un cran), se
//! décrit d'un double clic, se retire par sa croix ou `Suppr`.
//!
//! # Par sa bande, et non par son cadre
//!
//! Le cadre d'une case est fait pour recevoir une image : le prendre volerait le clic à l'image
//! qu'on y pose, et au rectangle de sélection qu'on y trace. La bande, elle, n'est qu'à la case
//! — la règle des zones, et la demande la plus votée du forum de Figma pour ses sections.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use glucose_core::geometry::Rect;
use glucose_core::store::storyboard::{Disposition, Grille, DESCRIPTION};

/// La case tenue : laquelle, son coin et le point du monde au départ, et les images qu'elle
/// emporte.
#[derive(Debug, Clone, PartialEq)]
pub struct Tenue {
    case: String,
    depart: (f64, f64),
    pointeur: (f64, f64),
    /// Les images qu'elle porte, qui la suivent.
    portees: Vec<String>,
    /// Le storyboard au départ du geste : sa grille, le coin de sa première place, son nombre
    /// de cases — la case qu'on tient a pu être la première.
    suite: Disposition,
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

    /// **Ce que la grille armée poserait** : ce que le tracé en cours dirait au relâchement —
    /// ou, sans tracé, le clic sous le curseur.
    fn disposition_armee(&self) -> Option<Disposition> {
        if let Some(trace) = &self.ui.placement.trace {
            return self.disposition_du_storyboard_trace(trace);
        }
        let grille = self.ui.placement.storyboard?;
        let board = &self.store.project.active_board_id;
        let centre = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport());
        Some(self.store.disposition_au_clic(board, grille, centre))
    }

    /// **Le fantôme du storyboard**, une boîte par case — son cadre et sa bande —, en unités du
    /// monde : la grille armée, ou la place où la case tenue tomberait.
    pub(crate) fn fantome_du_storyboard(&self) -> Vec<Rect> {
        if let Some(place) = self.place_de_la_case_tenue() {
            return vec![place];
        }
        let Some(d) = self.disposition_armee() else {
            return Vec::new();
        };
        let bande = DESCRIPTION;
        d.cadres()
            .into_iter()
            .map(|(x, y, l, h)| Rect::new(x, y, l, h + bande))
            .collect()
    }

    /// **La souris bouge sous la grille armée** : son fantôme la suit, donc une image est à
    /// refaire — sans quoi il resterait où il était jusqu'au prochain dessin (son retour du
    /// 13/10 : « l'outil storyboard ne fonctionne pas au survol »).
    pub(crate) fn suivre_la_grille_armee(&mut self) {
        if self.ui.placement.storyboard.is_some() {
            self.mark_dirty();
        }
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

    /// **La poignée du storyboard** : au coin bas droit de sa boîte, quand une de ses cases est
    /// choisie — son coin dans le monde, et la prise de la main autour, celle des poignées des
    /// nœuds (`handle_slop_world`).
    fn poignee_du_storyboard(&self) -> Option<(Disposition, (f64, f64), f64)> {
        self.ui.storyboard.choisie.as_ref()?;
        let board = &self.store.project.active_board_id;
        let d = self.store.disposition_actuelle(board, Grille::default())?;
        let (x, y, w, h) = d.boite();
        let zoom = self.store.viewport().scale;
        let prise = glucose_core::hit_priority::handle_slop_world(zoom, w, h)?;
        Some((d, (x + w, y + h), prise))
    }

    /// **Tirer la poignée retrace le storyboard** (STORYBOARD-2) : c'est un tracé parti du
    /// coin de sa première case, dont la main tient le coin opposé — les cases apparaissent et
    /// disparaissent sous elle, celles qui portent quelque chose restent. Le clic est pris.
    pub(crate) fn prendre_la_poignee_du_storyboard(&mut self) -> bool {
        let Some((d, coin, prise)) = self.poignee_du_storyboard() else {
            return false;
        };
        let main = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport());
        if (main.0 - coin.0).abs() > prise || (main.1 - coin.1).abs() > prise {
            return false;
        }
        let vp = self.store.viewport();
        self.ui.placement.storyboard = Some(d.grille);
        self.ui.placement.trace = Some(crate::interactions::trace::Trace {
            depart: d.origine,
            depart_ecran: crate::canvas::world_to_screen(d.origine.0, d.origine.1, &vp),
            point: Some(coin),
            decalage: (coin.0 - main.0, coin.1 - main.1),
        });
        self.mark_dirty();
        true
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
        let Some(suite) = self.store.disposition_actuelle(&board, Grille::default()) else {
            return false;
        };
        self.store.begin_live_edit();
        self.ui.storyboard.tenue = Some(Tenue {
            portees: self.store.images_de_la_case(&board, &case.id),
            case: case.id,
            depart: (case.x, case.y),
            pointeur: point,
            suite,
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

    /// **La place où la case tenue tomberait** : celle sous le centre de son cadre, en boîte du
    /// monde — cadre et bande.
    fn place_de_la_case_tenue(&self) -> Option<Rect> {
        let tenue = self.ui.storyboard.tenue.as_ref()?;
        let rang = self.rang_de_la_case_tenue(tenue)?;
        let (x, y) = tenue.suite.grille.coin(tenue.suite.origine, rang);
        let g = tenue.suite.grille;
        Some(Rect::new(x, y, g.largeur, g.hauteur() + DESCRIPTION))
    }

    /// Le rang de la place sous le centre du cadre de la case tenue.
    fn rang_de_la_case_tenue(&self, tenue: &Tenue) -> Option<usize> {
        let board = &self.store.project.active_board_id;
        let case = self
            .store
            .cases(board)
            .into_iter()
            .find(|c| c.id == tenue.case)?;
        let centre = (case.x + case.width / 2.0, case.y + case.height / 2.0);
        Some(tenue.suite.rang_sous(centre))
    }

    /// **Le geste finit : la case prend la place où on la lâche**, les autres glissent d'un
    /// cran, chacune avec ses images — une seule entrée du journal, ou aucune si elle revient
    /// à sa place.
    pub(crate) fn lacher_la_case(&mut self) {
        let Some(tenue) = self.ui.storyboard.tenue.take() else {
            return;
        };
        if let Some(rang) = self.rang_de_la_case_tenue(&tenue) {
            let board = self.store.project.active_board_id.clone();
            let tenue_par = (tenue.case.as_str(), tenue.portees.as_slice());
            self.store
                .deplacer_la_case(&board, tenue_par, rang, tenue.suite.origine);
        }
        self.store.end_live_edit();
        self.mark_dirty();
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

    /// **Tab dans la description d'une case** : elle s'enregistre, et la suivante se décrit —
    /// Maj+Tab, la précédente. Légender un storyboard d'une traite, comme dans Boords ou un
    /// tableur (fiche 65 § 9). Après la dernière, la question se referme. Rend vrai si la
    /// question posée décrivait une case.
    pub(crate) fn decrire_la_voisine(&mut self, arriere: bool) -> bool {
        use crate::ui::question::{Reponse, Suite};
        let Some((_, Suite::DecrireLaCase { board, case })) = &self.ui.question else {
            return false;
        };
        let cases = self.store.cases(board);
        let rang = cases.iter().position(|c| c.id == *case);
        let voisine = match (rang, arriere) {
            (Some(r), true) => r.checked_sub(1),
            (Some(r), false) => Some(r + 1),
            (None, _) => None,
        };
        let voisine = voisine.and_then(|r| cases.get(r)).map(|c| c.id.clone());
        self.repondre_a_la_question(Some(Reponse::Oui));
        if let Some(id) = voisine {
            self.decrire_la_case(&id);
        }
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
