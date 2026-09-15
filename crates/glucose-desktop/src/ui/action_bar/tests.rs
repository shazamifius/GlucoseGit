//! Ce que la barre d'action montre, où elle le montre, et ce qu'elle refuse de laisser
//! passer.

use super::*;
use crate::typography::Typography;
use glucose_core::types::{Annotation, BoardImage};

const SCREEN: (f32, f32) = (1440.0, 900.0);

fn store_with(images: usize, cartes: usize) -> Store {
    let mut store = Store::new("barre");
    let board = store.project.active_board_id.clone();
    for k in 0..images {
        store.add_image(
            &board,
            BoardImage::new(format!("i{k}"), 0.0, 0.0, 100.0, 100.0),
        );
    }
    for k in 0..cartes {
        store.add_annotation(&board, Annotation::text(format!("a{k}"), 0.0, 0.0, "x"));
    }
    store.set_selected_image_ids((0..images).map(|k| format!("i{k}")).collect());
    store.set_selected_annotation_ids((0..cartes).map(|k| format!("a{k}")).collect());
    store
}

fn bar(store: &Store, typo: &Typography) -> Option<ActionBar> {
    layout_action_bar(store, typo, SCREEN, 1.0)
}

#[test]
fn test_sans_selection_la_barre_nexiste_pas() {
    let typo = Typography::new();
    let mut store = store_with(2, 0);
    store.clear_selection();
    assert!(
        bar(&store, &typo).is_none(),
        "une barre vide volerait une bande de canevas pour ne rien dire"
    );
}

#[test]
fn test_le_compteur_saccorde() {
    let typo = Typography::new();
    assert_eq!(
        bar(&store_with(1, 0), &typo)
            .expect("une barre")
            .count_label,
        "1 sélectionné"
    );
    assert_eq!(
        bar(&store_with(2, 0), &typo)
            .expect("une barre")
            .count_label,
        "2 sélectionnés"
    );
    // Images et cartes comptent ensemble : c'est « ce qui est sélectionné », pas « les
    // images sélectionnées ».
    assert_eq!(
        bar(&store_with(2, 3), &typo)
            .expect("une barre")
            .count_label,
        "5 sélectionnés"
    );
}

/// Fiche 10 — la pastille est **centrée** et posée à 12 px du bas.
#[test]
fn test_la_pastille_est_centree_en_bas() {
    let typo = Typography::new();
    let b = bar(&store_with(3, 0), &typo).expect("une barre");
    let (x, y, w, h) = b.rect;
    assert!(
        ((x + w / 2.0) - SCREEN.0 / 2.0).abs() < 0.5,
        "centre à {} au lieu de {}",
        x + w / 2.0,
        SCREEN.0 / 2.0
    );
    assert!(
        ((y + h) - (SCREEN.1 - BOTTOM)).abs() < 0.5,
        "bas à {} au lieu de {}",
        y + h,
        SCREEN.1 - BOTTOM
    );
}

/// Une carte ou un dossier ne porte pas de verrou : le bouton n'a alors rien à faire là.
#[test]
fn test_sans_image_le_bouton_du_verrou_nest_pas_propose() {
    let typo = Typography::new();
    let b = bar(&store_with(0, 2), &typo).expect("une barre");
    let clics: Vec<_> = b.buttons.iter().map(|x| x.click).collect();
    assert_eq!(clics, vec![ActionBarClick::Delete]);
}

#[test]
fn test_le_bouton_dit_letat_et_pas_seulement_laction() {
    let typo = Typography::new();
    let mut store = store_with(2, 0);
    let b = bar(&store, &typo).expect("une barre");
    let verrou = &b.buttons[0];
    assert_eq!(verrou.label, "Verrouiller");
    assert_eq!(verrou.icon, IconType::Unlock);
    assert!(!verrou.on);

    let board = store.project.active_board_id.clone();
    store.toggle_lock_selection(&board);
    let b = bar(&store, &typo).expect("une barre");
    let verrou = &b.buttons[0];
    assert_eq!(verrou.label, "Verrouillé", "l'état, pas l'action");
    assert_eq!(verrou.icon, IconType::Lock);
    assert!(verrou.on, "et il porte le rouge de l'interface");
}

/// Une sélection mixte n'est pas « verrouillée » : le bouton propose de la fermer.
#[test]
fn test_une_selection_partiellement_verrouillee_propose_de_verrouiller() {
    let typo = Typography::new();
    let mut store = store_with(3, 0);
    let board = store.project.active_board_id.clone();
    store.set_selected_image_ids(vec!["i0".into()]);
    store.toggle_lock_selection(&board);
    store.set_selected_image_ids(vec!["i0".into(), "i1".into(), "i2".into()]);

    let b = bar(&store, &typo).expect("une barre");
    assert_eq!(b.buttons[0].label, "Verrouiller");
    assert!(!b.buttons[0].on);
}

/// Le clic et le tracé lisent la même géométrie (loi L4) : viser le centre d'un bouton
/// déclenche ce bouton, toujours.
#[test]
fn test_chaque_bouton_repond_en_son_centre() {
    let typo = Typography::new();
    let b = bar(&store_with(2, 1), &typo).expect("une barre");
    assert_eq!(b.buttons.len(), 2, "verrou et suppression");
    for btn in &b.buttons {
        let (x, y, w, h) = btn.rect;
        assert_eq!(
            hit_action_bar(&b, x + w / 2.0, y + h / 2.0),
            Some(btn.click),
            "{:?} ne répond pas en son centre",
            btn.click
        );
    }
}

/// Ce qui tombe sur la pastille ne descend jamais au canevas : sinon, cliquer entre deux
/// boutons désélectionne, et la barre disparaît sous le doigt.
#[test]
fn test_la_pastille_prend_tout_ce_qui_tombe_dessus() {
    let typo = Typography::new();
    let b = bar(&store_with(2, 0), &typo).expect("une barre");
    let (x, y, w, h) = b.rect;

    // Le creux entre le compteur et le premier bouton : aucun bouton, mais la pastille.
    let creux = (b.separator_x + 1.0, y + h / 2.0);
    assert!(covers(&b, creux.0, creux.1));
    assert_eq!(hit_action_bar(&b, creux.0, creux.1), None);

    assert!(!covers(&b, x - 1.0, y + h / 2.0), "à gauche de la pastille");
    assert!(!covers(&b, x + w / 2.0, y - 1.0), "au-dessus");
}

/// Les boutons se suivent sans se chevaucher ni sortir de la pastille.
#[test]
fn test_les_boutons_tiennent_dans_la_pastille_et_ne_se_marchent_pas_dessus() {
    let typo = Typography::new();
    for (images, cartes) in [(1, 0), (0, 1), (3, 2), (11, 0)] {
        let store = store_with(images, cartes);
        let b = bar(&store, &typo).expect("une barre");
        let (bx, _, bw, _) = b.rect;
        let mut precedent = b.separator_x;
        for btn in &b.buttons {
            let (x, _, w, _) = btn.rect;
            assert!(x >= precedent, "{images}+{cartes} : {btn:?} recule");
            assert!(
                x + w <= bx + bw + 0.5,
                "{images}+{cartes} : {btn:?} déborde de la pastille"
            );
            precedent = x + w;
        }
    }
}

/// La barre garde sa marge sous elle, à toutes les échelles d'interface.
///
/// Sur la capture témoin, elle **paraît** toucher le bord. L'œil sur une image réduite n'est
/// pas une preuve — celui-ci mesure. Si la marge est là, le reproche porte sur sa valeur et
/// non sur un défaut de calcul, et c'est une question de goût qui se tranche en la comparant
/// à la référence.
#[test]
fn test_the_action_bar_keeps_its_margin_below() {
    let typo = crate::typography::Typography::new();
    let store = store_with(2, 2);
    for s in [1.0f32, 1.25, 1.5, 2.0] {
        for (w, h) in [(1280.0f32, 720.0f32), (1800.0, 900.0), (3840.0, 2160.0)] {
            let Some(bar) = layout_action_bar(&store, &typo, (w, h), s) else {
                panic!("une barre pour 4 éléments");
            };
            let bas = bar.rect.1 + bar.rect.3;
            let marge = h - bas;
            assert!(
                marge >= BOTTOM * s - 0.5,
                "échelle {s}, écran {w}×{h} : {marge} px sous la barre, attendu {}",
                BOTTOM * s
            );
        }
    }
}
