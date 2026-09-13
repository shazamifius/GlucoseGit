//! Tests de la carte de texte : auto-similarité (SCALE-1), étirement (CARD-1), reflux
//! (WRAP-1) et hauteur suivie (TEXT-FIT-1).

use super::*;
use crate::renderer::math::MathRenderer;
use crate::typography::Face;
use glucose_core::types::Annotation;

/// Les lignes visuelles d'une carte au repos — la vue que l'on regarde.
fn lignes(typo: &Typography, source: &str, width: f32) -> Vec<VisualLine> {
    card_text_layout(
        typo,
        &MathRenderer::new(),
        source,
        width,
        TextMode::Rendered,
    )
    .lines
}

/// Carte d'essai partagée avec les autres suites du renderer.
pub fn probe_card(id: &str, x: f64, y: f64) -> Annotation {
    Annotation::Text {
        id: id.into(),
        x,
        y,
        width: Some(200.0),
        height: Some(50.0),
        text: format!("Card {id}"),
        font_size: Some(14.0),
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    }
}

#[test]
fn test_scale_1_the_layout_is_self_similar_at_every_zoom() {
    // Le rapport de chaque mesure à la largeur de la boîte doit être celui du monde.
    let world = CardLayout::text_card(260.0, 120.0, 4);
    for zoom in [0.25_f64, 0.5, 1.0, 2.0, 4.0, 16.0] {
        let screen = world.scaled(WorldScale::new(zoom));
        let pairs = [
            (screen.font, world.font),
            (screen.pad_x, world.pad_x),
            (screen.pad_y, world.pad_y),
            (screen.radius, world.radius),
            (screen.indent, world.indent),
            (screen.bullet, world.bullet),
            (screen.border, world.border),
            (screen.line_height, world.line_height),
            (screen.height, world.height),
        ];
        for (on_screen, in_world) in pairs {
            let expected = in_world / world.width;
            let observed = on_screen / screen.width;
            assert!(
                (observed - expected).abs() < 1e-6,
                "zoom {zoom} : rapport {observed} au lieu de {expected}"
            );
        }
    }
}

#[test]
fn test_a_card_stretches_to_fit_its_text_in_world_units() {
    // La hauteur necessaire se calcule AVANT la mise a l'echelle : deux zooms doivent
    // donner la meme carte a un facteur pres, sinon la mise en page se reorganise.
    // Une ligne demande 16 + 14 × 1,4 + 16 = 51,6 px (fiche 06 § 5.1) : 60 est assez haut.
    let short = CardLayout::text_card(260.0, 60.0, 1);
    let tall = CardLayout::text_card(260.0, 60.0, 8);
    assert_eq!(short.height, 60.0, "une carte assez haute garde sa hauteur");
    assert!(tall.height > 60.0, "une carte trop courte s'etire");
    let ratio_1 =
        tall.scaled(WorldScale::new(0.3)).height / tall.scaled(WorldScale::new(0.3)).width;
    let ratio_2 =
        tall.scaled(WorldScale::new(3.0)).height / tall.scaled(WorldScale::new(3.0)).width;
    assert!((ratio_1 - ratio_2).abs() < 1e-6, "{ratio_1} != {ratio_2}");
}

const LONG_TEXT: &str =
    "Une carte de texte redimensionnée en largeur reflue son texte, et sa hauteur suit.";

#[test]
fn test_wrap_1_a_narrower_card_has_more_lines_and_none_overflows() {
    let typo = Typography::new();
    let wide = lignes(&typo, LONG_TEXT, 900.0);
    let narrow = lignes(&typo, LONG_TEXT, 240.0);
    assert_eq!(wide.len(), 1, "{wide:?}");
    assert!(narrow.len() > 2, "{narrow:?}");
    // Aucune ligne visuelle ne dépasse la largeur utile, mesurée avec la même police.
    let usable = 240.0 - PAD_X * 2.0;
    for line in &narrow {
        let (w, _) = typo.measure_text(&LONG_TEXT[line.start..line.end], BODY_FONT, Face::Regular);
        assert!(
            w <= usable + 1e-3,
            "« {} » mesure {w} > {usable}",
            &LONG_TEXT[line.start..line.end]
        );
    }
    // Recollées, les lignes redonnent le texte, aux espaces de coupe près.
    let joined: Vec<&str> = narrow.iter().map(|l| &LONG_TEXT[l.start..l.end]).collect();
    assert_eq!(joined.join(" "), LONG_TEXT);
}

#[test]
fn test_wrap_1_a_bulleted_paragraph_keeps_its_bullet_on_its_first_line_only() {
    let typo = Typography::new();
    let text = "# Titre\n- une puce assez longue pour être coupée en deux lignes au moins\ncorps";
    let lines = lignes(&typo, text, 200.0);
    let bullets: Vec<&VisualLine> = lines
        .iter()
        .filter(|l| l.kind == LineKind::Bullet)
        .collect();
    assert!(bullets.len() >= 2, "{lines:?}");
    assert!(bullets[0].first && bullets[1..].iter().all(|l| !l.first));
    // La ligne couvre la source, préfixe compris — c'est ce dont le curseur a besoin — mais
    // le préfixe est un fragment marqué, que le rendu saute (MODE-1).
    assert_eq!(&text[bullets[0].start..bullets[0].start + 5], "- une");
    assert_eq!(lines[0].kind, LineKind::Heading1);
    assert_eq!(&text[lines[0].start..lines[0].end], "# Titre");
    assert_eq!(lines.last().map(|l| l.kind), Some(LineKind::Body));
}

#[test]
fn test_text_fit_1_the_fitted_height_follows_the_line_count() {
    let typo = Typography::new();
    let one = text_card_fit_height(&typo, &MathRenderer::new(), "court", 240.0);
    assert!(
        (one - (PAD_Y * 2.0 + BODY_FONT * LINE_FACTOR) as f64).abs() < 1e-4,
        "{one}"
    );
    let wide = text_card_fit_height(&typo, &MathRenderer::new(), LONG_TEXT, 900.0);
    let narrow = text_card_fit_height(&typo, &MathRenderer::new(), LONG_TEXT, 240.0);
    assert!(narrow > wide, "{narrow} <= {wide}");
    assert_eq!(
        text_card_fit_height(&typo, &MathRenderer::new(), "", 240.0),
        one,
        "une carte vide garde une ligne"
    );
}

/// Fiche 08 § 2.1 — ce que le moteur reconnaît comme **genre de bloc**, ni plus ni moins :
/// `#` et `##` comme titres, `-` et `*` comme puces. `###` à `######`, les citations, les
/// tableaux et les blocs de code sont encore lus comme du corps de texte, et ce test tombera
/// quand ils seront écrits — c'est son but.
///
/// Le gras, l'italique, le barré et le code **en ligne** sont, eux, compris : ils ne sont pas
/// des genres de bloc mais des emphases à l'intérieur d'une ligne, et c'est
/// [`glucose_core::text`] qui les analyse. Une ligne qui commence par `**` reste donc du
/// corps — avec du gras dedans.
#[test]
fn test_the_markdown_the_card_understands_and_the_markdown_it_does_not() {
    assert_eq!(LineKind::of("# Titre"), (LineKind::Heading1, 2));
    assert_eq!(LineKind::of("## Sous-titre"), (LineKind::Heading2, 3));
    assert_eq!(LineKind::of("- une puce"), (LineKind::Bullet, 2));
    assert_eq!(LineKind::of("* une autre"), (LineKind::Bullet, 2));

    for not_yet in ["### H3", "###### H6", "> citation", "| a | b |", "```rust"] {
        assert_eq!(
            LineKind::of(not_yet),
            (LineKind::Body, 0),
            "{not_yet:?} n'est pas encore compris"
        );
    }
}

/// Fiche 06 § 5.1 — la carte « nuage / brume » : padding `16px 24px`, coins de 32 px, corps
/// 14 px, interligne 1,4.
#[test]
fn test_the_text_card_metrics_are_those_of_the_spec() {
    assert_eq!((PAD_X, PAD_Y), (24.0, 16.0));
    assert_eq!(CORNER_RADIUS, 32.0);
    assert_eq!(BODY_FONT, 14.0);
    assert_eq!(LINE_FACTOR, 1.4);
}

// ── MODE-1 : la carte qu'on corrige montre ses signes ───────────────────────

/// Le texte des deux captures du mode : chaque emphase, et le préfixe d'un bloc.
const MODE_TEXT: &str = "# Titre **net**\nUn `code` et du ~~barré~~.";

/// Rend la même carte au repos puis en édition, et rend les deux images.
///
/// Le PNG est écrit dans `target/mode-1/` : ces deux vues ne sont pas dans la scène témoin
/// — une capture de scène ne connaît pas de session d'édition — donc c'est ici qu'on peut
/// les regarder après un changement du rendu.
fn deux_modes(texte: &str) -> (tiny_skia::Pixmap, tiny_skia::Pixmap) {
    use crate::params::ViewPass;
    use crate::renderer::hue::SymbioticHueCache;
    use crate::renderer::pass::draw_annotations;
    use crate::renderer::{PaintKit, TextEditSession};
    use glucose_core::store::Store;
    use glucose_core::types::Viewport;
    use std::collections::HashSet;

    let mut store = Store::new("mode-1");
    let board = store.project.active_board_id.clone();
    let mut carte = probe_card("mode", 0.0, 0.0);
    if let Annotation::Text { text, width, .. } = &mut carte {
        *text = texte.to_string();
        *width = Some(300.0);
    }
    store.add_annotation(&board, carte);

    let session = TextEditSession {
        ann_id: "mode".into(),
        buffer: texte.to_string(),
        cursor_idx: texte.len(),
        // Le curseur clignote une demi-seconde sur deux : posé ici dans sa phase éteinte, il
        // ne vient pas ajouter sa barre à la différence que ce test mesure.
        blink_timer: std::time::Instant::now()
            .checked_sub(std::time::Duration::from_millis(500))
            .expect("une demi-seconde avant maintenant"),
    };
    let ids: HashSet<&str> = std::iter::once("mode").collect();
    let theme = crate::theme::Theme::dark();
    let typo = Typography::new();
    let math = MathRenderer::new();
    let mut tints = crate::renderer::domain::DomainTints::new();
    tints.refresh(&store, &theme);

    let rendu = |edition: Option<&TextEditSession>| {
        let mut pixmap = tiny_skia::Pixmap::new(420, 200).expect("pixmap");
        pixmap.fill(Color::from_rgba8(13, 14, 18, 255));
        let mut hue = SymbioticHueCache::new();
        let mut view = pixmap.as_mut();
        draw_annotations(
            &mut hue,
            PaintKit {
                typography: &typo,
                math: &math,
                tints: &tints,
                theme: &theme,
            },
            &mut view,
            &store,
            edition,
            ViewPass {
                vp: Viewport {
                    x: 30.0,
                    y: 30.0,
                    scale: 1.4,
                },
                visible_ids: &ids,
                header_h: 0.0,
            },
        );
        pixmap
    };
    (rendu(None), rendu(Some(&session)))
}

/// **MODE-1, à l'écran.** Au repos, la carte ne montre aucun signe ; en édition, elle les
/// montre tous — et ce sont bien deux images différentes, pas deux fois la même.
///
/// La mesure porte sur l'**encre posée**, pas sur la mise en page qui l'a posée : les tests
/// de `richtext` prouvent le découpage, celui-ci prouve qu'il arrive jusqu'au pixmap.
#[test]
fn test_mode_1_editing_shows_the_markdown_signs_on_screen() {
    let (repos, edition) = deux_modes(MODE_TEXT);
    let dir = std::path::Path::new("target/mode-1");
    std::fs::create_dir_all(dir).expect("dossier de capture");
    repos.save_png(dir.join("repos.png")).expect("écrire repos");
    edition
        .save_png(dir.join("edition.png"))
        .expect("écrire edition");

    assert_ne!(
        repos.data(),
        edition.data(),
        "un texte qui porte des signes ne se dessine pas pareil dans les deux modes"
    );

    // La preuve par contraste : sans le moindre signe, les deux modes donnent **exactement**
    // la même image. C'est donc bien l'affichage des signes qui les distingue, et rien
    // d'autre — ni une police, ni une position, ni une couleur qui changerait à l'édition.
    let (repos_nu, edition_nu) = deux_modes("Titre net\nUn code et du barre.");
    assert_eq!(
        repos_nu.data(),
        edition_nu.data(),
        "sans signe à montrer, éditer ne doit rien changer au dessin"
    );
}
