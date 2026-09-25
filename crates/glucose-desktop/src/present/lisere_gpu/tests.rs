//! **Le liseré de la carte est celui du processeur** : la même loi, à l'arrondi près.

use super::*;
use crate::dock::DockPass;
use crate::params::{Pointer, ScreenFrame};
use crate::present::banc_gpu;
use crate::theme::Theme;
use crate::typography::Typography;
use tiny_skia::Pixmap;

const TAILLE: (u32, u32) = (640, 400);

/// Ce que le **processeur** peint : le liseré d'avant, sur un fond noir opaque.
fn par_le_processeur(s: f32) -> Pixmap {
    let mut p = Pixmap::new(TAILLE.0, TAILLE.1).expect("un pixmap");
    p.fill(tiny_skia::Color::BLACK);
    let (typo, theme) = (Typography::new(), Theme::dark());
    let pass = DockPass {
        typo: &typo,
        theme: &theme,
        screen: ScreenFrame {
            width: TAILLE.0 as f32,
            height: TAILLE.1 as f32,
            header_h: 0.0,
            scale: s,
        },
        pointer: Pointer { x: 0.0, y: 0.0 },
        cache: None,
    };
    crate::dock::lisere_du_passe(&mut p.as_mut(), &pass, s);
    p
}

/// Ce que la **carte** peint, sur le même fond.
fn par_la_carte(s: f32) -> Option<Pixmap> {
    let (peripherique, file) = banc_gpu::carte()?;
    let mut passe = LisereGpu::nouveau(&peripherique, banc_gpu::FORMAT);
    let lisere = crate::dock::lisere(&Theme::dark(), s);
    passe.preparer(&file, (TAILLE.0 as f32, TAILLE.1 as f32), Some(lisere));
    let cible = banc_gpu::cible(&peripherique, TAILLE);
    let vue = cible.create_view(&Default::default());
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    banc_gpu::passe(&mut encodeur, &vue, wgpu::Color::BLACK, |p| passe.poser(p));
    file.submit(Some(encodeur.finish()));
    banc_gpu::relire(&peripherique, &file, &cible, TAILLE)
}

/// L'écart admis, en niveaux sur 255 — **mesuré, et localisé**.
///
/// Deux niveaux partout ; trois dans les coins, où deux dégradés se superposent : le processeur
/// arrondit le premier à huit bits en l'écrivant, puis compose le second sur ce résultat
/// arrondi, là où la carte compose les deux en flottant. Mesuré à 640 × 400 : 144 pixels à
/// trois, tous dans les coins, 6 616 à deux, 73 232 à un.
const ECART_ADMIS: u8 = 2;
const ECART_DANS_LES_COINS: u8 = 3;

/// **Les deux voies peignent le même liseré**, aux deux échelles d'interface courantes, et il
/// est bien là : un trait ambré au bord, un voile qui s'éteint vers l'intérieur.
#[test]
fn test_le_lisere_de_la_carte_est_celui_du_processeur() {
    for s in [1.0_f32, 1.5] {
        let Some(carte) = par_la_carte(s) else {
            eprintln!("aucune carte utilisable : test saute");
            return;
        };
        let processeur = par_le_processeur(s);
        let voile = 60.0 * s;
        for y in 0..TAILLE.1 {
            for x in 0..TAILLE.0 {
                let (a, b) = (processeur.pixel(x, y), carte.pixel(x, y));
                let (Some(a), Some(b)) = (a, b) else {
                    continue;
                };
                let ecart = [
                    a.red().abs_diff(b.red()),
                    a.green().abs_diff(b.green()),
                    a.blue().abs_diff(b.blue()),
                ]
                .into_iter()
                .max()
                .unwrap_or(0);
                let (fx, fy) = (x as f32, y as f32);
                let pres = |v: f32, t: u32| v < voile || v > t as f32 - voile;
                let admis = if pres(fx, TAILLE.0) && pres(fy, TAILLE.1) {
                    ECART_DANS_LES_COINS
                } else {
                    ECART_ADMIS
                };
                assert!(ecart <= admis, "échelle {s}, ({x}, {y}) : {ecart} niveaux");
            }
        }
        let pixel = |x: u32, y: u32| carte.pixel(x, y).expect("dedans");
        assert!(pixel(1, 200).red() > 150, "le trait ambré au bord");
        assert!(
            pixel(TAILLE.0 / 2, TAILLE.1 / 2).red() == 0,
            "rien au milieu"
        );
    }
}

/// **Aucun liseré, aucun pixel** : sans aperçu du passé, la passe ne pose rien.
#[test]
fn test_sans_lisere_rien_ne_se_pose() {
    let Some((peripherique, file)) = banc_gpu::carte() else {
        return;
    };
    let mut passe = LisereGpu::nouveau(&peripherique, banc_gpu::FORMAT);
    passe.preparer(&file, (TAILLE.0 as f32, TAILLE.1 as f32), None);
    let cible = banc_gpu::cible(&peripherique, TAILLE);
    let vue = cible.create_view(&Default::default());
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    banc_gpu::passe(&mut encodeur, &vue, wgpu::Color::BLACK, |p| passe.poser(p));
    file.submit(Some(encodeur.finish()));
    let image = banc_gpu::relire(&peripherique, &file, &cible, TAILLE).expect("relue");
    assert!(image
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .all(|p| *p == [0, 0, 0, 255]));
}
