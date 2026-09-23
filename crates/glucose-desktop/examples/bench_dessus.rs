//! **Ce que la couche du dessus touche vraiment**, et ce que l'effacer et la téléverser
//! entière coûte pour le reste.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_dessus
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! Les chroniques de terrain donnent, sur chaque geste et sur les deux cartes graphiques :
//!
//! ```text
//!     effacer   2,05 ms        blit   1,72 ms
//! ```
//!
//! Soit **3,77 millisecondes sur une image qui en coûte 9,74** — et ces deux postes ne
//! dessinent rien. Le premier remplit de transparent les seize mébioctets de la couche du
//! dessus ; le second les envoie à la carte. Tous deux travaillent sur l'**écran entier**,
//! quelle que soit la part qui a changé.
//!
//! Or cette couche ne porte plus, depuis COMPOSANT-1, que la chrome — barre, onglets, docks,
//! minimap — et les ornements du geste. La question qui décide du chantier est donc : **quelle
//! part de l'écran cette couche touche-t-elle réellement ?**
//!
//! Si elle en touche les trois quarts, il n'y a rien à gagner et il faut chercher ailleurs. Si
//! elle en touche le tiers, ramener les deux postes à cette part ferait passer l'image sous
//! les 8,33 ms que deux balayages autorisent — c'est-à-dire **cent vingt images par seconde**,
//! et le plancher de la charte tenu pour la première fois.
//!
//! # Ce qu'il mesure, et il ne suppose rien
//!
//! Il rend la scène par la voie graphique, puis **balaie** la couche du dessus et compte les
//! lignes qui portent au moins un pixel non transparent. Aucune déclaration, aucune boîte
//! calculée d'avance : ce qui est mesuré est ce que le dessin a réellement écrit.
//!
//! Il le fait sur trois cas, parce que la réponse en dépend : un canevas nu, un document
//! chargé au repos, et le même avec une sélection — qui ajoute les ornements au milieu de
//! l'écran.

use glucose_core::store::Store;
use glucose_core::types::Viewport;
use glucose_desktop::interactions::tools::text_card;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::renderer::{Regard, Renderer};
use glucose_desktop::ui::UiState;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (2560, 1600);
const CARTES: usize = 120;

/// Ce qu'un balayage de la couche du dessus a trouvé.
struct Empreinte {
    /// Combien de lignes portent au moins un pixel non transparent.
    lignes: u32,
    /// La première et la dernière de ces lignes — l'étendue d'un téléversement en un bloc.
    etendue: (u32, u32),
    /// **Ce que le relevé lui-même coûte.** Sans lui, ce banc dirait ce qu'il y a à gagner
    /// sans dire ce qu'il faut payer pour le savoir — et le terrain a montré que le second
    /// pouvait dépasser le premier.
    cout_ms: f64,
}

fn main() {
    println!(
        "\n  Ce que la couche du dessus touche, sur un ecran de {} x {}\n",
        ECRAN.0, ECRAN.1
    );
    println!(
        "    cas                     lignes ecrites   etendue     part   en un bloc   relever"
    );
    for (nom, cartes, selection) in [
        ("canevas nu", 0, false),
        ("document charge", CARTES, false),
        ("avec une selection", CARTES, true),
    ] {
        let e = mesurer(cartes, selection);
        let part = f64::from(e.lignes) / f64::from(ECRAN.1);
        let bloc = f64::from(e.etendue.1.saturating_sub(e.etendue.0) + 1) / f64::from(ECRAN.1);
        println!(
            "    {nom:<22}  {:>6} / {}   {:>4}-{:<5} {:>6.1} %     {:>5.1} %   {:>6.2} ms",
            e.lignes,
            ECRAN.1,
            e.etendue.0,
            e.etendue.1,
            part * 100.0,
            bloc * 100.0,
            e.cout_ms
        );
    }
    println!(
        "\n  « part » est ce qu'un televersement ligne a ligne enverrait ; « en un bloc » ce\n  \
         qu'enverrait un seul rectangle allant de la premiere ligne ecrite a la derniere.\n  \
         L'ecart entre les deux dit s'il faut plusieurs rectangles ou si un seul suffit.\n"
    );
}

/// Rend une scène par la voie graphique et balaie la couche du dessus.
fn mesurer(cartes: usize, selection: bool) -> Empreinte {
    let mut renderer = Renderer::new();
    let mut store = mur(&renderer, cartes);
    let board = store.project.active_board_id.clone();
    store.set_viewport(
        &board,
        Viewport {
            x: 100.0,
            y: 100.0,
            scale: 1.0,
        },
    );
    if selection {
        // Trois cartes selectionnees : leurs ornements -- cadre, poignees -- se posent au
        // milieu de l'ecran, loin de la chrome, et c'est le cas qui decide s'il faut un
        // rectangle ou plusieurs.
        store.selected_annotation_ids = (0..3).map(|i| format!("carte-{i}")).collect();
    }
    let mut dessous = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let mut dessus = Pixmap::new(ECRAN.0, ECRAN.1).expect("un pixmap");
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    dessus.fill(tiny_skia::Color::TRANSPARENT);
    renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        &store,
        (&mut ui, Pointer { x: 0.0, y: 0.0 }),
        SceneOverlay {
            arrivages: &[],
            guides: &guides,
            selection_box: None,
            editing: None,
        },
        Regard {
            degradation_permise: false,
            en_mouvement: false,
        },
    );
    // Le relevé se mesure sur dix passes : une seule serait noyée dans le bruit de la
    // première lecture, qui remplit les caches.
    let debut = std::time::Instant::now();
    let mut e = balayer(&dessus);
    for _ in 0..9 {
        e = balayer(&dessus);
    }
    e.cout_ms = debut.elapsed().as_secs_f64() * 100.0;
    e
}

/// Les lignes de ce pixmap qui portent au moins un pixel non transparent.
///
/// **C'est le releveur de production qui est appelé**, et non une copie : un banc qui mesure
/// autre chose que ce qui tourne ne mesure rien.
fn balayer(p: &Pixmap) -> Empreinte {
    let bandes = glucose_desktop::present::bandes::Bandes::relever(p);
    let lignes = bandes.lignes();
    let premiere = bandes.intervalles().first().map_or(u32::MAX, |b| b.start);
    let derniere = bandes.intervalles().last().map_or(0, |b| b.end - 1);
    Empreinte {
        lignes,
        etendue: if lignes == 0 {
            (0, 0)
        } else {
            (premiere, derniere)
        },
        cout_ms: 0.0,
    }
}

/// Un mur de cartes de texte, comme `bench_texte` le monte.
fn mur(renderer: &Renderer, cartes: usize) -> Store {
    let mut store = Store::new("dessus");
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
    store
}
