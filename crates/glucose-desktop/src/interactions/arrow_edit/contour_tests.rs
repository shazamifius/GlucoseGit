//! **FLECHE-5 par les vraies pièces de l'application** : une flèche qui contourne se vise là où
//! elle passe, se cherche une fois par état du document, s'écarte pendant qu'on glisse, et se
//! saisit par les étapes de son détour.

use super::*;
use crate::renderer::arrow::NoeudsDuRendu;
use glucose_core::arrow::contour::ECART;
use glucose_core::types::Annotation;

/// Une carte de texte de taille connue.
fn carte(id: &str, (x, y): (f64, f64), (w, h): (f64, f64)) -> Annotation {
    let mut a = Annotation::text(id, x, y, "carte");
    if let Annotation::Text { width, height, .. } = &mut a {
        *width = Some(w);
        *height = Some(h);
    }
    a
}

/// La carte « o », posée sur le trajet droit de « s » à « t ».
const OBSTACLE: (f64, f64, f64, f64) = (250.0, -40.0, 100.0, 140.0);

/// Deux cartes reliées par une flèche, et une troisième entre elles, sur le trajet droit.
fn application() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.folders.clear();
    }
    let (ox, oy, ow, oh) = OBSTACLE;
    for a in [
        carte("s", (0.0, 0.0), (100.0, 60.0)),
        carte("t", (600.0, 0.0), (100.0, 60.0)),
        carte("o", (ox, oy), (ow, oh)),
    ] {
        app.store.add_annotation(&board, a);
    }
    let mut fleche = Annotation::arrow("f", 50.0, 30.0, 650.0, 30.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut fleche
    {
        *source_id = Some("s".into());
        *target_id = Some("t".into());
    }
    app.store.add_annotation(&board, fleche);
    app.store.set_selected_annotation_ids(vec!["f".into()]);
    app.store.journal.clear();
    app.une_image_sans_fenetre((1200, 800));
    app
}

fn noeuds(app: &GlucoseApp) -> NoeudsDuRendu<'_> {
    NoeudsDuRendu {
        board: app.store.active_board().expect("un tableau"),
        index: Some(&app.renderer.spatial_hash),
        typographie: &app.renderer.typography,
        math: &app.renderer.math,
        contournement: Some(app.renderer.contournement()),
    }
}

fn fleche(app: &GlucoseApp) -> &Annotation {
    app.store
        .active_board()
        .and_then(|b| b.annotations.iter().find(|a| a.id() == "f"))
        .expect("la flèche")
}

fn chemin(app: &GlucoseApp) -> Vec<(f64, f64)> {
    glucose_core::arrow::path_with(fleche(app), noeuds(app)).expect("un chemin")
}

fn coudes(app: &GlucoseApp) -> Vec<(f64, f64)> {
    match fleche(app) {
        Annotation::Arrow { waypoints, .. } => waypoints.iter().map(|p| (p.x, p.y)).collect(),
        _ => Vec::new(),
    }
}

/// Un tronçon passe-t-il dans cette boîte ? Deux mille points le long de lui.
fn traverse(p: (f64, f64), q: (f64, f64), (x, y, w, h): (f64, f64, f64, f64)) -> bool {
    (1..2000).any(|k| {
        let t = k as f64 / 2000.0;
        let (px, py) = (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t);
        px > x && px < x + w && py > y && py < y + h
    })
}

/// **La carte posée sur le trajet se contourne**, par deux coins de sa boîte gonflée de
/// l'écart, et aucun tronçon ne la traverse.
#[test]
fn test_fleche_5_une_carte_sur_le_trajet_se_contourne() {
    let app = application();
    let c = chemin(&app);
    assert_eq!(c.len(), 4, "deux étapes : {c:?}");
    for s in c.windows(2) {
        assert!(!traverse(s[0], s[1], OBSTACLE), "{s:?} traverse la carte");
    }
    let (ox, oy, ow, oh) = OBSTACLE;
    let coins = [
        (ox - ECART, oy - ECART),
        (ox + ow + ECART, oy - ECART),
        (ox - ECART, oy + oh + ECART),
        (ox + ow + ECART, oy + oh + ECART),
    ];
    for etape in &c[1..3] {
        assert!(coins.contains(etape), "{etape:?} n'est pas un coin gonflé");
    }
}

/// **On vise la flèche sur son détour**, et plus sur le trajet droit qu'elle ne suit pas.
#[test]
fn test_fleche_5_on_la_vise_sur_son_detour() {
    let app = application();
    let c = chemin(&app);
    let board = app.store.active_board().expect("un tableau");
    let milieu = ((c[1].0 + c[2].0) / 2.0, (c[1].1 + c[2].1) / 2.0);
    let visee = glucose_core::arrow::at(&board.annotations, noeuds(&app), milieu, 1.0);
    assert_eq!(visee.map(|(a, _)| a.id()), Some("f"), "sur le détour");
    let ancien = (OBSTACLE.0 + OBSTACLE.2 / 2.0, 30.0);
    let visee = glucose_core::arrow::at(&board.annotations, noeuds(&app), ancien, 1.0);
    assert!(visee.is_none(), "plus sur le trajet droit, dans la carte");
}

/// **L'itinéraire se cherche une fois par état du document** : dix lectures, une recherche ;
/// la carte déplacée, une seule de plus — et la flèche va droit.
#[test]
fn test_fleche_5_l_itineraire_se_cherche_une_fois_par_etat() {
    let mut app = application();
    assert_eq!(chemin(&app).len(), 4, "le détour d'abord");
    let avant = app.renderer.itineraires.recherches();
    for _ in 0..10 {
        chemin(&app);
    }
    assert_eq!(
        app.renderer.itineraires.recherches(),
        avant,
        "rien de neuf à chercher"
    );
    let board = app.store.project.active_board_id.clone();
    app.store.set_selected_annotation_ids(vec!["o".into()]);
    app.store.move_selected(&board, 0.0, 500.0);
    app.une_image_sans_fenetre((1200, 800));
    let apres = app.renderer.itineraires.recherches();
    assert!(apres > avant, "le document a changé : cherché de nouveau");
    for _ in 0..10 {
        chemin(&app);
    }
    assert_eq!(app.renderer.itineraires.recherches(), apres);
    assert_eq!(chemin(&app).len(), 2, "la carte partie, la flèche va droit");
}

/// **Pendant qu'on glisse la carte, la flèche s'en écarte** — la version du document n'avance
/// qu'au relâchement, mais le geste en cours se voit.
#[test]
fn test_fleche_5_elle_s_ecarte_pendant_qu_on_glisse() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store.set_selected_annotation_ids(vec!["o".into()]);
    app.store.begin_live_edit();
    app.store.move_selected(&board, 0.0, 500.0);
    app.une_image_sans_fenetre((1200, 800));
    assert_eq!(
        chemin(&app).len(),
        2,
        "la carte partie : droit, en plein geste"
    );
    app.store.move_selected(&board, 0.0, -500.0);
    app.une_image_sans_fenetre((1200, 800));
    assert_eq!(
        chemin(&app).len(),
        4,
        "la carte revenue : le détour, en plein geste"
    );
    app.store.end_live_edit();
}

/// **Un simple clic sur une étape ne fige rien** : la flèche continue de contourner.
#[test]
fn test_fleche_5_un_clic_sur_une_etape_ne_fige_rien() {
    let mut app = application();
    let etape = chemin(&app)[1];
    assert!(
        app.begin_arrow_bend(etape.0, etape.1),
        "la poignée est prise"
    );
    app.finish_bend();
    assert!(coudes(&app).is_empty(), "aucun coude : {:?}", coudes(&app));
}

/// **Glisser une étape fige l'itinéraire en coudes**, l'étape tenue sous la main ; un seul
/// retour en arrière rend la flèche qui contourne.
#[test]
fn test_fleche_5_glisser_une_etape_fige_l_itineraire() {
    let mut app = application();
    let c = chemin(&app);
    assert!(app.begin_arrow_bend(c[1].0, c[1].1));
    app.update_bend(c[1].0 - 50.0, c[1].1 + 20.0);
    app.finish_bend();
    assert_eq!(
        coudes(&app),
        vec![(c[1].0 - 50.0, c[1].1 + 20.0), c[2]],
        "l'itinéraire figé, l'étape déplacée"
    );
    app.store.undo();
    assert!(coudes(&app).is_empty(), "un seul Ctrl+Z");
}

/// **Un double-clic sur une étape la retire** : l'itinéraire se fige sans elle.
#[test]
fn test_fleche_5_un_double_clic_retire_une_etape() {
    let mut app = application();
    let c = chemin(&app);
    assert!(app.begin_arrow_bend(c[1].0, c[1].1));
    app.finish_bend();
    assert!(app.begin_arrow_bend(c[1].0, c[1].1));
    assert_eq!(coudes(&app), vec![c[2]], "figé, sans l'étape");
    app.store.undo();
    assert!(coudes(&app).is_empty(), "un seul Ctrl+Z");
}

/// **Un appui au milieu d'un tronçon du détour** fige l'itinéraire et y insère un coude, comme
/// sur une flèche pliée à la main.
#[test]
fn test_fleche_5_un_appui_au_milieu_du_detour_insere_un_coude() {
    let mut app = application();
    let c = chemin(&app);
    let milieu = ((c[1].0 + c[2].0) / 2.0, (c[1].1 + c[2].1) / 2.0);
    assert!(app.begin_arrow_bend(milieu.0, milieu.1));
    app.finish_bend();
    assert_eq!(coudes(&app), vec![c[1], milieu, c[2]]);
}

/// **Ce que coûte le contournement** : quarante mille cartes en grille, mille flèches qui en
/// traversent chacune deux à cinq, et mille qui ne traversent rien. Le temps de la première
/// lecture de tous les tracés (tout se cherche), puis de la seconde (tout est retenu).
///
/// `cargo test --release -p glucose-desktop --lib mesure_du_contournement -- --ignored --nocapture`
#[test]
#[ignore = "mesure un temps : sensible à la charge de la machine"]
fn mesure_du_contournement() {
    let mut app = GlucoseApp::new();
    let (colonnes, rangees) = (200, 200);
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
        b.folders.clear();
        for k in 0..colonnes * rangees {
            let (i, j) = ((k % colonnes) as f64, (k / colonnes) as f64);
            b.annotations.push(carte(
                &format!("c{k}"),
                (i * 300.0, j * 200.0),
                (160.0, 80.0),
            ));
        }
        for n in 0..2000 {
            let rangee = (n * 7) % rangees;
            let colonne = (n * 13) % (colonnes - 6);
            // La moitié traverse deux à cinq cartes de sa rangée ; l'autre descend d'une
            // rangée, sans rien traverser.
            let (s, t) = if n % 2 == 0 {
                (
                    rangee * colonnes + colonne,
                    rangee * colonnes + colonne + 3 + n % 3,
                )
            } else {
                (
                    rangee * colonnes + colonne,
                    ((rangee + 1) % rangees) * colonnes + colonne,
                )
            };
            let mut f = Annotation::arrow(format!("f{n}"), 0.0, 0.0, 0.0, 0.0);
            if let Annotation::Arrow {
                source_id,
                target_id,
                ..
            } = &mut f
            {
                *source_id = Some(format!("c{s}"));
                *target_id = Some(format!("c{t}"));
            }
            b.annotations.push(f);
        }
    }
    app.store.bump_version();
    app.renderer.sync_spatial_index(&app.store);
    let board = app.store.active_board().expect("un tableau");
    let fleches: Vec<&Annotation> = board
        .annotations
        .iter()
        .filter(|a| matches!(a, Annotation::Arrow { .. }))
        .collect();
    let lire = |app: &GlucoseApp| {
        let debut = std::time::Instant::now();
        let mut etapes = 0;
        for f in &fleches {
            etapes += glucose_core::arrow::path_with(f, noeuds(app)).map_or(0, |c| c.len() - 2);
        }
        (debut.elapsed(), etapes)
    };
    let mut couts: Vec<(f64, usize, String)> = fleches
        .iter()
        .map(|f| {
            let debut = std::time::Instant::now();
            let n = glucose_core::arrow::path_with(f, noeuds(&app)).map_or(0, |c| c.len());
            (debut.elapsed().as_secs_f64() * 1e6, n, f.id().to_string())
        })
        .collect();
    couts.sort_by(|a, b| b.0.total_cmp(&a.0));
    let total: f64 = couts.iter().map(|c| c.0).sum();
    eprintln!(
        "MESURE total {total:.0} µs ; les plus chères : {:?}",
        &couts[..8]
    );
    eprintln!("MESURE médiane {:.1} µs", couts[couts.len() / 2].0);
    app.renderer.itineraires.oublier();
    let (froid, etapes) = lire(&app);
    let (chaud, _) = lire(&app);
    // Les droites seules, mémoire oubliée : ce que coûte une flèche que rien ne gêne.
    app.renderer.itineraires.oublier();
    let droites: Vec<&Annotation> = fleches.iter().copied().skip(1).step_by(2).collect();
    let debut = std::time::Instant::now();
    for f in &droites {
        glucose_core::arrow::path_with(f, noeuds(&app));
    }
    let par_droite = debut.elapsed().as_secs_f64() * 1e6 / droites.len() as f64;
    eprintln!("MESURE une flèche droite, cherchée : {par_droite:.1} µs");
    let detours = app.renderer.itineraires.recherches();
    eprintln!(
        "MESURE {} nœuds, {} flèches : première lecture {:?} ({:.1} µs par flèche, {etapes} \
         étapes), seconde {:?} ({:.2} µs par flèche), {detours} itinéraires cherchés",
        board.annotations.len(),
        fleches.len(),
        froid,
        froid.as_secs_f64() * 1e6 / fleches.len() as f64,
        chaud,
        chaud.as_secs_f64() * 1e6 / fleches.len() as f64,
    );
    assert!(etapes > 1000, "les flèches contournent vraiment");
}

/// La scène d'épreuve, plus une carte « loin », à l'écart de tout, et une seconde flèche droite
/// « g » de « s2 » à « t2 », sous la première.
fn application_a_deux_fleches() -> GlucoseApp {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    for a in [
        carte("loin", (0.0, 2000.0), (100.0, 60.0)),
        carte("s2", (0.0, 600.0), (100.0, 60.0)),
        carte("t2", (600.0, 600.0), (100.0, 60.0)),
    ] {
        app.store.add_annotation(&board, a);
    }
    let mut g = Annotation::arrow("g", 50.0, 630.0, 650.0, 630.0);
    if let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = &mut g
    {
        *source_id = Some("s2".into());
        *target_id = Some("t2".into());
    }
    app.store.add_annotation(&board, g);
    app.store.journal.clear();
    app.une_image_sans_fenetre((1200, 800));
    app
}

fn chemin_de(app: &GlucoseApp, id: &str) -> Vec<(f64, f64)> {
    let board = app.store.active_board().expect("un tableau");
    let f = board.annotations.iter().find(|a| a.id() == id).expect(id);
    glucose_core::arrow::path_with(f, noeuds(app)).expect("un chemin")
}

/// **Pendant un glisser, une carte sans rapport ne fait rien chercher** : ni « f » ni « g » ne
/// l'ont lue, elle ne barre aucun de leurs chemins.
#[test]
fn test_fleche_5_un_glisser_sans_rapport_ne_cherche_rien() {
    let mut app = application_a_deux_fleches();
    let (f, g) = (chemin_de(&app, "f"), chemin_de(&app, "g"));
    let avant = app.renderer.itineraires.recherches();
    let board = app.store.project.active_board_id.clone();
    app.store.set_selected_annotation_ids(vec!["loin".into()]);
    app.store.begin_live_edit();
    for _ in 0..5 {
        app.store.move_selected(&board, 40.0, 10.0);
        app.une_image_sans_fenetre((1200, 800));
        assert_eq!(
            (chemin_de(&app, "f"), chemin_de(&app, "g")),
            (f.clone(), g.clone())
        );
    }
    app.store.end_live_edit();
    assert_eq!(
        app.renderer.itineraires.recherches(),
        avant,
        "aucune recherche pendant le geste"
    );
}

/// **Une carte glissée sur le chemin d'une flèche la fait contourner, en plein geste** — et
/// seule cette flèche-là se recherche.
#[test]
fn test_fleche_5_une_carte_glissee_sur_un_chemin_le_fait_contourner() {
    let mut app = application_a_deux_fleches();
    assert_eq!(chemin_de(&app, "g").len(), 2, "g va droit");
    let f = chemin_de(&app, "f");
    let avant = app.renderer.itineraires.recherches();
    let board = app.store.project.active_board_id.clone();
    app.store.set_selected_annotation_ids(vec!["loin".into()]);
    app.store.begin_live_edit();
    // De (0, 2000) à (300, 600) : en travers du chemin de g.
    app.store.move_selected(&board, 300.0, -1400.0);
    app.une_image_sans_fenetre((1200, 800));
    assert_eq!(
        chemin_de(&app, "g").len(),
        4,
        "g contourne la carte glissée"
    );
    assert_eq!(chemin_de(&app, "f"), f, "f n'a pas bougé");
    assert_eq!(
        app.renderer.itineraires.recherches(),
        avant + 1,
        "seule g s'est recherchée"
    );
    app.store.end_live_edit();
}

/// **Un geste annulé rend le détour** : le document revient à l'état d'avant sans nouvelle
/// version, et ce qui s'est cherché pendant le geste ne vaut plus.
#[test]
fn test_fleche_5_un_geste_annule_rend_le_detour() {
    let mut app = application();
    let detour = chemin(&app);
    let board = app.store.project.active_board_id.clone();
    app.store.set_selected_annotation_ids(vec!["o".into()]);
    app.store.begin_live_edit();
    app.store.move_selected(&board, 0.0, 500.0);
    app.une_image_sans_fenetre((1200, 800));
    assert_eq!(chemin(&app).len(), 2, "la carte partie : droit");
    assert!(app.store.cancel_live_edit());
    app.une_image_sans_fenetre((1200, 800));
    assert_eq!(chemin(&app), detour, "la carte revenue : le détour d'avant");
}

/// **Une photo penchée se contourne par sa boîte englobante** : penchée d'un huitième de tour,
/// elle déborde largement de sa boîte droite, et la flèche passe au large de tout ce qu'elle
/// couvre.
#[test]
fn test_fleche_5_une_photo_penchee_se_contourne() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store.remove_annotations(&board, &["o"]);
    let mut photo = glucose_core::types::BoardImage::new("p", 350.0, 30.0, 220.0, 40.0);
    photo.rotation = std::f64::consts::FRAC_PI_4;
    let englobante = photo.bounds();
    if let Some(b) = app.store.active_board_mut() {
        b.images.push(photo);
    }
    app.store.bump_version();
    app.une_image_sans_fenetre((1200, 800));
    let c = chemin(&app);
    assert!(c.len() > 2, "la photo se contourne : {c:?}");
    let zone = (
        englobante.left,
        englobante.top,
        englobante.width,
        englobante.height,
    );
    for s in c.windows(2) {
        assert!(
            !traverse(s[0], s[1], zone),
            "{s:?} traverse la photo penchée"
        );
    }
}

/// **Une flèche pliée à la main passe par où la main l'a dit** : elle ne contourne pas, même
/// quand son chemin traverse une carte.
#[test]
fn test_fleche_5_une_fleche_pliee_a_la_main_ne_contourne_pas() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store.insert_arrow_bend(&board, "f", 0, (300.0, 30.0));
    app.une_image_sans_fenetre((1200, 800));
    let c = chemin(&app);
    assert_eq!(c.len(), 3, "le seul coude posé : {c:?}");
    assert_eq!(c[1], (300.0, 30.0));
}

/// **Figer ne touche pas une flèche déjà pliée** : ce sont ses coudes que la main tient.
#[test]
fn test_fleche_5_figer_ne_touche_pas_une_fleche_pliee() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store.insert_arrow_bend(&board, "f", 0, (300.0, 30.0));
    app.store
        .figer_arrow_route(&board, "f", &[(1.0, 2.0), (3.0, 4.0)]);
    assert_eq!(coudes(&app), vec![(300.0, 30.0)]);
}

/// **Un nœud posé sur un bout ne relance rien** : la recherche l'ignorerait — on ne sort pas
/// d'une boîte où l'on est déjà —, l'itinéraire tient donc tel quel.
#[test]
fn test_fleche_5_un_noeud_pose_sur_un_bout_ne_relance_rien() {
    let mut app = application_a_deux_fleches();
    let f = chemin_de(&app, "f");
    let avant = app.renderer.itineraires.recherches();
    let board = app.store.project.active_board_id.clone();
    app.store.set_selected_annotation_ids(vec!["loin".into()]);
    app.store.begin_live_edit();
    // De (0, 2000) à (0, 0) : exactement sur la carte de départ de f.
    app.store.move_selected(&board, 0.0, -2000.0);
    app.une_image_sans_fenetre((1200, 800));
    assert_eq!(chemin_de(&app, "f"), f);
    assert_eq!(
        app.renderer.itineraires.recherches(),
        avant,
        "rien à chercher"
    );
    app.store.end_live_edit();
}

/// **Une flèche libre contourne aussi** : sans carte à ses bouts, elle évite ce qu'elle
/// traverserait, comme chez Tauri.
#[test]
fn test_fleche_5_une_fleche_libre_contourne_aussi() {
    let mut app = application();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::arrow("libre", 200.0, 30.0, 400.0, 30.0));
    app.une_image_sans_fenetre((1200, 800));
    let c = chemin_de(&app, "libre");
    assert_eq!(
        (c[0], c[c.len() - 1]),
        ((200.0, 30.0), (400.0, 30.0)),
        "ses bouts"
    );
    assert_eq!(c.len(), 4, "la carte « o » contournée : {c:?}");
}

/// **Ce que coûte le grand tour d'un mur** : des photos de 200 unités, serrées à 5 d'écart, entre
/// les deux cartes — vingt de côté, puis cent — par le vrai index de l'application.
///
/// `cargo test --release -p glucose-desktop --lib mesure_du_mur -- --ignored --nocapture`
#[test]
#[ignore = "mesure un temps : sensible à la charge de la machine"]
fn mesure_du_mur() {
    for cote in [20, 100] {
        let mut app = application();
        let board = app.store.project.active_board_id.clone();
        app.store.remove_annotations(&board, &["o"]);
        if let Some(b) = app.store.active_board_mut() {
            b.annotations.retain(|a| a.id() != "t");
            b.annotations.push(carte(
                "t",
                (320.0 + cote as f64 * 205.0, 0.0),
                (100.0, 60.0),
            ));
            for k in 0..cote * cote {
                let (i, j) = ((k % cote) as f64, (k / cote) as f64);
                let (x, y) = (300.0 + i * 205.0, 30.0 - cote as f64 * 102.5 + j * 205.0);
                let photo =
                    glucose_core::types::BoardImage::new(format!("p{k}"), x, y, 200.0, 200.0);
                b.images.push(photo);
            }
        }
        app.store.bump_version();
        app.renderer.sync_spatial_index(&app.store);
        let debut = std::time::Instant::now();
        let c = chemin(&app);
        let froid = debut.elapsed();
        eprintln!(
            "MESURE mur de {cote}×{cote} photos : cherché {froid:?}, {} étapes",
            c.len() - 2
        );
    }
}
