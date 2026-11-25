//! Le miroir d'un dossier, sur de vrais fichiers : une photo se reconnaît au décodeur, un dossier
//! illisible à la permission que le système refuse.

use super::*;
use glucose_core::types::{Annotation, AssetRef};
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
        self.png_de(nom, 3, 2)
    }

    fn png_de(&self, nom: &str, largeur: u32, hauteur: u32) -> PathBuf {
        let mut pixmap = tiny_skia::Pixmap::new(largeur, hauteur).expect("pixmap");
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

/// Attend que les lectures en route reviennent de l'atelier — un autre fil —, et les applique.
fn attendre(app: &mut GlucoseApp) {
    let limite = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !app.miroirs.en_lecture.is_empty() && std::time::Instant::now() < limite {
        if let Some(a) = &app.miroirs.atelier {
            if let Ok(lu) = a.recu.recv_timeout(std::time::Duration::from_millis(20)) {
                app.miroirs.en_attente.push(lu);
            }
        }
        app.suivre_les_miroirs();
    }
    assert!(
        app.miroirs.en_lecture.is_empty(),
        "l'atelier n'a rien rendu"
    );
}

/// Lâche ce dossier, entre dans son miroir, et attend sa lecture.
fn lache_et_entre(app: &mut GlucoseApp, dossier: &Path) {
    app.drop_files(&[dossier.to_path_buf()]);
    let id = app.store.active_board().expect("ici").folders[0].id.clone();
    app.store.try_enter_folder(&id).expect("on entre");
    app.lire_le_miroir_ou_l_on_entre();
    attendre(app);
}

/// La fenêtre reprend la main : le miroir se relit.
fn revenir(app: &mut GlucoseApp) {
    app.relire_le_miroir();
    attendre(app);
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

/// Le dossier lâché n'est lu qu'à l'entrée, comme ses sous-dossiers : le dépôt ne lit rien, et un
/// seul `Ctrl+Z` l'emporte.
#[test]
fn test_le_depot_d_un_dossier_se_defait_d_un_seul_geste() {
    let bac = Bac::garni("geste");
    let mut app = app();
    let avant = app.store.undo_depth();
    app.drop_files(std::slice::from_ref(&bac.0));
    assert_eq!(app.store.undo_depth(), avant + 1);
    let f = &app.store.active_board().expect("ici").folders[0];
    assert!(f.mirror_source.as_ref().is_some_and(|s| s.pending_scan));
    assert!(app.miroirs.atelier.is_none(), "rien ne s'est lu");
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
    attendre(&mut app);
    assert_eq!(contenu(&app), (0, 1, 0), "dedans.png");

    // Le Ctrl+Z vide le dossier où l'on est ; il ne se relit pas tant qu'on y reste.
    assert!(app.store.undo());
    app.lire_le_miroir_ou_l_on_entre();
    attendre(&mut app);
    assert_eq!(contenu(&app), (0, 0, 0));
    // Ressortir puis revenir le relit : redevenu « à lire », ce qu'il avait vu ne compte plus.
    app.store.exit_folder();
    app.lire_le_miroir_ou_l_on_entre();
    app.store.try_enter_folder(&sous).expect("on revient");
    app.lire_le_miroir_ou_l_on_entre();
    attendre(&mut app);
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
/// les deux, pour qu'elle ne passe pas parce que rien ne se scelle. La photo du miroir posée sur son
/// canevas devient sienne : elle se scelle ; et une photo d'ailleurs lâchée **dans** le miroir aussi
/// — sans quoi elle ne vivrait que dans un fichier qu'il peut effacer.
#[test]
fn test_une_photo_liee_ne_se_scelle_pas_dans_le_document() {
    use crate::persist::disque::tests::{application, dossier, image_suivante};
    let bac = Bac::garni("scelle");
    let seule = bac.png("seule/seule.png");
    let dedans = bac.png("ailleurs/lachee-dedans.png");
    let d = dossier("disque-1-scelle");
    let mut app = application(&d);
    app.save_to(d.join("miroir.glucose"));
    app.drop_files(std::slice::from_ref(&seule));
    let canevas = app.store.project.active_board_id.clone();
    lache_et_entre(&mut app, &bac.0);
    app.drop_files(std::slice::from_ref(&dedans));
    image_suivante(&mut app);
    let scellee = |app: &GlucoseApp, p: &Path| app.disque.objets.est_scellee(&p.to_string_lossy());
    let liee = bac.0.join("photo.png");
    assert!(app
        .store
        .active_board()
        .expect("ici")
        .images
        .iter()
        .any(|i| i.src.as_deref() == liee.to_str()));
    assert!(scellee(&app, &seule), "la photo lâchée seule se scelle");
    assert!(
        scellee(&app, &dedans),
        "lâchée dans le miroir, elle est sienne"
    );
    assert!(!scellee(&app, &liee), "la photo du miroir reste liée");
    // Ni quand le document scelle tout ce qui attend (à l'ouverture, à l'enregistrement).
    app.sceller_ce_qui_attend();
    image_suivante(&mut app);
    assert!(!scellee(&app, &liee), "ni par sceller_tout");

    let mut copie = app
        .store
        .active_board()
        .expect("ici")
        .images
        .iter()
        .find(|i| i.src.as_deref() == liee.to_str())
        .cloned()
        .expect("la photo du miroir");
    copie.id = "copie-sur-son-canevas".to_string();
    app.store.add_image(&canevas, copie);
    image_suivante(&mut app);
    assert!(scellee(&app, &liee), "sortie du miroir, elle est sienne");
}

/// Où le contenu du tableau regardé paraît, en pixels de la fenêtre : (gauche, haut, droite, bas).
fn contenu_a_l_ecran(app: &GlucoseApp) -> (f64, f64, f64, f64) {
    let tableau = app.store.project.active_board_id.clone();
    let c = app.store.content_bounds(&tableau).expect("un miroir lu");
    let v = app.store.viewport();
    let x = |m: f64| m * v.scale + v.x;
    let y = |m: f64| m * v.scale + v.y;
    (
        x(c.left),
        y(c.top),
        x(c.left + c.width),
        y(c.top + c.height),
    )
}

#[test]
fn test_la_premiere_vue_d_un_miroir_montre_tout_sans_grossir_puis_ne_se_reprend_plus() {
    let grande = {
        let mut p = tiny_skia::Pixmap::new(800, 600).expect("pixmap");
        p.fill(tiny_skia::Color::from_rgba8(30, 90, 200, 255));
        p.encode_png().expect("png")
    };
    let peu = Bac::neuf("vue-peu");
    peu.png("seule.png");
    let beaucoup = Bac::neuf("vue-beaucoup");
    for k in 0..40 {
        beaucoup.ecrit(&format!("p{k:02}.png"), &grande);
    }
    for (bac, grossirait) in [(&peu, true), (&beaucoup, false)] {
        let mut app = app();
        let (l, h) = app.taille_de_la_fenetre();
        let (l, h, bande) = (
            f64::from(l),
            f64::from(h),
            f64::from(app.ui.header_height()),
        );
        lache_et_entre(&mut app, &bac.0);
        let vue = app.store.viewport();
        let (g, haut, d, bas) = contenu_a_l_ecran(&app);
        assert!(
            g >= 0.0 && d <= l && haut >= bande && bas <= h,
            "tout se voit, sous la bande : {:?} dans {l} × {h}",
            (g, haut, d, bas)
        );
        assert!(
            ((g + d) / 2.0 - l / 2.0).abs() < 1.0
                && ((haut + bas) / 2.0 - (bande + h) / 2.0).abs() < 1.0,
            "au centre de ce qui reste sous la bande : {:?}",
            (g, haut, d, bas)
        );
        if grossirait {
            assert_eq!(vue.scale, 1.0, "jamais plus grand que nature");
        } else {
            assert!(vue.scale < 1.0, "40 grandes photos : on recule ({vue:?})");
        }
    }

    // Une vue qu'on s'est faite ne se reprend jamais : ressortir puis revenir ne relit rien.
    let mut app = app();
    lache_et_entre(&mut app, &beaucoup.0);
    let tableau = app.store.project.active_board_id.clone();
    let la_sienne = Viewport {
        x: 12.0,
        y: 34.0,
        scale: 0.5,
    };
    app.store.set_viewport(&tableau, la_sienne);
    app.store.exit_folder();
    app.lire_le_miroir_ou_l_on_entre();
    let dossier = app.store.active_board().expect("ici").folders[0].id.clone();
    app.store.try_enter_folder(&dossier).expect("on revient");
    app.lire_le_miroir_ou_l_on_entre();
    attendre(&mut app);
    assert_eq!(app.store.viewport(), la_sienne);
}

/// Le nœud de ce fichier dans le tableau regardé, et son identifiant.
fn noeud_de(app: &GlucoseApp, chemin: &Path) -> Option<String> {
    let b = app.store.active_board()?;
    let c = chemin.to_str()?;
    let photo = b.images.iter().find(|i| i.src.as_deref() == Some(c));
    let note = b.annotations.iter().find(|a| match a {
        Annotation::Text { source_file, .. } | Annotation::Sticky { source_file, .. } => {
            source_file.as_deref() == Some(c)
        }
        _ => false,
    });
    photo
        .map(|i| i.id.clone())
        .or_else(|| note.map(|a| a.id().to_string()))
}

fn texte_de(app: &GlucoseApp, id: &str) -> String {
    let b = app.store.active_board().expect("ici");
    match b.annotations.iter().find(|a| a.id() == id) {
        Some(Annotation::Text { text, .. }) => text.clone(),
        autre => panic!("pas un texte : {autre:?}"),
    }
}

/// **Vivant** (DISQUE-2) : la fenêtre qui reprend la main relit le miroir — le neuf entre sous ce
/// qu'il a rangé, un texte changé se relit, un fichier renommé garde son nœud, un fichier disparu
/// emporte le sien ; un seul geste, signé Glucose, qui se défait d'un `Ctrl+Z`.
#[test]
fn test_un_miroir_vivant_suit_son_dossier() {
    let bac = Bac::garni("vivant");
    let mut app = app();
    lache_et_entre(&mut app, &bac.0);
    let notes = noeud_de(&app, &bac.0.join("notes.md")).expect("notes.md");
    let photo = noeud_de(&app, &bac.0.join("photo.png")).expect("photo.png");
    let range = app
        .store
        .content_bounds(&app.store.project.active_board_id)
        .expect("lu");
    let avant = app.store.undo_depth();

    // D'une autre taille : deux PNG pareils de la même seconde rendraient le renommage ambigu.
    let neuve = bac.png_de("neuve.png", 5, 4);
    bac.ecrit("notes.md", b"# Mes notes, relues depuis le disque");
    std::fs::rename(bac.0.join("photo.png"), bac.0.join("renommee.png")).expect("renommer");
    std::fs::remove_file(bac.0.join("scene.blend")).expect("effacer");
    revenir(&mut app);

    assert_eq!(app.store.undo_depth(), avant + 1, "un seul geste");
    let id_neuve = noeud_de(&app, &neuve).expect("la neuve est entrée");
    let b = app.store.active_board().expect("ici");
    let posee = b.images.iter().find(|i| i.id == id_neuve).expect("là");
    assert!(posee.y > range.top + range.height, "sous ce qu'il y avait");
    assert!(texte_de(&app, &notes).contains("relues depuis le disque"));
    assert_eq!(
        noeud_de(&app, &bac.0.join("renommee.png")),
        Some(photo),
        "le même nœud"
    );
    assert!(
        noeud_de(&app, &bac.0.join("scene.blend")).is_none(),
        "la tuile est partie"
    );
    assert_eq!(contenu(&app), (1, 2, 1));

    // Rien de neuf : aucun geste.
    revenir(&mut app);
    assert_eq!(app.store.undo_depth(), avant + 1);
    assert!(app.store.undo());
    assert!(noeud_de(&app, &neuve).is_none());
    assert!(noeud_de(&app, &bac.0.join("scene.blend")).is_some());
}

/// Un nœud qu'il retire ne revient pas — tant que son fichier ne change pas ; un texte qu'il a récrit
/// ne se relit plus jamais (un seul sens).
#[test]
fn test_ce_qu_il_retire_ou_recrit_reste_a_lui() {
    let bac = Bac::garni("sa-main");
    let mut app = app();
    lache_et_entre(&mut app, &bac.0);
    let ici = app.store.project.active_board_id.clone();
    let photo = noeud_de(&app, &bac.0.join("photo.png")).expect("photo.png");
    app.store.remove_images(&ici, &[photo.as_str()]);
    let notes = noeud_de(&app, &bac.0.join("notes.md")).expect("notes.md");
    app.store.update_annotation(&ici, &notes, |a| {
        if let Annotation::Text { text, .. } = a {
            *text = "à moi".into();
        }
    });
    bac.ecrit("notes.md", b"# Le fichier a encore change");
    revenir(&mut app);
    assert!(
        noeud_de(&app, &bac.0.join("photo.png")).is_none(),
        "retirée, elle ne revient pas"
    );
    assert_eq!(texte_de(&app, &notes), "à moi");

    // Refaite sous le même nom (un rendu), c'est une autre image : elle revient.
    bac.png_de("photo.png", 9, 9);
    revenir(&mut app);
    assert!(
        noeud_de(&app, &bac.0.join("photo.png")).is_some(),
        "le rendu refait revient"
    );
}

/// **Ce qu'un miroir a vu survit à la mémoire** : écrit dans son fichier, relu par l'atelier quand
/// l'application ne le sait plus (un redémarrage) — un nœud retiré ne revient pas pour autant.
#[test]
fn test_ce_qu_un_miroir_a_vu_se_relit_de_son_fichier() {
    use crate::persist::disque::tests::{application, dossier};
    let bac = Bac::garni("memoire");
    let d = dossier("disque-2-memoire");
    let mut app = application(&d);
    app.disque.miroirs = d.join("miroirs");
    app.save_to(d.join("miroir.glucose"));
    lache_et_entre(&mut app, &bac.0);
    let ici = app.store.project.active_board_id.clone();
    let photo = noeud_de(&app, &bac.0.join("photo.png")).expect("photo.png");
    app.store.remove_images(&ici, &[photo.as_str()]);
    revenir(&mut app);
    assert!(std::fs::read_dir(d.join("miroirs")).is_ok_and(|mut f| f.next().is_some()));

    app.miroirs.vus.clear();
    revenir(&mut app);
    assert!(
        noeud_de(&app, &bac.0.join("photo.png")).is_none(),
        "relu de son fichier"
    );
}

/// **Un rendu refait sous son nom** (une autre résolution) : le même nœud, sa nouvelle forme, et ce
/// que le magasin savait de l'ancienne image oublié — elle se redécodera.
#[test]
fn test_un_rendu_refait_sous_son_nom_se_redecode() {
    let bac = Bac::garni("refait");
    let mut app = app();
    lache_et_entre(&mut app, &bac.0);
    let chemin = bac.0.join("photo.png");
    let photo = noeud_de(&app, &chemin).expect("photo.png");
    let src = chemin.to_string_lossy().into_owned();
    app.renderer.magasin.echecs.insert(src.clone());
    bac.png_de("photo.png", 9, 9);
    revenir(&mut app);
    let ici = app.store.active_board_id().to_string();
    let img = app.store.image(&ici, &photo).expect("le même nœud");
    assert_eq!((img.original_width, img.original_height), (9.0, 9.0));
    assert_eq!(img.height, img.width, "sa forme a suivi");
    assert!(
        !app.renderer.magasin.echecs.contains(&src),
        "l'ancienne image s'oublie"
    );
}

/// **Un fichier sans fin ne se lit pas en entier** : `/dev/zero`, le témoin parfait — sans la borne,
/// la lecture ne rendrait jamais la main.
#[cfg(unix)]
#[test]
fn test_un_texte_ne_se_lit_qu_en_tete() {
    let (envoi, recu) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = envoi.send(atelier::markdown_du_fichier(
            Path::new("/dev/zero"),
            "zero.txt",
        ));
    });
    let markdown = recu
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("la lecture rend la main")
        .expect("lisible");
    assert!(markdown.contains("tronqué"));
    assert!(markdown.len() < 101_000, "{}", markdown.len());
}
