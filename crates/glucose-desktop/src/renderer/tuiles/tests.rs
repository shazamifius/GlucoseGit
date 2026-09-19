//! Ce que le cache de tuiles doit garantir — et surtout, qu'il **sert**.
//!
//! La leçon des vignettes est écrite ici : un cache qui repeindrait tout à chaque image
//! rendrait les mêmes pixels et passerait toutes les épreuves d'aspect. Ces tests portent donc
//! sur les **comptes** — ce qui a été peint, ce qui a été repris — parce que c'est la seule
//! chose qu'un cache inutile ne peut pas simuler.

use super::*;
use glucose_core::geometry::Rect;
use glucose_core::types::BoardImage;

fn carre(cote: u32) -> Pixmap {
    Pixmap::new(cote, cote).expect("une tuile")
}

fn empreinte(n: u64) -> Empreinte {
    Empreinte::depuis(n)
}

fn adresse(x: i64, y: i64) -> Adresse {
    Adresse { niveau: 0, x, y }
}

/// **Deux cases de même empreinte ne se peignent qu'une fois.** C'est la mémoïsation de
/// Hashlife, et la seule raison d'être de ce cache.
#[test]
fn deux_cases_de_meme_empreinte_partagent_un_seul_rendu() {
    let mut t = Tuiles::nouveau();
    t.ouvrir();

    let e = empreinte(0x1234);
    assert!(t.deja_peinte(e).is_none(), "rien n'est peint au depart");
    t.ranger(e, carre(COTE));

    // Une autre case du monde, mais le même contenu : elle trouve le rendu déjà là.
    assert!(t.deja_peinte(e).is_some());
    assert_eq!(t.peintes(), 1, "une seule peinture pour les deux cases");
    assert_eq!(t.reprises(), 1);
    assert_eq!(t.gardes(), 1, "un seul rendu garde, pas deux");
}

/// **Ce qui n'a pas servi à l'image s'en va.** La borne du cache est donc ce que l'écran
/// demande, et aucun nombre n'a eu à être choisi.
#[test]
fn ce_qui_n_a_pas_servi_est_oublie() {
    let mut t = Tuiles::nouveau();
    t.ouvrir();
    t.ranger(empreinte(1), carre(COTE));
    t.ranger(empreinte(2), carre(COTE));
    t.noter(adresse(0, 0), empreinte(1));
    t.noter(adresse(1, 0), empreinte(2));
    t.fermer();
    assert_eq!(t.gardes(), 2, "les deux viennent de servir");

    // Image suivante : une seule des deux sert.
    t.ouvrir();
    assert!(t.deja_peinte(empreinte(1)).is_some());
    t.fermer();
    assert_eq!(t.gardes(), 1, "l'autre est oubliee");
    assert!(t.portee(adresse(1, 0)).is_none(), "et sa case avec elle");
    assert_eq!(t.portee(adresse(0, 0)), Some(empreinte(1)));
}

/// **Le cache ne grandit pas indéfiniment**, même si l'on traverse tout le document : ce qui
/// sort de l'écran sort du cache à l'image suivante.
#[test]
fn traverser_le_document_ne_fait_pas_enfler_le_cache() {
    let mut t = Tuiles::nouveau();
    for pas in 0..200u64 {
        t.ouvrir();
        // Quatre tuiles à l'écran, toutes différentes, qui défilent.
        for k in 0..4 {
            let e = empreinte(pas * 4 + k);
            if t.deja_peinte(e).is_none() {
                t.ranger(e, carre(COTE));
            }
        }
        t.fermer();
        assert!(
            t.gardes() <= 4,
            "a l'image {pas}, le cache garde {} rendus",
            t.gardes()
        );
    }
}

/// **Une case vide porte l'empreinte vide**, et toutes les cases vides se valent : traverser
/// une zone sans contenu ne peint qu'une seule tuile, pour tout le vide du document.
#[test]
fn tout_le_vide_du_document_ne_coute_qu_une_tuile() {
    let store = Store::new("essai");
    let mut t = Tuiles::nouveau();
    t.ouvrir();

    for (x, y) in [(0, 0), (12, 7), (-400, 900), (1_000_000, -1_000_000)] {
        let e = ce_que_porte(&store, adresse(x, y), &[]);
        assert_eq!(e, Empreinte::vide(), "la case ({x}, {y}) est vide");
        if t.deja_peinte(e).is_none() {
            t.ranger(e, carre(COTE));
        }
    }
    assert_eq!(t.peintes(), 1, "une seule peinture pour tout le vide");
    assert_eq!(t.reprises(), 3);
}

/// Une photo **hors** de la tuile n'entre pas dans son empreinte : la tuile n'en montre rien,
/// et s'en souvenir la ferait périmer pour une raison qu'elle n'affiche pas.
#[test]
fn une_photo_hors_de_la_tuile_ne_compte_pas() {
    let mut store = Store::new("essai");
    let board = store.project.active_board_id.clone();
    let cote = Adresse::cote_monde(0);
    // Une photo bien à l'intérieur de la case (2, 2), donc hors de la case (0, 0).
    let mut img = BoardImage::new("i1", cote * 2.0 + 10.0, cote * 2.0 + 10.0, 50.0, 50.0);
    img.src = Some("photo.png".into());
    store.add_image(&board, img);

    let rangs = [0u32];
    assert_eq!(
        ce_que_porte(&store, adresse(0, 0), &rangs),
        Empreinte::vide(),
        "la case (0,0) ne montre rien de cette photo"
    );
    assert_ne!(
        ce_que_porte(&store, adresse(2, 2), &rangs),
        Empreinte::vide(),
        "la case (2,2) la montre"
    );
}

/// **Le même mur de photos, posé deux fois dans le document, ne se peint qu'une.** C'est la
/// promesse de Hashlife, vérifiée de bout en bout : depuis le document, pas depuis des
/// occupants fabriqués à la main.
#[test]
fn un_motif_pose_deux_fois_ne_se_peint_qu_une_fois() {
    let mut store = Store::new("essai");
    let board = store.project.active_board_id.clone();
    let cote = Adresse::cote_monde(0);
    // Le même motif dans la case (0, 0) et dans la case (5, 3), à la même place relative.
    for (rang, (tx, ty)) in [(0i64, 0i64), (5, 3)].into_iter().enumerate() {
        for (dx, dy) in [(20.0, 30.0), (90.0, 15.0)] {
            let mut img = BoardImage::new(
                format!("i{rang}-{dx}"),
                tx as f64 * cote + dx,
                ty as f64 * cote + dy,
                40.0,
                40.0,
            );
            img.src = Some("meme-photo.png".into());
            store.add_image(&board, img);
        }
    }

    let rangs: Vec<u32> = (0..4).collect();
    let ici = ce_que_porte(&store, adresse(0, 0), &rangs);
    let ailleurs = ce_que_porte(&store, adresse(5, 3), &rangs);
    assert_ne!(ici, Empreinte::vide());
    assert_eq!(
        ici, ailleurs,
        "le meme motif partout donne la meme empreinte"
    );

    let mut t = Tuiles::nouveau();
    t.ouvrir();
    for e in [ici, ailleurs] {
        if t.deja_peinte(e).is_none() {
            t.ranger(e, carre(COTE));
        }
    }
    assert_eq!(t.peintes(), 1, "un seul rendu pour les deux murs");
}

/// **Toucher une photo change l'empreinte de sa tuile**, et d'elle seule. Sans quoi le cache
/// montrerait un état périmé — le pire défaut qu'un cache puisse avoir.
#[test]
fn deplacer_une_photo_perime_sa_tuile_et_pas_les_autres() {
    let mut store = Store::new("essai");
    let board = store.project.active_board_id.clone();
    let mut img = BoardImage::new("i1", 30.0, 30.0, 50.0, 50.0);
    img.src = Some("photo.png".into());
    store.add_image(&board, img);

    let rangs = [0u32];
    let avant = ce_que_porte(&store, adresse(0, 0), &rangs);
    let voisine_avant = ce_que_porte(&store, adresse(1, 0), &rangs);

    // On la bouge d'un pixel, sans quitter sa case.
    if let Some(b) = store.project.boards.iter_mut().find(|b| b.id == board) {
        b.images[0].x += 1.0;
    }
    assert_ne!(
        ce_que_porte(&store, adresse(0, 0), &rangs),
        avant,
        "sa case doit se perimer"
    );
    assert_eq!(
        ce_que_porte(&store, adresse(1, 0), &rangs),
        voisine_avant,
        "et la case voisine, non"
    );
}

/// **Toute propriété qui change les pixels change l'empreinte.** En oublier une ferait
/// réutiliser un rendu qui n'est pas le bon, et le défaut resterait invisible tant que deux
/// nœuds ne diffèrent pas *seulement* par elle.
#[test]
fn aucune_propriete_visible_n_echappe_a_l_empreinte() {
    let base = {
        let mut img = BoardImage::new("i", 0.0, 0.0, 10.0, 10.0);
        img.src = Some("a.png".into());
        img
    };
    let reference = aspect_de(&base);

    let mut fichier = base.clone();
    fichier.src = Some("b.png".into());
    let mut tourne = base.clone();
    tourne.rotation = 0.1;
    let mut video = base.clone();
    video.is_video = true;
    let mut cadre = base.clone();
    cadre.fit = Some("cover".into());
    let mut verrouille = base.clone();
    verrouille.locked = true;
    let mut domaine = base.clone();
    domaine.domains.push(glucose_core::types::DomainAssignment {
        domain_id: "d1".into(),
        weight: 1.0,
    });

    for (nom, variante) in [
        ("le fichier", fichier),
        ("la rotation", tourne),
        ("la nature video", video),
        ("le cadrage", cadre),
        ("le verrou", verrouille),
        ("le domaine", domaine),
    ] {
        assert_ne!(
            aspect_de(&variante),
            reference,
            "{nom} ne compte pas dans l'aspect"
        );
    }
}

/// Les tuiles retenues couvrent **tout** l'écran : un trou y laisserait une bande vide.
#[test]
fn les_tuiles_retenues_couvrent_tout_l_ecran() {
    let vue = Viewport {
        x: -137.0,
        y: 42.0,
        scale: 1.3,
    };
    let ecran = (1920.0f32, 1080.0f32);
    let (niveau, tuiles) = a_l_ecran(vue, ecran);
    let tuiles: Vec<Adresse> = tuiles.collect();
    assert!(!tuiles.is_empty());

    // Le niveau ne dépasse jamais l'échelle : les tuiles s'agrandissent, elles ne rétrécissent
    // pas — c'est ce qui garde la composition dans le régime mesuré à 1,7 ms.
    assert!(Adresse::echelle(niveau) <= vue.scale);

    for (sx, sy) in [(0.0, 0.0), (960.0, 540.0), (1919.0, 1079.0)] {
        let (wx, wy) = crate::canvas::screen_to_world(sx, sy, &vue);
        assert!(
            tuiles.contains(&Adresse::contenant(niveau, wx, wy)),
            "le point d'ecran ({sx}, {sy}) n'est couvert par aucune tuile"
        );
    }
}

/// Le rectangle du monde qu'une tuile couvre et son origine en pixels disent la même chose :
/// les confondre décalerait tout le contenu d'une tuile entière.
#[test]
fn l_origine_en_pixels_et_le_rectangle_du_monde_s_accordent() {
    for niveau in [-2, 0, 3] {
        let a = Adresse {
            niveau,
            x: 7,
            y: -4,
        };
        let couverte: Rect = a.couvre();
        let (ox, oy) = a.origine_en_pixels();
        let echelle = Adresse::echelle(niveau);
        assert!(
            (couverte.left * echelle - ox).abs() < 1e-6,
            "au niveau {niveau}"
        );
        assert!(
            (couverte.top * echelle - oy).abs() < 1e-6,
            "au niveau {niveau}"
        );
    }
}
