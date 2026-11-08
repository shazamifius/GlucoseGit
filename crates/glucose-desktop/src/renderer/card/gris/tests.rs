//! GRIS-1 — de loin, un nœud montre le gris de ses lignes, et ce gris est ce que ses lettres
//! couvrent : la moyenne de son dessin de près. Un nœud de texte, un post-it, et le dessin
//! vectoriel.

use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, Viewport};

const TAILLE: (u32, u32) = (900, 700);
/// Le nœud : 600 × 400 du monde, son coin à (0, 0).
const NOEUD: (f64, f64) = (600.0, 400.0);

/// Un texte qui remplit le nœud, et porte chaque ornement : puces, numéros, citation, trait, bloc
/// de code — de loin, chacun se pose tel quel ou montre son gris.
const TEXTE: &str = "# L'eau\n\nL'eau est un composé chimique ubiquitaire sur la Terre, essentiel \
pour tous les organismes vivants connus. Elle se trouve en général dans son état liquide et \
possède à température ambiante des propriétés uniques : c'est notamment un solvant efficace.\n\n\
- une molécule polaire\n- trois états\n- un cycle\n\n1. la glace\n2. l'eau liquide\n\n\
> L'eau est la force motrice de toute la nature.\n\n---\n\n```\nH2O = 2 H + O\n```\n\n\
Elle couvre environ soixante et onze pour cent de la surface de la Terre, surtout dans les \
océans, et elle circule sans cesse entre la mer, l'air et la terre.";

/// **Ce qu'un pixel de 8 bits garantit** : chaque image s'écarte de sa couleur exacte de moins
/// d'un niveau par pixel et par canal, donc sa moyenne aussi ; deux moyennes, de moins de deux.
/// Le gris est la moyenne exacte des lettres (leur encre sur leur place) : la borne ne vient que
/// de là, aucun pourcentage n'est choisi.
const DEUX_IMAGES_DE_8_BITS: f64 = 2.0;

/// Un nœud de texte de 600 × 400, son coin à l'origine.
fn texte(corps: &str) -> Annotation {
    let mut t = Annotation::text("t", 0.0, 0.0, corps);
    if let Annotation::Text { width, height, .. } = &mut t {
        (*width, *height) = (Some(NOEUD.0), Some(NOEUD.1));
    }
    t
}

/// Un post-it de 600 × 400, son coin à l'origine.
fn post_it(corps: &str) -> Annotation {
    let mut p = Annotation::sticky("t", 0.0, 0.0, corps);
    if let Annotation::Sticky { width, height, .. } = &mut p {
        (*width, *height) = (Some(NOEUD.0), Some(NOEUD.1));
    }
    p
}

/// La couleur moyenne de l'intérieur de ce nœud seul, vu à ce zoom — son coin posé à (100, 150)
/// de l'écran. L'intérieur : le nœud sans le dixième de chaque bord, là où ni coin arrondi ni
/// liseré ne tombent.
fn moyenne(noeud: Annotation, zoom: f64) -> [f64; 3] {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    app.store.add_annotation(&board, noeud);
    app.store.clear_selection();
    let (x0, y0) = (100.0, 150.0);
    app.store.set_viewport(
        &board,
        Viewport {
            x: x0,
            y: y0,
            scale: zoom,
        },
    );
    app.une_image_sans_fenetre(TAILLE);
    let p = app.pixmap.as_ref().expect("une image");
    let (l, h) = (NOEUD.0 * zoom, NOEUD.1 * zoom);
    let (gauche, haut) = ((x0 + l * 0.1) as usize, (y0 + h * 0.1) as usize);
    let (droite, bas) = ((x0 + l * 0.9) as usize, (y0 + h * 0.9) as usize);
    let mut somme = [0.0f64; 3];
    for y in haut..bas {
        for x in gauche..droite {
            let c = p.pixels()[y * p.width() as usize + x].demultiply();
            for (s, v) in somme.iter_mut().zip([c.red(), c.green(), c.blue()]) {
                *s += f64::from(v);
            }
        }
    }
    let n = ((droite - gauche) * (bas - haut)) as f64;
    somme.map(|s| s / n)
}

/// **Vu de loin, ce nœud a la couleur moyenne de son dessin de près**, canal par canal, à ce que
/// deux images de 8 bits garantissent. Le cadre vide d'avant (SCALE-2) n'en montrait que le fond :
/// la preuve à l'envers est l'écart du texte à son fond, qu'il faut d'abord voir dépasser deux fois
/// la borne sur un canal au moins — sinon l'épreuve ne verrait pas un nœud vide (un post-it jaune
/// à l'encre sombre ne bouge presque pas dans le bleu).
fn garde_la_couleur_de_ses_lettres(faire: fn(&str) -> Annotation) {
    let pres = moyenne(faire(TEXTE), 1.0);
    let loin = moyenne(faire(TEXTE), 0.1);
    let fond = moyenne(faire(" "), 1.0);
    eprintln!("MESURE gris : loin {loin:?}, près {pres:?}, fond {fond:?}");
    let marque = (0..3)
        .map(|k| (pres[k] - fond[k]).abs())
        .fold(0.0, f64::max);
    assert!(
        marque > 2.0 * DEUX_IMAGES_DE_8_BITS,
        "le texte doit se voir de près, sur un canal au moins : {pres:?} contre le fond {fond:?}"
    );
    for k in 0..3 {
        assert!(
            (loin[k] - pres[k]).abs() < DEUX_IMAGES_DE_8_BITS,
            "canal {k} : de loin {loin:?}, de près {pres:?}, le fond {fond:?}"
        );
    }
}

#[test]
fn gris_1_de_loin_le_noeud_garde_la_couleur_de_ses_lettres() {
    garde_la_couleur_de_ses_lettres(texte);
}

/// **Le post-it aussi**, par la même loi, sur les lignes que son papier montre.
#[test]
fn gris_1_de_loin_le_post_it_garde_la_couleur_de_ses_lettres() {
    garde_la_couleur_de_ses_lettres(post_it);
}

/// **Le dessin vectoriel pose le même gris** (l'export SVG, la carte graphique pour un nœud plus
/// grand que l'écran) : de loin, aucune lettre, et les formes de chaque ligne — jamais la seule
/// brume du nœud.
#[test]
fn gris_1_le_dessin_vectoriel_pose_le_gris_des_lignes() {
    use crate::renderer::card::vecteurs::carte_en_vecteurs;
    use crate::renderer::card::TextCard;
    use crate::renderer::pass::{Clip, Pass};
    use crate::renderer::scale::WorldScale;
    use crate::renderer::svg::Svg;
    let renderer = crate::renderer::Renderer::new();
    let kit = renderer.kit();
    let ecran = (900.0f32, 700.0f32);
    let zoom = 0.1;
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        mises_en_page: kit.mises_en_page,
        tints: kit.tints,
        theme: kit.theme,
        vp: Viewport {
            x: 100.0,
            y: 150.0,
            scale: zoom,
        },
        scale: WorldScale::new(zoom, 1.0),
        clip: Clip {
            width: ecran.0,
            height: ecran.1,
            top: 0.0,
        },
        poignees: true,
    };
    let carte = |corps| TextCard {
        origin: (0.0, 0.0),
        size: (NOEUD.0 as f32, NOEUD.1 as f32),
        body: corps,
        tint: (120, 160, 220),
        fond: (20, 16, 16),
        eclaires: None,
        selected: false,
        editing: None,
    };
    let dessin = |corps| {
        let mut svg = Svg::new();
        carte_en_vecteurs(&ctx, carte(corps), &mut svg);
        svg.finir((ecran.0 as u32, ecran.1 as u32))
    };
    let (plein, vide) = (dessin(TEXTE), dessin(" "));
    let rectangles = |s: &str| s.matches("<rect").count();
    assert!(
        rectangles(&plein) > rectangles(&vide) + 10,
        "de loin, le texte pose le gris de ses lignes :\n{plein}"
    );
    assert!(!plein.contains("<use"), "de loin, aucune lettre ne se pose");
}
