//! Ce que poser un tampon doit garantir — au bit près, contre le rasteriseur qu'il remplace.

use super::*;
use tiny_skia::{Color, PremultipliedColorU8, Transform};

fn image(largeur: u32, hauteur: u32, couleur: Color) -> Pixmap {
    let mut p = Pixmap::new(largeur, hauteur).expect("un pixmap");
    p.fill(couleur);
    p
}

/// La référence : ce que `tiny-skia` aurait écrit.
fn par_le_rasteriseur(
    fond: &Pixmap,
    tampon: &Pixmap,
    (x, y): (i32, i32),
    mode: tiny_skia::BlendMode,
) -> Pixmap {
    let mut attendu = fond.clone();
    attendu.as_mut().draw_pixmap(
        x,
        y,
        tampon.as_ref(),
        &tiny_skia::PixmapPaint {
            blend_mode: mode,
            ..Default::default()
        },
        Transform::identity(),
        None,
    );
    attendu
}

/// **Le même résultat que le rasteriseur, au bit près.** C'est la garantie qui autorise le
/// remplacement : l'adaptation change *comment* on arrive au résultat, jamais le résultat.
#[test]
fn poser_un_tampon_opaque_donne_les_memes_octets_que_le_rasteriseur() {
    let fond = image(64, 48, Color::from_rgba8(20, 30, 40, 255));
    let tampon = image(20, 16, Color::from_rgba8(200, 100, 50, 255));

    let mut obtenu = fond.clone();
    assert!(poser(
        &mut obtenu.as_mut(),
        &tampon,
        (7.0, 5.0),
        Melange::Composer
    ));

    let attendu = par_le_rasteriseur(&fond, &tampon, (7, 5), tiny_skia::BlendMode::SourceOver);
    assert_eq!(obtenu.data(), attendu.data());
}

/// Un tampon **translucide** compose par-dessus, et le mélange doit valoir celui de référence.
#[test]
fn poser_un_tampon_translucide_compose_comme_le_rasteriseur() {
    let fond = image(40, 40, Color::from_rgba8(10, 200, 30, 255));
    let tampon = image(24, 24, Color::from_rgba8(255, 0, 0, 128));

    let mut obtenu = fond.clone();
    assert!(poser(
        &mut obtenu.as_mut(),
        &tampon,
        (8.0, 8.0),
        Melange::Composer
    ));

    let attendu = par_le_rasteriseur(&fond, &tampon, (8, 8), tiny_skia::BlendMode::SourceOver);
    assert_eq!(obtenu.data(), attendu.data());
}

/// **Remplacer n'est pas composer.** Le rendu par région repose des pixels périmés : les
/// composer redoublerait tout ce qui n'est pas opaque, et l'erreur serait invisible sur un
/// fond sombre.
#[test]
fn remplacer_ecrase_le_fond_au_lieu_de_s_y_meler() {
    let fond = image(32, 32, Color::from_rgba8(0, 0, 255, 255));
    let tampon = image(10, 10, Color::from_rgba8(255, 0, 0, 100));

    let mut obtenu = fond.clone();
    assert!(poser(
        &mut obtenu.as_mut(),
        &tampon,
        (4.0, 4.0),
        Melange::Remplacer
    ));

    let pixel = obtenu.pixel(6, 6).expect("un pixel dans la zone remplacee");
    let source = tampon.pixel(2, 2).expect("le pixel source");
    assert_eq!(
        pixel, source,
        "remplacer rend la source telle quelle, sans le bleu du fond"
    );
    // Et hors de la zone, le fond est intact.
    assert_eq!(
        obtenu.pixel(20, 20),
        PremultipliedColorU8::from_rgba(0, 0, 255, 255)
    );
}

/// Un tampon qui dépasse du bord se pose **en partie**, sans déborder ni paniquer.
#[test]
fn un_tampon_qui_deborde_se_pose_sans_sortir_de_l_image() {
    let fond = image(24, 24, Color::from_rgba8(0, 0, 0, 255));
    let tampon = image(16, 16, Color::from_rgba8(255, 255, 255, 255));

    let mut obtenu = fond.clone();
    assert!(poser(
        &mut obtenu.as_mut(),
        &tampon,
        (16.0, 16.0),
        Melange::Composer
    ));

    let attendu = par_le_rasteriseur(&fond, &tampon, (16, 16), tiny_skia::BlendMode::SourceOver);
    assert_eq!(obtenu.data(), attendu.data());
}
