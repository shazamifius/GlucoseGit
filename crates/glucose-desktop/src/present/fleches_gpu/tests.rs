//! **Ce que la voie graphique doit garantir sur les flèches** : les pixels du processeur, à
//! l'arrondi de l'écriture près — et chaque pixel écrit une fois, coudes compris.

use super::*;
use crate::present::banc_gpu;
use glucose_core::arrow::champ::{Champ, Disque};
use tiny_skia::Pixmap;

const TAILLE: (u32, u32) = (400, 260);

fn disque(centre: [f32; 2], rayon: f32, fond: [f32; 3], contour: [f32; 3]) -> Option<Disque> {
    Some(Disque {
        centre,
        rayon,
        demi_contour: 1.0,
        fond,
        contour,
    })
}

/// Des flèches qui éprouvent tout : un coude aigu, une courbe aplatie en nombreux segments, une
/// flèche sélectionnée aux gros disques, deux flèches qui se croisent, une plus fine qu'un
/// pixel, et des disques qui se chevauchent.
fn fleches() -> Vec<Champ> {
    let base = |segments: Vec<[f32; 4]>| Champ {
        axe: [
            segments[0][0],
            segments[0][1],
            segments.last().map_or(0.0, |s| s[2]),
            segments.last().map_or(0.0, |s| s[3]),
        ],
        segments,
        teintes: [[0.9, 0.35, 0.25], [0.95, 0.8, 0.4], [0.3, 0.55, 0.95]],
        halo: [3.0, 0.18],
        ame: [1.0, 0.92],
        ame_blanche: false,
        disques: [None, None],
    };
    let mut coude = base(vec![
        [20.3, 200.0, 150.0, 30.5],
        [150.0, 30.5, 190.7, 210.0],
    ]);
    coude.disques[1] = disque([190.7, 210.0], 3.0, [0.067; 3], [0.3, 0.55, 0.95]);
    let arc: Vec<[f32; 4]> = (0..24)
        .map(|k| {
            let a = |k: f32| {
                let t = k / 24.0 * std::f32::consts::PI;
                (230.0 + 70.0 * t.cos(), 140.0 - 90.0 * t.sin())
            };
            let (p, q) = (a(k as f32), a(k as f32 + 1.0));
            [p.0, p.1, q.0, q.1]
        })
        .collect();
    let mut courbe = base(arc);
    courbe.disques[0] = disque([300.0, 140.0], 3.0, [0.067; 3], [0.9, 0.35, 0.25]);
    courbe.disques[1] = disque([160.0, 140.0], 3.0, [0.067; 3], [0.3, 0.55, 0.95]);
    let mut choisie = base(vec![[30.0, 60.0, 370.0, 90.0]]);
    choisie.ame_blanche = true;
    choisie.halo = [6.0, 0.45];
    choisie.disques = [
        disque([30.0, 60.0], 12.0, [0.9, 0.35, 0.25], [1.0; 3]),
        disque([370.0, 90.0], 12.0, [0.3, 0.55, 0.95], [1.0; 3]),
    ];
    let croisee = base(vec![[40.0, 240.0, 380.0, 20.0]]);
    let mut fine = base(vec![[10.5, 120.25, 390.5, 130.75]]);
    fine.halo = [0.4, 0.18];
    fine.ame = [0.3, 0.92];
    let mut serres = base(vec![[100.0, 230.0, 104.0, 232.0]]);
    serres.disques = [
        disque([100.0, 230.0], 8.0, [0.067; 3], [0.9, 0.35, 0.25]),
        disque([104.0, 232.0], 8.0, [0.067; 3], [0.3, 0.55, 0.95]),
    ];
    vec![coude, courbe, choisie, croisee, fine, serres]
}

/// Ce que le **processeur** rend : un fond noir opaque, puis les flèches, dans l'ordre.
fn par_le_processeur(champs: &[Champ]) -> Pixmap {
    let mut p = Pixmap::new(TAILLE.0, TAILLE.1).expect("un pixmap");
    p.fill(tiny_skia::Color::BLACK);
    for c in champs {
        let (pixels, _) = p.data_mut().as_chunks_mut::<4>();
        c.peindre(pixels, TAILLE.0, TAILLE.1);
    }
    p
}

/// Ce que la **carte** rend, sur le même fond.
fn par_la_carte(champs: &[Champ]) -> Option<Pixmap> {
    let (peripherique, file) = banc_gpu::carte()?;
    let mut passe = FlechesGpu::nouvelles(&peripherique, banc_gpu::FORMAT);
    passe.preparer(
        &peripherique,
        &file,
        (TAILLE.0 as f32, TAILLE.1 as f32),
        champs,
    );
    let cible = banc_gpu::cible(&peripherique, TAILLE);
    let vue = cible.create_view(&Default::default());
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    banc_gpu::passe(&mut encodeur, &vue, wgpu::Color::BLACK, |p| passe.poser(p));
    file.submit(Some(encodeur.finish()));
    banc_gpu::relire(&peripherique, &file, &cible, TAILLE)
}

/// **La carte peint les flèches du processeur**, pixel pour pixel. Un coude composé deux fois
/// y doublerait le halo — de 18 % à 33 % — et un disque oublié y manquerait entier : ni l'un
/// ni l'autre ne tiendrait la borne.
///
/// # La borne : un niveau par flèche superposée — mesurée, et déduite
///
/// Les deux voies évaluent la même loi ; chacune compose ensuite la flèche sur l'image **en
/// huit bits**, et arrondit — le processeur en virgule fixe, la carte à son mélangeur. Chaque
/// composition peut donc les écarter d'un demi-niveau de chaque côté : d'un niveau par flèche
/// qui passe sur le pixel. Mesuré sur ce banc : 825 pixels à un niveau sous une flèche, 432
/// sous deux, et deux pixels à deux niveaux, là où trois se croisent.
#[test]
fn test_fleche_2_la_carte_peint_les_fleches_du_processeur() {
    let champs = fleches();
    let Some(carte) = par_la_carte(&champs) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let processeur = par_le_processeur(&champs);
    for y in 0..TAILLE.1 {
        for x in 0..TAILLE.0 {
            let (a, b) = (
                processeur.pixel(x, y).expect("dedans"),
                carte.pixel(x, y).expect("dedans"),
            );
            let ecart = [
                a.red().abs_diff(b.red()),
                a.green().abs_diff(b.green()),
                a.blue().abs_diff(b.blue()),
            ]
            .into_iter()
            .max()
            .unwrap_or(0);
            let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
            let couches = champs.iter().filter(|c| c.couleur(cx, cy)[3] > 0.0).count();
            assert!(
                usize::from(ecart) <= couches,
                "({x}, {y}) : {ecart} niveaux sous {couches} flèche(s)"
            );
        }
    }
}

/// **Aucune flèche, aucun pixel** : la passe ne pose rien.
#[test]
fn test_fleche_2_sans_fleche_rien_ne_se_pose() {
    let Some(carte) = par_la_carte(&[]) else {
        return;
    };
    assert!(carte
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .all(|p| *p == [0, 0, 0, 255]));
}

/// **Chaque flèche occupe exactement la place que le nuanceur lit** : treize `vec4`. Un champ
/// oublié décalerait toutes les flèches qui suivent.
#[test]
fn test_fleche_2_une_fleche_occupe_la_place_du_nuanceur() {
    let champs = fleches();
    let ecrit = Ecrit::de(&champs);
    assert_eq!(ecrit.fleches.len(), champs.len() * OCTETS_FLECHE);
    let segments: usize = champs.iter().map(|c| c.segments.len()).sum();
    assert_eq!(ecrit.segments.len(), segments * OCTETS_SEGMENT);
}
