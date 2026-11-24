//! Les épreuves du flou en pyramide (Q12, fiche 137).

use super::*;

/// Une image de `largeur × hauteur` noire, un seul pixel allumé à `(x, y)`.
fn impulsion((largeur, hauteur): (usize, usize), (x, y): (usize, usize)) -> Image {
    let mut pixels = vec![[0f32; 4]; largeur * hauteur];
    pixels[y * largeur + x] = [255.0, 0.0, 0.0, 255.0];
    Image {
        largeur,
        hauteur,
        pixels,
    }
}

/// Des octets RVBA d'une image dont chaque canal tient en un octet.
fn octets(img: &Image) -> Vec<u8> {
    img.pixels
        .iter()
        .flat_map(|p| p.map(|c| c.round().clamp(0.0, 255.0) as u8))
        .collect()
}

/// Le flou entier, jusqu'à l'image : la pyramide puis la dernière montée.
fn flou_entier(img: &Image, plan: &Plan) -> Image {
    let demi = flouter(&octets(img), (img.largeur, img.hauteur), plan);
    if plan.etages == 0 {
        demi
    } else {
        agrandir(&demi, (img.largeur, img.hauteur))
    }
}

/// La masse, le centre et la variance d'une image sur l'axe des x (canal rouge).
fn moments(img: &Image) -> (f64, f64, f64) {
    let mut masse = 0.0;
    let mut premier = 0.0;
    for y in 0..img.hauteur {
        for x in 0..img.largeur {
            let v = f64::from(img.pixels[y * img.largeur + x][0]);
            masse += v;
            premier += v * x as f64;
        }
    }
    let centre = premier / masse;
    let mut second = 0.0;
    for y in 0..img.hauteur {
        for x in 0..img.largeur {
            let v = f64::from(img.pixels[y * img.largeur + x][0]);
            second += v * (x as f64 - centre).powi(2);
        }
    }
    (masse, centre, second / masse)
}

/// **La loi des variances tient** : la réponse de la pyramide à une impulsion a la variance σ²
/// annoncée — à la troncature près du noyau à 3σ, qui en retire moins de 1,5 % — et garde toute
/// sa masse, quel que soit le nombre d'étages et la place de l'impulsion sur la grille.
#[test]
fn test_flou_1_la_reponse_a_une_impulsion_a_la_variance_annoncee() {
    for sigma in [2.0, 3.0, 5.6, 11.2, 20.0] {
        let plan = Plan::de(sigma);
        for phase in 0..4 {
            let img = impulsion((257, 161), (128 + phase, 80 + phase));
            let (masse, _, variance) = moments(&flou_entier(&img, &plan));
            assert!(
                (masse - 255.0).abs() < 1e-2,
                "σ = {sigma} : la masse se garde ({masse})"
            );
            let attendue = sigma * sigma;
            assert!(
                variance <= attendue && variance > 0.985 * attendue,
                "σ = {sigma}, phase {phase}, {} étage(s) : variance {variance} pour {attendue}",
                plan.etages
            );
        }
    }
}

/// **Combien d'étages : la loi de Nyquist, pas un seuil choisi.** Le flou qui reste sur la grille
/// la plus grossière efface encore sa fréquence de Nyquist sous un niveau de huit bits, et un étage
/// de plus ne le ferait plus. À l'envers : le seuil de Vello (réduire tant que la variance dépasse
/// 4) réduit une fois un flou de σ = 2,3 que la loi garde entier.
#[test]
fn test_flou_1_le_nombre_d_etages_suit_la_loi_de_nyquist() {
    let seuil = variance_de_nyquist();
    assert!((seuil - 1.1229).abs() < 1e-3, "2 ln 255 / π² = {seuil}");
    for sigma in [1.0, 2.3, 5.6, 14.0, 40.0] {
        let plan = Plan::de(sigma);
        // La variance qui reste, depuis le noyau.
        let r = plan.noyau.len() - 1;
        let reste: f64 = (1..=r)
            .map(|x| 2.0 * f64::from(plan.noyau[x]) * (x * x) as f64)
            .sum();
        let s = (sigma * sigma - 1.5 * ((1 << (2 * plan.etages)) as f64 - 1.0) / 3.0)
            / f64::from(1u32 << (2 * plan.etages));
        assert!(
            (reste - s).abs() / s.max(1e-9) < 0.02 || r == 0,
            "σ = {sigma} : le noyau porte {reste}, la loi {s}"
        );
        if plan.etages > 0 {
            assert!(s >= seuil, "σ = {sigma} : le reste {s} efface sa Nyquist");
        }
        assert!(
            (s - 1.5) / 4.0 < seuil,
            "σ = {sigma} : un étage de plus ne l'effacerait plus"
        );
    }
    assert_eq!(Plan::de(2.3).etages, 0, "la loi garde σ = 2,3 entier");
    let vello = |sigma: f64| {
        let (mut v, mut n) = (sigma * sigma, 0);
        while v > 4.0 {
            v = (v - 1.5) * 0.25;
            n += 1;
        }
        n
    };
    assert_eq!(vello(2.3), 1, "le seuil de Vello l'aurait réduit une fois");
}

/// **La réduction est le filtre `[1, 3, 3, 1] / 8`**, bords prolongés : sur une rampe, chaque
/// pixel de la petite grille est la moyenne pondérée de quatre de la grande — et une largeur
/// impaire garde sa dernière maille.
#[test]
fn test_flou_1_la_reduction_est_le_filtre_binomial() {
    let rampe = Image {
        largeur: 7,
        hauteur: 1,
        pixels: (0..7).map(|x| [x as f32 * 8.0, 0.0, 0.0, 0.0]).collect(),
    };
    let petite = reduire(&rampe);
    assert_eq!((petite.largeur, petite.hauteur), (4, 1));
    let v: Vec<f32> = petite.pixels.iter().map(|p| p[0]).collect();
    // i = 0 : (a0 + 3 a0 + 3 a1 + a2) / 8, a(−1) prolongé en a0.
    assert_eq!(v[0], (0.0 + 0.0 + 3.0 * 8.0 + 16.0) / 8.0);
    assert_eq!(v[1], (8.0 + 3.0 * 16.0 + 3.0 * 24.0 + 32.0) / 8.0);
    // i = 3 : a5, a6, puis a7 et a8 prolongés en a6.
    assert_eq!(v[3], (40.0 + 3.0 * 48.0 + 3.0 * 48.0 + 48.0) / 8.0);
}

/// **L'agrandissement lit sous le centre de chaque pixel** : les poids `¾` et `¼`, ceux d'une
/// lecture bilinéaire de carte graphique au quart d'un texel.
#[test]
fn test_flou_1_l_agrandissement_lit_aux_quarts() {
    let deux = Image {
        largeur: 2,
        hauteur: 1,
        pixels: vec![[0.0; 4], [100.0, 0.0, 0.0, 0.0]],
    };
    let quatre = agrandir(&deux, (4, 1));
    let v: Vec<f32> = quatre.pixels.iter().map(|p| p[0]).collect();
    assert_eq!(v, vec![0.0, 25.0, 75.0, 100.0]);
}

/// **Le mélange** : où le masque est plein, l'image nette reste telle quelle ; où il est vide,
/// c'est le flou ; à moitié, entre les deux.
#[test]
fn test_flou_1_le_masque_dit_ce_qui_reste_net() {
    let (l, h) = (16usize, 16usize);
    let mut damier = vec![0u8; l * h * 4];
    for y in 0..h {
        for x in 0..l {
            let v = if (x + y) % 2 == 0 { 255 } else { 0 };
            damier[(y * l + x) * 4..][..4].copy_from_slice(&[v, v, v, 255]);
        }
    }
    let plan = Plan::de(4.0);
    assert_eq!(plan.etages, 1);
    let flou = flouter(&damier, (l, h), &plan);
    assert_eq!(grille(&plan, (l, h)), (8, 8));
    let masque_de = |v: u8| Masque {
        largeur: 8,
        hauteur: 8,
        octets: vec![v; 64],
    };
    let mut plein = damier.clone();
    meler(&mut plein, (l, h), (&flou, &plan), &masque_de(255));
    assert_eq!(plein, damier, "masque plein : l'image reste nette");
    let mut vide = damier.clone();
    meler(&mut vide, (l, h), (&flou, &plan), &masque_de(0));
    // À moitié : chaque pixel où le net et le flou diffèrent tombe strictement entre les deux.
    let mut moitie = damier.clone();
    meler(&mut moitie, (l, h), (&flou, &plan), &masque_de(128));
    let mut entre = 0;
    for i in (0..l * h * 4).step_by(4) {
        let (n, f, m) = (damier[i], vide[i], moitie[i]);
        if n.abs_diff(f) > 4 {
            assert!(
                m > n.min(f) && m < n.max(f),
                "à moitié, {m} n'est pas entre le net {n} et le flou {f}"
            );
            entre += 1;
        }
    }
    assert!(entre > 50, "le mélange s'éprouve sur {entre} pixels");
    // Le damier flouté tend vers le gris moyen.
    let milieu = (8 * l + 8) * 4;
    assert!(
        (i32::from(vide[milieu]) - 128).abs() < 24,
        "flouté, le damier est gris : {}",
        vide[milieu]
    );
}

/// **Le mélange rend au bit près la lecture pixel par pixel** (fiche 140 § 5) : ses pas, calculés
/// une fois par colonne et par rangée, donnent les nombres mêmes de [`lire`] et [`Masque::lire`] à
/// chaque pixel — l'ancienne loi, rejouée ici sur des tailles impaires, un masque en dégradé (net,
/// flou et entre les deux), et sans étage comme avec deux.
#[test]
fn test_flou_meler_rend_au_bit_pres_la_lecture_pixel_par_pixel() {
    for sigma in [0.6, 5.6] {
        let plan = Plan::de(sigma);
        let (l, h) = (37usize, 23usize);
        let net: Vec<u8> = (0..l * h * 4).map(|i| (i * 37 % 251) as u8).collect();
        let flou = flouter(&net, (l, h), &plan);
        let (gl, gh) = grille(&plan, (l, h));
        let masque = Masque {
            largeur: gl,
            hauteur: gh,
            octets: (0..gl * gh).map(|i| (i * 97 % 256) as u8).collect(),
        };
        let mut nouveau = net.clone();
        meler(&mut nouveau, (l, h), (&flou, &plan), &masque);
        let mut ancien = net.clone();
        let maille = plan.maille() as f32;
        let centre = |j: usize| (j as f32 + 0.5) / maille - 0.5;
        for y in 0..h {
            for x in 0..l {
                let f = if plan.etages == 0 {
                    flou.pixels[y * l + x]
                } else {
                    lire(&flou, (sous(x), sous(y)))
                };
                let m = masque.lire((centre(x), centre(y))).clamp(0.0, 1.0);
                if m >= 1.0 {
                    continue;
                }
                let i = (y * l + x) * 4;
                for c in 0..4 {
                    let v = f32::from(ancien[i + c]) * m + f[c] * (1.0 - m);
                    ancien[i + c] = v.round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        assert_eq!(nouveau, ancien, "σ = {sigma}, {} étage(s)", plan.etages);
        assert_ne!(nouveau, net, "le mélange a bien changé l'image");
    }
}

/// **L'agrandissement rend au bit près la lecture pixel par pixel** (fiche 140 § 5) : ses pas,
/// calculés une fois par colonne et par rangée, donnent les nombres mêmes de [`lire`] sous le centre
/// de chaque pixel — l'ancienne loi, rejouée sur des tailles impaires, les bords compris.
#[test]
fn test_flou_agrandir_rend_au_bit_pres_la_lecture_pixel_par_pixel() {
    let (l, h) = (19usize, 11usize);
    let octets: Vec<u8> = (0..l * h * 4).map(|i| (i * 53 % 253) as u8).collect();
    let img = Image::d_octets(&octets, l, h);
    let (gl, gh) = (2 * l - 1, 2 * h);
    let ancien: Vec<Pixel> = (0..gh)
        .flat_map(|y| (0..gl).map(move |x| (x, y)))
        .map(|(x, y)| lire(&img, (sous(x), sous(y))))
        .collect();
    assert_eq!(agrandir(&img, (gl, gh)).pixels, ancien);
}

/// **La réduction rend au bit près la lecture voisin par voisin** (fiche 140 § 5) : ses rangées
/// lues d'un coup et ses fenêtres calculées une fois donnent les nombres mêmes que l'ancienne loi —
/// quatre voisins bornés lus un à un, le filtre binomial —, sur des tailles impaires et paires, des
/// octets comme d'une image.
#[test]
fn test_flou_reduire_rend_au_bit_pres_la_lecture_voisin_par_voisin() {
    let ancienne = |(l, h): (usize, usize), a: &dyn Fn(isize, isize) -> Pixel| -> Vec<Pixel> {
        let pese = |p: [Pixel; 4]| -> Pixel {
            std::array::from_fn(|c| (p[0][c] + 3.0 * p[1][c] + 3.0 * p[2][c] + p[3][c]) / 8.0)
        };
        let (pl, ph) = (moitie(l) as isize, moitie(h) as isize);
        let borne = |v: isize, n: usize| v.clamp(0, n as isize - 1);
        let ligne = |i: isize, y: isize| {
            let x = 2 * i;
            pese([a(x - 1, y), a(x, y), a(x + 1, y), a(x + 2, y)])
        };
        let mut sortie = Vec::new();
        for j in 0..ph {
            for i in 0..pl {
                let y = 2 * j;
                let r = [y - 1, y, y + 1, y + 2].map(|v| ligne(i, borne(v, h)));
                sortie.push(pese(r));
            }
        }
        sortie
    };
    for (l, h) in [(17usize, 9usize), (16, 10), (1, 1)] {
        let octets: Vec<u8> = (0..l * h * 4).map(|i| (i * 71 % 251) as u8).collect();
        let lu = |x: isize, y: isize| {
            let (x, y) = (
                x.clamp(0, l as isize - 1) as usize,
                y.clamp(0, h as isize - 1) as usize,
            );
            let i = (y * l + x) * 4;
            [0, 1, 2, 3].map(|c| f32::from(octets[i + c]))
        };
        let attendu = ancienne((l, h), &lu);
        assert_eq!(
            reduire_des_octets(&octets, (l, h)).pixels,
            attendu,
            "{l} x {h}, octets"
        );
        let img = Image::d_octets(&octets, l, h);
        assert_eq!(reduire(&img).pixels, attendu, "{l} x {h}, image");
    }
}

/// **`octet` rend `v.round() as u8`, au bit près** (fiche 140 § 5) : à chaque entier et chaque demi
/// de −4 à 300, et à un ulp de part et d'autre — là où une troncature mal arrondie se tromperait —,
/// tous les 1/1024 entre eux, puis l'infini, NaN, et les très grands.
#[test]
fn test_flou_octet_arrondit_comme_round() {
    let mut valeurs = vec![
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        1e9,
        -1e9,
        0.0,
        -0.0,
    ];
    for k in -4 * 1024..=300 * 1024 {
        let v = k as f32 / 1024.0;
        valeurs.extend([v, v.next_up(), v.next_down()]);
    }
    for v in valeurs {
        assert_eq!(octet(v), v.round() as u8, "v = {v:?}");
    }
}
