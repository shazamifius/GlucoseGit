//! Preuve visuelle et numérique de la réglette de domaines.
//!
//! Comme pour R-45 (`card/proof.rs`), on ne mesure pas les nombres qui ont servi à poser la
//! réglette — ce sont eux qu'on met en doute — mais **l'encre réellement écrite sur le
//! pixmap**. Un remplissage de poids `w` s'obtient en dessinant deux fois la même scène, une
//! fois avec le poids `w` et une fois avec le poids `0`, puis en comparant les deux images :
//! la piste, le cadre, le texte et le halo sont identiques dans les deux, donc la différence
//! **est** le remplissage, et rien d'autre. La mesure resterait valable si le moteur de rendu
//! changeait.

use super::*;
use crate::params::ViewPass;
use crate::renderer::card::draw_annotations;
use crate::renderer::hue::SymbioticHueCache;
use glucose_core::store::DomainPatch;
use glucose_core::types::{Annotation, BoardImage, Domain, Viewport};
use std::collections::HashSet;
use tiny_skia::Pixmap;

/// Taille des captures.
const CANVAS: (u32, u32) = (1100, 620);
/// Origine écran du nœud dans les captures — assez bas pour que la réglette tienne au-dessus.
const ORIGIN: (f64, f64) = (90.0, 190.0);
/// Largeur et hauteur du nœud d'essai, en unités monde.
const NODE: (f64, f64) = (300.0, 140.0);

/// Les trois domaines de la preuve : identifiant, couleur, sigle.
const PROOF_DOMAINS: [(&str, &str, &str); 3] =
    [("d-sci", "#38bdf8", "SCI"), ("d-art", "#f472b6", "ART"), ("d-jv", "#fbbf24", "JV")];

/// Un document portant un nœud texte muni des poids donnés, un par domaine.
fn proof_store(weights: &[f64]) -> Store {
    let mut store = Store::new("Réglette");
    let board = store.project.active_board_id.clone();
    store.add_annotation(
        &board,
        Annotation::Text {
            id: "porteur".into(),
            x: 0.0,
            y: 0.0,
            width: Some(NODE.0),
            height: Some(NODE.1),
            text: "# Newton\n- optique\n- gravitation".into(),
            font_size: Some(14.0),
            color: Some("#94a3b8".into()),
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        },
    );
    for (index, (id, color, icon)) in PROOF_DOMAINS.iter().enumerate() {
        store
            .try_add_domain(Domain {
                id: (*id).into(),
                name: (*id).into(),
                color: (*color).into(),
                icon: (*icon).into(),
                created_at: 0,
            })
            .expect("identifiants distincts");
        if let Some(weight) = weights.get(index) {
            store
                .try_assign_domain_to_node(&board, "porteur", id, *weight)
                .expect("le nœud et le domaine existent");
        }
    }
    store
}

/// Dessine la scène et rend le pixmap.
fn render(typo: &Typography, store: &Store, zoom: f64) -> Pixmap {
    let theme = Theme::dark();
    let mut tints = DomainTints::new();
    tints.refresh(store, &theme);

    let mut pixmap = Pixmap::new(CANVAS.0, CANVAS.1).expect("pixmap de la preuve");
    pixmap.fill(theme.bg_canvas);
    let ids: HashSet<&str> = std::iter::once("porteur").collect();
    let pass = ViewPass {
        vp: Viewport { x: ORIGIN.0, y: ORIGIN.1, scale: zoom },
        visible_ids: &ids,
        header_h: 0.0,
    };
    let mut hue = SymbioticHueCache::new();
    let mut view = pixmap.as_mut();
    draw_annotations(&mut hue, typo, &tints, &mut view, store, None, pass);
    pixmap
}

/// Boîte englobante des pixels qui diffèrent entre deux captures.
fn ink_bbox(left: &Pixmap, right: &Pixmap) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let w = left.width();
    let (a, _) = left.data().as_chunks::<4>();
    let (b, _) = right.data().as_chunks::<4>();
    for (i, (p, q)) in a.iter().zip(b).enumerate() {
        if p != q {
            let (x, y) = (i as u32 % w, i as u32 / w);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
        }
    }
    (x0 != u32::MAX).then_some((x0, y0, x1, y1))
}

/// **La preuve.** La hauteur d'encre d'un remplissage vaut `poids × hauteur de piste`.
///
/// La tolérance couvre deux arrondis et rien d'autre : la boîte englobante est discrétisée au
/// pixel (± 1 px de chaque côté), et l'anti-crénelage des coins arrondis déborde d'un demi
/// pixel. Au-delà, c'est que la réglette ne dit plus la vérité sur le document.
#[test]
fn test_the_filled_height_is_proportional_to_the_weight() {
    let typo = Typography::new();
    let track = GaugeLayout::world().track_height as f64;
    let mut report = String::new();

    for zoom in [0.5_f64, 1.0, 2.0] {
        let empty = render(&typo, &proof_store(&[0.0]), zoom);
        for weight in [0.25_f64, 0.5, 0.75, 1.0] {
            let inked = render(&typo, &proof_store(&[weight]), zoom);
            let (_, y0, _, y1) = ink_bbox(&inked, &empty).expect("un poids non nul doit marquer");
            let measured = (y1 - y0) as f64;
            let expected = weight * track * zoom;
            report.push_str(&format!(
                "zoom={zoom:.2} poids={weight:.2} attendu={expected:.1}px mesuré={measured:.1}px\n"
            ));
            assert!(
                (measured - expected).abs() <= 3.0,
                "zoom {zoom}, poids {weight} : {measured:.1} px au lieu de {expected:.1} px\n{report}"
            );
        }
    }
    println!("{report}");
}

/// Deux poids différents se distinguent **par la hauteur seule**, sans rien lire d'autre.
///
/// L'écart se mesure en pixels, pas en rapport : sur une barre de cinq pixels, l'arrondi au
/// pixel de la boîte englobante pèse 20 % à lui seul, et un rapport n'y voudrait plus rien
/// dire. L'écart de hauteur, lui, ne dépend pas de la taille des deux barres.
#[test]
fn test_two_weights_are_told_apart_by_their_height_alone() {
    let typo = Typography::new();
    let track = GaugeLayout::world().track_height as f64;
    let empty = render(&typo, &proof_store(&[0.0]), 2.0);
    let mut heights = Vec::new();
    for weight in [0.2_f64, 0.8] {
        let inked = render(&typo, &proof_store(&[weight]), 2.0);
        let (_, y0, _, y1) = ink_bbox(&inked, &empty).expect("un poids non nul doit marquer");
        heights.push((y1 - y0) as f64);
    }
    let observed = heights[1] - heights[0];
    let expected = 0.6 * track * 2.0;
    assert!(
        (observed - expected).abs() <= 3.0,
        "0,8 devrait dépasser 0,2 de {expected:.1} px, écart observé {observed:.1} ({heights:?})"
    );
}

/// SCALE-1 — la réglette est semblable à elle-même à tous les zooms : rapportée à la largeur
/// de la carte, l'encre d'un même poids occupe la même fraction.
#[test]
fn test_scale_1_the_gauge_keeps_its_share_of_the_node_at_every_zoom() {
    let typo = Typography::new();
    let mut shares = Vec::new();
    for zoom in [0.5_f64, 1.0, 2.0, 4.0] {
        let inked = render(&typo, &proof_store(&[0.6]), zoom);
        let empty = render(&typo, &proof_store(&[0.0]), zoom);
        let (x0, y0, x1, y1) = ink_bbox(&inked, &empty).expect("le remplissage doit marquer");
        let node_width = NODE.0 * zoom;
        shares.push((zoom, (y1 - y0) as f64 / node_width, (x1 - x0) as f64 / node_width));
    }
    let (_, ref_h, ref_w) = shares[1];
    for &(zoom, h, w) in &shares {
        // Comparaison en pixels, pas en rapports : une carte de 150 px et une de 1200 px ne
        // peuvent pas porter le même rapport au millième, l'encre étant discrétisée au pixel.
        let node_width = NODE.0 * zoom;
        assert!(
            (h - ref_h).abs() * node_width <= 3.0,
            "zoom {zoom} : hauteur {h:.4} au lieu de {ref_h:.4} ({shares:?})"
        );
        assert!(
            (w - ref_w).abs() * node_width <= 3.0,
            "zoom {zoom} : largeur {w:.4} au lieu de {ref_w:.4} ({shares:?})"
        );
    }
}

/// SCALE-2 — sous le seuil de détail, la réglette garde ses colonnes et laisse tomber ses
/// sigles, comme la carte garde son cadre et laisse tomber son texte.
#[test]
fn test_scale_2_below_the_threshold_the_gauge_keeps_its_bars_and_drops_its_sigils() {
    let typo = Typography::new();
    let below = f64::from(WorldScale::SIMPLIFIED_BELOW) / 2.0;
    let weights = [1.0, 1.0, 1.0];

    // Sous le seuil, un sigle vide et un sigle plein donnent la même image.
    let mut naked = proof_store(&weights);
    for (id, _, _) in PROOF_DOMAINS {
        naked
            .try_update_domain(id, DomainPatch::new().with_icon(""))
            .expect("le domaine est au catalogue");
    }
    let with_sigils = render(&typo, &proof_store(&weights), below);
    let without = render(&typo, &naked, below);
    assert!(
        ink_bbox(&with_sigils, &without).is_none(),
        "sous le seuil, aucun glyphe ne doit être rastérisé"
    );

    // Les colonnes, elles, restent : la réglette est simplifiée, pas absente.
    let no_weight = render(&typo, &proof_store(&[0.0, 0.0, 0.0]), below);
    assert!(
        ink_bbox(&with_sigils, &no_weight).is_some(),
        "les remplissages doivent rester visibles sous le seuil"
    );
}

/// Un nœud à trois domaines dessine trois colonnes distinctes, côte à côte, sans recouvrement.
#[test]
fn test_several_domains_stay_readable_side_by_side() {
    let typo = Typography::new();
    let layout = GaugeLayout::world();
    // Chaque capture se compare à SON propre document de référence, celui qui porte les mêmes
    // domaines à poids nul : la différence est alors le remplissage seul, et non les pistes
    // que l'autre document ne dessine pas.
    let one = render(&typo, &proof_store(&[0.9]), 1.0);
    let one_empty = render(&typo, &proof_store(&[0.0]), 1.0);
    let three = render(&typo, &proof_store(&[0.9, 0.9, 0.9]), 1.0);
    let three_empty = render(&typo, &proof_store(&[0.0, 0.0, 0.0]), 1.0);

    let (x0_one, _, x1_one, _) = ink_bbox(&one, &one_empty).expect("une colonne");
    let (x0_three, _, x1_three, _) = ink_bbox(&three, &three_empty).expect("trois colonnes");

    assert_eq!(x0_one, x0_three, "la première colonne ne doit pas bouger");
    let widened = (x1_three - x1_one) as f32;
    let expected = 2.0 * (layout.bar_width + layout.bar_gap);
    assert!(
        (widened - expected).abs() <= 2.0,
        "deux colonnes de plus devraient ajouter {expected:.1} px, pas {widened:.1}"
    );
}

/// La capture demandée : un nœud qui porte ses domaines, à trois zooms, plus une image et une
/// membrane pour montrer que la réglette ne parle pas qu'aux cartes de texte.
#[test]
fn test_domain_gauge_png_capture() {
    let dir = std::path::Path::new("target/domain-gauge");
    std::fs::create_dir_all(dir).expect("dossier de capture");
    let typo = Typography::new();

    for zoom in [0.5_f64, 1.0, 2.0] {
        let pixmap = render(&typo, &proof_store(&[0.35, 0.7, 1.0]), zoom);
        pixmap
            .save_png(dir.join(format!("card-x{zoom:.1}.png")))
            .expect("écriture du png");
    }
    capture_scene_pass(&typo, dir);
}

/// Capture de la passe de scène : une image et une membrane, chacune avec sa réglette.
fn capture_scene_pass(typo: &Typography, dir: &std::path::Path) {
    let theme = Theme::dark();
    let mut store = proof_store(&[]);
    let board_id = store.project.active_board_id.clone();
    if let Some(board) = store.active_board_mut() {
        board.annotations.clear();
        board.images.push(BoardImage::new("img", 260.0, 200.0, 320.0, 200.0));
        board.annotations.push(Annotation::Membrane {
            id: "memb".into(),
            x: 520.0,
            y: 60.0,
            width: 420.0,
            height: 300.0,
            color: Some("#a78bfa".into()),
            text: Some("Membrane".into()),
            mode: Default::default(),
            curtains: Vec::new(),
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        });
    }
    for (index, (id, _, _)) in PROOF_DOMAINS.iter().enumerate() {
        let weight = 0.4 + 0.3 * index as f64;
        for node in ["img", "memb"] {
            store
                .try_assign_domain_to_node(&board_id, node, id, weight.min(1.0))
                .expect("le nœud et le domaine existent");
        }
    }

    let mut tints = DomainTints::new();
    tints.refresh(&store, &theme);
    let mut pixmap = Pixmap::new(CANVAS.0, CANVAS.1).expect("pixmap de la preuve");
    pixmap.fill(theme.bg_canvas);
    let ids: HashSet<&str> = ["img", "memb"].into_iter().collect();
    let pass = ViewPass {
        vp: Viewport { x: 40.0, y: 120.0, scale: 1.0 },
        visible_ids: &ids,
        header_h: 0.0,
    };
    let mut view = pixmap.as_mut();
    let mut cache = std::collections::HashMap::new();
    let mut failed = HashSet::new();
    crate::renderer::scene::draw_membranes(typo, &tints, &mut view, &store, pass);
    crate::renderer::scene::draw_images(&mut cache, &mut failed, typo, &tints, &mut view, &store, pass);
    pixmap.save_png(dir.join("scene-image-et-membrane.png")).expect("écriture du png");
}
