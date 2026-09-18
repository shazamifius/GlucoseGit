//! OCCLUSION-2 — ce qu'on se permet de ne pas dessiner.
//!
//! La regle qui gouverne tous ces tests : **on peut designer un peu trop, jamais trop peu**.
//! Dessiner en trop coute ; dessiner en moins laisse un trou a l'ecran, et c'est le seul
//! defaut vraiment grave ici.

use super::*;
use std::ops::Not;

const FENETRE: Boite = Boite {
    x: 0.0,
    y: 0.0,
    largeur: 1000.0,
    hauteur: 800.0,
};

fn opaque(x: f32, y: f32, w: f32, h: f32) -> Calque {
    Calque {
        boite: Boite::nouvelle(x, y, w, h),
        opaque: true,
    }
}

fn voile(x: f32, y: f32, w: f32, h: f32) -> Calque {
    Calque {
        boite: Boite::nouvelle(x, y, w, h),
        opaque: false,
    }
}

/// Ce qui reste a peindre, calque par calque : une liste vide veut dire entierement cache.
fn visibles(calques: &[Calque]) -> Vec<Vec<Boite>> {
    let mut out = Visibles::default();
    ce_qui_se_voit(calques, FENETRE, &mut out);
    (0..calques.len()).map(|r| out.parts(r).to_vec()).collect()
}

#[test]
fn test_une_opaque_qui_couvre_tout_cache_ce_qui_precede() {
    let r = visibles(&[
        opaque(10.0, 10.0, 50.0, 50.0),
        opaque(0.0, 0.0, 1000.0, 800.0),
    ]);
    assert!(r[0].is_empty(), "la premiere est entierement cachee");
    assert!(r[1].is_empty().not(), "celle qui couvre se dessine");
}

#[test]
fn test_une_translucide_ne_cache_rien() {
    // Le fond transparait a travers elle : tout ce qu'il y a dessous reste visible.
    let r = visibles(&[
        opaque(10.0, 10.0, 50.0, 50.0),
        voile(0.0, 0.0, 1000.0, 800.0),
    ]);
    assert!(r[0].is_empty().not(), "rien n'est cache par un voile");
    assert!(r[1].is_empty().not());
}

#[test]
fn test_ce_qui_vient_apres_n_est_jamais_cache() {
    // L'ordre est celui du dessin : une opaque ne peut cacher que ce qui la PRECEDE.
    let r = visibles(&[
        opaque(0.0, 0.0, 1000.0, 800.0),
        opaque(10.0, 10.0, 50.0, 50.0),
    ]);
    assert!(
        r[0].is_empty().not(),
        "la premiere est dessous, elle se voit encore"
    );
    assert!(r[1].is_empty().not());
}

#[test]
fn test_plusieurs_opaques_cachent_ensemble_ce_qu_aucune_ne_cache_seule() {
    // **Le cas reel, et celui qu'un test de deux rectangles ne trouve jamais.** Deux moities
    // d'ecran ne couvrent chacune que la moitie ; ensemble elles couvrent tout.
    let r = visibles(&[
        opaque(100.0, 100.0, 200.0, 200.0),
        opaque(0.0, 0.0, 500.0, 800.0),
        opaque(500.0, 0.0, 500.0, 800.0),
    ]);
    assert!(
        r[0].is_empty(),
        "deux moities d'ecran cachent ce qui est dessous, meme si aucune ne le fait seule"
    );
}

#[test]
fn test_un_calque_partiellement_couvert_garde_sa_partie_visible() {
    // La moitie gauche est recouverte ; la droite doit rester.
    let r = visibles(&[
        opaque(0.0, 0.0, 1000.0, 800.0),
        opaque(0.0, 0.0, 500.0, 800.0),
    ]);
    assert!(!r[0].is_empty(), "le fond se voit encore a droite");
    let reste = r[0].iter().fold(r[0][0], |a, b| a.englobant(b));
    assert!(
        reste.droite() > 500.0,
        "la partie droite doit rester : {reste:?}"
    );
    assert!(
        reste.x >= 400.0,
        "et la partie gauche doit avoir ete retiree : {reste:?}"
    );
}

#[test]
fn test_un_calque_hors_fenetre_ne_se_dessine_pas() {
    let r = visibles(&[opaque(-5000.0, -5000.0, 100.0, 100.0)]);
    assert!(r[0].is_empty(), "ce qui est hors champ n'a rien a dessiner");
}

#[test]
fn test_la_partie_utile_est_toujours_dans_la_fenetre() {
    // Une boite a cheval ne doit jamais rendre de coordonnees hors de l'ecran : ce serait une
    // ecriture hors du tampon, pas un simple gaspillage.
    let r = visibles(&[opaque(900.0, 700.0, 500.0, 500.0)]);
    assert!(!r[0].is_empty(), "la partie visible existe");
    let utile = r[0][0];
    assert!(utile.x >= FENETRE.x && utile.y >= FENETRE.y);
    assert!(utile.droite() <= FENETRE.droite() + 1.0);
    assert!(utile.bas() <= FENETRE.bas() + 1.0);
}

/// La surface qu'il faudrait peindre sans occlusion, et avec -- en pixels.
///
/// C'est **la** grandeur qui compte, et non le nombre de calques elimines : un calque qui
/// garde une bande de trois pixels coute presque rien, alors qu'il compte encore pour un.
/// Compter les calques aurait masque le vrai gain, dans un sens comme dans l'autre.
fn surfaces(calques: &[Calque]) -> (f64, f64) {
    let mut out = Visibles::default();
    ce_qui_se_voit(calques, FENETRE, &mut out);
    let sans: f64 = calques
        .iter()
        .map(|c| {
            let v = c.boite.commune(&FENETRE);
            if v.est_vide() {
                0.0
            } else {
                f64::from(v.largeur) * f64::from(v.hauteur)
            }
        })
        .sum();
    (sans, out.surface())
}

#[test]
fn test_une_pile_de_photos_qui_se_chevauchent() {
    // Le cas mesure sur le terrain : cent cinq photos qui se chevauchent partiellement, sans
    // qu'aucune ne remplisse l'ecran. C'est celui qu'OCCLUSION-1 ne savait pas traiter.
    //
    // Aucune n'est ENTIEREMENT cachee -- chacune garde une bande de quelques pixels -- et
    // c'est pourquoi ce test regarde la SURFACE et non le nombre de calques.
    let mut calques = Vec::new();
    for i in 0..105 {
        let d = i as f32 * 3.0;
        calques.push(opaque(d, d, 700.0, 500.0));
    }
    let (sans, avec) = surfaces(&calques);
    assert!(
        avec < sans / 2.0,
        "la surface a peindre doit fondre : {sans:.0} -> {avec:.0} pixels"
    );
}

#[test]
fn test_ce_qui_est_entierement_recouvert_disparait_vraiment() {
    // Quand une photo en contient une autre, la cachee ne doit plus rien couter du tout.
    let mut calques = vec![opaque(100.0, 100.0, 200.0, 150.0)];
    calques.push(opaque(0.0, 0.0, 900.0, 700.0));
    let r = visibles(&calques);
    assert!(r[0].is_empty(), "une photo contenue disparait entierement");
}

#[test]
fn test_rien_a_dessiner_ne_panique_pas() {
    let mut out = Visibles::default();
    ce_qui_se_voit(&[], FENETRE, &mut out);
    assert_eq!(out.calques(), 0);

    // Une fenetre vide non plus : cela arrive a la toute premiere image, avant que la taille
    // soit connue.
    ce_qui_se_voit(
        &[opaque(0.0, 0.0, 10.0, 10.0)],
        Boite::nouvelle(0.0, 0.0, 0.0, 0.0),
        &mut out,
    );
    assert_eq!(out.calques(), 1);
    assert!(out.parts(0).is_empty());
}

#[test]
fn test_le_tampon_est_reutilise_sans_laisser_de_reste() {
    // Le tampon est passe plutot que rendu, pour eviter une allocation par image. Il doit donc
    // etre vide de ce qu'il portait : un reste ferait dessiner un calque qui n'existe plus.
    let mut out = Visibles::default();
    ce_qui_se_voit(&[opaque(0.0, 0.0, 100.0, 100.0); 5], FENETRE, &mut out);
    assert_eq!(out.calques(), 5);
    ce_qui_se_voit(&[opaque(0.0, 0.0, 100.0, 100.0)], FENETRE, &mut out);
    assert_eq!(out.calques(), 1);
}

#[test]
fn test_la_reponse_est_conservatrice_et_non_exacte() {
    // Une tuile a moitie couverte compte comme NON couverte : la reponse designe donc un peu
    // plus que le strict necessaire. C'est voulu, et c'est ce qui garantit qu'on ne dessine
    // jamais trop peu -- le seul defaut qui se verrait.
    let r = visibles(&[
        opaque(0.0, 0.0, 1000.0, 800.0),
        // Elle couvre presque tout, mais s'arrete a mi-tuile.
        opaque(0.0, 0.0, 990.0, 790.0),
    ]);
    assert!(
        r[0].is_empty().not(),
        "le fond garde une bande visible, fut-elle arrondie a la tuile"
    );
}
