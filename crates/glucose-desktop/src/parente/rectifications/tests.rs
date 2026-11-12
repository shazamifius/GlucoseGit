//! **RECTIF-1 — le journal des rectifications** (fiche 110) : ce que l'humain fait autrement que
//! l'IA s'écrit dans l'histoire, signé de l'IA de Glucose, attaché au bon geste, nommant le bon
//! nœud — et rien d'autre ne s'écrit. L'avis de l'IA est posé à la main
//! ([`SensDesTextes::avec_l_avis`]) : ces épreuves regardent ce qu'on fait de lui, pas l'IA.

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use crate::epreuve::Dossier;
use crate::parente::{Avis, AvisDUnNoeud};
use crate::renderer::sens::SensDesTextes;
use glucose_core::persist::histoire::{Auteur, Note, Ouvert};
use glucose_core::store::journal::Main;
use glucose_core::types::Annotation;

/// Un avis : sa famille (ou aucune), sa sûreté, son plus proche parent.
fn avis(famille: Option<usize>, surete: f32, parent: Option<&str>) -> AvisDUnNoeud {
    AvisDUnNoeud {
        famille,
        teinte: famille.map(|_| 0.0),
        surete,
        parent: parent.map(String::from),
    }
}

/// Une application qui écrit son brouillon dans un dossier d'épreuve, son tableau vide.
fn application(nom: &str) -> (GlucoseApp, Dossier, String) {
    let dossier = Dossier::nouveau(nom);
    let mut app = GlucoseApp::new();
    app.habiter(&dossier);
    let board = app.store.project.active_board_id.clone();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    (app, dossier, board)
}

/// Les écrits de l'IA de Glucose dans l'histoire relue du fichier.
fn rectifications(app: &mut GlucoseApp) -> (Vec<Note>, Ouvert) {
    let o = app.lire_l_histoire().expect("l'histoire se relit");
    let ia = Auteur::de_glucose().id;
    let notes = o.notes.iter().filter(|n| n.auteur == ia).cloned().collect();
    (notes, o)
}

/// Donne cette couleur à un texte, par un geste.
fn colorer_de(app: &mut GlucoseApp, board: &str, id: &str, couleur: &str) {
    app.store.update_annotation(board, id, |t| {
        if let Annotation::Text { color, .. } = t {
            *color = Some(couleur.into());
        }
    });
}

/// Donne la couleur de l'épreuve à un texte, par un geste.
fn colorer(app: &mut GlucoseApp, board: &str, id: &str) {
    colorer_de(app, board, id, "#e06c75");
}

/// **Colorer un texte que l'IA rangeait dans une famille s'écrit** : au rang de ce geste, sur ce
/// texte, avec sa sûreté, le texte qui lui ressemble le plus, les mots qui les rapprochent et la
/// couleur donnée ; la Time Machine en montre l'auteur. **Ne s'écrit pas** : un texte sans famille
/// (l'IA ne proposait rien), un texte déjà coloré (l'humain l'avait déjà dit), le geste d'un agent.
#[test]
fn rectif_1_colorer_un_texte_de_famille_s_ecrit() {
    let (mut app, _d, board) = application("rectif-1-texte");
    for (id, texte) in [
        ("a", "La pluie tombe sur les toits de la ville."),
        ("b", "La pluie remplit les nappes souterraines."),
        ("seul", "Une cathédrale gothique."),
        ("deja", "Les nuages portent la pluie."),
    ] {
        app.store
            .add_annotation(&board, Annotation::text(id, 0.0, 0.0, texte));
    }
    // Une autre couleur que celle qu'il recevra : un geste qui ne change rien ne s'écrit pas, et
    // l'épreuve ne verrait plus le cas (le sabotage l'a montré, fiche 110).
    colorer_de(&mut app, &board, "deja", "#112233");
    app.consigner();
    let a: Avis = [
        ("a", avis(Some(0), 0.9, Some("b"))),
        ("b", avis(Some(0), 0.9, Some("a"))),
        ("seul", avis(None, 0.0, Some("a"))),
        ("deja", avis(Some(0), 0.8, Some("a"))),
    ]
    .into_iter()
    .map(|(id, a)| (id.to_string(), a))
    .collect();
    app.renderer.sens_des_textes = SensDesTextes::avec_l_avis(&board, a);

    for id in ["seul", "deja", "a"] {
        colorer(&mut app, &board, id);
    }
    app.store.journal.main = Main::Agent("proteo".into());
    colorer(&mut app, &board, "b");
    app.store.journal.main = Main::Humain;
    app.consigner();

    let (notes, o) = rectifications(&mut app);
    assert_eq!(notes.len(), 1, "une seule rectification : {notes:?}");
    let n = &notes[0];
    assert_eq!(n.sujets, ["a"]);
    assert_eq!(
        n.geste as usize,
        o.gestes.len() - 2,
        "le geste qui colore « a »"
    );
    for attendu in [
        "sûre à 90 %",
        "« La pluie remplit les nappes souterraines. »",
        "« pluie »",
        "#e06c75",
    ] {
        assert!(
            n.texte.contains(attendu),
            "« {attendu} » dans : {}",
            n.texte
        );
    }
    let vus: Vec<&str> = app
        .dock_manager
        .temps
        .ecrits
        .iter()
        .map(|e| e.auteur.as_str())
        .collect();
    assert_eq!(vus, ["l'IA de Glucose"], "la Time Machine la nomme");
    // Ce que l'humain voit, une IA le lit : `glucose_histoire` la nomme, avec son sujet.
    let lue = app
        .executer_pour_un_agent("glucose_histoire", &Json::objet([]))
        .expect("l'histoire se lit");
    let gestes = lue
        .get("gestes")
        .and_then(Json::as_liste)
        .expect("des gestes");
    let ecrit = gestes
        .iter()
        .flat_map(|g| g.get("ecrits").and_then(Json::as_liste).unwrap_or_default())
        .next()
        .expect("un écrit");
    assert_eq!(
        ecrit.get("auteur").and_then(Json::as_str),
        Some("l'IA de Glucose")
    );
    let sujets = ecrit
        .get("sujets")
        .and_then(Json::as_liste)
        .unwrap_or_default();
    assert_eq!(sujets.first().and_then(Json::as_str), Some("a"));
}

/// **Retoucher ou retirer une flèche d'agent que l'IA jugeait s'écrit**, au rang du geste, avec sa
/// sûreté — la flèche jugée sûre comme celle d'habitude : aucun seuil. **Ne s'écrit pas** : la
/// flèche de l'humain, une flèche d'agent que l'IA ne jugeait pas, et la retouche d'un agent.
#[test]
fn rectif_1_reprendre_une_fleche_jugee_s_ecrit() {
    let (mut app, _d, board) = application("rectif-1-fleche");
    app.store
        .add_annotation(&board, Annotation::arrow("h", 0.0, 0.0, 9.0, 9.0));
    app.store.journal.main = Main::Agent("squelette".into());
    for id in ["habitude", "sens", "non-jugee", "par-l-agent"] {
        app.store
            .add_annotation(&board, Annotation::arrow(id, 0.0, 0.0, 1.0, 1.0));
    }
    app.store.journal.main = Main::Humain;
    app.consigner();
    let a: Avis = [
        ("h", 0.1),
        ("habitude", 0.2),
        ("sens", 0.85),
        ("par-l-agent", 0.3),
    ]
    .into_iter()
    .map(|(id, s)| (id.to_string(), avis(None, s, None)))
    .collect();
    app.renderer.sens_des_textes = SensDesTextes::avec_l_avis(&board, a);

    let retoucher = |app: &mut GlucoseApp, id: &str| {
        app.store.update_annotation(&board, id, |f| {
            if let Annotation::Arrow { text, .. } = f {
                *text = Some("retouchée".into());
            }
        });
    };
    retoucher(&mut app, "h");
    retoucher(&mut app, "habitude");
    app.store.remove_annotations(&board, &["sens"]);
    retoucher(&mut app, "non-jugee");
    app.store.journal.main = Main::Agent("squelette".into());
    retoucher(&mut app, "par-l-agent");
    app.store.journal.main = Main::Humain;
    app.consigner();

    let (notes, o) = rectifications(&mut app);
    let sujets: Vec<&str> = notes.iter().map(|n| n.sujets[0].as_str()).collect();
    assert_eq!(sujets, ["habitude", "sens"], "{notes:?}");
    let n = o.gestes.len();
    assert_eq!(
        notes[0].geste as usize,
        n - 4,
        "le geste qui retouche « habitude »"
    );
    assert_eq!(
        notes[1].geste as usize,
        n - 3,
        "le geste qui retire « sens »"
    );
    assert!(notes[0].texte.contains("sûre à 20 %") && notes[0].texte.contains("retouchée"));
    assert!(notes[1].texte.contains("sûre à 85 %") && notes[1].texte.contains("retirée"));
}

/// **Suivre une flèche d'agent que l'IA jugeait s'écrit**, attaché au dernier geste du document —
/// et ne la sort pas du jugement : ce n'est pas un geste, il ne se défait pas (D1). Suivre la
/// flèche de l'humain n'écrit rien.
#[test]
fn rectif_1_suivre_une_fleche_jugee_s_ecrit() {
    let (mut app, _d, board) = application("rectif-1-suivre");
    for (id, x) in [("a", 0.0), ("b", 5000.0)] {
        app.store
            .add_annotation(&board, Annotation::text(id, x, 0.0, "un bloc"));
    }
    let relier = |id: &str| {
        let mut f = Annotation::arrow(id, 0.0, 0.0, 5000.0, 0.0);
        if let Annotation::Arrow {
            source_id,
            target_id,
            ..
        } = &mut f
        {
            *source_id = Some("a".into());
            *target_id = Some("b".into());
        }
        f
    };
    app.store.add_annotation(&board, relier("h"));
    app.store.journal.main = Main::Agent("squelette".into());
    app.store.add_annotation(&board, relier("f"));
    app.store.journal.main = Main::Humain;
    app.consigner();
    let a: Avis = [("f", 0.4), ("h", 0.4)]
        .into_iter()
        .map(|(id, s)| (id.to_string(), avis(None, s, None)))
        .collect();
    app.renderer.sens_des_textes = SensDesTextes::avec_l_avis(&board, a);

    for id in ["h", "f"] {
        app.store.clear_selection();
        app.store.select_annotation(id.to_string(), false);
        assert!(app.suivre_le_lien_choisi(), "la flèche {id} se suit");
    }

    let (notes, o) = rectifications(&mut app);
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert_eq!(notes[0].sujets, ["f"]);
    assert_eq!(
        notes[0].geste as usize,
        o.gestes.len() - 1,
        "le dernier geste"
    );
    assert!(notes[0].texte.contains("a suivi") && notes[0].texte.contains("sûre à 40 %"));
    assert!(
        app.store.journal.des_agents.contains("f"),
        "suivre ne la sort pas du jugement"
    );
}
