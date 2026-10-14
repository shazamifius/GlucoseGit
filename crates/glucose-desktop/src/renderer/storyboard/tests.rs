//! Les épreuves du dessin des cases du storyboard (STORYBOARD-1) : le cadre, la bande, le
//! numéro, aux couleurs de Glucose Tauri, par la voie processeur.

use crate::params::{Pointer, SceneOverlay};
use crate::renderer::{Regard, Renderer};
use crate::ui::UiState;
use glucose_core::store::storyboard::{Format, Grille, DESCRIPTION};
use glucose_core::store::Store;
use glucose_core::types::Viewport;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (1440, 900);

/// Un storyboard de huit cases, vu à l'échelle 1 depuis l'origine.
fn document() -> Store {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    store.set_viewport(&board, vue);
    let grille = Grille {
        format: Format::Cinema,
        largeur: 300.0,
        colonnes: 4,
        ecart: 24.0,
    };
    // Centrée sur l'écran, comme avant FANTOME-1 : son coin s'en déduit.
    let (l, h) = grille.etendue(grille.nombre_initial());
    store.poser_le_storyboard(&board, grille, (720.0 - l / 2.0, 480.0 - h / 2.0));
    store.clear_selection();
    store
}

fn rendre(store: &Store) -> Pixmap {
    let mut image = Pixmap::new(ECRAN.0, ECRAN.1).expect("une image");
    let guides = glucose_core::smart_align::SnapGuides::default();
    Renderer::new().render(
        &mut image.as_mut(),
        store,
        &mut UiState::new(),
        SceneOverlay::sans_rien(&guides),
        Pointer { x: -1.0, y: -1.0 },
        Regard::immobile(),
    );
    image
}

fn rgb(image: &Pixmap, (x, y): (f64, f64)) -> (u8, u8, u8) {
    let p = image.pixel(x as u32, y as u32).expect("dans l'image");
    (p.red(), p.green(), p.blue())
}

/// **Le cadre en `#1a1a1a`, la bande en `#111`, le badge en `#222` à 0,9** — les couleurs de
/// Tauri, là où elles se posent ; et le numéro s'écrit dans le badge.
#[test]
fn test_une_case_se_dessine_aux_couleurs_de_tauri() {
    let store = document();
    let board = store.project.active_board_id.clone();
    let case = store.cases(&board)[1].clone();
    let image = rendre(&store);
    let milieu = (case.x + case.width / 2.0, case.y + case.height / 2.0);
    assert_eq!(rgb(&image, milieu), (0x1a, 0x1a, 0x1a), "le cadre");
    let bande = (
        case.x + case.width / 2.0,
        case.y + case.height + DESCRIPTION - 4.0,
    );
    assert_eq!(rgb(&image, bande), (0x11, 0x11, 0x11), "la bande");
    // Le badge, en (4, h + 4), 24 × 18 : son coin est du gris du badge, son milieu porte
    // l'encre claire du numéro « 2 ».
    let coin = (case.x + 5.0, case.y + case.height + 5.0);
    let (r, _, _) = rgb(&image, coin);
    assert!((0x1f..=0x22).contains(&r), "le badge : {r:#x}");
    let encre = (0..24)
        .flat_map(|dx| (0..18).map(move |dy| (dx, dy)))
        .map(|(dx, dy)| {
            let at = (
                case.x + 4.0 + f64::from(dx),
                case.y + case.height + 4.0 + f64::from(dy),
            );
            rgb(&image, at).0
        })
        .filter(|r| *r > 0x50)
        .count();
    assert!(
        encre > 5,
        "le numéro s'écrit dans son badge : {encre} pixels clairs"
    );
}

/// **La description s'écrit dans la bande**, à droite du numéro, et seulement là.
#[test]
fn test_la_description_s_ecrit_dans_la_bande() {
    let mut store = document();
    let board = store.project.active_board_id.clone();
    let case = store.cases(&board)[0].clone();
    let sans = rendre(&store);
    store.update_panel(&board, &case.id, "Plan large");
    let avec = rendre(&store);
    let mut changes = Vec::new();
    for y in 0..ECRAN.1 {
        for x in 0..ECRAN.0 {
            if sans.pixel(x, y) != avec.pixel(x, y) {
                changes.push((f64::from(x), f64::from(y)));
            }
        }
    }
    assert!(!changes.is_empty(), "la description ne s'écrit pas");
    let bas = case.y + case.height;
    let dans_la_bande = |&(x, y): &(f64, f64)| {
        x >= case.x + 28.0 && x <= case.x + case.width && y >= bas && y <= bas + DESCRIPTION
    };
    assert!(
        changes.iter().all(dans_la_bande),
        "elle déborde de sa bande"
    );
}

/// **La case choisie porte le contour blanc de Tauri**, à deux unités de son cadre — dans la
/// couche du dessus, par-dessus l'image qu'elle porte. Non choisie, rien.
#[test]
fn test_la_case_choisie_porte_son_contour() {
    let store = document();
    let board = store.project.active_board_id.clone();
    let case = store.cases(&board)[1].clone();
    let rendre_avec = |choisie: Option<String>| {
        let mut image = Pixmap::new(ECRAN.0, ECRAN.1).expect("une image");
        let guides = glucose_core::smart_align::SnapGuides::default();
        let mut ui = UiState::new();
        ui.storyboard.choisie = choisie;
        Renderer::new().render(
            &mut image.as_mut(),
            &store,
            &mut ui,
            SceneOverlay::sans_rien(&guides),
            Pointer { x: -1.0, y: -1.0 },
            Regard::immobile(),
        );
        image
    };
    let au_contour = (case.x + case.width / 2.0, case.y - 1.5);
    let sans = rgb(&rendre_avec(None), au_contour);
    let avec = rgb(&rendre_avec(Some(case.id.clone())), au_contour);
    assert!(sans.0 < 0x40, "sans choix, le canevas : {sans:?}");
    assert!(avec.0 > 0xa0, "choisie, le contour blanc : {avec:?}");
    // P2, proposition 3 (fiche 66) : ses deux « + », au milieu de ses bords — le croisement de
    // leurs traits, clair, là où la géométrie que le clic lit les place.
    let boite = super::boite_a_l_ecran(&case, &store.viewport());
    let [gauche, droite] = super::plus(boite, UiState::new().scale());
    let milieu_de = |(x, y, w, h): (f32, f32, f32, f32)| (x + w / 2.0, y + h / 2.0);
    let mi_hauteur = boite.1 + boite.3 / 2.0;
    assert_eq!(
        milieu_de(gauche),
        (boite.0, mi_hauteur),
        "au milieu du bord gauche"
    );
    assert_eq!(
        milieu_de(droite),
        (boite.0 + boite.2, mi_hauteur),
        "et du droit"
    );
    for (x, y, w, h) in [gauche, droite] {
        let milieu = (f64::from(x + w / 2.0), f64::from(y + h / 2.0));
        let sans = rgb(&rendre_avec(None), milieu);
        let avec = rgb(&rendre_avec(Some(case.id.clone())), milieu);
        assert!(sans.0 < 0x40, "sans choix, pas de « + » : {sans:?}");
        assert!(avec.0 > 0xa0, "choisie, le « + » : {avec:?}");
    }
}
