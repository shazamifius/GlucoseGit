//! La géométrie du texte d'une carte : sa mise en page, et les points qui s'y posent.
//!
//! Le clavier en a besoin pour `↑`, `↓`, `Début` et `Fin` — qui parlent de lignes
//! **visuelles**, donc du reflux — et la souris pour convertir un clic en offset. Les deux
//! passent par ici, donc par la même mise en page que le dessin (HIT-1).
//!
//! # Le texte se donne, il ne se lit pas
//!
//! Chaque fonction reçoit le texte au lieu d'aller le chercher dans la session d'édition.
//! C'est ce qui permet de viser un mot **avant** d'ouvrir la saisie — un double-clic sur une
//! carte au repos sélectionne le mot visé, alors qu'aucune session ne le porte encore.

use crate::app::GlucoseApp;
use crate::canvas::screen_to_world;
use crate::renderer::card::{card_text_layout, text_box, TEXT_ORIGIN};
use crate::renderer::richtext::hit::offset_at;
use crate::renderer::richtext::{TextLayout, TextMode};
use glucose_core::types::Annotation;

/// Ce qu'il faut d'une carte pour y poser un point : son coin et sa largeur, en unités monde.
pub(crate) struct CardBox {
    pub origin: (f64, f64),
    pub width: f32,
}

impl GlucoseApp {
    /// La boîte d'une carte de texte du tableau actif.
    ///
    /// Les pense-bêtes et les membranes s'éditent aussi, mais leur texte ne se reflue pas de
    /// la même façon ; la sélection à la souris s'y branchera quand leur mise en page passera
    /// par [`crate::renderer::richtext`], et rendre `None` ici est la façon honnête de le dire.
    pub(crate) fn card_box(&self, id: &str) -> Option<CardBox> {
        let ann = self
            .store
            .active_board()?
            .annotations
            .iter()
            .find(|a| a.id() == id)?;
        let Annotation::Text { x, y, .. } = ann else {
            return None;
        };
        let (w, _) = ann.size()?;
        Some(CardBox {
            origin: (*x, *y),
            width: w as f32,
        })
    }

    /// La mise en page qu'aurait `text` dans la carte `id`, **dans le mode demandé**.
    ///
    /// Le mode n'est pas un détail : une carte au repos et une carte en saisie ne posent pas
    /// le même texte aux mêmes abscisses (MODE-1). Viser un mot pour ouvrir la saisie doit
    /// donc se faire dans la vue que l'utilisateur avait **sous les yeux** — la vue rendue —
    /// et non dans celle qui n'apparaîtra qu'après son geste.
    pub(crate) fn card_layout_of(
        &self,
        id: &str,
        text: &str,
        mode: TextMode,
    ) -> Option<(TextLayout, f32)> {
        let width = self.card_box(id)?.width;
        let layout = card_text_layout(
            &self.renderer.typography,
            &self.renderer.math,
            text,
            width,
            mode,
        );
        Some((layout, width))
    }

    /// La mise en page du texte en cours de saisie — **celle qui est dessinée** (MODE-1).
    pub(crate) fn editing_layout(&self) -> Option<(TextLayout, f32)> {
        let session = self.editing_session.as_ref()?;
        self.card_layout_of(&session.ann_id, &session.buffer, TextMode::Source)
    }

    /// L'offset du texte sous un point écran, pour le nœud en cours d'édition.
    pub(crate) fn offset_under(&self, screen: (f64, f64)) -> Option<usize> {
        let session = self.editing_session.as_ref()?;
        self.offset_in_card(&session.ann_id, &session.buffer, screen, TextMode::Source)
    }

    /// L'offset du texte sous un point écran, dans la carte `id` supposée porter `text`.
    ///
    /// Le point est ramené en unités monde puis rapporté au coin du texte : la mise en page
    /// ignore le zoom (CARD-1), donc un clic tombe sur le même caractère à tous les zooms.
    pub(crate) fn offset_in_card(
        &self,
        id: &str,
        text: &str,
        screen: (f64, f64),
        mode: TextMode,
    ) -> Option<usize> {
        let boite = self.card_box(id)?;
        let (layout, width) = self.card_layout_of(id, text, mode)?;
        let vp = self.store.active_board().map(|b| b.viewport)?;
        let (wx, wy) = screen_to_world(screen.0, screen.1, &vp);
        let at = (
            (wx - boite.origin.0) as f32 - TEXT_ORIGIN.0,
            (wy - boite.origin.1) as f32 - TEXT_ORIGIN.1,
        );
        Some(offset_at(
            &self.renderer.typography,
            &layout,
            text,
            at,
            &text_box(width),
        ))
    }
}
