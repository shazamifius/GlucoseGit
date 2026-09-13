//! La souris dans un texte : poser un curseur, glisser, prendre un mot, étendre au `Maj`.
//!
//! # MOUSE-1 — le nombre de clics choisit la maille, et le glisser la garde
//!
//! Un clic pose un curseur, deux prennent un mot, trois un paragraphe. Ce n'est pas trois
//! gestes mais **un seul à trois mailles** ([`Granularity`]) : le glisser qui suit n'a rien de
//! particulier à savoir, il rappelle [`expand`] avec la maille du clic qui l'a ouvert.
//!
//! C'est ce qui fait qu'un glisser entamé par un double-clic continue de prendre des mots
//! entiers, dans les deux sens, sans jamais en laisser un à moitié derrière lui.
//!
//! # `Maj` étend, il ne repose pas
//!
//! `Maj+clic` déplace la tête et **laisse l'ancre où elle est** (SEL-1) : c'est ainsi qu'on
//! sélectionne d'un point à un autre sans avoir à glisser d'un trait entre les deux — le
//! geste que la main a appris dans un navigateur.

use crate::app::GlucoseApp;
use crate::renderer::richtext::TextMode;
use glucose_core::text::selection::{expand, Granularity, Selection};

/// Un glisser de sélection de texte en cours.
#[derive(Debug, Clone, Copy)]
pub struct TextDrag {
    /// L'octet où le geste a commencé — jamais déplacé tant que le bouton est enfoncé.
    pub anchor: usize,
    /// La maille du clic qui a ouvert le geste.
    pub granularity: Granularity,
}

/// La maille que `count` clics rapprochés demandent. Au-delà de trois, on recommence à un —
/// comme dans un navigateur, où un quatrième clic repose un curseur.
pub fn granularity_of(count: u32) -> Granularity {
    match count % 3 {
        2 => Granularity::Word,
        0 => Granularity::Paragraph,
        _ => Granularity::Char,
    }
}

impl GlucoseApp {
    /// Un clic sur le nœud **déjà en édition** : il vise du texte, pas le nœud.
    ///
    /// Rend `false` si le clic n'a rien à voir avec une saisie — c'est alors un clic de canevas
    /// ordinaire, et l'appelant reprend la main.
    pub fn click_text_at(&mut self, screen: (f64, f64), count: u32, extend: bool) -> bool {
        let Some(offset) = self.offset_under(screen) else {
            return false;
        };
        let Some(session) = self.editing_session.as_mut() else {
            return false;
        };
        let granularity = granularity_of(count);
        // `Maj` garde l'ancre posée ; un clic neuf la pose là où l'on vient de cliquer.
        let anchor = if extend {
            session.selection.anchor
        } else {
            offset
        };
        session.selection = expand(&session.buffer, anchor, offset, granularity);
        session.blink_timer = std::time::Instant::now();
        self.text_drag = Some(TextDrag {
            anchor,
            granularity,
        });
        self.mark_dirty();
        true
    }

    /// Le curseur bouge, bouton enfoncé : la tête suit, l'ancre reste.
    pub fn drag_text_to(&mut self, screen: (f64, f64)) -> bool {
        let Some(drag) = self.text_drag else {
            return false;
        };
        let Some(offset) = self.offset_under(screen) else {
            return true;
        };
        let Some(session) = self.editing_session.as_mut() else {
            return false;
        };
        let suivante = expand(&session.buffer, drag.anchor, offset, drag.granularity);
        if suivante != session.selection {
            session.selection = suivante;
            session.blink_timer = std::time::Instant::now();
            self.mark_dirty();
        }
        true
    }

    /// Le bouton est relâché : le glisser se termine, la sélection reste.
    pub fn end_text_drag(&mut self) {
        self.text_drag = None;
    }

    /// La sélection que le prochain clic devrait poser sur un nœud qu'on ouvre à l'édition.
    ///
    /// Ouvrir une carte au double-clic sélectionne le mot visé — c'est ce que fait un
    /// traitement de texte, et c'est ce qui permet de remplacer un mot d'un seul geste.
    pub fn selection_opening_at(&self, id: &str, text: &str, screen: (f64, f64)) -> Selection {
        // **La vue qu'on avait sous les yeux**, pas celle qui va s'ouvrir : au repos les signes
        // du Markdown n'occupent aucune place, et viser dans la mise en page de la saisie
        // décalerait le mot désigné de toute la largeur des signes qui le précèdent.
        match self.offset_in_card(id, text, screen, TextMode::Rendered) {
            Some(offset) => expand(text, offset, offset, Granularity::Word),
            // Sans mise en page — le nœud n'est pas une carte de texte — le curseur va à la
            // fin, ce qui reste le geste le plus utile sur un pense-bête court.
            None => Selection::at(text.len()),
        }
    }
}

#[cfg(test)]
mod tests;
