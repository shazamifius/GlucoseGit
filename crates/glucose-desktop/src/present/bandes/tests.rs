//! Ce que les bandes doivent garantir : qu'elles voient tout ce qui est écrit, qu'elles
//! n'effacent que ce qu'elles nomment, et qu'une ligne qui cesse d'être écrite s'efface
//! quand même.

// Une bande EST un intervalle : `[0..10]` dit exactement ce qu'il veut dire, et la
// réécriture que clippy propose -- `(0..10).collect()` -- dirait autre chose.
#![allow(clippy::single_range_in_vec_init)]

use super::*;
use tiny_skia::{Color, Paint, Rect, Transform};

/// Un pixmap transparent de cette taille.
fn vide(l: u32, h: u32) -> Pixmap {
    Pixmap::new(l, h).expect("un pixmap")
}

/// Peint un rectangle opaque, comme la chrome le ferait.
fn peindre(p: &mut Pixmap, y: f32, hauteur: f32) {
    let mut encre = Paint::default();
    encre.set_color(Color::from_rgba8(200, 100, 50, 255));
    p.fill_rect(
        Rect::from_xywh(0.0, y, p.width() as f32, hauteur).expect("un rectangle"),
        &encre,
        Transform::identity(),
        None,
    );
}

/// **Deux zones séparées donnent deux bandes**, et non une qui les engloberait.
///
/// C'est toute la raison d'être du module : `bench_dessus` mesure 12 % de lignes écrites pour
/// une étendue de 99 %, parce que la chrome occupe le haut et le bas. Une boîte englobante
/// n'économiserait donc rien.
#[test]
fn test_deux_zones_separees_donnent_deux_bandes() {
    let mut p = vide(64, 100);
    peindre(&mut p, 0.0, 10.0);
    peindre(&mut p, 90.0, 10.0);

    let b = Bandes::relever(&p);
    assert_eq!(b.intervalles(), [0..10, 90..100].as_slice());
    assert_eq!(b.lignes(), 20, "vingt lignes ecrites sur cent");
}

/// **Une ligne écrite est vue, fût-ce d'un seul pixel à peine visible.**
///
/// Le balayage existe pour qu'aucun dessin ne puisse lui échapper ; un alpha de 1 sur 255 est
/// le cas limite qui le vérifie.
#[test]
fn test_un_seul_pixel_presque_transparent_suffit_a_salir_sa_ligne() {
    let mut p = vide(64, 100);
    let mut encre = Paint::default();
    encre.set_color(Color::from_rgba8(255, 255, 255, 1));
    p.fill_rect(
        Rect::from_xywh(30.0, 42.0, 1.0, 1.0).expect("un rectangle"),
        &encre,
        Transform::identity(),
        None,
    );

    assert_eq!(Bandes::relever(&p).intervalles(), [42..43].as_slice());
}

/// **Effacer ne touche que les lignes nommées.**
#[test]
fn test_effacer_laisse_intact_ce_qui_n_est_pas_dans_la_bande() {
    let mut p = vide(64, 100);
    peindre(&mut p, 0.0, 10.0);
    peindre(&mut p, 90.0, 10.0);

    Bandes(vec![0..10]).effacer(&mut p);

    let reste = Bandes::relever(&p);
    assert_eq!(
        reste.intervalles(),
        [90..100].as_slice(),
        "le haut est efface, le bas ne doit pas l'etre"
    );
}

/// **Une ligne qui cesse d'être écrite s'efface quand même** — et c'est l'union qui le garantit.
///
/// Sans elle, la couche garderait ce que l'image d'avant y avait mis : une poignée de
/// sélection resterait affichée après la désélection, et rien dans le rendu de l'image
/// courante ne le montrerait, puisqu'elle ne dessine plus rien à cet endroit.
#[test]
fn test_l_union_couvre_ce_que_l_image_precedente_ecrivait() {
    let precedentes = Bandes(vec![0..10, 40..50]);
    let courantes = Bandes(vec![0..10]);

    let a_traiter = precedentes.union(&courantes);
    assert_eq!(
        a_traiter.intervalles(),
        [0..10, 40..50].as_slice(),
        "la bande que l'image courante n'ecrit plus doit rester a effacer"
    );
}

/// Deux bandes qui se touchent ou se chevauchent n'en font qu'une : sans quoi le nombre de
/// téléversements grandirait sans raison, et chacun coûte un appel.
#[test]
fn test_l_union_fond_ce_qui_se_touche() {
    let a = Bandes(vec![0..10, 30..40]);
    let b = Bandes(vec![10..20, 35..60]);

    assert_eq!(a.union(&b).intervalles(), [0..20, 30..60].as_slice());
}

/// Une couche vierge ne demande ni effacement ni téléversement — c'est la même loi que la
/// couche du dessous, qui **cesse d'exister** quand elle est vide (fiche 22 § 4).
#[test]
fn test_une_couche_vierge_ne_porte_aucune_bande() {
    let p = vide(64, 100);
    let b = Bandes::relever(&p);
    assert!(b.vides());
    assert_eq!(b.lignes(), 0);
}

/// Tout, d'un bloc : ce qu'il faut quand la texture vient d'être créée et ne contient rien de
/// ce qu'on croit.
#[test]
fn test_tout_couvre_la_hauteur_entiere() {
    assert_eq!(Bandes::tout(100).intervalles(), [0..100].as_slice());
    assert_eq!(Bandes::tout(100).lignes(), 100);
    assert!(Bandes::tout(0).vides());
}
