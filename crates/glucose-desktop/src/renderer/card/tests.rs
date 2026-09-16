//! Tests de la carte de texte : auto-similarité (SCALE-1), étirement (CARD-1), reflux
//! (WRAP-1) et hauteur suivie (TEXT-FIT-1).

use super::*;
use crate::renderer::math::MathRenderer;
use crate::renderer::richtext::VisualLine;
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
        .filter(|l| l.kind == BlockKind::Bullet)
        .collect();
    assert!(bullets.len() >= 2, "{lines:?}");
    assert!(bullets[0].first && bullets[1..].iter().all(|l| !l.first));
    // La ligne couvre la source, préfixe compris — c'est ce dont le curseur a besoin — mais
    // le préfixe est un fragment marqué, que le rendu saute (MODE-1).
    assert_eq!(&text[bullets[0].start..bullets[0].start + 5], "- une");
    assert_eq!(lines[0].kind, BlockKind::Heading(1));
    assert_eq!(&text[lines[0].start..lines[0].end], "# Titre");
    assert_eq!(lines.last().map(|l| l.kind), Some(BlockKind::Body));
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

/// Fiche 08 § 2.1 — ce que le moteur reconnaît comme **genre de bloc**, ni plus ni moins.
///
/// Les genres eux-mêmes sont analysés et testés dans [`glucose_core::text::block`] ; ce test
/// vérifie l'autre moitié du contrat — que la mise en page de la carte les reçoit bien, et
/// qu'aucun ne se perd entre le noyau et l'écran.
///
/// Le gras, l'italique, le barré et le code **en ligne** ne sont pas des genres de bloc mais
/// des emphases à l'intérieur d'une ligne. Une ligne qui commence par `**` reste donc du
/// corps — avec du gras dedans.
#[test]
fn test_the_markdown_the_card_understands_and_the_markdown_it_does_not() {
    let typo = Typography::new();
    let genre = |source: &str| lignes(&typo, source, 400.0)[0].kind;

    assert_eq!(genre("# Titre"), BlockKind::Heading(1));
    assert_eq!(genre("## Sous-titre"), BlockKind::Heading(2));
    assert_eq!(genre("###### Six"), BlockKind::Heading(6));
    assert_eq!(genre("- une puce"), BlockKind::Bullet);
    assert_eq!(genre("* une autre"), BlockKind::Bullet);
    assert_eq!(genre("1. un élément"), BlockKind::Ordered(1));
    assert_eq!(genre("> une citation"), BlockKind::Quote);
    assert_eq!(genre("-# du petit texte"), BlockKind::Small);
    assert_eq!(genre("---"), BlockKind::Rule);
    assert_eq!(genre("$$x$$"), BlockKind::Math { display: true });
    assert_eq!(genre("**gras** en tête"), BlockKind::Body);

    // Un bloc de code : sa clôture ne prend aucune place au repos, et ce qu'elle enferme
    // n'est plus du Markdown.
    let code = lignes(&typo, "```rust\n# pas un titre\n```", 400.0);
    assert_eq!(
        code.iter().map(|l| l.kind).collect::<Vec<_>>(),
        vec![BlockKind::Code],
        "au repos, seules les lignes de code occupent un rang"
    );

    assert_eq!(genre("| a | b |"), BlockKind::TableRow);
    assert_eq!(genre("|---|---|"), BlockKind::TableRule);
    // Un lien n'est pas un genre de bloc : c'est une emphase dans une ligne de corps.
    assert_eq!(genre("[lien](https://a.b)"), BlockKind::Body);

    // Ce qui n'est toujours pas compris, et le restera jusqu'à ce que quelqu'un l'écrive :
    // ce test tombera ce jour-là, c'est son but.
    for not_yet in [
        "  indenté de deux espaces",
        "Terme
: sa définition",
    ] {
        assert_eq!(
            genre(not_yet),
            BlockKind::Body,
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
    (
        rendu_de(texte, None),
        rendu_de(texte, Some(&session(texte))),
    )
}

/// Une session d'édition d'essai, **curseur éteint**.
///
/// Le curseur clignote une demi-seconde sur deux : posé ici dans sa phase éteinte, il n'ajoute
/// pas sa barre aux différences que ces tests mesurent.
fn session(texte: &str) -> crate::renderer::TextEditSession {
    crate::renderer::TextEditSession {
        ann_id: "mode".into(),
        buffer: texte.to_string(),
        selection: glucose_core::text::Selection::at(texte.len()),
        goal_x: None,
        blink_timer: std::time::Instant::now()
            .checked_sub(std::time::Duration::from_millis(500))
            .expect("une demi-seconde avant maintenant"),
    }
}

/// La carte d'essai portant `texte`, rendue avec ou sans session d'édition.
fn rendu_de(texte: &str, edition: Option<&crate::renderer::TextEditSession>) -> tiny_skia::Pixmap {
    use crate::params::ViewPass;
    use crate::renderer::hue::SymbioticHueCache;
    use crate::renderer::pass::draw_annotations;
    use crate::renderer::PaintKit;
    use glucose_core::store::Store;
    use glucose_core::types::Viewport;

    let mut store = Store::new("mode-1");
    let board = store.project.active_board_id.clone();
    let mut carte = probe_card("mode", 0.0, 0.0);
    if let Annotation::Text { text, width, .. } = &mut carte {
        *text = texte.to_string();
        *width = Some(300.0);
    }
    store.add_annotation(&board, carte);

    let theme = crate::theme::Theme::dark();
    let typo = Typography::new();
    let math = MathRenderer::new();
    let mut tints = crate::renderer::domain::DomainTints::new();
    tints.refresh(&store, &theme);

    let mut pixmap = tiny_skia::Pixmap::new(420, 200).expect("pixmap");
    pixmap.fill(Color::from_rgba8(13, 14, 18, 255));
    let rangs = glucose_core::quadtree::tous_les_rangs(
        store.active_board().expect("la preuve a un tableau actif"),
    );
    let mut index = glucose_core::quadtree::SpatialHash::new(1000.0);
    if let Some(b) = store.active_board() {
        index.index_board(b);
    }
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
            visibles: &rangs,
            index: &index,
            header_h: 0.0,
        },
    );
    pixmap
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

/// **La sélection se voit.** Un texte sélectionné se détache d'un fond, et ce fond passe
/// **sous** l'encre — dessiné après, il recouvrirait le texte qu'il doit mettre en valeur.
///
/// La capture est écrite dans `target/mode-1/selection.png` : c'est l'une des vues que la
/// scène témoin ne peut pas porter, faute de session d'édition.
#[test]
fn test_a_selection_is_visible_behind_the_text_it_marks() {
    let avec_selection = |selection| crate::renderer::TextEditSession {
        selection,
        ..session(MODE_TEXT)
    };
    let sans = rendu_de(
        MODE_TEXT,
        Some(&avec_selection(glucose_core::text::Selection::at(0))),
    );
    let avec = rendu_de(
        MODE_TEXT,
        Some(&avec_selection(glucose_core::text::Selection::all(
            MODE_TEXT,
        ))),
    );
    let dir = std::path::Path::new("target/mode-1");
    std::fs::create_dir_all(dir).expect("dossier de capture");
    avec.save_png(dir.join("selection.png"))
        .expect("écrire selection");

    // Le fond bleu s'ajoute là où il n'y avait que la carte : beaucoup de pixels changent.
    let differents = sans
        .pixels()
        .iter()
        .zip(avec.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differents > 500,
        "une sélection de tout le texte doit se voir largement : {differents} pixels"
    );

    // Et l'encre du texte survit : les pixels les plus clairs de la vue sélectionnée sont au
    // moins aussi clairs que ceux de l'autre. Un fond posé par-dessus les aurait effacés.
    let plus_clair = |p: &tiny_skia::Pixmap| {
        p.pixels()
            .iter()
            .map(|px| px.red() as u32 + px.green() as u32 + px.blue() as u32)
            .max()
            .unwrap_or(0)
    };
    assert!(
        plus_clair(&avec) >= plus_clair(&sans),
        "le surlignage a recouvert le texte"
    );
}

// ── Les délimiteurs d'une formule, pendant l'édition (fiche 12 § 1.A.3) ───────

/// L'encre des `$` d'une ligne, telle que le tracé la choisirait.
fn encre_des_signes(source: &str, editing: bool) -> tiny_skia::Color {
    let typo = Typography::new();
    let math = MathRenderer::new();
    let theme = Theme::dark();
    let lignes = lignes(&typo, source, 400.0);
    let ligne = lignes.first().expect("au moins une ligne");
    marker_ink(&math, &theme, ligne, source, editing)
}

/// Écrire du LaTeX sans savoir s'il compile, c'est écrire à l'aveugle. Les délimiteurs le
/// disent, là où l'auteur corrige.
#[test]
fn test_les_delimiteurs_disent_si_la_formule_compile() {
    let theme = Theme::dark();
    assert_eq!(
        encre_des_signes("$$a^2 + b^2$$", true),
        theme.success,
        "une formule valide : ses signes passent au vert"
    );
    assert_eq!(
        encre_des_signes("$$a^{2$$", true),
        theme.danger,
        "une formule fausse : ses signes passent au rouge"
    );
}

/// Au repos, les signes n'existent plus : les colorer n'aurait personne à qui parler.
#[test]
fn test_au_repos_les_delimiteurs_restent_gris() {
    let theme = Theme::dark();
    assert_eq!(encre_des_signes("$$a^2 + b^2$$", false), theme.card_marker);
    assert_eq!(encre_des_signes("$$a^{2$$", false), theme.card_marker);
}

/// Le vert et le rouge sont réservés aux formules : les autres signes du Markdown restent
/// gris, même en édition. Un `**` n'a rien à dire de sa propre validité.
#[test]
fn test_les_autres_signes_ne_prennent_jamais_ces_couleurs() {
    let theme = Theme::dark();
    for source in [
        "# Un titre",
        "**gras**",
        "- une puce",
        "> une citation",
        "du corps",
    ] {
        assert_eq!(
            encre_des_signes(source, true),
            theme.card_marker,
            "{source:?} ne doit pas se colorer"
        );
    }
}
