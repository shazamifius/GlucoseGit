//! **Ce que la passe des formes doit garantir** : les pixels des deux lois du noyau — Slug
//! (`courbes::couverture`) et les membranes (`membrane_forme`) —, à l'arrondi de l'écriture près.
//!
//! La référence compose comme la carte : chaque primitive à son tour, en prémultiplié, l'image
//! écrite sur huit bits après chacune.

use super::*;
use crate::present::banc_gpu;
use crate::typography::{Face, Typography};
use glucose_core::courbes::couverture::couverture;
use glucose_core::membrane_forme::{couverture_d_un_plein, couverture_d_un_trait};
use tiny_skia::Pixmap;

/// La borne de l'écart entre la carte et la loi, en niveaux sur 255 — **mesurée, pas choisie** :
/// un niveau, l'arrondi d'une égalité et de quelques ulps (`sqrt`, la division) que la carte
/// calcule avec ses circuits. Le point de la forme sous chaque pixel se calcule des deux côtés
/// par l'inverse de la matrice ; l'interpoler depuis les coins du quad écartait de dix-huit
/// niveaux une lettre de 2 400 pixels, là où un rayon frôle la courbe (fiche 73).
const ECART_ADMIS: u8 = 1;

/// Ce que la **carte** rend sur un fond noir opaque.
fn par_la_carte(taille: (u32, u32), formes: &Formes) -> Option<Pixmap> {
    par_la_carte_apres(taille, &Formes::default(), formes)
}

/// Ce que la carte rend de `formes`, l'image d'**après** `avant` — son atlas déjà rempli.
fn par_la_carte_apres(taille: (u32, u32), avant: &Formes, formes: &Formes) -> Option<Pixmap> {
    let (peripherique, file) = banc_gpu::carte()?;
    let mut passe_formes = FormesGpu::nouvelles(&peripherique, banc_gpu::FORMAT);
    let ecran = (taille.0 as f32, taille.1 as f32);
    passe_formes.preparer((&peripherique, &file), ecran, avant);
    passe_formes.preparer((&peripherique, &file), ecran, formes);
    let cible = banc_gpu::cible(&peripherique, taille);
    let vue = cible.create_view(&Default::default());
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    banc_gpu::passe(&mut encodeur, &vue, wgpu::Color::BLACK, |p| {
        for s in 0..formes.suites.len() {
            passe_formes.poser(p, s);
        }
    });
    file.submit(Some(encodeur.finish()));
    banc_gpu::relire(&peripherique, &file, &cible, taille)
}

/// La part du pixel de centre `p` qu'une primitive couvre, par la loi du noyau.
fn couverture_de(atlas: &mut Atlas, primitive: &Primitive, p: [f32; 2]) -> f32 {
    let dans_la_rogne = {
        let r = primitive.rogne;
        let x = ((p[0] + 0.5).min(r[2]) - (p[0] - 0.5).max(r[0])).clamp(0.0, 1.0);
        let y = ((p[1] + 0.5).min(r[3]) - (p[1] - 0.5).max(r[1])).clamp(0.0, 1.0);
        x * y
    };
    let c = match &primitive.genre {
        Genre::Arrondi { forme, demi } => {
            let d = forme.distance(p[0], p[1]);
            if *demi > 0.0 {
                couverture_d_un_trait(d.abs(), *demi)
            } else {
                couverture_d_un_plein(d)
            }
        }
        Genre::Courbes { source, matrice: m } => {
            let Some(e) = atlas.emballage(source) else {
                return 0.0;
            };
            // L'inverse de la matrice : le point de la forme sous ce pixel (RACINES-1).
            let i = inverse(*m);
            let (x, y) = (p[0] - m[4], p[1] - m[5]);
            let q = [i[0] * x + i[2] * y, i[1] * x + i[3] * y];
            let ppu = [m[0].hypot(m[1]), m[2].hypot(m[3])];
            couverture(&e, q, ppu)
        }
    };
    c * dans_la_rogne
}

/// Ce que **la loi** rend : chaque primitive composée à son tour, en prémultiplié.
fn par_la_loi(taille: (u32, u32), formes: &Formes) -> Pixmap {
    let mut image = Pixmap::new(taille.0, taille.1).expect("une image");
    image.fill(tiny_skia::Color::BLACK);
    let mut atlas = Atlas::default();
    let largeur = taille.0 as usize;
    for primitive in formes
        .suites
        .iter()
        .flat_map(|s| &formes.primitives[s.primitives.clone()])
    {
        let donnees = image.data_mut();
        for j in 0..taille.1 as usize {
            for i in 0..largeur {
                let p = [i as f32 + 0.5, j as f32 + 0.5];
                let c = couverture_de(&mut atlas, primitive, p);
                if c <= 0.0 {
                    continue;
                }
                let k = 4 * (j * largeur + i);
                let source = primitive.couleur.map(|v| v * c);
                for canal in 0..4 {
                    let dessous = f32::from(donnees[k + canal]) / 255.0;
                    let v = source[canal] + dessous * (1.0 - source[3]);
                    donnees[k + canal] = (v * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    image
}

/// Une suite de ces primitives.
fn formes(primitives: Vec<Primitive>) -> Formes {
    let n = primitives.len();
    Formes {
        primitives,
        suites: vec![Suite {
            tranche: Tranche::Cartes,
            avant: 0,
            primitives: 0..n,
        }],
    }
}

/// Un glyphe de ce caractère, à ce corps, sa plume et sa ligne de base en `at`.
fn lettre(typo: &Typography, (ch, face): (char, Face), at: (f32, f32), corps: f32) -> Primitive {
    let (police, indice) = typo.source(ch, face);
    Primitive {
        genre: Genre::Courbes {
            source: Source::Glyphe(police, indice),
            matrice: [corps, 0.0, 0.0, -corps, at.0, at.1],
        },
        couleur: [1.0, 1.0, 1.0, 1.0],
        rogne: PARTOUT,
    }
}

/// Compare la carte à la loi ; rend le nombre de pixels que la forme a touchés.
fn comparer(nom: &str, taille: (u32, u32), f: &Formes) -> Option<usize> {
    let Some(carte) = par_la_carte(taille, f) else {
        eprintln!("aucune carte utilisable : test saute");
        banc_gpu::sans_carte();
        return None;
    };
    let loi = par_la_loi(taille, f);
    let pire = banc_gpu::pire_ecart(&loi, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "{nom} : la carte s'écarte de la loi de {pire} niveaux (admis {ECART_ADMIS})"
    );
    Some(loi.pixels().iter().filter(|p| p.red() > 0).count())
}

/// **Une lettre dessinée par la carte suit la loi de Slug** — droite et italique (des cubiques,
/// CUBIQUE-1), un emoji (Noto), à un corps fractionnaire et à une place fractionnaire, petite
/// et immense.
#[test]
fn test_une_lettre_par_la_carte_suit_la_loi_de_slug() {
    let typo = Typography::new();
    let primitives = vec![
        lettre(&typo, ('g', Face::Regular), (10.25, 60.5), 48.0),
        lettre(&typo, ('@', Face::Regular), (70.6, 30.3), 13.7),
        lettre(&typo, ('S', Face::Italic), (100.0, 150.0), 140.0),
        lettre(&typo, ('\u{2764}', Face::Regular), (20.0, 150.0), 64.0),
        // Immense : on n'en voit qu'un morceau, et un pixel y vaut un deux-millième d'em.
        lettre(&typo, ('O', Face::Bold), (-900.0, 1500.0), 2400.0),
    ];
    let touches = comparer("lettres", (240, 180), &formes(primitives));
    if let Some(n) = touches {
        assert!(
            n > 2000,
            "les lettres n'ont presque rien couvert ({n} pixels)"
        );
    }
}

/// **Un arrondi dessiné par la carte suit la loi des membranes** : plein, en anneau, en disque.
#[test]
fn test_un_arrondi_par_la_carte_suit_la_loi_des_membranes() {
    let plein = |forme, demi, couleur| Primitive {
        genre: Genre::Arrondi { forme, demi },
        couleur,
        rogne: PARTOUT,
    };
    let primitives = vec![
        plein(
            Arrondi::nouveau(10.3, 12.7, 150.0, 90.5, 24.0),
            0.0,
            [0.03, 0.03, 0.05, 1.0],
        ),
        plein(
            Arrondi::nouveau(30.0, 30.0, 80.25, 20.5, 4.0).dilate(3.0),
            0.75,
            [0.4, 0.3, 0.0, 0.5],
        ),
        plein(
            Arrondi::nouveau(170.5, 20.0, 9.0, 9.0, 4.5),
            0.0,
            [0.7, 0.7, 0.7, 0.8],
        ),
    ];
    comparer("arrondis", (200, 120), &formes(primitives));
}

/// **Ce qui rogne borne la forme** : hors de sa boîte, rien — et la carte le dit comme la loi.
#[test]
fn test_ce_qui_rogne_borne_la_forme() {
    let typo = Typography::new();
    let mut a = lettre(&typo, ('W', Face::Regular), (5.0, 90.0), 100.0);
    a.rogne = [20.5, 10.0, 70.25, 60.0];
    let Some(carte) = par_la_carte((120, 100), &formes(vec![a.clone()])) else {
        return;
    };
    for (k, p) in carte.pixels().iter().enumerate() {
        let (i, j) = ((k % 120) as f32, (k / 120) as f32);
        if !(19.5..=70.75).contains(&(i + 0.5)) || !(9.5..=60.5).contains(&(j + 0.5)) {
            assert_eq!(p.red(), 0, "un pixel hors de la rogne en ({i}, {j})");
        }
    }
    comparer("rognee", (120, 100), &formes(vec![a]));
}

/// **L'atlas se resserre sans rien perdre** : ce qui a servi garde ses courbes et ses bandes,
/// au bit près, à sa nouvelle place ; ce qui n'a pas servi part.
#[test]
fn test_l_atlas_se_resserre_sans_rien_perdre() {
    let typo = Typography::new();
    let mut atlas = Atlas::default();
    let sources: Vec<Source> = "abcdefgh"
        .chars()
        .map(|c| {
            let (p, i) = typo.source(c, Face::Regular);
            Source::Glyphe(p, i)
        })
        .collect();
    atlas.ouvrir();
    for s in &sources {
        atlas.ranger(s);
    }
    let avant = atlas.emballage(&sources[6]).expect("le g");
    // L'image suivante ne demande que le g : sept formes ne servent plus, et pèsent plus que lui.
    atlas.ouvrir();
    atlas.ranger(&sources[6]);
    assert!(atlas.resserrer(), "l'atlas aurait dû se resserrer");
    assert_eq!(atlas.combien(), 1);
    assert_eq!(atlas.envoyees, (0, 0), "tout doit repartir vers la carte");
    assert_eq!(atlas.emballage(&sources[6]).expect("le g"), avant);
    assert_eq!(atlas.courbes.len(), avant.courbes.len());
    // Et il ne se resserre pas quand ce qui sert pèse plus que ce qui ne sert pas.
    atlas.ouvrir();
    atlas.ranger(&sources[6]);
    atlas.ranger(&sources[0]);
    atlas.ouvrir();
    atlas.ranger(&sources[6]);
    assert!(!atlas.resserrer());
}

/// **Un atlas qui grandit entre deux images renvoie tout** : ses tampons refaits sont vides, et
/// la lettre de l'image d'avant y manquerait.
#[test]
fn test_un_atlas_qui_grandit_renvoie_tout() {
    let typo = Typography::new();
    let a = formes(vec![lettre(
        &typo,
        ('g', Face::Regular),
        (10.0, 60.0),
        48.0,
    )]);
    let mut toutes = vec![lettre(&typo, ('g', Face::Regular), (10.0, 60.0), 48.0)];
    for (k, c) in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars().enumerate() {
        toutes.push(lettre(
            &typo,
            (c, Face::Italic),
            (70.0 + (k % 9) as f32 * 18.0, 40.0 + (k / 9) as f32 * 30.0),
            24.0,
        ));
    }
    let b = formes(toutes);
    let Some(carte) = par_la_carte_apres((240, 140), &a, &b) else {
        return;
    };
    let loi = par_la_loi((240, 140), &b);
    let pire = banc_gpu::pire_ecart(&loi, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "apres avoir grandi, la carte s'ecarte de {pire} niveaux"
    );
}
