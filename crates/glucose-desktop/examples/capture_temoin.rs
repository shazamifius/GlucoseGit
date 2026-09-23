//! Écrit la capture de la scène témoin, pour être **regardée**.
//!
//! ```text
//! cargo run -p glucose-desktop --example capture_temoin
//! cargo run -p glucose-desktop --example capture_temoin -- mon/dossier
//! ```
//!
//! Le test `test_l_empreinte_de_la_scene_temoin_n_a_pas_change` garde l'empreinte de cette
//! image. Quand il échoue, c'est que le rendu a changé — volontairement ou non. Le geste est
//! alors toujours le même : lancer cet exemple, **ouvrir le PNG et le regarder**, puis mettre
//! l'empreinte à jour si le changement est celui qu'on voulait.
//!
//! Garder une empreinte plutôt que l'image elle-même est délibéré : une image de référence
//! versionnée grossirait le dépôt à chaque évolution du rendu, et il y en aura beaucoup.

use glucose_core::hash::{hex_of, sha256};
use glucose_core::synth;
use glucose_core::text::Selection;
use glucose_desktop::bench;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::renderer::{Renderer, TextEditSession};
use glucose_desktop::ui::UiState;

fn main() {
    let dossier = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let temoin = synth::witness();
    let (w0, h0) = synth::WITNESS_SIZE;

    // La seconde capture est la même scène **vue de plus loin** : c'est elle qui montre ce que
    // le texte devient quand le zoom descend, là où la fiche 03 signalait onze bornes qui
    // figent la police pendant que la boîte, elle, se met à l'échelle.
    let mut dezoome = temoin.clone();
    bench::frame_document(&mut dezoome, 0.45, w0, h0);

    // La vitrine : le document soigné, celui des captures du dépôt.
    let vitrine = synth::showcase();
    let mut vitrine_large = vitrine.clone();
    bench::frame_document(&mut vitrine_large, 0.8, 1920, 1080);

    // Un gros plan sur les formules : c'est ce qui se voit le moins sur une vue d'ensemble,
    // et le plus sur une capture de dépôt.
    let mut vitrine_zoom = vitrine.clone();
    {
        let board = vitrine_zoom.project.active_board_id.clone();
        vitrine_zoom.set_viewport(
            &board,
            glucose_core::types::Viewport {
                x: 1_150.0,
                y: 640.0,
                scale: 1.9,
            },
        );
    }

    // Une carte **en cours d'édition**, le curseur posé dans une formule : c'est la seule
    // façon de regarder les délimiteurs colorés et la pastille de prévisualisation.
    //
    // Sans empreinte, volontairement : le curseur clignote sur une horloge réelle, et figer
    // cette image reviendrait à parier que la capture tient toujours dans la demi-seconde où
    // il est allumé. Ce qui se teste ici se teste par la logique (`renderer::card::tests`),
    // pas par les pixels.
    {
        let store = synth::witness();
        let texte = store
            .active_board()
            .and_then(|b| b.annotations.iter().find(|a| a.id() == "t-maths"))
            .and_then(|a| match a {
                glucose_core::types::Annotation::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .expect("la carte des formules du témoin");
        // Le curseur dans la formule valide, en deuxième ligne.
        let tete = texte.find("$$").map(|i| i + 8).unwrap_or(0);
        let session = TextEditSession {
            ann_id: "t-maths".to_string(),
            buffer: texte.clone(),
            selection: Selection::at(tete),
            goal_x: None,
            blink_timer: std::time::Instant::now(),
            // Le temoin est une capture de reference : son curseur est eteint, sans quoi
            // l'image changerait selon l'instant ou elle est prise (BLINK-1).
            curseur_visible: false,
        };
        let mut renderer = Renderer::new();
        let mut ui = UiState::new();
        let mut pixmap = tiny_skia::Pixmap::new(w0, h0).expect("un pixmap");
        let guides = glucose_core::smart_align::SnapGuides::default();
        renderer.render(
            &mut pixmap.as_mut(),
            &store,
            &mut ui,
            SceneOverlay {
                arrivages: &[],
                guides: &guides,
                selection_box: None,
                editing: Some(&session),
            },
            Pointer { x: 0.0, y: 0.0 },
            glucose_desktop::renderer::Regard::immobile(),
        );
        let png = pixmap.encode_png().expect("encoder un PNG");
        let chemin = format!("{dossier}/temoin-edition.png");
        match std::fs::write(&chemin, &png) {
            Ok(()) => println!(
                "{chemin} — {w0}x{h0}, {} octets (sans empreinte)",
                png.len()
            ),
            Err(e) => eprintln!("{chemin} : {e}"),
        }
    }

    // Le menu contextuel n'est pas dans le document : il se capture par l'état d'interface.
    {
        let store = synth::witness_selected();
        let png = bench::capture_with(&store, w0, h0, |ui| {
            ui.context_menu_at = Some((640.0, 320.0));
        });
        let chemin = format!("{dossier}/temoin-menu.png");
        let empreinte = &hex_of(&sha256(&png))[..16];
        match std::fs::write(&chemin, &png) {
            Ok(()) => println!(
                "{chemin} — {w0}x{h0}, {} octets, empreinte {empreinte}",
                png.len()
            ),
            Err(e) => eprintln!("{chemin} : {e}"),
        }
    }

    for (nom, store, w, h) in [
        ("temoin", &temoin, w0, h0),
        ("temoin-dezoome", &dezoome, w0, h0),
        ("temoin-selection", &synth::witness_selected(), w0, h0),
        ("vitrine", &vitrine, 1920u32, 1080u32),
        ("vitrine-large", &vitrine_large, 1920, 1080),
        ("vitrine-zoom", &vitrine_zoom, 1600, 900),
    ] {
        let png = bench::capture(store, w, h);
        let chemin = format!("{dossier}/{nom}.png");
        match std::fs::write(&chemin, &png) {
            Ok(()) => println!(
                "{chemin} — {w}x{h}, {} octets, empreinte {}",
                png.len(),
                &hex_of(&sha256(&png))[..16]
            ),
            Err(e) => eprintln!("{chemin} : {e}"),
        }
    }
}
