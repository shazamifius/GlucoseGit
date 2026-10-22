//! Chaque outil, joué sur une application d'épreuve — sans fenêtre, sans porte.

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use crate::epreuve::Dossier;
use crate::interactions::links::tests::pages_ouvertes;
use glucose_core::types::Annotation;

fn args(texte: &str) -> Json {
    Json::lire(texte).expect("des arguments en JSON")
}

/// Appelle l'outil, et rend son résultat — l'épreuve tombe s'il échoue.
fn appeler(app: &mut GlucoseApp, nom: &str, arguments: &str) -> Json {
    app.executer_pour_un_agent(nom, &args(arguments))
        .unwrap_or_else(|e| panic!("{nom} a échoué : {e}"))
}

fn board(app: &GlucoseApp) -> String {
    app.store.project.active_board_id.clone()
}

fn texte_de(app: &GlucoseApp, id: &str) -> Option<String> {
    app.store.project.annotation(&board(app), id)?.own_text()
}

fn nombre(v: &Json, chemin: &[&str]) -> f64 {
    let mut v = v;
    for c in chemin {
        v = v
            .get(c)
            .unwrap_or_else(|| panic!("{c} manque dans {}", v.ecrire()));
    }
    v.as_f64().expect("un nombre")
}

#[test]
fn un_texte_pose_dans_une_membrane_y_appartient_et_la_fait_grandir() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"bulle","titre":"Recherche","x":0,"y":0}"#,
    );
    let long = "Une phrase qui revient. ".repeat(60);
    let r = appeler(
        &mut app,
        "glucose_poser_texte",
        &format!(r#"{{"id":"doc","texte":"{long}","dans":"bulle"}}"#),
    );
    let hauteur = nombre(&r, &["boite", "hauteur"]);
    assert!(
        hauteur > 300.0,
        "la hauteur est celle du texte mesuré : {hauteur}"
    );
    let b = board(&app);
    let doc = app.store.project.annotation(&b, "doc").expect("doc");
    assert_eq!(doc.membrane_id(), Some("bulle"));
    let bulle = app
        .store
        .project
        .annotation(&b, "bulle")
        .expect("bulle")
        .bounds();
    assert!(bulle.bottom() > doc.bounds().bottom(), "{bulle:?}");
}

#[test]
fn lire_rend_le_tableau_et_dans_ne_rend_que_la_membrane() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"bulle","x":0,"y":0}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"dedans","texte":"a","dans":"bulle"}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"dehors","texte":"b","x":5000,"y":0}"#,
    );
    let tout = appeler(&mut app, "glucose_lire", "{}");
    let ids = |v: &Json| -> Vec<String> {
        v.get("noeuds")
            .and_then(Json::as_liste)
            .expect("des nœuds")
            .iter()
            .filter_map(|n| n.get("id").and_then(Json::as_str).map(str::to_string))
            .collect()
    };
    for id in ["bulle", "dedans", "dehors"] {
        assert!(
            ids(&tout).contains(&id.to_string()),
            "{id} dans {}",
            tout.ecrire()
        );
    }
    let dedans = appeler(&mut app, "glucose_lire", r#"{"dans":"bulle"}"#);
    let mut vus = ids(&dedans);
    vus.sort();
    assert_eq!(vus, ["bulle", "dedans"]);
    assert!(tout.get("visible").is_some() && tout.get("tableaux").is_some());
}

/// **Lire par niveaux** (fiche 79) : le sommaire ne porte pas les textes, `ids` les rend en entier,
/// `cherche` rend la phrase trouvée. Le 21/10/2026, Protéo avait lu un manuel de 1 037 lignes d'un
/// bloc : son sommaire tient en une phrase.
#[test]
fn lire_par_niveaux_ne_rend_que_ce_qu_on_demande() {
    let mut app = GlucoseApp::new();
    let manuel = format!(
        "# MeshForge — manuel. {}",
        "Une ligne du manuel.\n".repeat(1037)
    );
    let poser = format!(
        r#"{{"id":"manuel","texte":{}}}"#,
        Json::from(manuel.as_str()).ecrire()
    );
    appeler(&mut app, "glucose_poser_texte", &poser);
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"note","texte":"Le portage existe. La commande polyDissolve n'existe pas dans Maya.","x":5000,"y":0}"#,
    );
    let noeud = |v: &Json, id: &str| -> Json {
        v.get("noeuds")
            .and_then(Json::as_liste)
            .and_then(|l| {
                l.iter()
                    .find(|n| n.get("id").and_then(Json::as_str) == Some(id))
                    .cloned()
            })
            .unwrap_or_else(|| panic!("{id} dans {}", v.ecrire()))
    };
    let sommaire = appeler(&mut app, "glucose_lire", "{}");
    let m = noeud(&sommaire, "manuel");
    assert!(
        m.get("texte").is_none(),
        "le sommaire ne porte pas le texte"
    );
    assert_eq!(
        m.get("debut").and_then(Json::as_str),
        Some("# MeshForge — manuel.")
    );
    assert_eq!(
        m.get("taille").and_then(Json::as_f64),
        Some(manuel.chars().count() as f64)
    );
    let complet = appeler(&mut app, "glucose_lire", r#"{"niveau":"complet"}"#);
    assert!(
        sommaire.ecrire().len() * 20 < complet.ecrire().len(),
        "le sommaire est au moins vingt fois plus court : {} contre {}",
        sommaire.ecrire().len(),
        complet.ecrire().len()
    );
    let entier = appeler(&mut app, "glucose_lire", r#"{"ids":["note"]}"#);
    assert_eq!(
        entier
            .get("noeuds")
            .and_then(Json::as_liste)
            .map(<[Json]>::len),
        Some(1)
    );
    assert!(
        noeud(&entier, "note").get("texte").is_some(),
        "ids : le texte entier"
    );
    let trouve = appeler(&mut app, "glucose_lire", r#"{"cherche":"polydissolve"}"#);
    assert_eq!(
        noeud(&trouve, "note").get("passage").and_then(Json::as_str),
        Some("La commande polyDissolve n'existe pas dans Maya.")
    );
    assert_eq!(
        trouve
            .get("noeuds")
            .and_then(Json::as_liste)
            .map(<[Json]>::len),
        Some(1),
        "seul le nœud qui contient les mots"
    );
}

fn liste_de(v: &Json, cle: &str) -> Vec<String> {
    v.get(cle)
        .and_then(Json::as_liste)
        .unwrap_or_else(|| panic!("{cle} dans {}", v.ecrire()))
        .iter()
        .filter_map(Json::as_str)
        .map(str::to_string)
        .collect()
}

/// **Il montre du doigt** : ce qu'il a sélectionné, l'agent le lit — sur le tableau qu'il
/// regarde, pas ailleurs.
#[test]
fn la_selection_de_l_utilisateur_se_lit() {
    let mut app = GlucoseApp::new();
    // Au démarrage, le nœud de bienvenue est sélectionné : poser ne change rien à cela (AGENT-2).
    let au_depart = app.store.selected_annotation_ids.clone();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"a","texte":"Un."}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"b","texte":"Deux."}"#,
    );
    assert_eq!(
        liste_de(&appeler(&mut app, "glucose_lire", "{}"), "selection"),
        au_depart
    );
    app.store.selected_annotation_ids = vec!["b".into()];
    let lu = appeler(&mut app, "glucose_lire", "{}");
    assert_eq!(liste_de(&lu, "selection"), ["b"], "« ça », c'est b");
    app.store.selected_image_ids = vec!["une-photo".into()];
    let avec_photo = appeler(&mut app, "glucose_lire", "{}");
    assert_eq!(liste_de(&avec_photo, "selection"), ["une-photo", "b"]);
    app.store.selected_image_ids.clear();
    let autre = app.store.add_board("Ailleurs");
    let ailleurs = appeler(
        &mut app,
        "glucose_lire",
        &format!(r#"{{"tableau":"{autre}"}}"#),
    );
    assert!(
        liste_de(&ailleurs, "selection").is_empty(),
        "sa sélection n'est pas sur l'autre tableau"
    );
    assert_eq!(app.store.selected_annotation_ids, ["b"], "lue, pas touchée");
    // L'identité du document : un agent range ce qu'il a vu sous elle, jamais sous `main` seul.
    let doc = lu
        .get("document")
        .and_then(Json::as_str)
        .expect("le document");
    assert_eq!(doc, app.store.identite);
}

/// **Ce qu'il a récrit se voit sans tout relire** : l'empreinte d'un nœud est la même tant que
/// son texte l'est, à tout niveau de lecture, et change quand il le récrit.
#[test]
fn l_empreinte_dit_ce_qui_a_ete_recrit() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"rapport","texte":"La citation trois est juste. Fin."}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"autre","texte":"Rien."}"#,
    );
    let empreinte_de = |v: &Json, id: &str| -> String {
        v.get("noeuds")
            .and_then(Json::as_liste)
            .and_then(|l| {
                l.iter()
                    .find(|n| n.get("id").and_then(Json::as_str) == Some(id))
            })
            .and_then(|n| n.get("empreinte"))
            .and_then(Json::as_str)
            .unwrap_or_else(|| panic!("l'empreinte de {id} dans {}", v.ecrire()))
            .to_string()
    };
    let avant = appeler(&mut app, "glucose_lire", "{}");
    let entier = appeler(&mut app, "glucose_lire", r#"{"ids":["rapport"]}"#);
    assert_eq!(
        empreinte_de(&avant, "rapport"),
        empreinte_de(&entier, "rapport"),
        "la même au sommaire et en entier"
    );
    appeler(
        &mut app,
        "glucose_ecrire",
        r#"{"id":"rapport","texte":"La citation trois est fausse. Fin."}"#,
    );
    let apres = appeler(&mut app, "glucose_lire", "{}");
    assert_ne!(
        empreinte_de(&avant, "rapport"),
        empreinte_de(&apres, "rapport")
    );
    assert_eq!(
        empreinte_de(&avant, "autre"),
        empreinte_de(&apres, "autre"),
        "ce qui n'a pas changé garde la sienne"
    );
}

/// **Son carnet** : un onglet à l'agent, où il pose avec « tableau », sans que l'utilisateur
/// change de regard.
#[test]
fn un_agent_ouvre_son_tableau_et_y_pose() {
    let mut app = GlucoseApp::new();
    let regarde = board(&app);
    let r = appeler(&mut app, "glucose_ouvrir_tableau", r#"{"nom":"Carnet"}"#);
    let carnet = r
        .get("tableau")
        .and_then(Json::as_str)
        .expect("son id")
        .to_string();
    assert_eq!(r.get("cree").and_then(Json::as_bool), Some(true));
    let encore = appeler(&mut app, "glucose_ouvrir_tableau", r#"{"nom":"Carnet"}"#);
    assert_eq!(
        encore.get("tableau").and_then(Json::as_str),
        Some(carnet.as_str())
    );
    let poser = format!(r#"{{"id":"lecon","texte":"Une leçon.","tableau":"{carnet}"}}"#);
    appeler(&mut app, "glucose_poser_texte", &poser);
    assert!(app.store.project.annotation(&carnet, "lecon").is_some());
    assert_eq!(board(&app), regarde, "il regarde toujours le sien");
    assert!(app
        .executer_pour_un_agent("glucose_ouvrir_tableau", &args(r#"{"nom":"  "}"#))
        .is_err());
}

#[test]
fn ecrire_allonge_le_document_et_sa_hauteur_suit() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r##"{"id":"doc","texte":"# Rapport"}"##,
    );
    let avant = app
        .store
        .project
        .annotation(&board(&app), "doc")
        .expect("doc")
        .bounds();
    let suite = "\\n\\nUne ligne de plus.".repeat(20);
    appeler(
        &mut app,
        "glucose_ecrire",
        &format!(r#"{{"id":"doc","texte":"{suite}","ajouter":true}}"#),
    );
    let apres = app
        .store
        .project
        .annotation(&board(&app), "doc")
        .expect("doc")
        .bounds();
    assert!(texte_de(&app, "doc")
        .expect("texte")
        .starts_with("# Rapport\n\nUne ligne"));
    assert!(apres.height > avant.height, "{avant:?} → {apres:?}");
    assert_eq!(
        (apres.left, apres.top),
        (avant.left, avant.top),
        "le document ne bouge pas"
    );
}

#[test]
fn relier_surligne_les_passages_et_refuse_une_relation_inconnue() {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"r","texte":"Le glucose est un sucre [1]."}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"p","texte":"Un ose à six carbones."}"#,
    );
    let refus = app.executer_pour_un_agent(
        "glucose_relier",
        &args(r#"{"source":"r","cible":"p","relation":"aime"}"#),
    );
    assert!(refus.is_err_and(|e| e.contains("aime")));
    appeler(
        &mut app,
        "glucose_relier",
        r#"{"id":"f","source":"r","cible":"p","passage_source":"[1]","passage_cible":"six carbones","relation":"depend_de"}"#,
    );
    let Some(Annotation::Arrow {
        source_text_sel,
        target_text_sel,
        predicate,
        ..
    }) = app.store.project.annotation(&board(&app), "f")
    else {
        panic!("la flèche");
    };
    assert!(source_text_sel.is_some() && target_text_sel.is_some());
    assert_eq!(predicate.map(|p| p.as_str()), Some("depend_de"));
}

#[test]
fn une_page_se_pose_avec_sa_capture_et_le_double_clic_l_ouvre() {
    let mut app = GlucoseApp::new();
    let dossier = Dossier::nouveau("agents-page");
    let capture = dossier.join("capture.png");
    image::RgbaImage::from_pixel(1280, 800, image::Rgba([200, 100, 50, 255]))
        .save(&capture)
        .expect("une capture");
    let chemin = capture.to_string_lossy().replace('\\', "/");
    let r = appeler(
        &mut app,
        "glucose_poser_page",
        &format!(
            r#"{{"id":"wiki","adresse":"https://fr.wikipedia.org/wiki/Glucose","titre":"Glucose — Wikipédia","capture":"{chemin}","largeur":400}}"#
        ),
    );
    assert_eq!(r.get("photo").and_then(Json::as_str), Some("wiki"));
    assert_eq!(r.get("texte").and_then(Json::as_str), Some("wiki-texte"));
    let b = board(&app);
    let photo = app.store.image(&b, "wiki").expect("la photo");
    assert_eq!(
        photo.source_url.as_deref(),
        Some("https://fr.wikipedia.org/wiki/Glucose")
    );
    assert_eq!(
        (photo.width, photo.height),
        (400.0, 250.0),
        "aux proportions de la capture"
    );
    let legende = texte_de(&app, "wiki-texte").expect("la légende");
    assert!(legende.contains("**Glucose — Wikipédia**"));
    assert!(legende.contains("[fr.wikipedia.org](https://fr.wikipedia.org/wiki/Glucose)"));
    // La capture est confiée au document : le fichier de l'agent peut disparaître.
    drop(dossier);
    assert!(app.ouvrir_la_page_d_une_photo("wiki"));
    assert_eq!(pages_ouvertes(), ["https://fr.wikipedia.org/wiki/Glucose"]);
}

#[test]
fn une_page_sans_capture_n_est_qu_un_texte_et_une_adresse_douteuse_est_refusee() {
    let mut app = GlucoseApp::new();
    let r = appeler(
        &mut app,
        "glucose_poser_page",
        r#"{"id":"p","adresse":"https://exemple.fr/a"}"#,
    );
    assert_eq!(r.get("photo"), Some(&Json::Nul));
    assert!(texte_de(&app, "p-texte").is_some());
    for douteuse in ["javascript:alert(1)", "file:///C:/Windows/calc.exe"] {
        let refus = app.executer_pour_un_agent(
            "glucose_poser_page",
            &args(&format!(r#"{{"adresse":"{douteuse}"}}"#)),
        );
        assert!(refus.is_err(), "{douteuse}");
    }
    let manquante = app.executer_pour_un_agent(
        "glucose_poser_page",
        &args(r#"{"id":"q","adresse":"https://exemple.fr","capture":"Z:/rien/du/tout.png"}"#),
    );
    assert!(manquante.is_err());
    assert!(texte_de(&app, "q-texte").is_none(), "rien à moitié");
}

#[test]
fn chaque_appel_est_un_geste_et_sa_selection_reste_la_sienne() {
    let mut app = GlucoseApp::new();
    let b = board(&app);
    app.store
        .add_annotation(&b, Annotation::text("sien", 0.0, 0.0, "à lui"));
    app.store.journal.clear();
    let profondeur = app.store.undo_depth();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"a","texte":"un"}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"b","texte":"deux"}"#,
    );
    assert_eq!(app.store.undo_depth(), profondeur + 2);
    assert_eq!(app.store.selected_annotation_ids, ["sien"]);
    assert!(app.store.undo());
    assert!(app.store.project.annotation(&b, "b").is_none());
    assert!(app.store.project.annotation(&b, "a").is_some());
}

#[test]
fn une_couleur_qui_n_en_est_pas_une_est_refusee() {
    let mut app = GlucoseApp::new();
    let refus = app.executer_pour_un_agent(
        "glucose_poser_texte",
        &args(r#"{"texte":"x","couleur":"rouge"}"#),
    );
    assert!(refus.is_err_and(|e| e.contains("rouge")));
    appeler(
        &mut app,
        "glucose_poser_texte",
        r##"{"id":"c","texte":"x","couleur":"#aa3344"}"##,
    );
}

#[test]
fn chaque_outil_annonce_s_execute() {
    let mut app = GlucoseApp::new();
    for nom in crate::agents::outils::noms() {
        let issue = app.executer_pour_un_agent(nom, &Json::objet([]));
        if let Err(e) = issue {
            assert!(
                !e.contains("pas d'outil"),
                "{nom} est annoncé sans être exécuté"
            );
        }
    }
}

/// Une photo de page, posée loin de la carte d'accueil, la vue cadrée sur elle ; et le point de
/// l'écran qui la touche.
fn une_page_a_l_ecran(app: &mut GlucoseApp, adresse: &str) -> (f64, f64) {
    let b = board(app);
    let mut img = glucose_core::types::BoardImage::new("page", 5000.0, 5000.0, 400.0, 250.0);
    img.source_url = Some(adresse.to_string());
    app.store.add_image(&b, img);
    app.store.clear_selection();
    let (w, h) = app.taille_de_la_fenetre();
    let vue = glucose_core::types::Viewport {
        x: f64::from(w) / 2.0 - 5000.0,
        y: f64::from(h) / 2.0 - 5000.0,
        scale: 1.0,
    };
    app.store.set_viewport(&b, vue);
    crate::canvas::world_to_screen(5000.0, 5000.0, &app.store.viewport())
}

fn cliquer(app: &mut GlucoseApp, (sx, sy): (f64, f64)) {
    let (w, h) = app.taille_de_la_fenetre();
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(sx, sy));
    app.handle_mouse_down(winit::event::MouseButton::Left, w, h);
    app.handle_mouse_up(winit::event::MouseButton::Left);
}

#[test]
fn le_vrai_double_clic_sur_une_page_l_ouvre_et_un_seul_clic_la_choisit() {
    let mut app = GlucoseApp::new();
    let ou = une_page_a_l_ecran(&mut app, "https://exemple.fr/page");
    cliquer(&mut app, ou);
    assert!(
        pages_ouvertes().is_empty(),
        "un clic la choisit, il ne l'ouvre pas"
    );
    assert_eq!(app.store.selected_image_ids, ["page"]);
    cliquer(&mut app, ou);
    assert_eq!(pages_ouvertes(), ["https://exemple.fr/page"]);
}

#[test]
fn ctrl_clic_sur_une_page_l_ouvre_comme_un_lien() {
    let mut app = GlucoseApp::new();
    let ou = une_page_a_l_ecran(&mut app, "https://exemple.fr/ctrl");
    app.modifiers = winit::keyboard::ModifiersState::CONTROL;
    cliquer(&mut app, ou);
    assert_eq!(pages_ouvertes(), ["https://exemple.fr/ctrl"]);
}

#[test]
fn une_photo_venue_d_un_fichier_ne_s_ouvre_pas_comme_une_page() {
    let mut app = GlucoseApp::new();
    une_page_a_l_ecran(&mut app, "file:///C:/Windows/System32/calc.exe");
    assert!(!app.ouvrir_la_page_d_une_photo("page"));
    assert!(pages_ouvertes().is_empty());
}

#[test]
fn ce_qu_un_agent_pose_demande_une_image() {
    let mut app = GlucoseApp::new();
    let _ = app.prendre_la_salissure();
    appeler(&mut app, "glucose_poser_texte", r#"{"texte":"x"}"#);
    assert!(matches!(
        app.prendre_la_salissure(),
        crate::salissure::Salissure::Tout
    ));
}
