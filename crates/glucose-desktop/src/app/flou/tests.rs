//! Les épreuves du flou côté application (Q12, fiche 137) : quand il vit, son réglage, et l'image
//! entière qu'il demande.

use super::*;
use crate::commandes::Commande;
use crate::persist::disque::tests::{application, dossier};
use glucose_core::types::{Annotation, Viewport};

/// Une application dont le tableau porte un texte choisi et un texte témoin, vus au zoom 1.
fn application_choisie(nom: &str) -> GlucoseApp {
    let mut app = application(&dossier(nom));
    let board = app.store.project.boards.first_mut().expect("un tableau");
    board.viewport = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    board.annotations.clear();
    let mut choisi = Annotation::text("n", 300.0, 260.0, "Le col du Lautaret");
    let mut temoin = Annotation::text("t", 80.0, 470.0, "Un témoin jamais choisi");
    for t in [&mut choisi, &mut temoin] {
        if let Annotation::Text { width, height, .. } = t {
            (*width, *height) = (Some(260.0), Some(120.0));
        }
    }
    board.annotations.extend([choisi, temoin]);
    app.store.selected_annotation_ids.push("n".into());
    app
}

/// **Le flou vit la main au repos, et s'efface pendant un geste** : un glisser, un rectangle de
/// sélection, un texte qu'on écrit.
#[test]
fn test_q12_le_flou_s_efface_pendant_un_geste() {
    let mut app = application_choisie("flou-geste");
    assert!(app.le_flou_vit(), "au repos, le flou vit");
    app.is_dragging_item = true;
    assert!(!app.le_flou_vit(), "pendant un glisser, il s'efface");
    app.is_dragging_item = false;
    app.selection_box = Some((0.0, 0.0, 10.0, 10.0));
    assert!(
        !app.le_flou_vit(),
        "pendant le rectangle de sélection aussi"
    );
    app.selection_box = None;
    app.start_text_edit("n".into(), "Le col".into());
    assert!(!app.le_flou_vit(), "pendant qu'on écrit aussi");
}

/// **Le réglage l'éteint, le menu dit l'autre geste, et le choix se retient** d'une session à
/// l'autre.
#[test]
fn test_q12_le_reglage_l_eteint_et_se_retient() {
    let mut app = application_choisie("flou-reglage");
    assert!(app.s_applique(Commande::Flou(false)));
    assert!(!app.s_applique(Commande::Flou(true)));
    assert!(app.executer(Commande::Flou(false)));
    assert!(!app.le_flou_vit(), "éteint, il ne vit plus");
    assert!(
        app.s_applique(Commande::Flou(true)),
        "le menu offre de le rallumer"
    );
    assert!(!app.flou_retenu(), "le choix se retient");
    app.executer(Commande::Flou(true));
    assert!(app.le_flou_vit() && app.flou_retenu());
}

/// **Tant que le flou vit, l'image se repeint entière** : une région salie seule — le témoin
/// qu'une IA aurait retouché — se peindrait nette dans un écran flouté, ou se flouterait sur des
/// pixels déjà floutés. L'image d'après une salissure locale est celle d'un rendu entier.
#[test]
fn test_q12_tant_que_le_flou_vit_l_image_se_repeint_entiere() {
    let taille = (900, 700);
    let mut app = application_choisie("flou-entiere");
    app.une_image_sans_fenetre(taille);
    app.salir(glucose_core::geometry::Rect::new(80.0, 470.0, 260.0, 120.0));
    app.peindre_ce_qui_a_change(taille, false);
    let apres_la_region = app.pixmap.clone().expect("une image");
    app.une_image_sans_fenetre(taille);
    let entiere = app.pixmap.clone().expect("une image");
    // Le témoin et ses abords — la barre d'action, en bas, a sa propre vie d'une image à l'autre.
    let ecarts = (440..620u32)
        .flat_map(|y| (50..370u32).map(move |x| (x, y)))
        .filter(|&(x, y)| apres_la_region.pixel(x, y) != entiere.pixel(x, y))
        .count();
    assert_eq!(
        ecarts, 0,
        "la région salie s'est repeinte seule dans un écran flouté"
    );
}
