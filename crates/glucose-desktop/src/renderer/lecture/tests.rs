//! Les épreuves du dessin de l'animatique (fiche 67) : le voile hors de la case, la case
//! intacte, ses sous-titres — par la voie processeur, l'image regardée.

use super::sous_titres;
use crate::interactions::lecture::{vue_sur_la_case, Lecture, Temps};
use crate::params::{Pointer, SceneOverlay};
use crate::renderer::{Regard, Renderer};
use crate::ui::UiState;
use glucose_core::store::storyboard::{Grille, Storyboard, Texte};
use glucose_core::store::Store;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (1440, 900);

/// Un storyboard ; la deuxième case décrite, avec un dialogue — vue comme la lecture la cadre.
fn document() -> (Store, String) {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    store.poser_le_storyboard(&board, Grille::default(), (0.0, 0.0));
    let id = crate::epreuve::les_cases(&store, &board)[1].id.clone();
    store.ecrire_dans_la_case(&board, &id, Texte::Description, "La ville, au loin");
    store.ecrire_dans_la_case(&board, &id, Texte::Champ(0), "Il est tard.");
    let c = crate::epreuve::les_cases(&store, &board)[1].clone();
    let ecran = (0.0, 0.0, f64::from(ECRAN.0), f64::from(ECRAN.1));
    store.set_viewport(
        &board,
        vue_sur_la_case((c.x, c.y, c.width, c.height), ecran),
    );
    store.clear_selection();
    (store, board)
}

fn rendre(store: &Store, ui: &mut UiState) -> Pixmap {
    let mut image = Pixmap::new(ECRAN.0, ECRAN.1).expect("une image");
    let guides = glucose_core::smart_align::SnapGuides::default();
    Renderer::new().render(
        &mut image.as_mut(),
        store,
        ui,
        SceneOverlay::sans_rien(&guides),
        Pointer { x: -1.0, y: -1.0 },
        Regard::immobile(),
    );
    image
}

fn rgb(image: &Pixmap, (x, y): (u32, u32)) -> (u8, u8, u8) {
    let p = image.pixel(x, y).expect("dans l'image");
    (p.red(), p.green(), p.blue())
}

fn lecture((store, board): (&Store, &str), rang: usize) -> Lecture {
    Lecture {
        board: board.to_string(),
        storyboard: crate::epreuve::seul_storyboard(store, board).expect("un storyboard"),
        rang,
        temps: Temps::Depuis(0),
        vue_d_avant: glucose_core::types::Viewport::default(),
    }
}

/// **Les sous-titres** : la description, puis chaque champ qui dit quelque chose, par son nom.
#[test]
fn test_les_sous_titres_disent_la_description_puis_les_champs() {
    let (store, board) = document();
    let s = store
        .storyboard(
            &board,
            &crate::epreuve::seul_storyboard(&store, &board).unwrap(),
        )
        .expect("posé");
    let case = crate::epreuve::les_cases(&store, &board)[1];
    assert_eq!(
        sous_titres(case, s),
        ["La ville, au loin", "Dialogue : Il est tard."]
    );
    let vide = Storyboard::neuf("vide", Grille::default());
    assert!(sous_titres(crate::epreuve::les_cases(&store, &board)[0], &vide).is_empty());
}

/// **Hors de la case, tout est noir ; la case reste ce qu'elle est** — et sans lecture, le même
/// pixel montrait le canevas. Un 16:9 dans un écran 16:10 : des bandes en haut et en bas.
#[test]
fn test_la_lecture_voile_tout_ce_qui_n_est_pas_la_case() {
    let (store, board) = document();
    let bande_du_haut = (720, 20);
    let mut sans = UiState::new();
    sans.reference = true; // l'interface cachée, comme en lecture : seul le voile diffère
    let avant = rendre(&store, &mut sans);
    assert_ne!(
        rgb(&avant, bande_du_haut),
        (0, 0, 0),
        "sans lecture, le canevas"
    );

    let mut ui = UiState::new();
    ui.lecture = Some(lecture((&store, &board), 1));
    let image = rendre(&store, &mut ui);
    assert_eq!(rgb(&image, bande_du_haut), (0, 0, 0), "le voile");
    assert_eq!(rgb(&image, (1430, 880)), (0, 0, 0), "en bas aussi");
    assert_eq!(
        rgb(&image, (720, 300)),
        (0x1a, 0x1a, 0x1a),
        "la case, à ses couleurs"
    );
    let _ = image.save_png(std::env::temp_dir().join("glucose-lecture.png"));
}

/// **Les sous-titres se posent au bas de la case**, en blanc sur un bandeau sombre.
#[test]
fn test_les_sous_titres_s_ecrivent_au_bas_de_la_case() {
    let (store, board) = document();
    let mut ui = UiState::new();
    ui.lecture = Some(lecture((&store, &board), 1));
    let image = rendre(&store, &mut ui);
    // La case va de 45 à 855 en hauteur ; ses sous-titres tiennent dans son dernier quart.
    let clairs = (650..855)
        .flat_map(|y| (200..1240).map(move |x| (x, y)))
        .filter(|&p| rgb(&image, p).0 > 200)
        .count();
    assert!(clairs > 200, "des lettres blanches : {clairs}");
    let haut = (100..600)
        .flat_map(|y| (200..1240).map(move |x| (x, y)))
        .filter(|&p| rgb(&image, p).0 > 200)
        .count();
    assert_eq!(haut, 0, "rien au-dessus : l'image reste ce qu'on lit");
}

/// **Pendant la lecture, aucune prise** : la case choisie n'y montre ni ses « + » ni rien de ce
/// qui se prend — à l'envers, sans lecture, le « + » de gauche s'y dessine.
#[test]
fn test_la_lecture_cache_les_prises_de_la_case_choisie() {
    let (store, board) = document();
    let case = crate::epreuve::les_cases(&store, &board)[1].clone();
    let vp = store.viewport();
    let (x, y, _, h) = crate::renderer::storyboard::boite_a_l_ecran(&case, &vp);
    // Le « + » de gauche, à moitié dans le cadre : sa moitié intérieure.
    let ou = ((x + 4.0) as u32, (y + h / 2.0) as u32);
    let mut ui = UiState::new();
    ui.reference = true;
    ui.storyboard.choisie = Some(case.id.clone());
    let sans = rendre(&store, &mut ui);
    assert_ne!(
        rgb(&sans, ou),
        (0x1a, 0x1a, 0x1a),
        "sans lecture, le « + » se voit"
    );
    ui.lecture = Some(lecture((&store, &board), 1));
    let avec = rendre(&store, &mut ui);
    assert_eq!(
        rgb(&avec, ou),
        (0x1a, 0x1a, 0x1a),
        "en lecture, la case seule"
    );
}
