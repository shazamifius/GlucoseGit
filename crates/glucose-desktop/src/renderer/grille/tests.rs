//! Ce que la grille doit garantir : qu'elle **sert**, et qu'elle ne **ment** pas.
//!
//! # Les deux façons dont un cache de rendu peut échouer sans qu'on le voie
//!
//! La première est de ne jamais servir : il repeint tout à chaque image, rend les mêmes
//! pixels, et passe toutes les épreuves d'aspect. On ne le sait qu'au chronomètre — c'est la
//! leçon des vignettes, qui ont servi à zéro pour cent pendant cinq sessions. Ces tests lisent
//! donc les **comptes** de tuiles peintes et reprises.
//!
//! La seconde est de servir à tort : garder une tuile qui ne dit plus la vérité. Une image
//! sélectionnée puis désélectionnée, une photo arrivée après sa tuile — les deux cas sont
//! couverts, parce que ce sont exactement ceux que la conception du module prévoit.

use super::*;
use crate::renderer::Renderer;
use glucose_core::types::BoardImage;
use tiny_skia::Pixmap;

/// Un écran modeste, pour que les tests restent rapides.
const ECRAN: (u32, u32) = (512, 384);

/// Une photo opaque de couleur unie, décodée dans le magasin sous ce nom.
fn decoder(renderer: &mut Renderer, src: &str, couleur: [u8; 4]) {
    let mut pixmap = Pixmap::new(64, 64).expect("une image");
    for bloc in pixmap.data_mut().as_chunks_mut::<4>().0 {
        bloc.copy_from_slice(&couleur);
    }
    renderer.magasin.cache.insert(
        src.to_string(),
        crate::renderer::magasin::Entree::pour_test(crate::renderer::photo::Pyramide::nouvelle(
            pixmap,
        )),
    );
}

/// Un document de quelques photos, toutes décodées, sous une vue donnée.
fn document(vue: Viewport) -> (Store, Renderer) {
    let mut store = Store::new("Grille");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = vue;
    }
    let mut renderer = Renderer::new();
    for i in 0..6 {
        let src = format!("photo{i}.png");
        decoder(&mut renderer, &src, [40 * i as u8, 90, 160, 255]);
        let mut img = BoardImage::new(
            format!("i{i}"),
            (i % 3) as f64 * 180.0 + 100.0,
            (i / 3) as f64 * 160.0 + 100.0,
            120.0,
            90.0,
        );
        img.src = Some(src);
        store.add_image(&board, img);
    }
    // `add_image` selectionne ce qu'elle ajoute : un document de depart avec un cadre
    // autour de la derniere photo ferait passer ce cadre pour un defaut de la grille.
    store.clear_selection();
    renderer.sync_spatial_index(&store);
    (store, renderer)
}

fn interface() -> crate::ui::UiState {
    let mut ui = crate::ui::UiState::new();
    ui.current_toast = None;
    ui
}

fn sans_reperes<'a>() -> crate::params::SceneOverlay<'a> {
    let guides: &'static glucose_core::smart_align::SnapGuides =
        Box::leak(Box::new(glucose_core::smart_align::SnapGuides::default()));
    crate::params::SceneOverlay {
        guides,
        selection_box: None,
        editing: None,
    }
}

/// Rend une image entière de la scène, et rend les pixels.
fn une_image(renderer: &mut Renderer, store: &Store, cadrage: Cadrage) -> Pixmap {
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("l'ecran");
    let ui = interface();
    renderer.magasin.ouvrir();
    renderer.rendre_la_region(
        &mut pixmap.as_mut(),
        store,
        &ui,
        sans_reperes(),
        0.0,
        cadrage,
    );
    renderer.magasin.fermer();
    pixmap
}

fn vue(x: f64, y: f64, scale: f64) -> Viewport {
    Viewport { x, y, scale }
}

/// **Le régime se lit sur l'échelle et sur ce que l'œil tolère**, et sur rien d'autre.
#[test]
fn test_le_regime_suit_l_echelle_et_la_perception() {
    let plein = Cadrage::plein();
    assert_eq!(Regime::pour(plein, vue(0.0, 0.0, 1.0)), Regime::Exact);
    assert_eq!(Regime::pour(plein, vue(0.0, 0.0, 0.5)), Regime::Exact);
    assert_eq!(Regime::pour(plein, vue(0.0, 0.0, 4.0)), Regime::Exact);
    // Entre deux niveaux : l'œil décide.
    assert_eq!(Regime::pour(plein, vue(0.0, 0.0, 1.3)), Regime::Direct);
    assert_eq!(
        Regime::pour(plein.avec_degradation(true), vue(0.0, 0.0, 1.3)),
        Regime::Entre
    );
    // Une tuile ne se rend jamais par la grille, ni une scène réduite.
    assert_eq!(
        Regime::pour(Cadrage::tuile(0, (0.0, 0.0)), vue(0.0, 0.0, 1.0)),
        Regime::Direct
    );
    assert_eq!(
        Regime::pour(Cadrage::reduit(2), vue(0.0, 0.0, 1.0)),
        Regime::Direct
    );
}

/// **La grille sert.** La deuxième image identique ne peint aucune tuile.
///
/// C'est le test que les vignettes n'auraient jamais passé, et la seule preuve déterministe
/// qu'un cache de rendu fait ce qu'il promet.
#[test]
fn test_une_vue_immobile_ne_repeint_aucune_tuile() {
    let (store, mut renderer) = document(vue(0.0, 0.0, 1.0));
    let premiere = une_image(&mut renderer, &store, Cadrage::plein());
    let peintes = renderer.tuiles.peintes();
    assert!(
        peintes > 0,
        "la premiere image doit peindre des tuiles : {peintes}"
    );

    let seconde = une_image(&mut renderer, &store, Cadrage::plein());
    assert_eq!(
        renderer.tuiles.peintes(),
        peintes,
        "la seconde image ne doit rien repeindre"
    );
    assert!(renderer.tuiles.reprises() > 0, "et tout est repris");
    assert_eq!(
        premiere.data(),
        seconde.data(),
        "deux images identiques donnent les memes pixels"
    );
}

/// **Un déplacement ne repeint que ce qui entre.** Les tuiles déjà vues restent.
#[test]
fn test_un_deplacement_ne_repeint_que_ce_qui_entre() {
    let (store, mut renderer) = document(vue(0.0, 0.0, 1.0));
    une_image(&mut renderer, &store, Cadrage::plein());
    let peintes = renderer.tuiles.peintes();

    // Un pas de quelques pixels vers la droite et le bas : une colonne de tuiles entre par la
    // droite, et c'est tout ce qui peut avoir a se peindre. Les tuiles deja vues restent.
    let mut bougee = store.clone();
    if let Some(b) = bougee.active_board_mut() {
        b.viewport = vue(-3.0, -2.0, 1.0);
    }
    une_image(&mut renderer, &bougee, Cadrage::plein());
    let neuves = renderer.tuiles.peintes() - peintes;
    let colonne = ECRAN.1.div_ceil(COTE_TUILE) as u64 + 1;
    assert!(
        neuves <= colonne,
        "seule la colonne qui entre peut se peindre : {neuves} tuiles pour {colonne} au plus"
    );

    // Et revenir en arriere ne repeint rien de ce qui est encore a l'ecran.
    let peintes = renderer.tuiles.peintes();
    une_image(&mut renderer, &store, Cadrage::plein());
    assert!(
        renderer.tuiles.peintes() - peintes <= colonne,
        "revenir ne repeint que ce qui etait sorti de l'ecran"
    );
}

/// **La grille ne ment pas sur la sélection.** Les ornements ne sont pas dans les tuiles.
#[test]
fn test_selectionner_une_image_change_l_ecran_sans_repeindre_sa_tuile() {
    let (mut store, mut renderer) = document(vue(0.0, 0.0, 1.0));
    let avant = une_image(&mut renderer, &store, Cadrage::plein());
    let peintes = renderer.tuiles.peintes();

    store.select_image("i0".to_string(), false);
    let apres = une_image(&mut renderer, &store, Cadrage::plein());
    assert_ne!(
        avant.data(),
        apres.data(),
        "le cadre de selection doit apparaitre"
    );
    assert_eq!(
        renderer.tuiles.peintes(),
        peintes,
        "sans qu'aucune tuile soit repeinte : la selection n'appartient pas au document"
    );

    store.clear_selection();
    let redevenu = une_image(&mut renderer, &store, Cadrage::plein());
    assert_eq!(
        avant.data(),
        redevenu.data(),
        "et il disparait avec elle -- une tuile qui l'aurait garde mentirait"
    );
}

/// **Une photo encore en chemin ne fige pas sa tuile.** Elle se repeint quand les octets
/// arrivent.
#[test]
fn test_une_photo_en_chemin_ne_fige_pas_sa_tuile() {
    let (mut store, mut renderer) = document(vue(0.0, 0.0, 1.0));
    // Une septième photo, dont le fichier n'est pas décodé.
    let board = store.project.active_board_id.clone();
    let mut img = BoardImage::new("tard", 300.0, 300.0, 100.0, 80.0);
    img.src = Some("tard.png".to_string());
    store.add_image(&board, img);
    renderer.sync_spatial_index(&store);

    let en_chemin = une_image(&mut renderer, &store, Cadrage::plein());
    // La photo arrive.
    decoder(&mut renderer, "tard.png", [250, 10, 10, 255]);
    let arrivee = une_image(&mut renderer, &store, Cadrage::plein());
    assert_ne!(
        en_chemin.data(),
        arrivee.data(),
        "la photo doit paraitre a l'image ou ses octets sont la"
    );
}

/// **Modifier le document invalide exactement les tuiles touchées.**
#[test]
fn test_deplacer_une_photo_repeint_ses_tuiles_et_pas_les_autres() {
    let (mut store, mut renderer) = document(vue(0.0, 0.0, 1.0));
    une_image(&mut renderer, &store, Cadrage::plein());
    let peintes = renderer.tuiles.peintes();
    let gardes = renderer.tuiles.gardes();

    let board = store.project.active_board_id.clone();
    store.update_image(&board, "i0", |img| img.x += 7.0);
    renderer.sync_spatial_index(&store);
    une_image(&mut renderer, &store, Cadrage::plein());
    let repeintes = renderer.tuiles.peintes() - peintes;
    assert!(
        repeintes > 0,
        "la tuile de la photo bougee doit se repeindre"
    );
    assert!(
        repeintes < gardes as u64,
        "mais pas toutes : {repeintes} repeintes pour {gardes} gardees"
    );
}

/// **Une photo à cheval sur deux tuiles se voit en entier.**
///
/// Le premier branchement l'a raté : `ce_que_porte` prenait le centre d'une photo pour son
/// coin, et la moitié gauche d'une image posée sur une frontière de tuile disparaissait. La
/// preuve de redimensionnement l'a attrapé — « bord à 500 px, le document dit 400 » — et ce
/// test le garde.
#[test]
fn test_une_photo_a_cheval_sur_deux_tuiles_se_voit_en_entier() {
    let mut store = Store::new("Cheval");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = vue(500.0, 400.0, 1.0);
    }
    let mut renderer = Renderer::new();
    decoder(&mut renderer, "p.png", [250, 120, 30, 255]);
    // Centrée sur l'origine du monde, donc exactement à cheval sur la frontière x = 0.
    let mut img = BoardImage::new("probe", 0.0, 0.0, 200.0, 100.0);
    img.src = Some("p.png".to_string());
    store.add_image(&board, img);
    store.clear_selection();
    renderer.sync_spatial_index(&store);

    let mut pixmap = Pixmap::new(1280, 720).expect("l'ecran");
    let ui = interface();
    renderer.magasin.ouvrir();
    renderer.rendre_la_region(
        &mut pixmap.as_mut(),
        &store,
        &ui,
        sans_reperes(),
        0.0,
        Cadrage::plein(),
    );
    renderer.magasin.fermer();

    let (mut x0, mut x1) = (u32::MAX, 0);
    for (i, p) in pixmap.data().chunks(4).enumerate() {
        if p[0] == 250 && p[1] == 120 && p[2] == 30 {
            let x = i as u32 % 1280;
            x0 = x0.min(x);
            x1 = x1.max(x);
        }
    }
    // Le document la place de 400 à 600 a l'ecran.
    assert_eq!(
        x0, 400,
        "le bord gauche de la photo doit etre la ou le document le met"
    );
    assert_eq!(x1, 599, "et le bord droit aussi");
    // Centree sur l'origine, elle est a cheval en x ET en y : quatre tuiles la portent.
    assert_eq!(
        renderer.tuiles.peintes(),
        4,
        "les quatre tuiles qui portent la photo doivent etre peintes"
    );
}

/// **Aucune couture entre deux tuiles, à aucune échelle ni à aucune position.**
///
/// L'utilisateur a vu des « croix noires se former » dès qu'il bougeait lentement entre deux
/// niveaux dyadiques : chaque tuile s'arrondissait de son côté, et `x + 332,4` donnait 342
/// pour l'une et 343 pour la suivante — un pixel de fond entre les deux. Une photo unie posée
/// sur plusieurs tuiles doit rester unie, à toute échelle et à toute phase sous-pixel.
#[test]
fn test_aucune_couture_entre_les_tuiles_a_une_echelle_non_dyadique() {
    let couleur = [250u8, 120, 30, 255];
    for (echelle, phase) in [
        (1.3, 0.37),
        (1.3, 0.62),
        (1.7, 0.5),
        (1.05, 0.9),
        (0.6, 0.3),
    ] {
        let mut store = Store::new("Couture");
        let board = store.project.active_board_id.clone();
        if let Some(b) = store.active_board_mut() {
            b.annotations.clear();
            b.viewport = vue(100.0 + phase, 80.0 + phase, echelle);
        }
        let mut renderer = Renderer::new();
        decoder(&mut renderer, "unie.png", couleur);
        // Une photo qui traverse plusieurs frontières de tuiles dans les deux sens.
        let mut img = BoardImage::new("large", 500.0, 400.0, 900.0, 700.0);
        img.src = Some("unie.png".to_string());
        store.add_image(&board, img);
        store.clear_selection();
        renderer.sync_spatial_index(&store);

        let mut pixmap = Pixmap::new(1280, 900).expect("l'ecran");
        let ui = interface();
        renderer.magasin.ouvrir();
        renderer.rendre_la_region(
            &mut pixmap.as_mut(),
            &store,
            &ui,
            sans_reperes(),
            0.0,
            Cadrage::plein().sous_le_regard(Regard {
                degradation_permise: false,
                en_mouvement: true,
            }),
        );
        renderer.magasin.fermer();

        // L'intérieur de la photo à l'écran, à trois pixels des bords pour ignorer l'arrondi
        // de son propre contour.
        let (sx, sy) = crate::canvas::world_to_screen(50.0, 50.0, &store.viewport());
        let (ex, ey) = crate::canvas::world_to_screen(950.0, 750.0, &store.viewport());
        let (ex, ey) = (ex.min(1280.0 - 3.0), ey.min(900.0 - 3.0));
        let mut trous = Vec::new();
        for y in (sy.max(3.0) as u32 + 3)..(ey as u32 - 3) {
            for x in (sx.max(3.0) as u32 + 3)..(ex as u32 - 3) {
                let i = ((y * 1280 + x) * 4) as usize;
                if pixmap.data()[i..i + 4] != couleur {
                    trous.push((x, y));
                }
            }
        }
        assert!(
            trous.is_empty(),
            "a l'echelle {echelle} et a la phase {phase}, {} pixels de fond dans une photo \
             unie -- premiers : {:?}",
            trous.len(),
            &trous[..trous.len().min(6)]
        );
    }
}

/// Un document dont les photos **pavent** l'écran : le cas du mur de photos, celui où le fond
/// n'a aucune chance d'être vu.
fn mur(vue: Viewport) -> (Store, Renderer) {
    let mut store = Store::new("Mur");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = vue;
    }
    let mut renderer = Renderer::new();
    decoder(&mut renderer, "mur.png", [60, 120, 200, 255]);
    // Des photos jointives, largement au-delà des bords de l'écran : la tuile la plus
    // excentrée est couverte comme les autres.
    let cote = 200.0;
    for ligne in -4..8 {
        for colonne in -4..10 {
            let mut img = BoardImage::new(
                format!("m{ligne}_{colonne}"),
                f64::from(colonne) * cote,
                f64::from(ligne) * cote,
                cote,
                cote,
            );
            img.src = Some("mur.png".to_string());
            store.add_image(&board, img);
        }
    }
    store.clear_selection();
    renderer.sync_spatial_index(&store);
    (store, renderer)
}

/// Rend une image sur un fond **donné**, pour voir si ce fond transparaît.
fn sur_le_fond(renderer: &mut Renderer, store: &Store, cadrage: Cadrage, fond: [u8; 4]) -> Pixmap {
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("l'ecran");
    for bloc in pixmap.data_mut().as_chunks_mut::<4>().0 {
        bloc.copy_from_slice(&fond);
    }
    let ui = interface();
    renderer.magasin.ouvrir();
    renderer.rendre_la_region(
        &mut pixmap.as_mut(),
        store,
        &ui,
        sans_reperes(),
        0.0,
        cadrage,
    );
    renderer.magasin.fermer();
    pixmap
}

/// **Quand la grille annonce qu'elle recouvre tout, le fond ne se voit pas.**
///
/// C'est la seule formulation qui prouve ce qu'il faut : si un seul pixel du fond survivait,
/// deux fonds différents donneraient deux images différentes. Aucun bord n'est calculé ici,
/// aucune tolérance n'est admise — c'est une égalité au bit près, sur l'écran entier.
///
/// Et le contraire se vérifie aussi, sinon le test passerait pour une scène vide : sur un
/// document clairsemé, la grille dit non, et le fond se voit bel et bien.
#[test]
fn test_quand_la_grille_recouvre_tout_le_fond_ne_se_voit_pas() {
    let cadrage = Cadrage::plein();
    let (store, mut renderer) = mur(vue(-150.0, -150.0, 1.0));
    // La première image peint les tuiles ; c'est la suivante qui peut sauter le fond.
    une_image(&mut renderer, &store, cadrage);
    let rouge = sur_le_fond(&mut renderer, &store, cadrage, [255, 0, 0, 255]);
    let noir = sur_le_fond(&mut renderer, &store, cadrage, [0, 0, 0, 255]);
    assert_eq!(
        rouge.data(),
        noir.data(),
        "le fond transparaît alors que la grille annonce le recouvrir"
    );
    assert!(
        crate::perf::valeur_du_compteur("fond_saute").unwrap_or(0.0) > 0.0,
        "le fond aurait dû être sauté : sinon ce test ne prouve que l'opacité des photos"
    );

    // Un document clairsemé : la grille dit non, et le fond se voit.
    let (store, mut renderer) = document(vue(0.0, 0.0, 1.0));
    une_image(&mut renderer, &store, cadrage);
    let rouge = sur_le_fond(&mut renderer, &store, cadrage, [255, 0, 0, 255]);
    let noir = sur_le_fond(&mut renderer, &store, cadrage, [0, 0, 0, 255]);
    assert_eq!(
        rouge.data(),
        noir.data(),
        "sur un document clairsemé, le fond EST redessiné : les deux images doivent être          identiques, et par le fond peint, pas par un fond sauté"
    );
    assert_eq!(
        crate::perf::valeur_du_compteur("fond_saute").unwrap_or(0.0),
        0.0,
        "un document clairsemé ne doit jamais faire sauter le fond"
    );
}
