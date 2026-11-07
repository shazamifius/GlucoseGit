//! Les domaines à la porte des agents : chaque outil, sur une application d'épreuve, et ce que le
//! recensement sait d'autres fichiers.

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::recensement::recenser;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, Domain};

fn args(texte: &str) -> Json {
    Json::lire(texte).expect("des arguments en JSON")
}

fn appeler(app: &mut GlucoseApp, nom: &str, arguments: &str) -> Json {
    app.executer_pour_un_agent(nom, &args(arguments))
        .unwrap_or_else(|e| panic!("{nom} a échoué : {e}"))
}

/// Ce que l'appel doit refuser : rend sa phrase.
fn refus(app: &mut GlucoseApp, nom: &str, arguments: &str) -> String {
    match app.executer_pour_un_agent(nom, &args(arguments)) {
        Err(e) => e,
        Ok(r) => panic!("{nom} devait refuser, et rend {}", r.ecrire()),
    }
}

fn liste<'j>(v: &'j Json, cle: &str) -> &'j [Json] {
    v.get(cle)
        .and_then(Json::as_liste)
        .unwrap_or_else(|| panic!("« {cle} » manque dans {}", v.ecrire()))
}

fn texte<'j>(v: &'j Json, cle: &str) -> &'j str {
    v.get(cle)
        .and_then(Json::as_str)
        .unwrap_or_else(|| panic!("« {cle} » manque dans {}", v.ecrire()))
}

fn nombre(v: &Json, cle: &str) -> f64 {
    v.get(cle)
        .and_then(Json::as_f64)
        .unwrap_or_else(|| panic!("« {cle} » manque dans {}", v.ecrire()))
}

/// Une application dont le tableau porte deux textes.
fn app_avec_deux_textes() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"a","texte":"L'eau, ça mouille. Toujours."}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"b","texte":"Eau = H2O."}"#,
    );
    app
}

/// Ce qu'un autre fichier dit : un vrai document, écrit par le vrai noyau, recensé sous ce chemin.
/// Rend la définition du domaine qu'il porte.
fn autre_fichier(
    app: &mut GlucoseApp,
    chemin: &str,
    (nom, naissance): (&str, i64),
    (noeud, ecrit): (&str, &str),
) -> Domain {
    let mut s = Store::new("Autre");
    let board = s.project.active_board_id.clone();
    s.add_annotation(&board, Annotation::text(noeud, 0.0, 0.0, ecrit));
    let id = s
        .creer_un_domaine(nom, ("#123456".into(), "AUT".into()), naissance)
        .expect("créé");
    s.try_assign_domain_to_node(&board, noeud, &id, 0.9)
        .expect("porté");
    app.disque
        .recenseur
        .etat
        .poser(recenser(chemin, &s.project));
    s.domain(&id).expect("là").clone()
}

/// **Lu comme il est vu** : chaque nœud dit ses domaines et leur poids ; le catalogue dit combien de
/// nœuds les portent ; un nœud sans domaine ne coûte aucun mot.
#[test]
fn lire_dit_les_domaines() {
    let mut app = app_avec_deux_textes();
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"Eau","ids":["a"],"poids":0.8}"#,
    );
    let lu = appeler(&mut app, "glucose_lire", "{}");
    let noeud = |id: &str| {
        liste(&lu, "noeuds")
            .iter()
            .find(|n| texte(n, "id") == id)
            .expect("le nœud")
    };
    let portes = liste(noeud("a"), "domaines");
    assert_eq!(texte(&portes[0], "domaine"), "Eau");
    assert_eq!(nombre(&portes[0], "poids"), 0.8);
    assert!(noeud("b").get("domaines").is_none(), "aucun mot pour rien");
    let catalogue = liste(&lu, "domaines");
    assert_eq!(texte(&catalogue[0], "nom"), "Eau");
    assert_eq!(nombre(&catalogue[0], "noeuds"), 1.0);
    assert!(catalogue[0].get("a_preciser").is_none());
}

/// **`domaine` ne garde que ses porteurs**, et un domaine inconnu est une faute qui dit quoi faire ;
/// il ne se crée jamais à la lecture.
#[test]
fn lire_par_domaine_ne_rend_que_ses_noeuds() {
    let mut app = app_avec_deux_textes();
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"Eau","ids":["a"]}"#,
    );
    let lu = appeler(&mut app, "glucose_lire", r#"{"domaine":" eau "}"#);
    let ids: Vec<_> = liste(&lu, "noeuds")
        .iter()
        .map(|n| texte(n, "id"))
        .collect();
    assert_eq!(ids, ["a"]);
    let faute = refus(&mut app, "glucose_lire", r#"{"domaine":"feu"}"#);
    assert!(faute.contains("feu") && faute.contains("Eau"), "{faute}");
    assert_eq!(app.store.project.domains.len(), 1, "rien de créé");
}

/// **Poser** : un geste, visible au panneau (qui lit le même catalogue), annulé d'un seul `Ctrl+Z`.
#[test]
fn poser_un_domaine_est_un_geste_annulable() {
    let mut app = app_avec_deux_textes();
    let profondeur = app.store.undo_depth();
    let r = appeler(
        &mut app,
        "glucose_poser_domaine",
        r##"{"nouveau":"Eau","ids":["a","b"],"poids":0.5,"couleur":"#102030"}"##,
    );
    assert_eq!(texte(&r, "domaine"), "Eau");
    assert_eq!(r.get("cree"), Some(&Json::Bool(true)));
    assert_eq!(nombre(&r, "noeuds"), 2.0);
    assert_eq!(app.store.undo_depth(), profondeur + 1, "un geste");
    assert_eq!(app.store.project.domains[0].color, "#102030");
    app.store.undo();
    assert!(app.store.project.domains.is_empty());
    let board = app.store.project.active_board_id.clone();
    assert!(app.store.node_domains(&board, "a").unwrap().is_empty());
}

/// **Sa règle, pour une IA** : créer un nom pris est refusé ; la réponse dit ce qu'il désigne, par un
/// exemple tiré de ses nœuds, et les deux réponses possibles. Rien n'a bougé. Puis le mot précisé, et
/// le même sujet par « domaine » : ses deux issues.
#[test]
fn creer_un_nom_pris_pose_la_question() {
    let mut app = app_avec_deux_textes();
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"Python","ids":["a"]}"#,
    );
    let profondeur = app.store.undo_depth();
    let question = refus(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"python","ids":["b"]}"#,
    );
    assert!(question.contains("existe déjà"), "{question}");
    assert!(
        question.contains("L'eau, ça mouille."),
        "son exemple : {question}"
    );
    assert!(
        question.contains("« domaine »") && question.contains("précise"),
        "{question}"
    );
    assert_eq!(app.store.project.domains.len(), 1);
    assert_eq!(app.store.undo_depth(), profondeur);

    let board = app.store.project.active_board_id.clone();
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"python serpent","ids":["b"]}"#,
    );
    assert_eq!(app.store.node_domains(&board, "b").unwrap().len(), 1);
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"PYTHON","ids":["b"]}"#,
    );
    assert_eq!(
        app.store.node_domains(&board, "b").unwrap().len(),
        2,
        "le même sujet"
    );
    assert_eq!(app.store.project.domains.len(), 2);
}

/// **Une faute se corrige toute seule** : un nom inconnu dit ce qui existe et comment créer ; les deux
/// arguments, ou aucun, disent lequel donner.
#[test]
fn les_fautes_disent_quoi_faire() {
    let mut app = app_avec_deux_textes();
    appeler(&mut app, "glucose_poser_domaine", r#"{"nouveau":"Eau"}"#);
    let faute = refus(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"Feu","ids":["a"]}"#,
    );
    assert!(
        faute.contains("Eau") && faute.contains("« nouveau »"),
        "{faute}"
    );
    let deux = refus(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"a","nouveau":"b"}"#,
    );
    assert!(deux.contains("pas les deux"), "{deux}");
    let aucun = refus(&mut app, "glucose_poser_domaine", r#"{"ids":["a"]}"#);
    assert!(aucun.contains("il manque"), "{aucun}");
    let inconnu = refus(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"Eau","ids":["z"]}"#,
    );
    assert!(inconnu.contains("« z »"), "{inconnu}");
}

/// **AGENT-6** : un poids nul retire ; sur un domaine que personne ne porte, il ne fait rien entrer.
#[test]
fn un_poids_nul_retire_et_ne_cree_rien() {
    let mut app = app_avec_deux_textes();
    let board = app.store.project.active_board_id.clone();
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"Eau","ids":["a"]}"#,
    );
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"eau","ids":["a"],"poids":0}"#,
    );
    assert!(app.store.node_domains(&board, "a").unwrap().is_empty());
    let r = appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"feu","ids":["a"],"poids":0}"#,
    );
    assert_eq!(r.get("cree"), Some(&Json::Bool(false)));
    assert_eq!(app.store.project.domains.len(), 1);
}

/// **Un nœud et ses domaines, en un seul geste** : `domaines` sur chaque outil qui pose, ses trois
/// formes ; et un nom refusé ne laisse pas de nœud derrière lui.
#[test]
fn poser_un_noeud_avec_ses_domaines_est_un_seul_geste() {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    appeler(&mut app, "glucose_poser_domaine", r#"{"nouveau":"Eau"}"#);
    let profondeur = app.store.undo_depth();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"t","texte":"H2O.","domaines":["eau",{"domaine":"EAU","poids":0.4},{"nouveau":"Chimie"}]}"#,
    );
    assert_eq!(app.store.undo_depth(), profondeur + 1, "un seul geste");
    let portes = app.store.node_domains(&board, "t").unwrap();
    assert_eq!(portes.len(), 2, "{portes:?}");
    assert_eq!(portes[0].weight, 0.4, "le dernier poids écrase le premier");
    app.store.undo();
    assert!(app.store.project.annotation(&board, "t").is_none());
    assert_eq!(app.store.project.domains.len(), 1, "« Chimie » est défait");

    let faute = refus(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"u","texte":"x","domaines":[{"nouveau":"eau"}]}"#,
    );
    assert!(faute.contains("existe déjà"), "{faute}");
    assert!(
        app.store.project.annotation(&board, "u").is_none(),
        "pas de nœud"
    );

    appeler(
        &mut app,
        "glucose_poser_membrane",
        r#"{"id":"m","titre":"Bulle","domaines":["eau"]}"#,
    );
    assert_eq!(app.store.node_domains(&board, "m").unwrap().len(), 1);
    appeler(
        &mut app,
        "glucose_poser_page",
        r#"{"id":"p","adresse":"https://exemple.org/","domaines":["eau"]}"#,
    );
    assert_eq!(
        app.store.node_domains(&board, "p-texte").unwrap().len(),
        1,
        "sa légende"
    );
    let faute = refus(
        &mut app,
        "glucose_poser_texte",
        r#"{"texte":"x","domaines":"eau"}"#,
    );
    assert!(faute.contains("liste"), "{faute}");
}

/// **Reposer ne jette rien, par l'outil même** : l'humain a mis un domaine, l'agent récrit le nœud.
#[test]
fn reposer_un_noeud_garde_les_domaines_de_l_humain() {
    let mut app = app_avec_deux_textes();
    let board = app.store.project.active_board_id.clone();
    let id = app
        .store
        .creer_un_domaine("Eau", ("#000000".into(), "E".into()), 1)
        .unwrap();
    app.store
        .try_assign_domain_to_node(&board, "a", &id, 1.0)
        .unwrap();
    appeler(
        &mut app,
        "glucose_poser_texte",
        r#"{"id":"a","texte":"Récrit."}"#,
    );
    assert_eq!(app.store.node_domains(&board, "a").unwrap().len(), 1);
}

/// **Traverser les fichiers** : la table des matières dit chaque domaine, dans ce document et dans
/// les autres, sans qu'aucun ne s'ouvre ; les nœuds d'un domaine viennent de tous, du plus lourd au
/// plus léger.
#[test]
fn glucose_domaines_traverse_les_fichiers() {
    let mut app = app_avec_deux_textes();
    autre_fichier(
        &mut app,
        "/ailleurs.glucose",
        ("Jeu vidéo", 7),
        ("jeu", "Le bruit fait la vague. Et le reste."),
    );
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"Eau","ids":["a"]}"#,
    );
    let identite = app.store.identite.clone();

    let table = appeler(&mut app, "glucose_domaines", "{}");
    let domaines = liste(&table, "domaines");
    let noms: Vec<_> = domaines.iter().map(|d| texte(d, "nom")).collect();
    assert_eq!(noms, ["Eau", "Jeu vidéo"]);
    let jeu = &domaines[1];
    assert_eq!(nombre(jeu, "noeuds"), 1.0);
    assert_eq!(
        texte(&liste(jeu, "fichiers")[0], "fichier"),
        "/ailleurs.glucose"
    );
    assert_eq!(
        texte(&table, "fichier_ouvert"),
        identite,
        "sans fichier encore : son identité"
    );
    assert_eq!(table.get("a_jour"), Some(&Json::Bool(true)));

    let lu = appeler(&mut app, "glucose_domaines", r#"{"domaine":"jeu vidéo"}"#);
    let noeud = &liste(&lu, "noeuds")[0];
    assert_eq!(texte(noeud, "fichier"), "/ailleurs.glucose");
    assert_eq!(texte(noeud, "id"), "jeu");
    assert_eq!(nombre(noeud, "poids"), 0.9);
    assert_eq!(texte(noeud, "debut"), "Le bruit fait la vague.");
    assert!(noeud.get("texte").is_none(), "le sommaire, pas le texte");
    assert_eq!(app.store.identite, identite, "rien ne s'ouvre à l'écran");

    let entier = appeler(
        &mut app,
        "glucose_domaines",
        r#"{"domaine":"jeu vidéo","entier":true}"#,
    );
    assert_eq!(
        texte(&liste(&entier, "noeuds")[0], "texte"),
        "Le bruit fait la vague. Et le reste."
    );
    let cherche = appeler(
        &mut app,
        "glucose_domaines",
        r#"{"domaine":"jeu vidéo","cherche":"RESTE"}"#,
    );
    assert_eq!(
        texte(&liste(&cherche, "noeuds")[0], "passage"),
        "Et le reste."
    );
    let rien = appeler(
        &mut app,
        "glucose_domaines",
        r#"{"domaine":"jeu vidéo","cherche":"absent"}"#,
    );
    assert!(liste(&rien, "noeuds").is_empty());
    let faute = refus(&mut app, "glucose_domaines", r#"{"domaine":"feu"}"#);
    assert!(faute.contains("Jeu vidéo"), "{faute}");
}

/// **Un domaine connu d'un autre fichier se pose ici** sous son identité : le nœud rejoint le même
/// domaine, et sa définition entre dans ce fichier, pour qu'il reste entier.
#[test]
fn poser_un_domaine_d_un_autre_fichier_rejoint_le_meme() {
    let mut app = app_avec_deux_textes();
    let venu = autre_fichier(
        &mut app,
        "/ailleurs.glucose",
        ("Eau", 7),
        ("x", "Une vague."),
    );
    let question = refus(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"eau","ids":["a"]}"#,
    );
    assert!(question.contains("Une vague."), "il y existe : {question}");

    let r = appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"eau","ids":["a"]}"#,
    );
    assert_eq!(r.get("cree"), Some(&Json::Bool(false)));
    assert_eq!(
        app.store.project.domains,
        std::slice::from_ref(&venu),
        "entière, sous son identité"
    );

    let lu = appeler(&mut app, "glucose_domaines", r#"{"domaine":"eau"}"#);
    let documents: Vec<_> = liste(&lu, "noeuds")
        .iter()
        .map(|n| texte(n, "fichier"))
        .collect();
    assert_eq!(
        documents.len(),
        2,
        "un seul domaine, deux fichiers : {documents:?}"
    );
    let tete = lu.get("domaine").expect("le domaine");
    assert_eq!(liste(tete, "fichiers").len(), 2);
}

/// **Deux créations séparées se rencontrent** : rien ne les réunit en silence. Le nom est marqué, la
/// pose refusée avec la question ; « renommer » le précise, et les deux redeviennent libres.
#[test]
fn deux_creations_separees_demandent_de_preciser() {
    let mut app = app_avec_deux_textes();
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"Python","ids":["a"]}"#,
    );
    autre_fichier(
        &mut app,
        "/serpent.glucose",
        ("python", 7),
        ("s", "Un long serpent."),
    );

    let lu = appeler(&mut app, "glucose_lire", "{}");
    assert_eq!(
        liste(&lu, "domaines")[0].get("a_preciser"),
        Some(&Json::Bool(true))
    );
    let table = appeler(&mut app, "glucose_domaines", "{}");
    assert!(liste(&table, "domaines")
        .iter()
        .all(|d| d.get("a_preciser").is_some()));
    let tous = appeler(&mut app, "glucose_domaines", r#"{"domaine":"python"}"#);
    assert!(texte(&tous, "question").contains("créés séparément"));
    assert_eq!(liste(&tous, "noeuds").len(), 2, "les deux, pour juger");

    let question = refus(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"python","ids":["b"]}"#,
    );
    assert!(
        question.contains("Un long serpent.") && question.contains("meme_sujet"),
        "{question}"
    );
    assert!(question.contains("renommer"), "{question}");

    let pris = refus(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"python","renommer":"PYTHON"}"#,
    );
    assert!(
        pris.contains("existe déjà"),
        "l'autre le porte encore : {pris}"
    );
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"python","renommer":"Python code"}"#,
    );
    let table = appeler(&mut app, "glucose_domaines", "{}");
    let noms: Vec<_> = liste(&table, "domaines")
        .iter()
        .map(|d| texte(d, "nom"))
        .collect();
    assert_eq!(noms, ["python", "Python code"]);
    assert!(liste(&table, "domaines")
        .iter()
        .all(|d| d.get("a_preciser").is_none()));
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"python code","ids":["b"]}"#,
    );
}

/// **« C'est le même sujet »** : le domaine de ce document rejoint l'autre, tout ce qui le porte suit,
/// en un geste ; la question disparaît. Sans homonyme, ou sans domaine ici, l'appel le dit.
#[test]
fn meme_sujet_reunit_les_deux() {
    let mut app = app_avec_deux_textes();
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"Eau","ids":["a","b"]}"#,
    );
    let seul = refus(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"eau","meme_sujet":true}"#,
    );
    assert!(seul.contains("rien à réunir"), "{seul}");
    let venu = autre_fichier(
        &mut app,
        "/ailleurs.glucose",
        ("eau", 7),
        ("x", "Une vague."),
    );
    let profondeur = app.store.undo_depth();

    let r = appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"eau","meme_sujet":true}"#,
    );
    assert_eq!(nombre(&r, "noeuds"), 2.0);
    assert_eq!(app.store.undo_depth(), profondeur + 1);
    assert_eq!(app.store.project.domains[0].id, venu.id, "son identité");
    let board = app.store.project.active_board_id.clone();
    assert_eq!(
        app.store.node_domains(&board, "a").unwrap()[0].domain_id,
        venu.id
    );

    let table = appeler(&mut app, "glucose_domaines", "{}");
    let domaines = liste(&table, "domaines");
    assert_eq!(domaines.len(), 1, "un seul : {}", table.ecrire());
    assert_eq!(nombre(&domaines[0], "noeuds"), 3.0);
    assert!(domaines[0].get("a_preciser").is_none());

    let mut autre = app_avec_deux_textes();
    let sans = refus(
        &mut autre,
        "glucose_poser_domaine",
        r#"{"domaine":"eau","meme_sujet":true}"#,
    );
    assert!(sans.contains("ne porte pas"), "{sans}");
}

/// **Renommer un domaine connu d'ailleurs seulement** : il entre ici sous son nom neuf, son identité
/// ne bouge pas, et la définition la plus récente l'emporte dans la vue.
#[test]
fn renommer_un_domaine_d_ailleurs_garde_son_identite() {
    let mut app = app_avec_deux_textes();
    let venu = autre_fichier(
        &mut app,
        "/ailleurs.glucose",
        ("Eau", 7),
        ("x", "Une vague."),
    );
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"eau","renommer":"Eau pure"}"#,
    );
    assert_eq!(app.store.project.domains[0].id, venu.id);
    assert_eq!(app.store.project.domains[0].name, "Eau pure");
    let table = appeler(&mut app, "glucose_domaines", "{}");
    let domaines = liste(&table, "domaines");
    assert_eq!(domaines.len(), 1);
    assert_eq!(texte(&domaines[0], "nom"), "Eau pure");
    let absent = refus(
        &mut app,
        "glucose_poser_domaine",
        r#"{"domaine":"feu","renommer":"x"}"#,
    );
    assert!(absent.contains("feu"), "{absent}");
}

/// **Le document ouvert ne se compte pas deux fois**, quel que soit le nom qu'on lui connaît : il a un
/// fichier (ici son brouillon), que le fil lit aussi, et une identité qu'enregistrer ne change pas.
/// Vu sur un essai réel : quatre nœuds pour deux. La preuve à l'envers : sous son identité, il comptait
/// double.
#[test]
fn le_document_ouvert_ne_se_compte_pas_deux_fois() {
    let dossier = crate::epreuve::Dossier::nouveau("pas-deux-fois");
    let chemin = dossier.join("ici.glucose");
    let mut app = app_avec_deux_textes();
    appeler(
        &mut app,
        "glucose_poser_domaine",
        r#"{"nouveau":"Eau","ids":["a","b"]}"#,
    );
    app.save_to(chemin.clone());
    let fichier = chemin.display().to_string();
    assert_ne!(
        app.store.identite, fichier,
        "enregistrer ne change pas l'identité"
    );
    assert_eq!(app.fichier_de_ce_document(), fichier);

    // Ce que le fil a lu de ce même fichier.
    app.disque
        .recenseur
        .etat
        .poser(recenser(&fichier, &app.store.project));
    let table = appeler(&mut app, "glucose_domaines", "{}");
    let eau = &liste(&table, "domaines")[0];
    assert_eq!(nombre(eau, "noeuds"), 2.0, "{}", table.ecrire());
    assert_eq!(liste(eau, "fichiers").len(), 1);
    let lu = appeler(&mut app, "glucose_lire", "{}");
    assert_eq!(texte(&lu, "fichier"), fichier);

    let double = app
        .disque
        .recenseur
        .etat
        .vue(Some(&recenser(&app.store.identite, &app.store.project)));
    assert_eq!(double[0].noeuds(), 4, "à l'envers");
    drop(app);
}

/// **« a_jour » finit par dire vrai** quand le fil a tout lu : un agent qui rappelle tant qu'il lit
/// faux ne tourne pas en rond. Joué avec le vrai fil, sur un vrai fichier ; une boucle sans fin fait
/// tomber l'épreuve, elle ne la fige pas.
#[test]
fn a_jour_finit_par_dire_vrai() {
    let glucose = crate::epreuve::Dossier::nouveau("a-jour-glucose");
    let ici = crate::epreuve::Dossier::nouveau("a-jour-fichiers");
    let chemin = ici.join("ailleurs.glucose");
    let mut s = Store::new("Autre");
    let board = s.project.active_board_id.clone();
    s.add_annotation(&board, Annotation::text("x", 0.0, 0.0, "Une vague."));
    let id = s
        .creer_un_domaine("Eau", ("#123456".into(), "E".into()), 7)
        .expect("créé");
    s.try_assign_domain_to_node(&board, "x", &id, 1.0)
        .expect("porté");
    let octets =
        glucose_core::persist::encode(&s.project, &glucose_core::types::AssetStore::default(), 0);
    std::fs::write(&chemin, octets).expect("écrit");
    std::fs::write(
        glucose.join(crate::recensement::FICHIER),
        format!("{}\n", chemin.display()),
    )
    .expect("connu");

    let mut app = GlucoseApp::new();
    app.disque
        .recenseur
        .demarrer(&glucose, Vec::new(), std::sync::Arc::new(|| {}));
    let debut = std::time::Instant::now();
    let table = loop {
        let table = appeler(&mut app, "glucose_domaines", "{}");
        if table.get("a_jour") == Some(&Json::Bool(true)) {
            break table;
        }
        assert!(
            debut.elapsed() < std::time::Duration::from_secs(20),
            "« a_jour » ne dit jamais vrai"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    let domaines = liste(&table, "domaines");
    assert_eq!(texte(&domaines[0], "nom"), "Eau", "{}", table.ecrire());
}
