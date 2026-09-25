//! La mise en page des onglets : ce que le clic et le dessin lisent tous deux.

use super::*;
use crate::ui::UiState;

/// **Chaque point d'un onglet le désigne, et sa croix passe avant lui** ; le « + » est au bout.
#[test]
fn test_la_croix_passe_avant_son_onglet() {
    let mut store = Store::new("onglets");
    store.add_board("Un nom plus long");
    let ui = UiState::new();
    let typo = Typography::new();
    let tabs = layout_tabs(&store, &ui, &typo);
    assert_eq!(tabs.len(), 3);
    for t in tabs.iter().filter(|t| !t.is_plus) {
        let (cx, cy, c) = t.fermer.expect("deux onglets : chacun a sa croix");
        assert!(cx + c <= t.x + t.width, "la croix est dans l'onglet");
        assert_eq!(
            cible(&tabs, cx + c / 2.0, cy + c / 2.0),
            Some(CibleOnglet::Fermer(t.board_id.clone()))
        );
        assert_eq!(
            cible(&tabs, t.x + 2.0, t.y + t.height / 2.0),
            Some(CibleOnglet::Onglet(t.board_id.clone()))
        );
    }
    let plus = tabs.last().expect("le +");
    assert_eq!(
        cible(&tabs, plus.x + 2.0, plus.y + 2.0),
        Some(CibleOnglet::Plus)
    );
}

/// **L'onglet qu'on renomme prend la largeur du champ, et perd sa croix.**
#[test]
fn test_l_onglet_qu_on_renomme_prend_la_largeur_du_champ() {
    let mut store = Store::new("renommer");
    let b = store.add_board("B");
    let mut ui = UiState::new();
    let typo = Typography::new();
    ui.onglets.renomme = Some((b.clone(), TextEntry::new("B")));
    let tabs = layout_tabs(&store, &ui, &typo);
    let t = tabs.iter().find(|t| t.board_id == b).expect("B");
    assert!(t.fermer.is_none());
    assert!(t.width >= CHAMP * ui.scale(), "{}", t.width);
}
