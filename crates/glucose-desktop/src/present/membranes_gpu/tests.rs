//! **Ce que la voie graphique doit garantir sur les membranes** : les pixels du processeur, à
//! l'arrondi de l'écriture près.
//!
//! Les deux voies évaluent la même fonction, en `f32`, dans le même ordre. Ce qui peut les
//! séparer : l'arrondi d'une égalité — la carte écrit au plus proche, le processeur aussi
//! mais en virgule fixe —, et quelques ulps sur `atan2` et `sqrt`, que la carte calcule avec
//! ses propres circuits. Une membrane plafonne à `A ≈ 0,94` sur un bord sélectionné.

use super::*;
use crate::present::banc_gpu;
use glucose_core::membrane_forme::{self, Arrondi, Bord};
use tiny_skia::Pixmap;

/// Une membrane de Glucose à l'échelle `e`, posée en `(x, y)` : les couches et les opacités du
/// rendu, sans passer par le document.
fn membrane(x: f32, y: f32, (l, h): (f32, f32), e: f32, selection: bool) -> Membrane {
    let r = 60.0 * e;
    let fond = Arrondi::nouveau(x, y, l, h, r);
    let halo = |pad: f32| {
        Arrondi::nouveau(
            x - pad,
            y - pad,
            l + 2.0 * pad,
            h + 2.0 * pad,
            r + pad / 2.0,
        )
    };
    let tiret = 10.0 * e;
    Membrane {
        remplissages: [
            (halo(20.0 * e), 8.0 / 255.0),
            (halo(10.0 * e), 14.0 / 255.0),
            (fond, 8.0 / 255.0),
        ],
        bord: Bord {
            forme: fond,
            demi_largeur: if selection { 1.0 } else { e },
            pointille: (!selection && tiret >= 1.0).then(|| fond.pointille(tiret)),
            alpha: if selection {
                235.0 / 255.0
            } else {
                115.0 / 255.0
            },
        },
        teinte: [96.0 / 255.0, 165.0 / 255.0, 250.0 / 255.0],
    }
}

/// Ce que le **processeur** rend : un fond noir opaque, puis les membranes, dans l'ordre.
fn par_le_processeur(taille: (u32, u32), membranes: &[Membrane]) -> Pixmap {
    let mut p = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    p.fill(tiny_skia::Color::BLACK);
    for m in membranes {
        let (pixels, _) = p.data_mut().as_chunks_mut::<4>();
        membrane_forme::peindre(m, pixels, taille.0, taille.1);
    }
    p
}

/// Ce que la **carte** rend, sur le même fond noir opaque.
fn par_la_carte(taille: (u32, u32), membranes: &[Membrane]) -> Option<Pixmap> {
    let (peripherique, file) = banc_gpu::carte()?;
    let mut passe_membranes = Membranes::nouvelles(&peripherique, banc_gpu::FORMAT);
    passe_membranes.preparer(
        &peripherique,
        &file,
        (taille.0 as f32, taille.1 as f32),
        membranes,
    );
    let cible = banc_gpu::cible(&peripherique, taille);
    let vue = cible.create_view(&Default::default());
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    banc_gpu::passe(&mut encodeur, &vue, wgpu::Color::BLACK, |p| {
        passe_membranes.poser(p);
    });
    file.submit(Some(encodeur.finish()));
    banc_gpu::relire(&peripherique, &file, &cible, taille)
}

/// La borne de l'écart entre les deux voies, en niveaux sur 255 — **mesurée, pas choisie**.
///
/// Un niveau : l'arrondi d'une égalité, que la carte et la virgule fixe peuvent trancher
/// différemment. La tenir serrée est tout l'intérêt : à quatre, une erreur de phase sur un
/// tiret ou de rayon sur un coin passerait encore.
const ECART_ADMIS: u8 = 1;

/// Compare les deux voies sur ces membranes, et rend l'image de la carte pour qui veut la
/// regarder de plus près.
fn comparer(nom: &str, taille: (u32, u32), membranes: &[Membrane]) -> Option<Pixmap> {
    let Some(carte) = par_la_carte(taille, membranes) else {
        eprintln!("aucune carte utilisable : test saute");
        return None;
    };
    let processeur = par_le_processeur(taille, membranes);
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "{nom} : les deux voies divergent de {pire} niveaux (admis {ECART_ADMIS})"
    );
    Some(carte)
}

/// **Une membrane entière, en pointillé, rend les mêmes pixels des deux côtés.**
#[test]
fn test_une_membrane_rend_la_meme_chose_des_deux_cotes() {
    comparer(
        "entiere",
        (420, 300),
        &[membrane(60.5, 50.25, (300.0, 190.0), 1.0, false)],
    );
}

/// **Sélectionnée, vue de très près par son coin, au trait plus fin qu'un pixel, de très loin,
/// sans rayon** : les recoins de la loi — le trait plein, l'arc immense, la couverture
/// plafonnée, le rayon sous le trait.
#[test]
fn test_les_recoins_de_la_loi_rendent_la_meme_chose_des_deux_cotes() {
    let cas = [
        (
            "selectionnee",
            membrane(40.0, 30.0, (320.0, 200.0), 0.8, true),
        ),
        (
            "par son coin, de pres",
            membrane(120.0, 90.0, (60_000.0, 40_000.0), 40.0, false),
        ),
        (
            "trait fin",
            membrane(20.0, 20.0, (340.0, 220.0), 0.2, false),
        ),
        // Vue de très loin, le rayon passe sous le trait : c'est là seulement que la
        // distance au plus profond de la forme compte.
        (
            "de tres loin, selectionnee",
            membrane(30.0, 25.0, (300.0, 200.0), 0.02, true),
        ),
        ("sans rayon", {
            let mut m = membrane(50.0, 40.0, (260.0, 170.0), 1.0, false);
            m.bord.forme.rayon = 0.0;
            m.remplissages.iter_mut().for_each(|(f, _)| f.rayon = 0.0);
            m
        }),
    ];
    for (nom, m) in cas {
        comparer(nom, (400, 280), &[m]);
    }
}

/// **Deux membranes imbriquées se composent dans le même ordre des deux côtés.**
#[test]
fn test_deux_membranes_imbriquees_se_composent_dans_le_meme_ordre() {
    let exterieure = membrane(30.0, 30.0, (360.0, 240.0), 0.6, false);
    let mut interieure = membrane(90.0, 80.0, (180.0, 120.0), 0.6, true);
    interieure.teinte = [250.0 / 255.0, 120.0 / 255.0, 80.0 / 255.0];
    comparer("imbriquees", (420, 300), &[exterieure, interieure]);
}

/// **La membrane est bien là, et pas seulement « proche du noir ».** Sans cette épreuve, un
/// nuanceur qui ne dessinerait rien passerait les autres dès que le processeur se tromperait
/// dans le même sens.
#[test]
fn test_la_membrane_ecrit_vraiment_des_pixels() {
    let m = membrane(60.0, 50.0, (300.0, 190.0), 1.0, false);
    let Some(carte) = comparer("encre", (420, 300), &[m]) else {
        return;
    };
    let teintes = carte
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[2] > 0)
        .count();
    assert!(
        teintes > 50_000,
        "une membrane de 300 x 190 et ses halos touchent bien plus que {teintes} pixels"
    );
}

/// Le tampon grandit quand l'écran demande plus de membranes que la capacité de départ, et il
/// les pose toutes.
#[test]
fn test_le_tampon_suit_ce_que_l_ecran_demande() {
    let combien = MEMBRANES_AU_DEPART + 5;
    let membranes: Vec<Membrane> = (0..combien)
        .map(|i| {
            let (x, y) = ((i % 8) as f32 * 40.0, (i / 8) as f32 * 40.0);
            membrane(x + 4.0, y + 4.0, (30.0, 30.0), 0.1, true)
        })
        .collect();
    comparer("nombreuses", (330, 210), &membranes);
}
