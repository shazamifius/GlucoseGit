//! **SAISIE-2 — ce qu'on écrit est à l'écran** (fiche 99).
//!
//! `Ctrl+G` entoure ce qu'on a choisi d'une membrane neuve, avec la place de son titre au-dessus.
//! Quand ce qu'on avait choisi touchait le haut de l'écran — le cas ordinaire : on choisit ce qu'on
//! voit —, cette place tombait **sous la barre des onglets**, et le double-clic qui la nomme ouvrait
//! une saisie qu'on ne voyait pas. Tout éditeur ramène le curseur à l'écran ; ici, quand une saisie
//! s'ouvre sur un titre caché, la vue glisse **du strict nécessaire** pour le montrer, à la même
//! échelle, par le vol de `F` et des signets.
//!
//! Le titre d'une membrane seulement : un texte et un post-it s'ouvrent là où l'on vient de
//! cliquer, donc à l'écran — et les ramener entiers à l'écran ferait sauter la vue sur un long
//! texte dont on a double-cliqué un mot du milieu.

use crate::app::GlucoseApp;
use glucose_core::geometry::Rect;
use glucose_core::types::{Annotation, Viewport};

impl GlucoseApp {
    /// Si la saisie qui s'ouvre écrit un titre que l'écran ne montre pas en entier, la vue y
    /// glisse.
    pub(super) fn montrer_ce_qu_on_ecrit(&mut self, id: &str) {
        let tableau = self.store.project.active_board_id.clone();
        let Some(boite) = self
            .store
            .project
            .annotation(&tableau, id)
            .filter(|a| matches!(a, Annotation::Membrane { .. }))
            .map(Annotation::bounds)
        else {
            return;
        };
        let (l, h) = self.taille_de_la_fenetre();
        let canevas = (
            f64::from(l),
            f64::from(self.ui.header_height()),
            f64::from(h),
        );
        let vue = self.store.viewport();
        if let Some(cible) =
            vue_qui_montre(glucose_core::membrane_titre::bande(boite), vue, canevas)
        {
            self.vol.voler_vers(cible);
            self.mark_dirty();
        }
    }
}

/// **La vue la plus proche de `vue` qui montre `lieu`** (en unités monde) dans le canevas — sa
/// largeur, le bas du bandeau, sa hauteur, en pixels —, à la même échelle ; `None` si `vue` le
/// montre déjà.
///
/// Le lieu garde autour de lui **sa propre hauteur** d'air : un titre collé au bandeau se lit mal,
/// et aucune marge n'est à choisir — c'est la hauteur du titre qui la donne. Un lieu plus grand
/// que le canevas en montre le début, en haut à gauche : là où un titre commence.
fn vue_qui_montre(
    lieu: Rect,
    vue: Viewport,
    (largeur, bandeau, hauteur): (f64, f64, f64),
) -> Option<Viewport> {
    let s = vue.scale;
    let air = lieu.height * s;
    let gauche = lieu.left * s + vue.x - air;
    let haut = lieu.top * s + vue.y - air;
    let (l, h) = (lieu.width * s + 2.0 * air, lieu.height * s + 2.0 * air);
    let dx = rapprocher((gauche, gauche + l), (0.0, largeur));
    let dy = rapprocher((haut, haut + h), (bandeau, hauteur));
    (dx != 0.0 || dy != 0.0).then_some(Viewport {
        x: vue.x + dx,
        y: vue.y + dy,
        scale: s,
    })
}

/// **Le plus petit décalage qui fait entrer `[a, b]` dans `[debut, fin]`** — ou qui pose `a` sur
/// `debut` quand l'intervalle est plus long que la place.
fn rapprocher((a, b): (f64, f64), (debut, fin): (f64, f64)) -> f64 {
    if a < debut || b - a > fin - debut {
        debut - a
    } else if b > fin {
        fin - b
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::GlucoseApp;

    const CANEVAS: (f64, f64, f64) = (1000.0, 80.0, 700.0);

    fn vue(x: f64, y: f64, scale: f64) -> Viewport {
        Viewport { x, y, scale }
    }

    /// **Un titre caché sous le bandeau descend juste assez** : il gagne sa hauteur d'air sous le
    /// bandeau, et rien d'autre ne bouge — ni l'abscisse, ni l'échelle.
    #[test]
    fn saisie_2_un_titre_sous_le_bandeau_descend_juste_assez() {
        let lieu = Rect::new(100.0, -20.0, 400.0, 20.0);
        let cible = vue_qui_montre(lieu, vue(0.0, 60.0, 1.0), CANEVAS).expect("la vue glisse");
        assert_eq!((cible.x, cible.scale), (0.0, 1.0));
        let haut_a_l_ecran = lieu.top * cible.scale + cible.y;
        assert!(
            (haut_a_l_ecran - (80.0 + 20.0)).abs() < 1e-9,
            "le titre sous le bandeau, à sa hauteur d'air : {haut_a_l_ecran}"
        );
    }

    /// **Un titre qu'on voit ne bouge pas la vue** : on vient de double-cliquer dessus.
    #[test]
    fn saisie_2_un_titre_a_l_ecran_ne_bouge_pas_la_vue() {
        let lieu = Rect::new(100.0, 200.0, 400.0, 20.0);
        assert_eq!(vue_qui_montre(lieu, vue(0.0, 0.0, 1.0), CANEVAS), None);
    }

    /// **Hors de l'écran à droite, en bas, ou plus large que lui** : le lieu entre par le plus court
    /// chemin ; trop large, il montre son début.
    #[test]
    fn saisie_2_le_plus_court_chemin_et_le_debut_d_un_titre_trop_large() {
        let a_droite = Rect::new(1100.0, 300.0, 200.0, 20.0);
        let c = vue_qui_montre(a_droite, vue(0.0, 0.0, 1.0), CANEVAS).expect("glisse");
        assert!((1100.0 + 200.0 + 20.0 + c.x - 1000.0).abs() < 1e-9 && c.y == 0.0);
        let en_bas = Rect::new(100.0, 690.0, 200.0, 20.0);
        let c = vue_qui_montre(en_bas, vue(0.0, 0.0, 1.0), CANEVAS).expect("glisse");
        assert!((690.0 + 20.0 + 20.0 + c.y - 700.0).abs() < 1e-9 && c.x == 0.0);
        // Trop large et commencé à l'écran : son début vient au bord, pas sa fin.
        let commence_a_l_ecran = Rect::new(100.0, 300.0, 3000.0, 20.0);
        let c = vue_qui_montre(commence_a_l_ecran, vue(0.0, 0.0, 1.0), CANEVAS).expect("glisse");
        assert!(
            (c.x + 80.0).abs() < 1e-9,
            "son début au bord gauche : {}",
            c.x
        );
        let trop_large = Rect::new(-500.0, 300.0, 3000.0, 20.0);
        let c = vue_qui_montre(trop_large, vue(0.0, 0.0, 1.0), CANEVAS).expect("glisse");
        assert!(
            (-500.0 - 20.0 + c.x).abs() < 1e-9,
            "son début au bord gauche : {}",
            c.x
        );
    }

    /// Une application dont le seul nœud est `noeud`, son bord haut **sur** le bas du bandeau.
    fn collee_au_bandeau(noeud: Annotation) -> GlucoseApp {
        let mut app = GlucoseApp::new();
        app.ui.current_toast = None;
        if let Some(b) = app.store.active_board_mut() {
            b.annotations.clear();
        }
        let tableau = app.store.project.active_board_id.clone();
        app.store.add_annotation(&tableau, noeud);
        let bandeau = f64::from(app.ui.header_height());
        app.store.set_viewport(&tableau, vue(40.0, bandeau, 1.0));
        app
    }

    /// **Nommer une membrane neuve dont le titre est sous le bandeau fait glisser la vue** — le cas
    /// de `Ctrl+G` sur ce qu'on voit en haut de l'écran : à l'arrivée, le titre est sous le bandeau.
    #[test]
    fn saisie_2_nommer_une_membrane_au_titre_cache_fait_glisser_la_vue() {
        let mut app = collee_au_bandeau(Annotation::membrane("m", 0.0, 0.0, 480.0, 240.0));
        app.start_text_edit("m".into(), String::new());
        let cible = app.vol.cible().expect("la vue glisse vers le titre");
        let bande = glucose_core::membrane_titre::bande(Rect::new(0.0, 0.0, 480.0, 240.0));
        let haut = bande.top * cible.scale + cible.y;
        assert!(
            haut >= f64::from(app.ui.header_height()),
            "à l'arrivée, le titre est sous le bandeau : {haut}"
        );
    }

    /// **Un texte s'écrit là où on l'a ouvert** : collé au bandeau lui aussi, il ne fait pas glisser
    /// la vue — on vient de double-cliquer un de ses mots.
    #[test]
    fn saisie_2_un_texte_ne_fait_pas_glisser_la_vue() {
        let mut app = collee_au_bandeau(Annotation::text("t", 0.0, -10.0, "Un texte"));
        app.start_text_edit("t".into(), String::new());
        assert_eq!(app.vol.cible(), None);
    }
}
