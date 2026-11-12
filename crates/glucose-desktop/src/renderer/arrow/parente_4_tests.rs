//! **PARENTE-4 — la souris ne rapproche que les liens de sens** (fiche 107) : la densité ne change
//! pas ; la proximité de la souris est multipliée par la sûreté que l'IA donne au lien — sur la voie
//! graphique et au processeur.

use super::presence::PLANCHER;
use super::*;
use crate::app::GlucoseApp;
use crate::parente::{Avis, AvisDUnNoeud};
use crate::renderer::sens::SensDesTextes;
use glucose_core::types::Viewport;
use winit::dpi::PhysicalPosition;

/// L'avis de l'IA : ces liens, de ces sûretés.
fn avis_de(liens: &[(&str, f32)]) -> Avis {
    liens
        .iter()
        .map(|&(id, surete)| {
            let a = AvisDUnNoeud {
                famille: None,
                teinte: None,
                surete,
            };
            (id.to_string(), a)
        })
        .collect()
}

/// Le sens des textes qui porte cet avis.
fn avis(liens: &[(&str, f32)]) -> SensDesTextes {
    let mut sens = SensDesTextes::default();
    sens.poser(&avis_de(liens));
    sens
}

/// Un texte au centre, `n` liens vers des textes lointains, en étoile ; la vue à la taille réelle,
/// le centre en `(600, 400)`. Le lien `f{k}` part dans la direction `k × 0,4833`.
fn etoile(n: u32) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let texte = |id: String, x: f64, y: f64| {
        let mut t = Annotation::text(id, x, y, "un nœud");
        if let Annotation::Text { width, height, .. } = &mut t {
            (*width, *height) = (Some(100.0), Some(50.0));
        }
        t
    };
    app.store
        .add_annotation(&board, texte("c".into(), -50.0, -25.0));
    for k in 0..n {
        let angle = f64::from(k) * 0.4833;
        let id = format!("t{k}");
        let bout = texte(id.clone(), 20_000.0 * angle.cos(), 20_000.0 * angle.sin());
        app.store.add_annotation(&board, bout);
        let mut f = Annotation::arrow(format!("f{k}"), 0.0, 0.0, 1.0, 1.0);
        if let Annotation::Arrow {
            source_id,
            target_id,
            ..
        } = &mut f
        {
            (*source_id, *target_id) = (Some("c".into()), Some(id));
        }
        // Posées par un script, comme la carte de Wikipédia : l'IA les juge (MAIN-1, fiche 108).
        app.store.journal.main = glucose_core::store::journal::Main::Agent("squelette".into());
        app.store.add_annotation(&board, f);
        app.store.journal.main = glucose_core::store::journal::Main::Humain;
    }
    app.store.clear_selection();
    let vue = Viewport {
        x: 600.0,
        y: 400.0,
        scale: 1.0,
    };
    app.store.set_viewport(&board, vue);
    app.une_image_sans_fenetre((1200, 800));
    app
}

/// L'opacité de l'âme de chaque lien que la carte graphique reçoit, par son identité — la souris en
/// `souris`, rien de survolé.
fn ames(app: &GlucoseApp, sens: &SensDesTextes, souris: Option<(f64, f64)>) -> Vec<(String, f32)> {
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
    let liens: Vec<String> = Visibles::nouvelles(&rangs, board)
        .annotations()
        .filter(|a| matches!(a, Annotation::Arrow { .. }))
        .map(|a| a.id().to_string())
        .collect();
    let main = Main {
        survole: None,
        souris,
    };
    let champs = fleches_a_poser(
        sens,
        (&app.store, pass, main),
        (&app.renderer.typography, &app.renderer.math),
        (1200.0, 800.0),
    );
    liens
        .into_iter()
        .zip(champs.iter().map(|c| c.ame[1]))
        .collect()
}

/// **Sur la voie graphique** : quarante liens, la densité à zéro, la souris sur le nœud central (rien
/// de survolé). Le lien sûr est plein, celui à moitié sûr à moitié, celui d'habitude au plancher ;
/// un lien que l'IA n'a pas jugé se rapproche entier, comme avant. **Sans souris, rien ne change** :
/// tous au plancher, le sûr compris — l'IA n'ajoute pas d'encre à ce que la densité borne (la loi
/// essayée d'abord doublait l'encre des flèches à « Philosophie »). **La preuve à l'envers** : sans
/// l'avis de l'IA, la souris les allume tous.
#[test]
fn test_parente_4_la_souris_ne_rapproche_que_les_liens_de_sens() {
    let app = etoile(40);
    let plein = aspect::OPACITE_DU_TRAIT;
    let sens = avis(&[("f0", 1.0), ("f1", 0.5), ("f2", 0.0)]);
    for (id, ame) in ames(&app, &sens, Some((0.0, 0.0))) {
        let attendu = match id.as_str() {
            "f1" => plein * 0.5,
            "f2" => plein * PLANCHER,
            _ => plein,
        };
        assert!((ame - attendu).abs() < 1e-6, "{id} : {ame} pour {attendu}");
    }
    for (id, ame) in ames(&app, &sens, None) {
        assert!(
            (ame - plein * PLANCHER).abs() < 1e-6,
            "{id} sans souris : {ame}"
        );
    }
    for (id, ame) in ames(&app, &SensDesTextes::default(), Some((0.0, 0.0))) {
        assert!((ame - plein).abs() < 1e-6, "{id} sans avis : {ame}");
    }
}

/// **Peu de liens : l'IA ne pâlit rien** — un lien seul, que l'IA juge d'habitude, reste plein, la
/// souris loin : les quelques flèches qu'un humain tire lui-même ne pâlissent pas par son doute.
#[test]
fn test_parente_4_un_lien_seul_reste_plein() {
    let app = etoile(1);
    let plein = aspect::OPACITE_DU_TRAIT;
    let ames = ames(&app, &avis(&[("f0", 0.0)]), Some((90_000.0, 90_000.0)));
    assert_eq!(ames.len(), 1);
    assert!((ames[0].1 - plein).abs() < 1e-6, "{ames:?}");
}

/// La plus vive des composantes autour d'un point de l'écran, dans un carré de cinq pixels.
fn le_plus_vif(image: &tiny_skia::Pixmap, (x, y): (i32, i32)) -> u8 {
    let w = image.width() as i32;
    (x - 2..=x + 2)
        .flat_map(|i| (y - 2..=y + 2).map(move |j| (i, j)))
        .map(|(i, j)| image.pixels()[(j * w + i) as usize].demultiply())
        .map(|p| p.red().max(p.green()).max(p.blue()))
        .max()
        .unwrap_or(0)
}

/// **Au processeur, par la vraie souris** : posée juste à droite du nœud central (sans le survoler),
/// elle rapproche le lien sûr, pas celui d'habitude — leurs traits, à trois cents pixels du centre.
/// **La preuve à l'envers** : avant l'avis de l'IA, les deux traits sont pareils.
#[test]
fn test_parente_4_le_processeur_suit_la_surete() {
    let mut app = etoile(40);
    // Juste à droite du nœud central, hors du rayon horizontal de l'étoile : f0, f13, f26 et f39
    // partent presque exactement vers la droite, à la hauteur de son centre.
    app.handle_cursor_moved(PhysicalPosition::new(652.0, 388.0));
    assert_eq!(app.ui.noeud_survole, None, "rien sous la souris");
    let trait_de = |app: &mut GlucoseApp, k: i32| {
        let angle = f64::from(k) * 0.4833;
        let point = (600.0 + 300.0 * angle.cos(), 400.0 + 300.0 * angle.sin());
        app.une_image_sans_fenetre((1200, 800));
        let image = app.pixmap.take().expect("une image");
        le_plus_vif(&image, (point.0 as i32, point.1 as i32))
    };
    let (avant_sur, avant_habitude) = (trait_de(&mut app, 3), trait_de(&mut app, 4));
    assert_eq!(
        avant_sur, avant_habitude,
        "avant l'avis, deux traits pareils"
    );
    // Posé sur ce que le fil a déjà lu : remplacer le tout lui ferait tout relire à l'image suivante.
    // Le rayon entier : 13 × 0,4833 ≈ 2π, et f3, f16, f29 partent dans la même direction (f4, f17,
    // f30 aussi) — un lien du rayon que l'avis ne jugerait pas s'allumerait entier au point mesuré.
    app.renderer.sens_des_textes.poser(&avis_de(&[
        ("f3", 1.0),
        ("f16", 1.0),
        ("f29", 1.0),
        ("f4", 0.0),
        ("f17", 0.0),
        ("f30", 0.0),
    ]));
    let (sur, habitude) = (trait_de(&mut app, 3), trait_de(&mut app, 4));
    assert!(sur > habitude + 40, "le sûr {sur}, l'habitude {habitude}");
}

/// **La souris ne redessine pas pour un lien d'habitude** (PARENTE-4, SURVOL-2) : quarante liens que
/// l'IA juge d'habitude, la souris qui vient de loin tout près du nœud central (sans le survoler) —
/// aucun ne change d'un niveau, donc aucune image n'est due ; refaire la carte pour un trait qui ne
/// bouge pas est la dépense que la fiche 103 a chassée. **La preuve à l'envers** : un seul lien sûr,
/// et le même geste demande une image.
#[test]
fn test_parente_4_la_souris_ne_redessine_pas_pour_un_lien_d_habitude() {
    use super::presence::{la_main_change_un_lien, liens_a_l_ecran};
    let app = etoile(40);
    let board = app.store.active_board().expect("un tableau");
    let rangs = glucose_core::quadtree::tous_les_rangs(board);
    let noeuds = NoeudsDuRendu {
        board,
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
        contournement: None,
    };
    let n = liens_a_l_ecran(&rangs, board);
    let lecture = crate::renderer::scale::WorldScale::new(1.0, 1.0).lecture();
    let mains = [
        Main {
            survole: None,
            souris: Some((90_000.0, 90_000.0)),
        },
        Main {
            survole: None,
            souris: Some((80.0, 0.0)),
        },
    ];
    let change = |sens: &SensDesTextes| {
        let liens = Visibles::nouvelles(&rangs, board).annotations();
        la_main_change_un_lien((&app.store, sens), liens, noeuds, mains, (n, lecture))
    };
    let ids: Vec<String> = (0..40).map(|k| format!("f{k}")).collect();
    let habitudes: Vec<(&str, f32)> = ids.iter().map(|id| (id.as_str(), 0.0)).collect();
    assert!(
        !change(&avis(&habitudes)),
        "des liens d'habitude ne demandent aucune image"
    );
    let mut un_sur = habitudes.clone();
    un_sur[0].1 = 1.0;
    assert!(change(&avis(&un_sur)), "un lien de sens s'allume");
}
