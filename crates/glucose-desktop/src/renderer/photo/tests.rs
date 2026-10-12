//! MIP-1 — la pyramide, ses niveaux, et ce qu'elle promet de ne pas faire.

use super::*;

/// Une image unie de la taille demandée.
fn unie(largeur: u32, hauteur: u32, couleur: [u8; 4]) -> Pixmap {
    let mut p = Pixmap::new(largeur, hauteur).expect("une image");
    for bloc in p.data_mut().as_chunks_mut::<4>().0 {
        bloc.copy_from_slice(&couleur);
    }
    p
}

/// Une pyramide naît complète, jusqu'à son niveau de 1 × 1.
///
/// C'est ce qui permet à `niveau_pour` de ne rien fabriquer en pleine frame. La paresse
/// précédente déplaçait le coût dans l'image qui posait la photo : 58 ms mesurés pour une
/// seule photo de téléphone, alors même que le décodage était déjà parti sur un fil de fond.
#[test]
fn test_une_pyramide_nait_complete() {
    let p = Pyramide::nouvelle(unie(4032, 3024, [10, 20, 30, 255]));
    assert_eq!(p.dimensions_natives().0, 4032);
    // 4032 → 2016 → 1008 → 504 → 252 → 126 → 63 → 32 → 16 → 8 → 4 → 2 → 1 : treize niveaux,
    // et le dernier mesure un pixel de large. Rien n'a été choisi : la descente s'arrête où
    // la division par deux ne bouge plus.
    assert_eq!(p.niveaux_construits(), 13);
    assert_eq!(
        p.niveau_pour(1.0)
            .expect("une pyramide neuve tient tous ses niveaux")
            .width(),
        1
    );
}

/// Le coût de la pyramide entière est borné par une somme géométrique, pas par un réglage.
///
/// Chaque réduction divise la surface par quatre : `1 + 1/4 + 1/16 + ...` converge vers `4/3`.
/// Tout garder coûte donc environ **un tiers de plus** que le niveau natif.
///
/// # La réserve, que ce test a imposée
///
/// La borne des 4/3 est **asymptotique**. Les côtés sont divisés en arrondissant vers le
/// haut, et cet arrondi ajoute jusqu'à un pixel par côté et par niveau : sur une image de
/// 7 × 5, il porte le rapport à 1,49 et non 1,33. J'avais écrit « quelle que soit l'image »,
/// et c'était faux — c'est ce test qui l'a dit.
///
/// Ce qui reste vrai, et qui suffit : le surcoût d'arrondi est en `O(largeur + hauteur)`
/// quand la source est en `O(largeur × hauteur)`. Il disparaît donc dès que l'image a une
/// taille réelle, et sur une image minuscule il se compte en dizaines d'octets.
#[test]
fn test_la_pyramide_entiere_coute_un_tiers_de_plus_que_sa_source() {
    for (w, h) in [(4032u32, 3024u32), (1920, 1080), (800, 600), (256, 256)] {
        let p = Pyramide::nouvelle(unie(w, h, [10, 20, 30, 255]));
        let natif = w as f64 * h as f64 * 4.0;
        let rapport = p.octets() as f64 / natif;
        assert!(
            rapport < 1.36,
            "{w}×{h} : la pyramide pèse {rapport:.3} fois sa source (bornée par 4/3)"
        );
    }

    // Une image minuscule dépasse le rapport, et c'est sans conséquence : ce qui compte est
    // le nombre d'octets, pas le rapport.
    let minuscule = Pyramide::nouvelle(unie(7, 5, [10, 20, 30, 255]));
    assert!(
        minuscule.octets() < 7 * 5 * 4 + 128,
        "l'arrondi coûte quelques dizaines d'octets, pas davantage : {}",
        minuscule.octets()
    );
}

/// Le niveau rendu couvre la taille demandée, et le suivant ne la couvrirait plus.
///
/// C'est toute la définition : plus petit que ça, on agrandirait, donc on perdrait du détail
/// que la source avait.
#[test]
fn test_le_niveau_choisi_couvre_la_taille_demandee() {
    let p = Pyramide::nouvelle(unie(4032, 3024, [10, 20, 30, 255]));
    let niveau = p
        .niveau_pour(420.0)
        .expect("une pyramide neuve tient tous ses niveaux");

    assert!(
        niveau.width() >= 420,
        "le niveau doit couvrir la taille posée, or il fait {}",
        niveau.width()
    );
    assert!(
        (niveau.width() as f32) < 840.0,
        "et ne pas être deux fois trop grand, or il fait {}",
        niveau.width()
    );
}

/// Choisir un niveau ne construit jamais rien — quelle que soit la taille demandée, et quel
/// que soit le nombre de fois.
///
/// C'est ce qui rend `niveau_pour` utilisable en `&self`, donc en pleine frame : un zoom
/// continu parcourt toutes les tailles sans qu'aucune n'ajoute de travail.
#[test]
fn test_choisir_un_niveau_ne_construit_jamais_rien() {
    let p = Pyramide::nouvelle(unie(4032, 3024, [10, 20, 30, 255]));
    let a_la_naissance = p.niveaux_construits();
    for largeur in [4032.0, 2000.0, 420.0, 64.0, 3.0, 1.0, 0.5] {
        p.niveau_pour(largeur);
    }
    assert_eq!(p.niveaux_construits(), a_la_naissance);
}

/// Une image unie le reste à tous les niveaux : la moyenne de quatre valeurs égales est cette
/// valeur. Si une réduction décalait ses indices, la couleur baverait.
#[test]
fn test_une_image_unie_le_reste_a_tous_les_niveaux() {
    let couleur = [40, 80, 120, 255];
    let p = Pyramide::nouvelle(unie(64, 64, couleur));
    let niveau = p
        .niveau_pour(3.0)
        .expect("une pyramide neuve tient tous ses niveaux");

    assert!(niveau.width() <= 4);
    for bloc in niveau.data().as_chunks::<4>().0 {
        assert_eq!(bloc, &couleur, "la réduction a changé une couleur unie");
    }
}

/// Une largeur impaire ne perd pas sa dernière colonne et ne déborde pas : la moyenne porte
/// sur les pixels qui existent.
#[test]
fn test_un_cote_impair_se_reduit_sans_deborder() {
    let p = Pyramide::nouvelle(unie(7, 5, [9, 9, 9, 255]));
    let niveau = p
        .niveau_pour(3.0)
        .expect("une pyramide neuve tient tous ses niveaux");
    assert_eq!((niveau.width(), niveau.height()), (4, 3));
    for bloc in niveau.data().as_chunks::<4>().0 {
        assert_eq!(bloc, &[9, 9, 9, 255]);
    }
}

/// La descente s'arrête au pixel, quelle que soit la taille demandée.
#[test]
fn test_la_descente_s_arrete_au_pixel() {
    let p = Pyramide::nouvelle(unie(64, 64, [1, 2, 3, 255]));
    let niveau = p
        .niveau_pour(0.0)
        .expect("une pyramide neuve tient tous ses niveaux");
    assert_eq!(niveau.width(), 1);
}

/// La moyenne est bien une moyenne : deux moitiés opposées donnent le milieu.
#[test]
fn test_la_reduction_moyenne_ses_quatre_pixels() {
    let mut source = Pixmap::new(2, 2).expect("une image");
    let data = source.data_mut();
    data[0..4].copy_from_slice(&[0, 0, 0, 255]);
    data[4..8].copy_from_slice(&[100, 100, 100, 255]);
    data[8..12].copy_from_slice(&[0, 0, 0, 255]);
    data[12..16].copy_from_slice(&[100, 100, 100, 255]);

    let p = Pyramide::nouvelle(source);
    let niveau = p
        .niveau_pour(1.0)
        .expect("une pyramide neuve tient tous ses niveaux");
    assert_eq!((niveau.width(), niveau.height()), (1, 1));
    assert_eq!(&niveau.data()[0..4], &[50, 50, 50, 255]);
}

/// Le coût en mémoire d'une pyramide complète reste sous un tiers de plus que le natif — la
/// somme de 1 + 1/4 + 1/16 + … C'est la contrepartie de MIP-1, et elle doit rester bornée.
#[test]
fn test_la_pyramide_complete_coute_un_tiers_de_plus() {
    let natif = 4032 * 3024 * 4;
    let p = Pyramide::nouvelle(unie(4032, 3024, [0, 0, 0, 255]));
    p.niveau_pour(1.0);

    assert!(
        p.octets() < natif * 3 / 2,
        "la pyramide occupe {} octets pour un natif de {natif}",
        p.octets()
    );
}

/// **PLAFOND-1 — le niveau se réduit jusqu'à tenir dans ce que la carte accepte**, et seulement
/// s'il le faut : sans plafond, ou quand le voulu tient déjà, il ne change pas. Un plafond que
/// rien ne tient donne le dernier niveau, le plus petit qui existe.
#[test]
fn test_plafond_1_le_niveau_se_reduit_jusqu_a_tenir() {
    let p = Pyramide::nouvelle(unie(4032, 3024, [10, 20, 30, 255]));
    assert_eq!(p.tenir_dans(1, None), 1, "sans plafond, rien ne change");
    assert_eq!(p.tenir_dans(1, Some(2048)), 2, "4032 ne tient pas : 2016");
    assert_eq!(p.tenir_dans(4, Some(2048)), 4, "1008 tient déjà");
    assert_eq!(p.tenir_dans(1, Some(4096)), 1, "4032 tient dans 4096");
    let dernier = p.tenir_dans(1, Some(0));
    assert_eq!(
        p.dimensions(dernier),
        p.dimensions(1 << 31),
        "le plus petit niveau"
    );
    assert!(!p.tient_dans(1, Some(2048)) && p.tient_dans(2, Some(2048)));
    assert!(p.tient_dans(1, None));
}

/// **Un repli trop grand pour la carte ne se pose pas** : le voulu attend à sa place.
#[test]
fn test_plafond_1_un_repli_trop_grand_ne_se_pose_pas() {
    let p = Pyramide::nouvelle(unie(4032, 3024, [10, 20, 30, 255]));
    assert_eq!(
        p.repli_qui_tient(4, 2, Some(2048)),
        4,
        "1008 tient : il se pose"
    );
    assert_eq!(
        p.repli_qui_tient(1, 2, Some(2048)),
        2,
        "4032 ne tient pas : le voulu"
    );
    assert_eq!(p.repli_qui_tient(1, 2, None), 1, "sans plafond, le tenu");
}

/// Une rangée de gris opaques, de ces valeurs.
fn rangee(gris: &[u8]) -> Pixmap {
    let mut p = Pixmap::new(gris.len() as u32, 1).expect("une image");
    for (bloc, &g) in p.data_mut().as_chunks_mut::<4>().0.iter_mut().zip(gris) {
        *bloc = [g, g, g, 255];
    }
    p
}

/// L'ancienne réduction, rejouée : quatre sources à poids égaux, tronquées — d'étage en étage
/// jusqu'au pixel, sur une rangée.
fn ancien_sommet(mut gris: Vec<u32>) -> u32 {
    while gris.len() > 1 {
        gris = gris
            .chunks(2)
            .map(|b| b.iter().sum::<u32>() / b.len() as u32)
            .collect();
    }
    gris[0]
}

/// **MOYENNE-1 — le sommet est la couleur moyenne de l'image**, même quand un côté n'est pas
/// une puissance de deux. Cinq pixels, un seul blanc : la moyenne vaut 51.
///
/// La preuve à l'envers : l'ancienne réduction donnait au dernier pixel le poids d'un bloc
/// entier à chaque étage, et rendait 127 — le cinquième de l'image pesait la moitié.
#[test]
fn test_moyenne_1_le_sommet_est_la_moyenne_de_l_image() {
    let p = Pyramide::nouvelle(rangee(&[0, 0, 0, 0, 255]));
    assert_eq!(p.couleur_moyenne(), Some([51, 51, 51, 255]));
    assert_eq!(ancien_sommet(vec![0, 0, 0, 0, 255]), 127);

    let p = Pyramide::nouvelle(rangee(&[0, 0, 255]));
    assert_eq!(p.couleur_moyenne(), Some([85, 85, 85, 255]));
    assert_eq!(ancien_sommet(vec![0, 0, 255]), 127);

    // En deux dimensions, où les blocs entiers prennent le chemin rapide : un carré de 5 × 5
    // noir, sa dernière colonne blanche — le cinquième de l'image, 51 encore.
    let mut carre = unie(5, 5, [0, 0, 0, 255]);
    for (k, bloc) in carre
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        if k % 5 == 4 {
            *bloc = [255, 255, 255, 255];
        }
    }
    assert_eq!(
        Pyramide::nouvelle(carre).couleur_moyenne(),
        Some([51, 51, 51, 255])
    );
}

/// **MOYENNE-1, sur une image quelconque** : le sommet s'écarte de la moyenne exacte d'au plus
/// un demi par étage — l'arrondi de chacun. Et la moyenne s'arrondit au plus proche : l'ancienne
/// troncature assombrissait une image d'un gris à l'autre jusqu'à sortir de cette borne.
#[test]
fn test_moyenne_1_le_sommet_tient_a_un_arrondi_par_etage() {
    let (l, h) = (37u32, 23u32);
    let mut source = Pixmap::new(l, h).expect("une image");
    let mut somme = [0u64; 3];
    for (k, bloc) in source
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        let v = |m: usize| ((k * m + k * k / 7) % 256) as u8;
        *bloc = [v(31), v(57), v(11), 255];
        for c in 0..3 {
            somme[c] += u64::from(bloc[c]);
        }
    }
    let p = Pyramide::nouvelle(source);
    let sommet = p.couleur_moyenne().expect("un sommet tenu");
    let etages = (p.niveaux_construits() - 1) as f64;
    for c in 0..3 {
        let exacte = somme[c] as f64 / f64::from(l * h);
        let ecart = (f64::from(sommet[c]) - exacte).abs();
        assert!(
            ecart <= etages / 2.0,
            "canal {c} : {} contre {exacte}",
            sommet[c]
        );
    }
    assert_eq!(sommet[3], 255);
}

/// La moyenne s'arrondit au plus proche, et non vers le bas : deux pixels de 0 et 1 donnent 1
/// (0,5 arrondi au-dessus), là où la troncature rendait 0 — sur une rangée, et sur un bloc
/// entier de 2 × 2, qui prend le chemin rapide.
#[test]
fn test_moyenne_1_l_arrondi_est_au_plus_proche() {
    let p = Pyramide::nouvelle(rangee(&[0, 1]));
    assert_eq!(p.couleur_moyenne(), Some([1, 1, 1, 255]));
    assert_eq!(ancien_sommet(vec![0, 1]), 0);

    let mut bloc = unie(2, 2, [0, 0, 0, 255]);
    for (k, px) in bloc
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .enumerate()
    {
        if k == 1 || k == 2 {
            *px = [1, 1, 1, 255];
        }
    }
    assert_eq!(
        Pyramide::nouvelle(bloc).couleur_moyenne(),
        Some([1, 1, 1, 255])
    );
}
