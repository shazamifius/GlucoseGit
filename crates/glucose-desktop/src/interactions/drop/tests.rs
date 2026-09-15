//! Le routage d'un fichier déposé : trois issues, un lot, une entrée d'annulation.
//!
//! Tout se joue sur de vrais fichiers, écrits dans un dossier temporaire : le classement
//! d'une image passe par le **décodeur**, et un test qui l'aurait simulé n'aurait rien
//! prouvé — c'est justement la divergence entre une liste d'extensions et le décodeur que
//! ce module supprime.

use super::*;
use glucose_core::types::Annotation;
use std::path::PathBuf;

/// Un dossier de travail à soi, effacé à la fin du test.
struct Bac(PathBuf);

impl Bac {
    fn neuf(nom: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("glucose-drop-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier de test");
        Self(dir)
    }

    fn ecrit(&self, nom: &str, contenu: &[u8]) -> PathBuf {
        let path = self.0.join(nom);
        std::fs::write(&path, contenu).expect("écriture de test");
        path
    }

    /// Un PNG minuscule mais **vrai** : c'est le décodeur qui doit le reconnaître.
    fn png(&self, nom: &str) -> PathBuf {
        let mut pixmap = tiny_skia::Pixmap::new(3, 2).expect("pixmap");
        pixmap.fill(tiny_skia::Color::from_rgba8(200, 30, 30, 255));
        self.ecrit(nom, &pixmap.encode_png().expect("png"))
    }

    fn sous_dossier(&self, nom: &str) -> PathBuf {
        let path = self.0.join(nom);
        std::fs::create_dir_all(&path).expect("sous-dossier de test");
        path
    }
}

impl Drop for Bac {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn app() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    app.store.journal.clear();
    app
}

fn annotations(app: &GlucoseApp) -> Vec<Annotation> {
    app.store
        .active_board()
        .map(|b| b.annotations.clone())
        .unwrap_or_default()
}

// ── Les trois issues ──────────────────────────────────────────────────────────

/// Un fichier de code devient une carte, avec son contenu dans un bloc marqué.
///
/// C'est le défaut corrigé, dit en une ligne : avant, un `.rs` déposé ne produisait qu'un
/// message d'erreur de décodeur d'images.
#[test]
fn test_a_source_file_becomes_a_card_carrying_its_content() {
    let bac = Bac::neuf("source");
    let path = bac.ecrit("module.rs", b"fn main() { let x = 1; }");
    let mut app = app();
    app.drop_files(std::slice::from_ref(&path));

    let posees = annotations(&app);
    assert_eq!(posees.len(), 1, "une carte, et une seule");
    let Annotation::Text { text, width, .. } = &posees[0] else {
        panic!("un fichier lisible doit donner une carte de texte, pas {posees:?}");
    };
    assert!(text.contains("module.rs"), "le titre porte le nom");
    assert!(
        text.contains("fn main() { let x = 1; }"),
        "le contenu y est"
    );
    assert!(
        text.contains("```rust"),
        "le bloc est marqué de son langage"
    );
    assert_eq!(*width, Some(READABLE_WIDTH));
}

/// Un Markdown se pose tel quel, sans se faire enrober dans un bloc de code.
#[test]
fn test_a_markdown_file_is_not_wrapped_in_a_code_block() {
    let bac = Bac::neuf("markdown");
    let path = bac.ecrit("notes.md", "# Mon titre\n\ndu texte".as_bytes());
    let mut app = app();
    app.drop_files(&[path]);

    let Annotation::Text { text, .. } = &annotations(&app)[0] else {
        panic!("une carte de texte");
    };
    assert!(text.contains("# Mon titre"));
    assert!(
        !text.contains("```"),
        "un Markdown ne s'enrobe pas : {text}"
    );
}

/// Une image se pose comme image — et c'est le **décodeur** qui l'a dit, pas son extension.
#[test]
fn test_an_image_is_recognised_by_its_header_not_its_extension() {
    let bac = Bac::neuf("image");
    // L'extension ment : le fichier s'appelle `.bin`, mais c'est un vrai PNG.
    let menteur = bac.png("photo.bin");
    let mut app = app();
    app.drop_files(&[menteur]);

    let images = app
        .store
        .active_board()
        .map(|b| b.images.len())
        .unwrap_or(0);
    assert_eq!(
        images, 1,
        "le décodeur reconnaît le PNG quelle que soit l'extension"
    );
    assert!(
        annotations(&app).is_empty(),
        "et il ne pose pas aussi un lanceur"
    );
}

/// Un fichier opaque devient une tuile teintée, qui porte le chemin du fichier.
#[test]
fn test_an_opaque_file_becomes_a_tinted_launcher() {
    let bac = Bac::neuf("opaque");
    let path = bac.ecrit("scene.blend", &[0u8, 1, 2, 3]);
    let mut app = app();
    app.drop_files(std::slice::from_ref(&path));

    let posees = annotations(&app);
    let Annotation::Sticky {
        text,
        bg_color,
        source_file,
        width,
        ..
    } = &posees[0]
    else {
        panic!("un fichier opaque doit donner une tuile, pas {posees:?}");
    };
    assert_eq!(text, "scene.blend", "la tuile porte le nom du fichier");
    assert_eq!(
        bg_color.as_deref(),
        Some("#e87d0d"),
        "la teinte de Blender, reprise de Glucose Tauri"
    );
    assert_eq!(source_file.as_deref(), Some(path.to_str().expect("chemin")));
    assert_eq!(*width, Some(LAUNCHER_SIZE.0));
}

/// Une extension inconnue prend la teinte neutre, et reste un lanceur utilisable.
#[test]
fn test_an_unknown_extension_still_gets_a_launcher() {
    let bac = Bac::neuf("inconnu");
    let path = bac.ecrit("chose.zzz", &[9u8; 16]);
    let mut app = app();
    app.drop_files(&[path]);

    let Annotation::Sticky { bg_color, .. } = &annotations(&app)[0] else {
        panic!("une tuile");
    };
    assert_eq!(bg_color.as_deref(), Some("#1a1a2e"));
}

/// Un **dossier** devient un lanceur, et non une erreur : on ne sait pas le lire, on sait
/// y mener.
///
/// Le miroir de dossier est un chantier à part entière (fiche 12, 2.C.3). En attendant,
/// une tuile qui ouvre le dossier dans l'explorateur est vraie et utile ; ne rien faire du
/// tout ne l'était pas.
#[test]
fn test_a_folder_becomes_a_launcher_rather_than_an_error() {
    let bac = Bac::neuf("dossier");
    // Le nom porte une extension lisible, pour vérifier que c'est bien `is_dir` qui tranche
    // et non l'extension : un dossier nommé « travaux.md » n'est pas un Markdown.
    let dossier = bac.sous_dossier("travaux.md");
    let mut app = app();
    app.drop_files(std::slice::from_ref(&dossier));

    let posees = annotations(&app);
    let Annotation::Sticky { source_file, .. } = &posees[0] else {
        panic!("un dossier doit donner une tuile, pas {posees:?}");
    };
    assert_eq!(
        source_file.as_deref(),
        Some(dossier.to_str().expect("chemin"))
    );
}

// ── Le lot ────────────────────────────────────────────────────────────────────

/// Un lot se pose en cascade, et tient dans **une** entrée d'annulation.
///
/// winit émet un `DroppedFile` par fichier ; les accumuler puis les poser ensemble est ce
/// qui fait que huit fichiers déposés d'un geste se défont d'un seul `Ctrl+Z`, et non de
/// huit — et qu'ils ne se superposent pas tous au même point.
#[test]
fn test_a_batch_cascades_and_undoes_in_one_step() {
    let bac = Bac::neuf("lot");
    let lot = vec![
        bac.ecrit("un.md", b"premier"),
        bac.ecrit("deux.md", b"second"),
        bac.ecrit("trois.blend", &[0u8; 4]),
    ];
    let mut app = app();
    app.drop_files(&lot);

    let posees = annotations(&app);
    assert_eq!(posees.len(), 3);
    let points: Vec<(f64, f64)> = posees.iter().map(|a| (a.x(), a.y())).collect();
    for paire in points.windows(2) {
        assert!(
            (paire[1].0 - paire[0].0 - CASCADE).abs() < 1e-9,
            "la cascade n'avance pas de {CASCADE} : {points:?}"
        );
    }

    app.store.undo();
    assert!(
        annotations(&app).is_empty(),
        "un seul Ctrl+Z doit défaire le lot entier"
    );
}

/// Un lot vide ne fait rien — et surtout pas une entrée d'annulation ni un toast.
#[test]
fn test_an_empty_batch_does_nothing_at_all() {
    let mut app = app();
    let version = app.store.version;
    let toast = app.ui.current_toast.as_ref().map(|t| t.message.clone());
    app.drop_files(&[]);
    assert_eq!(app.store.version, version, "aucune entree d'annulation");
    assert_eq!(
        app.ui.current_toast.as_ref().map(|t| t.message.clone()),
        toast,
        "et rien a dire : un lot vide n'est pas un echec"
    );
}

/// Un fichier illisible ne fait pas échouer le lot : il devient un lanceur.
#[test]
fn test_a_file_that_cannot_be_read_still_lands() {
    let bac = Bac::neuf("absent");
    let present = bac.ecrit("vrai.md", b"du texte");
    let absent = bac.0.join("jamais-ecrit.md");
    let mut app = app();
    app.drop_files(&[present, absent]);

    assert_eq!(
        annotations(&app).len(),
        1,
        "le fichier présent se pose, l'absent est signalé et passé"
    );
}

// ── Le lanceur, et ce qu'il promet ────────────────────────────────────────────

/// Un lanceur mène à son fichier ; une carte de texte, elle, s'édite.
///
/// Les deux portent un `source_file` — c'est vrai des deux, et c'est utile. La différence
/// est dans le **geste**, pas dans les données : une carte de texte a du texte à éditer,
/// une tuile n'a qu'un nom de fichier.
#[test]
fn test_only_a_tile_leads_to_its_file_a_card_is_edited() {
    let bac = Bac::neuf("geste");
    let mut app = app();
    app.drop_files(&[
        bac.ecrit("lisible.md", b"du texte"),
        bac.ecrit("o.blend", &[0]),
    ]);

    let posees = annotations(&app);
    let carte = posees
        .iter()
        .find(|a| matches!(a, Annotation::Text { .. }))
        .expect("une carte");
    let tuile = posees
        .iter()
        .find(|a| matches!(a, Annotation::Sticky { .. }))
        .expect("une tuile");

    assert!(
        app.store.launcher_target(carte.id()).is_none(),
        "une carte de texte n'est pas un lanceur : un double-clic doit l'ouvrir en édition"
    );
    assert!(
        app.store.launcher_target(tuile.id()).is_some(),
        "la tuile, elle, mène à son fichier"
    );
}

/// La table des teintes est triée et sans doublon — la dichotomie en dépend.
#[test]
fn test_the_accent_table_is_sorted_and_has_no_duplicate() {
    for paire in ACCENT.windows(2) {
        assert!(
            paire[0].0 < paire[1].0,
            "la table des teintes n'est pas triée : {:?} avant {:?}",
            paire[0].0,
            paire[1].0
        );
    }
}

/// Le compte-rendu d'un lot dit ce qui s'est passé, y compris ce qui a raté.
///
/// Un lot ne produit qu'**un** message : huit fichiers déposés ne doivent pas donner huit
/// toasts, et celui qui a raté s'y compte au lieu de s'annoncer seul au milieu des autres.
#[test]
fn test_the_batch_report_counts_both_what_landed_and_what_did_not() {
    assert_eq!(compte_rendu(1, 0), "1 élément posé");
    assert_eq!(compte_rendu(3, 0), "3 éléments posés");
    assert_eq!(compte_rendu(2, 1), "2 éléments posés, 1 illisible");
    assert_eq!(compte_rendu(1, 2), "1 élément posé, 2 illisibles");
    assert_eq!(compte_rendu(0, 4), "Aucun des 4 fichiers n'a pu être posé");
    assert_eq!(compte_rendu(0, 0), "", "un lot vide n'a rien à dire");
}
