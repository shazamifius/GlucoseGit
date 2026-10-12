//! PRESET-1 — une zone se dessine comme dans Glucose Tauri : son fond, son bandeau, son bord
//! en pointillés — et un trait continu de loin.

use crate::renderer::Renderer;
use crate::ui::UiState;
use glucose_core::store::Store;
use glucose_core::types::{BoardZone, Viewport};
use tiny_skia::Pixmap;

/// Une zone « Détails » de CharaDesign (rouge), de 340 × 700, posée en (100, 100), vue à
/// cette échelle depuis l'origine — sans grille, pour ne lire que la zone.
fn rendre(echelle: f64) -> Pixmap {
    let mut store = Store::new("zones");
    let board = store.project.active_board_id.clone();
    store.poser_les_zones(
        &board,
        vec![BoardZone::new(
            "charadesign/detail",
            100.0,
            100.0,
            340.0,
            700.0,
        )],
    );
    let mut pixmap = Pixmap::new(1000, 1000).expect("un pixmap");
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: echelle,
    };
    renderer.rendre_pour_l_export(&mut pixmap.as_mut(), &store, &mut ui, vue);
    pixmap
}

/// Le pixel en `(x, y)`, en rouge, vert, bleu.
fn px(p: &Pixmap, x: u32, y: u32) -> [u8; 3] {
    let c = p.pixel(x, y).expect("dans l'image").demultiply();
    [c.red(), c.green(), c.blue()]
}

/// **Le fond, le bandeau, le filet** : trois teintes distinctes, la zone visible sur le
/// canevas, le bandeau rougi par sa catégorie et le filet plus encore.
#[test]
fn test_preset_1_une_zone_a_son_fond_son_bandeau_et_son_filet() {
    let p = rendre(1.0);
    let canevas = px(&p, 50, 50);
    let fond = px(&p, 300, 600);
    let bandeau = px(&p, 300, 125);
    let filet = px(&p, 300, 100);
    assert_ne!(fond, canevas, "la zone se voit");
    assert!(
        bandeau[0] > fond[0] && bandeau[0] > bandeau[1],
        "le bandeau rougit : {bandeau:?}"
    );
    assert!(
        filet[0] > bandeau[0],
        "le filet plus encore : {filet:?} {bandeau:?}"
    );
}

/// **Le bord en pointillés** : huit unités de trait, six de vide, depuis le coin — sur le bord
/// du bas, un tiret en 4, un vide en 10, un tiret en 18.
#[test]
fn test_preset_1_le_bord_est_en_pointilles() {
    let p = rendre(1.0);
    let fond = px(&p, 300, 600);
    let bas = 100 + 700 - 1;
    let (tiret, vide, encore) = (px(&p, 104, bas), px(&p, 110, bas), px(&p, 118, bas));
    assert_ne!(tiret, fond, "un tiret");
    assert_eq!(vide, fond, "un vide");
    assert_eq!(encore, tiret, "le tiret suivant");
    assert!(
        tiret[0] > tiret[1],
        "de la couleur de la catégorie : {tiret:?}"
    );
}

/// **De loin, un trait continu de l'opacité moyenne** : quand une période de pointillé tient
/// dans un pixel, le bord ne montre plus de vides, et il est plus pâle qu'un tiret — huit
/// quatorzièmes de son opacité.
#[test]
fn test_preset_1_de_loin_le_bord_est_un_trait_moyen() {
    let pres = rendre(1.0);
    let tiret = px(&pres, 104, 799);
    let loin = rendre(0.05);
    // La zone va de 5 à 22 en x, de 5 à 40 en y : son bord du bas est la rangée 39.
    let bord: Vec<[u8; 3]> = (7..20).map(|x| px(&loin, x, 39)).collect();
    assert!(
        bord.windows(2).all(|w| w[0] == w[1]),
        "aucun vide : {bord:?}"
    );
    let fond = px(&loin, 12, 30);
    assert!(
        bord[0][0] > fond[0],
        "le bord se voit : {:?} {fond:?}",
        bord[0]
    );
    assert!(
        bord[0][0] < tiret[0],
        "plus pâle qu'un tiret : {:?} {tiret:?}",
        bord[0]
    );
}
