use super::*;
use std::cmp::Ordering::{Greater, Less};

fn entree(nom: &str, sorte: Sorte, taille: u64, date: i64) -> Entree {
    Entree {
        nom: nom.to_string(),
        chemin: format!("/d/{nom}"),
        sorte,
        taille,
        date,
    }
}

fn noms(entrees: &[Entree]) -> Vec<&str> {
    entrees.iter().map(|e| e.nom.as_str()).collect()
}

#[test]
fn test_rien_de_cache_n_entre_dans_un_miroir() {
    for cache in [
        ".env",
        ".gitignore",
        ".DS_Store",
        "Thumbs.db",
        "desktop.ini",
    ] {
        assert!(masquee(cache, false), "{cache} devrait se taire");
    }
    assert!(masquee(".git", true), "un dossier caché se tait aussi");
    assert!(
        masquee("setup.EXE", false),
        "un binaire se tait, en majuscules aussi"
    );
    for montre in ["photo.jpg", "notes.md", "scene.blend", "README"] {
        assert!(!masquee(montre, false), "{montre} devrait se montrer");
    }
    assert!(
        !masquee("outils.exe", true),
        "un dossier n'est jamais un binaire"
    );
}

#[test]
fn test_les_noms_se_rangent_comme_dans_un_explorateur() {
    assert_eq!(comparer_les_noms("photo 2", "photo 10"), Less);
    assert_eq!(comparer_les_noms("x100", "x10"), Greater);
    assert_eq!(comparer_les_noms("Été", "zèbre"), Less, "sans accents");
    assert_eq!(
        comparer_les_noms("abricot", "Banane"),
        Less,
        "sans majuscules"
    );
    // Égaux pour l'œil, départagés tels qu'ils s'écrivent : l'ordre est total.
    assert_eq!(comparer_les_noms("img007", "img7"), Less);
    assert_eq!(comparer_les_noms("img7", "img007"), Greater);
    let n = "9".repeat(40);
    assert_eq!(
        comparer_les_noms(&format!("a{n}"), "a1"),
        Greater,
        "aucun nombre ne déborde"
    );
}

#[test]
fn test_les_dossiers_d_abord_puis_le_tri_du_dossier() {
    let mut e = vec![
        entree(
            "b.png",
            Sorte::Photo {
                largeur: 4,
                hauteur: 3,
            },
            30,
            1,
        ),
        entree("Zeta", Sorte::Dossier, 0, 5),
        entree("a.md", Sorte::Lisible, 10, 3),
        entree("alpha", Sorte::Dossier, 0, 9),
        entree("c.blend", Sorte::Autre, 20, 2),
    ];
    trier(&mut e, None);
    assert_eq!(noms(&e), ["alpha", "Zeta", "a.md", "b.png", "c.blend"]);
    trier(&mut e, Some(FolderSortMode::SizeDesc));
    assert_eq!(noms(&e), ["alpha", "Zeta", "b.png", "c.blend", "a.md"]);
    trier(&mut e, Some(FolderSortMode::ModifiedDesc));
    assert_eq!(noms(&e), ["alpha", "Zeta", "a.md", "c.blend", "b.png"]);
    trier(&mut e, Some(FolderSortMode::NameDesc));
    assert_eq!(noms(&e), ["Zeta", "alpha", "c.blend", "b.png", "a.md"]);
    trier(&mut e, Some(FolderSortMode::Type));
    assert_eq!(noms(&e), ["alpha", "Zeta", "c.blend", "a.md", "b.png"]);
}

#[test]
fn test_un_sous_dossier_attend_d_etre_lu() {
    let s = source_en_attente("/d/sous", Some(FolderSortMode::SizeAsc));
    assert!(s.pending_scan);
    assert_eq!(s.root_path, "/d/sous");
    assert_eq!(s.sort_by, Some(FolderSortMode::SizeAsc));
    assert_eq!(s.mode, "snapshot");
}
