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
//!
//! **Pendant qu'on écrit, la ligne du curseur** (CLAVIER-3, fiche 57 ; SUIVRE-1, fiche 134 § 15) :
//! [`GlucoseApp::garder_la_ligne_en_vue`] — la même loi du plus petit décalage, pour la seule ligne
//! où l'on écrit, après chaque commande d'écriture.

use crate::app::GlucoseApp;
use crate::renderer::richtext::hit::{line_of_offset, offset_to_x};
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

impl GlucoseApp {
    /// **La ligne qu'on écrit reste en vue** (CLAVIER-3) : quand le clavier du téléphone sort, la
    /// fenêtre rétrécit (`adjustResize`), et ce qu'on écrit pouvait passer dessous. **Au bureau
    /// aussi** (SUIVRE-1, sa note `text-wiov-325` : *« quand tu écris et que ça dépasse ton écran,
    /// il faut se déplacer juste pour pouvoir voir ce qu'on est en train d'écrire »*) : chaque
    /// commande d'écriture la demande — la frappe, un collage, les flèches, `Ctrl+Z`.
    ///
    /// La vue bouge du moins possible — zéro si la ligne se voit déjà —, à la même échelle, sans
    /// entrer dans l'histoire de la caméra (ce n'est pas un geste) : en hauteur, **la ligne
    /// entière** ; en largeur, **le curseur, avec la hauteur de sa ligne d'air** de chaque côté —
    /// ce qu'on va taper se voit, et aucune marge n'est à choisir (SAISIE-2). Une ligne plus haute
    /// que ce qui reste montre son haut, là où le curseur commence.
    pub(crate) fn garder_la_ligne_en_vue(&mut self) {
        let Some(session) = self.editing_session.as_ref() else {
            return;
        };
        let (Some(boite), Some((mise_en_page, corps))) =
            (self.card_box(&session.ann_id), self.editing_layout())
        else {
            return;
        };
        let rang = line_of_offset(&mise_en_page, session.selection.head);
        let x_du_curseur = mise_en_page.lines.get(rang).map_or(0.0, |ligne| {
            let police = crate::renderer::richtext::font_of(ligne.kind, corps.body);
            let (typo, tampon) = (&self.renderer.typography, &session.buffer);
            offset_to_x(
                typo,
                &mise_en_page,
                ligne,
                tampon,
                session.selection.head,
                police,
            )
        });
        let hauteur = f64::from(corps.line_height);
        let Some(board) = self.store.active_board() else {
            return;
        };
        let (mut vue, id) = (board.viewport, board.id.clone());
        let s = vue.scale;
        let haut = (boite.origin.1 + f64::from(boite.marge.1) + rang as f64 * hauteur) * s + vue.y;
        let x = (boite.origin.0 + f64::from(boite.marge.0 + x_du_curseur)) * s + vue.x;
        let air = hauteur * s;
        let plafond = f64::from(self.ui.header_height());
        let (droite, plancher) = self.ce_qui_se_voit();
        let dy = rapprocher((haut, haut + air), (plafond, f64::from(plancher)));
        let dx = rapprocher((x - air, x + air), (0.0, f64::from(droite)));
        if dx != 0.0 || dy != 0.0 {
            vue.x += dx;
            vue.y += dy;
            self.store.set_viewport(&id, vue);
            self.mark_dirty();
        }
    }
}

impl GlucoseApp {
    /// **Ce qui se voit du canevas, à droite et en bas** : la fenêtre, moins le clavier et les
    /// marges du système (BORD-1), et moins les barres que la sélection fait naître en bas —
    /// pendant qu'on écrit, le nœud est choisi, et la barre de couleur, la plus haute, couvre le
    /// bas de l'écran (vu au pilote : la dernière ligne s'arrêtait dessous, à demi cachée).
    pub(crate) fn ce_qui_se_voit(&self) -> (f32, f32) {
        let (l, h) = self.taille_de_la_fenetre();
        let visible = self.ui.ecran_visible((l, h));
        let echelle = self.screen_frame(l, h).scale;
        let typo = &self.renderer.typography;
        let barre =
            crate::ui::couleurs::layout_barre_de_couleur(&self.store, typo, visible, echelle);
        (
            visible.0,
            barre.map_or(visible.1, |b| visible.1.min(b.rect.1)),
        )
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

    // ── SUIVRE-1 — au bureau, la ligne qu'on écrit reste en vue ─────────────────────────────

    /// Un texte de 240 de large en `(x, y)`, en saisie au bout de « Un », sous la vue `v`.
    fn en_saisie((x, y): (f64, f64), v: Viewport) -> GlucoseApp {
        let mut app = GlucoseApp::new();
        app.ui.current_toast = None;
        let tableau = app.store.project.active_board_id.clone();
        if let Some(b) = app.store.active_board_mut() {
            b.annotations.clear();
        }
        let mut t = Annotation::text("t", x, y, "Un");
        if let Annotation::Text { width, height, .. } = &mut t {
            (*width, *height) = (Some(240.0), Some(60.0));
        }
        app.store.add_annotation(&tableau, t);
        app.store.set_viewport(&tableau, v);
        app.start_text_edit("t".into(), "Un".into());
        app
    }

    /// La frappe, lettre après lettre, par le point unique où toute écriture passe — celui où la
    /// touche mène (un `KeyEvent` de winit ne se construit pas hors de la boucle).
    fn taper(app: &mut GlucoseApp, texte: &str) {
        use crate::interactions::text_edit::keys::Command;
        for c in texte.chars() {
            app.apply_text_command(Command::Insert(c.to_string()), false);
        }
    }

    /// Le haut et le bas de la ligne du curseur à l'écran, et l'abscisse du curseur, sous `v`.
    fn ligne_a_l_ecran(app: &GlucoseApp, v: Viewport) -> (f64, f64, f64) {
        use crate::renderer::card::TEXT_ORIGIN;
        use crate::renderer::richtext::{font_of, hit};
        let session = app.editing_session.as_ref().expect("en saisie");
        let boite = app.card_box("t").expect("sa boîte");
        let (mise, corps) = app.editing_layout().expect("sa mise en page");
        let rang = hit::line_of_offset(&mise, session.selection.head);
        let ligne = &mise.lines[rang];
        let typo = &app.renderer.typography;
        let police = font_of(ligne.kind, corps.body);
        let x = hit::offset_to_x(
            typo,
            &mise,
            ligne,
            &session.buffer,
            session.selection.head,
            police,
        );
        let h = f64::from(corps.line_height);
        let haut = (boite.origin.1 + f64::from(TEXT_ORIGIN.1) + rang as f64 * h) * v.scale + v.y;
        let gauche = (boite.origin.0 + f64::from(TEXT_ORIGIN.0 + x)) * v.scale + v.x;
        (haut, haut + h * v.scale, gauche)
    }

    /// Une phrase qui, dans un texte de 240, descend de huit lignes.
    const PHRASE: &str = " texte qui grandit, mot après mot, ligne après ligne, jusqu'à passer \
                          sous la barre de couleur et le bas de l'écran : la vue doit suivre la \
                          ligne où l'on écrit, sans changer d'échelle.";

    fn vue_de(app: &GlucoseApp) -> Viewport {
        app.store.active_board().expect("un tableau").viewport
    }

    /// **Écrire sous le bas de l'écran fait monter la vue** (sa note `text-wiov-325` : *« quand tu
    /// écris et que ça dépasse ton écran, il faut se déplacer »*) : la ligne du curseur reste
    /// au-dessus du bas, à la même échelle. **L'envers** : sous la vue d'avant, cette ligne était
    /// sous le bas de la fenêtre — c'est ce qu'il voyait.
    #[test]
    fn suivre_1_ecrire_sous_le_bas_fait_monter_la_vue() {
        let avant = vue(0.0, 0.0, 1.0);
        let mut app = en_saisie((100.0, 700.0), avant);
        taper(&mut app, PHRASE);
        let plancher = f64::from(app.ce_qui_se_voit().1);
        let (_, bas_avant, _) = ligne_a_l_ecran(&app, avant);
        assert!(
            bas_avant > plancher,
            "l'envers : cachée sous la vue d'avant"
        );
        let fenetre = f64::from(app.ui.ecran_visible(app.taille_de_la_fenetre()).1);
        assert!(
            plancher < fenetre,
            "la barre de couleur couvre le bas pendant qu'on écrit"
        );
        let premiere = 700.0 + f64::from(crate::renderer::card::TEXT_ORIGIN.1);
        assert!(premiere < plancher, "on avait commencé à l'écran");
        let apres = vue_de(&app);
        assert_eq!(
            (apres.x, apres.scale),
            (0.0, 1.0),
            "ni l'abscisse ni l'échelle"
        );
        let (haut, bas, _) = ligne_a_l_ecran(&app, apres);
        assert!(
            (bas - plancher).abs() < 1e-6,
            "la ligne juste au-dessus du bas : {bas} contre {plancher}"
        );
        assert!(haut >= f64::from(app.ui.header_height()));
    }

    /// **Écrire au-delà du bord droit, de près, fait glisser la vue** : le curseur garde la hauteur
    /// de sa ligne d'air avant le bord. **L'envers** : sous la vue d'avant, il était dehors.
    #[test]
    fn suivre_1_ecrire_au_dela_du_bord_droit_fait_glisser_la_vue() {
        let avant = vue(-200.0, -900.0, 4.0);
        let mut app = en_saisie((300.0, 300.0), avant);
        taper(&mut app, " mot de plus");
        let droite = f64::from(app.ce_qui_se_voit().0);
        let air = f64::from(crate::renderer::card::text_box(240.0).line_height) * 4.0;
        let (_, _, x_avant) = ligne_a_l_ecran(&app, avant);
        assert!(
            x_avant + air > droite,
            "l'envers : dehors sous la vue d'avant"
        );
        let apres = vue_de(&app);
        assert_eq!(
            (apres.y, apres.scale),
            (-900.0, 4.0),
            "ni la hauteur ni l'échelle"
        );
        let (_, _, x) = ligne_a_l_ecran(&app, apres);
        assert!(
            (x + air - droite).abs() < 1e-6,
            "le curseur à sa ligne d'air du bord : {x}"
        );
    }

    /// **`Ctrl+Z` pendant l'écriture ramène aussi la ligne** : on a regardé ailleurs, on défait le
    /// dernier mot — la vue revient là où l'on écrit. **L'envers** : sous la vue où l'on était allé
    /// regarder, cette ligne était cachée.
    #[test]
    fn suivre_1_ctrl_z_ramene_la_ligne_qu_on_ecrit() {
        use crate::interactions::text_edit::keys::Command;
        let avant = vue(0.0, 0.0, 1.0);
        let mut app = en_saisie((100.0, 700.0), avant);
        taper(&mut app, PHRASE);
        let tableau = app.store.project.active_board_id.clone();
        app.store.set_viewport(&tableau, avant);
        app.apply_text_command(Command::Undo, false);
        let plancher = f64::from(app.ce_qui_se_voit().1);
        let (_, bas_avant, _) = ligne_a_l_ecran(&app, avant);
        assert!(
            bas_avant > plancher,
            "l'envers : cachée là où l'on regardait"
        );
        let (_, bas, _) = ligne_a_l_ecran(&app, vue_de(&app));
        assert!(
            (bas - plancher).abs() < 1e-6,
            "revenue au-dessus du bas : {bas}"
        );
    }
}
