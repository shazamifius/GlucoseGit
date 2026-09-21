//! Ce que les cartes texturées garantissent : rendues hors contexte, elles donnent les mêmes
//! pixels qu'en place.

use super::*;

/// Le palier dyadique le plus proche l'est au sens du logarithme : entre deux puissances de
/// deux, la frontière est leur moyenne géométrique.
#[test]
fn test_le_palier_dyadique_est_le_plus_proche_au_sens_du_logarithme() {
    assert_eq!(palier_dyadique(1.0), 1.0);
    assert_eq!(palier_dyadique(2.0), 2.0);
    assert_eq!(palier_dyadique(0.5), 0.5);
    // racine de deux est la frontiere : juste en dessous, un ; juste au-dessus, deux.
    assert_eq!(palier_dyadique(1.40), 1.0);
    assert_eq!(palier_dyadique(1.42), 2.0);
    assert_eq!(palier_dyadique(0.72), 1.0);
    assert_eq!(palier_dyadique(0.70), 0.5);
    assert_eq!(
        palier_dyadique(0.0),
        1.0,
        "une echelle nulle ne fait pas paniquer"
    );
    assert_eq!(palier_dyadique(f64::NAN), 1.0);
}

use crate::renderer::Renderer;
use glucose_core::types::BoardImage;
use tiny_skia::Pixmap;

/// Le composant d'une photo en chemin, rendu à part et reposé sur un pixel entier, donne les
/// pixels que le processeur écrit en place — au bit près, puisque c'est le même code à la
/// même phase.
fn ecart_photo_en_chemin(x: f64, y: f64, rotation: f64) -> (u8, usize) {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let mut img = BoardImage::new("p".to_string(), x, y, 180.0, 120.0);
    img.rotation = rotation;
    let vp = Viewport {
        scale: 1.0,
        x: 20.5,
        y: 10.25,
    };
    let taille = (400u32, 300u32);
    let regime = Regime::de(vp, Regard::immobile(), taille, 0.0);

    // En place, comme la voie processeur.
    let mut en_place = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let (sx, sy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &vp);
    draw_missing_image(
        kit.typography,
        kit.theme,
        &mut en_place.as_mut(),
        (sx as f32, sy as f32),
        (
            (img.width * vp.scale) as f32,
            (img.height * vp.scale) as f32,
        ),
        &img.id,
        img.rotation,
    );

    // A part, puis repose la ou la pose le dit.
    let composant = regime.photo_en_chemin(&img).expect("un composant");
    let texture = composant.rendre(kit).expect("une texture");
    let mut reposee = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let (px, py) = (composant.pose.x, composant.pose.y);
    assert_eq!(px, px.floor(), "a l'arret, la pose est entiere");
    assert_eq!(py, py.floor(), "a l'arret, la pose est entiere");
    crate::composition::poser(
        &mut reposee.as_mut(),
        &texture,
        (px, py),
        glucose_core::report::Melange::Composer,
    );

    let pire = en_place
        .data()
        .iter()
        .zip(reposee.data())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    let canaux = en_place
        .data()
        .iter()
        .zip(reposee.data())
        .filter(|(a, b)| a.abs_diff(**b) > 1)
        .count();
    (pire, canaux)
}

#[test]
fn test_une_photo_en_chemin_droite_se_repose_au_bit_pres() {
    let (pire, canaux) = ecart_photo_en_chemin(150.3, 100.7, 0.0);
    assert!(
        pire <= 1 && canaux == 0,
        "photo droite : pire {pire}, {canaux} canaux au-dela de 1"
    );
}

#[test]
fn test_une_photo_en_chemin_penchee_se_repose_au_bit_pres() {
    let (pire, canaux) = ecart_photo_en_chemin(150.3, 100.7, std::f64::consts::FRAC_PI_8);
    assert!(
        pire <= 1 && canaux == 0,
        "photo penchee : pire {pire}, {canaux} canaux au-dela de 1"
    );
}

/// Un composant de carte, rendu à part et reposé, comparé à la carte dessinée en place.
fn ecart_carte(selectionnee: bool) -> (u8, usize) {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let vp = Viewport {
        scale: 1.0,
        x: 20.5,
        y: 10.25,
    };
    let taille = (500u32, 300u32);
    let regime = Regime::de(vp, Regard::immobile(), taille, 0.0);
    let (x, y, w, h) = (60.3, 40.7, 240.0f32, 60.0f32);
    let corps = "Accents : éàçùôêîï — « guillemets »
Et un lien.";
    let teinte = (96, 165, 250);

    let mut en_place = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        tints: kit.tints,
        theme: kit.theme,
        vp,
        scale: WorldScale::new(vp.scale),
        clip: Clip {
            width: taille.0 as f32,
            height: taille.1 as f32,
            top: 0.0,
        },
    };
    draw_card_contenu(
        &ctx,
        &mut en_place.as_mut(),
        TextCard {
            origin: (x, y),
            size: (w, h),
            body: corps,
            tint: teinte,
            selected: selectionnee,
            editing: None,
        },
    );
    let composant = regime
        .carte(kit, "c", (x, y, w, h), (corps, teinte, selectionnee))
        .expect("un composant");
    let texture = composant.rendre(kit).expect("une texture");
    let mut reposee = Pixmap::new(taille.0, taille.1).expect("pixmap");
    crate::composition::poser(
        &mut reposee.as_mut(),
        &texture,
        (composant.pose.x, composant.pose.y),
        glucose_core::report::Melange::Composer,
    );
    let pire = en_place
        .data()
        .iter()
        .zip(reposee.data())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    let canaux = en_place
        .data()
        .iter()
        .zip(reposee.data())
        .filter(|(a, b)| a.abs_diff(**b) > 1)
        .count();
    (pire, canaux)
}

/// **Une carte au repos se repose au bit près** : même code, même mise en page, même phase.
#[test]
fn test_une_carte_se_repose_au_bit_pres() {
    let (pire, canaux) = ecart_carte(false);
    assert!(
        pire <= 1 && canaux == 0,
        "carte au repos : pire {pire}, {canaux} canaux au-dela de 1"
    );
}

/// **Une carte sélectionnée se repose à un cran de couverture près**, et ce cran n'est pas
/// à nous.
///
/// Le trait de sélection est un anneau anti-crénelé de deux pixels, et `tiny-skia` accumule
/// ses bords en virgule fixe le long de chaque ligne : la même forme, translatée d'un nombre
/// **entier** de pixels, ne donne pas toujours la même couverture aux points où la tangente
/// d'un coin arrondi frôle une frontière de sous-pixel. L'écart vaut un cran de son
/// suréchantillonnage — au plus une vingtaine de niveaux, sur quelques dizaines de pixels — et
/// il existe déjà sur la voie processeur seule, entre deux images d'un glissement.
///
/// Ce n'est donc pas une différence de loi entre les voies : c'est la sensibilité du
/// rastériseur à la position absolue, mesurée et bornée ici pour qu'elle ne grandisse pas.
#[test]
fn test_une_carte_selectionnee_se_repose_a_un_cran_de_couverture_pres() {
    let (pire, canaux) = ecart_carte(true);
    assert!(
        pire <= 26,
        "carte selectionnee : pire {pire} -- plus qu'un cran de couverture"
    );
    assert!(
        canaux <= 200,
        "carte selectionnee : {canaux} canaux au-dela de 1 -- ce n'est plus un coin, c'est un          bord entier"
    );
}
