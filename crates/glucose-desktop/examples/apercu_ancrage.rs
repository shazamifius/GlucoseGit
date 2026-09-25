//! **La fenêtre de l'éditeur du texte lié, peinte hors écran** (ANCRE-UX) : à l'étape de la
//! source, un passage choisi, puis deux ; à l'étape de la cible. À la taille et à l'échelle de
//! son écran — 2160 × 1350 à 150 % —, écrites en PNG pour être regardées.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_ancrage -- <dossier>
//! ```

use glucose_core::store::Store;
use glucose_core::text_anchors::create_anchor;
use glucose_core::types::{Annotation, Viewport};
use glucose_desktop::bench;
use glucose_desktop::renderer::Renderer;
use glucose_desktop::ui::ancrage::{Ancrage, Etape};
use glucose_desktop::ui::UiState;

const ECRAN: (u32, u32) = (2160, 1350);
const SOURCE: &str = "testetsetetstetsetest\n\ntestes\nt";
const CIBLE: &str = "## idée du projet\n**a la base le projet vennais d'un jeux**\n- nous somme sur une ile et nous incarnons un personnage\n- la formule $E = mc^2$ au milieu d'une ligne";

fn document() -> Store {
    let mut store = Store::new("apercu");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("source", 0.0, 0.0, SOURCE));
    store.add_annotation(&board, Annotation::text("cible", 500.0, 0.0, CIBLE));
    let mut f = Annotation::arrow("f", 240.0, 40.0, 500.0, 40.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut f
    {
        *source_id = Some("source".into());
        *target_id = Some("cible".into());
    }
    store.add_annotation(&board, f);
    store.clear_selection();
    store.set_viewport(
        &board,
        Viewport {
            x: 300.0,
            y: 300.0,
            scale: 1.5,
        },
    );
    bench::ouvert(store)
}

fn main() {
    let dossier = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let store = document();
    let testes = SOURCE.find("testes").expect("testes");
    let formule = CIBLE.find("$E").expect("la formule");
    let cas = [
        ("source-vide", Etape::Source, vec![], vec![]),
        (
            "source-un",
            Etape::Source,
            vec![(testes, testes + 6)],
            vec![],
        ),
        (
            "source-deux",
            Etape::Source,
            vec![(0, 4), (testes, testes + 6)],
            vec![],
        ),
        (
            "cible",
            Etape::Cible,
            vec![(testes, testes + 6)],
            vec![(formule, formule + 11)],
        ),
    ];
    for (nom, etape, source, cible) in cas {
        let ancres = |texte: &str, plages: Vec<(usize, usize)>| {
            plages
                .into_iter()
                .filter_map(|(a, b)| create_anchor(texte, a, b))
                .collect()
        };
        let mut ui = UiState::new();
        ui.scale_factor = 1.5;
        ui.ancrage = Some(Ancrage {
            fleche: "f".into(),
            etape,
            cartes: (Some("source".into()), Some("cible".into())),
            source: ancres(SOURCE, source),
            cible: ancres(CIBLE, cible),
            glisse: None,
            defilement: 0.0,
        });
        let mut renderer = Renderer::new();
        let image = bench::render_frame(&mut renderer, &mut ui, &store, ECRAN.0, ECRAN.1);
        let chemin = format!("{dossier}/ancrage-{nom}.png");
        match image.save_png(&chemin) {
            Ok(()) => println!("{chemin}"),
            Err(e) => eprintln!("{chemin} : {e}"),
        }
    }
}
