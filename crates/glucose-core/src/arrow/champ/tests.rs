use super::*;

/// Un champ de flèche coudée, avec ses deux disques — de quoi éprouver toutes les couches.
fn coudee(ame_blanche: bool) -> Champ {
    Champ {
        segments: vec![[20.0, 150.0, 160.0, 40.5], [160.0, 40.5, 300.3, 170.0]],
        axe: [20.0, 150.0, 300.3, 170.0],
        teintes: [[0.9, 0.3, 0.2], [0.95, 0.8, 0.4], [0.3, 0.5, 0.95]],
        halo: [3.0, 0.18],
        ame: [1.0, 0.92],
        ame_blanche,
        disques: [
            Some(Disque {
                centre: [20.0, 150.0],
                rayon: 5.0,
                demi_contour: 1.0,
                fond: [0.9, 0.3, 0.2],
                contour: [1.0, 1.0, 1.0],
            }),
            Some(Disque {
                centre: [300.3, 170.0],
                rayon: 3.0,
                demi_contour: 1.0,
                fond: [0.067, 0.067, 0.067],
                contour: [0.3, 0.5, 0.95],
            }),
        ],
    }
}

/// Un fond qui n'est pas uni, pour que la composition se voie.
fn fond(l: u32, h: u32) -> Vec<Pixel> {
    (0..l * h)
        .map(|i| {
            let (x, y) = (i % l, i / l);
            [(x % 97) as u8, (y % 89) as u8, ((x + y) % 50) as u8, 255]
        })
        .collect()
}

/// Des champs qui éprouvent chaque borne : horizontal, vertical, raide, aux coordonnées
/// fractionnaires, plus fin qu'un pixel, un gros disque seul.
fn champs() -> Vec<Champ> {
    let seul = |segments: Vec<[f32; 4]>, halo: f32, ame: f32| Champ {
        segments,
        halo: [halo, 0.18],
        ame: [ame, 0.92],
        disques: [None, None],
        ..coudee(false)
    };
    let mut disque = seul(Vec::new(), 3.0, 1.0);
    disque.disques[0] = Some(Disque {
        centre: [250.4, 100.6],
        rayon: 20.3,
        demi_contour: 1.0,
        fond: [0.2, 0.2, 0.2],
        contour: [0.9, 0.8, 0.1],
    });
    vec![
        coudee(false),
        coudee(true),
        seul(vec![[10.3, 50.5, 200.7, 50.5]], 3.0, 1.0),
        seul(vec![[100.25, 10.0, 100.25, 180.0]], 3.0, 1.0),
        seul(vec![[30.0, 190.0, 45.7, 5.2]], 3.0, 1.0),
        seul(vec![[5.1, 120.3, 310.9, 131.7]], 0.4, 0.3),
        disque,
    ]
}

/// **Le peintre rapide pose exactement la loi, en chaque pixel de l'image** — ceux qu'il visite
/// comme ceux qu'il saute. S'il oubliait un pixel qui porte de l'encre, ou en posait un deux
/// fois, l'image différerait.
#[test]
fn test_fleche_2_le_peintre_pose_la_loi_au_bit_pres() {
    for (rang, champ) in champs().into_iter().enumerate() {
        let (l, h) = (320, 200);
        let mut peinte = fond(l, h);
        assert!(champ.peindre(&mut peinte, l, h));
        let mut attendue = fond(l, h);
        for (i, pixel) in attendue.iter_mut().enumerate() {
            let (x, y) = ((i as u32 % l) as f32 + 0.5, (i as u32 / l) as f32 + 0.5);
            let o = champ.couleur(x, y);
            if o[3] > 0.0 {
                composer(pixel, o);
            }
        }
        let differents = peinte.iter().zip(&attendue).filter(|(a, b)| a != b).count();
        assert_eq!(differents, 0, "champ {rang}");
    }
}

/// **Un coude n'est couvert qu'une fois** : l'opacité du halo, à la même distance du tracé,
/// est la même au coude et au milieu d'un tronçon.
#[test]
fn test_fleche_2_un_coude_n_est_couvert_qu_une_fois() {
    let mut champ = coudee(false);
    champ.disques = [None, None];
    // À 2,5 pixels au-dessus du sommet, et à 2,5 pixels du milieu du premier tronçon.
    let au_coude = champ.couleur(160.0, 38.0)[3];
    let (mx, my) = (90.0_f32, 95.25_f32);
    let (dx, dy) = (140.0_f32, -109.5_f32);
    let n = (dx * dx + dy * dy).sqrt();
    let au_milieu = champ.couleur(mx - dy / n * 2.5, my + dx / n * 2.5)[3];
    assert!(
        (au_coude - au_milieu).abs() < 1e-3,
        "{au_coude} au coude, {au_milieu} au milieu"
    );
}

/// **Rien hors de l'enveloppe** : le champ n'y pose aucune encre.
#[test]
fn test_fleche_2_rien_hors_de_l_enveloppe() {
    let champ = coudee(false);
    let e = champ.enveloppe();
    for (x, y) in [
        (e[0] - 0.6, 100.0),
        (e[2] + 0.6, 100.0),
        (100.0, e[1] - 0.6),
        (100.0, e[3] + 0.6),
    ] {
        assert_eq!(champ.couleur(x, y)[3], 0.0, "({x}, {y})");
    }
}

/// **Découper garde ce qui est dans le cadre, exactement** : un segment qui le traverse est
/// ramené à son bord, un segment dehors disparaît, un segment dedans ne bouge pas.
#[test]
fn test_fleche_2_decouper_au_cadre() {
    let cadre = [0.0, 0.0, 100.0, 100.0];
    let d = decouper(
        &[
            [-100.0, 50.0, 200.0, 50.0],
            [150.0, 150.0, 300.0, 300.0],
            [10.0, 20.0, 30.0, 40.0],
        ],
        cadre,
    );
    assert_eq!(d, vec![[0.0, 50.0, 100.0, 50.0], [10.0, 20.0, 30.0, 40.0]]);
}

/// **Découper rend la précision** : un segment qui part à un million de pixels, découpé,
/// pose son trait au même endroit qu'un segment court sur la même droite.
#[test]
fn test_fleche_2_decouper_rend_la_precision() {
    let loin = decouper(
        &[[-1.0e6, 50.25, 1.0e6, 50.25]],
        [-10.0, -10.0, 110.0, 110.0],
    );
    assert_eq!(loin, vec![[-10.0, 50.25, 110.0, 50.25]]);
}
