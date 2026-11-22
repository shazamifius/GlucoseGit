//! Le dessin du tiroir DISCUSSION : le titre et qui réfléchit, le fil, le champ, le pied.
//!
//! L'encre dit qui parle : l'IA en encre pleine, l'utilisateur en encre secondaire, les gestes de
//! l'IA en encre discrète, une bascule en **ambre** (l'avertissement de sa demande du 21/11), une
//! erreur en rouge. Pas d'émoji : la police n'en a pas (R-51).

use super::{layout_discussion_panel, DiscussionUi, Genre, AUTEUR, CORPS, LIGNE};
use crate::dock::paint::{Brush, ButtonLook};
use crate::dock::WidgetRect;
use crate::params::ScaledRect;
use crate::typography::Face;
use tiny_skia::PixmapMut;

/// Ce que dit le champ vide.
pub const INVITE: &str = "Écris à l'IA…";
/// La phrase de pied : où part ce qu'elle lit (C1, fiche 116).
pub const PIED: &str = "Ce que l'IA lit de ce document part chez Google.";

pub fn render_discussion_panel(
    pixmap: &mut PixmapMut,
    brush: &Brush,
    frame: ScaledRect,
    ui: &DiscussionUi,
) {
    let layout = layout_discussion_panel(frame, ui);
    let theme = brush.theme;
    let marge = brush.px(14.0);
    let titre = (frame.x + marge, frame.y + brush.px(22.0));
    brush.text(
        pixmap,
        "DISCUSSION",
        titre,
        10.0,
        theme.text_muted,
        Face::Bold,
    );
    if let Some(modele) = &ui.en_cours {
        let dit = format!("{modele} réfléchit…");
        let large = brush
            .typo
            .measure_text(&dit, brush.px(9.0), Face::Regular)
            .0;
        let x = frame.x + frame.w - marge - large;
        brush.text(
            pixmap,
            &dit,
            (x, titre.1),
            9.0,
            theme.success,
            Face::Regular,
        );
    }
    for (k, ligne) in ui.lignes.iter().enumerate() {
        let y = layout.fil.y + k as f32 * brush.px(LIGNE);
        let (taille, encre, face) = match ligne.genre {
            Genre::Auteur => (AUTEUR, theme.text_muted, Face::Bold),
            Genre::Moi => (CORPS, theme.text_secondary, Face::Regular),
            Genre::Ia => (CORPS, theme.text_primary, Face::Regular),
            Genre::Geste => (CORPS, theme.text_muted, Face::Regular),
            Genre::Note => (CORPS, theme.temps.ambre, Face::Regular),
            Genre::Erreur => (CORPS, theme.danger, Face::Regular),
        };
        brush.text(pixmap, &ligne.texte, (layout.fil.x, y), taille, encre, face);
    }
    champ(pixmap, brush, layout.champ, ui);
    let libelle = if ui.en_cours.is_some() {
        "Arrêter"
    } else {
        "Envoyer"
    };
    let vide = ui.entree.texte.trim().is_empty();
    let look = ButtonLook {
        disabled: ui.en_cours.is_none() && vide,
        ..ButtonLook::default()
    };
    brush.button(pixmap, layout.bouton, libelle, 10.5, look);
    brush.text(
        pixmap,
        PIED,
        layout.pied,
        8.5,
        theme.text_muted,
        Face::Regular,
    );
}

/// **Le champ** : la fin de ce qu'on écrit jusqu'au curseur, le reste tant qu'il tient, et le
/// curseur quand le clavier y va ; l'invite quand il est vide.
fn champ(pixmap: &mut PixmapMut, brush: &Brush, rect: WidgetRect, ui: &DiscussionUi) {
    let theme = brush.theme;
    brush.fill(pixmap, rect, brush.px(3.0), theme.input_bg);
    if ui.tape {
        brush.stroke(
            pixmap,
            rect,
            brush.px(3.0),
            theme.border_accent,
            brush.px(1.0),
        );
    }
    let x = rect.x + brush.px(8.0);
    let y = rect.y + (rect.h - brush.px(CORPS) * 1.2) / 2.0;
    let place = rect.w - brush.px(16.0);
    let large = |t: &str| brush.typo.measure_text(t, brush.px(CORPS), Face::Regular).0;
    let texte = &ui.entree.texte;
    if texte.is_empty() && !ui.tape {
        brush.text(
            pixmap,
            INVITE,
            (x, y),
            CORPS,
            theme.text_muted,
            Face::Regular,
        );
        return;
    }
    let tete = ui.entree.selection.head.min(texte.len());
    let (avant, apres) = visible(&texte[..tete], texte, &large, place);
    brush.text(
        pixmap,
        &format!("{avant}{apres}"),
        (x, y),
        CORPS,
        theme.text_primary,
        Face::Regular,
    );
    if ui.tape {
        let curseur = WidgetRect::new(x + large(avant), y, brush.px(1.0), brush.px(CORPS) * 1.2);
        brush.fill(pixmap, curseur, 0.0, theme.text_primary);
    }
}

/// **Ce qui se voit du champ** : la plus longue fin du texte avant le curseur qui tient, puis ce
/// qui le suit tant qu'il tient encore.
pub fn visible<'a>(
    avant: &'a str,
    tout: &'a str,
    large: &dyn Fn(&str) -> f32,
    place: f32,
) -> (&'a str, &'a str) {
    let debut = avant
        .char_indices()
        .map(|(i, _)| i)
        .find(|&i| large(&avant[i..]) <= place)
        .unwrap_or(avant.len());
    let vu = &avant[debut..];
    let reste = &tout[avant.len()..];
    let fin = reste
        .char_indices()
        .map(|(i, c)| i + c.len_utf8())
        .take_while(|&i| large(vu) + large(&reste[..i]) <= place)
        .last()
        .unwrap_or(0);
    (vu, &reste[..fin])
}
