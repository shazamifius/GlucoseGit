//! Ce qu'une étiquette de flèche dessine — et surtout, la largeur qu'elle se donne.

use super::super::pass::Clip;
use super::super::scale::WorldScale;
use super::*;
use crate::renderer::math::MathRenderer;
use crate::renderer::DomainTints;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::text::Selection;
use glucose_core::types::Viewport;
use tiny_skia::{Color, Pixmap};

/// Le fond de la toile, pour reconnaître ce qui a été peint.
const FOND: (u8, u8, u8) = (13, 14, 18);

fn fleche(text: Option<&str>) -> Annotation {
    let mut a = Annotation::arrow("a", 100.0, 300.0, 700.0, 300.0);
    if let Annotation::Arrow { text: t, .. } = &mut a {
        *t = text.map(str::to_string);
    }
    a
}

fn plateau(arrow: Annotation) -> Board {
    let mut board = Board::new("b", "plateau");
    board.annotations.push(arrow);
    board
}

fn session(buffer: &str, caret: usize) -> TextEditSession {
    TextEditSession {
        ann_id: "a".into(),
        buffer: buffer.to_string(),
        selection: Selection::at(caret),
        goal_x: None,
        blink_timer: std::time::Instant::now(),
    }
}

/// Rend l'étiquette seule sur un fond uni, et rend la pixmap.
fn rendu(arrow: &Annotation, editing: Option<&TextEditSession>) -> Pixmap {
    let mut pixmap = Pixmap::new(800, 600).expect("pixmap de test");
    pixmap.fill(Color::from_rgba8(FOND.0, FOND.1, FOND.2, 255));
    let board = plateau(arrow.clone());
    let typography = Typography::new();
    let math = MathRenderer::new();
    let tints = DomainTints::default();
    let theme = Theme::dark();
    let vp = Viewport {
        x: 0.0,
        y: 0.0,
        scale: 1.0,
    };
    let ctx = Pass {
        typography: &typography,
        math: &math,
        tints: &tints,
        theme: &theme,
        vp,
        scale: WorldScale::new(vp.scale),
        clip: Clip {
            width: 800.0,
            height: 600.0,
            top: 0.0,
        },
    };
    draw_arrow_label(&ctx, &mut pixmap.as_mut(), &board, arrow, editing, false);
    pixmap
}

/// L'étendue horizontale de ce qui a été peint, en pixels : `None` si rien ne l'a été.
fn largeur_peinte(pixmap: &Pixmap) -> Option<f32> {
    let w = pixmap.width() as usize;
    let mut min = usize::MAX;
    let mut max = 0usize;
    for (index, pixel) in pixmap.pixels().iter().enumerate() {
        if (pixel.red(), pixel.green(), pixel.blue()) != FOND {
            let x = index % w;
            min = min.min(x);
            max = max.max(x);
        }
    }
    (min <= max).then(|| (max - min + 1) as f32)
}

/// Une flèche sans étiquette ne peint rien du tout.
#[test]
fn test_an_arrow_without_a_label_paints_nothing() {
    assert_eq!(largeur_peinte(&rendu(&fleche(None), None)), None);
}

/// Une étiquette blanche n'en est pas une : elle ne peint pas de pastille creuse.
#[test]
fn test_a_blank_label_is_not_a_label() {
    assert_eq!(largeur_peinte(&rendu(&fleche(Some("   ")), None)), None);
}

/// Une étiquette posée se dessine.
#[test]
fn test_a_label_is_painted() {
    let large = largeur_peinte(&rendu(&fleche(Some("contredit")), None));
    assert!(
        large.is_some_and(|w| w > 20.0),
        "rien de lisible : {large:?}"
    );
}

/// **LABEL-1** — la pastille est mesurée, pas estimée d'après le nombre de caractères.
///
/// C'est le test qui porte la correction. Glucose Tauri calcule sa largeur par
/// `text.length * 6.5` : sur ces deux chaînes de **même longueur**, il donnerait donc
/// exactement la même pastille, alors qu'un `W` est trois fois plus large qu'un `i`. Le
/// texte déborderait d'un côté, et flotterait de l'autre.
#[test]
fn test_label_1_the_pill_is_measured_not_estimated() {
    let larges = largeur_peinte(&rendu(&fleche(Some("WWWWWW")), None)).expect("peint");
    let etroits = largeur_peinte(&rendu(&fleche(Some("iiiiii")), None)).expect("peint");
    assert_eq!(
        "WWWWWW".len(),
        "iiiiii".len(),
        "le test ne vaut que sur deux chaînes de même longueur"
    );
    assert!(
        larges > etroits * 1.5,
        "six W tiennent dans {larges} px et six i dans {etroits} : la largeur est devinée"
    );
}

/// Une étiquette plus longue fait une pastille plus large — la boîte suit son contenu.
#[test]
fn test_a_longer_label_gets_a_wider_pill() {
    let court = largeur_peinte(&rendu(&fleche(Some("a")), None)).expect("peint");
    let long = largeur_peinte(&rendu(&fleche(Some("a b c d e f g")), None)).expect("peint");
    assert!(long > court * 2.0, "{court} px contre {long} px");
}

/// En saisie, c'est le **tampon** qui s'affiche, et non le texte enregistré.
#[test]
fn test_while_typing_the_buffer_is_shown_not_the_stored_text() {
    let arrow = fleche(Some("ancien"));
    let en_saisie = session("un texte bien plus long", 5);
    let pose = largeur_peinte(&rendu(&arrow, None)).expect("peint");
    let tape = largeur_peinte(&rendu(&arrow, Some(&en_saisie))).expect("peint");
    assert!(tape > pose, "{pose} px contre {tape} px");
}

/// Une étiquette **vide en cours de saisie** se dessine quand même.
///
/// C'est ce qui rend visible qu'on écrit dans une flèche qui n'avait pas encore de nom :
/// sans pastille ni barre, un double-clic sur une flèche nue n'aurait aucun effet visible,
/// et le geste aurait l'air de n'avoir rien fait.
#[test]
fn test_an_empty_label_being_typed_still_shows_its_caret() {
    let vide = session("", 0);
    assert!(
        largeur_peinte(&rendu(&fleche(None), Some(&vide))).is_some(),
        "la saisie d'une étiquette vide ne montre rien"
    );
}

/// La barre de saisie avance avec le curseur.
#[test]
fn test_the_caret_moves_with_the_cursor() {
    // Le texte est le même ; seul le curseur bouge. La pastille garde donc sa largeur, et
    // c'est la position de la barre qui doit changer — donc le motif de pixels.
    let arrow = fleche(None);
    let debut = rendu(&arrow, Some(&session("abcdefgh", 0)));
    let fin = rendu(&arrow, Some(&session("abcdefgh", 8)));
    assert_ne!(
        debut.data(),
        fin.data(),
        "la barre de saisie ne bouge pas avec le curseur"
    );
}

/// L'étendue verticale de ce qui a été peint, en pixels.
fn hauteur_peinte(pixmap: &Pixmap) -> Option<f32> {
    let w = pixmap.width() as usize;
    let mut min = usize::MAX;
    let mut max = 0usize;
    for (index, pixel) in pixmap.pixels().iter().enumerate() {
        if (pixel.red(), pixel.green(), pixel.blue()) != FOND {
            let y = index / w;
            min = min.min(y);
            max = max.max(y);
        }
    }
    (min <= max).then(|| (max - min + 1) as f32)
}

/// Le texte tient **dans** sa pastille : rien ne déborde en haut ni en bas.
///
/// Ce test-là manquait, et la capture l'a montré avant lui : `draw_text` prend le **haut**
/// de la ligne et non sa ligne de base, si bien que le libellé se dessinait sous sa
/// pastille au lieu d'être dedans. Les huit autres tests mesuraient des largeurs et ne
/// pouvaient pas le voir — regarder l'écran reste ce qui trouve ce genre de faute, et un
/// test est ce qui l'empêche de revenir.
#[test]
fn test_the_text_fits_inside_its_pill() {
    let peinte = hauteur_peinte(&rendu(&fleche(Some("Ambigüité ÉQJ")), None)).expect("peint");
    let pastille = LABEL_HALF_H * 2.0;
    assert!(
        peinte <= pastille,
        "la marque fait {peinte} px de haut pour une pastille de {pastille} : le texte déborde"
    );
    // Et elle n'est pas vide non plus : une pastille sans son texte remplirait la borne.
    assert!(peinte > LABEL_HALF_H, "seulement {peinte} px peints");
}

/// Un texte à jambages descendants ne se fait pas rogner par le bas de la pastille.
#[test]
fn test_descenders_are_not_clipped_by_the_pill() {
    let sans = hauteur_peinte(&rendu(&fleche(Some("noon")), None)).expect("peint");
    let avec = hauteur_peinte(&rendu(&fleche(Some("pgjqy")), None)).expect("peint");
    assert!(
        avec >= sans,
        "les jambages descendants ({avec} px) tiennent moins de place que « noon » ({sans} px)"
    );
}
