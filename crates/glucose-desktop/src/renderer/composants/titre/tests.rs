//! **TITRE-3** : le titre d'une membrane, rendu à part dans sa texture puis reposé, donne les
//! pixels que la voie processeur écrit en place ; sa texture contient toute son encre ; et sa clé
//! ne change pas pendant un zoom.

use super::super::Regime;
use super::{titre_par_la_carte, TitreDuMonde};
use crate::canvas::world_to_screen;
use crate::renderer::scale::WorldScale;
use crate::renderer::{Regard, Renderer};
use crate::typography::{Face, TextStyle};
use glucose_core::types::Viewport;
use tiny_skia::{Color, Pixmap};

const TAILLE: (u32, u32) = (2400, 600);
const BLEU: (u8, u8, u8) = (96, 165, 250);

/// Un titre de ce texte et de ce corps du monde, sur une membrane dont le coin tombe entre deux
/// pixels.
fn titre(texte: &str, corps: f32) -> TitreDuMonde<'_> {
    TitreDuMonde {
        membrane: "m",
        texte,
        coin: (100.3, 260.7),
        mesures: [corps, corps, corps * 18.0 / 16.0],
        teintes: (BLEU, Color::from_rgba8(18, 18, 20, 255)),
    }
}

/// Ce qui sépare le titre en place du titre reposé : le pire écart d'un canal, le nombre de
/// canaux qui diffèrent, et combien d'entre eux tombent **au milieu** d'une surface — un pixel
/// que ses huit voisins, en place, répètent exactement : l'intérieur d'une lettre, de son halo,
/// ou du vide.
struct Ecart {
    pire: u8,
    canaux: usize,
    au_milieu: usize,
}

/// Le pixel `i` de ce tampon et ses huit voisins sont-ils identiques ?
fn au_milieu(p: &Pixmap, i: usize) -> bool {
    let (l, h) = (p.width() as usize, p.height() as usize);
    let (x, y) = (i % l, i / l);
    if x == 0 || y == 0 || x + 1 >= l || y + 1 >= h {
        return false;
    }
    let px = p.pixels();
    (y - 1..=y + 1).all(|v| (x - 1..=x + 1).all(|u| px[v * l + u] == px[i]))
}

/// **Le titre écrit en place** par la voie processeur, et **le même rendu dans sa texture puis
/// reposé** où sa pose le dit — après avoir exigé qu'aucune encre ne touche le bord de la
/// texture.
fn ecart(texte: &str, corps: f32, vp: Viewport) -> Ecart {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let t = titre(texte, corps);
    let s = WorldScale::new(vp.scale, 1.0);
    let (sx, sy) = world_to_screen(t.coin.0, t.coin.1, &vp);
    let [c, r, e] = t.mesures;
    let style = TextStyle {
        size: s.world(c),
        color: Color::from_rgba8(BLEU.0, BLEU.1, BLEU.2, 255),
        face: Face::Bold,
    };
    let mut en_place = Pixmap::new(TAILLE.0, TAILLE.1).expect("pixmap");
    let plume = (sx as f32 + s.world(r), sy as f32 - s.world(e));
    kit.typography.draw_text_with_outline(
        &mut en_place.as_mut(),
        texte,
        plume.0,
        plume.1,
        style,
        t.teintes.1,
    );

    let regime = Regime::de((vp, 1.0), Regard::immobile(), TAILLE, 0.0);
    let composant = regime.titre(kit, &t);
    let texture = composant.rendre(kit).expect("une texture");
    assert_eq!(
        encre_au_bord(&texture),
        0,
        "« {texte} » au corps {} : de l'encre touche le bord de sa texture, elle est coupée",
        s.world(c)
    );
    let (px, py) = (composant.pose.x, composant.pose.y);
    assert_eq!(
        (px, py),
        (px.floor(), py.floor()),
        "à l'arrêt, la pose est entière"
    );
    let mut reposee = Pixmap::new(TAILLE.0, TAILLE.1).expect("pixmap");
    crate::composition::poser(
        &mut reposee.as_mut(),
        &texture,
        (px, py),
        glucose_core::report::Melange::Composer,
    );
    assert!(
        en_place.data().iter().any(|&o| o > 0),
        "« {texte} » à ×{} : le titre en place doit porter de l'encre, sinon l'épreuve ne \
         compare rien",
        vp.scale
    );
    let pixels = || {
        en_place
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .zip(reposee.data().as_chunks::<4>().0)
    };
    let canaux = |(a, b): (&[u8; 4], &[u8; 4])| (0..4).filter(|&k| a[k] != b[k]).count();
    Ecart {
        pire: pixels()
            .flat_map(|(a, b)| (0..4).map(move |k| a[k].abs_diff(b[k])))
            .max()
            .unwrap_or(0),
        canaux: pixels().map(canaux).sum(),
        au_milieu: pixels()
            .enumerate()
            .filter(|&(i, _)| au_milieu(&en_place, i))
            .map(|(_, p)| canaux(p))
            .sum(),
    }
}

/// **L'écart de tiny-skia sous une translation entière**, en niveaux : la même forme posée un
/// nombre entier de pixels plus loin ne reçoit pas toujours la même couverture là où son bord
/// frôle une frontière de sous-pixel — mesuré à 25 sur le trait d'une carte par l'épreuve des
/// deux voies (`tests/voies_suite.rs`, `ECART_ADMIS`). Une lettre **tracée** (plus de 128
/// pixels, GRAND-1) est une telle forme ; une lettre **posée** est notre propre composition,
/// au bit près.
const ECART_DE_TINY_SKIA: u8 = 25;

/// Les pixels encrés du pourtour d'une texture.
fn encre_au_bord(p: &Pixmap) -> usize {
    let (l, h) = (p.width(), p.height());
    (0..l)
        .flat_map(|x| [(x, 0), (x, h - 1)])
        .chain((0..h).flat_map(|y| [(0, y), (l - 1, y)]))
        .filter(|&(x, y)| p.pixel(x, y).is_some_and(|c| c.alpha() > 0))
        .count()
}

/// **TITRE-3 — à l'arrêt, la texture d'un titre est le titre du processeur**, et elle contient
/// toute son encre : des accents en haut, des jambages en bas, un `j` qui déborde à gauche, des
/// lettres tracées (plus de 128 pixels), à des phases sous le pixel. **Au bit près** pour des
/// lettres posées ; pour des lettres tracées, l'écart de tiny-skia, et seulement aux bords.
#[test]
fn titre_3_la_texture_d_un_titre_est_le_titre_du_processeur() {
    // L'émoji s'écrit par son code : FONT-1 exige d'Inter chaque caractère littéral du crate, et
    // celui-ci, Noto Emoji le dessine.
    let vague = char::from_u32(0x1F30A).expect("la vague");
    let textes = [
        "Hydrologie".to_string(),
        "Économie d'Ågård — jpqy".to_string(),
        "jungle".to_string(),
        format!("Océan {vague}"),
    ];
    // Le coin de la membrane tombe au même endroit de l'écran à chaque échelle, entre deux
    // pixels : le titre, au-dessus, reste dans le tampon.
    let vues = [
        (1.0, (120.5, 410.25)),
        (0.37, (113.37, 402.19)),
        (2.3, (98.4, 433.3)),
    ];
    for texte in &textes {
        for corps in [16.0_f32, 23.7, 61.0] {
            for (scale, (ex, ey)) in vues {
                let (cx, cy) = titre(texte, corps).coin;
                let vp = Viewport {
                    scale,
                    x: ex - cx * scale,
                    y: ey - cy * scale,
                };
                let e = ecart(texte, corps, vp);
                let cas = format!("« {texte} », corps {corps} à l'échelle {scale}");
                if crate::typography::trace::se_trace(corps * scale as f32) {
                    assert!(
                        e.pire <= ECART_DE_TINY_SKIA && e.au_milieu == 0,
                        "{cas}, tracé : pire écart {}, {} canaux différents au milieu d'une \
                         surface",
                        e.pire,
                        e.au_milieu
                    );
                } else {
                    assert!(
                        e.pire == 0 && e.canaux == 0,
                        "{cas} : pire écart {}, {} canaux différents",
                        e.pire,
                        e.canaux
                    );
                }
            }
        }
    }
}

/// **TITRE-3 — pendant un zoom, la texture d'un titre garde sa clé tant que le palier ne change
/// pas** : rendue au palier, sans phase, d'un corps du monde. Une clé qui changerait à chaque
/// image referait la texture à chaque image — le coût qu'elle devait retirer.
#[test]
fn titre_3_sa_cle_ne_change_pas_pendant_un_zoom() {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let t = titre("Hydrologie", 23.7);
    let en_zoom = Regard {
        degradation_permise: true,
        en_mouvement: true,
        translation_entiere: None,
    };
    let cle = |scale: f64| {
        let vp = Viewport {
            scale,
            x: 17.3,
            y: 5.9,
        };
        Regime::de((vp, 1.0), en_zoom, TAILLE, 0.0)
            .titre(kit, &t)
            .cle
    };
    assert_eq!(cle(0.30), cle(0.33), "le même palier, la même texture");
    assert_ne!(
        cle(0.30),
        cle(0.60),
        "un autre palier, une autre texture : sinon l'épreuve ne regarde pas la clé"
    );
}

/// **TITRE-3 — un titre se pose par la carte s'il tient dans la fenêtre au plus grand palier**
/// où il peut se rendre ; sinon, il reste au processeur.
#[test]
fn titre_3_un_titre_plus_grand_que_la_fenetre_reste_au_processeur() {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let titre = ("Hydrologie", 16.0);
    assert!(titre_par_la_carte(kit, titre, 1.0, TAILLE));
    // À ×64, ses lettres font mille pixels : sa texture ne tient pas dans la fenêtre.
    assert!(!titre_par_la_carte(kit, titre, 64.0, TAILLE));
    // Le palier compte : à ×1,45, il se rend en mouvement au palier ×2, plus grand que la vue.
    let (largeur, _) = kit
        .typography
        .measure_text("Hydrologie", 16.0 * 1.45, Face::Bold);
    let juste = ((largeur + 30.0) as u32, TAILLE.1);
    assert!(
        !titre_par_la_carte(kit, titre, 1.45, juste),
        "sa texture au palier ×2 dépasse une fenêtre où il tient à ×1,45"
    );
}

/// La boîte de l'encre franche d'un tampon — les pixels au moins à moitié couverts :
/// `(gauche, haut, droite, bas)`, bornes incluses.
fn encre_franche(p: &Pixmap) -> (i64, i64, i64, i64) {
    let l = p.width() as usize;
    let mut boite = (i64::MAX, i64::MAX, i64::MIN, i64::MIN);
    for (i, c) in p.pixels().iter().enumerate() {
        if c.alpha() >= 128 {
            let (x, y) = ((i % l) as i64, (i / l) as i64);
            boite = (
                boite.0.min(x),
                boite.1.min(y),
                boite.2.max(x),
                boite.3.max(y),
            );
        }
    }
    boite
}

/// **TITRE-3 — en mouvement, le titre reste à sa place** : rendu au palier, sans phase, posé à
/// sa place fractionnaire et agrandi par la carte, son encre tombe là où la voie processeur
/// l'écrit à l'échelle de la vue.
///
/// À ×1,4, la texture se rend au palier ×1 et la carte l'agrandit de 1,4 : la plume doit tomber
/// au même point, et le coin de la texture se mesurer à l'échelle où elle est rendue. L'écart
/// admis est de deux pixels : la cellule où une lettre tombe à chacune des deux échelles (un
/// pixel), et le halo de 1,5 pixel agrandi avec la texture, qu'adoucit le filtre.
#[test]
fn titre_3_en_mouvement_le_titre_reste_a_sa_place() {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let t = titre("Hydrologie", 24.0);
    let (cx, cy) = t.coin;
    let scale = 1.4;
    let vp = Viewport {
        scale,
        x: 120.37 - cx * scale,
        y: 410.6 - cy * scale,
    };
    let s = WorldScale::new(scale, 1.0);
    let (sx, sy) = world_to_screen(cx, cy, &vp);
    let [c, r, e] = t.mesures;
    let style = TextStyle {
        size: s.world(c),
        color: Color::from_rgba8(BLEU.0, BLEU.1, BLEU.2, 255),
        face: Face::Bold,
    };
    let mut en_place = Pixmap::new(TAILLE.0, TAILLE.1).expect("pixmap");
    let plume = (sx as f32 + s.world(r), sy as f32 - s.world(e));
    kit.typography.draw_text_with_outline(
        &mut en_place.as_mut(),
        t.texte,
        plume.0,
        plume.1,
        style,
        t.teintes.1,
    );

    let en_zoom = Regard {
        degradation_permise: true,
        en_mouvement: true,
        translation_entiere: None,
    };
    let composant = Regime::de((vp, 1.0), en_zoom, TAILLE, 0.0).titre(kit, &t);
    let texture = composant.rendre(kit).expect("une texture");
    let pose = composant.pose;
    let agrandi = (
        pose.largeur / texture.width() as f32,
        pose.hauteur / texture.height() as f32,
    );
    assert!(
        (agrandi.0 - 1.4).abs() < 1e-3,
        "la texture se rend au palier ×1 : la carte l'agrandit de 1,4 ({agrandi:?})"
    );
    let mut posee = Pixmap::new(TAILLE.0, TAILLE.1).expect("pixmap");
    posee.draw_pixmap(
        0,
        0,
        texture.as_ref(),
        &tiny_skia::PixmapPaint {
            quality: tiny_skia::FilterQuality::Bilinear,
            ..tiny_skia::PixmapPaint::default()
        },
        tiny_skia::Transform::from_row(agrandi.0, 0.0, 0.0, agrandi.1, pose.x, pose.y),
        None,
    );
    let (a, b) = (encre_franche(&en_place), encre_franche(&posee));
    let ecarts = [b.0 - a.0, b.1 - a.1, b.2 - a.2, b.3 - a.3];
    assert!(
        ecarts.iter().all(|d| d.abs() <= 2),
        "en mouvement, le titre se déplace : l'encre en place {a:?}, posée {b:?}"
    );
}
