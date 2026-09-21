//! **Ce que la voie graphique doit garantir sur le fond** : la même couleur et les mêmes
//! points que le processeur, à un écart borné et mesuré.
//!
//! # D'où vient l'écart, et pourquoi il n'est pas un défaut
//!
//! Le processeur pré-calcule seize masques de point — quatre phases sous-pixel par axe — et
//! pose celui dont la phase est la plus proche. Il ne peut pas faire autrement : un masque
//! discret ne peut pas suivre une position continue.
//!
//! La carte n'a pas cette contrainte et évalue la couverture **exactement là où le point
//! tombe**. L'écart entre les deux est donc celui de la quantification du processeur : au
//! plus un huitième de pixel de position, soit au plus un huitième de niveau de couverture,
//! que l'opacité d'un point — au plus 0,45 — ramène à quelques niveaux de gris.
//!
//! C'est la carte qui est **plus juste**, et le test le dit dans ce sens-là.

use super::*;
use crate::present::banc_gpu;
use crate::renderer::scene::grid;
use glucose_core::types::Viewport;
use tiny_skia::Pixmap;

/// La couleur du canevas, telle que le thème la donne.
const CANVAS: (u8, u8, u8) = (0x0d, 0x0d, 0x0d);
/// La hauteur du bandeau dans ces épreuves.
const HEADER: f32 = 48.0;

/// Le réglage que le socle produirait pour cette vue.
fn reglage(vp: &Viewport) -> Fond {
    let (pas, rayon, opacite) = grid::grid_params(vp).unwrap_or((0.0, 0.0, 0.0));
    Fond {
        rouge: f32::from(CANVAS.0) / 255.0,
        vert: f32::from(CANVAS.1) / 255.0,
        bleu: f32::from(CANVAS.2) / 255.0,
        echelle: vp.scale as f32,
        vue_x: vp.x as f32,
        vue_y: vp.y as f32,
        pas: pas as f32,
        rayon,
        opacite,
        gris: f32::from(grid::grid_grey()) / 255.0,
        header: HEADER,
    }
}

/// Ce que le **processeur** rend : la couleur du canevas, puis la grille.
fn par_le_processeur(taille: (u32, u32), vp: &Viewport) -> Pixmap {
    let mut p = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    p.fill(tiny_skia::Color::from_rgba8(
        CANVAS.0, CANVAS.1, CANVAS.2, 255,
    ));
    grid::draw_grid(&mut p.as_mut(), vp, taille.0, taille.1, HEADER);
    p
}

/// Ce que la **carte** rend pour la même vue.
fn par_la_carte(taille: (u32, u32), vp: &Viewport) -> Option<Pixmap> {
    let (peripherique, file) = banc_gpu::carte()?;
    let mut fond = FondGpu::nouveau(&peripherique, banc_gpu::FORMAT);
    fond.preparer(&file, (taille.0 as f32, taille.1 as f32), Some(reglage(vp)));
    let cible = banc_gpu::cible(&peripherique, taille);
    let vue = cible.create_view(&Default::default());
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    // Vert vif : si le nuanceur ne couvrait pas tout l'ecran, cela se verrait au premier
    // pixel. Le fond REMPLACE, donc rien de cette couleur ne doit survivre.
    banc_gpu::passe(
        &mut encodeur,
        &vue,
        wgpu::Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        },
        |p| fond.poser(p),
    );
    file.submit(Some(encodeur.finish()));
    banc_gpu::relire(&peripherique, &file, &cible, taille)
}

/// La borne de l'écart entre les deux voies, en niveaux de gris sur 255.
///
/// Elle vient de la quantification en seize phases du processeur : un huitième de pixel de
/// position, donc au plus un huitième de couverture, que l'opacité maximale d'un point —
/// 0,45 — et l'écart entre le gris 136 et le fond 13 ramènent à `0,125 × 0,45 × 123 ≈ 7`.
///
/// **La mesure donne exactement sept**, ce qui est la meilleure confirmation possible du
/// raisonnement : la borne est atteinte, donc elle est juste, et elle n'a aucune marge où
/// une dérive pourrait se cacher.
const ECART_ADMIS: u8 = 7;

/// Au-delà de cet écart, un pixel est plus qu'un décalage de phase — et il doit rester rare.
///
/// Le pire écart seul ne dit pas si un pixel est en cause ou dix mille : c'est la leçon de la
/// chronique, qui laissait une image aberrante décider du portrait d'un geste (fiche 20
/// § 4.5). Les deux grandeurs se lisent donc ensemble.
const ECART_COURANT: u8 = 2;

/// **La carte rend le même fond que le processeur** — couleur et points.
#[test]
fn test_le_fond_rend_la_meme_chose_des_deux_cotes() {
    let taille = (400u32, 300u32);
    // Une translation qui ne tombe pas sur un pixel entier : le cas ou la quantification du
    // processeur se voit le plus.
    for vp in [
        Viewport {
            scale: 1.0,
            x: 17.37,
            y: 91.62,
        },
        Viewport {
            scale: 2.0,
            x: -240.5,
            y: 60.25,
        },
        Viewport {
            scale: 0.5,
            x: 3.0,
            y: 7.0,
        },
    ] {
        let Some(carte) = par_la_carte(taille, &vp) else {
            eprintln!("aucune carte utilisable : test saute");
            return;
        };
        let processeur = par_le_processeur(taille, &vp);
        let pire = banc_gpu::pire_ecart(&processeur, &carte);
        assert!(
            pire <= ECART_ADMIS,
            "a l'echelle {} les deux voies divergent de {pire} niveaux (admis {ECART_ADMIS})",
            vp.scale
        );
        // Et l'ecart ne touche qu'une frange : les points d'une grille couvrent quelques
        // milliemes de l'ecran, et seuls leurs bords sont concernes par la phase.
        let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
        let canaux = processeur.data().len();
        assert!(
            larges * 200 < canaux,
            "a l'echelle {} l'ecart depasse {ECART_COURANT} sur {larges} canaux sur {canaux} : \
             ce n'est plus une question de phase",
            vp.scale
        );
    }
}

/// **Le fond couvre tout l'écran** : rien de ce qui était là avant ne survit.
#[test]
fn test_le_fond_remplace_tout_ce_qui_etait_la() {
    let taille = (128u32, 96u32);
    let vp = Viewport {
        scale: 1.0,
        x: 0.0,
        y: 0.0,
    };
    let Some(carte) = par_la_carte(taille, &vp) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    // Le vert de l'effacement ne doit apparaitre nulle part : un canal vert au-dela du gris
    // d'un point trahirait un pixel que le nuanceur n'a pas ecrit.
    let survivants = carte
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[1] > 160)
        .count();
    assert_eq!(survivants, 0, "{survivants} pixels n'ont pas ete repeints");
}

/// **Aucun point ne se pose au-dessus du bandeau**, des deux côtés.
#[test]
fn test_le_bandeau_reste_uni() {
    let taille = (256u32, 200u32);
    let vp = Viewport {
        scale: 1.0,
        x: 11.5,
        y: 5.5,
    };
    let Some(carte) = par_la_carte(taille, &vp) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let lignes = HEADER as usize;
    let attendu = [CANVAS.0, CANVAS.1, CANVAS.2, 255];
    let fautifs = carte.data().as_chunks::<4>().0[..lignes * taille.0 as usize]
        .iter()
        .filter(|p| **p != attendu)
        .count();
    assert_eq!(fautifs, 0, "{fautifs} pixels de bandeau portent un point");
}

/// **Sous l'échelle d'extinction, le fond est uni** — comme la fiche 06 le demande.
#[test]
fn test_sous_l_extinction_la_grille_disparait() {
    let taille = (128u32, 128u32);
    let vp = Viewport {
        scale: 0.05,
        x: 0.0,
        y: 0.0,
    };
    let Some(carte) = par_la_carte(taille, &vp) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let attendu = [CANVAS.0, CANVAS.1, CANVAS.2, 255];
    assert!(
        carte
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == attendu),
        "la grille se dessine encore sous son echelle d'extinction"
    );
}

/// **La carte pose vraiment des points** — sans quoi les tests d'égalité passeraient sur un
/// fond uni des deux côtés (fiche 17 § 3.1 : un zéro se lit comme une mesure).
#[test]
fn test_la_grille_pose_vraiment_des_points() {
    let taille = (400u32, 300u32);
    let vp = Viewport {
        scale: 1.0,
        x: 17.37,
        y: 91.62,
    };
    let Some(carte) = par_la_carte(taille, &vp) else {
        eprintln!("aucune carte utilisable : test saute");
        return;
    };
    let points = carte
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] > CANVAS.0 + 2)
        .count();
    // Un pas de soixante unites monde sur 400 x 252 utiles donne une trentaine de points, de
    // quelques pixels chacun.
    assert!(
        points > 50,
        "la grille n'a pose que {points} pixels : elle ne se dessine pas"
    );
}
