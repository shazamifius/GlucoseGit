//! **ETAGES-1** — tenu ce que l'écran montre, offert tout le reste, repris quand il revient.
//!
//! Chaque épreuve fait passer au vrai magasin les images du rendu telles que l'application
//! les enchaîne — ouvrir, récolter, réclamer, lire, fermer — et attend que l'atelier ait fini
//! ce que la fermeture lui a confié. Rien n'est posé à la main dans un état : l'état se lit
//! après les gestes.

use super::super::*;
use crate::renderer::photo::{Retour, Transit};

/// Une photo de 512 × 384 aux pixels tous différents : ses premiers niveaux occupent des
/// pages entières, donc s'offrent, et une reprise qui rendrait d'autres octets se verrait.
fn grande_photo(nom: &str) -> String {
    let dossier = std::env::temp_dir().join("glucose-magasin-tests");
    std::fs::create_dir_all(&dossier).expect("dossier de test");
    let chemin = dossier.join(nom);
    if !chemin.exists() {
        let mut brute = image::RgbaImage::new(512, 384);
        for (x, y, px) in brute.enumerate_pixels_mut() {
            *px = image::Rgba([(x % 251) as u8, (y % 241) as u8, ((x ^ y) % 256) as u8, 255]);
        }
        brute.save(&chemin).expect("écriture de la photo témoin");
    }
    chemin.to_string_lossy().to_string()
}

/// Une image du rendu, et l'attente de tout ce que sa fermeture a confié à l'atelier.
fn une_image(magasin: &mut Magasin, rendu: impl FnOnce(&mut Magasin)) {
    magasin.ouvrir();
    magasin.recolter();
    rendu(magasin);
    magasin.fermer();
    magasin.attendre_le_chantier();
}

/// Des images du rendu jusqu'à ce que la photo soit décodée.
fn arrivee(magasin: &mut Magasin, src: &str) {
    for _ in 0..1000 {
        une_image(magasin, |m| {
            m.reclamer(src, 0.0);
        });
        if magasin.cache.contains_key(src) {
            return;
        }
    }
    panic!("la photo témoin n'est jamais arrivée");
}

/// L'écran montre la photo à `largeur` pixels : le rendu la réclame et lit son niveau.
fn montree_a(largeur: f32, src: &str) -> impl FnOnce(&mut Magasin) + '_ {
    move |m| {
        m.reclamer(src, 512.0);
        let _ = m.cache[src].pyramide.meilleur_pour(largeur);
    }
}

fn natif(magasin: &Magasin, src: &str) -> Vec<u8> {
    magasin.cache[src]
        .pyramide
        .native()
        .expect("l'original est tenu")
        .data()
        .to_vec()
}

/// **Ce que l'écran ne montre pas s'offre** — et seulement cela.
///
/// Montrée à cent pixels, la photo veut son niveau de 128 : lui reste tenu, l'original et
/// le niveau de 256 partent chez le système, et ce qui tient dans moins d'une page reste là
/// parce que le système n'offre que des pages entières.
#[test]
fn test_ce_que_l_ecran_ne_montre_pas_s_offre() {
    let src = grande_photo("etages-offre.png");
    let mut magasin = Magasin::nouveau();
    arrivee(&mut magasin, &src);

    une_image(&mut magasin, montree_a(100.0, &src));

    let p = &magasin.cache[&src].pyramide;
    let voulu = p.rang_pour(100.0);
    assert_eq!(p.dimensions(1 << voulu).0, 128);
    for rang in 0..p.niveaux_construits() {
        let attendu = if rang != voulu && p.offrable(rang) {
            Etat::Offert
        } else {
            Etat::Tenu
        };
        assert_eq!(p.etat(rang), attendu, "le niveau de rang {rang}");
    }
    assert!(magasin.octets_en(Etat::Offert) > magasin.octets_en(Etat::Tenu));
    assert!(magasin.mouvements().offerts >= 2);
}

/// **Ce que l'écran redemande revient, au bit près.**
#[test]
fn test_ce_que_l_ecran_redemande_revient_intact() {
    let src = grande_photo("etages-reprise.png");
    let mut magasin = Magasin::nouveau();
    arrivee(&mut magasin, &src);
    let avant = natif(&magasin, &src);

    // Lire l'original pour s'en souvenir est un usage : il reste tenu une image de plus, et
    // c'est la suivante, où personne ne le lit, qui l'offre.
    une_image(&mut magasin, montree_a(100.0, &src));
    une_image(&mut magasin, montree_a(100.0, &src));
    assert_eq!(magasin.cache[&src].pyramide.etat(0), Etat::Offert);

    // Montrée plus grande que l'original : c'est lui que l'écran veut.
    une_image(&mut magasin, montree_a(600.0, &src));

    assert_eq!(magasin.cache[&src].pyramide.etat(0), Etat::Tenu);
    assert!(natif(&magasin, &src) == avant, "l'original repris a d'autres octets");
    assert!(magasin.mouvements().repris >= 1);
}

/// **La vue d'ensemble reste tenue, quoi qu'il arrive** : dézoomer jusqu'au tableau entier ne
/// demande rien au système.
///
/// L'échelle d'ensemble d'un quart pose la photo, large de 512 dans le document, à 128
/// pixels : le niveau de 128 et tous les plus petits restent, même sans être montrés.
#[test]
fn test_la_vue_d_ensemble_reste_tenue() {
    let src = grande_photo("etages-ensemble.png");
    let mut magasin = Magasin::nouveau();
    arrivee(&mut magasin, &src);
    magasin.echelle_ensemble = 0.25;

    // Réclamée sans qu'aucun niveau soit lu : seule la vue d'ensemble la retient.
    une_image(&mut magasin, |m| {
        m.reclamer(&src, 512.0);
    });

    let p = &magasin.cache[&src].pyramide;
    let queue = p.rang_pour(128.0);
    for rang in 0..p.niveaux_construits() {
        let attendu = if rang < queue && p.offrable(rang) {
            Etat::Offert
        } else {
            Etat::Tenu
        };
        assert_eq!(p.etat(rang), attendu, "le niveau de rang {rang}");
    }
}

/// **Un niveau que le système a jeté se redécode** depuis son fichier, et revient identique.
#[test]
fn test_un_niveau_perdu_se_redecode() {
    let src = grande_photo("etages-perdu.png");
    let mut magasin = Magasin::nouveau();
    arrivee(&mut magasin, &src);
    let avant = natif(&magasin, &src);
    // Lire l'original est un usage : il faut une image sans lecture pour qu'il s'offre.
    une_image(&mut magasin, montree_a(100.0, &src));
    une_image(&mut magasin, montree_a(100.0, &src));

    // Ce que la reprise dirait si le système l'avait jeté.
    let entree = magasin.cache.get_mut(&src).expect("la photo");
    assert!(matches!(
        entree.pyramide.sortir(0),
        Some(Transit::AReprendre(_))
    ));
    entree.pyramide.rentrer(0, Retour::Repris(None));
    assert_eq!(entree.pyramide.etat(0), Etat::Perdu);

    une_image(&mut magasin, montree_a(600.0, &src));

    assert!(natif(&magasin, &src) == avant, "l'original redécodé diffère");
}

/// **Ce qui revient pour une pyramide remplacée ne s'y pose pas** : une image redécodée en a
/// une neuve, et un niveau de l'ancienne y mettrait des octets d'ailleurs.
#[test]
fn test_ce_qui_revient_pour_une_autre_pyramide_ne_s_y_pose_pas() {
    let src = grande_photo("etages-generation.png");
    let mut magasin = Magasin::nouveau();
    arrivee(&mut magasin, &src);
    une_image(&mut magasin, montree_a(100.0, &src));

    let entree = magasin.cache.get_mut(&src).expect("la photo");
    let generation = entree.generation;
    let Some(Transit::AReprendre(offerte)) = entree.pyramide.sortir(0) else {
        panic!("l'original était offert");
    };
    let retour = |generation| Deplacement {
        src: src.clone(),
        generation,
        rang: 0,
        charge: Retour::Repris(None),
    };
    magasin.rentrer(retour(generation + 1));
    assert_eq!(magasin.cache[&src].pyramide.etat(0), Etat::EnChemin);

    drop(offerte);
    magasin.rentrer(retour(generation));
    assert_eq!(magasin.cache[&src].pyramide.etat(0), Etat::Perdu);
}

/// **Une image vue une fois s'ouvre ensuite déjà montrée** (ETAGES-4).
///
/// Première session : la photo est décodée, l'écran la montre, et sa vue d'ensemble — le
/// niveau de 128 pour une échelle d'ensemble d'un quart — s'écrit sur le disque. Seconde
/// session, un magasin neuf sur le même dossier : la photo arrive par son aperçu, ses petits
/// niveaux tenus et ses grands perdus ; quand l'écran veut l'original, il se redécode.
#[test]
fn test_une_image_vue_une_fois_s_ouvre_deja_montree() {
    let src = grande_photo("etages-apercu.png");
    let dossier =
        std::env::temp_dir().join(format!("glucose-apercus-magasin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dossier);
    let session = || {
        let mut m = Magasin::nouveau();
        m.brancher_les_apercus(dossier.clone());
        m.echelle_ensemble = 0.25;
        for _ in 0..1000 {
            une_image(&mut m, |m| {
                m.reclamer(&src, 512.0);
            });
            if m.cache.contains_key(&src) {
                return m;
            }
        }
        panic!("la photo temoin n'est jamais arrivee");
    };

    let mut premiere = session();
    une_image(&mut premiere, montree_a(100.0, &src));
    let chemin = crate::renderer::apercu::chemin(&dossier, &src).expect("un chemin");
    assert!(chemin.exists(), "la vue d'ensemble s'est ecrite");

    let mut seconde = session();
    let p = &seconde.cache[&src].pyramide;
    let queue = p.rang_pour(128.0);
    assert_eq!(p.dimensions(1 << queue).0, 128);
    assert_eq!(p.etat(0), Etat::Perdu, "l'original ne vient pas du disque");
    assert_eq!(p.etat(queue), Etat::Tenu, "la vue d'ensemble, si");

    une_image(&mut seconde, montree_a(600.0, &src));
    assert_eq!(
        seconde.cache[&src].pyramide.etat(0),
        Etat::Tenu,
        "l'ecran voulait l'original : il s'est redecode"
    );
}
