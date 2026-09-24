//! **Le filet de la frappe, par la vraie application.** Un arrêt brutal est une application
//! qu'on lâche sans la fermer : ce qui a été confié au scribe est sur le disque, rien d'autre.

use crate::app::GlucoseApp;
use crate::interactions::text_edit::keys::Command;
use crate::persist::disque::tests::{application, dossier, image_suivante, noter, rouvrir};
use glucose_core::types::Annotation;
use std::path::Path;

fn taper(app: &mut GlucoseApp, texte: &str) {
    app.apply_text_command(Command::Insert(texte.to_string()), false);
}

fn texte_de(app: &GlucoseApp, id: &str) -> String {
    let p = &app.store.project;
    p.annotation(&p.active_board_id, id)
        .and_then(Annotation::own_text)
        .expect("la carte est là")
}

fn saisies(d: &Path) -> usize {
    std::fs::read_dir(d.join("brouillons")).map_or(0, |l| {
        l.flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "saisie"))
            .count()
    })
}

fn toast(app: &GlucoseApp) -> String {
    app.ui
        .current_toast
        .as_ref()
        .map(|t| t.message.clone())
        .unwrap_or_default()
}

/// Un document nommé, une carte `c1` qui dit « avant », et la carte ouverte en édition, le
/// curseur à la fin.
fn en_train_d_ecrire(d: &Path) -> GlucoseApp {
    let mut app = application(d);
    noter(&mut app, "c1", "avant");
    app.save_to(d.join("notes.glucose"));
    image_suivante(&mut app);
    app.start_text_edit("c1".into(), "avant".into());
    app
}

/// **Le texte tapé survit à un arrêt** : il est gardé à chaque image, le lancement suivant
/// rouvre le document et le lui rend — par un geste, que `Ctrl+Z` retire. Rendu, son fichier
/// de saisie s'efface.
#[test]
fn test_le_texte_tape_survit_a_un_arret() {
    let d = dossier("frappe-arret");
    let mut app = en_train_d_ecrire(&d);
    taper(&mut app, ", puis dix minutes de note");
    image_suivante(&mut app);
    assert_eq!(saisies(&d), 1, "le texte en cours est gardé");
    assert_eq!(
        texte_de(&app, "c1"),
        "avant",
        "le document ne l'a pas encore"
    );
    drop(app);

    let mut relance = application(&d);
    relance.retrouver_le_travail();
    assert_eq!(
        relance.project_path.as_deref(),
        Some(&*d.join("notes.glucose"))
    );
    assert_eq!(texte_de(&relance, "c1"), "avant, puis dix minutes de note");
    assert!(toast(&relance).contains("tapais"), "{}", toast(&relance));
    image_suivante(&mut relance);
    assert_eq!(saisies(&d), 0, "rendu, il n'a plus à être gardé");

    relance.store.undo();
    assert_eq!(texte_de(&relance, "c1"), "avant", "Ctrl+Z le retire");
    relance.store.redo();
    image_suivante(&mut relance);
    drop(relance);
    let troisieme = rouvrir(&d, &d.join("notes.glucose"));
    assert_eq!(
        texte_de(&troisieme, "c1"),
        "avant, puis dix minutes de note"
    );
    assert!(
        !toast(&troisieme).contains("tapais"),
        "rien de plus à rendre"
    );
}

/// **Une saisie validée ne revient pas** : sa validation est un geste écrit, et son texte en
/// cours s'efface derrière lui.
#[test]
fn test_une_saisie_validee_s_oublie() {
    let d = dossier("frappe-validee");
    let mut app = en_train_d_ecrire(&d);
    taper(&mut app, " et après");
    image_suivante(&mut app);
    assert_eq!(saisies(&d), 1);
    app.commit_editing();
    image_suivante(&mut app);
    assert_eq!(saisies(&d), 0, "validée, elle est dans l'histoire");
    drop(app);
    let relu = rouvrir(&d, &d.join("notes.glucose"));
    assert_eq!(texte_de(&relu, "c1"), "avant et après");
    assert!(!toast(&relu).contains("tapais"));
}

/// **Une saisie qu'un geste a suivie ne vaut plus** : c'est le cas d'un arrêt entre l'écriture
/// d'une validation et l'effacement de sa saisie. Rendre ce texte écraserait ce qui a été
/// fait depuis ; il est ignoré, puis effacé.
#[test]
fn test_une_saisie_suivie_d_un_geste_ne_vaut_plus() {
    let d = dossier("frappe-caduque");
    let mut app = en_train_d_ecrire(&d);
    taper(&mut app, " (brouillon d'idée)");
    image_suivante(&mut app);
    let fichier = std::fs::read_dir(d.join("brouillons"))
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "saisie"))
        .expect("gardée");
    let gardee = std::fs::read(&fichier).unwrap();
    app.commit_editing();
    app.start_text_edit("c1".into(), texte_de(&app, "c1"));
    app.apply_text_command(Command::SelectAll, false);
    taper(&mut app, "tout autre chose");
    app.commit_editing();
    image_suivante(&mut app);
    drop(app);
    std::fs::write(&fichier, gardee).unwrap();

    let mut relu = rouvrir(&d, &d.join("notes.glucose"));
    assert_eq!(texte_de(&relu, "c1"), "tout autre chose");
    assert!(!toast(&relu).contains("tapais"), "{}", toast(&relu));
    image_suivante(&mut relu);
    assert_eq!(saisies(&d), 0, "caduque, elle s'efface");
}

/// **Un document qu'une autre fenêtre écrit ne s'écrit pas deux fois** : la seconde l'ouvre
/// comme un fichier en lecture seule, et ce qu'elle y tape — dès la première lettre — part
/// dans un brouillon, que le lancement suivant retrouve avec son texte.
#[test]
fn test_une_seconde_fenetre_tape_dans_un_brouillon() {
    let d = dossier("frappe-seconde");
    let mut premiere = en_train_d_ecrire(&d);
    premiere.commit_editing();
    image_suivante(&mut premiere);
    let chemin = d.join("notes.glucose");
    let avant = std::fs::read(&chemin).unwrap();

    let mut seconde = rouvrir(&d, &chemin);
    if cfg!(windows) {
        assert!(
            toast(&seconde).contains("ouvert en écriture ailleurs"),
            "{}",
            toast(&seconde)
        );
    }
    seconde.start_text_edit("c1".into(), "avant".into());
    image_suivante(&mut seconde);
    assert!(
        seconde.disque.ecriture.is_none(),
        "ouvrir une carte n'est pas encore du travail"
    );
    taper(&mut seconde, " — depuis la seconde fenêtre");
    image_suivante(&mut seconde);
    let brouillon = seconde
        .disque
        .ecriture
        .as_ref()
        .expect("la frappe en donne un");
    assert!(
        brouillon.brouillon,
        "dans un brouillon, pas dans le document"
    );
    drop(seconde);
    drop(premiere);
    assert_eq!(
        std::fs::read(&chemin).unwrap(),
        avant,
        "le document est intact"
    );

    let mut relance = application(&d);
    relance.retrouver_le_travail();
    assert!(
        relance.project_path.is_none(),
        "c'est le brouillon qui revient"
    );
    assert_eq!(
        texte_de(&relance, "c1"),
        "avant — depuis la seconde fenêtre"
    );
    assert!(toast(&relance).contains("tapais"), "{}", toast(&relance));
}

/// **« Ne pas enregistrer » ne laisse rien** : ni le brouillon, ni le texte qu'on y tapait.
#[test]
fn test_abandonner_un_brouillon_efface_aussi_sa_frappe() {
    let d = dossier("frappe-abandon");
    let mut app = application(&d);
    noter(&mut app, "c1", "sans nom");
    image_suivante(&mut app);
    app.start_text_edit("c1".into(), "sans nom".into());
    taper(&mut app, ", et une suite");
    image_suivante(&mut app);
    assert_eq!(saisies(&d), 1);
    assert!(app.close_with(crate::persist::close::CloseChoice::Discard));
    let restants = std::fs::read_dir(d.join("brouillons")).unwrap().count();
    assert_eq!(restants, 0, "plus rien à rouvrir");
}

/// **Fermer la fenêtre pendant une frappe valide le texte** — comme un clic ailleurs. Il ne
/// reste rien à rendre au lancement suivant.
#[test]
fn test_fermer_la_fenetre_pendant_une_frappe_valide_le_texte() {
    let d = dossier("frappe-fermer");
    let mut app = en_train_d_ecrire(&d);
    taper(&mut app, " — dernière phrase");
    assert!(
        app.request_close(),
        "un document nommé se ferme sans question"
    );
    assert!(app.editing_session.is_none());
    drop(app);
    assert_eq!(saisies(&d), 0);
    let relu = rouvrir(&d, &d.join("notes.glucose"));
    assert_eq!(texte_de(&relu, "c1"), "avant — dernière phrase");
}
