//! **Collaborer, une seule fenêtre** (fiche 192) : tout ce qui touche aux humains s'y lit et s'y
//! fait, chaque bouton par son libellé, comme sous sa main ; les IA ont un étage, « Retour »
//! remonte.

use super::super::{lien, CanevasPartage, Entree, Pair};
use crate::app::GlucoseApp;
use crate::ui::question::Suite;
use glucose_core::salle::code::Code;
use std::sync::Arc;
use std::time::Instant;

/// La question de Collaborer ouverte : son titre, son texte, ses libellés.
fn ouverte(app: &GlucoseApp) -> (String, String, Vec<String>) {
    let Some((q, Suite::Collaborer)) = &app.ui.question else {
        panic!("la question de Collaborer, pas {:?}", app.ui.question);
    };
    let libelles = q.choix.iter().map(|(l, _)| l.clone()).collect();
    (q.titre.clone(), q.texte.clone(), libelles)
}

/// Répond par le bouton qui porte ce libellé.
fn cliquer(app: &mut GlucoseApp, libelle: &str) {
    let reponse = app.ui.question.as_ref().and_then(|(q, _)| {
        let bouton = q.choix.iter().find(|(l, _)| l == libelle);
        bouton.map(|(_, r)| *r)
    });
    let reponse = reponse.unwrap_or_else(|| panic!("« {libelle} » parmi les choix"));
    app.repondre_a_la_question(Some(reponse));
}

/// Ce canevas partagé sous ce code, avec ces gens dedans.
fn partager(app: &mut GlucoseApp, code: Code, gens: &[&str]) {
    let boite = crate::agents::Boite::nouvelle(Arc::new(|| {}));
    let lien = lien::lancer("http://127.0.0.1:9", code.clone(), 0, boite);
    let qui = ("b0b".into(), "Bob".into(), 0);
    let objets = Arc::clone(&app.disque.objets);
    let mut cp = CanevasPartage::neuf(lien, code, qui, Entree::Dedans, objets);
    for (i, nom) in gens.iter().enumerate() {
        let pair = Pair {
            nom: (*nom).into(),
            couleur: 0,
            curseur: None,
            regard: None,
            vu: Instant::now(),
        };
        cp.pairs.insert(format!("p{i}"), pair);
    }
    app.lancement.agents.partage = Some(cp);
}

/// **Une seule fenêtre, seul** : partager, rejoindre, son nom, l'étage des IA, les programmes —
/// et ce qu'elle dit de chacun ; l'étage des IA dit son chemin, et « Retour » remonte.
#[test]
fn une_seule_fenetre_dit_tout_et_l_etage_des_ia_remonte() {
    let mut app = GlucoseApp::new();
    app.montrer_collaborer();
    let (titre, texte, choix) = ouverte(&app);
    assert_eq!(titre, "Collaborer");
    let attendus = [
        "Partager ce canevas",
        "Rejoindre…",
        "Ton nom et ta couleur…",
        "IA…",
        "Les programmes de cette machine…",
        "Fermer",
    ];
    assert_eq!(choix, attendus);
    for dit in [
        "Ce canevas n'est pas partagé.",
        "la porte aux IA publiques est fermée",
        "aucun n'a ton accord",
    ] {
        assert!(texte.contains(dit), "« {dit} » dans {texte}");
    }

    cliquer(&mut app, "IA…");
    let (titre, texte, _) = ouverte(&app);
    assert_eq!(titre, "Collaborer › IA");
    assert!(
        texte.starts_with("La porte aux IA publiques est fermée"),
        "{texte}"
    );
    assert!(texte.contains("Aucune IA n'a encore ta porte"), "{texte}");
    cliquer(&mut app, "Retour");
    assert_eq!(ouverte(&app).0, "Collaborer");
}

/// **L'étage des IA mène à ses trois fenêtres** : la porte MCP du nuage, l'API publique, et
/// les IA qui ont la porte, à qui la retirer.
#[test]
fn l_etage_des_ia_mene_a_la_porte_a_l_api_et_a_ceux_qui_l_ont() {
    let mut app = GlucoseApp::new();
    app.montrer_collaborer();
    cliquer(&mut app, "IA…");
    cliquer(&mut app, "La porte aux IA publiques…");
    assert!(
        matches!(&app.ui.question, Some((q, Suite::PortePublique)) if q.texte.starts_with("Fermée."))
    );
    app.montrer_collaborer();
    cliquer(&mut app, "IA…");
    cliquer(&mut app, "L'API publique…");
    assert!(
        matches!(&app.ui.question, Some((q, Suite::ApiPublique)) if q.titre == "L'API publique")
    );
    app.montrer_collaborer();
    cliquer(&mut app, "IA…");
    cliquer(&mut app, "Les IA du nuage…");
    assert!(matches!(app.ui.question, Some((_, Suite::RetirerUneIa))));
}

/// **Partagé** : où en est la salle et qui y est ; un code qui écrit se copie pour travailler
/// et pour regarder, un code qui regarde ne donne que lui ; l'accord des programmes se dit, et
/// se retire d'ici.
#[test]
fn partage_la_fenetre_dit_qui_est_la_et_donne_les_codes_permis() {
    let mut app = GlucoseApp::new();
    partager(&mut app, Code::neuf([7; 32]), &["Alice"]);
    app.montrer_collaborer();
    let (_, texte, choix) = ouverte(&app);
    let attendus = [
        "Copier le code pour y travailler",
        "Copier le code pour regarder",
        "Changer les codes…",
        "Quitter le partage",
        "Ton nom et ta couleur…",
        "IA…",
        "Les programmes de cette machine…",
        "Fermer",
    ];
    assert_eq!(choix, attendus);
    assert!(texte.contains("Avec : Alice."), "{texte}");

    // L'accord durable des programmes de cette machine se dit — sans fichier, rien ne s'écrit.
    let identite = app.store.identite.clone();
    app.lancement.agents.toujours.retenir(&identite);
    app.montrer_collaborer();
    let (_, texte, _) = ouverte(&app);
    let dit = "ont ton accord : tout programme de cette machine, toujours";
    assert!(texte.contains(dit), "{texte}");
    cliquer(&mut app, "Les programmes de cette machine…");
    assert!(matches!(
        app.ui.question,
        Some((_, Suite::OublierLesAgents))
    ));

    let lire = Code::lire_le_texte(&Code::neuf([7; 32]).texte_lire()).expect("un code lisible");
    partager(&mut app, lire, &[]);
    app.montrer_collaborer_avec(None);
    let (_, texte, choix) = ouverte(&app);
    let lecteur: Vec<&str> = (attendus.iter().copied())
        .filter(|l| !l.starts_with("Copier le code pour y") && !l.starts_with("Changer"))
        .collect();
    assert_eq!(choix, lecteur, "un lecteur ne donne ni ne change de code");
    assert!(texte.contains("Ton code ne fait que regarder"), "{texte}");
    assert!(texte.contains("Personne d'autre"), "{texte}");
}

/// **Changer les codes** (fiche 194) : une salle neuve sous des codes neufs ; sans réseau, rien ne
/// change et il reste dans la sienne.
#[test]
fn changer_les_codes_ouvre_une_salle_neuve() {
    let mut app = GlucoseApp::new();
    let ancien = Code::neuf([7; 32]);
    partager(&mut app, ancien.clone(), &[]);
    let code = |app: &GlucoseApp| {
        app.lancement
            .agents
            .partage
            .as_ref()
            .map(|cp| cp.code.texte_lire())
    };

    app.montrer_collaborer();
    cliquer(&mut app, "Changer les codes…");
    assert_eq!(ouverte(&app).0, "Collaborer › Changer les codes");
    cliquer(&mut app, "Changer les codes");
    assert_eq!(
        code(&app),
        Some(ancien.texte_lire()),
        "sans réseau, rien ne change"
    );
    assert!(ouverte(&app).1.contains("pas de réseau"), "et il le sait");

    app.lancement.agents.boite = Some(crate::agents::Boite::nouvelle(Arc::new(|| {})));
    app.lancement.agents.relais = Some("http://127.0.0.1:9".into());
    app.montrer_collaborer();
    cliquer(&mut app, "Changer les codes…");
    cliquer(&mut app, "Retour");
    assert_eq!(
        code(&app),
        Some(ancien.texte_lire()),
        "Retour ne change rien"
    );
    cliquer(&mut app, "Changer les codes…");
    cliquer(&mut app, "Changer les codes");
    let neuf = code(&app).expect("toujours partagé");
    assert_ne!(neuf, ancien.texte_lire(), "une salle neuve");
    assert!(app.store.partage.is_some(), "le canevas y part");

    let lire = Code::lire_le_texte(&ancien.texte_lire()).expect("un code lisible");
    partager(&mut app, lire, &[]);
    app.repondre_a_collaborer(super::super::changer::CHANGER_VRAIMENT);
    assert_eq!(
        code(&app),
        Some(ancien.texte_lire()),
        "un lecteur ne change rien"
    );
}
