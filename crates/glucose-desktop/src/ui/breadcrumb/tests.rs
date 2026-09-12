//! Les lois du fil d'Ariane. Il n'apparaît que si l'on est quelque part, et il ramène là où
//! il dit qu'il ramène.

use super::*;
use glucose_core::types::CanvasFolder;

/// Un store où l'on est descendu de `profondeur` dossiers imbriqués, nommés comme un vrai
/// chemin de travail.
fn descendu(profondeur: usize) -> Store {
    let noms = ["Recherches", "Sources", "Brouillons", "Archives"];
    let mut store = Store::new("Projet");
    for i in 0..profondeur {
        let board = store.project.active_board_id.clone();
        let mut f = CanvasFolder::new(format!("fold-{i}"), noms[i % noms.len()], String::new());
        f.x = 0.0;
        f.y = 0.0;
        store.create_folder(&board, f);
        let id = store
            .active_board()
            .and_then(|b| b.folders.last())
            .map(|f| f.id.clone())
            .expect("le dossier vient d'être créé");
        store.try_enter_folder(&id).expect("y entrer");
    }
    store
}

fn typo() -> Typography {
    Typography::new()
}

/// **À la racine, le fil ne s'affiche pas.** Un seul segment n'apprend rien et volerait une
/// bande de canevas à qui n'entre jamais dans un dossier.
#[test]
fn test_a_la_racine_le_fil_ne_s_affiche_pas() {
    let store = Store::new("Projet");
    assert_eq!(store.folder_path(), vec!["Projet".to_string()]);
    assert!(layout_breadcrumb(&store, &typo(), 78.0, 1.0).is_empty());
}

/// Dès le premier dossier, le chemin complet s'affiche, racine comprise.
#[test]
fn test_le_chemin_affiche_la_racine_et_les_dossiers_traverses() {
    let store = descendu(3);
    assert_eq!(
        store.folder_path(),
        vec!["Projet", "Recherches", "Sources", "Brouillons"]
    );
    let segs = layout_breadcrumb(&store, &typo(), 78.0, 1.0);
    assert_eq!(segs.len(), 4);
    assert_eq!(
        segs.iter().map(|s| s.label.as_str()).collect::<Vec<_>>(),
        vec!["Projet", "Recherches", "Sources", "Brouillons"]
    );
    // Seul le dernier est le segment courant.
    assert_eq!(segs.iter().filter(|s| s.current).count(), 1);
    assert!(segs.last().expect("un dernier").current);
}

/// Les segments se suivent de gauche à droite sans se chevaucher : la mise en page est celle
/// que le clic lira, donc ils ne peuvent pas se disputer un pixel.
#[test]
fn test_les_segments_se_suivent_sans_se_chevaucher() {
    let segs = layout_breadcrumb(&descendu(4), &typo(), 78.0, 1.0);
    assert_eq!(segs.len(), 5);
    for paire in segs.windows(2) {
        let (gauche, droite) = (&paire[0], &paire[1]);
        assert!(
            gauche.rect.0 + gauche.rect.2 <= droite.rect.0,
            "{} chevauche {}",
            gauche.label,
            droite.label
        );
        assert_eq!(gauche.rect.1, droite.rect.1, "tous sur la même ligne");
    }
    assert_eq!(segs[0].rect.0, MARGIN, "le premier commence à la marge");
}

/// **Cliquer un segment remonte à sa profondeur**, et à aucune autre.
#[test]
fn test_cliquer_un_segment_remonte_a_sa_profondeur() {
    for cible in 0..3 {
        let mut store = descendu(3);
        let segs = layout_breadcrumb(&store, &typo(), 78.0, 1.0);
        let seg = &segs[cible];
        let au_milieu = (seg.rect.0 + seg.rect.2 / 2.0, seg.rect.1 + seg.rect.3 / 2.0);

        let depth = hit_breadcrumb(&store, &typo(), 78.0, 1.0, au_milieu)
            .unwrap_or_else(|| panic!("le segment {cible} doit être cliquable"));
        assert_eq!(depth, cible);

        assert!(store.exit_to_depth(depth));
        assert_eq!(store.folder_path().len(), cible + 1);
    }
}

/// Cliquer le segment où l'on est déjà n'est pas une erreur : c'est un geste sans effet.
#[test]
fn test_cliquer_la_ou_on_est_ne_fait_rien() {
    let mut store = descendu(2);
    let segs = layout_breadcrumb(&store, &typo(), 78.0, 1.0);
    let dernier = segs.last().expect("un dernier");
    let au_milieu = (
        dernier.rect.0 + dernier.rect.2 / 2.0,
        dernier.rect.1 + dernier.rect.3 / 2.0,
    );
    assert_eq!(hit_breadcrumb(&store, &typo(), 78.0, 1.0, au_milieu), None);

    let avant = store.project.active_board_id.clone();
    assert!(!store.exit_to_depth(2), "remonter là où l'on est ne bouge rien");
    assert_eq!(store.project.active_board_id, avant);
}

/// Un clic hors de la bande ne touche pas au fil — le canevas garde ses clics.
#[test]
fn test_un_clic_hors_de_la_bande_ne_remonte_pas() {
    let store = descendu(2);
    for point in [(5.0, 40.0), (5.0, 200.0), (2_000.0, 85.0), (-10.0, 85.0)] {
        assert_eq!(hit_breadcrumb(&store, &typo(), 78.0, 1.0, point), None, "{point:?}");
    }
}

/// Un nom de dossier trop long est tronqué, comme sur le cadre — le fil ne peut pas pousser
/// la barre d'onglets hors de l'écran.
#[test]
fn test_un_nom_trop_long_est_tronque() {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    let long = "un nom de dossier vraiment très long qui ne tiendrait jamais".to_string();
    let mut f = CanvasFolder::new("f", long.clone(), String::new());
    f.x = 0.0;
    store.create_folder(&board, f);
    let id = store
        .active_board()
        .and_then(|b| b.folders.last())
        .map(|f| f.id.clone())
        .expect("le dossier");
    store.try_enter_folder(&id).expect("y entrer");

    let segs = layout_breadcrumb(&store, &typo(), 78.0, 1.0);
    let dernier = segs.last().expect("un dernier");
    assert!(dernier.label.chars().count() <= SEGMENT_MAX_CHARS);
    assert!(dernier.label.ends_with('…'));
    assert_ne!(dernier.label, long);
}

/// Un dossier dont le nom a disparu du tableau parent donne un segment incertain, jamais un
/// trou : un chemin à trou serait pire.
#[test]
fn test_un_dossier_disparu_laisse_un_segment_incertain() {
    let mut store = descendu(2);
    // Retirer le dossier du tableau parent sans toucher à la pile de navigation.
    if let Some((parent, folder_id)) = store.folder_stack.last().cloned() {
        if let Some(b) = store.project.boards.iter_mut().find(|b| b.id == parent) {
            b.folders.retain(|f| f.id != folder_id);
        }
    }
    let chemin = store.folder_path();
    assert_eq!(chemin.len(), 3, "la profondeur ne change pas");
    assert_eq!(chemin.last().map(String::as_str), Some("?"));
}

/// Le fil dessine bien quelque chose, et rien du tout à la racine.
#[test]
fn test_le_fil_laisse_des_pixels_des_qu_on_est_entre() {
    let theme = Theme::dark();
    let typography = typo();
    let encre = |store: &Store| {
        let mut pixmap = tiny_skia::Pixmap::new(900, 200).expect("pixmap");
        pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
        let h = draw_breadcrumb(&mut pixmap.as_mut(), store, &typography, &theme, 78.0, 1.0);
        let encres = pixmap
            .pixels()
            .iter()
            .filter(|p| p.red() > 0 || p.green() > 0 || p.blue() > 0)
            .count();
        (h, encres)
    };

    let (h_racine, encre_racine) = encre(&Store::new("Projet"));
    assert_eq!(h_racine, 0.0, "à la racine, le fil n'occupe aucune hauteur");
    assert_eq!(encre_racine, 0);

    let (h, encres) = encre(&descendu(2));
    assert_eq!(h, HEIGHT);
    assert!(encres > 500, "{encres} pixels : le fil doit se lire");
}

/// Les mesures de la bande sont celles qu'annonce le module.
#[test]
fn test_les_metriques_de_la_bande() {
    assert_eq!((HEIGHT, MARGIN, FONT), (26.0, 12.0, 12.0));
    assert_eq!(SEGMENT_MAX_CHARS, 24);
    assert_eq!(SEPARATOR, "  ›  ");
}
