//! DOCUMENTS-2 — renommer, dupliquer, supprimer, et l'appui long qui y mène.

use super::*;
use crate::ui::question::Reponse;
use glucose_core::types::Annotation;

/// Un dossier à soi, effacé à la fin.
struct Bac(PathBuf);

impl Bac {
    fn neuf(nom: &str) -> Self {
        let d = std::env::temp_dir().join(format!("glucose-gestion-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("un dossier d'épreuve");
        Self(d)
    }
    fn documents(&self) -> PathBuf {
        self.0.join("documents")
    }
}

impl Drop for Bac {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Glucose au téléphone, dans ce dossier, avec « Canevas 1 » rangé et **ouvert**, qui porte
/// un texte.
fn avec_un_document_ouvert(bac: &Bac) -> (GlucoseApp, PathBuf) {
    let mut app = GlucoseApp::new();
    app.habiter(&bac.0);
    app.ui.questions_dessinees = true;
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("t1", 0.0, 0.0, "à garder"));
    app.consigner();
    app.nouveau_document();
    let chemin = bac.documents().join("Canevas 1.glucose");
    assert!(chemin.exists(), "rangé");
    app.open_from(chemin.clone());
    assert_eq!(app.project_path.as_deref(), Some(chemin.as_path()));
    (app, chemin)
}

/// Le texte `id` est-il dans le document de ce fichier ? Lu par une autre application.
fn porte(chemin: &Path, id: &str) -> bool {
    let mut lecteur = GlucoseApp::new();
    lecteur.open_from(chemin.to_path_buf());
    lecteur
        .store
        .active_board()
        .is_some_and(|b| b.annotations.iter().any(|a| a.id() == id))
}

#[test]
fn test_documents_2_un_nom_se_valide_ou_dit_pourquoi() {
    assert_eq!(nom_valide("  Planches Mary "), Ok("Planches Mary".into()));
    assert_eq!(
        nom_valide("Mary.glucose"),
        Ok("Mary".into()),
        "l'extension tapée ne se double pas"
    );
    for refuse in [
        "", "   ", "a/b", "a\\b", "a:b", "..", ".cache", "fin.", "a\u{7}b",
    ] {
        assert!(nom_valide(refuse).is_err(), "« {refuse} » est refusé");
    }
    let long = "é".repeat(LONGUEUR_MAX + 1);
    assert!(nom_valide(&long).is_err(), "trop long");
    assert!(
        nom_valide(&"é".repeat(LONGUEUR_MAX)).is_ok(),
        "à la limite, oui"
    );
    assert!(
        format!("{}.{FILE_EXTENSION}", "\u{10348}".repeat(LONGUEUR_MAX)).len() <= 255,
        "même en caractères de quatre octets, le fichier tient en 255"
    );
}

#[test]
fn test_documents_2_la_copie_prend_le_premier_nom_libre() {
    let bac = Bac::neuf("copie");
    let x = bac.0.join("X.glucose");
    assert_eq!(copie_libre(&x), bac.0.join("X copie.glucose"));
    std::fs::write(bac.0.join("X copie.glucose"), b"x").expect("écrire");
    assert_eq!(copie_libre(&x), bac.0.join("X copie 2.glucose"));
}

/// **Renommer un document fermé** : le fichier change de nom, rien d'autre ; la liste revient.
#[test]
fn test_documents_2_renommer_un_document_ferme() {
    let bac = Bac::neuf("renommer");
    let (mut app, chemin) = avec_un_document_ouvert(&bac);
    app.adopter_un_document_vierge();
    app.renommer_le_document(&chemin, "Planches Mary");
    let cible = bac.documents().join("Planches Mary.glucose");
    assert!(
        cible.exists() && !chemin.exists(),
        "le fichier a changé de nom"
    );
    assert!(porte(&cible, "t1"), "son contenu l'a suivi");
    let Some((liste, Suite::Ouvrir(_))) = &app.ui.question else {
        panic!("la liste revient");
    };
    assert!(
        liste.texte.starts_with("Renommé « Planches Mary »."),
        "et dit ce qui vient d'arriver : {}",
        liste.texte
    );
}

/// **Un nom pris ou refusé se redemande**, avec ce qu'on avait tapé ; aucun fichier ne bouge.
#[test]
fn test_documents_2_un_nom_pris_ou_refuse_se_redemande() {
    let bac = Bac::neuf("pris");
    let (mut app, chemin) = avec_un_document_ouvert(&bac);
    let autre = bac.documents().join("Mary.glucose");
    std::fs::write(&autre, b"un autre").expect("écrire");
    for tape in ["Mary", "a/b"] {
        app.renommer_le_document(&chemin, tape);
        let Some((question, Suite::Renommer(_))) = &app.ui.question else {
            panic!("« {tape} » : le nom se redemande");
        };
        let champ = question.champ.as_ref().expect("un champ");
        assert_eq!(champ.texte, tape, "ce qu'on avait tapé reste");
        assert!(!question.texte.is_empty(), "et la question dit pourquoi");
        assert!(chemin.exists(), "rien n'a bougé");
    }
    assert_eq!(
        std::fs::read(&autre).expect("lire"),
        b"un autre",
        "jamais écrasé"
    );
}

/// **Renommer le document ouvert** : il continue de s'écrire sous son nouveau nom, histoire
/// comprise, et l'ancien fichier s'en va.
#[test]
fn test_documents_2_renommer_le_document_ouvert() {
    let bac = Bac::neuf("ouvert");
    let (mut app, chemin) = avec_un_document_ouvert(&bac);
    app.renommer_le_document(&chemin, "Planches");
    let cible = bac.documents().join("Planches.glucose");
    assert_eq!(app.project_path.as_deref(), Some(cible.as_path()));
    assert!(!chemin.exists(), "l'ancien s'en va");
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("t2", 0.0, 50.0, "après"));
    app.consigner();
    app.adopter_un_document_vierge();
    assert!(porte(&cible, "t1") && porte(&cible, "t2"), "tout y est");
}

/// **Dupliquer le document ouvert** porte jusqu'à son dernier geste.
#[test]
fn test_documents_2_dupliquer_porte_le_dernier_geste() {
    let bac = Bac::neuf("dupliquer");
    let (mut app, chemin) = avec_un_document_ouvert(&bac);
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("t2", 0.0, 50.0, "le dernier"));
    app.dupliquer_le_document(&chemin);
    let copie = bac.documents().join("Canevas 1 copie.glucose");
    assert!(porte(&copie, "t2"), "la copie porte le dernier geste");
    assert_eq!(
        app.project_path.as_deref(),
        Some(chemin.as_path()),
        "on reste sur l'original"
    );
}

/// **Supprimer** : le fichier s'en va ; le document ouvert est d'abord remplacé.
#[test]
fn test_documents_2_supprimer() {
    let bac = Bac::neuf("supprimer");
    let (mut app, chemin) = avec_un_document_ouvert(&bac);
    app.supprimer_le_document(&chemin);
    assert!(!chemin.exists(), "supprimé");
    assert!(
        app.project_path.is_none(),
        "un document vierge l'a remplacé"
    );
    assert!(
        matches!(app.ui.question, Some((_, Suite::Ouvrir(_)))),
        "la liste revient"
    );
}

/// **Un appui long sur un document de la liste** demande ce qu'on en fait — sans l'ouvrir.
#[test]
fn test_documents_2_l_appui_long_sur_la_liste_demande_quoi_faire() {
    let bac = Bac::neuf("appui");
    let (mut app, chemin) = avec_un_document_ouvert(&bac);
    app.adopter_un_document_vierge();
    app.choisir_un_document();
    let (question, _) = app.ui.question.clone().expect("la liste");
    let (w, h) = app.taille_de_la_fenetre();
    let placee = crate::ui::question::placer(
        &question,
        &app.renderer.typography,
        (w, h),
        app.ui.scale_factor,
    );
    let ((x, y, bw, bh), _, _) = placee
        .boutons
        .iter()
        .find(|(_, _, r)| *r == Reponse::Choix(0))
        .expect("Canevas 1")
        .clone();
    let ici = winit::dpi::PhysicalPosition::new(f64::from(x + bw / 2.0), f64::from(y + bh / 2.0));
    let doigt = |phase| winit::event::Touch {
        device_id: winit::event::DeviceId::dummy(),
        phase,
        location: ici,
        force: None,
        id: 1,
    };
    app.toucher(&doigt(winit::event::TouchPhase::Started));
    let maintenant = std::time::Instant::now();
    if let Some(appui) = &mut app.toucher.appui {
        appui.echeance = maintenant;
    }
    app.attente_de_l_appui();
    app.toucher(&doigt(winit::event::TouchPhase::Ended));
    let Some((question, Suite::Gerer(vise))) = &app.ui.question else {
        panic!("la question de l'appui long est posée");
    };
    assert_eq!(vise, &chemin);
    assert_eq!(question.titre, "« Canevas 1 »");
    assert!(app.project_path.is_none(), "rien ne s'est ouvert");
}

/// Touche la réponse `r` de la question posée, comme un doigt : appui puis relâchement dessus.
fn repondre(app: &mut GlucoseApp, r: Reponse) {
    let (question, _) = app.ui.question.clone().expect("une question");
    let (w, h) = app.taille_de_la_fenetre();
    let placee = crate::ui::question::placer(
        &question,
        &app.renderer.typography,
        (w, h),
        app.ui.scale_factor,
    );
    let ((x, y, bw, bh), _, _) = placee
        .boutons
        .iter()
        .find(|(_, _, reponse)| *reponse == r)
        .expect("la réponse")
        .clone();
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(
        f64::from(x + bw / 2.0),
        f64::from(y + bh / 2.0),
    ));
    app.handle_mouse_down(winit::event::MouseButton::Left, w, h);
    app.handle_mouse_up(winit::event::MouseButton::Left);
}

/// **Supprimer demande « Supprimer », et rien d'autre** : « Annuler » ne touche à rien et rend
/// la liste ; seul le oui efface.
#[test]
fn test_documents_2_seul_le_oui_supprime() {
    let bac = Bac::neuf("annuler");
    let (mut app, chemin) = avec_un_document_ouvert(&bac);
    app.adopter_un_document_vierge();
    app.gerer_un_document(chemin.clone());
    repondre(&mut app, Reponse::Choix(2));
    assert!(matches!(app.ui.question, Some((_, Suite::Supprimer(_)))));
    repondre(&mut app, Reponse::Annuler);
    assert!(chemin.exists(), "« Annuler » ne supprime rien");
    assert!(matches!(app.ui.question, Some((_, Suite::Ouvrir(_)))));
    app.gerer_un_document(chemin.clone());
    repondre(&mut app, Reponse::Choix(2));
    repondre(&mut app, Reponse::Oui);
    assert!(!chemin.exists(), "« Supprimer » supprime");
}

/// **Le nom écrit dans le champ est celui que prend le document**, de la question de l'appui
/// long jusqu'au fichier ; il s'ouvre tout sélectionné.
#[test]
fn test_documents_2_le_nom_du_champ_va_jusqu_au_fichier() {
    let bac = Bac::neuf("champ");
    let (mut app, chemin) = avec_un_document_ouvert(&bac);
    app.adopter_un_document_vierge();
    app.gerer_un_document(chemin.clone());
    repondre(&mut app, Reponse::Choix(0));
    let Some((question, Suite::Renommer(_))) = &mut app.ui.question else {
        panic!("le nom se demande");
    };
    let champ = question.champ.as_mut().expect("un champ");
    assert_eq!(champ.texte, "Canevas 1");
    assert_eq!(
        champ.selection,
        Selection::all("Canevas 1"),
        "tout sélectionné"
    );
    champ.texte = "Neuf".into();
    repondre(&mut app, Reponse::Oui);
    assert!(bac.documents().join("Neuf.glucose").exists() && !chemin.exists());
}

/// **`renommer_sans_ecraser` n'écrase jamais**, même appelé sans vérifier avant.
#[test]
fn test_documents_2_renommer_n_ecrase_jamais() {
    let bac = Bac::neuf("ecraser");
    let (a, b) = (bac.0.join("a.glucose"), bac.0.join("b.glucose"));
    std::fs::write(&a, b"a").expect("écrire");
    std::fs::write(&b, b"b").expect("écrire");
    assert!(crate::persist::atomic::renommer_sans_ecraser(&a, &b).is_err());
    assert_eq!(std::fs::read(&b).expect("lire"), b"b", "intact");
    assert!(a.exists(), "et la source reste");
}

/// **Dupliquer attend le scribe** : retenu pendant que le dernier geste attend dans sa file, il
/// est relâché un peu plus tard — la copie, partie avant, l'aurait manqué.
#[test]
fn test_documents_2_dupliquer_attend_le_scribe() {
    let bac = Bac::neuf("course");
    let (mut app, chemin) = avec_un_document_ouvert(&bac);
    let relache = app
        .disque
        .ecriture
        .as_ref()
        .expect("une écriture")
        .retenir();
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("t3", 0.0, 90.0, "dans la file"));
    let lacher = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let _ = relache.send(());
    });
    app.dupliquer_le_document(&chemin);
    let _ = lacher.join();
    let copie = bac.documents().join("Canevas 1 copie.glucose");
    assert!(porte(&copie, "t3"), "la copie a attendu le scribe");
}
