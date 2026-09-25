//! **BOARDS-1 — les onglets d'un document** : lesquels se montrent, comment ils se rangent, et
//! ce qu'emporte leur suppression.
//!
//! La règle qu'éprouvent ces épreuves : un tableau vit tant qu'on peut l'atteindre depuis un
//! onglet en suivant les dossiers. Supprimer un onglet ou un dossier retire exactement ce
//! qu'on n'atteint plus — un dossier imbriqué part avec son parent, un contenu qu'un miroir
//! tient encore reste —, et un seul `Ctrl+Z` rend le tout.

use glucose_core::store::Store;
use glucose_core::types::{CanvasFolder, Project};

/// Un dossier neuf dans ce tableau ; rend l'identifiant du tableau qu'il contient.
fn dossier(store: &mut Store, dans: &str, nom: &str) -> (String, String) {
    let id = format!("dossier-{nom}");
    store.create_folder(dans, CanvasFolder::new(id.clone(), nom, ""));
    let enfant = store
        .project
        .boards
        .iter()
        .find(|b| b.id == dans)
        .and_then(|b| b.folders.iter().find(|f| f.id == id))
        .map(|f| f.child_board_id.clone())
        .expect("le dossier est posé");
    (id, enfant)
}

fn onglets(store: &Store) -> Vec<(String, bool)> {
    store
        .onglets()
        .map(|(id, _, actif)| (id.to_string(), actif))
        .collect()
}

fn existe(p: &Project, id: &str) -> bool {
    p.boards.iter().any(|b| b.id == id)
}

/// **Seuls les tableaux racines sont des onglets**, et l'onglet d'un dossier ouvert reste
/// allumé : on est dedans.
#[test]
fn test_seuls_les_tableaux_racines_sont_des_onglets() {
    let mut store = Store::new("onglets");
    let a = store.project.active_board_id.clone();
    let b = store.add_board("B");
    let (dossier_a, dans_a) = dossier(&mut store, &a, "A1");
    let (_, dans_dans_a) = dossier(&mut store, &dans_a, "A2");
    store.set_active_board_id(a.clone());
    assert_eq!(onglets(&store), vec![(a.clone(), true), (b.clone(), false)]);
    store.try_enter_folder(&dossier_a).expect("on entre");
    assert_eq!(store.project.active_board_id, dans_a);
    assert_eq!(
        onglets(&store),
        vec![(a.clone(), true), (b, false)],
        "le contenu du dossier n'est pas un onglet, et celui de A reste allumé"
    );
    assert_eq!(store.racine_de(&dans_dans_a), a);
}

/// **Ranger un onglet ne change que l'ordre** — le geste pèse l'ordre, pas le tableau — et
/// `Ctrl+Z` le remet.
#[test]
fn test_ranger_un_onglet_ne_deplace_que_lui() {
    let mut store = Store::new("ranger");
    let a = store.project.active_board_id.clone();
    let b = store.add_board("B");
    let (_, dans_b) = dossier(&mut store, &b, "B1");
    let c = store.add_board("C");
    let ordre =
        |s: &Store| -> Vec<String> { s.project.boards.iter().map(|x| x.id.clone()).collect() };
    let avant = ordre(&store);
    assert_eq!(avant, vec![a.clone(), b.clone(), dans_b.clone(), c.clone()]);

    store.journal.prendre_les_ecrits();
    store.try_move_board(&c, Some(&a)).expect("C devant A");
    assert_eq!(
        ordre(&store),
        vec![c.clone(), a.clone(), b.clone(), dans_b.clone()]
    );
    let geste = store.journal.prendre_les_ecrits();
    assert_eq!(geste.len(), 1, "un seul geste");
    assert!(
        geste[0].edits.iter().map(|e| e.weight()).sum::<usize>() < 200,
        "le geste pèse l'ordre, pas le contenu des tableaux"
    );

    store.try_move_board(&c, None).expect("C au bout");
    assert_eq!(ordre(&store), avant, "au bout, chacun reprend sa place");
    assert!(store.undo());
    assert_eq!(ordre(&store)[0], c, "Ctrl+Z remet C devant");
    assert!(store.undo());
    assert_eq!(ordre(&store), avant);

    let rien = store.version;
    store.try_move_board(&a, Some(&b)).expect("déjà là");
    assert_eq!(
        store.version, rien,
        "ranger à sa propre place n'est pas un geste"
    );
    assert!(
        store.try_move_board(&dans_b, None).is_err(),
        "le contenu d'un dossier ne se range pas ici"
    );
}

/// **Supprimer un onglet emporte ses dossiers, imbriqués compris**, dans un seul geste qu'un
/// `Ctrl+Z` rend en entier.
#[test]
fn test_supprimer_un_onglet_emporte_ses_dossiers_imbriques() {
    let mut store = Store::new("supprimer");
    let a = store.project.active_board_id.clone();
    let b = store.add_board("B");
    let (_, dans_b) = dossier(&mut store, &b, "B1");
    let (_, dans_dans_b) = dossier(&mut store, &dans_b, "B2");
    store.set_active_board_id(dans_dans_b.clone());
    let avant = store.project.clone();

    store.try_remove_board(&b).expect("B se supprime");
    for parti in [&b, &dans_b, &dans_dans_b] {
        assert!(!existe(&store.project, parti), "{parti} est parti");
    }
    assert_eq!(
        store.project.active_board_id, a,
        "on était dedans : on passe à l'onglet de gauche"
    );
    assert!(store.undo(), "un seul geste");
    assert!(store.project == avant, "Ctrl+Z rend tout, à l'identique");
}

/// **Un miroir garde en vie le contenu qu'il partage** : supprimer l'un ou l'autre des deux
/// dossiers laisse le contenu ; il ne part qu'avec le dernier.
#[test]
fn test_un_miroir_garde_le_contenu_qu_il_partage() {
    let mut store = Store::new("miroir");
    let a = store.project.active_board_id.clone();
    let (original, contenu) = dossier(&mut store, &a, "Photos");
    let miroir = store
        .try_mirror_folder(&a, &original, 500.0, 0.0)
        .expect("le miroir se pose");

    store.remove_folders(&a, &[&miroir]);
    assert!(
        existe(&store.project, &contenu),
        "supprimer le miroir ne vide pas l'original"
    );
    assert!(store.undo());
    store.remove_folders(&a, &[&original]);
    assert!(
        existe(&store.project, &contenu),
        "le miroir tient encore le contenu"
    );
    store.remove_folders(&a, &[&miroir]);
    assert!(
        !existe(&store.project, &contenu),
        "le dernier parti, le contenu part"
    );
}

/// **Supprimer l'onglet actif passe à celui de gauche** — pas au premier.
#[test]
fn test_supprimer_l_onglet_actif_passe_a_celui_de_gauche() {
    let mut store = Store::new("gauche");
    let _a = store.project.active_board_id.clone();
    let b = store.add_board("B");
    let c = store.add_board("C");
    store.set_active_board_id(c.clone());
    store.try_remove_board(&c).expect("C se supprime");
    assert_eq!(store.project.active_board_id, b);
}

/// **Supprimer le premier onglet actif passe à celui de droite** ; le dernier onglet, et le
/// contenu d'un dossier, se refusent en disant quoi faire.
#[test]
fn test_les_refus_et_le_premier_onglet() {
    let mut store = Store::new("refus");
    let a = store.project.active_board_id.clone();
    let b = store.add_board("B");
    let (_, dans_b) = dossier(&mut store, &b, "B1");
    store.set_active_board_id(a.clone());
    store.try_remove_board(&a).expect("A se supprime");
    assert_eq!(
        store.project.active_board_id, b,
        "pas d'onglet à gauche : celui de droite"
    );
    let erreur = store
        .try_remove_board(&b)
        .expect_err("le dernier onglet reste");
    assert!(erreur.to_string().contains("dernier"), "{erreur}");
    let erreur = store
        .try_remove_board(&dans_b)
        .expect_err("un contenu de dossier");
    assert!(erreur.to_string().contains("dossier"), "{erreur}");
}
