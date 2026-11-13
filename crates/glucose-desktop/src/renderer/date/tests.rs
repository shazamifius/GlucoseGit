//! DATE-1 à l'écran : la pastille paraît en bas à droite du nœud daté, à l'encre ambre, rien sur
//! un nœud sans date ; de loin, elle ne se tait pas.
//!
//! **L'ambre se reconnaît sans seuil** : c'est le seul **chaud** de la scène (le rouge passe le
//! bleu). Tout le reste y est gris — le fond, la grille, le papier et le contour d'un texte sans
//! parent, la photo en chemin — donc rouge = bleu, ou froid. Un seuil de luminosité ne tiendrait
//! pas de loin : au dixième, la pastille fait deux pixels de haut et son ambre se mêle au fond
//! (un ambre de (53, 48, 41) sur (23, 23, 23), regardé le 13/11) — elle reste chaude.

use crate::bench::render_frame;
use crate::renderer::Renderer;
use crate::ui::UiState;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage, TemporalAnchor, Viewport};

const ECRAN: (u32, u32) = (800, 600);

/// Un texte à l'origine, une photo à droite (`BoardImage` se place par son **centre** : 450 à 550
/// en x) ; datés tous deux, ou aucun ; la vue à l'échelle `zoom`, l'origine à (100, 100).
fn scene(datee: bool, zoom: f64) -> Store {
    let mut store = Store::new("dates");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("a", 0.0, 0.0, "la Révolution"));
    store.add_image(&board, BoardImage::new("i", 500.0, 0.0, 100.0, 100.0));
    let revolution = TemporalAnchor {
        start: 1789,
        end: 1799,
        label: None,
    };
    if datee {
        let ids = ["a".to_string(), "i".to_string()];
        store.ancrer_dans_le_temps(&board, &ids, Some(&revolution));
    }
    store.clear_selection();
    store.set_viewport(
        &board,
        Viewport {
            x: 100.0,
            y: 100.0,
            scale: zoom,
        },
    );
    crate::bench::ouvert(store)
}

/// Les pixels **chauds** (le rouge passe le bleu) dans ce rectangle de l'écran — l'ambre de la
/// pastille, son encre `#fde68a` comme son filet `#fbbf24`.
fn chauds(store: &Store, (x0, y0, x1, y1): (u32, u32, u32, u32)) -> usize {
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let pixmap = render_frame(&mut renderer, &mut ui, store, ECRAN.0, ECRAN.1);
    let mut n = 0;
    for y in y0..y1.min(ECRAN.1) {
        for x in x0..x1.min(ECRAN.0) {
            let p = pixmap.pixel(x, y).expect("dans l'image");
            if p.red() > p.blue() {
                n += 1;
            }
        }
    }
    n
}

/// Le coin en bas à droite de ce nœud à l'écran, par la boîte que le noyau donne aux flèches —
/// celle où la pastille se pose.
fn coin(store: &Store, id: &str) -> (u32, u32) {
    let board = store.active_board().expect("le tableau");
    let b = glucose_core::arrow::emprise(board, id).expect("sa boîte");
    let (x, y) =
        crate::canvas::world_to_screen(b.left + b.width, b.top + b.height, &board.viewport);
    (x as u32, y as u32)
}

/// Autour de ce coin : la pastille déborde de 10 en bas et à droite, et s'étend vers la gauche
/// de la largeur de son mot.
fn autour((cx, cy): (u32, u32), (gauche, marge): (u32, u32)) -> (u32, u32, u32, u32) {
    (
        cx.saturating_sub(gauche),
        cy.saturating_sub(marge),
        cx + marge,
        cy + marge,
    )
}

/// **Daté, chaque nœud porte sa pastille à son coin** — le texte comme la photo ; sans date, le
/// même coin n'a rien de chaud.
#[test]
fn test_date_1_la_pastille_parait_au_coin_du_noeud_date() {
    let (datee, nue) = (scene(true, 1.0), scene(false, 1.0));
    for id in ["a", "i"] {
        let zone = autour(coin(&datee, id), (80, 14));
        let avec = chauds(&datee, zone);
        assert!(
            avec > 30,
            "la pastille au coin de {id} : {avec} pixels d'ambre"
        );
        assert_eq!(chauds(&nue, zone), 0, "{id} sans date : rien");
    }
}

/// **De loin, la date ne se tait pas** (rien ne se cache selon la distance) : au dixième, la
/// pastille suit son nœud, son mot devient le gris de son encre — et son ambre reste chaud.
#[test]
fn test_date_1_de_loin_la_pastille_reste() {
    let (loin, nue) = (scene(true, 0.1), scene(false, 0.1));
    for id in ["a", "i"] {
        let zone = autour(coin(&loin, id), (10, 4));
        assert!(
            chauds(&loin, zone) > 0,
            "{id} au dixième : encore de l'ambre"
        );
        assert_eq!(chauds(&nue, zone), 0, "{id} au dixième, sans date : rien");
    }
}

/// **L'export SVG porte la date, au même rang et au même tracé** (le dessin est écrit une fois
/// sur une `Surface`) : un filet ambre par nœud daté, l'encre de son mot ; rien sans date.
#[test]
fn test_date_1_l_export_svg_porte_la_date() {
    let svg = |store: &Store| {
        let mut renderer = Renderer::new();
        let mut ui = UiState::new();
        renderer.rendre_en_svg((store, &mut ui), store.viewport(), ECRAN, &mut |_| None)
    };
    let (datee, nue) = (svg(&scene(true, 1.0)), svg(&scene(false, 1.0)));
    let filet = r##"stroke="#fbbf24""##;
    assert_eq!(datee.matches(filet).count(), 2, "un filet par nœud daté");
    assert!(datee.contains("#fde68a"), "l'encre du mot");
    assert!(
        !nue.contains(filet) && !nue.contains("#fde68a"),
        "sans date, rien"
    );
}
