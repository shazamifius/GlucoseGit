//! **Ce qui attend une décision** : la barre d'action, les options de flèche, le toast, le menu
//! et la question — ce qui ne paraît que sur décision de l'utilisateur (DECISION-1, fiche 59).
//!
//! Extrait de [`super::render_ui`], qui le posait avec la chrome permanente : les panneaux se
//! peignaient donc **par-dessus**, et la question du journal technique paraissait sous Ordonner
//! et Pomodoro, ses réponses cachées (son téléphone, le 07/10). Le clic, lui, descendait déjà
//! dans l'ordre juste (`interactions::mouse::handle_left_down`) : on voyait un panneau et on
//! touchait une question qu'on ne voyait pas. Le moteur de rendu pose désormais ce groupe
//! **après** les panneaux ([`crate::renderer::Renderer::poser_ce_qui_attend_une_decision`]),
//! sur les deux voies.

use super::{action_bar, context_menu, options_de_fleche, question, toast, UiState};
use crate::params::Pointer;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::store::Store;
use tiny_skia::PixmapMut;

/// La barre d'action et les options de flèche — ce que la fenêtre des ancres recouvre.
pub fn poser_les_barres(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &UiState,
    (typo, theme): (&Typography, &Theme),
) {
    if ui.reference {
        return;
    }
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);
    let visible = ui.ecran_visible((w, h));
    action_bar::draw_action_bar(
        pixmap,
        store,
        typo,
        theme,
        (visible, ui.scale_factor),
        ui.origine_du_groupe,
    );
    // Pendant l'édition des ancres, sa fenêtre couvre tout : le moteur de rendu la pose
    // juste après (`poser_la_fenetre_d_ancrage`).
    if ui.ancrage.is_none() {
        options_de_fleche::draw_options_de_fleche(
            pixmap,
            store,
            typo,
            theme,
            (visible, ui.scale_factor),
        );
    }
}

/// Le toast, le menu et la question — ce qui passe par-dessus tout, la fenêtre des ancres
/// comprise.
///
/// L'ordre entre les trois n'est pas libre : le toast doit rester lisible par-dessus la barre
/// d'action, le menu par-dessus tout puisqu'il attend qu'on choisisse, et la question
/// par-dessus le menu : ce qui est derrière attend sa réponse (QUESTION-1).
pub fn poser_les_reponses_attendues(
    pixmap: &mut PixmapMut,
    store: &Store,
    ui: &UiState,
    (typo, theme): (&Typography, &Theme),
    pointer: Pointer,
) {
    let (w, h) = (pixmap.width() as f32, pixmap.height() as f32);
    let visible = ui.ecran_visible((w, h));
    if let Some(ref t) = ui.current_toast {
        toast::render_toast(pixmap, t, typo, theme, w, visible.1, ui.scale_factor);
    }
    dessiner_le_menu(pixmap, (ui, store), (typo, theme), (w, h), pointer);
    if let Some((q, _)) = &ui.question {
        let placee = question::placer(q, typo, visible, ui.scale_factor);
        let ou = (pointer.x, pointer.y);
        question::dessiner(pixmap, &placee, (typo, theme), ou, ui.scale_factor);
    }
}

/// Le menu contextuel, s'il est ouvert.
fn dessiner_le_menu(
    pixmap: &mut PixmapMut,
    (ui, store): (&UiState, &Store),
    (typo, theme): (&Typography, &Theme),
    (w, h): (f32, f32),
    pointer: Pointer,
) {
    let Some(at) = ui.context_menu_at else {
        return;
    };
    let Some(menu) = context_menu::layout_context_menu(
        store,
        typo,
        (at, ui.onglets.menu.as_deref()),
        (w, h),
        ui.scale_factor,
        ui.menu_au_doigt,
    ) else {
        return;
    };
    context_menu::draw_context_menu(
        pixmap,
        &menu,
        typo,
        theme,
        (pointer.x, pointer.y),
        ui.scale_factor,
    );
}
