//! Ce que le recadrage promet : il reste toujours quelque chose à voir, et le neutre est
//! vraiment neutre.

use super::*;

/// **L'invariant RECADRAGE-1 tient, quoi qu'on propose.**
///
/// Une détection de bordures sur une image entièrement noire proposerait de tout retirer ; une
/// poignée tirée au-delà de l'image opposée proposerait des marges qui se croisent. Dans les
/// deux cas, ce qui reste doit être une image — pas un infini au moment de diviser par la
/// largeur visible.
#[test]
fn test_il_reste_toujours_quelque_chose_a_voir() {
    for (g, h, d, b) in [
        (1.0, 1.0, 1.0, 1.0),
        (0.9, 0.0, 0.9, 0.0),
        (0.6, 0.6, 0.6, 0.6),
        (f64::INFINITY, 0.0, 0.0, 0.0),
        (-3.0, -3.0, 2.0, 2.0),
        (0.5, 0.0, 0.5, 0.0),
    ] {
        let r = Recadrage::depuis_les_marges(g, h, d, b);
        assert!(
            r.largeur_visible() > 0.0 && r.largeur_visible() <= 1.0,
            "largeur visible {} pour ({g}, {h}, {d}, {b})",
            r.largeur_visible()
        );
        assert!(
            r.hauteur_visible() > 0.0 && r.hauteur_visible() <= 1.0,
            "hauteur visible {} pour ({g}, {h}, {d}, {b})",
            r.hauteur_visible()
        );
        let (mg, mh, md, mb) = r.marges();
        for m in [mg, mh, md, mb] {
            assert!((0.0..1.0).contains(&m), "marge hors domaine : {m}");
        }
    }
}

/// **Des marges négatives ne poussent pas les bords vers l'extérieur.**
///
/// Une poignée tirée au-delà du bord de l'image proposerait une marge négative. L'accepter
/// ferait montrer des pixels qui n'existent pas — du vide, ou pire, ce qui traîne à côté dans
/// la texture d'un atlas.
#[test]
fn test_une_marge_negative_est_ramenee_a_zero() {
    let r = Recadrage::depuis_les_marges(-0.5, -0.2, 0.1, 0.0);
    let (g, h, d, b) = r.marges();
    assert_eq!((g, h), (0.0, 0.0));
    assert!((d - 0.1).abs() < 1e-12);
    assert_eq!(b, 0.0);
}

/// **Un cadrage décentré reste décentré quand on le ramène.**
///
/// Réduire les deux marges du même facteur garde leur proportion. Les ramener chacune à la
/// moitié du permis recentrerait le cadrage tout seul, et ce serait un geste que l'utilisateur
/// n'a pas fait.
#[test]
fn test_un_cadrage_decentre_reste_decentre() {
    // Neuf dixièmes à gauche, rien à droite : après réduction, tout reste à gauche.
    let r = Recadrage::depuis_les_marges(1.8, 0.0, 0.0, 0.0);
    let (g, _, d, _) = r.marges();
    assert_eq!(d, 0.0, "rien n'a ete propose a droite, rien n'y arrive");
    assert!((g - 0.99).abs() < 1e-12, "gauche vaut {g}");

    // Deux tiers d'un côté, un tiers de l'autre : le rapport se conserve.
    let r = Recadrage::depuis_les_marges(0.8, 0.0, 0.4, 0.0);
    let (g, _, d, _) = r.marges();
    assert!(
        (g / d - 2.0).abs() < 1e-9,
        "le rapport doit tenir : {g} / {d}"
    );
}

/// **Le neutre est vraiment neutre**, et il n'a pas de jumeau.
///
/// C'est la raison pour laquelle ce type n'est pas une `Option` : « pas de recadrage » et
/// « recadrage qui garde tout » doivent être le même état, sans quoi une comparaison ment.
#[test]
fn test_le_neutre_ne_retire_rien_et_n_a_pas_de_jumeau() {
    assert!(Recadrage::ENTIER.est_entier());
    assert!(Recadrage::default().est_entier());
    assert!(Recadrage::depuis_les_marges(0.0, 0.0, 0.0, 0.0).est_entier());
    assert_eq!(Recadrage::ENTIER.largeur_visible(), 1.0);
    assert_eq!(Recadrage::ENTIER.hauteur_visible(), 1.0);
    assert!(!Recadrage::depuis_les_marges(0.0, 0.1, 0.0, 0.0).est_entier());
}

/// **Le rapport suit ce qu'on montre**, et c'est ce qui empêche une image cadrée de s'écraser.
///
/// Une image en boîte aux lettres de 1920 × 1080 dont on retire deux bandes de 135 pixels
/// (un huitième en haut et en bas) montre 1920 × 810, soit un rapport de 2,37 — le format
/// large d'origine. Laisser sa boîte au rapport 16/9 l'étirerait verticalement.
#[test]
fn test_le_rapport_suit_ce_qu_on_montre() {
    let entier = Recadrage::ENTIER.rapport(1920.0, 1080.0);
    assert!((entier - 16.0 / 9.0).abs() < 1e-12);

    let sans_bandes = Recadrage::depuis_les_marges(0.0, 0.125, 0.0, 0.125);
    let vu = sans_bandes.rapport(1920.0, 1080.0);
    assert!((vu - 1920.0 / 810.0).abs() < 1e-9, "rapport obtenu : {vu}");
    assert!(vu > entier, "retirer du haut et du bas ELARGIT le rapport");
}

/// **Le plus serré des deux ne relâche jamais** : une détection ne rend pas une bande que
/// l'utilisateur avait déjà retirée, et garde ce qu'elle trouve en plus.
#[test]
fn test_le_plus_serre_ne_relache_jamais() {
    let main = Recadrage::depuis_les_marges(0.2, 0.0, 0.0, 0.1);
    let detecte = Recadrage::depuis_les_marges(0.05, 0.15, 0.05, 0.0);
    let (g, h, d, b) = main.le_plus_serre(detecte).marges();
    assert!((g - 0.2).abs() < 1e-12, "la main avait serre plus a gauche");
    assert!(
        (h - 0.15).abs() < 1e-12,
        "la detection a trouve une bande en haut"
    );
    assert!((d - 0.05).abs() < 1e-12);
    assert!((b - 0.1).abs() < 1e-12, "la main avait serre plus en bas");
}

/// **Retirer une bande ne déplace pas ce qu'on garde.**
///
/// Une image de 200 × 100 posée en (0, 0), dont on retire le quart gauche : sa boîte devient
/// (50, 0, 150, 100). Le contenu qui était en x = 50 y est toujours — la source entière se
/// pose toujours en (0, 0, 200, 100), et seule la fenêtre a changé.
#[test]
fn test_retirer_une_bande_ne_deplace_pas_ce_qu_on_garde() {
    let entier = Recadrage::ENTIER;
    let quart = Recadrage::depuis_les_marges(0.25, 0.0, 0.0, 0.0);
    let (x, y, w, h) = entier.boite_apres(quart, (0.0, 0.0, 200.0, 100.0));
    assert_eq!((x, y, w, h), (50.0, 0.0, 150.0, 100.0));

    // La source, vue depuis la nouvelle boîte et le nouveau recadrage, est la même : c'est
    // la preuve que rien n'a bougé.
    assert_eq!(
        quart.source_pour((x, y, w, h)),
        entier.source_pour((0.0, 0.0, 200.0, 100.0))
    );

    // Et un second recadrage par-dessus le premier se compose : retirer encore un dixième
    // du bas d'une image déjà cadrée au quart.
    let plus = Recadrage::depuis_les_marges(0.25, 0.0, 0.0, 0.1);
    let (x2, y2, w2, h2) = quart.boite_apres(plus, (x, y, w, h));
    assert_eq!((x2, y2, w2), (50.0, 0.0, 150.0));
    assert!((h2 - 90.0).abs() < 1e-9, "dix pour cent de cent : {h2}");
    assert_eq!(plus.source_pour((x2, y2, w2, h2)), (0.0, 0.0, 200.0, 100.0));
}

/// **Tirer un bord recadre de ce qu'on a tiré**, et le tirer en arrière rend ce qu'on avait
/// retiré — jamais plus que l'image.
#[test]
fn test_tirer_un_bord_recadre_de_ce_qu_on_a_tire() {
    use crate::resize::Handle;
    let boite = (0.0, 0.0, 200.0, 100.0);
    // Le bord gauche tiré de cinquante vers la droite : un quart de la source.
    let quart = Recadrage::ENTIER.en_tirant_le_bord(Handle::Left, (50.0, 0.0), boite);
    let (g, h, d, b) = quart.marges();
    assert!((g - 0.25).abs() < 1e-12, "gauche : {g}");
    assert_eq!((h, d, b), (0.0, 0.0, 0.0));

    // Depuis cette boîte cadrée (150 de large), le même bord tiré de vingt en arrière rend
    // vingt unités : la source entière fait toujours 200, donc un dixième.
    let boite_cadree = Recadrage::ENTIER.boite_apres(quart, boite);
    let moins = quart.en_tirant_le_bord(Handle::Left, (-20.0, 0.0), boite_cadree);
    assert!(
        (moins.marges().0 - 0.15).abs() < 1e-12,
        "gauche : {}",
        moins.marges().0
    );

    // Tiré bien au-delà de l'image : on s'arrête à l'image entière.
    let trop = quart.en_tirant_le_bord(Handle::Left, (-500.0, 0.0), boite_cadree);
    assert_eq!(trop.marges().0, 0.0);

    // Le bas se tire vers le haut ; un coin ne recadre pas.
    let bas = Recadrage::ENTIER.en_tirant_le_bord(Handle::Bottom, (0.0, -25.0), boite);
    assert!((bas.marges().3 - 0.25).abs() < 1e-12);
    assert_eq!(
        Recadrage::ENTIER.en_tirant_le_bord(Handle::TopLeft, (30.0, 30.0), boite),
        Recadrage::ENTIER
    );
}

// ── BORDURES-4 : les texels qu'on a le droit de lire ─────────────────────────────────

/// **Sans recadrage, tout se lit**, à tous les niveaux — y compris le dernier texel d'une
/// taille impaire, qui ne couvre qu'un pixel natif et le couvre entièrement.
#[test]
fn test_une_image_entiere_se_lit_entiere_a_tout_niveau() {
    for (l, h) in [(1200u32, 576u32), (1199, 575), (1, 1), (3, 7)] {
        for facteur in [1u32, 2, 4, 8] {
            let (lf, hf) = (l.div_ceil(facteur), h.div_ceil(facteur));
            assert_eq!(
                Recadrage::ENTIER.texels_lisibles((l, h), facteur),
                [0, 0, lf - 1, hf - 1],
                "{l} x {h} reduite {facteur} fois"
            );
        }
    }
}

/// **Un bord entier ne bascule jamais**, et c'est ce qui dispense de toute tolérance.
///
/// `Ctrl+B` pose des marges de la forme `x / largeur`, et `(x / largeur) · largeur` ne rend
/// pas toujours `x` en virgule flottante. La règle du centre lit `x − ½`, à un demi-pixel de
/// tout arrondi : chaque bord possible de trois largeurs réelles est vérifié, un par un.
#[test]
fn test_un_bord_entier_ne_bascule_jamais() {
    for l in [576u32, 1199, 1200, 4097] {
        // Au-dela, le constructeur ramene les marges pour garder un centieme (RECADRAGE-1).
        for x in 0..l * 99 / 200 {
            let marge = f64::from(x) / f64::from(l);
            let r = Recadrage::depuis_les_marges(marge, 0.0, marge, 0.0);
            let [g, _, d, _] = r.texels_lisibles((l, 10), 1);
            assert_eq!((g, d), (x, l - 1 - x), "largeur {l}, marge de {x} pixels");
        }
    }
}

/// **Un texel qui chevauche le bord ne se lit pas** : il ramènerait la moitié de la bande.
///
/// La forêt de l'utilisateur, recadrée par `Ctrl+B` : 14 colonnes à gauche, 15 à droite.
/// Réduite de moitié, la colonne 1 185 — retirée — partage son texel avec la 1 184 — gardée :
/// ce texel-là est refusé, et le bord se prolonge depuis son voisin entièrement dedans.
#[test]
fn test_un_texel_qui_chevauche_le_bord_ne_se_lit_pas() {
    let (l, h) = (1200u32, 576u32);
    let r = Recadrage::depuis_les_marges(
        14.0 / f64::from(l),
        12.0 / f64::from(h),
        15.0 / f64::from(l),
        17.0 / f64::from(h),
    );
    assert_eq!(r.texels_lisibles((l, h), 1), [14, 12, 1184, 558]);
    // Réduite deux fois : 1 184 et 1 185 font le texel 592, 558 et 559 le rang 279.
    assert_eq!(r.texels_lisibles((l, h), 2), [7, 6, 591, 278]);
    // Quatre fois : les colonnes 12 à 15 font le texel 3, dont deux sont retirées.
    assert_eq!(r.texels_lisibles((l, h), 4)[0], 4);
}

/// **Une fenêtre plus étroite qu'un texel lit celui qui contient son milieu** : aucun n'est
/// entièrement dedans, et il faut pourtant montrer quelque chose.
#[test]
fn test_une_fenetre_plus_etroite_qu_un_texel_lit_son_milieu() {
    // Un centième de 100 pixels : un seul pixel natif, le 50, réduit huit fois.
    let r = Recadrage::depuis_les_marges(0.5, 0.0, 0.49, 0.0);
    let [g, _, d, _] = r.texels_lisibles((100, 10), 8);
    assert_eq!((g, d), (6, 6), "le pixel 50 vit dans le texel 6");
}
