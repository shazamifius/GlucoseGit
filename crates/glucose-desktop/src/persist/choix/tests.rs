//! DIAL-2 — le sélecteur ne tient plus la boucle, et ce qui suit son choix (fiche 58) : surtout
//! les chemins qui écrivent ses données — fermer ou quitter un travail sans nom en
//! l'enregistrant. Des dossiers d'épreuve, jamais les siens.

use crate::app::GlucoseApp;
use crate::dialogue::epreuve::choisir;
use crate::interactions::question::tests::repondre;
use crate::persist::close::Apres;
use crate::ui::question::Reponse;
use std::path::PathBuf;

/// Un dossier d'épreuve, effacé à la fin.
struct Dossier(PathBuf);

impl Dossier {
    fn nouveau(nom: &str) -> Self {
        let p = std::env::temp_dir().join(format!("glucose-choix-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).expect("dossier d'épreuve");
        Self(p)
    }
}

impl Drop for Dossier {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Une application avec du travail sans nom.
fn travail_sans_nom() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    crate::persist::disque::tests::noter(&mut app, "n", "du travail sans nom");
    assert!(app.is_dirty() && app.project_path.is_none());
    app
}

/// **Fermer un travail sans nom en l'enregistrant** : le sélecteur demande où ; le choix revenu,
/// le document s'écrit là — extension comprise —, puis la fenêtre part.
#[test]
fn test_dial_2_fermer_en_enregistrant_ecrit_puis_ferme() {
    let d = Dossier::nouveau("fermer");
    let mut app = travail_sans_nom();
    app.fermer_puis(Apres::Quitter);
    choisir(Some(vec![d.0.join("carnet")]));
    repondre(&mut app, Reponse::Oui);
    assert!(!app.ui.fermer_la_fenetre, "rien ne part avant le choix");
    app.suivre_les_fichiers_choisis();
    let ecrit = d.0.join("carnet.glucose");
    assert!(ecrit.exists(), "le document s'est écrit");
    assert_eq!(app.project_path.as_ref(), Some(&ecrit));
    assert!(app.ui.fermer_la_fenetre, "puis la fenêtre part");
}

/// **Renoncer au sélecteur ne ferme rien et ne perd rien** : la fenêtre reste, le travail aussi.
#[test]
fn test_dial_2_renoncer_au_selecteur_garde_tout() {
    let mut app = travail_sans_nom();
    app.fermer_puis(Apres::Quitter);
    choisir(None);
    repondre(&mut app, Reponse::Oui);
    app.suivre_les_fichiers_choisis();
    assert!(!app.ui.fermer_la_fenetre, "la fenêtre reste");
    assert!(app.is_dirty(), "le travail reste");
    assert!(app.project_path.is_none());
}

/// **Quitter un travail sans nom pour un nouveau, en l'enregistrant** : il s'écrit, puis le
/// vierge le remplace (BROUILLON-1).
#[test]
fn test_dial_2_laisser_en_enregistrant_ecrit_puis_continue() {
    let d = Dossier::nouveau("laisser");
    let mut app = travail_sans_nom();
    app.nouveau_document();
    choisir(Some(vec![d.0.join("garde.glucose")]));
    repondre(&mut app, Reponse::Oui);
    app.suivre_les_fichiers_choisis();
    assert!(d.0.join("garde.glucose").exists(), "il s'est écrit");
    assert!(app.project_path.is_none(), "le vierge l'a remplacé");
    assert!(
        app.store.active_board().unwrap().annotations.is_empty(),
        "vierge"
    );
}

/// **Une mise à jour qui attendait l'enregistrement est reportée** si l'on renonce — et le dit.
#[test]
fn test_dial_2_la_mise_a_jour_se_reporte_si_l_on_renonce() {
    let mut app = travail_sans_nom();
    app.ui.current_toast = None;
    app.fermer_puis(Apres::Installer(PathBuf::from("glucose.exe")));
    choisir(None);
    repondre(&mut app, Reponse::Oui);
    app.suivre_les_fichiers_choisis();
    let toast = app.ui.current_toast.as_ref().expect("elle le dit");
    assert!(toast.message.contains("reportée"), "{}", toast.message);
    assert!(!app.ui.fermer_la_fenetre);
}

/// **Exporter** : le choix revenu, le tableau s'écrit là.
#[test]
fn test_dial_2_exporter_ecrit_au_choix() {
    let d = Dossier::nouveau("exporter");
    let mut app = travail_sans_nom();
    let sortie = d.0.join("tableau.md");
    choisir(Some(vec![sortie.clone()]));
    app.export_board();
    app.suivre_les_fichiers_choisis();
    assert!(sortie.exists(), "l'export s'est écrit");
}

/// **Ouvrir** : le choix revenu, le document qu'on regarde est laissé (il est propre), et
/// l'autre s'ouvre.
#[test]
fn test_dial_2_ouvrir_au_choix() {
    let d = Dossier::nouveau("ouvrir");
    let autre = d.0.join("autre.glucose");
    let mut source = GlucoseApp::new();
    crate::persist::disque::tests::noter(&mut source, "x", "l'autre document");
    source.save_to(autre.clone());
    drop(source);
    let mut app = GlucoseApp::new();
    choisir(Some(vec![autre.clone()]));
    app.open_project();
    app.suivre_les_fichiers_choisis();
    assert_eq!(app.project_path.as_ref(), Some(&autre));
}

/// **Un enregistrement raté ne ferme rien, et ne laisse rien partir** : le sélecteur rend un
/// dossier qui n'existe pas — l'écriture échoue, le dit, et la fenêtre reste ; quitter pour un
/// nouveau document ne remplace pas le travail non écrit.
#[test]
fn test_dial_2_un_enregistrement_rate_ne_ferme_rien() {
    let nulle_part = PathBuf::from("C:/nulle-part/dossier-absent/carnet.glucose");
    let mut app = travail_sans_nom();
    app.fermer_puis(Apres::Quitter);
    choisir(Some(vec![nulle_part.clone()]));
    repondre(&mut app, Reponse::Oui);
    app.suivre_les_fichiers_choisis();
    assert!(!app.ui.fermer_la_fenetre, "la fenêtre reste");
    assert!(app.is_dirty(), "le travail reste");

    let mut app = travail_sans_nom();
    app.nouveau_document();
    choisir(Some(vec![nulle_part]));
    repondre(&mut app, Reponse::Oui);
    app.suivre_les_fichiers_choisis();
    assert!(
        !app.store.active_board().unwrap().annotations.is_empty(),
        "le travail n'a pas été remplacé"
    );
}
