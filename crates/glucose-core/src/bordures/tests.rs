//! Ce que la détection promet : elle trouve les bandes unies, de n'importe quelle couleur,
//! malgré le bruit, un filigrane ou un filet au bord — et elle ne trouve rien là où il n'y a
//! rien.
//!
//! **Chaque cas se vérifie sur ses quatre bords.** Les premières versions ne regardaient que
//! celui dont elles parlaient, et cela cachait un défaut (BORDURES-5) : la « peinture » de
//! BORDURES-3, censée n'être jamais unie, ne variait que de ±20 niveaux — sous le bruit. Ses
//! colonnes passaient pour des bandes, le bord gauche la dévorait entière, et l'image finissait
//! en un trait d'un centième de large pendant que l'épreuve, qui ne lisait que le haut, passait.

use super::*;

/// Une image synthétique : ses pixels, sa largeur, sa hauteur.
struct Cas {
    pixels: Vec<Pixel>,
    l: u32,
    h: u32,
}

/// Ce qui fabrique un cas : l'épreuve du filet les rejoue tous.
type Fabrique = fn() -> Cas;

/// Une image `l × h` peinte par une fonction du pixel.
fn image(l: u32, h: u32, f: impl Fn(u32, u32) -> Pixel) -> Cas {
    let pixels = (0..h)
        .flat_map(|y| (0..l).map(move |x| (x, y)))
        .map(|(x, y)| f(x, y))
        .collect();
    Cas { pixels, l, h }
}

/// Le recadrage détecté, en pixels retirés — gauche, haut, droite, bas —, plus lisible qu'en
/// fractions.
fn en_pixels(cas: &Cas) -> (u32, u32, u32, u32) {
    let vue = Vue::nouvelle(&cas.pixels, cas.l, cas.h).expect("une vue");
    let (g, ht, d, b) = detecter(&vue).marges();
    let arrondi = |part: f64, dim: u32| (part * f64::from(dim)).round() as u32;
    (
        arrondi(g, cas.l),
        arrondi(ht, cas.h),
        arrondi(d, cas.l),
        arrondi(b, cas.h),
    )
}

const NOIR: Pixel = [0, 0, 0, 255];
const BLANC: Pixel = [255, 255, 255, 255];
const GRIS: Pixel = [120, 120, 120, 255];
/// Le filet de son image du 24/09, mesuré sur sa dernière colonne.
const SON_FILET: Pixel = [88, 88, 88, 255];

/// Un contenu qui n'est jamais uni : un damier de deux teintes franches.
fn contenu(x: u32, y: u32) -> Pixel {
    if (x / 4 + y / 4).is_multiple_of(2) {
        [200, 60, 30, 255]
    } else {
        [30, 90, 200, 255]
    }
}

/// Un contenu texturé comme une peinture, autour d'un brun-vert, et **jamais uni** : chaque
/// rang et chaque colonne parcourent quatre-vingt-trois niveaux, bien au-delà du bruit.
fn peinture(x: u32, y: u32) -> Pixel {
    let v = ((x * 37 + y * 11) % 83) as u8;
    [100 + v, 95 + v / 2, 45 + v / 3, 255]
}

/// Le mélange `a · bande + (1 − a) · pixel`, canal par canal : ce que l'anticrénelage et la
/// compression font du rang qui touche la bande.
fn fondu(bande: Pixel, p: Pixel, a: f64) -> Pixel {
    let m = |c: usize| (a * f64::from(bande[c]) + (1.0 - a) * f64::from(p[c])).round() as u8;
    [m(0), m(1), m(2), 255]
}

// ── Les images ──────────────────────────────────────────────────────────────────────

fn boite_aux_lettres() -> Cas {
    image(160, 90, |x, y| {
        if !(12..78).contains(&y) {
            NOIR
        } else {
            contenu(x, y)
        }
    })
}

fn bande_blanche() -> Cas {
    image(120, 80, |x, y| {
        if !(20..112).contains(&x) {
            BLANC
        } else {
            contenu(x, y)
        }
    })
}

fn quatre_couleurs() -> Cas {
    image(100, 100, |x, y| match (x, y) {
        (_, 0..5) => NOIR,
        (_, 93..) => BLANC,
        (0..9, _) => GRIS,
        (97.., _) => [10, 200, 10, 255],
        _ => contenu(x, y),
    })
}

fn sans_bordure() -> Cas {
    image(64, 48, contenu)
}

/// Un bruit déterministe, entre -10 et +10 par canal, sur un noir vidéo.
fn bruit_de_compression() -> Cas {
    let bruit = |x: u32, y: u32, c: u32| ((x * 7 + y * 13 + c * 29) % 21) as i32 - 10;
    image(200, 120, move |x, y| {
        if !(15..105).contains(&y) {
            let v = |c: u32| (12 + bruit(x, y, c)).clamp(0, 255) as u8;
            [v(0), v(1), v(2), 255]
        } else {
            contenu(x, y)
        }
    })
}

/// Une bande noire dont le logo blanc occupe les lignes `logo`.
fn filigrane(logo: std::ops::Range<u32>) -> Cas {
    image(200, 100, move |x, y| match y {
        0..10 if x < 30 && logo.contains(&y) => BLANC,
        0..10 => NOIR,
        _ => contenu(x, y),
    })
}

/// Vingt lignes de marge, puis du texte : sur la ligne 20, un pixel sur trente-trois est de
/// l'encre ; sur les suivantes, davantage.
fn texte_fin() -> Cas {
    image(300, 100, |x, y| {
        if y >= 20 && ((x + y) % 33 == 0 || (y > 22 && x % 5 == 0)) {
            NOIR
        } else {
            BLANC
        }
    })
}

fn ciel_uni() -> Cas {
    image(100, 100, |x, y| {
        if y < 40 {
            [135, 206, 235, 255]
        } else {
            contenu(x, y)
        }
    })
}

/// Six rangs de bande blanche, deux rangs de transition — `0,87 · blanc` puis `0,21 · blanc`
/// sur le premier rang de peinture —, puis la peinture.
fn rangs_de_transition() -> Cas {
    image(200, 60, |x, y| match y {
        0..6 => BLANC,
        6 => fondu(BLANC, peinture(x, 8), 0.87),
        7 => fondu(BLANC, peinture(x, 8), 0.21),
        _ => peinture(x, y),
    })
}

/// Une peinture qui s'éclaircit doucement vers son bord du bas, un rang de transition, puis
/// sept rangs de bande.
fn degrade_du_contenu() -> Cas {
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
    image(200, 60, move |x, y| match y {
        0..52 => degrade(x, y),
        52 => fondu(BLANC, degrade(x, 51), 0.54),
        _ => BLANC,
    })
}

/// Vingt rangs qui glissent vers le blanc par pas de un vingt-et-unième, puis dix de bande.
fn fondu_doux_rang(x: u32, y: u32) -> Pixel {
    match y {
        0..50 => peinture(x, y),
        50..70 => fondu(BLANC, peinture(x, 49), f64::from(y - 49) / 21.0),
        _ => BLANC,
    }
}

fn fondu_doux() -> Cas {
    image(200, 80, fondu_doux_rang)
}

/// Huit rangs blancs, un rang à 0,94 de blanc dont un vingtième déborde à 0,5, un rang à 0,3
/// dont le même vingtième est déjà de la peinture, puis la peinture.
fn bord_qui_ondule() -> Cas {
    let deborde = |x: u32| (10..20).contains(&x);
    image(200, 60, move |x, y| match y {
        0..8 => BLANC,
        8 if deborde(x) => fondu(BLANC, peinture(x, 10), 0.5),
        8 => fondu(BLANC, peinture(x, 10), 0.94),
        9 if deborde(x) => peinture(x, 10),
        9 => fondu(BLANC, peinture(x, 10), 0.3),
        _ => peinture(x, y),
    })
}

/// Un objet étroit au sommet flou sur trois rangs, et un objet large plus bas, sur un fond de
/// la couleur de la marge.
fn pointe_d_un_objet() -> Cas {
    let sombre = [30, 30, 30, 255];
    let pointe = |x: u32| (90..110).contains(&x);
    image(200, 80, move |x, y| match y {
        20 if pointe(x) => fondu(BLANC, sombre, 0.75),
        21 if pointe(x) => fondu(BLANC, sombre, 0.5),
        22 if pointe(x) => fondu(BLANC, sombre, 0.25),
        23..50 if pointe(x) => sombre,
        50..70 if (10..190).contains(&x) => sombre,
        _ => BLANC,
    })
}

/// Une peinture sombre en bandes obliques — sur chaque rang, quatre colonnes sur dix presque
/// noires, deux à quarante, quatre à quatre-vingts —, sous huit rangs de bande noire et un
/// rang qui en garde 58 %.
fn peinture_sombre() -> Cas {
    let sombre = |x: u32, y: u32| {
        let v = match (x + y + 6) % 10 {
            0..4 => 15,
            4..6 => 40,
            _ => 80,
        };
        [v, v, v, 255]
    };
    image(200, 60, move |x, y| match y {
        0..8 => NOIR,
        8 => fondu(NOIR, sombre(x, 9), 0.42),
        _ => sombre(x, y),
    })
}

/// **Son image du 24/09**, réduite : une marge blanche autour d'un contenu, et un filet gris
/// d'un pixel sur toute la hauteur du bord droit.
fn son_filet() -> Cas {
    let (l, h) = (212, 300);
    image(l, h, move |x, y| {
        if x == l - 1 {
            SON_FILET
        } else if (24..l - 28).contains(&x) && (26..h - 23).contains(&y) {
            contenu(x, y)
        } else {
            BLANC
        }
    })
}

/// **Sa gravure du 24/09**, réduite : une plaque au bord droit légèrement penché, sur une
/// marge de papier blanc, et deux taches sur la marge de droite — une de cinq colonnes sur huit
/// lignes, une de deux sur quatre —, chacune au-delà de la tolérance d'une colonne.
fn gravure_tachee() -> Cas {
    let (l, h) = (300, 260);
    image(l, h, move |x, y| {
        // Le bord droit de la plaque penche de trois pixels sur sa hauteur, comme le sien :
        // la colonne 250 est la premiere entierement blanche.
        let bord_droit = 247 + (y.saturating_sub(30)) * 4 / 200;
        let tache = ((280..285).contains(&x) && (100..108).contains(&y))
            || ((265..267).contains(&x) && (150..154).contains(&y));
        if (40..bord_droit).contains(&x) && (30..230).contains(&y) {
            contenu(x, y)
        } else if tache {
            [215, 215, 215, 255]
        } else {
            BLANC
        }
    })
}

/// Une page : une marge, un titre en lettres fines, un blanc, puis une image à bord net.
fn titre_au_dessus() -> Cas {
    image(300, 300, |x, y| match y {
        20..30 if (60..240).contains(&x) && x % 5 == 0 => NOIR,
        60..280 if (30..270).contains(&x) => contenu(x, y),
        _ => BLANC,
    })
}

/// Une page : une marge, un objet qui s'élargit rang après rang — la pointe d'un triangle —,
/// un blanc, puis une image à bord net.
fn objet_au_dessus() -> Cas {
    image(300, 300, |x, y| match y {
        20..30 if x.abs_diff(150) < 3 * (y - 19) => NOIR,
        60..280 if (30..270).contains(&x) => contenu(x, y),
        _ => BLANC,
    })
}

/// Une marge, une petite tache, la marge, puis un contenu qui commence en hésitant — un rang
/// d'encre, un rang deux fois plus clairsemé, puis l'image.
fn bord_qui_hesite() -> Cas {
    image(300, 300, |x, y| match y {
        25 if (100..105).contains(&x) => [215, 215, 215, 255],
        40 if (30..270).contains(&x) && x % 5 == 0 => NOIR,
        41 if (30..270).contains(&x) && x % 10 == 0 => NOIR,
        42..280 if (30..270).contains(&x) => contenu(x, y),
        _ => BLANC,
    })
}

/// Le même cas entouré d'un filet de cette couleur sur les bords choisis — gauche, haut,
/// droite, bas.
fn avec_un_filet(cas: &Cas, bords: [bool; 4], couleur: Pixel) -> Cas {
    let [g, ht, d, b] = bords.map(u32::from);
    let (l, h) = (cas.l + g + d, cas.h + ht + b);
    image(l, h, |x, y| {
        let dedans = (g..l - d).contains(&x) && (ht..h - b).contains(&y);
        if dedans {
            cas.pixels[((y - ht) * cas.l + (x - g)) as usize]
        } else {
            couleur
        }
    })
}

// ── Ce que chaque image donne ──────────────────────────────────────────────────────────

/// **Une boîte aux lettres noire se trouve exactement**, en haut et en bas.
#[test]
fn test_une_boite_aux_lettres_noire_se_trouve_exactement() {
    assert_eq!(en_pixels(&boite_aux_lettres()), (0, 12, 0, 12));
}

/// **Une bande blanche se trouve comme une noire** : ce qu'on mesure est l'écart à la couleur
/// du bord, pas la clarté.
#[test]
fn test_une_bande_blanche_se_trouve_comme_une_noire() {
    assert_eq!(en_pixels(&bande_blanche()), (20, 0, 8, 0));
}

/// **Des bandes de quatre couleurs différentes se trouvent chacune** : chaque bord a sa
/// propre référence.
#[test]
fn test_chaque_bord_a_sa_propre_couleur() {
    assert_eq!(en_pixels(&quatre_couleurs()), (9, 5, 3, 7));
}

/// **Une image sans bordure ne perd rien.**
#[test]
fn test_une_image_sans_bordure_ne_perd_rien() {
    assert_eq!(en_pixels(&sans_bordure()), (0, 0, 0, 0));
}

/// **Le bruit de compression ne casse pas une bande.**
///
/// Un JPEG ne rend jamais un noir parfaitement noir : chaque pixel s'écarte de quelques
/// niveaux. Le seuil vient de là, et il doit tenir sur un bruit de dix niveaux, ce qui est
/// plus que ce qu'une compression ordinaire produit sur une zone plate.
#[test]
fn test_le_bruit_de_compression_ne_casse_pas_une_bande() {
    assert_eq!(en_pixels(&bruit_de_compression()), (0, 15, 0, 15));
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
    assert_eq!(
        en_pixels(&filigrane(0..6)),
        (0, 0, 0, 0),
        "le logo occupe la premiere ligne : la bande du haut s'arrete la"
    );
    assert_eq!(
        en_pixels(&filigrane(4..8)),
        (0, 4, 0, 0),
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
    assert_eq!(
        en_pixels(&texte_fin()),
        (0, 20, 0, 0),
        "la marge se retire, la premiere ligne d'encre reste"
    );
}

/// **Une image entièrement unie n'a pas de bordure** (BORDURES-5) : il n'y a rien autour de
/// quoi en trouver une.
///
/// Avant, la détection proposait de tout retirer et l'invariant du recadrage en gardait un
/// centième — une nuance unie devenait un trait.
#[test]
fn test_une_image_unie_n_a_pas_de_bordure() {
    let cas = image(50, 50, |_, _| GRIS);
    let vue = Vue::nouvelle(&cas.pixels, cas.l, cas.h).expect("une vue");
    assert!(detecter(&vue).est_entier());
}

/// **Un contenu qui commence par une zone unie est rogné**, et ce test le dit franchement.
///
/// Un ciel sans nuage en haut d'une photo se retire comme une bande, parce que ce sont les
/// mêmes pixels et que seule l'intention les sépare. FFmpeg a le même défaut. C'est pourquoi
/// l'action est explicite et annulable, jamais automatique — et pourquoi ce cas est écrit ici
/// plutôt que découvert par l'utilisateur.
#[test]
fn test_un_ciel_uni_se_fait_rogner_et_c_est_assume() {
    assert_eq!(
        en_pixels(&ciel_uni()),
        (0, 40, 0, 0),
        "le ciel est pris pour une bande : c'est connu et annulable"
    );
}

// ── BORDURES-3 : les rangées de transition entre la bande et le contenu ─────────────

/// **Le liseré que l'utilisateur voit après `Ctrl+B`**, reconstitué depuis sa capture du 23/09.
///
/// Mesuré au bord du haut, de l'extérieur vers l'intérieur : le fond du canevas, puis
/// `243,239,218` — presque blanc —, `147,140,103` — à mi-chemin —, et l'image `118,111,58`.
/// La bande blanche est partie ; les deux rangs qui la mêlaient à l'image sont restés, et c'est
/// le liseré clair.
#[test]
fn test_les_rangs_de_transition_partent_avec_la_bande() {
    assert_eq!(
        en_pixels(&rangs_de_transition()),
        (0, 8, 0, 0),
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
    assert_eq!(
        en_pixels(&degrade_du_contenu()),
        (0, 0, 0, 8),
        "sept rangs de bande et UN rang de transition -- le dégradé reste entier"
    );
}

/// **Un fondu doux vers la bande ne se mange pas rang après rang.**
///
/// Le cas inverse, sans lequel les deux précédents ne prouveraient rien : une règle qui
/// retirerait tout rang « un peu plus proche de la bande » dévorerait un vignettage entier.
/// Aucun rang du fondu n'est une transition, et seuls ceux qui sont déjà de la bande partent.
#[test]
fn test_un_fondu_doux_vers_la_bande_ne_se_mange_pas() {
    let cas = fondu_doux();
    // Ce que la bande seule retire : les rangs du fondu qui sont déjà de la bande blanche.
    let proches = (50..70)
        .rev()
        .take_while(|&y| {
            let rang: Vec<Pixel> = (0..cas.l).map(|x| fondu_doux_rang(x, y)).collect();
            est_de_la_bande(&rang, BLANC)
        })
        .count() as u32;
    assert!(
        proches < 20,
        "le fondu ne doit pas etre tout entier de la bande"
    );
    assert_eq!(en_pixels(&cas), (0, 0, 0, 10 + proches));
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
#[test]
fn test_le_fondu_d_un_bord_qui_ondule_part_avec_la_bande() {
    assert_eq!(
        en_pixels(&bord_qui_ondule()),
        (0, 10, 0, 0),
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
    assert_eq!(
        en_pixels(&pointe_d_un_objet()),
        (10, 20, 10, 10),
        "la marge part, le haut flou de l'objet reste"
    );
}

/// **Un fondu dans une peinture sombre part aussi** — la colonne gauche d'une illustration de
/// l'utilisateur, sur bande noire.
///
/// Sa luminosité valait 58 % de celle de sa voisine, uniformément : un fondu, sans doute.
/// Mais un tiers de ses pixels étaient déjà presque noirs, et là un fondu vers le noir ne se
/// voit pas. Jugée sur tous les pixels, la médiane du déplacement tombait sous le bruit et la
/// colonne restait, en liseré sombre. Elle se juge sur ceux qui **peuvent** montrer un fondu :
/// parmi tous les pixels, seuls quatre sur dix ont bougé au-delà du bruit ; parmi ceux qui le
/// pouvaient, quatre sur six.
#[test]
fn test_un_fondu_dans_une_peinture_sombre_part_aussi() {
    assert_eq!(
        en_pixels(&peinture_sombre()),
        (0, 9, 0, 0),
        "la bande noire et son rang de fondu"
    );
}

// ── BORDURES-5 : un filet au bord ─────────────────────────────────────────────────────

/// **Un filet au bord ne cache pas la marge** — son image du 24/09.
///
/// Sa dernière colonne est un filet gris, et la marge blanche commence derrière. Le bord droit
/// prenait le gris pour sa bande, retirait un pixel, et s'arrêtait sur le blanc : toute la
/// marge de droite restait, quand les trois autres côtés étaient parfaits.
#[test]
fn test_un_filet_au_bord_ne_cache_pas_la_marge() {
    assert_eq!(
        en_pixels(&son_filet()),
        (24, 26, 28, 23),
        "le filet ET la marge de droite, comme les trois autres côtés"
    );
}

/// **Un filet au bord ne change rien à ce que `Ctrl+B` trouve derrière lui** — une ligne de
/// plus, exactement, sur chaque bord qui en porte un, et pour chaque image de ces épreuves.
///
/// Sur chaque bord seul, puis sur les quatre à la fois : un filet sur les deux côtés d'une
/// ligne courte la fait sortir de la tolérance, et c'est au passage suivant, une fois les côtés
/// retirés, que le bord du haut doit encore le reconnaître. Deux couleurs : le gris de son
/// image, et le blanc, qui se confond avec les bandes blanches de plusieurs cas.
///
/// L'image unie n'y est pas : tout y est bande, filet compris, et rien n'en est retiré.
#[test]
fn test_un_filet_au_bord_ne_change_rien_a_ce_qu_on_trouve_derriere() {
    let cas: [(&str, Fabrique); 19] = [
        ("boite aux lettres", boite_aux_lettres),
        ("bande blanche", bande_blanche),
        ("quatre couleurs", quatre_couleurs),
        ("sans bordure", sans_bordure),
        ("bruit", bruit_de_compression),
        ("filigrane au bord", || filigrane(0..6)),
        ("filigrane dans la bande", || filigrane(4..8)),
        ("texte fin", texte_fin),
        ("ciel uni", ciel_uni),
        ("rangs de transition", rangs_de_transition),
        ("degrade du contenu", degrade_du_contenu),
        ("fondu doux", fondu_doux),
        ("bord qui ondule", bord_qui_ondule),
        ("pointe d'un objet", pointe_d_un_objet),
        ("peinture sombre", peinture_sombre),
        ("gravure tachee", gravure_tachee),
        ("titre au-dessus", titre_au_dessus),
        ("objet au-dessus", objet_au_dessus),
        ("bord qui hesite", bord_qui_hesite),
    ];
    let seuls = (0..4).map(|i| std::array::from_fn(|j| i == j));
    for (nom, fabrique) in cas {
        let sans = en_pixels(&fabrique());
        for bords in seuls.clone().chain([[true; 4]]) {
            for couleur in [SON_FILET, BLANC] {
                let [g, ht, d, b] = bords.map(u32::from);
                assert_eq!(
                    en_pixels(&avec_un_filet(&fabrique(), bords, couleur)),
                    (sans.0 + g, sans.1 + ht, sans.2 + d, sans.3 + b),
                    "{nom}, filet {couleur:?} sur {bords:?}"
                );
            }
        }
    }
}

/// **Un dégradé au bord n'est pas une pile de filets.**
///
/// Le haut d'une photo de dune de l'utilisateur part d'un voile clair qui s'assombrit sur deux
/// lignes avant le ciel. Si des filets pouvaient se suivre, chaque ligne du dégradé en serait
/// un, et le ciel s'ouvrirait comme une bande derrière eux : soixante-quatorze lignes de ciel
/// partaient. Ici, un rang clair, un rang de transition, puis trente-huit rangs de ciel uni :
/// le rang clair et sa transition partent, le ciel reste.
#[test]
fn test_un_degrade_au_bord_n_est_pas_une_pile_de_filets() {
    let gris = |v: u8| [v, v, v, 255];
    let cas = image(200, 100, |x, y| match y {
        0 => gris(250),
        1 => gris(200),
        2..40 => gris(150),
        _ => contenu(x, y),
    });
    assert_eq!(en_pixels(&cas), (0, 2, 0, 0), "le ciel reste");
}

/// **Un filet ne se reconnaît que devant une bande.**
///
/// Une ligne unie derrière le premier rang n'en fait pas un filet si rien ne continue après
/// elle : ce serait retirer une ligne de contenu. Ici, le voile de la dune tel qu'il est —
/// `241`, puis `215`, puis un rang dont la moitié des pixels s'écartent de `215` au-delà du
/// bruit, et le ciel. Le premier rang part seul.
#[test]
fn test_un_filet_ne_se_reconnait_que_devant_une_bande() {
    let gris = |v: u8| [v, v, v, 255];
    let cas = image(200, 100, |x, y| match y {
        0 => gris(241),
        1 => gris(215),
        2 => gris(if x % 2 == 0 { 197 } else { 183 }),
        3..40 => gris(if x % 2 == 0 { 196 } else { 184 }),
        _ => contenu(x, y),
    });
    assert_eq!(en_pixels(&cas), (0, 1, 0, 0), "seul le premier rang part");
}

// ── BORDURES-6 : une tache sur la marge ───────────────────────────────────────────────

/// **Une tache sur la marge ne l'arrête pas** — sa gravure du 24/09.
///
/// Une marge de papier blanc autour d'une plaque, et, à quinze pixels du bord droit, une petite
/// tache invisible à l'œil : vingt-cinq à quarante niveaux sous le blanc. Chaque colonne qui la
/// traverse en porte un peu plus d'un centième, et le bord droit s'arrêtait là — onze pixels
/// retirés sur cinquante-six. La tache s'enjambe : la marge reprend derrière elle, tout ce
/// qu'on retire reste sous le centième, et la marge finit sur le bord net de la plaque, penché
/// comme le sien.
#[test]
fn test_une_tache_sur_la_marge_ne_l_arrete_pas() {
    assert_eq!(
        en_pixels(&gravure_tachee()),
        (40, 30, 50, 30),
        "toute la marge de droite, jusqu'au bord de la plaque"
    );
}

/// **Un titre au-dessus d'une image ne part pas.**
///
/// La marge reprend derrière lui et s'arrête sur le bord net d'une image : c'est exactement la
/// forme d'une tache. Mais ses lettres pèsent un pixel sur cinq de ses rangs, bien plus que le
/// centième de ce qu'on retirerait — c'est un contenu, et la marge s'arrête sur lui.
#[test]
fn test_un_titre_au_dessus_d_une_image_ne_part_pas() {
    assert_eq!(
        en_pixels(&titre_au_dessus()),
        (30, 20, 30, 20),
        "la marge part, le titre reste"
    );
}

/// **Un objet au-dessus d'une image ne part pas, même s'il s'élargit régulièrement.**
///
/// Rang après rang, il porte plus d'encre que le précédent : le balayage ne peut pas l'écarter
/// en chemin, puisqu'il pourrait être la rampe d'un bord net. C'est la reprise de la marge,
/// derrière lui, qui le juge — et il pèse bien plus que le centième de ce qu'on retirerait.
#[test]
fn test_un_objet_au_dessus_d_une_image_ne_part_pas() {
    assert_eq!(
        en_pixels(&objet_au_dessus()),
        (30, 20, 30, 20),
        "la marge part, l'objet reste"
    );
}

/// **Une tache devant un bord qui hésite reste.**
///
/// Ce qui séparait, sur ses 286 images, les deux vrais cas des quarante-six faux : le bord d'une
/// peinture qui s'éclaircit, d'une aquarelle, d'une zone sombre parsemée de détails, monte et
/// descend avant de devenir franc. Un bord droit ne fait que monter. Ici, un rang d'encre puis
/// un rang deux fois plus clairsemé : ce n'est pas un bord net, et la tache n'est pas enjambée.
#[test]
fn test_une_tache_devant_un_bord_qui_hesite_reste() {
    assert_eq!(
        en_pixels(&bord_qui_hesite()),
        (30, 25, 30, 20),
        "la marge s'arrete sur la tache"
    );
}
