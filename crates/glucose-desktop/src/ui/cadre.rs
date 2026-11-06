//! **Les trois boutons de la fenêtre, au bout de la bande** (BARRE-1, fiche 85) : réduire,
//! agrandir, fermer, là où Windows les mettait, et à sa largeur. Monochromes, comme toute
//! l'interface (`style.md`) : la croix ne vire pas au rouge.

use super::UiState;
use crate::renderer::scale::fill_crisp;
use crate::theme::Theme;
use tiny_skia::{LineCap, Paint, PathBuilder, PixmapMut, Rect, Stroke, Transform};

/// Un bouton de la fenêtre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bouton {
    Reduire,
    Agrandir,
    Fermer,
}

/// Dans l'ordre où ils se posent, de gauche à droite.
const ORDRE: [Bouton; 3] = [Bouton::Reduire, Bouton::Agrandir, Bouton::Fermer];

/// La largeur d'un bouton de légende de Windows, à cent pour cent.
pub const LARGEUR: f32 = 46.0;

/// Ce que les boutons retirent à la bande, en pixels.
pub fn largeur_reservee(ui: &UiState) -> f32 {
    if ui.cadre_maison && !ui.sans_interface() {
        3.0 * LARGEUR * ui.scale()
    } else {
        0.0
    }
}

/// **Où est chacun** : sur la barre ; sans elle (le rail), sur la rangée des onglets.
pub fn boutons(ui: &UiState, largeur: f32) -> Vec<(Bouton, [f32; 4])> {
    if largeur_reservee(ui) <= 0.0 {
        return Vec::new();
    }
    let l = LARGEUR * ui.scale();
    let y = ui.marge_du_haut();
    let h = if ui.rail.actif {
        ui.header_height()
    } else {
        ui.topbar_height()
    } - y
        - 1.0;
    let depart = largeur - 3.0 * l;
    ORDRE
        .iter()
        .enumerate()
        .map(|(i, b)| (*b, [depart + i as f32 * l, y, l, h]))
        .collect()
}

/// Le bouton sous ce point.
pub fn sous(ui: &UiState, largeur: f32, x: f32, y: f32) -> Option<Bouton> {
    boutons(ui, largeur)
        .into_iter()
        .find(|(_, [bx, by, l, h])| x >= *bx && x < bx + l && y >= *by && y < by + h)
        .map(|(b, _)| b)
}

/// **Dessine les trois**, le survolé d'un fond plus clair.
pub fn dessiner(
    pixmap: &mut PixmapMut,
    ui: &UiState,
    theme: &Theme,
    largeur: f32,
    survole: Option<Bouton>,
) {
    let s = ui.scale();
    let mut paint = Paint::default();
    paint.set_color(theme.text_primary);
    paint.anti_alias = true;
    let stroke = Stroke {
        width: s.max(1.0),
        line_cap: LineCap::Butt,
        ..Stroke::default()
    };
    for (bouton, [x, y, l, h]) in boutons(ui, largeur) {
        if let Some(fond) = Rect::from_xywh(x, y, l, h) {
            let couleur = if survole == Some(bouton) {
                theme.border_subtle
            } else {
                theme.bg_header
            };
            fill_crisp(pixmap, fond, couleur);
        }
        // La moitié d'un pixel : un trait d'un pixel tombe alors net sur la grille.
        let c = ((x + l / 2.0).floor() + 0.5, (y + h / 2.0).floor() + 0.5);
        let r = (5.0 * s).round();
        let mut p = PathBuilder::new();
        match bouton {
            Bouton::Reduire => {
                p.move_to(c.0 - r, c.1);
                p.line_to(c.0 + r, c.1);
            }
            Bouton::Agrandir if ui.fenetre_agrandie => {
                // Restaurer : deux carrés, celui de derrière dépasse en haut à droite.
                let d = (2.0 * s).round();
                if let Some(carre) = Rect::from_ltrb(c.0 - r, c.1 - r + d, c.0 + r - d, c.1 + r) {
                    p.push_rect(carre);
                }
                p.move_to(c.0 - r + d, c.1 - r + d);
                p.line_to(c.0 - r + d, c.1 - r);
                p.line_to(c.0 + r, c.1 - r);
                p.line_to(c.0 + r, c.1 + r - d);
                p.line_to(c.0 + r - d, c.1 + r - d);
            }
            Bouton::Agrandir => {
                if let Some(carre) = Rect::from_ltrb(c.0 - r, c.1 - r, c.0 + r, c.1 + r) {
                    p.push_rect(carre);
                }
            }
            Bouton::Fermer => {
                p.move_to(c.0 - r, c.1 - r);
                p.line_to(c.0 + r, c.1 + r);
                p.move_to(c.0 + r, c.1 - r);
                p.line_to(c.0 - r, c.1 + r);
            }
        }
        if let Some(path) = p.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ui() -> UiState {
        let mut ui = UiState::new();
        ui.cadre_maison = true;
        ui
    }

    #[test]
    fn test_barre_1_les_boutons_touchent_le_bord_droit_dans_l_ordre_de_windows() {
        let ui = ui();
        let b = boutons(&ui, 1000.0);
        assert_eq!(b.len(), 3);
        assert_eq!(b[0].0, Bouton::Reduire);
        assert_eq!(b[1].0, Bouton::Agrandir);
        assert_eq!(b[2].0, Bouton::Fermer);
        let [x, _, l, _] = b[2].1;
        assert!(
            (x + l - 1000.0).abs() < 0.01,
            "fermer touche le bord : on le vise sans regarder"
        );
        assert_eq!(sous(&ui, 1000.0, 999.0, 10.0), Some(Bouton::Fermer));
        assert_eq!(sous(&ui, 1000.0, 400.0, 10.0), None);
    }

    #[test]
    fn test_barre_1_sans_cadre_a_nous_il_n_y_a_aucun_bouton() {
        let ui = UiState::new();
        assert_eq!(
            largeur_reservee(&ui),
            0.0,
            "la barre du système est là : pas de doublon"
        );
        assert!(boutons(&ui, 1000.0).is_empty());
    }

    #[test]
    fn test_barre_1_le_mode_reference_garde_son_canevas_entier() {
        let mut ui = ui();
        ui.reference = true;
        assert!(boutons(&ui, 1000.0).is_empty());
    }
}
