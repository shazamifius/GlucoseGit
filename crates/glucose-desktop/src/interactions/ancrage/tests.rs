//! **L'éditeur d'ancres, par les vraies entrées** : la barre, la souris, le clavier.

use crate::app::GlucoseApp;
use crate::ui::ancrage::Etape;
use glucose_core::text_anchors::resolve_text_sel;
use glucose_core::types::{Annotation, TextSelection};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;
use winit::keyboard::{Key, ModifiersState, NamedKey};

const SOURCE: &str = "bonjours\ntest\ntest\nbonjours";
const CIBLE: &str = "aurevoire\ntest\ntest\naurevoire";
const ECRAN: (f32, f32) = (1440.0, 900.0);

/// Deux cartes, une flèche de l'une à l'autre, sélectionnée.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("source", 100.0, 150.0, SOURCE));
    app.store
        .add_annotation(&board, Annotation::text("cible", 700.0, 150.0, CIBLE));
    let mut f = Annotation::arrow("f", 220.0, 200.0, 820.0, 200.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut f
    {
        *source_id = Some("source".into());
        *target_id = Some("cible".into());
    }
    app.store.add_annotation(&board, f);
    // Mesurées, comme la saisie et l'import les mesurent (TEXT-FIT-1) : la boîte d'une carte
    // contient ses lignes.
    app.fit_text_card_height("source");
    app.fit_text_card_height("cible");
    app.store.clear_selection();
    app.store.select_annotation("f".into(), false);
    app.store.journal.clear();
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    app
}

fn aller(app: &mut GlucoseApp, (x, y): (f64, f64)) {
    app.handle_cursor_moved(PhysicalPosition::new(x, y));
}

fn clic(app: &mut GlucoseApp, p: (f64, f64)) {
    aller(app, p);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    app.handle_mouse_up(MouseButton::Left);
}

fn touche(app: &mut GlucoseApp, nom: NamedKey) {
    assert!(app.touche_de_l_ancrage(&Key::Named(nom)));
}

/// Le point d'écran d'un octet du texte de la fenêtre — sur la ligne qui le porte, là où il se
/// dessine. `carte` et `texte` disent de quelle étape il s'agit ; c'est la fenêtre qui le montre.
fn point(app: &GlucoseApp, carte: &str, texte: &str, octet: usize) -> (f64, f64) {
    assert_eq!(
        app.ui.ancrage.as_ref().and_then(|a| a.carte()),
        Some(carte),
        "la fenêtre montre la carte de l'étape"
    );
    let (_, montre) = app.fenetre_d_ancrage().expect("la fenêtre est ouverte");
    assert_eq!(montre, texte);
    point_vu(app, texte, octet)
}

/// « Éditer le texte lié » dans la barre d'options.
fn ouvrir(app: &mut GlucoseApp) {
    let barre = crate::ui::options_de_fleche::layout_options_de_fleche(
        &app.store,
        &app.renderer.typography,
        ECRAN,
        app.ui.scale_factor,
    )
    .expect("la barre");
    let b = barre
        .boutons
        .iter()
        .find(|b| {
            b.contenu
                == crate::ui::options_de_fleche::Contenu::Action(
                    crate::icons::IconType::Crayon,
                    crate::ui::options_de_fleche::EDITER_LE_TEXTE_LIE,
                )
        })
        .expect("le bouton « Éditer le texte lié »");
    clic(
        app,
        (
            f64::from(b.rect.0 + 4.0),
            f64::from(b.rect.1 + b.rect.3 / 2.0),
        ),
    );
    app.vol.poser();
}

/// Ce qu'un côté de la flèche désigne, tel quel dans son texte.
fn designe(app: &GlucoseApp, source: bool) -> Vec<String> {
    let board = app.store.active_board().expect("un tableau");
    let Some(Annotation::Arrow {
        source_text_sel,
        target_text_sel,
        ..
    }) = board.annotations.iter().find(|a| a.id() == "f")
    else {
        panic!("la flèche");
    };
    let (texte, sel): (&str, Option<&TextSelection>) = if source {
        (SOURCE, source_text_sel.as_ref())
    } else {
        (CIBLE, target_text_sel.as_ref())
    };
    resolve_text_sel(texte, sel)
        .into_iter()
        .map(|r| format!("{}@{}", &texte[r.start..r.end], r.start))
        .collect()
}

/// **De bout en bout** : un clic prend le second « bonjours », `Entrée` passe à la cible, un
/// glisser y prend « aure », `Entrée` termine — et un seul `Ctrl+Z` défait tout.
#[test]
fn test_fleche_4_ancrer_de_bout_en_bout() {
    let mut app = application();
    ouvrir(&mut app);
    assert_eq!(
        app.ui.ancrage.as_ref().map(|a| a.etape),
        Some(Etape::Source)
    );
    let second = SOURCE.rfind("bonjours").expect("le second");
    let p = point(&app, "source", SOURCE, second + 3);
    clic(&mut app, p);
    touche(&mut app, NamedKey::Enter);
    assert_eq!(app.ui.ancrage.as_ref().map(|a| a.etape), Some(Etape::Cible));
    app.vol.poser();

    // Un glisser sur « aure » seulement — ce qu'un clic, qui prend le mot entier, ne peut pas
    // donner : c'est bien le glisser qui choisit.
    let p = point(&app, "cible", CIBLE, 0);
    aller(&mut app, p);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    let p = point(&app, "cible", CIBLE, 4);
    aller(&mut app, p);
    app.handle_mouse_up(MouseButton::Left);
    touche(&mut app, NamedKey::Enter);

    assert!(app.ui.ancrage.is_none(), "l'éditeur se referme");
    assert_eq!(designe(&app, true), [format!("bonjours@{second}")]);
    assert_eq!(designe(&app, false), ["aure@0"]);
    assert!(app.store.undo(), "un geste");
    assert!(designe(&app, true).is_empty() && designe(&app, false).is_empty());
}

/// **`Échap` laisse tout comme avant**, et un clic hors de la fenêtre ne désélectionne rien.
#[test]
fn test_fleche_4_echap_annule_et_un_faux_clic_ne_defait_rien() {
    let mut app = application();
    ouvrir(&mut app);
    clic(&mut app, (20.0, 880.0));
    assert!(app.ui.ancrage.is_some(), "toujours ouvert");
    assert!(
        app.ui
            .ancrage
            .as_ref()
            .is_some_and(|a| a.ancres().is_empty()),
        "un clic hors de la fenêtre ne choisit rien"
    );
    assert_eq!(
        app.store.selected_arrows().len(),
        1,
        "toujours sélectionnée"
    );
    let p = point(&app, "source", SOURCE, 2);
    clic(&mut app, p);
    touche(&mut app, NamedKey::Escape);
    assert!(app.ui.ancrage.is_none());
    assert!(designe(&app, true).is_empty(), "rien n'est écrit");
}

/// **`Ctrl` ajoute un passage au lieu de remplacer** : les deux « bonjours », chacun le sien.
#[test]
fn test_fleche_4_ctrl_ajoute_un_passage() {
    let mut app = application();
    ouvrir(&mut app);
    let second = SOURCE.rfind("bonjours").expect("le second");
    let p = point(&app, "source", SOURCE, 2);
    clic(&mut app, p);
    app.modifiers = ModifiersState::CONTROL;
    let p = point(&app, "source", SOURCE, second + 2);
    clic(&mut app, p);
    app.modifiers = ModifiersState::empty();
    touche(&mut app, NamedKey::Enter);
    touche(&mut app, NamedKey::Enter);
    assert_eq!(
        designe(&app, true),
        ["bonjours@0".to_string(), format!("bonjours@{second}")]
    );
}

/// **Une puce se retire par sa croix**, et elle seule : la fenêtre de Tauri, sa liste de ce
/// qui est choisi, un passage à la fois.
#[test]
fn test_ancre_ux_une_puce_se_retire_par_sa_croix() {
    let mut app = application();
    ouvrir(&mut app);
    let second = SOURCE.rfind("bonjours").expect("le second");
    let p = point(&app, "source", SOURCE, 2);
    clic(&mut app, p);
    app.modifiers = ModifiersState::CONTROL;
    let p = point(&app, "source", SOURCE, second + 2);
    clic(&mut app, p);
    app.modifiers = ModifiersState::empty();
    let (fenetre, _) = app.fenetre_d_ancrage().expect("ouverte");
    let croix = fenetre.choisi.expect("deux puces").puces[0].croix;
    clic(
        &mut app,
        (
            f64::from(croix.0 + croix.2 / 2.0),
            f64::from(croix.1 + croix.3 / 2.0),
        ),
    );
    let reste: Vec<i64> = app
        .ui
        .ancrage
        .as_ref()
        .expect("ouverte")
        .ancres()
        .iter()
        .map(|a| a.start)
        .collect();
    assert_eq!(reste, [second as i64], "seul le premier est parti");
}

/// **La molette fait défiler un long texte dans la fenêtre**, et le canevas ne bouge pas.
#[test]
fn test_ancre_ux_la_molette_fait_defiler_la_fenetre() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    let long = "une ligne\n".repeat(60);
    app.store.update_annotation(&board, "source", |a| {
        if let Annotation::Text { text, .. } = a {
            text.clone_from(&long);
        }
    });
    ouvrir(&mut app);
    let vue = app.store.viewport();
    app.handle_mouse_wheel(winit::event::MouseScrollDelta::LineDelta(0.0, -3.0));
    let defile = app.ui.ancrage.as_ref().expect("ouverte").defilement;
    assert!(defile > 0.0, "le texte a défilé : {defile}");
    let (fenetre, _) = app.fenetre_d_ancrage().expect("ouverte");
    assert!(defile <= fenetre.zone.defilement_max());
    assert_eq!(app.store.viewport(), vue, "le canevas n'a pas bougé");
}

/// **Un choix qui touche une formule la prend entière** : on ne désigne pas la moitié d'une
/// fraction qu'on voit dessinée.
#[test]
fn test_ancre_ux_un_choix_dans_une_formule_la_prend_entiere() {
    let texte = "avant\n$$\\frac{a+b}{c}$$\napres";
    let app = GlucoseApp::new();
    let mise_en_page = crate::renderer::card::card_text_layout(
        &app.renderer.typography,
        &app.renderer.math,
        texte,
        400.0,
        crate::renderer::richtext::TextMode::Rendered,
    );
    let debut = texte.find("$$").expect("la formule");
    let fin = texte.rfind("$$").expect("sa fin") + 2;
    assert_eq!(
        super::etendre_aux_formules(&mise_en_page, (debut + 4, debut + 4)),
        (debut, fin),
        "un clic dans la formule"
    );
    assert_eq!(
        super::etendre_aux_formules(&mise_en_page, (1, debut + 3)),
        (1, fin),
        "un glisser qui y entre"
    );
    assert_eq!(
        super::etendre_aux_formules(&mise_en_page, (0, 5)),
        (0, 5),
        "ce qui ne la touche pas reste tel quel"
    );
}

/// **Par le vrai clic, une formule se choisit entière** : cliquer au milieu d'une fraction
/// dessinée désigne toute la formule, pas un morceau de sa source.
#[test]
fn test_ancre_ux_un_clic_sur_une_formule_la_choisit_entiere() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    let texte = "avant\n$$\\frac{a+b}{c}$$\napres".to_string();
    let copie = texte.clone();
    app.store.update_annotation(&board, "source", |a| {
        if let Annotation::Text { text, .. } = a {
            *text = copie;
        }
    });
    ouvrir(&mut app);
    let (fenetre, _) = app.fenetre_d_ancrage().expect("ouverte");
    let zone = fenetre.zone;
    let debut = texte.find("$$").expect("la formule");
    let fin = texte.rfind("$$").expect("sa fin") + 2;
    let r = crate::renderer::passages::rectangles(
        (&app.renderer.typography, &app.renderer.math),
        &texte,
        zone.largeur_monde,
        &[(debut, fin)],
    )[0];
    let centre = (
        f64::from(zone.rect.0 + (r.0 + r.2 / 2.0) * zone.s),
        f64::from(zone.rect.1 + (r.1 + r.3 / 2.0) * zone.s),
    );
    clic(&mut app, centre);
    let choisi: Vec<(i64, i64)> = app
        .ui
        .ancrage
        .as_ref()
        .expect("ouverte")
        .ancres()
        .iter()
        .map(|a| (a.start, a.end))
        .collect();
    assert_eq!(choisi, [(debut as i64, fin as i64)]);
}

/// Le point d'écran où l'octet se **dessine** dans la fenêtre — dans sa mise en page telle
/// qu'elle est, la place des passages déjà choisis ouverte (PASSAGE-2).
fn point_vu(app: &GlucoseApp, texte: &str, octet: usize) -> (f64, f64) {
    use crate::renderer::passages::{mise_en_page, Eclaires, DANS_L_EDITEUR};
    use crate::renderer::richtext::hit::{line_of_offset, x_du_caractere};
    let (fenetre, _) = app.fenetre_d_ancrage().expect("la fenêtre est ouverte");
    let zone = fenetre.zone;
    let choisies = app
        .ui
        .ancrage
        .as_ref()
        .expect("l'éditeur")
        .plages_choisies(texte);
    let eclaires = Eclaires {
        plages: &choisies,
        teinte: (0, 0, 0),
        style: &DANS_L_EDITEUR,
    };
    let outils = (&app.renderer.typography, &app.renderer.math);
    let vue = mise_en_page(
        outils,
        (texte, zone.largeur_monde),
        crate::renderer::richtext::TextMode::Rendered,
        Some(&eclaires),
    );
    let rang = line_of_offset(&vue, octet);
    let boite = crate::renderer::card::text_box(zone.largeur_monde);
    let x = x_du_caractere(outils.0, &vue, &vue.lines[rang], texte, octet, boite.body);
    let (ox, oy) = crate::renderer::card::TEXT_ORIGIN;
    let y = oy + (rang as f32 + 0.5) * boite.line_height;
    (
        f64::from(zone.rect.0 + (ox + x + 1.0) * zone.s),
        f64::from(zone.rect.1 + (y - zone.defilement) * zone.s),
    )
}

/// Un glisser de l'octet `de` à l'octet `a`, visés là où ils se dessinent.
fn glisser(app: &mut GlucoseApp, texte: &str, (de, a): (usize, usize)) {
    let p = point_vu(app, texte, de);
    aller(app, p);
    app.handle_mouse_down(MouseButton::Left, ECRAN.0, ECRAN.1);
    let p = point_vu(app, texte, a);
    aller(app, p);
    app.handle_mouse_up(MouseButton::Left);
}

/// **PASSAGE-2 — dans la fenêtre, on vise ce qu'on voit.** Un passage choisi au milieu d'un
/// mot écarte la suite de sa ligne de deux fois l'étendue de son cadre ; un second choix, fait
/// là où le texte est **dessiné** après l'écart, prend exactement les octets visés. Le clic
/// lit la même mise en page que le dessin.
#[test]
fn test_passage_2_la_fenetre_vise_ce_qu_elle_montre() {
    let texte = "abcdefghijklmnop qrstuvwxyz";
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("source", 100.0, 150.0, texte));
    app.store
        .add_annotation(&board, Annotation::text("cible", 700.0, 150.0, CIBLE));
    let mut f = Annotation::arrow("f", 220.0, 200.0, 820.0, 200.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut f
    {
        *source_id = Some("source".into());
        *target_id = Some("cible".into());
    }
    app.store.add_annotation(&board, f);
    app.fit_text_card_height("source");
    app.store.clear_selection();
    app.store.select_annotation("f".into(), false);
    app.une_image_sans_fenetre((ECRAN.0 as u32, ECRAN.1 as u32));
    ouvrir(&mut app);

    glisser(&mut app, texte, (4, 8));
    app.modifiers = ModifiersState::CONTROL;
    glisser(&mut app, texte, (18, 23));
    app.modifiers = ModifiersState::empty();
    let choisis: Vec<String> = app
        .ui
        .ancrage
        .as_ref()
        .expect("l'éditeur")
        .plages_choisies(texte)
        .into_iter()
        .map(|(a, b)| texte[a..b].to_string())
        .collect();
    assert_eq!(choisis, ["efgh", "rstuv"]);
}
