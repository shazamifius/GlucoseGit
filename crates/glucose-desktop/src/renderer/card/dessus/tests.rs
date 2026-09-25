//! **Où se pose le curseur** (COMPOSANT-3) : sur la ligne qui porte sa position, à sa place
//! dans la ligne — lu sur l'image, par différence entre le curseur allumé et éteint.
//!
//! L'épreuve des deux voies ne le peut pas : elles peignent le curseur par la même fonction,
//! et un curseur décalé d'une ligne l'est des deux côtés. Trouvé en sabotant : décaler le
//! curseur d'une ligne ne faisait rien tomber.

use super::super::{draw_text_card, CardLayout, TextCard};
use crate::renderer::pass::{Clip, Pass};
use crate::renderer::scale::WorldScale;
use crate::renderer::{Renderer, TextEditSession};
use glucose_core::text::Selection;
use glucose_core::types::Viewport;
use tiny_skia::Pixmap;

const LARGEUR: f32 = 300.0;
const ECRAN: (u32, u32) = (400, 300);

/// La boîte `(x0, y0, x1, y1)` des pixels que le curseur allumé change, ou `None` s'il ne se
/// pose nulle part.
fn curseur(corps: &str, tete: usize) -> Option<(u32, u32, u32, u32)> {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        tints: kit.tints,
        theme: kit.theme,
        vp: Viewport {
            scale: 1.0,
            x: 0.0,
            y: 0.0,
        },
        scale: WorldScale::new(1.0, 1.0),
        clip: Clip {
            width: ECRAN.0 as f32,
            height: ECRAN.1 as f32,
            top: 0.0,
        },
    };
    let rendu = |curseur_visible| {
        let saisie = TextEditSession {
            ann_id: "c".into(),
            buffer: corps.into(),
            selection: Selection::at(tete),
            goal_x: None,
            blink_timer: std::time::Instant::now(),
            curseur_visible,
        };
        let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("pixmap");
        let carte = TextCard {
            origin: (0.0, 0.0),
            size: (LARGEUR, 60.0),
            body: corps,
            tint: (96, 165, 250),
            selected: false,
            editing: Some(&saisie),
        };
        draw_text_card(&ctx, &mut pixmap.as_mut(), carte);
        pixmap
    };
    let (allume, eteint) = (rendu(true), rendu(false));
    let mut boite: Option<(u32, u32, u32, u32)> = None;
    let paires = allume.pixels().iter().zip(eteint.pixels());
    for (i, _) in paires.enumerate().filter(|(_, (a, b))| a != b) {
        let (x, y) = (i as u32 % ECRAN.0, i as u32 / ECRAN.0);
        boite = Some(match boite {
            None => (x, y, x, y),
            Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
        });
    }
    boite
}

/// Le haut de la ligne `rang` et la marge gauche du texte, à l'échelle 1.
fn ligne(rang: usize) -> (f32, f32) {
    let carte = CardLayout::text_card(LARGEUR, 60.0, 1);
    (carte.pad_y + rang as f32 * carte.line_height, carte.pad_x)
}

/// Le curseur se pose en haut de la ligne attendue, à un pixel près (il est calé sur la
/// grille), et à l'abscisse demandée.
fn sur_la_ligne(corps: &str, tete: usize, rang: usize) -> (u32, u32, u32, u32) {
    let boite = curseur(corps, tete).expect("le curseur allume doit se voir");
    let (haut, _) = ligne(rang);
    assert!(
        (boite.1 as f32 - haut).abs() <= 1.0,
        "{corps:?}, position {tete} : le curseur commence a {} au lieu de la ligne {rang} ({haut})",
        boite.1
    );
    boite
}

#[test]
fn test_composant_3_le_curseur_se_pose_sur_la_ligne_de_sa_position() {
    let corps = "un\ndeux\ntrois";
    let (_, gauche) = ligne(0);
    // Au début d'un paragraphe : la marge gauche de sa ligne.
    let debut = sur_la_ligne(corps, 3, 1);
    assert!(
        (debut.0 as f32 - gauche).abs() <= 1.0,
        "au debut de « deux », le curseur est a {} et non a la marge ({gauche})",
        debut.0
    );
    // À la fin du texte : après le dernier mot, sur la dernière ligne.
    let fin = sur_la_ligne(corps, corps.len(), 2);
    assert!(
        fin.0 as f32 > gauche + 20.0,
        "au bout de « trois », le curseur est a {}, sur la marge",
        fin.0
    );
    // Au milieu d'un mot, entre les deux.
    let milieu = sur_la_ligne(corps, 5, 1);
    assert!(debut.0 < milieu.0, "« de|ux » se pose apres « |deux »");
}

/// **Un paragraphe vide a sa ligne**, et le curseur qui s'y trouve s'y pose : c'est le cas
/// d'une carte qu'on vient de passer à la ligne.
#[test]
fn test_composant_3_le_curseur_d_une_ligne_neuve_se_pose_sur_elle() {
    let corps = "abc\n";
    let boite = sur_la_ligne(corps, corps.len(), 1);
    let (_, gauche) = ligne(1);
    assert!((boite.0 as f32 - gauche).abs() <= 1.0);
}

/// **Sur une ligne refluée, la coupe appartient à la ligne qu'elle termine.**
#[test]
fn test_composant_3_le_curseur_suit_le_reflux() {
    let corps = "Une phrase assez longue pour se replier sur une seconde ligne de la carte.";
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let lignes = super::super::card_text_layout(
        kit.typography,
        kit.math,
        corps,
        LARGEUR,
        crate::renderer::richtext::TextMode::Source,
    )
    .lines;
    assert!(lignes.len() >= 2, "le texte d'epreuve doit se replier");
    let coupe = lignes[1].start;
    sur_la_ligne(corps, coupe, 1);
    sur_la_ligne(corps, lignes[0].end, 0);
}

/// **Toute position du texte appartient à une ligne**, en mode source — celui où l'on écrit.
///
/// C'est ce qui garantit qu'un curseur a toujours où se poser, quel que soit le bloc qu'il
/// traverse. Un corpus de chaque genre de bloc, et chaque frontière de caractère.
#[test]
fn test_composant_3_toute_position_du_texte_a_sa_ligne() {
    let corpus = [
        "",
        "\n",
        "abc\n",
        "\n\n\n",
        "# Titre\nTexte",
        "> citation\n> suite",
        "- un\n- deux\n  - trois",
        "1. un\n2. deux",
        "- [ ] a faire\n- [x] fait",
        "```\ncode\n```",
        "```rust\nfn f() {}\n```\n",
        "---\n",
        "| a | b |\n|---|---|\n| 1 | 2 |\n",
        "$$x^2$$",
        concat!("$$\n", r"\frac{a}{b}", "\n$$\nsuite"),
        "Une phrase assez longue pour se replier sur une seconde ligne, puis   \n",
        "**gras** et *italique* et `code` et [lien](https://exemple.fr)",
        "é à ç — « guillemets »\n",
    ];
    let renderer = Renderer::new();
    let kit = renderer.kit();
    for corps in corpus {
        let lignes = super::super::card_text_layout(
            kit.typography,
            kit.math,
            corps,
            LARGEUR,
            crate::renderer::richtext::TextMode::Source,
        )
        .lines;
        for (c, _) in corps.char_indices().chain([(corps.len(), ' ')]) {
            assert!(
                lignes.iter().any(|l| l.start <= c && c <= l.end),
                "{corps:?} : la position {c} n'appartient a aucune ligne ({:?})",
                lignes.iter().map(|l| (l.start, l.end)).collect::<Vec<_>>()
            );
        }
    }
}
