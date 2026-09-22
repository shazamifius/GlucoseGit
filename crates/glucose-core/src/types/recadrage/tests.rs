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
