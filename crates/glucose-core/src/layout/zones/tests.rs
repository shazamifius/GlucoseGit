//! PRESET-1 — ranger dans les zones : chaque zone ses images, en rangées justifiées.

use super::*;

fn image(id: &str, x: f64, y: f64, rapport: f64) -> BoardImage {
    let mut i = BoardImage::new(id, x, y, 100.0, 100.0);
    i.original_width = 100.0 * rapport;
    i.original_height = 100.0;
    i
}

fn place<'a>(r: &'a [LayoutRect], id: &str) -> &'a LayoutRect {
    r.iter().find(|p| p.id == id).expect("une place")
}

/// **Chaque zone range ses images en elle-même** ; une image hors de toute zone n'a pas de
/// place nouvelle.
#[test]
fn test_preset_1_chaque_zone_range_ses_images() {
    let zones = [
        BoardZone::new("g/a", 0.0, 0.0, 340.0, 700.0),
        BoardZone::new("g/b", 400.0, 0.0, 340.0, 700.0),
    ];
    let images = [
        image("a1", 100.0, 500.0, 1.0),
        image("b1", 500.0, 300.0, 1.0),
        image("dehors", 2000.0, 0.0, 1.0),
    ];
    let r = ranger_dans_les_zones(&images, &zones, OrganizeSort::None, 100.0, 10.0);
    assert_eq!(r.len(), 2, "l'image du dehors ne bouge pas");
    let (a, b) = (place(&r, "a1"), place(&r, "b1"));
    assert!(a.x - a.width / 2.0 >= 0.0 + MARGE && a.x + a.width / 2.0 <= 340.0 - MARGE);
    assert!(b.x - b.width / 2.0 >= 400.0 + MARGE);
    assert_eq!(a.y - a.height / 2.0, BANDEAU + MARGE, "sous le bandeau");
}

/// **Une rangée pleine remplit exactement la largeur intérieure** : la somme de ses largeurs et
/// de ses écarts vaut la zone moins ses marges.
#[test]
fn test_preset_1_une_rangee_pleine_remplit_la_zone() {
    let zones = [BoardZone::new("g/a", 0.0, 0.0, 340.0, 700.0)];
    let images: Vec<BoardImage> = (0..5)
        .map(|k| image(&format!("i{k}"), 100.0, 300.0, 1.0 + 0.3 * k as f64))
        .collect();
    let r = ranger_dans_les_zones(&images, &zones, OrganizeSort::None, 120.0, 10.0);
    let premiere_y = r[0].y;
    let rangee: Vec<&LayoutRect> = r
        .iter()
        .filter(|p| (p.y - premiere_y).abs() < 1e-9)
        .collect();
    assert!(rangee.len() >= 2, "plusieurs images sur la première rangée");
    let gauche = rangee
        .iter()
        .map(|p| p.x - p.width / 2.0)
        .fold(f64::MAX, f64::min);
    let droite = rangee
        .iter()
        .map(|p| p.x + p.width / 2.0)
        .fold(f64::MIN, f64::max);
    assert!((gauche - MARGE).abs() < 1e-9);
    assert!((droite - (340.0 - MARGE)).abs() < 1e-9, "{droite}");
}

/// **Une image trop large se réduit** à la largeur de la zone, ses proportions gardées.
#[test]
fn test_preset_1_une_image_trop_large_se_reduit() {
    let zones = [BoardZone::new("g/a", 0.0, 0.0, 340.0, 700.0)];
    let images = [image("large", 100.0, 300.0, 4.0)];
    let r = ranger_dans_les_zones(&images, &zones, OrganizeSort::None, 200.0, 10.0);
    let p = place(&r, "large");
    assert!((p.width - (340.0 - 2.0 * MARGE)).abs() < 1e-9);
    assert!((p.width / p.height - 4.0).abs() < 1e-9);
}

/// **Le tri choisi vaut dans la zone** : par taille décroissante, la plus grande ouvre la
/// première rangée, quel que soit l'ordre du tableau.
#[test]
fn test_preset_1_le_tri_vaut_dans_la_zone() {
    let zones = [BoardZone::new("g/a", 0.0, 0.0, 600.0, 700.0)];
    let mut grande = image("grande", 300.0, 300.0, 1.0);
    (grande.width, grande.height) = (300.0, 300.0);
    let images = [image("petite", 100.0, 300.0, 1.0), grande];
    let r = ranger_dans_les_zones(&images, &zones, OrganizeSort::SizeDesc, 100.0, 10.0);
    assert!(
        place(&r, "grande").x < place(&r, "petite").x,
        "la grande d'abord"
    );
    let r = ranger_dans_les_zones(&images, &zones, OrganizeSort::None, 100.0, 10.0);
    assert!(
        place(&r, "petite").x < place(&r, "grande").x,
        "l'ordre du tableau"
    );
}

/// **Deux zones qui se chevauchent** : l'image va à la plus haute — celle qu'on voit.
#[test]
fn test_preset_1_la_zone_du_dessus_l_emporte() {
    let zones = [
        BoardZone::new("g/dessous", 0.0, 0.0, 400.0, 400.0),
        BoardZone::new("g/dessus", 200.0, 200.0, 400.0, 400.0),
    ];
    let images = [image("i", 300.0, 300.0, 1.0)];
    let r = ranger_dans_les_zones(&images, &zones, OrganizeSort::None, 100.0, 10.0);
    let p = place(&r, "i");
    assert_eq!(p.x - p.width / 2.0, 200.0 + MARGE, "dans la zone du dessus");
}

/// **Tout tient dans la zone** : trop d'images pour la hauteur visée, elles se réduisent —
/// aucune ne descend sous le bord du bas —, et ranger encore ne change plus rien.
#[test]
fn test_preset_1_tout_tient_dans_la_zone_et_le_geste_est_stable() {
    let zones = [
        BoardZone::new("g/haut", 0.0, 0.0, 340.0, 300.0),
        BoardZone::new("g/bas", 0.0, 320.0, 340.0, 300.0),
    ];
    let mut images: Vec<BoardImage> = (0..40)
        .map(|k| image(&format!("i{k}"), 100.0, 100.0, 0.7 + 0.1 * (k % 7) as f64))
        .collect();
    let r = ranger_dans_les_zones(&images, &zones, OrganizeSort::None, 200.0, 10.0);
    assert_eq!(r.len(), 40);
    for p in &r {
        assert!(
            p.y + p.height / 2.0 <= 300.0 - MARGE + 1e-9,
            "{} déborde",
            p.id
        );
        assert!(p.width > 1.0, "{} a encore une taille", p.id);
    }
    for (img, p) in images.iter_mut().zip(&r) {
        (img.x, img.y, img.width, img.height) = (p.x, p.y, p.width, p.height);
    }
    let encore = ranger_dans_les_zones(&images, &zones, OrganizeSort::None, 200.0, 10.0);
    assert_eq!(encore, r, "le même rangement, deux fois");
}

/// **Une zone minuscule chargée de milliers d'images** : la hauteur qui tient passe sous le
/// demi-point, et aucune image n'est pour autant réduite à rien.
#[test]
fn test_preset_1_une_zone_minuscule_garde_des_images() {
    let zones = [BoardZone::new("g/petite", 0.0, 0.0, 100.0, 80.0)];
    let images: Vec<BoardImage> = (0..20_000)
        .map(|k| image(&format!("i{k}"), 50.0, 50.0, 1.0))
        .collect();
    let r = ranger_dans_les_zones(&images, &zones, OrganizeSort::None, 200.0, 10.0);
    assert!(r
        .iter()
        .all(|p| p.width > 0.0 && p.y + p.height / 2.0 <= 80.0 - MARGE + 1e-9));
}
