//! Le miroir d'un dossier, sur de vrais fichiers : une photo se reconnaît au décodeur, un dossier
//! illisible à la permission que le système refuse.

use super::*;
use glucose_core::types::Annotation;
use std::path::PathBuf;

/// Un dossier de travail à soi, effacé à la fin de l'épreuve.
struct Bac(PathBuf);

impl Bac {
    fn neuf(nom: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("glucose-miroir-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dossier d'épreuve");
        Self(dir)
    }

    fn ecrit(&self, nom: &str, contenu: &[u8]) -> PathBuf {
        let path = self.0.join(nom);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("sous-dossier");
        std::fs::write(&path, contenu).expect("écriture d'épreuve");
        path
    }

    /// Un PNG minuscule mais vrai : c'est le décodeur qui le reconnaît.
    fn png(&self, nom: &str) -> PathBuf {
        let mut pixmap = tiny_skia::Pixmap::new(3, 2).expect("pixmap");
        pixmap.fill(tiny_skia::Color::from_rgba8(200, 30, 30, 255));
        self.ecrit(nom, &pixmap.encode_png().expect("png"))
    }

    /// Une photo, un texte, un fichier opaque, ce qui se tait, et un sous-dossier qui a sa photo.
    fn garni(nom: &str) -> Self {
        let bac = Self::neuf(nom);
        bac.png("photo.png");
        bac.ecrit("notes.md", b"# Mes notes");
        bac.ecrit("scene.blend", b"BLENDER-v300 opaque");
        bac.ecrit(".env", b"MOT_DE_PASSE=secret");
        bac.ecrit("Thumbs.db", b"miettes");
        bac.png("sous/dedans.png");
        bac
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
        b.folders.clear();
    }
    app.store.journal.clear();
    app
}

/// Lâche ce dossier, et entre dans son miroir.
fn lache_et_entre(app: &mut GlucoseApp, dossier: &Path) {
    app.drop_files(&[dossier.to_path_buf()]);
    let id = app.store.active_board().expect("ici").folders[0].id.clone();
    app.store.try_enter_folder(&id).expect("on entre");
    app.lire_le_miroir_ou_l_on_entre();
}

fn contenu(app: &GlucoseApp) -> (usize, usize, usize) {
    let b = app.store.active_board().expect("le tableau regardé");
    (b.folders.len(), b.images.len(), b.annotations.len())
}

#[test]
fn test_un_dossier_lache_devient_son_miroir_lu_d_un_niveau() {
    let bac = Bac::garni("lache");
    let mut app = app();
    lache_et_entre(&mut app, &bac.0);
    assert_eq!(
        contenu(&app),
        (1, 1, 2),
        "sous/, photo.png, notes.md et scene.blend ; rien de caché"
    );
    let b = app.store.active_board().expect("ici");
    let photo = &b.images[0];
    assert!(
        photo.asset.as_ref().is_some_and(AssetRef::is_link),
        "liée, jamais scellée"
    );
    assert_eq!(photo.src.as_deref(), bac.0.join("photo.png").to_str());
    assert_eq!((photo.original_width, photo.original_height), (3.0, 2.0));
    let tuile = b
        .annotations
        .iter()
        .find(|a| matches!(a, Annotation::Sticky { .. }));
    let Some(Annotation::Sticky { source_file, .. }) = tuile else {
        panic!("scene.blend en tuile : {:?}", b.annotations);
    };
    assert_eq!(source_file.as_deref(), bac.0.join("scene.blend").to_str());
    let sous = &b.folders[0];
    assert_eq!(sous.name, "sous");
    assert!(
        sous.mirror_source.as_ref().is_some_and(|s| s.pending_scan),
        "lu à l'entrée seulement"
    );
}

#[test]
fn test_le_depot_d_un_dossier_se_defait_d_un_seul_geste() {
    let bac = Bac::garni("geste");
    let mut app = app();
    let avant = app.store.undo_depth();
    app.drop_files(std::slice::from_ref(&bac.0));
    assert_eq!(app.store.undo_depth(), avant + 1);
    assert!(app.store.undo());
    assert!(app.store.active_board().expect("ici").folders.is_empty());
}

#[test]
fn test_un_sous_dossier_se_lit_en_y_entrant_et_un_ctrl_z_ne_s_y_enferme_pas() {
    let bac = Bac::garni("entrer");
    let mut app = app();
    lache_et_entre(&mut app, &bac.0);
    let sous = app.store.active_board().expect("ici").folders[0].id.clone();
    app.store
        .try_enter_folder(&sous)
        .expect("on entre dans sous/");
    app.lire_le_miroir_ou_l_on_entre();
    assert_eq!(contenu(&app), (0, 1, 0), "dedans.png");

    // Le Ctrl+Z vide le dossier où l'on est ; il ne se relit pas tant qu'on y reste.
    assert!(app.store.undo());
    app.lire_le_miroir_ou_l_on_entre();
    assert_eq!(contenu(&app), (0, 0, 0));
    // Ressortir puis revenir le relit.
    app.store.exit_folder();
    app.lire_le_miroir_ou_l_on_entre();
    app.store.try_enter_folder(&sous).expect("on revient");
    app.lire_le_miroir_ou_l_on_entre();
    assert_eq!(contenu(&app), (0, 1, 0));
}

#[cfg(unix)]
#[test]
fn test_un_dossier_illisible_reste_une_tuile_qui_y_mene() {
    use std::os::unix::fs::PermissionsExt;
    let bac = Bac::neuf("illisible");
    let ferme = bac.0.join("ferme");
    std::fs::create_dir_all(&ferme).expect("dossier");
    std::fs::set_permissions(&ferme, std::fs::Permissions::from_mode(0o000)).expect("fermé");
    if std::fs::read_dir(&ferme).is_ok() {
        return; // Le super-utilisateur lit tout : l'épreuve n'a rien à dire ici.
    }
    let mut app = app();
    app.drop_files(std::slice::from_ref(&ferme));
    std::fs::set_permissions(&ferme, std::fs::Permissions::from_mode(0o755)).expect("rouvert");
    let b = app.store.active_board().expect("ici");
    assert!(b.folders.is_empty());
    assert!(matches!(
        b.annotations.as_slice(),
        [Annotation::Sticky { .. }]
    ));
}

/// **Une photo du miroir reste sur son disque** ; une photo lâchée seule se scelle — l'épreuve dit
/// les deux, pour qu'elle ne passe pas parce que rien ne se scelle. Et la photo du miroir posée sur
/// son canevas devient sienne : elle se scelle.
#[test]
fn test_une_photo_liee_ne_se_scelle_pas_dans_le_document() {
    use crate::persist::disque::tests::{application, dossier, image_suivante};
    let bac = Bac::garni("scelle");
    let seule = bac.png("seule/seule.png");
    let d = dossier("disque-1-scelle");
    let mut app = application(&d);
    app.save_to(d.join("miroir.glucose"));
    app.drop_files(&[bac.0.clone(), seule.clone()]);
    image_suivante(&mut app);
    let liee = bac.0.join("photo.png").to_string_lossy().into_owned();
    let posee = seule.to_string_lossy().into_owned();
    assert!(app
        .store
        .project
        .boards
        .iter()
        .any(|b| b.images.iter().any(|i| i.src.as_deref() == Some(&liee))));
    assert!(
        app.disque.objets.est_scellee(&posee),
        "la photo lâchée seule se scelle"
    );
    assert!(
        !app.disque.objets.est_scellee(&liee),
        "la photo du miroir reste liée"
    );
    // Ni quand le document scelle tout ce qui attend (à l'ouverture, à l'enregistrement).
    app.sceller_ce_qui_attend();
    image_suivante(&mut app);
    assert!(!app.disque.objets.est_scellee(&liee), "ni par sceller_tout");

    let mut copie = app
        .store
        .project
        .boards
        .iter()
        .flat_map(|b| &b.images)
        .find(|i| i.src.as_deref() == Some(&liee))
        .cloned()
        .expect("la photo du miroir");
    copie.id = "copie-sur-son-canevas".to_string();
    let ici = app.store.project.active_board_id.clone();
    app.store.add_image(&ici, copie);
    image_suivante(&mut app);
    assert!(
        app.disque.objets.est_scellee(&liee),
        "sortie du miroir, elle est sienne"
    );
}
