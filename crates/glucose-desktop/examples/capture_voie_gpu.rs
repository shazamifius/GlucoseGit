//! **Écrit ce que la voie graphique compose**, pour être regardé.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example capture_voie_gpu
//! cargo run --release -p glucose-desktop --example capture_voie_gpu -- mon/dossier
//! ```
//!
//! # Pourquoi cet exemple existe
//!
//! `capture_temoin` montre ce que le **processeur** rend. Depuis que la carte peint le fond,
//! les lueurs et les photos, l'image que l'utilisateur voit n'est plus celle-là : elle est
//! composée en cinq temps sur la carte, et aucune épreuve ne la regardait.
//!
//! Or c'est exactement ce qui a coûté deux allers-retours à l'écran lors de l'étape 1 — un
//! écran noir, puis des poignées disparues. Les deux se voyaient au premier coup d'œil, et
//! aucun test ne pouvait les voir, parce que tous regardaient une moitié de l'image.
//!
//! Celui-ci compose **les cinq temps**, hors fenêtre, et écrit le résultat. C'est la seule
//! façon de vérifier l'ordre de composition sans ouvrir l'application : une lueur par-dessus
//! sa carte, un fond par-dessus tout, une couche du dessous qui remplace au lieu de composer
//! — rien de tout cela ne se voit dans le code, et tout se voit ici.
//!
//! Sans empreinte, volontairement : deux cartes graphiques n'arrondissent pas au même bit, et
//! une empreinte échouerait sur la moitié des machines. Ce qui se **tient** est l'écart entre
//! les deux voies, et c'est ce que les tests de `fond_gpu` et `lueurs_gpu` mesurent. Cette
//! image-ci se regarde.

use glucose_core::synth;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::present::banc_gpu;
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::UiState;

fn main() {
    let dossier = std::env::args().nth(1).unwrap_or_else(|| ".".to_string());
    let (largeur, hauteur) = synth::WITNESS_SIZE;
    let store = synth::witness_selected();

    let Some(dessous) = tiny_skia::Pixmap::new(largeur, hauteur) else {
        println!("pixmap impossible");
        return;
    };
    let Some(dessus) = tiny_skia::Pixmap::new(largeur, hauteur) else {
        println!("pixmap impossible");
        return;
    };
    let (mut dessous, mut dessus) = (dessous, dessus);
    dessous.fill(tiny_skia::Color::TRANSPARENT);
    dessus.fill(tiny_skia::Color::TRANSPARENT);

    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let confie = renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        &store,
        (&mut ui, Pointer { x: 0.0, y: 0.0 }),
        SceneOverlay {
            guides: &guides,
            selection_box: None,
            editing: None,
        },
        Regard::immobile(),
    );

    println!(
        "  {} lueur(s), {} photo(s), {} composant(s), dessous {}",
        confie.lueurs.len(),
        confie.photos.len(),
        confie.composants.len(),
        if confie.dessous_porte_quelque_chose {
            "porte de l'encre"
        } else {
            "VIDE -- il ne sera pas televerse"
        }
    );

    let Some(image) = composer(&renderer, &confie, (&dessous, &dessus), (largeur, hauteur)) else {
        println!("aucune carte utilisable : rien a capturer");
        return;
    };
    let chemin = std::path::Path::new(&dossier).join("voie-gpu.png");
    match image.save_png(&chemin) {
        Ok(()) => println!("  ecrit : {}", chemin.display()),
        Err(e) => println!("  capture non ecrite : {e}"),
    }
}

/// Compose les cinq temps hors fenêtre, exactement comme la présentation le fait.
fn composer(
    renderer: &Renderer,
    confie: &glucose_desktop::renderer::Confie,
    (dessous, dessus): (&tiny_skia::Pixmap, &tiny_skia::Pixmap),
    taille: (u32, u32),
) -> Option<tiny_skia::Pixmap> {
    let (peripherique, file) = banc_gpu::carte()?;
    banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        taille,
        confie,
        (dessous, dessus),
        &|cle| confie.composant(cle).and_then(|c| c.rendre(renderer.kit())),
    )
}
