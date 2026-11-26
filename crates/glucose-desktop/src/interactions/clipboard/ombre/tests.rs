//! **L'ombre des nœuds qu'on emporte** (Q10, fiche 155) : à la taille où on les voit, jamais plus
//! de 400 points, en pixels entiers à la densité de l'écran, le point de la main à sa place,
//! translucide sur un fond transparent.

use crate::app::GlucoseApp;
use crate::interactions::resize::tests::text_card;
use crate::renderer::export::debord;
use glucose_core::types::Viewport;

/// Un texte de `largeur` unités, choisi seul, vu à l'échelle `echelle`.
fn un_texte_choisi(largeur: f64, echelle: f64) -> (GlucoseApp, String) {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    let texte = text_card("a", 0.0, 0.0, largeur, "une ombre");
    app.store.add_annotation(&board, texte);
    app.store.set_selected_annotation_ids(vec!["a".into()]);
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: echelle,
    };
    app.store.set_viewport(&board, vue);
    (app, board)
}

/// **À la taille où on le voit** : 200 unités à moitié, cent points et la marge de l'export ; la
/// main, à son point.
#[test]
fn ombre_1_a_la_taille_ou_on_le_voit_et_la_main_a_sa_place() {
    let (app, board) = un_texte_choisi(200.0, 0.5);
    let ombre = app
        .ombre_des_noeuds(&board, (40.0, 20.0), 1.0)
        .expect("une ombre");
    let marge = debord(0.5);
    assert_eq!(ombre.densite, 1);
    assert_eq!(ombre.largeur, (100.0 + 2.0 * marge).ceil() as u32);
    let attendu = ((20.0 + marge).round() as i32, (10.0 + marge).round() as i32);
    assert_eq!(ombre.point, attendu);
    assert_eq!(
        ombre.rgba.len(),
        (ombre.largeur * ombre.hauteur * 4) as usize
    );
}

/// **Un grand nœud tient dans 400 points**, et sur un écran à 150 %, ses pixels vont deux par
/// point — un nombre pair, comme Wayland l'exige d'un tampon à l'échelle 2. Sous Windows, qui
/// montre l'ombre pixel pour pixel, un et demi : 600 pixels ([`densite_de_l_ombre`]).
///
/// [`densite_de_l_ombre`]: crate::plateforme::densite_de_l_ombre
#[test]
fn ombre_1_un_grand_noeud_tient_dans_400_points_a_la_densite_de_l_ecran() {
    let (app, board) = un_texte_choisi(2000.0, 1.0);
    let ombre = app
        .ombre_des_noeuds(&board, (2000.0, 0.0), 1.0)
        .expect("une ombre");
    assert!((390..=400).contains(&ombre.largeur), "{}", ombre.largeur);
    // Réduit, jamais rogné : son bord droit, marge comprise, reste dans l'ombre.
    assert!(ombre.point.0 < ombre.largeur as i32, "{:?}", ombre.point);
    let ombre = app
        .ombre_des_noeuds(&board, (0.0, 0.0), 1.5)
        .expect("une ombre");
    let densite = crate::plateforme::densite_de_l_ombre(1.5);
    let plafond = (400.0 * densite) as u32;
    assert!(
        (plafond - 20..=plafond).contains(&ombre.largeur),
        "{}",
        ombre.largeur
    );
    if cfg!(windows) {
        assert_eq!((densite, ombre.densite), (1.5, 1), "pixel pour pixel");
    } else {
        assert_eq!(ombre.densite, 2);
        assert_eq!(ombre.largeur % 2, 0, "{}", ombre.largeur);
        assert_eq!(ombre.hauteur % 2, 0, "{}", ombre.hauteur);
    }
}

/// **Un écran à 150 %** : la main se compte en pixels de l'ombre, deux par point, et une
/// largeur brute impaire devient paire. Sous Windows, un et demi par point, et rien n'a à être
/// pair.
#[test]
fn ombre_1_la_main_se_compte_en_pixels_de_l_ombre() {
    let echelle = 1.0 / 1.5 * crate::plateforme::densite_de_l_ombre(1.5);
    let marge = debord(echelle);
    let mut impaires = 0;
    for largeur in [100.0, 101.0, 102.0] {
        let (app, board) = un_texte_choisi(largeur, 1.0);
        let ombre = app
            .ombre_des_noeuds(&board, (30.0, 15.0), 1.5)
            .expect("une ombre");
        assert_eq!(ombre.point.0, (30.0 * echelle + marge).round() as i32);
        if cfg!(windows) {
            continue;
        }
        assert_eq!(ombre.largeur % 2, 0, "{}", ombre.largeur);
        impaires += ((largeur * echelle + 2.0 * marge).ceil() as u32) % 2;
    }
    assert!(
        impaires > 0 || cfg!(windows),
        "sans largeur brute impaire, l'épreuve serait aveugle"
    );
}

/// **Translucide, sur un fond transparent** : rien n'est plus opaque que l'ombre, le fond du
/// texte l'est autant qu'elle, et autour, le canevas n'est pas peint.
#[test]
fn ombre_1_translucide_sur_un_fond_transparent() {
    let (app, board) = un_texte_choisi(200.0, 1.0);
    let ombre = app
        .ombre_des_noeuds(&board, (0.0, 0.0), 1.0)
        .expect("une ombre");
    let alphas: Vec<u8> = ombre.rgba.iter().skip(3).step_by(4).copied().collect();
    let plus_opaque = alphas.iter().copied().max();
    assert_eq!(plus_opaque, Some((255.0_f32 * 0.8).round() as u8));
    assert!(alphas.contains(&0), "le canevas n'est pas peint autour");
}
