//! **Une application d'épreuve n'habite pas chez l'utilisateur.**
//!
//! Le défaut s'est présenté deux fois, sous deux noms : les brouillons, puis les aperçus. Les
//! deux fois, une suite d'épreuves déposait des fichiers dans son vrai dossier Glucose. Cette
//! épreuve regarde tout ce qu'une application écrit de durable, d'un coup.

use crate::app::GlucoseApp;

#[test]
fn test_une_application_hors_lancement_n_ecrit_rien_chez_l_utilisateur() {
    let app = GlucoseApp::new();
    let chez_lui = crate::present::souvenir::dossier();
    let apercus = app
        .renderer
        .magasin
        .atelier
        .dossier_des_apercus()
        .expect("les aperçus sont gardés, comme au vrai lancement");
    assert!(
        app.depot.telechargements.is_none(),
        "une application d'épreuve ne connaît pas ses téléchargements"
    );
    for (quoi, chemin) in [
        ("les brouillons", app.disque.brouillons.as_path()),
        ("les aperçus", apercus),
        (
            "le souvenir de la carte",
            app.souvenir_de_la_carte.as_path(),
        ),
    ] {
        assert!(
            !chemin.starts_with(&chez_lui),
            "{quoi} visent le dossier de l'utilisateur : {}",
            chemin.display()
        );
    }
}

/// **ACCUEIL-1 — la carte d'accueil n'est pas un choix de l'utilisateur** : elle naît par la
/// fabrique d'une carte posée d'un clic, qui la choisit — le document d'accueil part sans rien
/// de choisi. Sinon, au premier lancement, la barre d'action disait « 1 sélectionné », et un
/// agent lisait que l'utilisateur désignait la carte (fiche 81 : Protéo écrivait « il désigne :
/// Bienvenue dans Glucose » alors que personne n'avait rien montré).
#[test]
fn test_le_document_d_accueil_ne_choisit_rien() {
    let app = GlucoseApp::new();
    let tableau = app.store.active_board().expect("le tableau d'accueil");
    assert!(
        tableau.annotations.iter().any(|a| a.id() == "welcome-card"),
        "la carte d'accueil est posée"
    );
    assert!(
        app.store.selected_annotation_ids.is_empty(),
        "rien n'est choisi au lancement : {:?}",
        app.store.selected_annotation_ids
    );
    assert!(app.store.selected_image_ids.is_empty());
    assert!(app.store.selected_folder_id.is_none());
}

/// **Le vrai lancement, lui, y habite tout entier** : un seul appel, trois chemins.
#[test]
fn test_habiter_place_tout_dans_le_meme_dossier() {
    let mut app = GlucoseApp::new();
    let dossier = std::env::temp_dir().join("glucose-epreuve-habiter");
    app.habiter(&dossier);
    let apercus = app.renderer.magasin.atelier.dossier_des_apercus();
    assert!(app.disque.brouillons.starts_with(&dossier));
    assert!(apercus.is_some_and(|a| a.starts_with(&dossier)));
    assert!(app.souvenir_de_la_carte.starts_with(&dossier));
}

#[test]
fn test_accueil_2_le_noeud_de_bienvenue_ne_nait_pas_sous_la_barre() {
    let mut app = crate::app::GlucoseApp::new();
    app.montrer_l_accueil();
    let vue = app.store.viewport();
    let haut = f64::from(app.ui.header_height());
    assert!(
        vue.y > haut,
        "le titre doit paraître sous la barre : {} <= {haut}",
        vue.y
    );
    assert!(vue.x > 0.0, "et décollé du bord");
    assert!(
        !app.store.journal.can_undo(),
        "regarder ailleurs n'est pas un geste à défaire"
    );
}

#[test]
fn test_accueil_2_seul_un_lancement_sans_rien_a_rouvrir_le_montre() {
    let reprise = include_str!("../../persist/reprise.rs");
    assert!(reprise.contains("accueil_a_montrer = rouvert == \"rien\""));
    let fenetre = include_str!("../fenetre.rs");
    assert!(fenetre.contains("std::mem::take(&mut self.lancement.accueil_a_montrer)"));
}
