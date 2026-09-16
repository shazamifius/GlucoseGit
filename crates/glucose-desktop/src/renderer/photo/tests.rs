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

/// Une pyramide naît avec son seul niveau natif : regarder une image ne coûte rien tant qu'on
/// ne la dessine pas.
#[test]
fn test_une_pyramide_nait_paresseuse() {
    let p = Pyramide::nouvelle(unie(4032, 3024, [10, 20, 30, 255]));
    assert_eq!(p.niveaux_construits(), 1);
    assert_eq!(p.native().width(), 4032);
}

/// Le niveau rendu couvre la taille demandée, et le suivant ne la couvrirait plus.
///
/// C'est toute la définition : plus petit que ça, on agrandirait, donc on perdrait du détail
/// que la source avait.
#[test]
fn test_le_niveau_choisi_couvre_la_taille_demandee() {
    let mut p = Pyramide::nouvelle(unie(4032, 3024, [10, 20, 30, 255]));
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

/// Redemander la même taille ne construit rien de plus.
#[test]
fn test_redemander_le_meme_niveau_ne_construit_rien() {
    let mut p = Pyramide::nouvelle(unie(4032, 3024, [10, 20, 30, 255]));
    p.niveau_pour(420.0);
    let apres_le_premier = p.niveaux_construits();
    for _ in 0..50 {
        p.niveau_pour(420.0);
    }
    assert_eq!(p.niveaux_construits(), apres_le_premier);
}

/// Une image unie le reste à tous les niveaux : la moyenne de quatre valeurs égales est cette
/// valeur. Si une réduction décalait ses indices, la couleur baverait.
#[test]
fn test_une_image_unie_le_reste_a_tous_les_niveaux() {
    let couleur = [40, 80, 120, 255];
    let mut p = Pyramide::nouvelle(unie(64, 64, couleur));
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
    let mut p = Pyramide::nouvelle(unie(7, 5, [9, 9, 9, 255]));
    let niveau = p.niveau_pour(3.0);
    assert_eq!((niveau.width(), niveau.height()), (4, 3));
    for bloc in niveau.data().as_chunks::<4>().0 {
        assert_eq!(bloc, &[9, 9, 9, 255]);
    }
}

/// La descente s'arrête au pixel, quelle que soit la taille demandée.
#[test]
fn test_la_descente_s_arrete_au_pixel() {
    let mut p = Pyramide::nouvelle(unie(64, 64, [1, 2, 3, 255]));
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

    let mut p = Pyramide::nouvelle(source);
    let niveau = p.niveau_pour(1.0);
    assert_eq!((niveau.width(), niveau.height()), (1, 1));
    assert_eq!(&niveau.data()[0..4], &[50, 50, 50, 255]);
}

/// Le coût en mémoire d'une pyramide complète reste sous un tiers de plus que le natif — la
/// somme de 1 + 1/4 + 1/16 + … C'est la contrepartie de MIP-1, et elle doit rester bornée.
#[test]
fn test_la_pyramide_complete_coute_un_tiers_de_plus() {
    let natif = 4032 * 3024 * 4;
    let mut p = Pyramide::nouvelle(unie(4032, 3024, [0, 0, 0, 255]));
    p.niveau_pour(1.0);

    assert!(
        p.octets() < natif * 3 / 2,
        "la pyramide occupe {} octets pour un natif de {natif}",
        p.octets()
    );
}
