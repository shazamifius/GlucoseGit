//! **Enregistrer ne vide jamais un fichier avant que le nouveau soit entier** (SAUVER-1).
//!
//! Le premier enregistrement d'un document ouvrait le fichier choisi, le vidait, puis y
//! écrivait : choisir un document existant — « oui, le remplacer » — le détruisait avant que le
//! nouveau soit sur le disque. « Enregistrer sous » renommait sa copie avant de l'y pousser.

use super::tests::{application, dossier, image_suivante, noter, rouvrir};

/// **Un témoin lié en dur** à l'ancien fichier le prouve : il garde l'ancien contenu, parce que
/// le nouveau a pris la place d'un bloc au lieu d'être écrit dedans. Écrit en place, le témoin
/// — le même fichier sous un autre nom — aurait été vidé avec lui.
#[test]
fn test_enregistrer_par_dessus_un_fichier_ne_l_ecrit_jamais_en_place() {
    let d = dossier("sauver-par-dessus");
    let cible = d.join("existant.glucose");
    std::fs::write(&cible, b"le document d'hier").unwrap();
    let temoin = d.join("temoin.bin");
    std::fs::hard_link(&cible, &temoin).expect("un lien dur sur le même volume");

    let mut app = application(&d);
    noter(&mut app, "neuf", "le document d'aujourd'hui");
    app.save_to(cible.clone());
    assert_eq!(
        std::fs::read(&temoin).unwrap(),
        b"le document d'hier",
        "l'ancien fichier n'a jamais été ouvert pour être vidé"
    );
    assert!(app.fermer_le_document());
    assert_eq!(
        rouvrir(&d, &cible).store.project,
        app.store.project,
        "le nouveau est entier à sa place"
    );
}

/// **« Enregistrer sous » vers une place qu'on ne peut pas prendre** — ici un dossier du même
/// nom — échoue sans rien toucher : la place reste ce qu'elle était, aucun voisin ne traîne, et
/// le document continue de s'écrire là où il était.
#[test]
fn test_enregistrer_sous_une_place_impossible_ne_touche_a_rien() {
    let d = dossier("sauver-impossible");
    let chemin = d.join("vrai.glucose");
    let occupe = d.join("occupe.glucose");
    std::fs::create_dir_all(&occupe).unwrap();
    std::fs::write(occupe.join("dedans.txt"), b"ne pas toucher").unwrap();
    let mut app = application(&d);
    app.save_to(chemin.clone());

    app.save_to(occupe.clone());
    assert_eq!(app.project_path.as_deref(), Some(chemin.as_path()));
    assert_eq!(
        std::fs::read(occupe.join("dedans.txt")).unwrap(),
        b"ne pas toucher"
    );
    let voisins: Vec<String> = std::fs::read_dir(&d)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".tmp"))
        .collect();
    assert!(voisins.is_empty(), "aucun voisin ne traîne : {voisins:?}");

    noter(&mut app, "apres", "toujours écrit dans le vrai");
    image_suivante(&mut app);
    assert!(app.fermer_le_document());
    assert_eq!(rouvrir(&d, &chemin).store.project, app.store.project);
}
