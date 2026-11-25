use super::*;
use crate::miroir_disque::source_en_attente;

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

#[test]
fn test_un_miroir_se_remplit_en_un_geste_et_se_dit_lu() {
    let (mut store, id) = entre_dans_un_miroir();
    let (parent, dossier, source) = store.miroir_a_lire().expect("il attend sa lecture");
    assert_eq!((parent.as_str(), dossier.as_str()), ("main", id.as_str()));
    assert_eq!(source.root_path, "/d");
    let avant = store.undo_depth();

    let niveau = un_niveau(&mut store);
    assert!(store
        .remplir_le_miroir("main", &id, niveau, 16.0 / 9.0, 1_700_000_000)
        .is_some());
    assert_eq!(contenu(&store), (1, 6, 2));
    assert_eq!(store.undo_depth(), avant + 1, "un seul geste");
    assert!(store.miroir_a_lire().is_none(), "lu, il n'attend plus");
    let lu = store.project.boards[0].folders[0]
        .mirror_source
        .clone()
        .expect("sa source");
    assert_eq!(lu.last_scanned_at, 1_700_000_000);
    // Le sous-dossier a son tableau, et attendra qu'on y entre.
    let sous = &store.active_board().expect("ici").folders[0];
    assert!(store
        .project
        .boards
        .iter()
        .any(|b| b.id == sous.child_board_id));
    assert!(sous.mirror_source.as_ref().is_some_and(|s| s.pending_scan));

    // Un Ctrl+Z vide le dossier, qui redevient à lire ; et rien ne se remplit deux fois.
    assert!(store.undo());
    assert_eq!(contenu(&store), (0, 0, 0));
    assert!(store.miroir_a_lire().is_some());
    assert!(store.redo());
    assert_eq!(contenu(&store), (1, 6, 2));
    let encore = un_niveau(&mut store);
    assert!(store
        .remplir_le_miroir("main", &id, encore, 1.0, 0)
        .is_none());
    assert_eq!(contenu(&store), (1, 6, 2));
}

#[test]
fn test_un_dossier_ordinaire_ne_se_remplit_pas() {
    let mut store = Store::new("test");
    let id =
        store.create_folder_with_content("main", CanvasFolder::new("f", "à moi", ""), Vec::new());
    store.try_enter_folder(&id).expect("on entre");
    assert!(store.miroir_a_lire().is_none());
    let niveau = un_niveau(&mut store);
    assert!(store
        .remplir_le_miroir("main", &id, niveau, 1.0, 0)
        .is_none());
    assert_eq!(contenu(&store), (0, 0, 0));
}

#[test]
fn test_le_niveau_se_dispose_sans_qu_aucun_noeud_n_en_couvre_un_autre() {
    let (mut store, id) = entre_dans_un_miroir();
    let niveau = un_niveau(&mut store);
    assert!(store
        .remplir_le_miroir("main", &id, niveau, 16.0 / 9.0, 0)
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
fn test_remplir_dans_un_geste_ouvert_n_en_fait_qu_un() {
    let mut store = Store::new("test");
    let avant = store.undo_depth();
    store.begin_live_edit();
    let mut f = CanvasFolder::new("miroir", "Lâché", "");
    f.mirror_source = Some(source_en_attente("/d", None));
    let id = store.create_folder_with_content("main", f, Vec::new());
    let niveau = un_niveau(&mut store);
    assert!(store
        .remplir_le_miroir("main", &id, niveau, 1.0, 0)
        .is_some());
    store.end_live_edit();
    assert_eq!(
        store.undo_depth(),
        avant + 1,
        "le dépôt et sa lecture : un seul Ctrl+Z"
    );
    assert!(store.undo());
    assert!(store.project.boards[0].folders.is_empty());
}
