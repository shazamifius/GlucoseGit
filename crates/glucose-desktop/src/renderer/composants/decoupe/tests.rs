//! Ce que DE-PRES-1 garantit, prouvé sur les pixels : une carte plus grande que l'écran se
//! découpe en tuiles qui, reposées, donnent la carte dessinée en place ; seules les visibles
//! existent ; chacune porte dans sa gouttière ce que sa voisine montre, et un repli qui montre
//! le même morceau de la carte.

use super::super::*;
use crate::renderer::Renderer;
use glucose_core::tuile::COTE;
use tiny_skia::IntRect;

/// L'écran des épreuves : petit, pour qu'une carte le dépasse dès l'échelle quatre.
const ECRAN: (u32, u32) = (500, 300);
/// La carte témoin : son origine et sa taille dans le monde.
const CARTE: (f64, f64, f32, f32) = (60.3, 40.7, 240.0, 60.0);
const CORPS: &str = "Accents : éàçù — une ligne assez longue pour en faire deux.
Et une troisieme.";
const TEINTE: (u8, u8, u8) = (96, 165, 250);
/// L'échelle des épreuves : la carte y fait près de trois écrans de large.
const DE_PRES: f64 = 6.0;

/// **Une sous-ligne de suréchantillonnage** : `tiny-skia` calcule quatre sous-lignes par
/// pixel (`SUPERSAMPLE_SHIFT = 2`), et une seule qui bascule change la couverture d'un quart
/// — 64 niveaux sur 256. C'est le plus grand écart que l'arrondi d'un bord peut produire ;
/// au-delà, ce ne serait plus un arrondi.
const UNE_SOUS_LIGNE: u8 = 64;

/// La vue qui pose le coin de la carte en `(150,37 ; 80,61)` à l'écran, à cette échelle : une
/// phase sous-pixel quelconque, et le coin haut-gauche visible.
fn vue(echelle: f64) -> Viewport {
    Viewport {
        scale: echelle,
        x: 150.37 - CARTE.0 * echelle,
        y: 80.61 - CARTE.1 * echelle,
    }
}

/// Ce que la carte témoin devient à l'écran, à l'arrêt.
fn pieces(renderer: &Renderer, echelle: f64, selectionnee: bool) -> Pieces {
    Regime::de((vue(echelle), 1.0), Regard::immobile(), ECRAN, 0.0)
        .carte(
            renderer.kit(),
            "c",
            CARTE,
            (CORPS, TEINTE, selectionnee),
            None,
        )
        .expect("la carte touche l'ecran")
}

/// La carte témoin dessinée **en place**, comme la voie processeur la dessine.
fn en_place(kit: PaintKit<'_>, vp: Viewport, selectionnee: bool) -> Pixmap {
    let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("pixmap");
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        tints: kit.tints,
        theme: kit.theme,
        vp,
        scale: WorldScale::new(vp.scale, 1.0),
        clip: Clip {
            width: ECRAN.0 as f32,
            height: ECRAN.1 as f32,
            top: 0.0,
        },
    };
    draw_card_contenu(
        &ctx,
        &mut pixmap.as_mut(),
        TextCard {
            origin: (CARTE.0, CARTE.1),
            size: (CARTE.2, CARTE.3),
            body: CORPS,
            tint: TEINTE,
            selected: selectionnee,
            editing: None,
        },
    );
    pixmap
}

/// **Le contour de la carte à l'écran** : la bande de part et d'autre de son bord, large de
/// la marge que sa texture lui réserve, et ses quatre coins arrondis.
fn contour(kit: PaintKit<'_>, vp: Viewport) -> impl Fn(f32, f32) -> bool {
    let lignes =
        card_text_layout(kit.typography, kit.math, CORPS, CARTE.2, TextMode::Rendered).line_count();
    let carte =
        CardLayout::text_card(CARTE.2, CARTE.3, lignes).scaled(WorldScale::new(vp.scale, 1.0));
    let anneau = WorldScale::new(vp.scale, 1.0).screen(SELECTION_RING);
    let marge = (carte.border.max(anneau) / 2.0).ceil() + 1.0;
    let (sx, sy) = world_to_screen(CARTE.0, CARTE.1, &vp);
    let (sx, sy, l, h, r) = (
        sx as f32,
        sy as f32,
        carte.width,
        carte.height,
        carte.radius,
    );
    move |x, y| {
        let dans = |m: f32| x >= sx - m && x < sx + l + m && y >= sy - m && y < sy + h + m;
        let coin = (x < sx + r + marge || x >= sx + l - r - marge)
            && (y < sy + r + marge || y >= sy + h - r - marge);
        dans(marge) && (!dans(-marge) || coin)
    }
}

fn texel(p: &Pixmap, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * p.width() + x) * 4) as usize;
    let d = p.data();
    [d[i], d[i + 1], d[i + 2], d[i + 3]]
}

/// **Une carte qui dépasse l'écran se découpe, et garde un repli qui, lui, y tient.**
///
/// La même carte vue de plus loin reste une texture unique : le découpage ne se déclenche qu'à
/// la limite qui le justifie. Et une photo en chemin suit la même loi.
#[test]
fn test_une_carte_qui_depasse_l_ecran_se_decoupe_et_garde_un_repli() {
    let renderer = Renderer::new();
    let _ = pieces(&renderer, 1.0, false).seule();

    let de_pres = pieces(&renderer, DE_PRES, false);
    let n = de_pres.posees.len() as u32;
    // Une rangée ou une colonne à cheval sur le bord en ajoute une de chaque côté, au plus.
    let plafond = (ECRAN.0.div_ceil(COTE) + 1) * (ECRAN.1.div_ceil(COTE) + 1);
    assert!(
        n > 1 && n <= plafond,
        "{n} tuiles pour un ecran qui en montre au plus {plafond}"
    );
    for t in &de_pres.posees {
        assert!(
            t.pixels.0 <= COTE + 2 && t.pixels.1 <= COTE + 2,
            "une tuile ne depasse pas son cote et sa gouttiere"
        );
        assert!(t.repli.is_some(), "chaque tuile porte son repli");
    }
    let repli = de_pres.repli.expect("un repli");
    assert!(repli.tient_dans(ECRAN), "le repli tient dans l'ecran");
    assert_eq!(
        repli.identite, "carte:c",
        "il porte l'identite de la carte entiere : la texture d'avant le decoupage le sert"
    );
    assert_eq!(
        repli.echelle, 2.0,
        "le plus haut palier dyadique ou la carte tient : a quatre, 960 pixels depassent 500"
    );

    let photo = BoardImage::new("p".to_string(), 150.0, 100.0, 180.0, 120.0);
    let vp = Viewport {
        scale: DE_PRES,
        x: 250.0 - 150.0 * DE_PRES,
        y: 150.0 - 100.0 * DE_PRES,
    };
    let en_chemin = Regime::de((vp, 1.0), Regard::immobile(), ECRAN, 0.0)
        .photo_en_chemin(&photo)
        .expect("la photo touche l'ecran");
    assert!(
        en_chemin.posees.len() > 1 && en_chemin.repli.is_some(),
        "une photo en chemin vue de pres se decoupe aussi"
    );
}

/// **Des tuiles reposées donnent la carte dessinée en place**, hormis son contour — et là,
/// d'au plus une sous-ligne.
///
/// Le texte et le fond sont identiques au bit près : pas une couture aux frontières des
/// tuiles. Le contour — liseré et arrondi des coins — diffère d'un cran de couverture, et la
/// texture entière d'avant avait **le même** écart contre le rendu en place dès l'échelle deux :
/// `tiny-skia` arrondit ses bords en virgule fixe selon leur position absolue (voir
/// `test_une_carte_selectionnee_se_repose_a_un_cran_de_couverture_pres`).
#[test]
fn test_des_tuiles_reposees_donnent_la_carte_en_place_hormis_son_contour() {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let vp = vue(DE_PRES);
    let contour = contour(kit, vp);
    for selectionnee in [false, true] {
        let en_place = en_place(kit, vp, selectionnee);
        let mut reposee = Pixmap::new(ECRAN.0, ECRAN.1).expect("pixmap");
        for t in pieces(&renderer, DE_PRES, selectionnee).posees {
            assert_eq!(
                (t.pose.x, t.pose.y),
                (t.pose.x.floor(), t.pose.y.floor()),
                "a l'arret, une tuile se pose sur un pixel entier"
            );
            let (l, h) = (t.pixels.0 - 2, t.pixels.1 - 2);
            assert_eq!(
                t.pose.fenetre,
                [
                    1.0 / t.pixels.0 as f32,
                    1.0 / t.pixels.1 as f32,
                    l as f32 / t.pixels.0 as f32,
                    h as f32 / t.pixels.1 as f32,
                ],
                "la fenetre montre le centre, sans la gouttiere"
            );
            // La gouttière se retire ici comme le nuanceur la retire : par la fenêtre.
            let centre = t
                .rendre(kit)
                .and_then(|p| p.clone_rect(IntRect::from_xywh(1, 1, l, h)?))
                .expect("le centre de la tuile");
            crate::composition::poser(
                &mut reposee.as_mut(),
                &centre,
                (t.pose.x, t.pose.y),
                glucose_core::report::Melange::Composer,
            );
        }
        let mut pire = 0u8;
        let pixels = en_place.data().chunks(4).zip(reposee.data().chunks(4));
        for (i, (a, b)) in pixels.enumerate() {
            let ecart = a
                .iter()
                .zip(b)
                .map(|(u, v)| u.abs_diff(*v))
                .max()
                .unwrap_or(0);
            if ecart <= 1 {
                continue;
            }
            let (x, y) = ((i as u32 % ECRAN.0) as f32, (i as u32 / ECRAN.0) as f32);
            assert!(
                contour(x, y),
                "selectionnee {selectionnee} : {ecart} niveaux en ({x}, {y}), hors du contour \
                 -- une couture, ou un morceau faux"
            );
            pire = pire.max(ecart);
        }
        assert!(
            pire <= UNE_SOUS_LIGNE,
            "selectionnee {selectionnee} : {pire} niveaux sur le contour, plus qu'une sous-ligne"
        );
    }
}

/// **Une tuile porte dans sa gouttière ce que sa voisine montre** : c'est ce qui empêche une
/// couture pendant le geste, quand le filtre bilinéaire lit le texel d'à côté.
///
/// Comparé hors du contour, où deux tuiles voisines montrent le même texel au bit près.
#[test]
fn test_la_gouttiere_d_une_tuile_porte_les_texels_de_sa_voisine() {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let vp = vue(DE_PRES);
    let contour = contour(kit, vp);
    let tuiles = pieces(&renderer, DE_PRES, false).posees;
    let mut comparees = 0;
    for gauche in &tuiles {
        let largeur = (gauche.pixels.0 - 2) as f32;
        let Some(droite) = tuiles
            .iter()
            .find(|t| t.depart.1 == gauche.depart.1 && t.depart.0 == gauche.depart.0 + largeur)
        else {
            continue;
        };
        let (a, b) = (
            gauche.rendre(kit).expect("la tuile de gauche"),
            droite.rendre(kit).expect("la tuile de droite"),
        );
        // La dernière colonne de la gauche est sa gouttière : le premier texel que la droite
        // montre, qui est sa colonne un.
        let colonne = gauche.pixels.0 - 1;
        for y in 0..gauche.pixels.1 {
            let a_l_ecran = (gauche.pose.x + largeur, gauche.pose.y + y as f32 - 1.0);
            if contour(a_l_ecran.0, a_l_ecran.1) {
                continue;
            }
            assert_eq!(
                texel(&a, colonne, y),
                texel(&b, 1, y),
                "rangee {y} : la gouttiere ne porte pas ce que la voisine montre"
            );
            comparees += 1;
        }
    }
    assert!(
        comparees > 0,
        "l'epreuve doit comparer au moins une rangee hors du contour"
    );
}

/// **Le repli d'une tuile montre le même morceau de la carte**, à la même place.
///
/// Lu des deux côtés : le bord de la tuile ramené de l'écran au monde, et le bord de sa fenêtre
/// dans le repli ramené de la texture au monde. Une erreur de marge ou de phase décalerait le
/// repli d'autant pendant l'image où il remplace la tuile.
#[test]
fn test_le_repli_d_une_tuile_montre_le_meme_morceau_de_la_carte() {
    let renderer = Renderer::new();
    let vp = vue(DE_PRES);
    let de_pres = pieces(&renderer, DE_PRES, false);
    let repli = de_pres.repli.expect("un repli");
    for t in &de_pres.posees {
        let r = t.repli.as_ref().expect("son repli");
        assert_eq!(
            (r.identite.as_str(), r.cle.as_str()),
            (repli.identite.as_str(), repli.cle.as_str())
        );
        assert_eq!(
            (r.pose.x, r.pose.y, r.pose.largeur, r.pose.hauteur),
            (t.pose.x, t.pose.y, t.pose.largeur, t.pose.hauteur),
            "le repli se pose exactement a la place de la tuile"
        );
        for axe in 0..2 {
            let (pose, taille, vue, origine) = if axe == 0 {
                (t.pose.x, t.pose.largeur, vp.x, CARTE.0)
            } else {
                (t.pose.y, t.pose.hauteur, vp.y, CARTE.1)
            };
            let texels = if axe == 0 {
                repli.pixels.0
            } else {
                repli.pixels.1
            };
            let phase = if axe == 0 {
                repli.phase.0
            } else {
                repli.phase.1
            };
            let par_l_ecran = (f64::from(pose) - vue) / vp.scale;
            let debut = r.pose.fenetre[axe] * texels as f32;
            let par_le_repli = origine + f64::from(debut - repli.marge - phase) / repli.echelle;
            assert!(
                (par_l_ecran - par_le_repli).abs() < 1e-3,
                "axe {axe} : la tuile commence en {par_l_ecran} dans le monde, son repli en \
                 {par_le_repli}"
            );
            let largeur = f64::from(r.pose.fenetre[axe + 2] * texels as f32) / repli.echelle;
            assert!(
                (f64::from(taille) / vp.scale - largeur).abs() < 1e-3,
                "axe {axe} : la fenetre du repli n'a pas la taille de la tuile"
            );
        }
    }
}
