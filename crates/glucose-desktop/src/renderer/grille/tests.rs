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
    crate::params::SceneOverlay::sans_rien(guides)
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
    // Une tuile ne se rend jamais par la grille : ce serait se rendre soi-même.
    assert_eq!(
        Regime::pour(Cadrage::tuile(0, (0.0, 0.0)), vue(0.0, 0.0, 1.0)),
        Regime::Direct
    );
}

/// **Une scène réduite passe par la grille**, et ce test dit l'inverse de ce qu'il disait.
///
/// Il exigeait `Direct`, au motif qu'une scène réduite est « déjà une pixelisation ». Le
/// raisonnement confondait deux choses : la grille n'est pas un moyen de dégrader, c'est un
/// **cache**. L'en priver faisait repeindre la scène entière à chaque image, précisément
/// quand on cherchait à la rendre moins chère -- et cela bouclait, puisque le modèle de
/// résolution lisait ce surcoût comme une raison de réduire davantage.
///
/// Trois chroniques de terrain d'affilée l'ont montré : `agrandir` premier poste réel du
/// zoom, `grille` absente de son profil, et jusqu'à 80 % des images rendues plus petites.
///
/// Rien ne s'y opposait : `cadrer` divise l'échelle et la translation par `f`, donc une scène
/// réduite est une **vue** comme une autre, et son niveau dyadique suit de lui-même.
#[test]
fn test_une_scene_reduite_passe_par_la_grille_comme_les_autres() {
    // A l'echelle dyadique reduite, le chemin exact -- celui qui ne coute qu'un deplacement
    // de memoire par ligne.
    assert_eq!(
        Regime::pour(Cadrage::reduit(2), vue(0.0, 0.0, 1.0)),
        Regime::Exact
    );
    // Entre deux niveaux, la meme regle que partout : l'oeil decide. Une scene reduite l'est
    // TOUJOURS en mouvement -- c'est le plafond de la perception qui a permis le facteur --
    // donc elle tombe toujours du cote ou la grille sert.
    assert_eq!(
        Regime::pour(Cadrage::reduit(2), vue(0.0, 0.0, 1.3)),
        Regime::Entre
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

// ── RECADRAGE-1 : ce qu'on retire ne se voit plus, sur les deux chemins du processeur ────────

/// Une photo dont le quart gauche est une bande noire, décodée dans le magasin.
///
/// C'est la forme d'une image en boîte aux lettres tournée d'un quart : la bande est ce qu'un
/// recadrage doit faire disparaître, et le reste est blanc pour que la frontière se lise.
fn decoder_avec_une_bande(renderer: &mut Renderer, src: &str) {
    let mut pixmap = Pixmap::new(64, 64).expect("une image");
    let largeur = pixmap.width() as usize;
    for (i, bloc) in pixmap
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let x = i % largeur;
        let couleur = if x < largeur / 4 {
            [0, 0, 0, 255]
        } else {
            [255, 255, 255, 255]
        };
        bloc.copy_from_slice(&couleur);
    }
    renderer.magasin.cache.insert(
        src.to_string(),
        crate::renderer::magasin::Entree::pour_test(crate::renderer::photo::Pyramide::nouvelle(
            pixmap,
        )),
    );
}

/// Un document d'une seule photo à bande, cadrée ou non, sous une vue donnée.
fn document_a_bande(vue: Viewport, crop: glucose_core::types::Recadrage) -> (Store, Renderer) {
    let mut store = Store::new("Recadrage");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = vue;
    }
    let mut renderer = Renderer::new();
    decoder_avec_une_bande(&mut renderer, "bande.png");
    let mut img = BoardImage::new("i0", 256.0, 192.0, 200.0, 200.0);
    img.src = Some("bande.png".to_string());
    img.crop = crop;
    store.add_image(&board, img);
    store.clear_selection();
    renderer.sync_spatial_index(&store);
    (store, renderer)
}

/// Le pixel à `(x, y)` de l'écran, en RGBA.
fn pixel(p: &Pixmap, x: u32, y: u32) -> [u8; 4] {
    let i = (y * p.width() + x) as usize * 4;
    let d = p.data();
    [d[i], d[i + 1], d[i + 2], d[i + 3]]
}

/// **Retirer le quart gauche fait disparaître la bande noire**, sur le chemin des tuiles
/// comme sur le chemin direct.
///
/// La photo de 200 × 200 est centrée en (256, 192) : sa boîte va de x = 156 à 356. Sans
/// recadrage, sa bande noire couvre x ∈ [156, 206[ ; avec un recadrage d'un quart à gauche,
/// c'est le blanc qui doit y être — la source entière se pose sur 266 pixels de large, dont
/// les 66 premiers sont hors de la boîte et ne s'écrivent pas.
///
/// **Vérifié à l'envers dans le même test** : la même scène sans recadrage montre la bande.
/// Sans ce contrepoint, un test qui trouverait du blanc partout passerait aussi le jour où la
/// photo cesserait de se dessiner.
#[test]
fn test_retirer_le_quart_gauche_fait_disparaitre_la_bande_sur_les_deux_chemins() {
    let sans = glucose_core::types::Recadrage::ENTIER;
    let quart = glucose_core::types::Recadrage::depuis_les_marges(0.25, 0.0, 0.0, 0.0);
    // Échelle 1 : le régime est Exact, les tuiles peignent. Échelle 1,3 : Direct.
    for (echelle, chemin) in [(1.0, "tuiles"), (1.3, "direct")] {
        let v = vue(0.0, 0.0, echelle);
        let cadrage = Cadrage::plein();

        let (store, mut renderer) = document_a_bande(v, sans);
        let temoin = une_image(&mut renderer, &store, cadrage);
        let (store, mut renderer) = document_a_bande(v, quart);
        let cadre = une_image(&mut renderer, &store, cadrage);

        // Un point dans la bande, à un quart de la boîte depuis la gauche, au milieu en hauteur.
        let (sx, sy, sw, _) = (156.0 * echelle, 92.0 * echelle, 200.0 * echelle, 0.0);
        let x = (sx + sw * 0.12) as u32;
        let y = (sy + 100.0 * echelle) as u32;
        assert_eq!(
            pixel(&temoin, x, y),
            [0, 0, 0, 255],
            "chemin {chemin} : sans recadrage, la bande noire doit etre la"
        );
        assert_eq!(
            pixel(&cadre, x, y),
            [255, 255, 255, 255],
            "chemin {chemin} : le quart gauche retire, c'est le blanc qui doit paraitre"
        );
        // Et la boîte n'a pas bougé : juste à gauche d'elle, les deux images sont identiques.
        let hors = (sx - 4.0) as u32;
        assert_eq!(
            pixel(&temoin, hors, y),
            pixel(&cadre, hors, y),
            "chemin {chemin} : rien ne doit s'ecrire hors de la boite"
        );
    }
}
