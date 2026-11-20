//! Les épreuves du détourage (DETOURER-1 à 3, fiche 121).

use super::calcul::{agrandir, executer, maxpool, Tenseur};
use super::modele::{Conv, Modele};
use super::{conv, echantillon, embarque, fond_uni, premier_plan};

/// L'entrée d'épreuve, la même que `entree_d_epreuve` de `outils/detourage/reference.py` : un
/// disque uni sur des rayures, exact en binaire.
fn entree_d_epreuve(taille: usize) -> Tenseur {
    let (centre, rayon) = (taille / 2, taille * 9 / 32);
    let mut v = Vec::with_capacity(3 * taille * taille);
    for c in 0..3 {
        for y in 0..taille {
            for x in 0..taille {
                let (dx, dy) = (x as i64 - centre as i64, y as i64 - centre as i64);
                v.push(if dx * dx + dy * dy < (rayon * rayon) as i64 {
                    [1.5, 0.5, -1.0][c]
                } else {
                    (((c * 31 + x * 13) % 97) as f32 - 48.0) / 32.0
                });
            }
        }
    }
    Tenseur {
        c: 3,
        h: taille,
        l: taille,
        v,
    }
}

/// L'étalon écrit par `reference.py etalon` : un pixel sur huit, la moyenne, le maximum.
fn etalon() -> (usize, Vec<f32>, f32, f32) {
    let o = include_bytes!("tests/etalon-u2netp.bin");
    let f = |i: usize| f32::from_le_bytes([o[i], o[i + 1], o[i + 2], o[i + 3]]);
    let n = u32::from_le_bytes([o[0], o[1], o[2], o[3]]) as usize;
    let valeurs = (0..n * n).map(|k| f(8 + 4 * k)).collect();
    (n, valeurs, f(8 + 4 * n * n), f(12 + 4 * n * n))
}

#[test]
fn test_detourer_1_le_modele_embarque_se_lit() {
    let m = embarque().expect("le modèle embarqué se lit");
    assert_eq!(m.taille, 320);
    assert_eq!(m.operations.len(), 253);
}

/// **Le moteur rend l'étalon en numpy** (lui-même à 2·10⁻⁵ d'ONNX Runtime), sur toute l'image.
/// La borne : numpy accumule ses convolutions en f64, le moteur en f32 (et en FMA quand le
/// processeur l'a) — sur 119 convolutions enchaînées, l'écart mesuré est de 4·10⁻⁵ (le 18/11, et
/// encore avec la convolution par tuiles, le 19/11) ; un opérateur faux (un décalage d'un pixel, un
/// repère d'agrandissement, un ReLU oublié) le porte à plusieurs dixièmes.
#[test]
fn test_detourer_1_le_moteur_rend_l_etalon() {
    let m = embarque().expect("modèle");
    let brut = executer(m, entree_d_epreuve(320), 4);
    let (n, attendu, moyenne, max) = etalon();
    let mut pire = 0.0_f32;
    for (k, a) in attendu.iter().enumerate() {
        let (y, x) = (k / n * 8, k % n * 8);
        pire = pire.max((brut.v[y * 320 + x] - a).abs());
    }
    let moy = (brut.v.iter().map(|&v| f64::from(v)).sum::<f64>() / brut.v.len() as f64) as f32;
    let mx = brut.v.iter().copied().fold(f32::MIN, f32::max);
    assert!(pire < 1e-3, "écart à l'étalon : {pire}");
    assert!(
        (moy - moyenne).abs() < 1e-4 && (mx - max).abs() < 1e-4,
        "{moy} {moyenne} {mx} {max}"
    );
    // Le disque est la forme principale, le coin ne l'est pas.
    assert!(brut.v[160 * 320 + 160] > 0.99 && brut.v[5 * 320 + 5] < 0.01);
}

/// Une convolution, par sa définition, terme à terme.
fn conv_naive(x: &Tenseur, c: &Conv, relu: bool) -> Tenseur {
    let (ho, lo) = (c.sortie(x.h, c.kh).unwrap(), c.sortie(x.l, c.kl).unwrap());
    let mut v = vec![0.0_f64; c.cout * ho * lo];
    for co in 0..c.cout {
        for yo in 0..ho {
            for xo in 0..lo {
                let mut s = f64::from(c.biais[co]);
                for ci in 0..c.cin {
                    for ky in 0..c.kh {
                        for kx in 0..c.kl {
                            let iy = (yo * c.pas + ky * c.dilatation) as i64 - c.remplissage as i64;
                            let ix = (xo * c.pas + kx * c.dilatation) as i64 - c.remplissage as i64;
                            if iy < 0 || ix < 0 || iy >= x.h as i64 || ix >= x.l as i64 {
                                continue;
                            }
                            let w = c.poids[((co * c.cin + ci) * c.kh + ky) * c.kl + kx];
                            let e = x.v[(ci * x.h + iy as usize) * x.l + ix as usize];
                            s += f64::from(w) * f64::from(e);
                        }
                    }
                }
                v[(co * ho + yo) * lo + xo] = if relu { s.max(0.0) } else { s };
            }
        }
    }
    Tenseur {
        c: c.cout,
        h: ho,
        l: lo,
        v: v.into_iter().map(|x| x as f32).collect(),
    }
}

/// Des valeurs pseudo-aléatoires reproductibles, entre −1 et 1.
fn hasard(n: usize, graine: u64) -> Vec<f32> {
    let mut s = graine;
    (0..n)
        .map(|_| {
            s = s
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            ((s >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
        })
        .collect()
}

/// **La convolution rapide égale sa définition** — pas, dilatation, remplissage, un nombre de
/// canaux qui n'est pas multiple du bloc, un ou plusieurs fils (au bit près entre eux).
#[test]
fn test_detourer_1_la_convolution_egale_sa_definition() {
    let x = Tenseur {
        c: 5,
        h: 13,
        l: 11,
        v: hasard(5 * 13 * 11, 1),
    };
    for (cout, k, pas, dil, rempl, relu) in [
        (6, 3, 1, 2, 2, true),
        (6, 3, 2, 1, 1, false),
        (5, 3, 2, 2, 2, true),
        (1, 1, 1, 1, 0, false),
        (9, 3, 1, 4, 4, true),
    ] {
        let c = Conv {
            cin: 5,
            cout,
            kh: k,
            kl: k,
            pas,
            dilatation: dil,
            remplissage: rempl,
            poids: hasard(cout * 5 * k * k, 2 + cout as u64),
            biais: hasard(cout, 7),
        };
        let attendu = conv_naive(&x, &c, relu);
        let un = conv::conv(&x, &c, relu, 1);
        assert_eq!((un.c, un.h, un.l), (attendu.c, attendu.h, attendu.l));
        let pire =
            un.v.iter()
                .zip(&attendu.v)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f32::max);
        assert!(
            pire < 1e-5,
            "écart {pire} pour {cout} {k} {pas} {dil} {rempl}"
        );
        assert_eq!(
            conv::conv(&x, &c, relu, 3),
            un,
            "les fils changent le résultat"
        );
    }
}

#[test]
fn test_detourer_1_le_regroupement_au_plafond_lit_la_derniere_fenetre() {
    let x = Tenseur {
        c: 1,
        h: 3,
        l: 5,
        v: (0..15).map(|i| i as f32).collect(),
    };
    let y = maxpool(&x, 2, 2, true);
    assert_eq!((y.h, y.l), (2, 3));
    assert_eq!(y.v, vec![6.0, 8.0, 9.0, 11.0, 13.0, 14.0]);
    assert_eq!(maxpool(&x, 2, 2, false).l, 2);
}

/// L'agrandissement au repère « demi-pixel » : de 2 à 4, les poids ¼ et ¾, les bords tenus.
#[test]
fn test_detourer_1_l_agrandissement_est_au_demi_pixel() {
    let x = Tenseur {
        c: 1,
        h: 2,
        l: 2,
        v: vec![0.0, 4.0, 8.0, 12.0],
    };
    let y = agrandir(&x, 2, 4);
    // Ligne du haut, colonnes : 0 (bord), ¼·4 = 1, ¾·4 = 3, 4 (bord) ; idem en bas, +8.
    assert_eq!(&y.v[..4], &[0.0, 1.0, 3.0, 4.0]);
    assert_eq!(&y.v[4..], &[8.0, 9.0, 11.0, 12.0]);
}

/// Un fichier abîmé se refuse à la lecture — jamais au milieu d'un calcul.
#[test]
fn test_detourer_1_un_modele_abime_se_refuse() {
    let bon = super::U2NETP;
    assert!(Modele::lire(&bon[..bon.len() - 1]).is_err(), "tronqué");
    let mut plus = bon.to_vec();
    plus.push(0);
    assert!(Modele::lire(&plus).is_err(), "un octet de trop");
    let mut signe = bon.to_vec();
    signe[0] = b'X';
    assert!(Modele::lire(&signe).is_err(), "signature");
    // Le masque désigné ailleurs que sur la dernière sigmoïde : il n'a pas la forme de l'image.
    let mut sortie = bon.to_vec();
    sortie[20..24].copy_from_slice(&1u32.to_le_bytes());
    assert!(Modele::lire(&sortie).is_err(), "sortie");
}

#[test]
fn test_detourer_1_le_reechantillonnage_garde_l_identite_et_les_aplats() {
    let plan: Vec<f32> = hasard(7 * 5, 3);
    assert_eq!(echantillon::reechantillonner(&plan, (7, 5), (7, 5)), plan);
    let uni = vec![0.25; 40 * 30];
    for (l, h) in [(13, 9), (80, 70)] {
        let r = echantillon::reechantillonner(&uni, (40, 30), (l, h));
        assert!(r.iter().all(|v| (v - 0.25).abs() < 1e-6));
    }
}

/// **DETOURER-2** : sur un bord où `I = α·F + (1 − α)·B`, l'estimation rend `F` bien mieux que
/// `I` ne le faisait — le liseré du fond disparaît. La méthode est une approximation (sa première
/// passe part de l'image, déjà mêlée au bord) : mesuré ici, 0,72 → 0,023 ; la loi exige au moins
/// vingt fois moins de liseré. Sur ce sujet uni, la moyenne floue seule retombe déjà sur sa couleur :
/// la correction se garde à part, là où le sujet est plein (l'épreuve suivante, fiche 127).
#[test]
fn test_detourer_2_le_premier_plan_retire_la_couleur_du_fond() {
    let (l, h) = (120, 80);
    let (f, b) = ([0.9, 0.2, 0.1], [0.1, 0.8, 0.3]);
    let alpha: Vec<f32> = (0..l * h)
        .map(|i| ((i % l) as f32 - 55.0).clamp(0.0, 10.0) / 10.0)
        .collect();
    let rvb: [Vec<f32>; 3] =
        std::array::from_fn(|c| alpha.iter().map(|a| a * f[c] + (1.0 - a) * b[c]).collect());
    let estime = premier_plan::estimer(&rvb, &alpha, (l, h));
    let (mut avant, mut apres) = (0.0_f32, 0.0_f32);
    for i in (0..l * h).filter(|&i| alpha[i] > 0.05 && alpha[i] < 0.95) {
        for c in 0..3 {
            avant = avant.max((rvb[c][i] - f[c]).abs());
            apres = apres.max((estime[c][i] - f[c]).abs());
        }
    }
    assert!(
        apres * 20.0 < avant && avant > 0.5,
        "avant {avant}, après {apres}"
    );
}

/// **DETOURER-2 — là où le sujet est plein, son premier plan est l'image même** : à l'opacité 1,
/// `I = F`, et la correction de Forte le rend exactement (`fm + 1 · (I − fm)`), quand la moyenne floue
/// des couleurs voisines, seule, rendrait une bouillie de ses rayures. Le sujet est rayé pour que le
/// cas soit exercé : sur un sujet uni (l'épreuve d'au-dessus), le flou seul retombe sur sa couleur,
/// et rien ne distinguait la formule de sa moitié — un sabotage aveugle (fiche 127).
#[test]
fn test_detourer_2_la_ou_le_sujet_est_plein_le_premier_plan_est_l_image() {
    let (l, h) = (120, 80);
    let alpha: Vec<f32> = (0..l * h)
        .map(|i| ((i % l) as f32 - 55.0).clamp(0.0, 10.0) / 10.0)
        .collect();
    // Le sujet : des rayures de 8 pixels, rouges et bleues ; le fond, vert.
    let sujet = |i: usize| {
        if (i / l / 8) % 2 == 0 {
            [0.9, 0.2, 0.1]
        } else {
            [0.1, 0.2, 0.9]
        }
    };
    let b = [0.1, 0.8, 0.3];
    let rvb: [Vec<f32>; 3] = std::array::from_fn(|c| {
        (0..l * h)
            .map(|i| alpha[i] * sujet(i)[c] + (1.0 - alpha[i]) * b[c])
            .collect()
    });
    let estime = premier_plan::estimer(&rvb, &alpha, (l, h));
    let pleins: Vec<usize> = (0..l * h).filter(|&i| alpha[i] == 1.0).collect();
    let ecart_max = |plan: &dyn Fn(usize) -> f32| {
        pleins
            .iter()
            .map(|&i| (plan(i) - rvb[0][i]).abs())
            .fold(0.0_f32, f32::max)
    };
    let pire = ecart_max(&|i| estime[0][i]);
    assert!(pire < 1e-5, "le premier plan s'écarte de l'image de {pire}");
    // La preuve à l'envers : la moyenne floue seule (la première passe) ne rend pas les rayures.
    let fa: Vec<f32> = (0..l * h).map(|i| rvb[0][i] * alpha[i]).collect();
    let (fa, af) = (
        premier_plan::flou(&fa, (l, h), 90),
        premier_plan::flou(&alpha, (l, h), 90),
    );
    let flou_seul = ecart_max(&|i| fa[i] / (af[i] + 1e-5));
    assert!(
        flou_seul > 0.2,
        "le cas : le flou seul s'en écarte de {flou_seul}"
    );
}

#[test]
fn test_detourer_2_le_flou_est_la_moyenne_de_la_fenetre_dans_l_image() {
    let (l, h, r) = (9, 7, 2);
    let plan = hasard(l * h, 5);
    let flou = premier_plan::flou(&plan, (l, h), r);
    for y in 0..h {
        for x in 0..l {
            let (ys, xs) = (
                y.saturating_sub(r)..(y + r + 1).min(h),
                x.saturating_sub(r)..(x + r + 1).min(l),
            );
            let n = ys.len() * xs.len();
            let p = &plan;
            let s: f32 = ys
                .flat_map(|yy| xs.clone().map(move |xx| p[yy * l + xx]))
                .sum();
            assert!((flou[y * l + x] - s / n as f32).abs() < 1e-5);
        }
    }
}

/// **DETOURER-3** : un trait gris d'anticrénelage sur blanc devient du noir à demi transparent ;
/// reposée sur le blanc, l'image redonne l'original ; le blanc devient transparent.
#[test]
fn test_detourer_3_le_fond_uni_se_retire_exactement() {
    let (l, h) = (6, 4);
    let mut rvba = vec![255u8; l * h * 4];
    let gris = [(2, 1, 128u8), (3, 1, 0), (3, 2, 200)];
    for &(x, y, v) in &gris {
        rvba[(y * l + x) * 4..][..3].copy_from_slice(&[v, v, v]);
    }
    let sans = fond_uni::retirer(&rvba, (l, h)).expect("le bord est blanc");
    for (i, p) in sans.as_chunks::<4>().0.iter().enumerate() {
        let original = rvba[i * 4];
        if original == 255 {
            assert_eq!(p[3], 0, "le blanc devient transparent");
            continue;
        }
        assert_eq!(p[0], 0, "le trait devient noir");
        let a = f32::from(p[3]) / 255.0;
        let repose = a * f32::from(p[0]) + (1.0 - a) * 255.0;
        assert!(
            (repose - f32::from(original)).abs() <= 1.0,
            "{repose} contre {original}"
        );
    }
}

/// L'opacité qu'avait déjà un pixel se garde : on retire le fond, pas davantage.
#[test]
fn test_detourer_3_l_opacite_d_un_pixel_se_garde() {
    let mut rvba: Vec<u8> = (0..16).flat_map(|_| [255, 255, 255, 255]).collect();
    rvba[5 * 4..][..4].copy_from_slice(&[0, 0, 0, 128]);
    let sans = fond_uni::retirer(&rvba, (4, 4)).expect("un fond blanc");
    assert_eq!(&sans[5 * 4..][..4], &[0, 0, 0, 128]);
}

#[test]
fn test_detourer_3_un_bord_deja_transparent_n_a_pas_de_fond() {
    let rvba = vec![0u8; 4 * 4 * 4];
    assert!(fond_uni::retirer(&rvba, (4, 4)).is_none());
}

/// La couleur du fond est celle de la majorité du bord, même quand l'objet le touche.
#[test]
fn test_detourer_3_la_couleur_du_fond_est_la_mediane_du_bord() {
    let (l, h) = (5, 5);
    let mut rvba: Vec<u8> = (0..l * h).flat_map(|_| [240, 240, 240, 255]).collect();
    for x in 0..2 {
        rvba[x * 4..][..3].copy_from_slice(&[10, 20, 30]);
    }
    assert_eq!(
        fond_uni::couleur_du_bord(&rvba, (l, h)),
        Some([240, 240, 240])
    );
}

/// Un petit modèle écrit à la main, au format de `convertir.py` : l'entrée `3 × 8 × 8`, puis des
/// opérations `(code, entrées, sortie, corps)`.
fn petit_modele(tenseurs: u32, sortie: u32, ops: &[(u8, Vec<u32>, u32, Vec<u8>)]) -> Vec<u8> {
    let mut o = b"GLUCDET1".to_vec();
    for v in [8, tenseurs, ops.len() as u32, sortie] {
        o.extend(v.to_le_bytes());
    }
    for v in [0.0_f32, 0.0, 0.0, 1.0, 1.0, 1.0] {
        o.extend(v.to_le_bytes());
    }
    for (code, entrees, s, corps) in ops {
        o.extend([*code, 0]);
        o.extend((entrees.len() as u16).to_le_bytes());
        o.extend(s.to_le_bytes());
        entrees.iter().for_each(|e| o.extend(e.to_le_bytes()));
        o.extend(corps);
    }
    o
}

/// Le corps d'une convolution `1 × 1` de `cin` canaux vers un.
fn conv_vers_un(cin: u32) -> Vec<u8> {
    let mut o = Vec::new();
    [cin, 1, 1, 1, 1, 1, 0]
        .iter()
        .for_each(|v| o.extend(v.to_le_bytes()));
    (0..=cin).for_each(|_| o.extend(0.5_f32.to_le_bytes()));
    o
}

/// **Un graphe qui ne s'enchaîne pas se refuse à la lecture** : un modèle téléchargé abîmé (ISNet,
/// un jour) ne doit jamais planter au milieu d'un calcul. Le même graphe, juste, se lit.
#[test]
fn test_detourer_1_un_graphe_qui_ne_s_enchaine_pas_se_refuse() {
    let pool = 2_u32
        .to_le_bytes()
        .into_iter()
        .chain(2_u32.to_le_bytes())
        .chain(0_u32.to_le_bytes());
    let pool: Vec<u8> = pool.collect();
    let juste = petit_modele(
        3,
        2,
        &[(1, vec![0], 1, conv_vers_un(3)), (6, vec![1], 2, vec![])],
    );
    assert!(Modele::lire(&juste).is_ok(), "le graphe juste se lit");
    let faux = [
        // Chaque cas n'a qu'une faute — celle qu'il éprouve : sinon une autre garde le
        // refuserait, et la sienne pourrait disparaître sans qu'il tombe.
        (
            "lu avant d'être écrit",
            petit_modele(
                3,
                1,
                &[
                    (1, vec![2], 1, conv_vers_un(3)),
                    (1, vec![0], 2, conv_vers_un(3)),
                ],
            ),
        ),
        (
            "écrit deux fois",
            petit_modele(
                3,
                1,
                &[
                    (1, vec![0], 1, conv_vers_un(3)),
                    (1, vec![0], 1, conv_vers_un(3)),
                ],
            ),
        ),
        (
            "des canaux qui ne sont pas les siens",
            petit_modele(
                3,
                2,
                &[(1, vec![0], 1, conv_vers_un(2)), (6, vec![1], 2, vec![])],
            ),
        ),
        (
            "concaténer deux tailles",
            petit_modele(
                5,
                4,
                &[
                    (1, vec![0], 1, conv_vers_un(3)),
                    (2, vec![1], 2, pool.clone()),
                    (4, vec![1, 2], 3, vec![]),
                    (1, vec![3], 4, conv_vers_un(2)),
                ],
            ),
        ),
        (
            "ajouter deux formes",
            petit_modele(
                4,
                3,
                &[
                    (1, vec![0], 1, conv_vers_un(3)),
                    (2, vec![1], 2, pool.clone()),
                    (5, vec![1, 2], 3, vec![]),
                ],
            ),
        ),
        (
            "un masque plus petit que l'image",
            petit_modele(
                3,
                2,
                &[
                    (1, vec![0], 1, conv_vers_un(3)),
                    (2, vec![1], 2, pool.clone()),
                ],
            ),
        ),
    ];
    for (pourquoi, octets) in faux {
        assert!(Modele::lire(&octets).is_err(), "{pourquoi} : lu quand même");
    }
}

/// **Le masque s'étire de 0 à 1** (`normPRED`, les auteurs d'U²-Net) : à la taille même de
/// l'entrée, le rééchantillonnage est l'identité, et l'étirement se lit exactement.
#[test]
fn test_detourer_1_le_masque_s_etire_de_zero_a_un() {
    let m = super::embarque().expect("modèle");
    let (centre, rayon) = (160_i64, 90_i64);
    let rvba: Vec<u8> = (0..320 * 320)
        .flat_map(|i: i64| {
            let (x, y) = (i % 320 - centre, i / 320 - centre);
            if x * x + y * y < rayon * rayon {
                [230, 120, 20, 255]
            } else {
                [(i % 97) as u8 * 2, 90, 200 - (i % 53) as u8, 255]
            }
        })
        .collect();
    let masque = super::masque(m, &rvba, (320, 320), 4);
    let (min, max) = masque
        .iter()
        .fold((1.0_f32, 0.0_f32), |(a, b), &v| (a.min(v), b.max(v)));
    assert_eq!((min, max), (0.0, 1.0));
}
