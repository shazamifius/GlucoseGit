//! FIN-1 et FIN-SESSION-1 — Glucose finit sans rien perdre, et sans question quand le système
//! finit la session.

use super::*;
use crate::boite_noire::bilan::{bilan, Fin};
use crate::epreuve::Dossier;
use crate::interactions::question::tests::repondre;
use crate::persist::close::tests::{dirty_app, fermer};
use crate::persist::read_project_file;
use crate::ui::question::Reponse;
use std::path::{Path, PathBuf};

/// Les documents du dossier des brouillons.
fn brouillons(dossier: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dossier.join("brouillons"))
        .map(|lus| {
            lus.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e == "glucose"))
                .collect()
        })
        .unwrap_or_default()
}

/// Un document modifié qui habite `dossier`, sa boîte noire ouverte : rend le chemin du journal.
fn travail_en_cours(dossier: &Path) -> (GlucoseApp, PathBuf) {
    let mut app = dirty_app();
    app.habiter(dossier);
    app.chronique.ouvrir_la_boite_noire(dossier);
    let journal = app
        .chronique
        .boite_noire()
        .expect("la boîte noire ouverte")
        .chemin()
        .to_path_buf();
    (app, journal)
}

/// Comment la session s'est close, lu comme le lancement suivant le lira.
fn fin_de(journal: &Path) -> Fin {
    let texte = std::fs::read_to_string(journal).expect("le journal");
    bilan(&texte, None).expect("une session").fin
}

/// **FIN-1 : finir écrit le travail sans rien demander, et clôt la boîte noire** — le travail
/// sans nom devient un brouillon sur le disque, identique au document, et la session se lit
/// close proprement.
#[test]
fn test_fin_1_finir_ecrit_le_travail_sans_question_et_clot_la_boite_noire() {
    let d = Dossier::nouveau("fin-1");
    let (mut app, journal) = travail_en_cours(&d);
    assert!(brouillons(&d).is_empty(), "rien n'est encore écrit");
    app.finir();
    assert!(app.ui.question.is_none(), "personne n'est là pour répondre");
    let ecrits = brouillons(&d);
    assert_eq!(
        ecrits.len(),
        1,
        "le travail sans nom est un brouillon : {ecrits:?}"
    );
    let relu = read_project_file(&ecrits[0]).expect("le brouillon se relit");
    assert_eq!(
        relu.project, app.store.project,
        "le brouillon est le document"
    );
    assert_eq!(fin_de(&journal), Fin::Propre);
}

/// **FIN-1 : un brouillon abandonné ne renaît pas à la sortie.** « Ne pas enregistrer » a réglé
/// son sort ; un geste qui arrive encore avant que la boucle parte (un agent, par exemple) ne le
/// fait pas revenir au lancement suivant.
#[test]
fn test_fin_1_un_brouillon_abandonne_ne_renait_pas() {
    let d = Dossier::nouveau("fin-1-abandon");
    let (mut app, _) = travail_en_cours(&d);
    app.consigner();
    assert_eq!(brouillons(&d).len(), 1, "le brouillon est né");
    assert!(!fermer(&mut app), "la croix demande");
    repondre(&mut app, Reponse::Non);
    assert!(brouillons(&d).is_empty(), "« Ne pas enregistrer » l'efface");
    let tableau = app.store.project.active_board_id.clone();
    let mut encore = app
        .store
        .project
        .boards
        .iter()
        .flat_map(|b| &b.annotations)
        .find(|a| a.id() == "travail-en-cours")
        .expect("la note de dirty_app")
        .clone();
    if let glucose_core::types::Annotation::Text { id, .. } = &mut encore {
        *id = "encore".into();
    }
    let avant = app.store.version;
    app.store.add_annotation(&tableau, encore);
    assert_ne!(app.store.version, avant, "le geste a bien eu lieu");
    app.finir();
    assert!(brouillons(&d).is_empty(), "il ne renaît pas");
}

#[cfg(windows)]
mod windows_seulement {
    use super::*;
    use crate::plateforme::fin::tests::{detruire, envoyer_la_fin, fenetre_temoin, temoin};
    use crate::plateforme::fin::Session;
    use crate::ui::question::Suite;
    use winit::event::WindowEvent;

    /// **FIN-SESSION-1, par le vrai chemin** : Windows annonce la fin, la fenêtre transmet la
    /// fermeture à `winit` (la procédure témoin tient sa place), et la fermeture que `winit`
    /// rend à l'application finit tout — sans question, la boîte noire close, le brouillon écrit.
    #[test]
    fn test_fin_session_1_la_fin_de_session_finit_sans_question() {
        let d = Dossier::nouveau("fin-session-1");
        let (mut app, journal) = travail_en_cours(&d);
        let fenetre = fenetre_temoin();
        app.lancement.session = Session::de_la_poignee(fenetre);
        envoyer_la_fin(fenetre, true);
        assert_eq!(temoin().fermetures, 1, "winit reçoit la fermeture");
        app.aiguiller(WindowEvent::CloseRequested);
        assert!(app.ui.question.is_none(), "personne n'est là pour répondre");
        assert!(
            app.ui.fermer_la_fenetre,
            "la fenêtre part : le Restart Manager attend"
        );
        assert_eq!(brouillons(&d).len(), 1, "le travail est sur le disque");
        assert_eq!(
            fin_de(&journal),
            Fin::Propre,
            "une extinction n'est pas un plantage"
        );
        detruire(fenetre);
    }

    /// **À l'envers : la croix, elle, demande toujours** — sur la même fenêtre veillée, quand
    /// l'extinction est annulée.
    #[test]
    fn test_fin_session_1_sans_fin_de_session_la_croix_demande() {
        let d = Dossier::nouveau("fin-session-1-croix");
        let (mut app, journal) = travail_en_cours(&d);
        let fenetre = fenetre_temoin();
        app.lancement.session = Session::de_la_poignee(fenetre);
        envoyer_la_fin(fenetre, false);
        app.aiguiller(WindowEvent::CloseRequested);
        assert!(
            matches!(app.ui.question, Some((_, Suite::Fermer(_)))),
            "la question du travail non enregistré"
        );
        assert!(!app.ui.fermer_la_fenetre, "la fenêtre attend la réponse");
        assert_eq!(fin_de(&journal), Fin::Interrompue, "rien n'est clos");
        detruire(fenetre);
    }
}
