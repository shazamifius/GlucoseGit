//! Ce que la détection promet : elle trouve les bandes unies, de n'importe quelle couleur,
//! malgré le bruit et un filigrane — et elle ne trouve rien là où il n'y a rien.

use super::*;

/// Une image `l × h` peinte par une fonction du pixel.
fn image(l: u32, h: u32, f: impl Fn(u32, u32) -> Pixel) -> Vec<Pixel> {
    (0..h)
        .flat_map(|y| (0..l).map(move |x| (x, y)))
        .map(|(x, y)| f(x, y))
        .collect()
}

/// Le recadrage détecté, en pixels retirés de chaque bord — plus lisible qu'en fractions.
fn en_pixels(pixels: &[Pixel], l: u32, h: u32) -> (u32, u32, u32, u32) {
    let vue = Vue::nouvelle(pixels, l, h).expect("une vue");
    let (g, ht, d, b) = detecter(&vue).marges();
    let arrondi = |part: f64, dim: u32| (part * f64::from(dim)).round() as u32;
    (arrondi(g, l), arrondi(ht, h), arrondi(d, l), arrondi(b, h))
}

const NOIR: Pixel = [0, 0, 0, 255];
const BLANC: Pixel = [255, 255, 255, 255];
const GRIS: Pixel = [120, 120, 120, 255];

/// Un contenu qui n'est jamais uni : un damier de deux teintes franches.
fn contenu(x: u32, y: u32) -> Pixel {
    if (x / 4 + y / 4).is_multiple_of(2) {
        [200, 60, 30, 255]
    } else {
        [30, 90, 200, 255]
    }
}

/// **Une boîte aux lettres noire se trouve exactement**, en haut et en bas.
#[test]
fn test_une_boite_aux_lettres_noire_se_trouve_exactement() {
    let (l, h) = (160, 90);
    let img = image(l, h, |x, y| {
        if y < 12 || y >= h - 12 {
            NOIR
        } else {
            contenu(x, y)
        }
    });
    assert_eq!(en_pixels(&img, l, h), (0, 12, 0, 12));
}

/// **Une bande blanche se trouve comme une noire** : ce qu'on mesure est l'écart à la couleur
/// du bord, pas la clarté.
#[test]
fn test_une_bande_blanche_se_trouve_comme_une_noire() {
    let (l, h) = (120, 80);
    let img = image(l, h, |x, y| {
        if x < 20 || x >= l - 8 {
            BLANC
        } else {
            contenu(x, y)
        }
    });
    assert_eq!(en_pixels(&img, l, h), (20, 0, 8, 0));
}

/// **Des bandes de quatre couleurs différentes se trouvent chacune** : chaque bord a sa
/// propre référence.
#[test]
fn test_chaque_bord_a_sa_propre_couleur() {
    let (l, h) = (100, 100);
    let img = image(l, h, |x, y| {
        if y < 5 {
            NOIR
        } else if y >= h - 7 {
            BLANC
        } else if x < 9 {
            GRIS
        } else if x >= l - 3 {
            [10, 200, 10, 255]
        } else {
            contenu(x, y)
        }
    });
    assert_eq!(en_pixels(&img, l, h), (9, 5, 3, 7));
}

/// **Une image sans bordure ne perd rien.**
#[test]
fn test_une_image_sans_bordure_ne_perd_rien() {
    let (l, h) = (64, 48);
    let img = image(l, h, contenu);
    let vue = Vue::nouvelle(&img, l, h).expect("une vue");
    assert!(detecter(&vue).est_entier());
}

/// **Le bruit de compression ne casse pas une bande.**
///
/// Un JPEG ne rend jamais un noir parfaitement noir : chaque pixel s'écarte de quelques
/// niveaux. Le seuil vient de là, et il doit tenir sur un bruit de dix niveaux, ce qui est
/// plus que ce qu'une compression ordinaire produit sur une zone plate.
#[test]
fn test_le_bruit_de_compression_ne_casse_pas_une_bande() {
    let (l, h) = (200, 120);
    // Un bruit déterministe, entre -10 et +10 par canal.
    let bruit = |x: u32, y: u32, c: u32| ((x * 7 + y * 13 + c * 29) % 21) as i32 - 10;
    let img = image(l, h, |x, y| {
        if y < 15 || y >= h - 15 {
            let v = |c: u32| (12 + bruit(x, y, c)).clamp(0, 255) as u8;
            [v(0), v(1), v(2), 255]
        } else {
            contenu(x, y)
        }
    });
    assert_eq!(en_pixels(&img, l, h), (0, 15, 0, 15));
}

/// **Un filigrane posé dans la bande l'arrête à sa ligne**, et c'est assumé.
///
/// La première version de ce test exigeait l'inverse — que la bande soit trouvée malgré le
/// logo — et le critère qu'il a fallu pour y arriver, la médiane des écarts, prenait un damier
/// entier pour une bande : la moitié des pixels d'une ligne de texte sont du fond. On a choisi
/// le défaut qui ne se voit pas. Ici, les six lignes qui portent le logo restent ; les quatre
/// du dessous, unies, sont retirées quand même.
#[test]
fn test_un_filigrane_dans_la_bande_l_arrete_a_sa_ligne() {
    let (l, h) = (200, 100);
    let img = image(l, h, |x, y| {
        if y < 10 {
            if x < 30 && y < 6 {
                BLANC
            } else {
                NOIR
            }
        } else {
            contenu(x, y)
        }
    });
    let (_, haut, _, _) = en_pixels(&img, l, h);
    assert_eq!(
        haut, 0,
        "le logo occupe la premiere ligne : la bande du haut s'arrete la"
    );
    // Mais une bande dont le logo n'est pas au bord se retire jusqu'au logo.
    let img = image(l, h, |x, y| {
        if y < 10 {
            if x < 30 && (4..8).contains(&y) {
                BLANC
            } else {
                NOIR
            }
        } else {
            contenu(x, y)
        }
    });
    let (_, haut, _, _) = en_pixels(&img, l, h);
    assert_eq!(
        haut, 4,
        "les quatre lignes unies au-dessus du logo se retirent"
    );
}

/// **Du texte fin sur un fond uni ne se mange pas**, et c'est le cas qui a décidé du critère.
///
/// Une capture d'écran de code : une marge blanche, puis des lettres dont la première ligne de
/// pixels ne porte que trois pour cent d'encre. La moyenne de cette ligne reste sous le seuil
/// et la moyenne la mange ; la médiane la mange aussi, puisque plus de la moitié de ses pixels
/// sont du fond. Seul le pire pixel la voit. La marge, elle, se retire exactement.
#[test]
fn test_du_texte_fin_sur_un_fond_uni_ne_se_mange_pas() {
    let (l, h) = (300, 100);
    let img = image(l, h, |x, y| {
        // Vingt lignes de marge, puis du texte : sur la ligne 20, un pixel sur trente-trois
        // est de l'encre ; sur les suivantes, davantage.
        if y < 20 {
            BLANC
        } else if (x + y) % 33 == 0 || (y > 22 && x % 5 == 0) {
            NOIR
        } else {
            BLANC
        }
    });
    let (_, haut, _, _) = en_pixels(&img, l, h);
    assert_eq!(
        haut, 20,
        "la marge se retire, la premiere ligne d'encre reste"
    );
}

/// **Une image entièrement unie ne disparaît pas** : c'est l'invariant du recadrage qui la
/// sauve, et ce test vérifie que la détection le laisse faire au lieu de le contourner.
#[test]
fn test_une_image_unie_ne_disparait_pas() {
    let (l, h) = (50, 50);
    let img = image(l, h, |_, _| GRIS);
    let vue = Vue::nouvelle(&img, l, h).expect("une vue");
    let r = detecter(&vue);
    assert!(r.largeur_visible() > 0.0 && r.hauteur_visible() > 0.0);
}

/// **Un contenu qui commence par une zone unie est rogné**, et ce test le dit franchement.
///
/// Un ciel sans nuage en haut d'une photo se retire comme une bande, parce que ce sont les
/// mêmes pixels et que seule l'intention les sépare. FFmpeg a le même défaut. C'est pourquoi
/// l'action est explicite et annulable, jamais automatique — et pourquoi ce cas est écrit ici
/// plutôt que découvert par l'utilisateur.
#[test]
fn test_un_ciel_uni_se_fait_rogner_et_c_est_assume() {
    let (l, h) = (100, 100);
    let ciel = [135, 206, 235, 255];
    let img = image(l, h, |x, y| if y < 40 { ciel } else { contenu(x, y) });
    let (_, haut, _, _) = en_pixels(&img, l, h);
    assert_eq!(
        haut, 40,
        "le ciel est pris pour une bande : c'est connu et annulable"
    );
}

// ── BORDURES-3 : les rangées de transition entre la bande et le contenu ─────────────

/// Un contenu texturé comme une peinture : jamais uni, autour d'un brun-vert.
fn peinture(x: u32, y: u32) -> Pixel {
    let v = ((x * 37 + y * 11) % 41) as u8;
    [100 + v, 95 + v / 2, 45 + v / 3, 255]
}

/// Le mélange `a · bande + (1 − a) · pixel`, canal par canal : ce que l'anticrénelage et la
/// compression font du rang qui touche la bande.
fn fondu(bande: Pixel, p: Pixel, a: f64) -> Pixel {
    let m = |c: usize| (a * f64::from(bande[c]) + (1.0 - a) * f64::from(p[c])).round() as u8;
    [m(0), m(1), m(2), 255]
}

/// **Le liseré que l'utilisateur voit après `Ctrl+B`**, reconstitué depuis sa capture du 23/09.
///
/// Mesuré au bord du haut, de l'extérieur vers l'intérieur : le fond du canevas, puis
/// `243,239,218` — presque blanc —, `147,140,103` — à mi-chemin —, et l'image `118,111,58`.
/// La bande blanche est partie ; les deux rangs qui la mêlaient à l'image sont restés, et c'est
/// le liseré clair. Ici : six rangs de bande blanche, puis deux rangs de transition —
/// `0,87 · blanc` puis `0,21 · blanc` sur le premier rang de peinture —, puis la peinture.
#[test]
fn test_les_rangs_de_transition_partent_avec_la_bande() {
    let (l, h) = (200, 60);
    let img = image(l, h, |x, y| match y {
        0..6 => BLANC,
        6 => fondu(BLANC, peinture(x, 8), 0.87),
        7 => fondu(BLANC, peinture(x, 8), 0.21),
        _ => peinture(x, y),
    });
    assert_eq!(
        en_pixels(&img, l, h).1,
        8,
        "la bande ET ses deux rangs de transition, sinon le liseré reste"
    );
}

/// **Un dégradé du contenu n'est pas une transition** — c'est le bord du bas de la même capture.
///
/// Mesuré de l'intérieur vers l'extérieur : `76, 78, 80, 85, 93, 98, 103, 111` — l'image
/// s'éclaircit doucement vers son bord —, puis `186,183,177`, puis la bande. Seul `186` est une
/// transition : les rangs du dégradé ne diffèrent de leur voisin que de quelques niveaux, ce
/// qu'un contenu fait tout le temps.
#[test]
fn test_un_degrade_du_contenu_reste_seul_le_rang_de_transition_part() {
    let (l, h) = (200, 60);
    let degrade = |x: u32, y: u32| {
        let p = peinture(x, y);
        let k = (y.saturating_sub(44) * 5) as u8;
        [
            p[0].saturating_add(k),
            p[1].saturating_add(k),
            p[2].saturating_add(k),
            255,
        ]
    };
    let img = image(l, h, |x, y| match y {
        0..52 => degrade(x, y),
        52 => fondu(BLANC, degrade(x, 51), 0.54),
        _ => BLANC,
    });
    assert_eq!(
        en_pixels(&img, l, h).3,
        8,
        "sept rangs de bande et UN rang de transition -- le dégradé reste entier"
    );
}

/// **Un fondu doux vers la bande ne se mange pas rang après rang.**
///
/// Le cas inverse, sans lequel les deux précédents ne prouveraient rien : une règle qui
/// retirerait tout rang « un peu plus proche de la bande » dévorerait un vignettage entier.
/// Vingt rangs qui glissent vers le blanc par pas de sept niveaux : aucun n'est une transition,
/// et seuls les rangs déjà assez proches du blanc pour être de la bande partent.
#[test]
fn test_un_fondu_doux_vers_la_bande_ne_se_mange_pas() {
    let (l, h) = (200, 80);
    let img = image(l, h, |x, y| match y {
        0..50 => peinture(x, y),
        50..70 => fondu(BLANC, peinture(x, 49), f64::from(y - 49) / 21.0),
        _ => BLANC,
    });
    // Ce que la bande seule retire : les rangs à moins de vingt-quatre niveaux du blanc.
    let proches = (50..70)
        .rev()
        .take_while(|&y| {
            (0..l).all(|x| {
                let p = fondu(BLANC, peinture(x, 49), f64::from(y - 49) / 21.0);
                (0..3).all(|c| 255 - p[c] <= 24)
            })
        })
        .count() as u32;
    assert_eq!(en_pixels(&img, l, h).3, 10 + proches);
}

// ── BORDURES-4 : un bord qui ondule, et la pointe d'un objet ─────────────────────────

/// **Le bord d'une peinture ondule, et son fondu part quand même** — la forêt de
/// l'utilisateur, `Ctrl+B` du 23/09.
///
/// Mesuré au bas de son image, de la bande vers l'intérieur : un rang blanc à quatre-vingt-
/// quatorze pour cent, dont deux segments de peinture débordent sur la marge (x = 13 à 71 et
/// 765 à 774), puis un rang à mi-chemin où ces mêmes segments sont déjà de la peinture, puis
/// l'image. Aucun des deux n'est de la bande — trop de pixels s'en écartent — et le mélange
/// uniforme de BORDURES-3 n'en expliquait que 94 %. Ils restaient, et c'était le liseré.
///
/// Ici : huit rangs blancs, un rang à 0,94 de blanc dont un vingtième déborde à 0,5, un rang à
/// 0,3 dont le même vingtième est déjà de la peinture, puis la peinture. Les deux partent.
#[test]
fn test_le_fondu_d_un_bord_qui_ondule_part_avec_la_bande() {
    let (l, h) = (200, 60);
    let deborde = |x: u32| (10..20).contains(&x);
    let img = image(l, h, |x, y| match y {
        0..8 => BLANC,
        8 if deborde(x) => fondu(BLANC, peinture(x, 10), 0.5),
        8 => fondu(BLANC, peinture(x, 10), 0.94),
        9 if deborde(x) => peinture(x, 10),
        9 => fondu(BLANC, peinture(x, 10), 0.3),
        _ => peinture(x, y),
    });
    assert_eq!(
        en_pixels(&img, l, h).1,
        10,
        "la bande et ses deux rangs de fondu, malgré les coups de pinceau qui débordent"
    );
}

/// **La pointe d'un objet sur un fond uni ne se mange pas.**
///
/// Une illustration posée sur un fond blanc : le fond et la marge sont le même blanc. Un objet
/// étroit monte plus haut que les autres, et son sommet est flou sur trois rangs ; un objet
/// large, plus bas, fixe les bords gauche et droit — le fond reste donc dans le cadre, autour
/// de la pointe. BORDURES-3 prenait ses trois rangs pour des transitions : son mélange
/// uniforme les expliquait tous, puisque les neuf dixièmes de chaque rang sont du fond sur du
/// fond. Sur les images de l'utilisateur, cela rognait d'un à trois rangs le haut ou le flanc
/// d'un objet. Leur pixel **typique** n'a pas bougé : ce sont du contenu, et ils restent.
///
/// L'objet étroit **seul** ne le prouverait pas, et la première version de ce test l'a appris :
/// une fois les marges de gauche et de droite retirées, son sommet flou occupe toute la largeur
/// qui reste — c'est alors un vrai fondu contre la bande, et il part, à raison.
#[test]
fn test_la_pointe_d_un_objet_sur_un_fond_uni_ne_se_mange_pas() {
    let (l, h) = (200, 80);
    let sombre = [30, 30, 30, 255];
    let pointe = |x: u32| (90..110).contains(&x);
    let img = image(l, h, |x, y| match y {
        20 if pointe(x) => fondu(BLANC, sombre, 0.75),
        21 if pointe(x) => fondu(BLANC, sombre, 0.5),
        22 if pointe(x) => fondu(BLANC, sombre, 0.25),
        23..50 if pointe(x) => sombre,
        50..70 if (10..190).contains(&x) => sombre,
        _ => BLANC,
    });
    assert_eq!(
        en_pixels(&img, l, h).1,
        20,
        "la marge part, le haut flou de l'objet reste"
    );
}

/// **Un fondu dans une peinture sombre part aussi** — la colonne gauche d'une illustration de
/// l'utilisateur, sur bande noire.
///
/// Sa luminosité valait 58 % de celle de sa voisine, uniformément : un fondu, sans doute.
/// Mais un tiers de ses pixels étaient déjà presque noirs, et là un fondu vers le noir ne se
/// voit pas. Jugée sur tous les pixels, la médiane du déplacement tombait sous le bruit et la
/// colonne restait, en liseré sombre. Elle se juge sur ceux qui **peuvent** montrer un fondu.
///
/// Ici, des bandes verticales : quatre colonnes sur dix presque noires, deux à quarante, quatre
/// à quatre-vingts ; le rang de fondu en garde 58 %. Parmi tous les pixels, seuls quatre sur
/// dix ont bougé au-delà du bruit ; parmi ceux qui le pouvaient, quatre sur six.
#[test]
fn test_un_fondu_dans_une_peinture_sombre_part_aussi() {
    let (l, h) = (200, 60);
    let sombre = |x: u32| {
        let v = match (x + 6) % 10 {
            0..4 => 15,
            4..6 => 40,
            _ => 80,
        };
        [v, v, v, 255]
    };
    let img = image(l, h, |x, y| match y {
        0..8 => NOIR,
        8 => fondu(NOIR, sombre(x), 0.42),
        _ => sombre(x),
    });
    assert_eq!(
        en_pixels(&img, l, h).1,
        9,
        "la bande noire et son rang de fondu"
    );
}
