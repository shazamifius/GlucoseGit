//! Ce que le report doit garantir, et que la mesure ne dirait pas.

use super::*;

/// Une image damier, dont chaque pixel dépend de sa position : deux pixels échangés se voient.
fn damier(largeur: u32, hauteur: u32) -> Vec<Pixel> {
    (0..largeur * hauteur)
        .map(|i| {
            let (x, y) = (i % largeur, i / largeur);
            [(x * 7) as u8, (y * 11) as u8, (x ^ y) as u8, 255]
        })
        .collect()
}

fn unie(largeur: u32, hauteur: u32, p: Pixel) -> Vec<Pixel> {
    vec![p; (largeur * hauteur) as usize]
}

fn toute(largeur: u32, hauteur: u32) -> Boite {
    Boite::nouvelle(0.0, 0.0, largeur as f32, hauteur as f32)
}

#[test]
fn pose_entiere_a_la_taille_native_recopie_exactement() {
    let src = damier(4, 3);
    let vue = Vue::nouvelle(&src, 4, 3).unwrap();
    let mut fond = unie(10, 10, [0, 0, 0, 255]);
    let mut dest = VueMut::nouvelle(&mut fond, 10, 10).unwrap();

    let ecrits = reporter(
        &mut dest,
        &vue,
        Pose {
            x: 2.0,
            y: 1.0,
            largeur: 4.0,
            hauteur: 3.0,
        },
        toute(10, 10),
        Melange::Remplacer,
    );

    assert_eq!(ecrits, 12, "quatre par trois pixels");
    for y in 0..3 {
        for x in 0..4 {
            assert_eq!(
                fond[((y + 1) * 10 + (x + 2)) as usize],
                src[(y * 4 + x) as usize],
                "le pixel ({x}, {y}) n'a pas ete recopie tel quel"
            );
        }
    }
}

#[test]
fn rien_ne_sort_du_clip() {
    let src = unie(8, 8, [255, 0, 0, 255]);
    let vue = Vue::nouvelle(&src, 8, 8).unwrap();
    let mut fond = unie(16, 16, [0, 0, 0, 255]);
    let mut dest = VueMut::nouvelle(&mut fond, 16, 16).unwrap();

    let clip = Boite::nouvelle(4.0, 4.0, 2.0, 2.0);
    reporter(
        &mut dest,
        &vue,
        Pose {
            x: 0.0,
            y: 0.0,
            largeur: 8.0,
            hauteur: 8.0,
        },
        clip,
        Melange::Remplacer,
    );

    for y in 0..16u32 {
        for x in 0..16u32 {
            let dedans = (4..6).contains(&x) && (4..6).contains(&y);
            let pixel = fond[(y * 16 + x) as usize];
            if dedans {
                assert_eq!(pixel, [255, 0, 0, 255], "({x}, {y}) devait etre peint");
            } else {
                assert_eq!(pixel, [0, 0, 0, 255], "({x}, {y}) a ete peint hors du clip");
            }
        }
    }
}

/// **L'invariant central.** Découper la zone visible en morceaux ne change pas un seul pixel.
///
/// C'est ce qui autorise le rendu à ne peindre que les morceaux que l'occlusion laisse voir :
/// si cette égalité tombait, une photo partiellement recouverte se dessinerait autrement
/// qu'une photo entière, et le résultat dépendrait de ce qui se trouve par-dessus.
#[test]
fn peindre_par_morceaux_donne_la_meme_image_qu_en_une_fois() {
    let src = damier(13, 9);
    let vue = Vue::nouvelle(&src, 13, 9).unwrap();
    let pose = Pose {
        x: 3.5,
        y: 2.25,
        largeur: 27.0,
        hauteur: 19.0,
    };

    let mut en_une_fois = unie(40, 30, [9, 9, 9, 255]);
    {
        let mut dest = VueMut::nouvelle(&mut en_une_fois, 40, 30).unwrap();
        reporter(&mut dest, &vue, pose, toute(40, 30), Melange::Remplacer);
    }

    // Quatre morceaux qui pavent la même zone, dont deux bandes très fines.
    let morceaux = [
        Boite::nouvelle(0.0, 0.0, 40.0, 7.0),
        Boite::nouvelle(0.0, 7.0, 5.0, 23.0),
        Boite::nouvelle(5.0, 7.0, 1.0, 23.0),
        Boite::nouvelle(6.0, 7.0, 34.0, 23.0),
    ];
    let mut par_morceaux = unie(40, 30, [9, 9, 9, 255]);
    {
        let mut dest = VueMut::nouvelle(&mut par_morceaux, 40, 30).unwrap();
        for clip in morceaux {
            reporter(&mut dest, &vue, pose, clip, Melange::Remplacer);
        }
    }

    assert_eq!(
        en_une_fois, par_morceaux,
        "le decoupage en morceaux a change l'image"
    );
}

/// Le même invariant, sur le chemin de la recopie directe — il n'emprunte pas le même code.
#[test]
fn peindre_par_morceaux_vaut_aussi_pour_la_recopie_directe() {
    let src = damier(20, 20);
    let vue = Vue::nouvelle(&src, 20, 20).unwrap();
    let pose = Pose {
        x: 5.0,
        y: 5.0,
        largeur: 20.0,
        hauteur: 20.0,
    };

    let mut en_une_fois = unie(32, 32, [1, 2, 3, 255]);
    {
        let mut dest = VueMut::nouvelle(&mut en_une_fois, 32, 32).unwrap();
        reporter(&mut dest, &vue, pose, toute(32, 32), Melange::Remplacer);
    }

    let mut par_colonnes = unie(32, 32, [1, 2, 3, 255]);
    {
        let mut dest = VueMut::nouvelle(&mut par_colonnes, 32, 32).unwrap();
        for x in 0..32 {
            reporter(
                &mut dest,
                &vue,
                pose,
                Boite::nouvelle(x as f32, 0.0, 1.0, 32.0),
                Melange::Remplacer,
            );
        }
    }

    assert_eq!(en_une_fois, par_colonnes);
}

#[test]
fn un_clip_disjoint_n_ecrit_rien() {
    let src = unie(4, 4, [255, 255, 255, 255]);
    let vue = Vue::nouvelle(&src, 4, 4).unwrap();
    let avant = unie(10, 10, [7, 7, 7, 255]);
    let mut fond = avant.clone();
    let mut dest = VueMut::nouvelle(&mut fond, 10, 10).unwrap();

    let ecrits = reporter(
        &mut dest,
        &vue,
        Pose {
            x: 0.0,
            y: 0.0,
            largeur: 4.0,
            hauteur: 4.0,
        },
        Boite::nouvelle(6.0, 6.0, 4.0, 4.0),
        Melange::Remplacer,
    );

    assert_eq!(ecrits, 0, "une photo entierement recouverte ne coute rien");
    assert_eq!(fond, avant, "rien ne devait bouger");
}

#[test]
fn ce_qui_deborde_de_la_destination_est_ignore_sans_paniquer() {
    let src = unie(8, 8, [255, 0, 0, 255]);
    let vue = Vue::nouvelle(&src, 8, 8).unwrap();
    let mut fond = unie(6, 6, [0, 0, 0, 255]);
    let mut dest = VueMut::nouvelle(&mut fond, 6, 6).unwrap();

    let ecrits = reporter(
        &mut dest,
        &vue,
        Pose {
            x: -3.0,
            y: -3.0,
            largeur: 8.0,
            hauteur: 8.0,
        },
        toute(6, 6),
        Melange::Remplacer,
    );

    assert_eq!(ecrits, 25, "cinq colonnes par cinq lignes restent visibles");
    assert_eq!(fond[0], [255, 0, 0, 255]);
    assert_eq!(fond[35], [0, 0, 0, 255], "le coin bas-droit reste au fond");
}

#[test]
fn composer_une_source_transparente_ne_change_rien() {
    let src = unie(4, 4, [0, 0, 0, 0]);
    let vue = Vue::nouvelle(&src, 4, 4).unwrap();
    let avant = unie(8, 8, [30, 60, 90, 255]);
    let mut fond = avant.clone();
    let mut dest = VueMut::nouvelle(&mut fond, 8, 8).unwrap();

    reporter(
        &mut dest,
        &vue,
        Pose {
            x: 1.0,
            y: 1.0,
            largeur: 4.0,
            hauteur: 4.0,
        },
        toute(8, 8),
        Melange::Composer,
    );

    assert_eq!(fond, avant);
}

#[test]
fn composer_une_source_opaque_remplace() {
    let src = unie(4, 4, [10, 20, 30, 255]);
    let vue = Vue::nouvelle(&src, 4, 4).unwrap();
    let mut fond = unie(8, 8, [200, 200, 200, 255]);
    let mut dest = VueMut::nouvelle(&mut fond, 8, 8).unwrap();

    reporter(
        &mut dest,
        &vue,
        Pose {
            x: 0.0,
            y: 0.0,
            largeur: 4.0,
            hauteur: 4.0,
        },
        toute(8, 8),
        Melange::Composer,
    );

    assert_eq!(fond[0], [10, 20, 30, 255]);
}

/// Une couleur unie reste elle-même à toute échelle : l'interpolation ne doit rien inventer.
#[test]
fn agrandir_une_couleur_unie_ne_la_change_pas() {
    let src = unie(3, 3, [77, 133, 201, 255]);
    let vue = Vue::nouvelle(&src, 3, 3).unwrap();
    for taille in [4.0f32, 7.5, 31.0, 100.0] {
        let mut fond = unie(128, 128, [0, 0, 0, 255]);
        let mut dest = VueMut::nouvelle(&mut fond, 128, 128).unwrap();
        let ecrits = reporter(
            &mut dest,
            &vue,
            Pose {
                x: 2.0,
                y: 2.0,
                largeur: taille,
                hauteur: taille,
            },
            toute(128, 128),
            Melange::Remplacer,
        );
        assert!(ecrits > 0);
        // Le centre de la zone posée : loin des bords, donc aucune excuse.
        let (cx, cy) = (2 + taille as u32 / 2, 2 + taille as u32 / 2);
        assert_eq!(
            fond[(cy * 128 + cx) as usize],
            [77, 133, 201, 255],
            "a l'echelle {taille}, la couleur a derive"
        );
    }
}

/// Réduire ne doit jamais lire hors de la source, quelle que soit l'échelle.
#[test]
fn reduire_beaucoup_ne_sort_jamais_de_la_source() {
    let src = damier(64, 64);
    let vue = Vue::nouvelle(&src, 64, 64).unwrap();
    let mut fond = unie(32, 32, [0, 0, 0, 255]);
    let mut dest = VueMut::nouvelle(&mut fond, 32, 32).unwrap();

    let ecrits = reporter(
        &mut dest,
        &vue,
        Pose {
            x: 0.0,
            y: 0.0,
            largeur: 3.0,
            hauteur: 3.0,
        },
        toute(32, 32),
        Melange::Remplacer,
    );
    assert_eq!(ecrits, 9);
}

#[test]
fn une_vue_dont_la_taille_ment_est_refusee() {
    let pixels = unie(4, 4, [0, 0, 0, 255]);
    assert!(Vue::nouvelle(&pixels, 5, 4).is_none());
    assert!(Vue::nouvelle(&pixels, 0, 0).is_none());
    assert!(Vue::nouvelle(&pixels, 4, 4).is_some());
}

/// `a × b / 255` doit être exact partout : c'est la base de la composition.
#[test]
fn la_multiplication_par_un_octet_est_exacte() {
    for a in 0..=255u8 {
        for b in 0..=255u8 {
            let attendu = (u32::from(a) * u32::from(b) + 127) / 255;
            assert_eq!(u32::from(mul255(a, b)), attendu, "{a} x {b} / 255 est faux");
        }
    }
}

/// Le domaine écrit est exactement l'aire du rectangle visible : pas un pixel de plus.
#[test]
fn le_nombre_de_pixels_ecrits_est_l_aire_du_visible() {
    let src = unie(50, 50, [1, 1, 1, 255]);
    let vue = Vue::nouvelle(&src, 50, 50).unwrap();
    let mut fond = unie(64, 64, [0, 0, 0, 255]);
    let mut dest = VueMut::nouvelle(&mut fond, 64, 64).unwrap();

    // Une bande de trois pixels de large : c'est le cas que l'occlusion produit, et tout
    // l'intérêt de cette primitive est qu'il coûte trois colonnes et non cinquante.
    let ecrits = reporter(
        &mut dest,
        &vue,
        Pose {
            x: 0.0,
            y: 0.0,
            largeur: 50.0,
            hauteur: 50.0,
        },
        Boite::nouvelle(0.0, 0.0, 3.0, 50.0),
        Melange::Remplacer,
    );
    assert_eq!(ecrits, 150);
}

/// Les quatre canaux voyagent dans un même entier : rien ne doit déborder de l'un sur l'autre.
///
/// Une couleur unie ne le dirait pas — elle reste juste même si les champs se confondent. Il
/// faut donc quatre canaux différents, et deux qui saturent en sens contraire.
#[test]
fn l_interpolation_ne_melange_jamais_deux_canaux() {
    let a = [10, 20, 30, 40];
    let b = [210, 220, 230, 240];
    assert_eq!(melanger(a, b, 0), a, "poids nul : la source de gauche");
    assert_eq!(melanger(a, b, 128), [110, 120, 130, 140], "a mi-chemin");

    // Deux canaux au maximum, deux au minimum, echanges : un debordement se verrait aussitot.
    assert_eq!(melanger([255, 0, 255, 0], [0, 255, 0, 255], 128), [128; 4]);
    assert_eq!(melanger([255, 255, 255, 255], [0, 0, 0, 0], 0), [255; 4]);
}
