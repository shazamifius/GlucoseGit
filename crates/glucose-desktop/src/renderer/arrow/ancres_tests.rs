//! **Une flèche ancrée à un passage part de ce passage, et s'y vise** (FLECHE-4) — par les vraies
//! pièces de l'application : sa mise en page, son clic, sa saisie.

use crate::app::GlucoseApp;
use glucose_core::arrow::{self as noyau, Noeuds};
use glucose_core::text_anchors::{create_anchor, resolve_text_sel};
use glucose_core::types::{Annotation, TextSelection};

const SA_CARTE: &str = "bonjours\ntest\ntest\nbonjours";

/// Sa carte en `(0, 0)`, et une flèche qui part de son **second** « bonjours » vers un point libre
/// loin à droite, à la hauteur voulue.
fn application(hauteur_visee: f64) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("carte", 0.0, 0.0, SA_CARTE));
    let second = SA_CARTE.rfind("bonjours").expect("le second");
    let mut fleche = Annotation::arrow("f", 0.0, 0.0, 900.0, hauteur_visee);
    if let Annotation::Arrow {
        source_id,
        source_text_sel,
        ..
    } = &mut fleche
    {
        *source_id = Some("carte".to_string());
        *source_text_sel = Some(TextSelection::Anchors(vec![create_anchor(
            SA_CARTE,
            second,
            second + 8,
        )
        .expect("une ancre")]));
    }
    app.store.add_annotation(&board, fleche);
    app.store.clear_selection();
    app.store.journal.clear();
    // Une image, pour que l'index spatial soit celui du tableau.
    app.une_image_sans_fenetre((1200, 800));
    app
}

fn noeuds(app: &GlucoseApp) -> super::NoeudsDuRendu<'_> {
    super::NoeudsDuRendu {
        board: app.store.active_board().expect("un tableau"),
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
    }
}

/// La hauteur du milieu de la quatrième ligne de la carte, depuis son haut.
fn quatrieme_ligne() -> f64 {
    let ligne = f64::from(crate::renderer::card::text_box(240.0).line_height);
    f64::from(crate::renderer::card::TEXT_ORIGIN.1) + 3.5 * ligne
}

/// **La flèche part de la quatrième ligne** — celle du second « bonjours » —, et non du milieu
/// de la carte ; un clic sur son départ la prend.
#[test]
fn test_fleche_4_la_fleche_part_du_passage_et_s_y_vise() {
    let y = quatrieme_ligne();
    let app = application(y);
    let ann = app
        .store
        .project
        .annotation(&app.store.project.active_board_id, "f")
        .expect("la flèche");
    let chemin = noyau::path_with(ann, noeuds(&app)).expect("un chemin");
    assert!(
        (chemin[0].1 - y).abs() < 1e-6,
        "départ en {:?}, attendu à y = {y}",
        chemin[0]
    );
    let boite = noeuds(&app).boite("carte").expect("la carte");
    assert!(
        (boite.center().y - y).abs() > 10.0,
        "le passage n'est pas au milieu : l'épreuve ne distinguerait rien"
    );
    let depart = (chemin[0].0 + 20.0, y);
    let pris = app.pick_candidate_at(depart.0, depart.1).map(|c| c.id);
    assert_eq!(
        pris.as_deref(),
        Some("f"),
        "le clic la prend là où elle part"
    );
}

/// **Valider une saisie qui réécrit le mot ancré garde l'ancre sur ce qu'on a écrit à sa
/// place** — la saisie passe par le noyau, qui fait suivre les ancres dans le même geste.
///
/// C'est le cas qui distingue *suivre* de *retrouver* : sans suivi, l'ancre chercherait encore
/// « bonjours », et retomberait sur le premier — le seul qui reste.
#[test]
fn test_fleche_4_une_saisie_fait_suivre_l_ancre() {
    let mut app = application(0.0);
    let nouveau = "bonjours
test
test
bonsoirs";
    app.start_text_edit("carte".to_string(), nouveau.to_string());
    app.commit_editing();
    let board = app.store.project.active_board_id.clone();
    let Some(Annotation::Arrow {
        source_text_sel, ..
    }) = app.store.project.annotation(&board, "f")
    else {
        panic!("la flèche");
    };
    let plage = resolve_text_sel(nouveau, source_text_sel.as_ref())[0];
    assert_eq!(&nouveau[plage.start..plage.end], "bonsoirs");
}
