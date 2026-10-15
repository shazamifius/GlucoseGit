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

/// **REPRISE-1** (fiche 72) : ce que le lancement a rouvert s'écrit dans la boîte noire — le
/// brouillon qu'un arrêt brutal a laissé, et ses images. Le 15/10, un tableau de 6 000 images a
/// disparu au téléphone sans qu'aucune ligne dise si le lancement l'avait cherché.
#[test]
fn test_reprise_1_ce_que_le_lancement_rouvre_s_ecrit() {
    use crate::persist::disque::tests::png;
    let d = dossier("reprise-boite-noire");
    let mut tuee = application(&d);
    let sources: Vec<_> = (0..3u8)
        .map(|i| png(&d, &format!("{i}.png"), 40 * i))
        .collect();
    tuee.import_image_files(&sources);
    image_suivante(&mut tuee);
    // Ce qu'un processus tué laisse : rien n'est fermé, le brouillon reste.
    drop(tuee);

    let mut relance = application(&d);
    relance.chronique.ouvrir_la_boite_noire(&d);
    relance.retrouver_le_travail();
    assert_eq!(relance.store.project.toutes_les_images().count(), 3);
    let boite = relance.chronique.boite_noire().expect("ouverte");
    boite.synchroniser();
    let texte = std::fs::read_to_string(boite.chemin()).expect("la session");
    let ligne = texte
        .lines()
        .find(|l| l.contains("\"type\":\"reprise\""))
        .expect("une ligne reprise");
    assert!(ligne.contains("\"rouvert\":\"brouillon\""), "{ligne}");
    assert!(ligne.contains("\"brouillons\":1"), "{ligne}");
    assert!(ligne.contains("\"tenus\":0"), "{ligne}");
    assert!(ligne.contains("\"images\":3"), "{ligne}");
}

/// **LIBERE-1** (fiche 72) : le scénario du téléphone. Un Glucose tient encore son brouillon —
/// le processus qu'Android vient de tuer n'a pas fini de mourir — quand le nouveau démarre : la
/// reprise le saute, et le dit (`tenus`). Puis l'ancien le lâche : le travail revient, de
/// lui-même, dans le document d'accueil resté intact. Sous Windows, le verrou est un mode de
/// partage qu'on ne peut pas attendre : l'épreuve ne s'y joue pas.
#[cfg(not(windows))]
#[test]
fn test_libere_1_un_brouillon_tenu_au_lancement_revient_quand_on_le_lache() {
    use crate::persist::disque::tests::png;
    let d = dossier("reprise-libere");
    let mut mourant = application(&d);
    let sources: Vec<_> = (0..3u8)
        .map(|i| png(&d, &format!("{i}.png"), 40 * i))
        .collect();
    mourant.import_image_files(&sources);
    image_suivante(&mut mourant);

    let mut neuf = application(&d);
    neuf.chronique.ouvrir_la_boite_noire(&d);
    neuf.retrouver_le_travail();
    assert_eq!(
        neuf.store.project.toutes_les_images().count(),
        0,
        "tenu : sauté"
    );
    let boite = neuf.chronique.boite_noire().expect("ouverte");
    boite.synchroniser();
    let texte = std::fs::read_to_string(boite.chemin()).expect("la session");
    assert!(texte.contains("\"rouvert\":\"rien\""), "{texte}");
    assert!(texte.contains("\"tenus\":1"), "{texte}");

    drop(mourant);
    let fin = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while neuf.store.project.toutes_les_images().count() == 0 && std::time::Instant::now() < fin {
        neuf.suivre_les_brouillons_liberes();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        neuf.store.project.toutes_les_images().count(),
        3,
        "le travail est revenu"
    );
    let toast = neuf.ui.toast_message().unwrap_or_default();
    assert!(toast.starts_with("Ton travail est revenu"), "{toast}");
}

/// **Libéré trop tard, il ne remplace rien** : le document d'accueil a déjà reçu un geste — le
/// brouillon attend le prochain lancement, et le dit.
#[cfg(not(windows))]
#[test]
fn test_libere_1_un_brouillon_libere_ne_remplace_pas_un_travail_commence() {
    let d = dossier("reprise-libere-tard");
    let mut mourant = application(&d);
    noter(&mut mourant, "c1", "l'ancien");
    image_suivante(&mut mourant);

    let mut neuf = application(&d);
    neuf.retrouver_le_travail();
    noter(&mut neuf, "c2", "le nouveau, déjà commencé");
    image_suivante(&mut neuf);
    let avant = neuf.store.project.clone();
    drop(mourant);
    let fin = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let attend = |app: &crate::app::GlucoseApp| {
        !app.ui
            .toast_message()
            .is_some_and(|m| m.contains("prochain lancement"))
    };
    while attend(&neuf) && std::time::Instant::now() < fin {
        neuf.suivre_les_brouillons_liberes();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(neuf.store.project, avant, "rien n'est remplacé");
    assert!(
        !attend(&neuf),
        "le brouillon libéré attend le prochain lancement, et le dit"
    );
}
