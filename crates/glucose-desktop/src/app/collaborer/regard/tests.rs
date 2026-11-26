//! REGARD-1 : sa dernière sélection se retient, part aux autres, se redit à qui arrive ; celle
//! d'un autre se lit, datée de l'âge qu'elle dit.

use super::super::{lien, CanevasPartage, Entree};
use super::*;
use glucose_core::salle::code::Code;
use glucose_core::types::BoardImage;
use std::sync::{mpsc, Arc};

/// Une application où trois nœuds sont posés, `a`, `b`, `c`, et aucun choisi.
fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    let board = app.store.project.active_board_id.clone();
    for (i, id) in ["a", "b", "c"].into_iter().enumerate() {
        let x = 20.0 * i as f64;
        app.store
            .add_image(&board, BoardImage::new(id, x, 0.0, 10.0, 10.0));
    }
    // Poser un nœud le choisit.
    app.store.clear_selection();
    app
}

fn choisir(app: &mut GlucoseApp, ids: &[&str]) {
    app.store.clear_selection();
    for id in ids {
        app.store.select_image((*id).to_string(), true);
    }
}

/// Dans un canevas partagé, sous la marque de Bob ; rend ce que Glucose confie au fil.
fn dedans(app: &mut GlucoseApp) -> mpsc::Receiver<lien::Ordre> {
    let (fil, _salle, confies) = lien::a_l_ecoute();
    let qui = ("b0b".into(), "Bob".into(), 0);
    let objets = Arc::clone(&app.disque.objets);
    let code = Code::neuf([7; 32]);
    let cp = CanevasPartage::neuf(fil, code, qui, Entree::Dedans, objets);
    app.lancement.agents.partage = Some(cp);
    confies
}

/// La sélection que ce Glucose a confiée au fil, s'il en a confié une.
fn selection_partie(confies: &mpsc::Receiver<lien::Ordre>) -> Option<(Vec<String>, u64)> {
    confies.try_iter().find_map(|o| {
        let lien::Ordre::Curseur(clair) = o else {
            return None;
        };
        match Lettre::lire(&clair).expect("une lettre").corps {
            Corps::Selection { ids, age, .. } => Some((ids, age)),
            _ => None,
        }
    })
}

fn d_alice(corps: Corps) -> Lettre {
    Lettre {
        marque: "a1ce".into(),
        nom: "Aigle Émeraude".into(),
        couleur: 1,
        main: Main::Humain,
        corps,
    }
}

/// **Sa dernière sélection non vide est son regard** : vider la sélection ne l'efface pas ; la
/// même réécrite sans passer par le vide (un geste d'IA la remet telle qu'elle était) garde son
/// heure ; choisie de nouveau après le vide, elle en prend une neuve.
#[test]
fn regard_1_sa_derniere_selection_est_son_regard() {
    let mut app = app();
    app.suivre_le_regard();
    assert_eq!(app.lancement.agents.regards.ici, None, "rien de choisi");
    choisir(&mut app, &["b", "a"]);
    app.suivre_le_regard();
    let ici = app.lancement.agents.regards.ici.clone().expect("un regard");
    assert_eq!(
        (ici.ids, ici.en_tout),
        (vec!["b".to_string(), "a".into()], 2)
    );
    assert_eq!(ici.tableau, app.store.project.active_board_id);
    choisir(&mut app, &[]);
    app.suivre_le_regard();
    let reste = app.lancement.agents.regards.ici.clone().map(|r| r.ids);
    assert_eq!(
        reste,
        Some(vec!["b".into(), "a".into()]),
        "vider ne l'efface pas"
    );

    choisir(&mut app, &["c"]);
    app.suivre_le_regard();
    let regarde = |app: &GlucoseApp| app.lancement.agents.regards.ici.clone().expect("c");
    app.lancement
        .agents
        .regards
        .ici
        .as_mut()
        .expect("c")
        .quand_ms = 0;
    choisir(&mut app, &["c"]);
    app.suivre_le_regard();
    assert_eq!(
        regarde(&app).quand_ms,
        0,
        "la même, réécrite : son heure reste"
    );
    choisir(&mut app, &[]);
    app.suivre_le_regard();
    choisir(&mut app, &["c"]);
    app.suivre_le_regard();
    assert!(
        regarde(&app).quand_ms > 0,
        "choisie de nouveau : une heure neuve"
    );
}

/// **Mon regard part, et se redit à qui arrive** : quand il change, au bonjour d'un autre, au
/// mien ; jamais à son simple curseur. Celui d'un autre se lit, daté de l'âge qu'il dit, et une
/// IA les lit tous, le plus récent d'abord.
#[test]
fn regard_1_mon_regard_part_et_celui_d_un_autre_se_lit() {
    let mut app = app();
    let confies = dedans(&mut app);
    let entree = |app: &mut GlucoseApp, e| {
        app.lancement
            .agents
            .partage
            .as_mut()
            .expect("partagé")
            .entree = e;
    };
    entree(&mut app, Entree::AttendLaBase);
    choisir(&mut app, &["a"]);
    app.suivre_le_regard();
    assert_eq!(
        selection_partie(&confies),
        None,
        "rien ne part avant d'être dedans"
    );
    entree(&mut app, Entree::Dedans);
    app.dire_mon_regard();
    assert_eq!(selection_partie(&confies), Some((vec!["a".into()], 0)));
    choisir(&mut app, &["b"]);
    app.suivre_le_regard();
    assert_eq!(
        selection_partie(&confies),
        Some((vec!["b".into()], 0)),
        "il change"
    );
    app.suivre_le_regard();
    assert_eq!(
        selection_partie(&confies),
        None,
        "rien de neuf, rien ne part"
    );

    let curseur = Corps::Curseur {
        tableau: "main".into(),
        x: 1.0,
        y: 2.0,
    };
    app.un_curseur(d_alice(curseur));
    assert_eq!(
        selection_partie(&confies),
        None,
        "un curseur n'appelle rien"
    );
    let bonjour = Corps::Demande {
        cles: Vec::new(),
        bonjour: true,
    };
    app.une_lettre_qui_passe(d_alice(bonjour));
    assert!(
        selection_partie(&confies).is_some(),
        "redit au bonjour d'Alice"
    );
    app.salle_a_jour(0);
    assert!(selection_partie(&confies).is_some(), "redit à mon bonjour");

    let sienne = Corps::Selection {
        tableau: "main".into(),
        ids: vec!["b".into(), "c".into()],
        en_tout: 40,
        age: 120,
    };
    app.un_curseur(d_alice(sienne));
    let regards = app.regards_en_json();
    let texte = |j: &Json, cle| j.get(cle).and_then(Json::as_str).map(String::from);
    let appareils: Vec<_> = regards.iter().map(|r| texte(r, "appareil")).collect();
    assert_eq!(
        appareils,
        [Some("ici".into()), Some("Aigle Émeraude".into())],
        "le plus récent d'abord"
    );
    let alice = &regards[1];
    assert_eq!(alice.get("en_tout").and_then(Json::as_f64), Some(40.0));
    assert_eq!(
        regards[0].get("en_tout"),
        None,
        "une sélection entière ne le redit pas"
    );
    let quand = app.lancement.agents.partage.as_ref().expect("dedans").pairs["a1ce"]
        .regard
        .as_ref()
        .expect("son regard")
        .quand_ms;
    let il_y_a = maintenant_ms().saturating_sub(quand);
    assert!((119_000..=125_000).contains(&il_y_a), "il y a {il_y_a} ms");
}
