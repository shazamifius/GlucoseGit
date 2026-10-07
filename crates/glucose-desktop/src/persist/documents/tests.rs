//! Ce que garantissent les documents du téléphone (DOCUMENTS-1).

use super::*;
use glucose_core::types::Annotation;

/// Un dossier à soi, effacé à la fin.
struct Bac(PathBuf);

impl Bac {
    fn neuf(nom: &str) -> Self {
        let d =
            std::env::temp_dir().join(format!("glucose-documents-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("un dossier d'épreuve");
        Self(d)
    }
}

impl Drop for Bac {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// **Le premier nom libre** : « Canevas 1 », puis le suivant que personne ne porte.
#[test]
fn test_le_premier_nom_libre() {
    let bac = Bac::neuf("noms");
    assert_eq!(nom_libre(&bac.0), bac.0.join("Canevas 1.glucose"));
    std::fs::write(bac.0.join("Canevas 1.glucose"), b"x").expect("écrire");
    std::fs::write(bac.0.join("Canevas 3.glucose"), b"x").expect("écrire");
    assert_eq!(nom_libre(&bac.0), bac.0.join("Canevas 2.glucose"));
}

/// Glucose au téléphone, habitant ce dossier, avec un texte posé et jamais enregistré.
fn telephone(bac: &Bac) -> GlucoseApp {
    let mut app = GlucoseApp::new();
    app.habiter(&bac.0);
    app.ui.questions_dessinees = true;
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("t1", 0.0, 0.0, "à garder"));
    app.consigner();
    assert!(app.is_dirty(), "un travail sans nom, jamais enregistré");
    app
}

/// **Au téléphone, « Nouveau document » ne demande rien et ne perd rien** : le travail sans
/// nom se range sous « Canevas 1 », puis le document vierge le remplace. Avant, la question
/// répondait « annuler » et rien ne se passait.
#[test]
fn test_au_telephone_le_travail_sans_nom_se_range_sous_un_nom_libre() {
    let bac = Bac::neuf("ranger");
    let mut app = telephone(&bac);
    app.nouveau_document();
    let range = bac.0.join("documents").join("Canevas 1.glucose");
    assert!(range.exists(), "le travail est rangé, sous un nom libre");
    assert!(app.ui.question.is_none(), "rien ne s'est demandé");
    assert!(
        app.project_path.is_none(),
        "le document courant est le vierge"
    );
    let board = app.store.active_board().expect("un tableau");
    assert!(board.annotations.is_empty(), "et il est vierge");

    // Rouvert, le document rangé porte le texte.
    app.open_from(range);
    let board = app.store.active_board().expect("un tableau");
    assert!(
        board.annotations.iter().any(|a| a.id() == "t1"),
        "le texte est dans le document rangé"
    );
}

/// **Un document propre, lui, pose la question de NOUVEAU-1** — dessinée au téléphone ; et
/// « Annuler » ne change rien.
#[test]
fn test_un_document_propre_demande_avant_d_en_creer_un() {
    let bac = Bac::neuf("question");
    let mut app = GlucoseApp::new();
    app.habiter(&bac.0);
    app.ui.questions_dessinees = true;
    app.nouveau_document();
    assert!(
        matches!(
            app.ui.question,
            Some((_, crate::ui::question::Suite::NouveauDocument))
        ),
        "la question est posée"
    );
}

/// **La liste du téléphone** : les documents rangés, du plus récent au plus ancien ; un
/// toucher sur un nom range le travail en cours et ouvre celui-là — rien ne se perd en route.
#[test]
fn test_la_liste_du_telephone_rouvre_un_document_range() {
    use crate::ui::question::{placer, Reponse, Suite};
    let bac = Bac::neuf("liste");
    let mut app = telephone(&bac);
    app.nouveau_document();
    // Un second travail, sans nom lui aussi.
    let board = app.store.project.active_board_id.clone();
    app.store
        .add_annotation(&board, Annotation::text("t2", 0.0, 0.0, "le second"));
    app.consigner();

    app.choisir_un_document();
    let Some((question, Suite::Ouvrir(chemins))) = app.ui.question.clone() else {
        panic!("la liste est posée");
    };
    assert_eq!(
        chemins,
        vec![bac.0.join("documents").join("Canevas 1.glucose")]
    );
    assert_eq!(question.choix[0].0, "Canevas 1", "nommé sans son extension");

    let (w, h) = app.taille_de_la_fenetre();
    let placee = placer(
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
    app.handle_cursor_moved(winit::dpi::PhysicalPosition::new(
        f64::from(x + bw / 2.0),
        f64::from(y + bh / 2.0),
    ));
    app.handle_mouse_down(winit::event::MouseButton::Left, w, h);
    app.handle_mouse_up(winit::event::MouseButton::Left);

    assert!(app.ui.question.is_none(), "répondue, la liste s'en va");
    let ouvert = app.store.active_board().expect("un tableau");
    assert!(
        ouvert.annotations.iter().any(|a| a.id() == "t1"),
        "Canevas 1 est ouvert"
    );
    assert!(
        bac.0.join("documents").join("Canevas 2.glucose").exists(),
        "et le second travail s'est rangé avant"
    );
}

/// **Du plus récemment touché au plus ancien**, et rien d'autre que des documents.
#[test]
fn test_les_documents_du_plus_recent_au_plus_ancien() {
    let bac = Bac::neuf("ordre");
    let poser = |nom: &str, il_y_a: u64| {
        let chemin = bac.0.join(nom);
        let f = std::fs::File::create(&chemin).expect("créer");
        let quand = std::time::SystemTime::now() - std::time::Duration::from_secs(il_y_a);
        f.set_modified(quand).expect("dater");
        chemin
    };
    let vieux = poser("Vieux.glucose", 3_600);
    let recent = poser("Récent.glucose", 10);
    let moyen = poser("Moyen.glucose", 600);
    poser("note.txt", 1);
    assert_eq!(les_documents(&bac.0), vec![recent, moyen, vieux]);
}
