//! Les épreuves de REGION-1 : ce que la liste écarte ne se voyait pas, au bit près.

use super::super::{peindre, Arrondi, Bord, Membrane};
use super::Regions;
use crate::report::Pixel;

/// Un tirage déterministe, sans dépendance : la suite de Lehmer de Park et Miller.
struct Tirage(u64);

impl Tirage {
    fn suivant(&mut self) -> f32 {
        self.0 = self.0 * 48_271 % 2_147_483_647;
        self.0 as f32 / 2_147_483_647.0
    }

    fn entre(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.suivant()
    }
}

/// Un écran petit, pour lire chaque pixel.
const ECRAN: (f32, f32) = (40.0, 30.0);

/// Chaque centre de pixel de l'écran.
fn centres() -> impl Iterator<Item = (f32, f32)> {
    (0..ECRAN.1 as u32)
        .flat_map(|j| (0..ECRAN.0 as u32).map(move |i| (i as f32 + 0.5, j as f32 + 0.5)))
}

/// Un rectangle arrondi tiré autour de l'écran : de la taille d'un pixel à des millions — vu de très
/// près par son coin, il garde des bords loin de l'écran.
fn forme(t: &mut Tirage) -> Arrondi {
    let echelle = 10f32.powf(t.entre(0.0, 6.0));
    let (x, y) = (
        t.entre(-1.5, 1.0) * echelle + 20.0,
        t.entre(-1.5, 1.0) * echelle + 15.0,
    );
    let (l, h) = (t.entre(0.05, 3.0) * echelle, t.entre(0.05, 3.0) * echelle);
    Arrondi::nouveau(x, y, l, h, t.entre(0.0, 0.5) * l.min(h))
}

fn sans_bord(forme: Arrondi) -> Bord {
    Bord {
        forme,
        demi_largeur: 0.0,
        pointille: None,
        alpha: 0.0,
    }
}

/// Une frontière seule : un trait, plein ou pointillé, à l'intérieur du bord ou centré dessus.
fn frontiere(t: &mut Tirage, f: Arrondi) -> Membrane {
    let largeur = 10f32.powf(t.entre(-1.0, 2.0));
    let dedans = t.suivant() < 0.5;
    let forme = if dedans { f.dilate(-largeur / 2.0) } else { f };
    Membrane {
        remplissages: [(f, 0.0); 3],
        bord: Bord {
            forme,
            demi_largeur: largeur / 2.0,
            pointille: (t.suivant() < 0.5).then(|| forme.pointille(largeur * 5.0)),
            alpha: t.entre(0.1, 1.0),
        },
        teinte: [0.9, 0.5, 0.2],
    }
}

/// Un rectangle arrondi dont chaque bord frôle celui de l'écran, à trois pixels près : là où une
/// erreur d'un demi-pixel se verrait.
fn forme_au_bord(t: &mut Tirage) -> Arrondi {
    let (g, h) = (t.entre(-3.0, 1.0), t.entre(-3.0, 1.0));
    let (d, b) = (ECRAN.0 + t.entre(-1.0, 3.0), ECRAN.1 + t.entre(-1.0, 3.0));
    Arrondi::nouveau(g, h, d - g, b - h, t.entre(0.0, 4.0))
}

/// **Un champ qui dit couvrir l'écran y vaut 1 en chaque pixel** : ni un pixel de bord, ni un
/// coin arrondi qui laisserait passer ce qui est dessous — sur des formes de toutes tailles, et
/// sur des formes dont les bords frôlent ceux de l'écran, où les deux réponses arrivent.
#[test]
fn region_1_ce_qui_couvre_l_ecran_y_vaut_un_partout() {
    let mut t = Tirage(7);
    let mut au_bord = [0, 0];
    for i in 0..40_000 {
        let f = if i % 2 == 0 {
            forme(&mut t)
        } else {
            forme_au_bord(&mut t)
        };
        let plein = Membrane::plein(f, [0.2, 0.4, 0.6], 1.0);
        let couvre = plein.couvre(ECRAN);
        if couvre {
            assert!(
                centres().all(|(x, y)| plein.alpha(x, y) == 1.0),
                "{f:?} dit couvrir l'écran, et laisse passer un pixel"
            );
        }
        if i % 2 == 1 {
            au_bord[usize::from(couvre)] += 1;
        }
        let translucide = Membrane::plein(f, [0.2, 0.4, 0.6], 0.99);
        assert!(!translucide.couvre(ECRAN), "un voile ne couvre rien");
    }
    assert!(
        au_bord.iter().all(|&n| n > 2_000),
        "au bord, les deux réponses doivent arriver : {au_bord:?}"
    );
}

/// **Un champ qui dit ne rien toucher ne pose d'encre sur aucun pixel** : une frontière vue de
/// l'intérieur, une forme hors de l'écran. Et le cas d'une frontière dont l'enveloppe contient
/// l'écran sans y rien poser existe vraiment — c'est celui que la règle gagne.
#[test]
fn region_1_ce_qui_ne_touche_pas_l_ecran_n_y_pose_rien() {
    let mut t = Tirage(11);
    let (mut ecartees, mut dedans) = (0, 0);
    for _ in 0..20_000 {
        let f = forme(&mut t);
        let champs = [
            frontiere(&mut t, f),
            Membrane {
                remplissages: [(f.dilate(20.0), 0.03), (f.dilate(10.0), 0.05), (f, 0.0)],
                bord: sans_bord(f),
                teinte: [0.9, 0.5, 0.2],
            },
        ];
        for champ in champs {
            if champ.touche(ECRAN) {
                continue;
            }
            ecartees += 1;
            let e = champ.enveloppe();
            if e.gauche < 0.0 && e.haut < 0.0 && e.droite > ECRAN.0 && e.bas > ECRAN.1 {
                dedans += 1;
            }
            for (x, y) in centres() {
                assert_eq!(champ.alpha(x, y), 0.0, "{champ:?} en ({x}, {y})");
            }
        }
    }
    assert!(
        ecartees > 1_000,
        "l'épreuve doit écarter des champs : {ecartees}"
    );
    assert!(
        dedans > 200,
        "dont des frontières vues de l'intérieur : {dedans}"
    );
}

/// Les trois champs d'une région de la boîte `(x, y, l, h)`.
fn region(x: f32, y: f32, (l, h): (f32, f32), teinte: [f32; 3]) -> ([Membrane; 2], Membrane) {
    let fond = Arrondi::nouveau(x, y, l, h, 24.0);
    let halos = Membrane {
        remplissages: [
            (fond.dilate(20.0), 0.03),
            (fond.dilate(10.0), 0.05),
            (fond, 0.0),
        ],
        bord: sans_bord(fond),
        teinte,
    };
    let corps = Membrane::plein(fond, teinte.map(|c| 0.05 + 0.2 * c), 1.0);
    let trait_ = Membrane {
        remplissages: [(fond, 0.0); 3],
        bord: Bord {
            forme: fond.dilate(-1.5),
            demi_largeur: 1.5,
            pointille: None,
            alpha: 0.22,
        },
        teinte,
    };
    ([halos, corps], trait_)
}

fn peindre_tout(champs: &[&Membrane], (l, h): (u32, u32)) -> Vec<Pixel> {
    let mut image = vec![[13, 13, 13, 255]; (l * h) as usize];
    for m in champs {
        peindre(m, &mut image, l, h);
    }
    image
}

/// **Ce que la liste écarte ne se voyait pas** : des régions emboîtées, vues de loin puis de si
/// près que la plus petite couvre l'écran ; l'image de la liste est celle de tous les champs —
/// tous les sols, puis toutes les frontières —, au bit près. Rien ne se pose sous un corps qui
/// couvre l'écran : les mères sous la fille, leurs frontières loin de l'écran, s'écartent.
#[test]
fn region_1_ce_qui_ne_se_voit_pas_ne_se_pose_pas() {
    let (l, h) = (160, 120);
    let ecran = (l as f32, h as f32);
    for (zoom, ecarte) in [(0.4_f32, false), (1.0, true), (6.0, true), (40.0, true)] {
        // Trois régions emboîtées autour du centre de l'écran, puis une voisine.
        let boites = [
            (-60.0, -40.0, (280.0, 200.0)),
            (-10.0, 0.0, (180.0, 120.0)),
            (40.0, 30.0, (80.0, 60.0)),
            (150.0, 10.0, (60.0, 40.0)),
        ];
        let regions: Vec<_> = boites
            .iter()
            .enumerate()
            .map(|(i, &(x, y, (bl, bh)))| {
                let (cx, cy) = (80.0, 60.0);
                region(
                    cx + (x - cx) * zoom,
                    cy + (y - cy) * zoom,
                    (bl * zoom, bh * zoom),
                    [i as f32 / 4.0, 0.5, 1.0 - i as f32 / 4.0],
                )
            })
            .collect();
        let mut liste = Regions::default();
        for (sol, trait_) in &regions {
            liste.poser(*sol, *trait_, ecran);
        }
        let tous: Vec<&Membrane> = regions
            .iter()
            .flat_map(|(sol, _)| sol.iter())
            .chain(regions.iter().map(|(_, t)| t))
            .collect();
        let poses: Vec<&Membrane> = liste.sol().iter().chain(liste.frontieres()).collect();
        assert_eq!(
            peindre_tout(&poses, (l, h)),
            peindre_tout(&tous, (l, h)),
            "au zoom {zoom}, la liste change l'image"
        );
        // Chaque frontière posée se voit : posées en dernier, rien ne les couvre.
        let vide = peindre_tout(&[], (l, h));
        for f in liste.frontieres() {
            assert_ne!(
                peindre_tout(&[f], (l, h)),
                vide,
                "au zoom {zoom}, une frontière posée ne pose rien"
            );
        }
        // Rien ne se pose sous un corps qui couvre l'écran : un champ qui couvre ne peut être que
        // le premier du sol.
        assert!(
            liste.sol().iter().skip(1).all(|m| !m.couvre(ecran)),
            "au zoom {zoom}, un corps qui couvre l'écran est posé sur d'autres"
        );
        assert_eq!(
            liste.len() < tous.len(),
            ecarte,
            "au zoom {zoom} : {} champs posés sur {}",
            liste.len(),
            tous.len()
        );
    }
}
