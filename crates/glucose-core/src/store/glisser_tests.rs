//! GLISSER-1 : un glisser s'écrit en une translation, et le fichier rejoue l'écran au bit près.

use super::*;
use crate::store::journal::Bouts;
use crate::store::journal::Transaction;
use crate::types::BoardImage;

const B: &str = "main";

/// Une photo « a », une carte « t », et une flèche « f » qui part de la photo : la photo et la
/// carte sont sélectionnées, la flèche n'a que son origine qui suit.
fn magasin() -> Store {
    let mut store = Store::new("glisser");
    store.add_image(B, BoardImage::new("a", 0.1, 0.2, 100.0, 80.0));
    store.add_annotation(B, Annotation::text("t", 0.3, 0.7, "carte"));
    let mut f = Annotation::arrow("f", 50.1, 40.3, 400.7, 90.9);
    if let Annotation::Arrow { source_id, .. } = &mut f {
        *source_id = Some("a".into());
    }
    store.add_annotation(B, f);
    // Une flèche emportée en entier, avec un coude : il suit aussi.
    let mut g = Annotation::arrow("g", 10.3, 20.9, 300.1, 30.7);
    if let Annotation::Arrow { waypoints, .. } = &mut g {
        waypoints.push(crate::types::Point2D { x: 150.3, y: 60.7 });
    }
    store.add_annotation(B, g);
    // Une flèche libre qui arrive sur la carte : sa cible suit.
    let mut h = Annotation::arrow("h", 500.9, 500.1, 0.9, 1.3);
    if let Annotation::Arrow { target_id, .. } = &mut h {
        *target_id = Some("t".into());
    }
    store.add_annotation(B, h);
    store.clear_selection();
    store.select_image("a".into(), false);
    store.selected_annotation_ids = vec!["t".into(), "g".into()];
    store.journal.prendre_les_ecrits();
    store
}

/// Les déplacements totaux d'un glisser : des nombres qui ne s'additionnent pas exactement.
fn totaux() -> impl Iterator<Item = (f64, f64)> {
    (1..=60).map(|k| (0.1 * f64::from(k) + 0.013, -0.07 * f64::from(k)))
}

/// **Un glisser s'écrit en une translation, qui rejoue l'écran au bit près.** Pendant le geste,
/// chaque pas est lisible — l'écran et les flèches en ont besoin ; publié, il n'en reste qu'une
/// translation, du départ à l'arrivée. Et l'ancienne façon — un pas par mouvement, rejoué pas
/// à pas — posait les nœuds ailleurs que l'écran : l'épreuve le montre, sans quoi elle ne
/// prouverait rien.
#[test]
fn test_glisser_1_un_glisser_s_ecrit_en_une_translation_qui_rejoue_l_ecran() {
    let mut store = magasin();
    let depart = store.project.clone();
    store.begin_live_edit();
    for total in totaux() {
        store.glisser_la_selection(B, total);
    }
    assert_eq!(
        store.journal.en_cours().map(|(_, e)| e.len()),
        Some(60),
        "pendant le geste, chaque pas se lit"
    );
    store.end_live_edit();

    let publies = store.journal.prendre_les_ecrits();
    assert_eq!(publies.len(), 1);
    let dernier = totaux().last().unwrap_or_default();
    match &publies[0].edits[..] {
        [Edit::Translation {
            delta,
            images,
            annotations,
            bouts,
            ..
        }] => {
            assert_eq!(*delta, dernier, "du départ à l'arrivée");
            assert_eq!((images.len(), annotations.len(), bouts.len()), (1, 2, 2));
        }
        autre => panic!("une seule translation attendue : {autre:?}"),
    }
    let mut rejoue = depart.clone();
    assert!(publies[0].apply(&mut rejoue));
    assert_eq!(
        rejoue, store.project,
        "le fichier rejoue l'écran, au bit près"
    );

    // L'ancienne façon : un pas par mouvement, et chacun rejoué à la suite.
    let mut pas_a_pas = depart;
    let mut avant = (0.0, 0.0);
    for total in totaux() {
        let mut t = Transaction::default();
        t.push(Edit::Translation {
            board: B.into(),
            delta: (total.0 - avant.0, total.1 - avant.1),
            images: vec![0],
            annotations: vec![0, 2],
            folders: vec![],
            bouts: vec![
                (
                    1,
                    Bouts {
                        origine: true,
                        cible: false,
                    },
                ),
                (
                    3,
                    Bouts {
                        origine: false,
                        cible: true,
                    },
                ),
            ],
        });
        assert!(t.apply(&mut pas_a_pas));
        avant = total;
    }
    assert_ne!(
        pas_a_pas, store.project,
        "pas à pas, la carte et la flèche finissaient un ulp à côté de l'écran"
    );
}

/// **Un glisser revenu à son point de départ ne laisse rien** : ni geste, ni version, et chaque
/// chose exactement où elle était.
#[test]
fn test_glisser_1_revenu_au_depart_ne_laisse_rien() {
    let mut store = magasin();
    let depart = store.project.clone();
    let (version, profondeur) = (store.version, store.undo_depth());
    store.begin_live_edit();
    for total in totaux() {
        store.glisser_la_selection(B, total);
    }
    store.glisser_la_selection(B, (0.0, 0.0));
    store.end_live_edit();
    assert_eq!(store.project, depart);
    assert_eq!((store.version, store.undo_depth()), (version, profondeur));
}

/// **`Échap` rend chaque départ exact** — là où défaire les pas un à un laissait des ulps.
#[test]
fn test_glisser_1_echap_rend_les_departs_exacts() {
    let mut store = magasin();
    let depart = store.project.clone();
    store.begin_live_edit();
    for total in totaux() {
        store.glisser_la_selection(B, total);
    }
    assert!(store.cancel_live_edit());
    assert_eq!(store.project, depart);
    assert!(!store.in_live_edit());
}

/// **Deux glissers dans un même geste** : le premier se ferme en sa translation quand le second
/// commence, et le geste publié rejoue l'écran.
#[test]
fn test_glisser_1_deux_glissers_dans_un_geste() {
    let mut store = magasin();
    let depart = store.project.clone();
    store.begin_live_edit();
    for total in totaux() {
        store.glisser_la_selection(B, total);
    }
    // Une autre sélection : un autre glisser, qui part de là où le premier a laissé la photo.
    store.clear_selection();
    store.select_image("a".into(), false);
    store.rename_board(B, "renommé");
    for total in totaux() {
        store.glisser_la_selection(B, (total.1, total.0));
    }
    store.end_live_edit();
    let publies = store.journal.prendre_les_ecrits();
    assert_eq!(publies.len(), 1);
    assert_eq!(
        publies[0].edits.len(),
        3,
        "une translation, un nom, une translation"
    );
    let mut rejoue = depart;
    assert!(publies[0].apply(&mut rejoue));
    assert_eq!(rejoue, store.project);
}
