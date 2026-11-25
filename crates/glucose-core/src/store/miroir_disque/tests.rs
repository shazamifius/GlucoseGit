use super::*;
use crate::miroir_disque::source_en_attente;
use crate::store::journal::Main;

/// Un tableau « main » qui porte un dossier miroir du disque, pas encore lu, et l'on y est entré.
fn entre_dans_un_miroir() -> (Store, String) {
    let mut store = Store::new("test");
    let mut f = CanvasFolder::new("miroir", "Références", "");
    f.mirror_source = Some(source_en_attente("/d", None));
    let id = store.create_folder_with_content("main", f, Vec::new());
    store.try_enter_folder(&id).expect("on entre");
    (store, id)
}

fn un_niveau(store: &mut Store) -> Niveau {
    let mut sous = CanvasFolder::new(store.generate_id("folder"), "sous", "");
    sous.mirror_source = Some(source_en_attente("/d/sous", None));
    let photos = (0..6)
        .map(|k| BoardImage::new(format!("p{k}"), 0.0, 0.0, 300.0, 200.0 + 40.0 * k as f64))
        .collect();
    Niveau {
        dossiers: vec![sous],
        photos,
        textes: vec![Annotation::text("t", 0.0, 0.0, "# notes")],
        tuiles: vec![Annotation::sticky("s", 0.0, 0.0, "scene.blend")],
    }
}

fn contenu(store: &Store) -> (usize, usize, usize) {
    let b = store.active_board().expect("le tableau du dossier");
    (b.folders.len(), b.images.len(), b.annotations.len())
}

fn remplir(niveau: Niveau) -> Reconciliation {
    Reconciliation {
        neufs: niveau,
        ..Reconciliation::default()
    }
}

#[test]
fn test_un_miroir_se_remplit_en_un_geste_signe_et_se_dit_lu() {
    let (mut store, id) = entre_dans_un_miroir();
    let ouvert = store.miroir_ouvert().expect("on y est");
    assert_eq!(
        (ouvert.parent.as_str(), ouvert.dossier.as_str()),
        ("main", id.as_str())
    );
    assert_eq!(ouvert.source.root_path, "/d");
    assert!(ouvert.source.pending_scan);
    let avant = store.undo_depth();
    let _ = store.journal.prendre_les_ecrits_signes();

    let niveau = remplir(un_niveau(&mut store));
    let r = store.reconcilier_le_miroir("main", &id, niveau, 16.0 / 9.0, 1_700_000_000);
    assert_eq!(
        r,
        Some(true),
        "il a rempli un miroir vide : sa première vue est à poser"
    );
    assert_eq!(contenu(&store), (1, 6, 2));
    assert_eq!(store.undo_depth(), avant + 1, "un seul geste");
    let ecrits = store.journal.prendre_les_ecrits_signes();
    assert_eq!(ecrits.len(), 1);
    assert_eq!(ecrits[0].main, Main::Agent(GLUCOSE.into()), "signé Glucose");
    assert_eq!(store.journal.main, Main::Humain, "la main revient à lui");
    let lu = store.miroir_ouvert().expect("toujours là").source;
    assert!(!lu.pending_scan, "lu, il n'attend plus");
    assert_eq!(lu.last_scanned_at, 1_700_000_000);
    // Le sous-dossier a son tableau, et attendra qu'on y entre.
    let sous = &store.active_board().expect("ici").folders[0];
    assert!(store
        .project
        .boards
        .iter()
        .any(|b| b.id == sous.child_board_id));
    assert!(sous.mirror_source.as_ref().is_some_and(|s| s.pending_scan));

    // Une lecture sans rien de neuf n'écrit rien : pas de geste à chaque retour dans la fenêtre.
    let rien = store.reconcilier_le_miroir("main", &id, Reconciliation::default(), 1.0, 0);
    assert_eq!(rien, None);
    assert_eq!(store.undo_depth(), avant + 1);

    // Un Ctrl+Z vide le dossier, qui redevient à lire.
    assert!(store.undo());
    assert_eq!(contenu(&store), (0, 0, 0));
    assert!(store.miroir_ouvert().expect("là").source.pending_scan);
    assert!(store.redo());
    assert_eq!(contenu(&store), (1, 6, 2));
}

#[test]
fn test_un_dossier_ordinaire_ne_se_remplit_pas() {
    let mut store = Store::new("test");
    let id =
        store.create_folder_with_content("main", CanvasFolder::new("f", "à moi", ""), Vec::new());
    store.try_enter_folder(&id).expect("on entre");
    assert!(store.miroir_ouvert().is_none());
    let niveau = remplir(un_niveau(&mut store));
    assert_eq!(
        store.reconcilier_le_miroir("main", &id, niveau, 1.0, 0),
        None
    );
    assert_eq!(contenu(&store), (0, 0, 0));
}

/// Une seconde lecture : le neuf se pose sous ce qu'il a rangé, sans le déplacer ; ce qui suit son
/// fichier prend sa nouvelle version ; ce qui part part — un seul geste, qui se défait entier.
#[test]
fn test_une_relecture_pose_le_neuf_sous_son_rangement_et_se_defait_entiere() {
    let (mut store, id) = entre_dans_un_miroir();
    let niveau = remplir(un_niveau(&mut store));
    store.reconcilier_le_miroir("main", &id, niveau, 1.0, 0);
    let ici = store.project.active_board_id.clone();
    store.update_image(&ici, "p0", |i| (i.x, i.y) = (-500.0, -400.0));
    let range = store.content_bounds(&ici).expect("du contenu");
    let avant = store.undo_depth();

    let mut recrit = Annotation::text("t", 0.0, 0.0, "# notes, relues");
    recrit.translate(7.0, 8.0);
    let r = Reconciliation {
        neufs: Niveau {
            photos: vec![BoardImage::new("neuve", 0.0, 0.0, 100.0, 50.0)],
            ..Niveau::default()
        },
        suivis: Niveau {
            textes: vec![recrit.clone()],
            ..Niveau::default()
        },
        partis: vec![Noeud::Note("s".into())],
    };
    assert_eq!(
        store.reconcilier_le_miroir("main", &id, r, 1.0, 5),
        Some(false)
    );
    assert_eq!(store.undo_depth(), avant + 1, "un seul geste");
    let b = store.active_board().expect("ici");
    let neuve = b.images.iter().find(|i| i.id == "neuve").expect("posée");
    assert!(
        neuve.y - neuve.height / 2.0 > range.top + range.height,
        "sous son rangement : {neuve:?} / {range:?}"
    );
    let p0 = b.images.iter().find(|i| i.id == "p0").expect("là");
    assert_eq!((p0.x, p0.y), (-500.0, -400.0), "sa place ne bouge pas");
    assert!(b.annotations.contains(&recrit), "le texte suit son fichier");
    assert!(b.annotations.iter().all(|a| a.id() != "s"), "la tuile part");

    assert!(store.undo());
    let b = store.active_board().expect("ici");
    assert_eq!((b.images.len(), b.annotations.len()), (6, 2));
    assert!(b.annotations.iter().any(|a| a.id() == "s"));
    assert!(!b.annotations.contains(&recrit));
}

#[test]
fn test_le_niveau_se_dispose_sans_qu_aucun_noeud_n_en_couvre_un_autre() {
    let (mut store, id) = entre_dans_un_miroir();
    let niveau = remplir(un_niveau(&mut store));
    assert!(store
        .reconcilier_le_miroir("main", &id, niveau, 16.0 / 9.0, 0)
        .is_some());
    let b = store.active_board().expect("ici");
    let mut boites: Vec<Rect> = b
        .folders
        .iter()
        .map(|f| Rect::new(f.x, f.y, f.width, f.height))
        .collect();
    boites.extend(
        b.images
            .iter()
            .map(|i| Rect::new(i.x - i.width / 2.0, i.y - i.height / 2.0, i.width, i.height)),
    );
    boites.extend(b.annotations.iter().filter_map(Annotation::rect));
    assert_eq!(boites.len(), 9);
    for (k, a) in boites.iter().enumerate() {
        assert!(
            a.left >= 0.0 && a.top >= 0.0,
            "tout à droite et sous l'origine : {a:?}"
        );
        for c in &boites[k + 1..] {
            let couvre = a.left < c.left + c.width
                && c.left < a.left + a.width
                && a.top < c.top + c.height
                && c.top < a.top + a.height;
            assert!(!couvre, "{a:?} couvre {c:?}");
        }
    }
}

#[test]
fn test_une_lecture_attend_qu_un_geste_ouvert_se_ferme() {
    let (mut store, id) = entre_dans_un_miroir();
    store.begin_live_edit();
    let niveau = remplir(un_niveau(&mut store));
    assert_eq!(
        store.reconcilier_le_miroir("main", &id, niveau.clone(), 1.0, 0),
        None,
        "elle ne s'y fond pas : un Échap défairait les deux"
    );
    assert_eq!(contenu(&store), (0, 0, 0));
    store.end_live_edit();
    assert_eq!(
        store.reconcilier_le_miroir("main", &id, niveau, 1.0, 0),
        Some(true)
    );
}
