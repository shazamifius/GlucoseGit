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
    assert!(
        crate::persist::close::tests::fermer(&mut app),
        "une fermeture propre"
    );
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
    assert!(crate::persist::close::tests::fermer(&mut app));
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

fn brouillons(d: &std::path::Path) -> usize {
    std::fs::read_dir(d.join("brouillons")).map_or(0, |l| {
        l.flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "glucose"))
            .count()
    })
}

fn toast(app: &crate::app::GlucoseApp) -> String {
    app.ui
        .current_toast
        .as_ref()
        .map(|t| t.message.clone())
        .unwrap_or_default()
}

/// Un travail sans nom laissé par un plantage, dans `d` ; rend son état.
fn un_plantage_sans_nom(d: &std::path::Path, id: &str) -> glucose_core::types::Project {
    let mut sans_nom = application(d);
    noter(&mut sans_nom, id, "du travail sans nom");
    image_suivante(&mut sans_nom);
    sans_nom.store.project.clone()
}

/// **Un brouillon passe avant un document plus récent** (BROUILLON-1) : le document nommé est
/// en sûreté et `Ctrl+O` le rouvre ; le brouillon n'a que le lancement pour reparaître.
#[test]
fn test_un_brouillon_passe_avant_un_document_plus_recent() {
    let d = dossier("reprise-brouillon-d-abord");
    let attendu = un_plantage_sans_nom(&d, "b1");
    std::thread::sleep(std::time::Duration::from_millis(20));
    let mut nomme = application(&d);
    noter(&mut nomme, "c1", "plus récent, et nommé");
    nomme.save_to(d.join("recent.glucose"));
    image_suivante(&mut nomme);
    assert!(crate::persist::close::tests::fermer(&mut nomme));
    drop(nomme);

    let mut relance = application(&d);
    relance.retrouver_le_travail();
    assert!(relance.project_path.is_none(), "le brouillon d'abord");
    assert_eq!(relance.store.project, attendu);
}

/// **Les autres brouillons se disent**, et chacun reparaît à son tour : aucun n'attend en
/// silence.
#[test]
fn test_les_autres_brouillons_se_disent_et_reparaissent_a_leur_tour() {
    let d = dossier("reprise-deux-brouillons");
    let premier = un_plantage_sans_nom(&d, "b1");
    std::thread::sleep(std::time::Duration::from_millis(20));
    let second = un_plantage_sans_nom(&d, "b2");
    assert_eq!(brouillons(&d), 2);

    let mut relance = application(&d);
    relance.retrouver_le_travail();
    assert_eq!(relance.store.project, second, "le plus récent d'abord");
    assert!(
        toast(&relance).contains("un autre brouillon attend"),
        "{}",
        toast(&relance)
    );
    relance.project_path = Some(d.join("nomme.glucose"));
    assert!(relance.close_with(crate::persist::close::CloseChoice::Save));
    drop(relance);

    let mut encore = application(&d);
    encore.retrouver_le_travail();
    assert_eq!(encore.store.project, premier, "puis l'autre, à son tour");
    assert!(
        !toast(&encore).contains("autre brouillon"),
        "{}",
        toast(&encore)
    );
}

/// **Quitter un document sans nom pose la question** (BROUILLON-1), et aucune réponse ne
/// laisse de brouillon caché : « Annuler » garde tout ; « Ne pas enregistrer » l'efface ;
/// « Enregistrer » lui donne un nom.
#[test]
fn test_quitter_un_document_sans_nom_ne_laisse_rien_de_cache() {
    use crate::persist::close::CloseChoice;
    let d = dossier("reprise-quitter");
    let autre = d.join("autre.glucose");
    let mut a = application(&d);
    a.save_to(autre.clone());
    assert!(a.fermer_le_document());
    drop(a);

    let mut app = application(&d);
    noter(&mut app, "s1", "sans nom");
    image_suivante(&mut app);
    assert!(!app.laisser_avec(CloseChoice::Cancel), "Annuler reste");
    assert_eq!(brouillons(&d), 1, "et garde tout");
    assert!(app.laisser_avec(CloseChoice::Discard));
    app.open_from(autre.clone());
    assert_eq!(brouillons(&d), 0, "abandonné : effacé, pas caché");
    drop(app);

    let mut app = application(&d);
    noter(&mut app, "s2", "à nommer");
    image_suivante(&mut app);
    let travail = app.store.project.clone();
    let nom = d.join("nomme.glucose");
    app.project_path = Some(nom.clone()); // ce que le dialogue répondrait
    assert!(app.laisser_avec(CloseChoice::Save));
    app.open_from(autre);
    assert_eq!(brouillons(&d), 0, "nommé : il n'est plus un brouillon");
    drop(app);
    let relu = crate::persist::disque::tests::rouvrir(&d, &nom);
    assert_eq!(relu.store.project, travail);
}
