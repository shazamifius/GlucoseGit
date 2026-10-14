//! **Le storyboard sur le canevas** (STORYBOARD-1, fiche 64 ; STORYBOARD-2, fiche 65) : la
//! grille armée suit le curseur, se pose d'un clic ou **se trace** en rectangle (TRACE-1) ; une
//! case se prend par sa **bande** — la description sous son cadre —, se glisse avec les images
//! qu'elle porte et **prend la place où on la lâche** (les autres glissent d'un cran), se
//! décrit d'un double clic, se retire par sa croix ou `Suppr`.
//!
//! # Le storyboard courant (fiche 68)
//!
//! Un tableau porte plusieurs storyboards. Le panneau, la lecture, « + Ajouter », le menu
//! agissent sur **le courant** ([`storyboard_courant`]) : le dernier qu'on a touché — une case
//! choisie le règle — ou posé, s'il est encore là ; sinon le premier du tableau. Une seule
//! règle, que l'application et le panneau lisent.
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
use glucose_core::store::Store;

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

/// Ce que l'interface sait du storyboard : la case choisie, la case tenue, et le storyboard
/// qu'on a touché ou posé en dernier (fiche 68).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StoryboardUi {
    pub choisie: Option<String>,
    pub tenue: Option<Tenue>,
    pub courant: Option<String>,
}

/// **Le storyboard courant du tableau** (fiche 68) : le dernier touché ou posé, s'il est encore
/// là ; sinon le premier. Aucun sur un tableau sans storyboard.
///
/// Une seule source : choisir une case le règle (`prendre_la_bande`), et poser un storyboard
/// neuf rend la case choisie — la case choisie est donc toujours du courant. Lire d'abord la
/// case choisie aurait fait deux sources, qui divergeaient après « + Nouveau ».
pub fn storyboard_courant(store: &Store, board: &str, ui: &StoryboardUi) -> Option<String> {
    ui.courant
        .as_deref()
        .and_then(|sb| store.storyboard(board, sb))
        .or_else(|| store.storyboards(board).first())
        .map(|s| s.id.clone())
}

impl GlucoseApp {
    /// **Le storyboard courant** du tableau actif ([`storyboard_courant`]).
    pub(crate) fn storyboard_courant(&self) -> Option<String> {
        let board = &self.store.project.active_board_id;
        storyboard_courant(&self.store, board, &self.ui.storyboard)
    }

    /// **Arme la grille** : elle suit le curseur jusqu'au clic qui la pose — un storyboard qui
    /// naît, ou la `cible` rangée de nouveau. Le panneau se ferme, comme celui des Presets, pour
    /// laisser voir le canevas où elle va se poser — au téléphone, il le couvrirait.
    fn armer_le_storyboard(&mut self, grille: Grille, cible: Option<String>) {
        if self.dock_manager.is_open(crate::dock::TabId::Storyboard) {
            self.dock_manager.toggle_tab(crate::dock::TabId::Storyboard);
        }
        self.ui.placement.storyboard = Some(grille);
        self.ui.placement.cible = cible;
        self.mark_dirty();
    }

    /// **Ce que demande le panneau du storyboard** (STORYBOARD-1).
    pub(crate) fn suivre_le_panneau_du_storyboard(
        &mut self,
        intent: crate::dock::storyboard::StoryboardIntent,
    ) {
        use crate::dock::storyboard::StoryboardIntent as I;
        let board = self.store.project.active_board_id.clone();
        let courant = self.storyboard_courant();
        match intent {
            I::Armer(grille) => self.armer_le_storyboard(grille, None),
            I::MettreAJour(grille) => self.armer_le_storyboard(grille, courant),
            I::Changer(pas) => self.changer_de_storyboard(pas),
            I::Effacer => {
                if let Some(sb) = courant {
                    self.store.effacer_le_storyboard(&board, &sb);
                }
                self.ui.storyboard.choisie = None;
            }
            I::Ajouter => {
                if let Some(sb) = courant {
                    self.store.ajouter_une_case(&board, &sb);
                }
            }
            I::Ecrire(id, texte) => self.ecrire_la_case(&id, texte),
            I::Lire(depuis) => self.lire_le_storyboard(depuis.as_deref()),
            I::RendreLaDuree(id) => self.store.regler_la_duree(&board, &id, None),
            I::Plan(id, plan) => self.store.choisir_le_plan(&board, &id, plan),
            I::NommerLeChamp(rang) => self.nommer_le_champ(rang),
            I::Retirer(id) => {
                if self.ui.storyboard.choisie.as_deref() == Some(id.as_str()) {
                    self.ui.storyboard.choisie = None;
                }
                self.store.retirer_la_case(&board, &id);
            }
            I::Planches => self.exporter_les_planches(),
            I::Renumeroter => {
                if let Some(sb) = courant {
                    self.store.renumeroter_les_cases(&board, &sb);
                }
            }
        }
        self.mark_dirty();
    }

    /// **Passe au storyboard voisin** dans le panneau (‹ ›), en tournant : il devient le
    /// courant, et la case choisie — qui le désignerait à sa place — ne l'est plus.
    fn changer_de_storyboard(&mut self, pas: isize) {
        let board = self.store.project.active_board_id.clone();
        let courant = self.storyboard_courant();
        let tous = self.store.storyboards(&board);
        let Some(ici) = courant.and_then(|sb| tous.iter().position(|s| s.id == sb)) else {
            return;
        };
        let n = tous.len() as isize;
        let voisin = tous[(ici as isize + pas).rem_euclid(n) as usize].id.clone();
        self.ui.storyboard.choisie = None;
        self.ui.storyboard.courant = Some(voisin);
    }

    /// **Ce que la grille armée poserait** : ce que le tracé en cours dirait au relâchement —
    /// ou, sans tracé, le clic sous le curseur, la première case **par son coin haut gauche**,
    /// comme le tracé qui en partirait (FANTOME-1, fiche 66).
    fn disposition_armee(&self) -> Option<Disposition> {
        if let Some(trace) = &self.ui.placement.trace {
            return self.disposition_du_storyboard_trace(trace);
        }
        let grille = self.ui.placement.storyboard?;
        let board = &self.store.project.active_board_id;
        let coin = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport());
        let cible = self.ui.placement.cible.as_deref();
        Some(self.store.disposition_au_clic((board, cible), grille, coin))
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

    /// **Un storyboard de la sélection** (P2, proposition 2, fiche 66) : une case par image
    /// choisie, dans l'ordre de lecture ; il naît là où était la sélection, à la grille que le
    /// panneau règle, et devient le courant (fiche 68). Un geste.
    pub(crate) fn storyboard_de_la_selection(&mut self) {
        let board = self.store.project.active_board_id.clone();
        let Some(bornes) = self.store.selection_bounds(&board) else {
            return;
        };
        let grille = self.dock_manager.storyboard.grille();
        let coin = (bornes.left, bornes.top);
        if let Some((ne, _)) = self.store.storyboard_de_la_selection(&board, grille, coin) {
            self.ui.storyboard.choisie = None;
            self.ui.storyboard.courant = Some(ne);
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

    /// **Les boutons de la case choisie** passent avant les nœuds, comme les prises d'un nœud
    /// choisi : ses deux « + » insèrent une case avant ou après elle (P2, proposition 3), sa
    /// croix la retire.
    pub(crate) fn prendre_la_case_choisie(&mut self) -> bool {
        let Some(id) = self.ui.storyboard.choisie.clone() else {
            return false;
        };
        let board = self.store.project.active_board_id.clone();
        let Some(case) = self
            .store
            .toutes_les_cases(&board)
            .iter()
            .find(|c| c.id == id)
            .cloned()
        else {
            self.ui.storyboard.choisie = None;
            return false;
        };
        let vp = self.store.viewport();
        let boite = crate::renderer::storyboard::boite_a_l_ecran(&case, &vp);
        let bande = DESCRIPTION as f32 * vp.scale as f32;
        let (px, py) = (self.mouse_pos.0 as f32, self.mouse_pos.1 as f32);
        let dessous = |(bx, by, bw, bh): crate::renderer::zone::Boite| {
            (bx..=bx + bw).contains(&px) && (by..=by + bh).contains(&py)
        };
        let plus = crate::renderer::storyboard::plus(boite, self.ui.scale());
        if let Some(cote) = plus.into_iter().position(dessous) {
            return self.inserer_a_cote(&id, cote == 1);
        }
        if !dessous(crate::renderer::storyboard::croix(
            boite,
            bande,
            self.ui.scale(),
        )) {
            return false;
        }
        self.retirer_la_case_choisie()
    }

    /// **Insère une case vide avant ou après celle-ci** (P2, proposition 3, fiche 66), et la
    /// choisit : on la décrit, ou on y lâche une image, aussitôt.
    fn inserer_a_cote(&mut self, id: &str, apres: bool) -> bool {
        let board = self.store.project.active_board_id.clone();
        let Some(sb) = self
            .store
            .storyboard_de_la_case(&board, id)
            .map(|s| s.id.clone())
        else {
            return false;
        };
        let Some(rang) = self
            .store
            .cases(&board, &sb)
            .iter()
            .position(|c| c.id == id)
        else {
            return false;
        };
        let neuve = self
            .store
            .inserer_une_case(&board, &sb, rang + usize::from(apres));
        self.ui.storyboard.choisie = neuve;
        self.mark_dirty();
        true
    }

    /// **La poignée du storyboard** : au coin bas droit de sa boîte, quand une de ses cases est
    /// choisie — son storyboard, son coin dans le monde, et la prise de la main autour, celle
    /// des poignées des nœuds (`handle_slop_world`).
    fn poignee_du_storyboard(&self) -> Option<(String, Disposition, (f64, f64), f64)> {
        let choisie = self.ui.storyboard.choisie.as_deref()?;
        let board = &self.store.project.active_board_id;
        let sb = self.store.storyboard_de_la_case(board, choisie)?.id.clone();
        let d = self.store.disposition_actuelle(board, &sb)?;
        let (x, y, w, h) = d.boite();
        let zoom = self.store.viewport().scale;
        let prise = glucose_core::hit_priority::handle_slop_world(zoom, w, h)?;
        Some((sb, d, (x + w, y + h), prise))
    }

    /// **Tirer la poignée retrace le storyboard** (STORYBOARD-2) : c'est un tracé parti du
    /// coin de sa première case, dont la main tient le coin opposé — les cases apparaissent et
    /// disparaissent sous elle, celles qui portent quelque chose restent. Le clic est pris.
    pub(crate) fn prendre_la_poignee_du_storyboard(&mut self) -> bool {
        let Some((sb, d, coin, prise)) = self.poignee_du_storyboard() else {
            return false;
        };
        let main = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &self.store.viewport());
        if (main.0 - coin.0).abs() > prise || (main.1 - coin.1).abs() > prise {
            return false;
        }
        let vp = self.store.viewport();
        self.ui.placement.storyboard = Some(d.grille);
        self.ui.placement.cible = Some(sb);
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
        let case = self.store.toutes_les_cases(&board).iter().rev().find(|c| {
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
        self.ui.storyboard.courant = Some(case.storyboard.clone());
        self.store.clear_selection();
        if rang >= 2 {
            self.decrire_la_case(&case.id);
            return true;
        }
        let Some(suite) = self.store.disposition_actuelle(&board, &case.storyboard) else {
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
            .toutes_les_cases(board)
            .iter()
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
}

mod ecrire;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_sequences;
