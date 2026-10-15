//! Ce que garantissent les signets **sans clavier** (ACCES-1) : l'entrée « Signets… » du menu
//! pose, retrouve et remplace, par les mêmes gestes que `Ctrl+chiffre` et le chiffre seul.

use super::*;
use glucose_core::types::Viewport;

/// La vue posée sur ce point du monde, à cette échelle.
fn regarder(app: &mut GlucoseApp, x: f64, echelle: f64) {
    let tableau = app.store.project.active_board_id.clone();
    app.store.set_viewport(
        &tableau,
        Viewport {
            x,
            y: 0.0,
            scale: echelle,
        },
    );
}

/// Les réponses que la question posée propose.
fn reponses(app: &GlucoseApp) -> Vec<Reponse> {
    let (question, _) = app.ui.question.as_ref().expect("une question posee");
    question.choix.iter().map(|(_, r)| *r).collect()
}

/// **Poser, puis y revenir, sans une touche.** Sans signet, la question n'offre que d'en poser
/// un ; posé, il y paraît, et le toucher fait voler la vue vers lui.
#[test]
fn test_acces_1_les_signets_se_posent_et_se_retrouvent_sans_clavier() {
    let mut app = GlucoseApp::new();
    app.montrer_les_signets();
    assert_eq!(
        reponses(&app),
        vec![Reponse::Oui, Reponse::Annuler],
        "aucun signet : seulement en poser un"
    );
    regarder(&mut app, -300.0, 2.0);
    app.ui.question = None;
    app.suivre_les_signets(Reponse::Oui);
    let ici = app
        .vue_du_signet('1')
        .expect("le premier emplacement libre est pris");
    assert_eq!((ici.x, ici.scale), (-300.0, 2.0));

    app.montrer_les_signets();
    assert!(
        reponses(&app).contains(&Reponse::Choix(0)),
        "le signet 1 se propose"
    );
    regarder(&mut app, 500.0, 0.5);
    app.suivre_les_signets(Reponse::Choix(0));
    assert_eq!(app.vol.cible(), Some(ici), "le toucher vole vers le signet");
}

/// **Les neuf pris, « Poser » demande lequel remplacer** — et celui-là prend la vue d'ici. Au
/// clavier, `Ctrl+chiffre` remplace ; sans clavier, rien ne le pouvait.
#[test]
fn test_acces_1_les_neuf_pris_le_menu_demande_lequel_remplacer() {
    let mut app = GlucoseApp::new();
    for cle in CLES {
        let _ = app.poser_le_signet(cle);
    }
    regarder(&mut app, 777.0, 3.0);
    app.suivre_les_signets(Reponse::Oui);
    let suite = app.ui.question.as_ref().map(|(_, s)| s.clone());
    assert_eq!(
        suite,
        Some(Suite::RemplacerUnSignet),
        "la question suivante"
    );
    app.ui.question = None;
    app.remplacer_le_signet(Reponse::Choix(4));
    let cinq = app.vue_du_signet('5').expect("le signet 5");
    assert_eq!((cinq.x, cinq.scale), (777.0, 3.0), "il ramene ici");
    let un = app.vue_du_signet('1').expect("le signet 1");
    assert_ne!(un.x, 777.0, "les autres n'ont pas bouge");
}

const ECRAN: (f32, f32) = (1440.0, 900.0);

fn aller(app: &mut GlucoseApp, p: (f64, f64)) {
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(p.0, p.1));
}

/// **Le vrai menu, à la vraie souris** : un clic droit sur place l'ouvre, et un clic sur
/// l'entrée nommée la déclenche.
fn par_le_menu(app: &mut GlucoseApp, libelle: &str) {
    use winit::event::MouseButton;
    aller(app, (700.0, 500.0));
    app.handle_mouse_down(MouseButton::Right, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Right);
    let menu = crate::ui::context_menu::layout_context_menu(
        &app.store,
        (&app.renderer.typography, &app.ui.clavier),
        (app.ui.context_menu_at.expect("un menu ouvert"), None),
        ECRAN,
        app.ui.scale_factor,
        false,
    )
    .expect("un menu");
    let rect = menu
        .rows
        .iter()
        .find_map(|r| match r {
            crate::ui::context_menu::MenuRow::Item { label, rect, .. } if *label == libelle => {
                Some(*rect)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("« {libelle} » manque au menu : {:?}", menu.rows));
    aller(
        app,
        (
            f64::from(rect.0 + rect.2 / 2.0),
            f64::from(rect.1 + rect.3 / 2.0),
        ),
    );
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
}

/// **ACCES-1 — chaque entrée neuve fait ce que fait sa touche**, passée par le vrai menu :
/// « Signets… » pose sa question, « Annuler » défait le dernier geste, « Rétablir » le refait.
#[test]
fn test_acces_1_les_entrees_neuves_font_ce_que_fait_leur_touche() {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    // Loin de tout nœud, rien de choisi — la carte d'accueil naît choisie : le clic droit
    // tombe sur le vide, et ouvre le menu du vide.
    regarder(&mut app, 1.0e6, 1.0);
    app.store.clear_selection();
    par_le_menu(&mut app, "Signets…");
    assert_eq!(
        app.ui.question.as_ref().map(|(_, s)| s.clone()),
        Some(Suite::Signets)
    );
    app.ui.question = None;

    let compte = |app: &GlucoseApp| app.store.active_board().map_or(0, |b| b.annotations.len());
    let avant = compte(&app);
    let tableau = app.store.project.active_board_id.clone();
    app.store.add_annotation(
        &tableau,
        glucose_core::types::Annotation::text("t", -2000.0, -2000.0, "un geste"),
    );
    app.store.clear_selection();
    assert_eq!(compte(&app), avant + 1);
    par_le_menu(&mut app, "Annuler");
    assert_eq!(compte(&app), avant, "« Annuler » a defait le geste");
    par_le_menu(&mut app, "Rétablir");
    assert_eq!(compte(&app), avant + 1, "« Retablir » l'a refait");
}
