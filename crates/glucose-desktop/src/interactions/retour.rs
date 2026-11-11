//! **RETOUR-1 — revenir où l'on était** (fiche 99) : `Alt+Z`, et `Alt+Maj+Z` pour repartir.
//!
//! Sa parole du 9/11/2026 : *« tu focus sur un bloc qu'il ne fallait pas, Alt+Z, et tu es de
//! retour instantanément à ton lieu d'origine »*. C'est le retour d'un navigateur, et les touches
//! `[` et `]` de Maya (« Undo View Change », « Redo View Change »).
//!
//! # Ce qu'on retient : le lieu qu'on quitte à chaque départ
//!
//! Tout saut de la caméra — `F`, tout cadrer, un signet, un résultat de `Ctrl+F`, la place d'un
//! domaine, le mode Focus, la minimap, le titre qu'on va écrire (SAISIE-2) — est un **départ** du
//! vol ([`super::vol::Vol`]). Le vol note qu'il part ; l'image suivante, qui connaît la vue d'où
//! il part, la range ici. Un seul endroit, quel que soit le geste qui a demandé le saut : un geste
//! ajouté demain qui fait voler la vue sera retenu sans que personne y pense.
//!
//! **RETOUR-2 — et chaque geste de la main** (fiche 102 ; sa parole du 10/11 au soir : *« c'est
//! vraiment juste le Ctrl+Z, mais pour la caméra »*) : un tour de molette, un glisser, un pincement
//! — la vue qui quitte son lieu y laisse un lieu où revenir ([`GlucoseApp`], le suivi du
//! mouvement). Deux mouvements séparés de moins qu'un double-clic sont un seul pas, et la main qui
//! tient la vue le garde ouvert (fiche 103 § 2 : la première loi, « le premier arrêt ferme le
//! pas », faisait cinq pas de cinq crans). Un retour ferme le pas. La fiche 99 disait le contraire
//! (« on ne revient pas d'un tour de molette ») : sa parole l'a tranché.
//!
//! # Le retour est instantané, et n'est pas un départ
//!
//! Son mot : « instantanément ». La vue se pose, sans vol — et ne se note donc pas comme un départ :
//! revenir ne remplit pas ce qu'on remonte. Une navigation neuve, elle, efface l'avant.
//!
//! La navigation n'entre jamais dans l'annulation du document (le magasin, § 3) : `Ctrl+Z` défait
//! un geste sur le document, `Alt+Z` un déplacement de la vue.
//!
//! L'histoire elle-même vit dans le magasin ([`glucose_core::store::retour`]), à côté de la vue de
//! chaque tableau : un document remplacé l'emporte avec lui, et le menu sait s'il y a où revenir.

use crate::app::GlucoseApp;
use glucose_core::store::retour::Lieu;

impl GlucoseApp {
    /// **`Alt+Z`** (en arrière) ou **`Alt+Maj+Z`** (en avant) : la vue se pose au lieu précédent ou
    /// suivant. Rend `false` s'il n'y en a pas — la commande ne s'applique pas.
    pub(crate) fn revenir(&mut self, en_arriere: bool) -> bool {
        let Some(ici) = self.lieu_ou_l_on_est() else {
            return false;
        };
        loop {
            let la = if en_arriere {
                self.store.histoire_de_la_vue.reculer(ici.clone())
            } else {
                self.store.histoire_de_la_vue.avancer(ici.clone())
            };
            // Rien où aller : rien ne bouge, comme le retour d'un navigateur sans page d'avant —
            // et le menu n'offre l'entrée que s'il y a où revenir. Pas de message : un message ne
            // dit que ce qui vient d'avoir lieu (le cliquet des toasts).
            let Some(la) = la else {
                return false;
            };
            // Un tableau retiré depuis — un onglet fermé, un dossier supprimé — ne se rejoint
            // plus : on passe au lieu d'avant. Sur le tableau où l'on est, on n'en change pas :
            // changer de tableau vide la sélection, et revenir ne défait pas ce qu'on a choisi.
            let ailleurs = self.store.project.active_board_id != la.tableau;
            if ailleurs
                && self
                    .store
                    .try_set_active_board_id(la.tableau.clone())
                    .is_err()
            {
                continue;
            }
            self.vol.poser();
            self.fermer_le_pas_de_la_camera();
            self.store.set_viewport(&la.tableau, la.vue);
            self.mark_dirty();
            return true;
        }
    }

    fn lieu_ou_l_on_est(&self) -> Option<Lieu> {
        let tableau = self.store.active_board()?;
        Some(Lieu {
            tableau: tableau.id.clone(),
            vue: tableau.viewport,
        })
    }
}

#[cfg(test)]
#[path = "retour/camera_tests.rs"]
mod camera_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use glucose_core::types::Viewport;

    fn vue(x: f64) -> Viewport {
        Viewport {
            x,
            y: 0.0,
            scale: 1.0,
        }
    }

    /// Une application au tableau vide, l'accueil éteint, la vue posée en `x = 0`.
    fn app() -> GlucoseApp {
        let mut app = GlucoseApp::new();
        app.ui.current_toast = None;
        if let Some(b) = app.store.active_board_mut() {
            b.annotations.clear();
        }
        let tableau = app.store.project.active_board_id.clone();
        app.store.set_viewport(&tableau, vue(0.0));
        app
    }

    /// Une image comme la boucle la joue, présentée une période d'écran à 240 Hz après la
    /// précédente : c'est le temps qui passe qui fait voler.
    fn image(app: &mut GlucoseApp) {
        let periode = std::time::Duration::from_micros(4_167);
        app.horloge.accorder(periode);
        let t = app
            .horloge
            .derniere_presentation()
            .unwrap_or_else(std::time::Instant::now);
        app.horloge.presentee(t + periode, true);
        app.appliquer_l_elan(800, 600);
    }

    /// Un vol jusqu'au bout, image par image.
    fn voler(app: &mut GlucoseApp, x: f64) {
        app.vol.voler_vers(vue(x));
        for _ in 0..2_000 {
            image(app);
            if !app.vol.en_cours() {
                return;
            }
        }
        panic!("le vol n'arrive pas");
    }

    fn x(app: &GlucoseApp) -> f64 {
        app.store.viewport().x
    }

    /// **Un saut, `Alt+Z`, et l'on est de retour, instantanément** — puis `Alt+Maj+Z` repart là où
    /// le saut avait mené. Un déplacement que le modèle pose sans geste (`Store::pan`) n'est pas un
    /// pas : on revient au lieu d'origine. (Un geste de la main en est un, RETOUR-2.)
    #[test]
    fn retour_1_un_saut_puis_alt_z_ramene_au_lieu_d_origine() {
        let mut app = app();
        let tableau = app.store.project.active_board_id.clone();
        let texte = glucose_core::types::Annotation::text("t", 0.0, 0.0, "Eau");
        app.store.add_annotation(&tableau, texte);
        app.store.select_annotation("t".into(), false);
        voler(&mut app, 500.0);
        assert!(
            (x(&app) - 500.0).abs() < 1.0,
            "le vol est arrivé : {}",
            x(&app)
        );
        app.store.pan(30.0, 0.0);
        let apres = x(&app);
        assert!(app.revenir(true));
        assert_eq!(x(&app), 0.0, "de retour au lieu d'origine, au bit près");
        assert!(!app.vol.en_cours(), "instantanément : sans vol");
        assert!(
            app.store.selected_annotation_ids.contains("t"),
            "revenir ne défait pas la sélection"
        );
        assert!(app.revenir(false));
        assert_eq!(
            x(&app),
            apres,
            "repartir rend la vue qu'on avait en revenant"
        );
    }

    /// **Les retours remontent les sauts un à un, et une navigation neuve efface l'avant** — comme
    /// un navigateur. Revenir ne se note pas comme un départ.
    #[test]
    fn retour_1_les_sauts_se_remontent_et_un_depart_neuf_efface_l_avant() {
        let mut app = app();
        voler(&mut app, 100.0);
        voler(&mut app, 200.0);
        assert!(app.revenir(true));
        assert_eq!(x(&app), 100.0);
        assert!(app.revenir(true));
        assert_eq!(x(&app), 0.0);
        assert!(!app.revenir(true), "rien avant le premier lieu");
        assert!(app.revenir(false));
        assert_eq!(x(&app), 100.0, "revenir n'a rien noté de plus");
        voler(&mut app, 300.0);
        assert!(!app.revenir(false), "un départ neuf efface l'avant");
        assert!(app.revenir(true));
        assert_eq!(x(&app), 100.0);
    }

    /// **Un vol redirigé en route ne note que son premier départ** : en plein vol, on n'est nulle
    /// part qu'on voudrait retrouver.
    #[test]
    fn retour_1_un_vol_redirige_en_route_ne_note_que_son_depart() {
        let mut app = app();
        app.vol.voler_vers(vue(1000.0));
        image(&mut app);
        image(&mut app);
        voler(&mut app, 400.0);
        assert!(app.revenir(true));
        assert_eq!(x(&app), 0.0, "le lieu d'avant le premier départ");
        assert!(!app.revenir(true), "et lui seul");
    }

    /// Une touche jouée par le vrai chemin des raccourcis, ses modificateurs tenus.
    fn touche(app: &mut GlucoseApp, c: &str, mods: winit::keyboard::ModifiersState) {
        app.modifiers = mods;
        app.handle_shortcut_input(
            &winit::keyboard::Key::Character(c.into()),
            winit::event::ElementState::Pressed,
        );
    }

    /// **`Alt+Z` revient, `Alt+Maj+Z` repart** — au clavier, par la table des raccourcis.
    #[test]
    fn retour_1_alt_z_et_alt_maj_z_au_clavier() {
        use winit::keyboard::ModifiersState;
        let mut app = app();
        voler(&mut app, 500.0);
        touche(&mut app, "z", ModifiersState::ALT);
        assert_eq!(x(&app), 0.0, "Alt+Z revient");
        touche(&mut app, "Z", ModifiersState::ALT | ModifiersState::SHIFT);
        assert!(
            (x(&app) - 500.0).abs() < 1.0,
            "Alt+Maj+Z repart : {}",
            x(&app)
        );
    }

    /// Les libellés du menu ouvert sur le vide.
    fn menu_du_vide(app: &GlucoseApp) -> Vec<String> {
        crate::ui::context_menu::layout_context_menu(
            &app.store,
            (&app.renderer.typography, &app.ui.clavier),
            ((400.0, 300.0), None),
            (800.0, 600.0),
            1.0,
            false,
        )
        .expect("un menu")
        .rows
        .iter()
        .filter_map(|r| match r {
            crate::ui::context_menu::MenuRow::Item { label, .. } => Some(label.to_string()),
            _ => None,
        })
        .collect()
    }

    /// **Au doigt, il n'y a pas d'`Alt+Z`** : le menu du vide porte « Vue précédente » — seulement
    /// quand il y a où revenir, comme « Annuler ».
    #[test]
    fn retour_1_le_menu_du_vide_porte_la_vue_precedente_quand_il_y_a_ou_revenir() {
        let mut app = app();
        let vue_precedente =
            |app: &GlucoseApp| menu_du_vide(app).iter().any(|l| l == "Vue précédente");
        assert!(!vue_precedente(&app), "rien où revenir : pas d'entrée");
        voler(&mut app, 500.0);
        assert!(vue_precedente(&app), "après un saut, l'entrée existe");
    }

    /// **La minimap est un départ** — on la maintient pour voyager : on revient d'où l'on était.
    #[test]
    fn retour_1_un_voyage_par_la_minimap_se_remonte() {
        let mut app = app();
        app.vol.viser(vue(300.0));
        for _ in 0..2_000 {
            image(&mut app);
        }
        assert!(
            (x(&app) - 300.0).abs() < 1.0,
            "la minimap a mené : {}",
            x(&app)
        );
        assert!(app.revenir(true));
        assert_eq!(x(&app), 0.0);
    }

    /// **`Alt+Z` en plein vol arrête le vol** : la vue reste où elle revient, au bit près.
    #[test]
    fn retour_1_alt_z_en_plein_vol_arrete_le_vol() {
        let mut app = app();
        app.vol.voler_vers(vue(800.0));
        image(&mut app);
        image(&mut app);
        assert!(app.revenir(true));
        for _ in 0..50 {
            image(&mut app);
        }
        assert_eq!(x(&app), 0.0, "le vol ne reprend pas la vue");
    }

    /// **Un lieu dont le tableau a disparu se saute** — un onglet fermé depuis : on revient au lieu
    /// d'avant.
    #[test]
    fn retour_1_un_lieu_sur_un_tableau_disparu_se_saute() {
        let mut app = app();
        let tableau = app.store.project.active_board_id.clone();
        let fantome = String::from("un-tableau-ferme");
        app.store.histoire_de_la_vue.noter(Lieu {
            tableau,
            vue: vue(7.0),
        });
        app.store.histoire_de_la_vue.noter(Lieu {
            tableau: fantome,
            vue: vue(9.0),
        });
        assert!(app.revenir(true));
        assert_eq!(x(&app), 7.0);
    }
}
