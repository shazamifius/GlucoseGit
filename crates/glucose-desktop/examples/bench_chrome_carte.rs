//! **Ce que CHROME-1 retire de chaque image** : les lignes de la couche du dessus — effacées,
//! relevées et envoyées à la carte à chaque image — avant et après que la chrome passe sur la
//! carte, au PC et au téléphone.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_chrome_carte
//! ```
//!
//! # Ce qu'il compare, et pourquoi dans la même exécution
//!
//! L'ancienne loi se rejoue sur le même document : la couche du dessus rendue, puis la chrome
//! composée dedans par `render_ui` — exactement ce que la voie graphique faisait avant
//! CHROME-1. Des lignes et des octets, pas des millisecondes : ils ne dépendent pas de la
//! machine ni du bruit d'une exécution (MÉTHODE § 4, « comparer deux exécutions, c'est comparer
//! du bruit »).

use glucose_core::store::Store;
use glucose_core::types::Viewport;
use glucose_desktop::interactions::tools::text_card;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::present::bandes::Bandes;
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::rail::Rail;
use glucose_desktop::ui::{render_ui, UiState};
use tiny_skia::Pixmap;

/// Un écran, son échelle d'interface, et le rail s'il y en a un.
struct Cas {
    nom: &'static str,
    ecran: (u32, u32),
    echelle: f32,
    rail: Option<bool>,
}

const CAS: &[Cas] = &[
    Cas {
        nom: "son PC (2160 x 1350, 150 %)",
        ecran: (2160, 1350),
        echelle: 1.5,
        rail: None,
    },
    Cas {
        nom: "son telephone, rail replie",
        ecran: (1080, 2400),
        echelle: 2.75,
        rail: Some(false),
    },
    Cas {
        nom: "son telephone, rail deplie",
        ecran: (1080, 2400),
        echelle: 2.75,
        rail: Some(true),
    },
];

fn main() {
    println!("\n  Les lignes de la couche du dessus, a chaque image, sans rien de choisi\n");
    println!(
        "    {:<30} {:>14} {:>14} {:>16}",
        "ecran", "avant", "apres", "envoye en moins"
    );
    for cas in CAS {
        let (avant, apres) = mesurer(cas);
        let octets = |lignes: u32| f64::from(lignes) * f64::from(cas.ecran.0) * 4.0 / 1e6;
        println!(
            "    {:<30} {:>6} lignes  {:>6} lignes  {:>10.2} Mo",
            cas.nom,
            avant,
            apres,
            octets(avant) - octets(apres)
        );
    }
    println!(
        "\n  « avant » : la chrome composee dans la couche du dessus, comme la voie graphique le\n  \
         faisait ; « apres » : ce qu'elle porte quand la carte pose la chrome. Chaque ligne est\n  \
         effacee, relevee et envoyee a la carte a chaque image qui se redessine.\n"
    );
}

/// Les lignes de la couche du dessus, selon l'ancienne loi et selon la nouvelle.
fn mesurer(cas: &Cas) -> (u32, u32) {
    let mut renderer = Renderer::new();
    let store = mur(&renderer, 120);
    let ui = || {
        let mut ui = UiState::new();
        ui.scale_factor = cas.echelle;
        if let Some(ouvert) = cas.rail {
            ui.rail = Rail {
                actif: true,
                ouvert,
            };
        }
        ui
    };
    let (w, h) = cas.ecran;
    let mut dessous = Pixmap::new(w, h).expect("un pixmap");
    let mut dessus = Pixmap::new(w, h).expect("un pixmap");
    let guides = glucose_core::smart_align::SnapGuides::default();
    let pointeur = Pointer { x: -1.0, y: -1.0 };
    let mut ui_apres = ui();
    renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        &store,
        (&mut ui_apres, pointeur),
        SceneOverlay::sans_rien(&guides),
        Regard::immobile(),
    );
    let apres = Bandes::relever(&dessus).lignes();
    // L'ancienne loi : la même couche, plus la chrome composée dedans.
    let mut ui_avant = ui();
    let (typo, theme) = (&renderer.typography, &renderer.theme);
    render_ui(
        &mut dessus.as_mut(),
        &store,
        &mut ui_avant,
        typo,
        theme,
        pointeur,
    );
    let avant = Bandes::relever(&dessus).lignes();
    (avant, apres)
}

/// Un mur de cartes de texte, comme `bench_dessus` le monte — vu d'un peu plus loin.
fn mur(renderer: &Renderer, cartes: usize) -> Store {
    let mut store = Store::new("chrome");
    let board = store.project.active_board_id.clone();
    let colonnes = (cartes as f64).sqrt().ceil().max(1.0) as usize;
    for i in 0..cartes {
        let carte = text_card(
            &renderer.typography,
            &renderer.math,
            format!("carte-{i}"),
            (i % colonnes) as f64 * 320.0,
            (i / colonnes) as f64 * 180.0,
            format!("Carte {i} - une note posee sur le mur, avec du texte qui revient."),
        );
        store.add_annotation(&board, carte);
    }
    store.clear_selection();
    let vue = Viewport {
        x: 100.0,
        y: 100.0,
        scale: 1.0,
    };
    store.set_viewport(&board, vue);
    store
}
