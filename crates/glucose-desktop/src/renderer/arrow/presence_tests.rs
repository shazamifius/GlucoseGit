//! **LIEN-4 — la présence d'un lien : leur nombre à l'écran, et la souris.** Sa droite et son
//! plancher, exacts ; la proximité comptée en tailles de nœud ; la plus forte des deux ; et la
//! souris qui ne fait redessiner que quand un lien change d'au moins un niveau d'écran.

use super::presence::{densite, proximite, PLANCHER};
use super::*;
use crate::app::GlucoseApp;
use crate::salissure::Salissure;
use glucose_core::geometry::Rect;
use glucose_core::types::Viewport;
use winit::dpi::PhysicalPosition;

/// **Sa droite** : un lien seul, plein ; vingt, 30 % ; au-delà de vingt-huit, rien — le plancher
/// le relèvera.
#[test]
fn test_lien_4_sa_droite() {
    assert_eq!(densite(1), 1.0);
    assert_eq!(densite(0), 1.0, "aucun lien : rien à retirer");
    assert!((densite(20) - 0.30).abs() < 1e-6, "{}", densite(20));
    assert!((densite(5) - (1.0 - 0.70 * 4.0 / 19.0)).abs() < 1e-6);
    assert_eq!(densite(29), 0.0);
    assert_eq!(PLANCHER, 0.08);
}

/// **La proximité, en tailles de nœud** : dans le nœud, 1 ; à une taille de distance, la moitié ;
/// la même à toute échelle (la boîte et la distance changent ensemble).
#[test]
fn test_lien_4_la_proximite_en_tailles_de_noeud() {
    let boite = Rect::new(0.0, 0.0, 400.0, 100.0);
    let taille = 200.0; // √(400 × 100)
    assert_eq!(proximite((200.0, 50.0), boite), 1.0, "dedans");
    assert_eq!(proximite((400.0, 100.0), boite), 1.0, "au bord");
    assert!((proximite((400.0 + taille, 50.0), boite) - 0.5).abs() < 1e-6);
    let dix = Rect::new(0.0, 0.0, 4000.0, 1000.0);
    assert!((proximite((4000.0 + 10.0 * taille, 500.0), dix) - 0.5).abs() < 1e-6);
    assert!(proximite((4000.0 + 2000.0, 0.0), boite) < 0.1, "loin");
}

/// Un nœud « a », treize liens vers des nœuds loin de lui ; un nœud « z » très loin, treize liens
/// aussi : vingt-six liens, la densité à zéro. La vue à la taille réelle.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let texte = |id: String, x: f64, y: f64, (w, h): (f64, f64)| {
        let mut t = Annotation::text(id, x, y, "un nœud");
        if let Annotation::Text { width, height, .. } = &mut t {
            (*width, *height) = (Some(w), Some(h));
        }
        t
    };
    for (hub, cx) in [("a", 0.0), ("z", 1_000_000.0)] {
        app.store
            .add_annotation(&board, texte(hub.into(), cx - 200.0, -50.0, (400.0, 100.0)));
        for k in 0..13 {
            let angle = f64::from(k) * 0.4833;
            let (x, y) = (cx + 20_000.0 * angle.cos(), 20_000.0 * angle.sin());
            let id = format!("{hub}{k}");
            app.store
                .add_annotation(&board, texte(id.clone(), x, y, (100.0, 50.0)));
            let mut f = Annotation::arrow(format!("f-{id}"), 0.0, 0.0, 1.0, 1.0);
            if let Annotation::Arrow {
                source_id,
                target_id,
                ..
            } = &mut f
            {
                (*source_id, *target_id) = (Some(hub.into()), Some(id));
            }
            app.store.add_annotation(&board, f);
        }
    }
    app.store.clear_selection();
    app.store.set_viewport(
        &board,
        Viewport {
            x: 600.0,
            y: 400.0,
            scale: 1.0,
        },
    );
    app.une_image_sans_fenetre((1200, 800));
    app
}

/// L'opacité de l'âme de chaque flèche que la carte graphique reçoit, par son identité.
fn ames(app: &GlucoseApp, souris: Option<(f64, f64)>) -> Vec<(String, f32)> {
    let board = app.store.active_board().expect("un tableau");
    let rangs = glucose_core::quadtree::tous_les_rangs(board);
    let pass = crate::params::ViewPass {
        vp: board.viewport,
        visibles: &rangs,
        index: &app.renderer.spatial_hash,
        contournement: None,
        header_h: 0.0,
        densite: 1.0,
    };
    let fleches: Vec<String> = Visibles::nouvelles(&rangs, board)
        .annotations()
        .filter(|a| matches!(a, Annotation::Arrow { .. }))
        .map(|a| a.id().to_string())
        .collect();
    let champs = fleches_a_poser(
        &mut SymbioticHueCache::new(),
        (
            &app.store,
            pass,
            Main {
                survole: None,
                souris,
            },
        ),
        (&app.renderer.typography, &app.renderer.math),
        (1200.0, 800.0),
    );
    fleches
        .into_iter()
        .zip(champs.iter().map(|c| c.ame[1]))
        .collect()
}

/// **La souris sur « a » allume ses liens ; ceux de « z » restent au plancher** — jamais sous
/// 8 % ; sans souris, tous au plancher (la densité est à zéro).
#[test]
fn test_lien_4_la_souris_allume_les_liens_qu_elle_approche() {
    let app = application();
    let plein = aspect::OPACITE_DU_TRAIT;
    for (id, ame) in ames(&app, Some((0.0, 0.0))) {
        let attendu = if id.starts_with("f-a") {
            plein
        } else {
            plein * PLANCHER
        };
        assert!((ame - attendu).abs() < 1e-6, "{id} : {ame} pour {attendu}");
    }
    for (id, ame) in ames(&app, None) {
        assert!(
            (ame - plein * PLANCHER).abs() < 1e-6,
            "{id} sans souris : {ame}"
        );
    }
}

/// **À une taille de nœud de distance, ses liens sont à moitié** (sa formule, 100 / (1 + d)).
#[test]
fn test_lien_4_a_une_taille_de_distance_la_moitie() {
    let app = application();
    let taille = (400.0_f64 * 100.0).sqrt();
    let plein = aspect::OPACITE_DU_TRAIT;
    for (id, ame) in ames(&app, Some((-200.0 - taille, 0.0))) {
        if id.starts_with("f-a") {
            assert!((ame - plein * 0.5).abs() < 1e-4, "{id} : {ame}");
        }
    }
}

/// **Jamais sous 8 %, même de loin** : à un vingtième de la taille réelle, un lien que rien ne
/// relève reste au plancher.
#[test]
fn test_lien_4_le_plancher_vaut_aussi_de_loin() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store.set_viewport(
        &board,
        Viewport {
            x: 600.0,
            y: 400.0,
            scale: 0.05,
        },
    );
    let plein = aspect::OPACITE_DU_TRAIT;
    for (id, ame) in ames(&app, Some((0.0, 0.0))) {
        let attendu = if id.starts_with("f-a") {
            plein * 0.05
        } else {
            plein * PLANCHER
        };
        let attendu = attendu.max(plein * PLANCHER);
        assert!((ame - attendu).abs() < 1e-6, "{id} : {ame} pour {attendu}");
    }
}

/// **La souris fait redessiner quand elle change un lien** — par la vraie souris, sur le vide ; et
/// l'image du processeur montre le lien de « a » plus fort quand elle s'en approche, sans le
/// survoler.
#[test]
fn test_lien_4_la_souris_fait_redessiner() {
    let mut app = application();
    let encre = |app: &mut GlucoseApp, ecran: (f64, f64)| {
        app.prendre_la_salissure();
        app.handle_cursor_moved(PhysicalPosition::new(ecran.0, ecran.1));
        let sale = app.prendre_la_salissure() != Salissure::Rien;
        app.une_image_sans_fenetre((1200, 800));
        let pixmap = app.pixmap.as_ref().expect("une image");
        // Le premier lien de « a » part vers la droite, sur l'axe : une bande à 400 px du nœud,
        // à la hauteur de son centre.
        let somme: f64 = (390..410)
            .flat_map(|y| (1000..1004).map(move |x| (x, y)))
            .map(|(x, y)| {
                let p = pixmap.pixel(x, y).expect("dedans");
                f64::from(p.red()) + f64::from(p.green()) + f64::from(p.blue())
            })
            .sum();
        (sale, somme)
    };
    let (sale, loin) = encre(&mut app, (1150.0, 780.0));
    assert!(sale, "la souris bouge : les liens suivent");
    // À côté de « a », sur le vide : rien n'y est survolé, seule la souris rapproche son lien.
    let (sale, pres) = encre(&mut app, (850.0, 460.0));
    assert_eq!(app.ui.noeud_survole, None, "rien sous la souris");
    assert!(sale);
    assert!(
        pres > loin,
        "plus fort près de « a » : {pres} contre {loin}"
    );
}

/// La souris va en `ecran` par le vrai chemin : l'image en est-elle salie ?
fn salit(app: &mut GlucoseApp, ecran: (f64, f64)) -> bool {
    app.prendre_la_salissure();
    app.handle_cursor_moved(PhysicalPosition::new(ecran.0, ecran.1));
    app.prendre_la_salissure() != Salissure::Rien
}

/// **Une souris qui ne change aucun lien ne redessine rien** (fiche 103 § 3) : loin de « a », sa
/// proximité reste sous la densité de ses treize liens — leur opacité ne bouge pas d'un niveau.
#[test]
fn test_lien_4_une_souris_qui_ne_change_aucun_lien_ne_redessine_rien() {
    let mut app = application();
    assert!(
        salit(&mut app, (1150.0, 780.0)),
        "la première souris se montre"
    );
    app.une_image_sans_fenetre((1200, 800));
    assert!(
        !salit(&mut app, (1160.0, 770.0)),
        "loin de tout nœud lié, aucun lien ne change"
    );
}

/// **Moins d'un niveau ne redessine pas, et les petits pas s'additionnent** : un demi-pixel près
/// de « a » change ses liens de moins d'un 255ᵉ — rien ; trois demi-pixels depuis la souris que
/// l'image montre, plus d'un — l'image se refait.
#[test]
fn test_lien_4_moins_d_un_niveau_ne_redessine_pas_et_les_pas_s_additionnent() {
    let mut app = application();
    // À 50 du bord droit de « a » : sa proximité vaut 0,8, au-dessus de la densité.
    assert!(salit(&mut app, (850.0, 400.0)));
    app.une_image_sans_fenetre((1200, 800));
    assert!(!salit(&mut app, (850.5, 400.0)), "un demi-pixel : rien");
    assert!(!salit(&mut app, (851.0, 400.0)), "un pixel : encore rien");
    assert!(
        salit(&mut app, (851.5, 400.0)),
        "un pixel et demi depuis l'image : plus d'un niveau"
    );
}

/// **La plus forte des deux** : un lien seul à l'écran est plein, souris loin ou absente ; une
/// flèche libre se rapproche par elle-même.
#[test]
fn test_lien_4_la_plus_forte_des_deux() {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    app.store
        .add_annotation(&board, Annotation::arrow("libre", 0.0, 0.0, 1000.0, 0.0));
    app.une_image_sans_fenetre((1200, 800));
    let b = app.store.active_board().expect("un tableau");
    let fleche = b
        .annotations
        .iter()
        .find(|a| a.id() == "libre")
        .expect("la flèche");
    let noeuds = NoeudsDuRendu {
        board: b,
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
        contournement: None,
    };
    let presence = super::presence::presence;
    assert_eq!(
        presence(fleche, noeuds, Some((0.0, 90_000.0)), 1),
        1.0,
        "seul : plein"
    );
    assert_eq!(
        presence(fleche, noeuds, None, 1),
        1.0,
        "sans souris : la densité"
    );
    assert_eq!(presence(fleche, noeuds, None, 20), densite(20));
    assert_eq!(
        presence(fleche, noeuds, Some((500.0, 0.0)), 40),
        1.0,
        "sur elle"
    );
}

/// **Par l'entrée de la voie graphique** (`rendre_les_couches`) : la souris de la scène arrive
/// jusqu'aux flèches que la carte reçoit.
#[test]
fn test_lien_4_la_voie_graphique_recoit_la_souris() {
    let mut app = application();
    let ames = |app: &mut GlucoseApp, souris: Option<(f64, f64)>| {
        let overlay = crate::params::SceneOverlay {
            souris,
            ..crate::params::SceneOverlay::sans_rien(&app.active_guides)
        };
        let pointeur = crate::params::Pointer { x: 0.0, y: 0.0 };
        let (mut dessous, mut dessus) = (
            tiny_skia::Pixmap::new(1200, 800).expect("pixmap"),
            tiny_skia::Pixmap::new(1200, 800).expect("pixmap"),
        );
        let confie = app.renderer.rendre_les_couches(
            &mut dessous.as_mut(),
            &mut dessus.as_mut(),
            &app.store,
            (&mut app.ui, pointeur),
            overlay,
            crate::renderer::Regard::immobile(),
        );
        confie
            .fleches
            .iter()
            .map(|c| c.ame[1])
            .fold(0.0_f32, f32::max)
    };
    let plein = aspect::OPACITE_DU_TRAIT;
    assert!((ames(&mut app, None) - plein * PLANCHER).abs() < 1e-6);
    assert!((ames(&mut app, Some((0.0, 0.0))) - plein).abs() < 1e-6);
}

/// **L'export n'a pas de souris : la présence y est la densité** — vingt-six liens, le plancher.
#[test]
fn test_lien_4_l_export_prend_la_densite() {
    let mut app = application();
    let vue = app.store.viewport();
    let svg = app
        .renderer
        .rendre_en_svg((&app.store, &mut app.ui), vue, (1200, 800), &mut |_| None);
    assert!(
        svg.contains(r#"stroke-opacity="0.074""#),
        "l'âme au plancher"
    );
    assert!(!svg.contains(r#"stroke-opacity="0.92""#), "aucune pleine");
}

/// **Les blocs, pas les membranes** (fiche 103 § 3) : un lien d'un texte vers une grande membrane —
/// la forme de chaque lien de la carte de Wikipédia. La souris dans la membrane, loin du texte, ne
/// le rapproche pas : à quarante liens, il garde sa densité ; près du texte, il revient. La première
/// loi comptait la boîte de la membrane, et la souris était partout « dedans ».
#[test]
fn test_lien_4_la_souris_rapproche_des_blocs_pas_des_membranes() {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let mut texte = Annotation::text("t", -200.0, -50.0, "un résumé");
    if let Annotation::Text { width, height, .. } = &mut texte {
        (*width, *height) = (Some(400.0), Some(100.0));
    }
    app.store.add_annotation(&board, texte);
    app.store.add_annotation(
        &board,
        Annotation::membrane("m", 2_000.0, -3_000.0, 6_000.0, 6_000.0),
    );
    let mut lien = Annotation::arrow("l", 0.0, 0.0, 1.0, 1.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut lien
    {
        (*source_id, *target_id) = (Some("t".into()), Some("m".into()));
    }
    app.store.add_annotation(&board, lien);
    app.une_image_sans_fenetre((1200, 800));
    let b = app.store.active_board().expect("un tableau");
    let lien = b
        .annotations
        .iter()
        .find(|a| a.id() == "l")
        .expect("le lien");
    let noeuds = NoeudsDuRendu {
        board: b,
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
        contournement: None,
    };
    let presence = super::presence::presence;
    let dans_la_membrane = presence(lien, noeuds, Some((5_000.0, 0.0)), 40);
    assert!(
        dans_la_membrane < 0.05,
        "dans la membrane, loin du texte : {dans_la_membrane}"
    );
    assert_eq!(
        presence(lien, noeuds, Some((0.0, 0.0)), 40),
        1.0,
        "sur le texte"
    );
}

/// **Au doigt, il n'y a pas de souris** : un doigt posé ne rapproche aucun lien — la souris
/// qu'on avait s'efface de la scène.
#[test]
fn test_lien_4_au_doigt_il_n_y_a_pas_de_souris() {
    let mut app = application();
    app.handle_cursor_moved(PhysicalPosition::new(850.0, 460.0));
    assert!(app.souris_dans_le_monde().is_some(), "la souris");
    app.toucher(&winit::event::Touch {
        device_id: winit::event::DeviceId::dummy(),
        phase: winit::event::TouchPhase::Started,
        location: PhysicalPosition::new(850.0, 460.0),
        force: None,
        id: 1,
    });
    assert_eq!(
        app.souris_dans_le_monde(),
        None,
        "un doigt n'est pas une souris"
    );
}

/// **Seuls les liens comptent** — sa droite parle de liens à l'écran, pas de nœuds : un lien entre
/// deux textes, seul, est plein. (Les scènes d'au-dessus ont vingt-six liens : compter aussi leurs
/// vingt-huit textes y laissait la densité à zéro, et le sabotage passait — fiche 104.)
#[test]
fn test_lien_4_seuls_les_liens_comptent() {
    let board = glucose_core::types::Board {
        annotations: vec![
            Annotation::text("a", 0.0, 0.0, "un nœud"),
            Annotation::text("b", 900.0, 0.0, "un autre"),
            Annotation::arrow("f", 0.0, 0.0, 900.0, 0.0),
        ],
        ..Default::default()
    };
    let rangs = glucose_core::quadtree::tous_les_rangs(&board);
    let n = super::presence::liens_a_l_ecran(&rangs, &board);
    assert_eq!(n, 1, "un lien, deux textes");
    assert_eq!(densite(n), 1.0, "un lien seul est plein");
}

/// **SURVOL-2 — de loin, la souris ne redessine rien** (fiche 106) : sous le plancher à pleine
/// présence (la lecture à 5 %, sous 8 %), aucun lien hors lumière ne quitte ses 8 % — la souris
/// qui passe à côté de « a », sans rien survoler, ne salit pas l'image, et la présence de ses
/// liens ne se calcule pas. **La preuve à l'envers** : à la taille réelle, le même geste la salit
/// ([`test_lien_4_la_souris_fait_redessiner`]) ; et leur opacité de loin est bien le plancher
/// ([`test_lien_4_le_plancher_vaut_aussi_de_loin`]).
#[test]
fn test_survol_2_de_loin_la_souris_ne_redessine_rien() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store.set_viewport(
        &board,
        Viewport {
            x: 600.0,
            y: 400.0,
            scale: 0.05,
        },
    );
    app.une_image_sans_fenetre((1200, 800));
    salit(&mut app, (1150.0, 780.0));
    app.une_image_sans_fenetre((1200, 800));
    // Entre les liens de « a », qui partent tous les 28° : à soixante et cent pixels de lui
    // (douze cents et deux mille unités du monde : à la taille réelle, la souris les relèverait),
    // puis de nouveau loin. Sur un lien, la main l'allumerait — et redessiner serait juste.
    for ecran in [(645.0, 440.0), (697.0, 424.0), (1150.0, 780.0)] {
        assert!(
            !salit(&mut app, ecran),
            "de loin, la souris en {ecran:?} salit l'image"
        );
        assert_eq!(
            app.ui.noeud_survole, None,
            "rien sous la souris en {ecran:?}"
        );
    }
}
