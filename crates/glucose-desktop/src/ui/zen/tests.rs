//! ZEN-1, l'indicateur : sa place et son dessin.

use super::*;
use crate::params::Pointer;
use crate::ui::decision::poser_les_reponses_attendues;
use tiny_skia::Pixmap;

const ECRAN: (f32, f32) = (1440.0, 900.0);

/// **En haut à droite, dans l'écran, une cible de doigt** : la prise fait au moins 48 points de
/// côté et contient le libellé ; sous la barre d'état d'un téléphone, elle descend d'autant.
#[test]
fn zen_1_l_indicateur_en_haut_a_droite() {
    let typo = Typography::new();
    let mut ui = UiState::new();
    let i = placer(&ui, &typo, ECRAN);
    let (x, y, l, h) = i.prise;
    let s = ui.scale();
    assert!(
        x + l <= ECRAN.0 && x > ECRAN.0 / 2.0 && y >= 0.0 && y < ECRAN.1 / 4.0,
        "{i:?}"
    );
    assert!(
        l >= CIBLE_DU_DOIGT * s && h >= CIBLE_DU_DOIGT * s,
        "une cible de doigt"
    );
    assert!(sous_la_main(&i, i.libelle), "le libellé est dans la prise");
    ui.marges.haut = 40.0;
    let plus_bas = placer(&ui, &typo, ECRAN);
    assert_eq!(
        plus_bas.libelle.1,
        i.libelle.1 + 40.0,
        "sous la barre d'état"
    );
}

/// **Il se dessine en mode zen, et seulement dans sa prise** ; hors du mode, rien.
#[test]
fn zen_1_l_indicateur_se_dessine_dans_sa_prise() {
    let typo = Typography::new();
    let theme = Theme::dark();
    let store = glucose_core::store::Store::new("zen");
    let rendre = |zen: bool| {
        let mut ui = UiState::new();
        ui.current_toast = None;
        ui.zen = zen;
        let mut p = Pixmap::new(ECRAN.0 as u32, ECRAN.1 as u32).expect("pixmap");
        p.fill(theme.bg_canvas);
        let dehors = Pointer { x: -1.0, y: -1.0 };
        poser_les_reponses_attendues(&mut p.as_mut(), &store, &ui, (&typo, &theme), dehors);
        (p, ui)
    };
    let (sans, _) = rendre(false);
    let (avec, ui) = rendre(true);
    let i = placer(&ui, &typo, ECRAN);
    let (mut dedans, mut dehors) = (0, 0);
    for y in 0..avec.height() {
        for x in 0..avec.width() {
            if avec.pixel(x, y) != sans.pixel(x, y) {
                if sous_la_main(&i, (x as f32, y as f32)) {
                    dedans += 1;
                } else {
                    dehors += 1;
                }
            }
        }
    }
    assert!(dedans > 0, "le libellé se dessine");
    assert_eq!(dehors, 0, "rien hors de sa prise");
}
