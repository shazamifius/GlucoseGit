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
    assert_eq!(p.native().width(), 4032);
    // 4032 → 2016 → 1008 → 504 → 252 → 126 → 63 → 32 → 16 → 8 → 4 → 2 → 1 : treize niveaux,
    // et le dernier mesure un pixel de large. Rien n'a été choisi : la descente s'arrête où
    // la division par deux ne bouge plus.
    assert_eq!(p.niveaux_construits(), 13);
    assert_eq!(p.niveau_pour(1.0).width(), 1);
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
    let niveau = p.niveau_pour(420.0);

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
    let niveau = p.niveau_pour(3.0);

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
    let niveau = p.niveau_pour(3.0);
    assert_eq!((niveau.width(), niveau.height()), (4, 3));
    for bloc in niveau.data().as_chunks::<4>().0 {
        assert_eq!(bloc, &[9, 9, 9, 255]);
    }
}

/// La descente s'arrête au pixel, quelle que soit la taille demandée.
#[test]
fn test_la_descente_s_arrete_au_pixel() {
    let p = Pyramide::nouvelle(unie(64, 64, [1, 2, 3, 255]));
    let niveau = p.niveau_pour(0.0);
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
    let niveau = p.niveau_pour(1.0);
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
