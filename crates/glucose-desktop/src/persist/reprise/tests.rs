//! **Ce qui se rouvre au lancement**, par la vraie application : un lancement est une
//! application neuve à qui l'on demande `retrouver_le_travail`.

use crate::persist::disque::tests::{application, dossier, image_suivante, noter};

/// **Le dernier document se rouvre**, même fermé proprement : c'est là qu'on travaillait.
#[test]
fn test_le_dernier_document_se_rouvre_au_lancement() {
    let d = dossier("reprise-dernier");
    let chemin = d.join("carnet.glucose");
    let mut app = application(&d);
    noter(&mut app, "c1", "une idée");
    app.save_to(chemin.clone());
    image_suivante(&mut app);
    assert!(app.request_close(), "une fermeture propre");
    let attendu = app.store.project.clone();
    drop(app);

    let mut relance = application(&d);
    relance.retrouver_le_travail();
    assert_eq!(relance.project_path.as_deref(), Some(chemin.as_path()));
    assert_eq!(relance.store.project, attendu);
}

/// Entre le dernier document et un brouillon qu'un plantage a laissé, **le plus récent**
/// l'emporte : c'est celui sur lequel on travaillait.
#[test]
fn test_le_plus_recent_l_emporte() {
    let d = dossier("reprise-plus-recent");
    let chemin = d.join("ancien.glucose");
    let mut app = application(&d);
    noter(&mut app, "c1", "nommé");
    app.save_to(chemin.clone());
    image_suivante(&mut app);
    assert!(app.request_close());
    drop(app);

    // Plus tard, un document sans nom, et un plantage.
    std::thread::sleep(std::time::Duration::from_millis(20));
    let mut sans_nom = application(&d);
    noter(&mut sans_nom, "c2", "sans nom");
    image_suivante(&mut sans_nom);
    let attendu = sans_nom.store.project.clone();
    drop(sans_nom);

    let mut relance = application(&d);
    relance.retrouver_le_travail();
    assert!(relance.project_path.is_none(), "le brouillon, plus récent");
    assert_eq!(relance.store.project, attendu);
}

/// Un document qu'une autre fenêtre tient n'est pas repris : il est ouvert là-bas.
#[test]
fn test_un_document_tenu_ailleurs_n_est_pas_repris() {
    let d = dossier("reprise-tenu");
    let chemin = d.join("tenu.glucose");
    let mut premiere = application(&d);
    noter(&mut premiere, "c1", "ouvert ici");
    premiere.save_to(chemin.clone());
    image_suivante(&mut premiere);

    let mut seconde = application(&d);
    let accueil = seconde.store.project.clone();
    seconde.retrouver_le_travail();
    if cfg!(windows) {
        assert!(seconde.project_path.is_none());
        assert_eq!(
            seconde.store.project, accueil,
            "la seconde garde son accueil"
        );
    }
    drop(premiere);
}
