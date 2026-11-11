//! PICK-1 — regles d'arbitrage du clic, testees depuis l'exterieur du crate.
//! (Anciens tests unitaires de `hit_priority.rs`, deplaces lors de son decoupage.)

use glucose_core::hit_priority::*;
use glucose_core::types::{Annotation, BoardImage};
fn img(id: &str, x: f64, y: f64, w: f64, h: f64, locked: bool) -> BoardImage {
    let mut img = BoardImage::new(id, x, y, w, h);
    img.locked = locked;
    img
}

fn membrane(id: &str, x: f64, y: f64, w: f64, h: f64, text: Option<&str>) -> Annotation {
    Annotation::Membrane {
        id: id.to_string(),
        x,
        y,
        width: w,
        height: h,
        color: None,
        text: text.map(String::from),
        mode: glucose_core::types::MembraneMode::Classic,
        curtains: Vec::new(),
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

fn text_ann(id: &str, x: f64, y: f64, w: f64, h: f64) -> Annotation {
    Annotation::Text {
        id: id.to_string(),
        x,
        y,
        width: Some(w),
        height: Some(h),
        text: "hello".to_string(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

#[test]
fn test_pick_priority_image_in_membrane() {
    let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
    let images = [img("I1", 500.0, 400.0, 200.0, 150.0, false)];
    let annotations = [memb];
    let folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 500.0,
        wy: 400.0,
        scale: 1.0,
        images: &images,
        annotations: &annotations,
        folders: &folders,
        selected_image_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_annotation_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_folder_id: None,
        noeuds: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].id, "I1");
    assert_eq!(cands[0].kind, PickKind::Image);
    assert_eq!(cands[1].id, "M1");
    assert_eq!(cands[1].kind, PickKind::MembraneBody);
}

#[test]
fn test_pick_membrane_edge() {
    let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
    let images = [img("I1", 5.0, 400.0, 200.0, 150.0, false)];
    let annotations = [memb];
    let folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 5.0,
        wy: 400.0,
        scale: 1.0,
        images: &images,
        annotations: &annotations,
        folders: &folders,
        selected_image_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_annotation_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_folder_id: None,
        noeuds: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].id, "M1");
    assert_eq!(cands[0].kind, PickKind::MembraneEdge);
}

#[test]
fn test_nested_membranes_smallest_wins() {
    let inner = membrane("PETITE", 400.0, 300.0, 200.0, 200.0, None);
    let outer = membrane("GRANDE", 0.0, 0.0, 1000.0, 800.0, None);
    let annotations = [outer, inner];
    let empty_images = [];
    let empty_folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 500.0,
        wy: 400.0,
        scale: 1.0,
        images: &empty_images,
        annotations: &annotations,
        folders: &empty_folders,
        selected_image_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_annotation_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_folder_id: None,
        noeuds: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].id, "PETITE");
}

/// **PICK-2 — un texte posé dans une membrane se prend en plein dedans**, et le re-clic
/// descend jusqu'à la membrane : le texte n'est plus le fond de la pile (registre de Tauri,
/// n° 8 : « en plein dans un texte posé dans une membrane → le texte » ; « recliquer sans
/// bouger passe à l'élément suivant dessous »).
#[test]
fn test_pick_2_un_texte_dans_une_membrane_se_prend_et_le_reclic_descend() {
    let memb = membrane("M1", 0.0, 0.0, 1000.0, 800.0, None);
    let t = text_ann("T1", 400.0, 380.0, 200.0, 60.0);
    let annotations = [memb, t];
    let empty_images = [];
    let empty_folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 500.0,
        wy: 400.0,
        scale: 1.0,
        images: &empty_images,
        annotations: &annotations,
        folders: &empty_folders,
        selected_image_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_annotation_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_folder_id: None,
        noeuds: None,
    };

    let cands = collect_candidates(&input);
    assert_eq!(cands[0].id, "T1");
    assert_eq!(cands[1].id, "M1");

    // Un premier clic, puis un re-clic lent — au-delà de la fenêtre du double-clic — au même
    // endroit : au relâchement, la membrane.
    let (premier, cycle) = pick_at_down(&cands, None, 50.0, 50.0, 1_000, PickOptions::default());
    assert_eq!(premier.map(|c| c.id), Some("T1".to_string()));
    let (_, cycle) = advance_on_release(&cands, cycle.as_ref(), 1_060);
    let lent = 1_060 + pick_consts::DBLCLICK_MS + 100;
    let (_, cycle) = pick_at_down(
        &cands,
        cycle.as_ref(),
        50.0,
        50.0,
        lent,
        PickOptions::default(),
    );
    let (dessous, _) = advance_on_release(&cands, cycle.as_ref(), lent + 60);
    assert_eq!(
        dessous.map(|c| c.id),
        Some("M1".to_string()),
        "le re-clic lent sur un texte descend a ce qu'il y a dessous"
    );
}

#[test]
fn test_collect_candidates_indexed_matches_naive() {
    use glucose_core::quadtree::SpatialHash;

    let mut hash = SpatialHash::new(500.0);
    let mut images = Vec::new();
    let mut annotations = Vec::new();

    for i in 0..20 {
        let id = format!("I{}", i);
        let img_obj = img(&id, i as f64 * 1000.0, 0.0, 200.0, 200.0, false);
        hash.insert_image(&img_obj);
        images.push(img_obj);
    }

    let t = text_ann("T1", 5000.0, 0.0, 100.0, 40.0);
    hash.insert_annotation(&t);
    annotations.push(t);

    let empty_folders = [];
    let empty: [String; 0] = [];

    let input = PickInput {
        wx: 5020.0,
        wy: 10.0,
        scale: 1.0,
        images: &images,
        annotations: &annotations,
        folders: &empty_folders,
        selected_image_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_annotation_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_folder_id: None,
        noeuds: None,
    };

    let naive = collect_candidates(&input);
    let indexed = collect_candidates_indexed(&input, &hash);

    assert_eq!(naive.len(), indexed.len());
    assert_eq!(naive[0].id, indexed[0].id);
    // La carte est peinte au-dessus de l'image qu'elle recouvre : c'est elle qu'on prend
    // (PICK-2 ; l'ordre de Tauri rendait l'image).
    assert_eq!(naive[0].id, "T1");
    assert_eq!(naive[1].id, "I5");

    // Clic loin dans le vide
    let empty_input = PickInput {
        wx: 99999.0,
        wy: 99999.0,
        scale: 1.0,
        images: &images,
        annotations: &annotations,
        folders: &empty_folders,
        selected_image_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_annotation_ids: &glucose_core::store::choix::Choix::from(empty.to_vec()),
        selected_folder_id: None,
        noeuds: None,
    };
    let empty_indexed = collect_candidates_indexed(&empty_input, &hash);
    assert!(empty_indexed.is_empty());
}

/// **TITRE-1 — on saisit une grande membrane par son nom** : la bande du titre, au-dessus du bord,
/// grandit avec lui. 170 au-dessus d'une membrane de 3 200 × 2 400 (un titre de 180 de haut) la
/// prend, 250 non — au-dessus de son nom, il n'y a plus rien à saisir (fiche 106 : la bande est
/// celle où le titre est dessiné, l'encre que l'index connaît) ; 250 au-dessus d'une membrane neuve
/// (un titre de 18), rien ; sans titre, rien.
#[test]
fn test_titre_1_la_bande_du_titre_grandit_avec_lui() {
    let clic = |m: Annotation, wy: f64| {
        let annotations = [m];
        let (images, folders): ([BoardImage; 0], [glucose_core::types::CanvasFolder; 0]) = ([], []);
        let rien: [String; 0] = [];
        let input = PickInput {
            wx: 100.0,
            wy,
            scale: 1.0,
            images: &images,
            annotations: &annotations,
            folders: &folders,
            selected_image_ids: &glucose_core::store::choix::Choix::from(rien.to_vec()),
            selected_annotation_ids: &glucose_core::store::choix::Choix::from(rien.to_vec()),
            selected_folder_id: None,
            noeuds: None,
        };
        collect_candidates(&input).first().map(|c| c.kind)
    };
    let grande = membrane("G", 0.0, 0.0, 3200.0, 2400.0, Some("Eau"));
    assert_eq!(clic(grande.clone(), -170.0), Some(PickKind::MembraneEdge));
    assert_eq!(
        clic(grande, -250.0),
        None,
        "au-dessus de son nom, rien : la bande est celle où il est dessiné (fiche 106)"
    );
    let neuve = membrane("N", 0.0, 0.0, 320.0, 240.0, Some("Eau"));
    assert_eq!(
        clic(neuve, -250.0),
        None,
        "la bande d'une membrane neuve reste la sienne"
    );
    let muette = membrane("M", 0.0, 0.0, 3200.0, 2400.0, None);
    assert_eq!(clic(muette, -250.0), None, "sans titre, pas de bande");
    let vide = membrane("V", 0.0, 0.0, 3200.0, 2400.0, Some(""));
    assert_eq!(
        clic(vide, -20.0),
        None,
        "un titre vide ne se dessine pas : pas de bande"
    );
}

/// Un tableau semé de tout : des images, des textes, des membranes emboîtées, des flèches qui le
/// traversent — ce que la carte de Wikipédia porte, en petit.
fn tableau_seme(graine: u64) -> glucose_core::types::Board {
    let mut s = graine;
    let mut tirer = move |a: f64, b: f64| {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        a + (b - a) * ((s >> 11) as f64 / (1u64 << 53) as f64)
    };
    let mut b = glucose_core::types::Board::default();
    for i in 0..60 {
        let (x, y) = (tirer(-3000.0, 3000.0), tirer(-2000.0, 2000.0));
        b.images.push(img(
            &format!("I{i}"),
            x,
            y,
            tirer(50.0, 400.0),
            tirer(50.0, 400.0),
            i % 7 == 0,
        ));
    }
    for i in 0..40 {
        let (x, y) = (tirer(-3000.0, 2000.0), tirer(-2000.0, 1500.0));
        let (l, h) = (tirer(200.0, 2500.0), tirer(200.0, 2000.0));
        b.annotations
            .push(membrane(&format!("M{i}"), x, y, l, h, Some("Région")));
    }
    for i in 0..120 {
        let (x, y) = (tirer(-3000.0, 3000.0), tirer(-2000.0, 2000.0));
        b.annotations.push(text_ann(
            &format!("T{i}"),
            x,
            y,
            tirer(80.0, 300.0),
            tirer(30.0, 200.0),
        ));
    }
    for i in 0..80 {
        b.annotations.push(Annotation::arrow(
            format!("A{i}"),
            tirer(-3000.0, 3000.0),
            tirer(-2000.0, 2000.0),
            tirer(-3000.0, 3000.0),
            tirer(-2000.0, 2000.0),
        ));
    }
    b
}

/// **SURVOL-1 — viser par les rangs de l'index rend ce que la visée naïve rend** (fiche 106) :
/// un tableau entier indexé, deux mille points (dans le vide, sur les bords, sous des flèches qui
/// traversent), à trois échelles — les mêmes candidats, dans le même ordre. Et des tranches qui
/// ne sont pas celles du tableau indexé (un nœud de moins, un nœud remplacé à sa place) : le nom
/// fait foi, la réponse de l'ancienne visée.
#[test]
fn test_survol_1_viser_par_les_rangs_rend_ce_que_rend_la_visee_naive() {
    use glucose_core::quadtree::SpatialHash;
    let empty: [String; 0] = [];
    let aucun = glucose_core::store::choix::Choix::from(empty.to_vec());
    let dossiers = [];
    let board = tableau_seme(7);
    let mut index = SpatialHash::new(500.0);
    index.index_board(&board);
    // Des tranches qui ne sont pas celles du tableau indexé : un nœud de moins ; puis la même
    // longueur, un nœud remplacé à sa place.
    let mut moins = board.clone();
    moins.annotations.remove(3);
    // Le remplaçant prend la **place** de l'ancien, sous un autre nom : la position que l'index
    // connaît désigne alors un nœud qu'il ne connaît pas. Posé ailleurs, il n'était jamais sous le
    // pointeur là où l'index cherchait, et une visée qui ne vérifiait pas ses rangs passait (le
    // sabotage « un rang ne se vérifie pas » était aveugle, fiche 106).
    let ancien = board.annotations[10].rect().expect("une boîte");
    let mut remplace = board.clone();
    remplace.annotations[10] = text_ann(
        "T-neuf",
        ancien.left,
        ancien.top,
        ancien.width.max(1.0),
        ancien.height.max(1.0),
    );
    let mut s = 11u64;
    let mut tirer = move |a: f64, b: f64| {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        a + (b - a) * ((s >> 11) as f64 / (1u64 << 53) as f64)
    };
    let mut non_vides = 0;
    for k in 0..2000 {
        let scale = [0.05, 1.0, 4.0][k % 3];
        for (tranches, indexees) in [(&board, true), (&moins, false), (&remplace, false)] {
            let input = PickInput {
                wx: tirer(-3500.0, 3500.0),
                wy: tirer(-2500.0, 2500.0),
                scale,
                images: &tranches.images,
                annotations: &tranches.annotations,
                folders: &dossiers,
                selected_image_ids: &aucun,
                selected_annotation_ids: &aucun,
                selected_folder_id: None,
                noeuds: None,
            };
            // Le tableau indexé : la visée naïve, sur tout le tableau. D'autres tranches :
            // l'ancienne visée, par les noms que l'index connaît près du point.
            let naive = if indexees {
                collect_candidates(&input)
            } else {
                let marge = pick_consts::HANDLE_SLOP_PX * 2.0 / scale;
                let pres = index.query_rect_refs(input.wx, input.wy, input.wx, input.wy, marge);
                if pres.is_empty() {
                    Vec::new()
                } else {
                    let images: Vec<BoardImage> = tranches
                        .images
                        .iter()
                        .filter(|i| pres.contains(i.id.as_str()))
                        .cloned()
                        .collect();
                    let annotations: Vec<Annotation> = tranches
                        .annotations
                        .iter()
                        .filter(|a| pres.contains(a.id()))
                        .cloned()
                        .collect();
                    collect_candidates(&PickInput {
                        images: &images,
                        annotations: &annotations,
                        ..input
                    })
                }
            };
            let indexed = collect_candidates_indexed(&input, &index);
            let ids =
                |c: &[PickCandidate]| c.iter().map(|c| (c.id.clone(), c.kind)).collect::<Vec<_>>();
            assert_eq!(
                ids(&indexed),
                ids(&naive),
                "au point ({}, {}) à l'échelle {scale}",
                input.wx,
                input.wy
            );
            non_vides += usize::from(!naive.is_empty());
        }
    }
    assert!(
        non_vides > 2000,
        "l'épreuve doit viser des nœuds : {non_vides}"
    );
}
