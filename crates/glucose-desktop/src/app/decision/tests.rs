use super::*;
use tiny_skia::{Color, Paint, Rect, Transform};

/// Peint un rectangle opaque sur les lignes `haut..bas`, sur toute la largeur.
fn bande(haut: u32, bas: u32, gris: u8) -> impl FnOnce(&mut PixmapMut) {
    move |p: &mut PixmapMut| {
        let mut peinture = Paint::default();
        peinture.set_color(Color::from_rgba8(gris, gris, gris, 255));
        let rect = Rect::from_ltrb(0.0, haut as f32, p.width() as f32, bas as f32).expect("rect");
        p.fill_rect(rect, &peinture, Transform::identity(), None);
    }
}

/// **Rien d'écrit, rien à poser** : la carte ne reçoit pas une texture transparente.
#[test]
fn test_rien_d_ecrit_ne_pose_rien() {
    let mut d = Decision::default();
    assert!(d.peindre((40, 30), |_| {}).is_none());
    assert!(d.pixels(&cle(d.generation)).is_none());
}

/// **La tranche est exactement ce qui porte de l'encre** : posée à sa première ligne, haute de
/// ce qu'il faut, et ses pixels sont ceux qu'on a peints.
#[test]
fn test_la_tranche_est_ce_qui_porte_de_l_encre() {
    let mut d = Decision::default();
    let posee = d.peindre((40, 30), bande(10, 20, 200)).expect("posée");
    assert_eq!((posee.pose.y, posee.pose.hauteur), (10.0, 10.0));
    assert_eq!(posee.pose.largeur, 40.0);
    assert_eq!(posee.identite, IDENTITE);
    let pixels = d.pixels(&posee.cle).expect("les pixels de cette clé");
    assert_eq!((pixels.width(), pixels.height()), (40, 10));
    assert_eq!(pixels.pixel(5, 5).map(|p| p.red()), Some(200));
}

/// **Une image identique garde sa clé** : une question immobile ne repart pas sur le bus. Une
/// image changée en prend une neuve, et l'ancienne clé ne répond plus.
#[test]
fn test_la_cle_ne_change_que_si_les_pixels_changent() {
    let mut d = Decision::default();
    let premiere = d.peindre((40, 30), bande(10, 20, 200)).expect("posée").cle;
    let meme = d.peindre((40, 30), bande(10, 20, 200)).expect("posée").cle;
    assert_eq!(premiere, meme, "les mêmes pixels, la même clé");
    let autre = d.peindre((40, 30), bande(10, 20, 90)).expect("posée").cle;
    assert_ne!(premiere, autre, "d'autres pixels, une autre clé");
    assert!(
        d.pixels(&premiere).is_none(),
        "l'ancienne clé ne répond plus"
    );
    assert!(d.pixels(&autre).is_some());
}

/// **Ce que l'image précédente avait écrit s'efface** : une question fermée ne reste pas dans
/// le tampon, et la tranche suivante ne la porte pas.
#[test]
fn test_l_image_precedente_s_efface() {
    let mut d = Decision::default();
    d.peindre((40, 60), bande(10, 20, 200)).expect("posée");
    let posee = d.peindre((40, 60), bande(40, 50, 200)).expect("posée");
    assert_eq!((posee.pose.y, posee.pose.hauteur), (40.0, 10.0));
    assert!(d.peindre((40, 60), |_| {}).is_none(), "plus rien d'écrit");
}

/// **Retirer**, c'est ne plus rien prêter à la carte.
#[test]
fn test_retirer_ne_prete_plus_rien() {
    let mut d = Decision::default();
    let cle_posee = d.peindre((40, 30), bande(10, 20, 200)).expect("posée").cle;
    d.retirer();
    assert!(d.pixels(&cle_posee).is_none());
}
