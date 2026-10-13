//! **CHROME-1 au téléphone : la chrome se pose pareil sur les deux voies** — la bande, la
//! minimap et son rectangle de vue, le rail déplié qui passe dessus.
//!
//! La chrome a quitté la couche du dessus : la carte la pose d'après ses caches, en textures,
//! et le rectangle de la vue en filets. L'épreuve des deux voies (`voies_suite`) compare
//! l'écran d'un bureau, où le rail n'existe pas ; celle-ci prend un téléphone à l'échelle du
//! sien (interface à 275 %), mais **plus court** : sur son moto g54 debout (1080 × 2400), le
//! rail déplié s'arrête à la ligne 1 546 et la minimap commence à 2 037 — ils ne se couvrent
//! pas, et l'ordre ne s'y verrait pas. Sur un écran de 1 800 lignes, ses deux colonnes
//! descendent jusqu'à 1 546, et la minimap commence à 1 437 : le rail la couvre.
//! Chaque morceau se compare là où il est, sans qu'aucun pixel n'y échappe : sur un écran
//! entier, quatre filets d'un pixel passeraient sous n'importe quelle part tolérée.

use glucose_core::store::Store;
use glucose_core::types::Annotation;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::present::banc_gpu;
use glucose_desktop::renderer::{Confie, Regard, Renderer};
use glucose_desktop::ui::rail::Rail;
use glucose_desktop::ui::UiState;
use tiny_skia::Pixmap;

/// Un téléphone debout plus court que le sien, où le rail déplié couvre la minimap.
const TELEPHONE: (u32, u32) = (1080, 1800);

/// L'écart d'arrondi entre deux compositions huit bits (`voies_suite`) : au-delà, ce n'est plus
/// un arrondi, c'est un morceau qui manque ou un ordre inversé.
const ECART_COURANT: u8 = 3;

/// Un téléphone debout, à l'échelle du sien, le rail déplié.
fn au_telephone() -> UiState {
    let mut ui = UiState::new();
    ui.scale_factor = 2.75;
    ui.rail = Rail {
        actif: true,
        ouvert: true,
    };
    ui
}

/// Un document plus grand que la vue, et une carte de texte.
fn document() -> Store {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    let grande = Annotation::membrane("m", -2000.0, -2000.0, 6000.0, 8000.0);
    store.add_annotation(&board, grande);
    store.add_annotation(&board, Annotation::text("t", 80.0, 400.0, "Bonjour"));
    store.clear_selection();
    store
}

fn par_le_processeur(store: &Store) -> Pixmap {
    let mut image = Pixmap::new(TELEPHONE.0, TELEPHONE.1).expect("une image");
    let guides = glucose_core::smart_align::SnapGuides::default();
    Renderer::new().render(
        &mut image.as_mut(),
        store,
        &mut au_telephone(),
        SceneOverlay::sans_rien(&guides),
        Pointer { x: -1.0, y: -1.0 },
        Regard::immobile(),
    );
    image
}

fn par_la_carte(store: &Store) -> Option<(Pixmap, Confie)> {
    let mut dessous = Pixmap::new(TELEPHONE.0, TELEPHONE.1).expect("un pixmap");
    let mut dessus = Pixmap::new(TELEPHONE.0, TELEPHONE.1).expect("un pixmap");
    let mut renderer = Renderer::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let confie = renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        store,
        (&mut au_telephone(), Pointer { x: -1.0, y: -1.0 }),
        SceneOverlay::sans_rien(&guides),
        Regard::immobile(),
    );
    let (peripherique, file) = banc_gpu::carte()?;
    let image = banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        TELEPHONE,
        &confie,
        (&dessous, &dessus),
        &|cle| confie.pixels(&renderer, cle),
    )?;
    Some((image, confie))
}

/// Une boîte d'écran : gauche, haut, droite, bas, en pixels entiers.
type Boite = (u32, u32, u32, u32);

fn boite_de(pose: &glucose_desktop::present::scene_gpu::Pose) -> Boite {
    (
        pose.x as u32,
        pose.y as u32,
        (pose.x + pose.largeur) as u32,
        (pose.y + pose.hauteur) as u32,
    )
}

/// Combien de canaux de cette boîte diffèrent de plus d'un arrondi.
fn ecarts_dans(a: &Pixmap, b: &Pixmap, (g, h, d, bas): Boite) -> usize {
    let largeur = a.width() as usize * 4;
    let (g, d) = (g as usize * 4, d.min(a.width()) as usize * 4);
    (h..bas.min(a.height()))
        .map(|y| {
            let debut = y as usize * largeur;
            let (ligne_a, ligne_b) = (&a.data()[debut..], &b.data()[debut..]);
            (g..d)
                .filter(|&i| ligne_a[i].abs_diff(ligne_b[i]) > ECART_COURANT)
                .count()
        })
        .sum()
}

/// **La chrome du téléphone se pose pareil sur les deux voies**, morceau par morceau : la
/// bande, le fond de la minimap et son rectangle de vue, le rail déplié par-dessus.
#[test]
fn test_chrome_1_la_chrome_d_un_telephone_se_pose_pareil_sur_les_deux_voies() {
    let store = document();
    let Some((carte, confie)) = par_la_carte(&store) else {
        eprintln!("aucune carte utilisable : épreuve sautée");
        return;
    };
    let processeur = par_le_processeur(&store);
    let chrome = &confie.chrome;
    let par_nom = |nom: &str| {
        chrome
            .sous_le_cadre()
            .iter()
            .chain(chrome.sur_le_cadre())
            .find(|t| t.identite == nom)
            .map(|t| boite_de(&t.pose))
            .unwrap_or_else(|| panic!("{nom} n'est pas confié à la carte"))
    };
    let (bande, minimap, rail) = (
        par_nom("chrome:bande"),
        par_nom("chrome:minimap"),
        par_nom("chrome:rail"),
    );
    // Le rail déplié couvre la minimap : sans ce recouvrement, l'ordre ne se verrait pas.
    let recouvre = rail.0 < minimap.2 && minimap.0 < rail.2 && rail.1 < minimap.3;
    assert!(
        recouvre && minimap.1 < rail.3,
        "le rail {rail:?} ne couvre plus la minimap {minimap:?} : l'épreuve ne voit plus l'ordre"
    );
    for (quoi, boite) in [
        ("la bande", bande),
        ("la minimap", minimap),
        ("le rail", rail),
    ] {
        let ecarts = ecarts_dans(&processeur, &carte, boite);
        assert_eq!(
            ecarts, 0,
            "{quoi} diffère entre les deux voies sur {ecarts} canaux, dans {boite:?}"
        );
    }
    assert_eq!(
        chrome.cadre().len(),
        5,
        "le rectangle de la vue : quatre filets et un voile"
    );
}
