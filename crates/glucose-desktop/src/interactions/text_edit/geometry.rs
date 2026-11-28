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
use crate::renderer::composants::titre::mesures_du_monde;
use crate::renderer::curseur;
use crate::renderer::note::{sticky_text_box, STICKY_TEXT_ORIGIN};
use crate::renderer::richtext::brut::layout_brut;
use crate::renderer::richtext::hit::offset_at;
use crate::renderer::richtext::{TextBox, TextLayout, TextMode};
use glucose_core::text::Emphasis;
use glucose_core::types::Annotation;

/// Ce qu'il faut d'un nœud pour y poser un point : son coin, sa largeur, la boîte de son texte
/// et le coin de celui-ci, en unités monde — et, s'il est brut, l'emphase de toutes ses lettres
/// (le romain d'un post-it, le gras d'un titre : fiche 188).
pub(crate) struct CardBox {
    pub origin: (f64, f64),
    pub width: f32,
    pub corps: TextBox,
    pub marge: (f32, f32),
    pub brut: Option<Emphasis>,
    /// Sous cette ordonnée du monde, un clic ne vise plus le texte : le bord haut d'une membrane,
    /// dont le titre se tient au-dessus — un clic dans son corps n'est pas dans son nom.
    pub plancher: Option<f64>,
}

impl GlucoseApp {
    /// La boîte d'un nœud dont on choisit le texte `text` : un nœud de texte, un post-it ou le titre
    /// d'une membrane (fiche 188 ; sa note du 27/11, `text-n3rk-2587` : on n'y pouvait choisir aucun
    /// mot). Le titre se mesure sur le texte qu'il porte : un titre long rétrécit pour tenir dans
    /// sa membrane. Un post-it devenu opérateur (une pilule) n'a qu'un mot : `None`, la façon
    /// honnête de dire qu'aucune géométrie de texte ne le décrit.
    pub(crate) fn card_box(&self, id: &str, text: &str) -> Option<CardBox> {
        let ann = self
            .store
            .active_board()?
            .annotations
            .iter()
            .find(|a| a.id() == id)?;
        let (w, h) = ann.size()?;
        let width = w as f32;
        let (origin, corps, marge, brut, plancher) = match ann {
            Annotation::Text { x, y, .. } => ((*x, *y), text_box(width), TEXT_ORIGIN, None, None),
            Annotation::Sticky {
                x,
                y,
                operator: None,
                ..
            } => (
                (*x, *y),
                sticky_text_box(width),
                STICKY_TEXT_ORIGIN,
                Some(Emphasis::NONE),
                None,
            ),
            Annotation::Membrane { x, y, .. } => {
                let typo = &self.renderer.typography;
                let [corps, retrait, elevation] = mesures_du_monde(typo, text, (width, h as f32));
                let bande = TextBox {
                    usable: f32::INFINITY,
                    body: corps,
                    bullet_indent: 0.0,
                    line_height: corps * curseur::HAUTEUR,
                };
                let marge = (retrait, -elevation);
                ((*x, *y), bande, marge, Some(Emphasis::BOLD), Some(*y))
            }
            _ => return None,
        };
        Some(CardBox {
            origin,
            width,
            corps,
            marge,
            brut,
            plancher,
        })
    }

    /// La mise en page qu'aurait `text` dans le nœud `id`, **dans le mode demandé**.
    ///
    /// Le mode n'est pas un détail : une carte au repos et une carte en saisie ne posent pas
    /// le même texte aux mêmes abscisses (MODE-1). Viser un mot pour ouvrir la saisie doit
    /// donc se faire dans la vue que l'utilisateur avait **sous les yeux** — la vue rendue —
    /// et non dans celle qui n'apparaîtra qu'après son geste. Un texte brut n'a qu'une vue.
    pub(crate) fn card_layout_of(
        &self,
        id: &str,
        text: &str,
        mode: TextMode,
    ) -> Option<(TextLayout, TextBox)> {
        let boite = self.card_box(id, text)?;
        let typo = &self.renderer.typography;
        let layout = match boite.brut {
            Some(emphase) => layout_brut(typo, text, boite.corps, emphase),
            None => card_text_layout(typo, &self.renderer.math, text, boite.width, mode),
        };
        Some((layout, boite.corps))
    }

    /// La mise en page du texte en cours de saisie — **celle qui est dessinée** (MODE-1).
    pub(crate) fn editing_layout(&self) -> Option<(TextLayout, TextBox)> {
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
        let boite = self.card_box(id, text)?;
        let (layout, corps) = self.card_layout_of(id, text, mode)?;
        let vp = self.store.active_board().map(|b| b.viewport)?;
        let (wx, wy) = screen_to_world(screen.0, screen.1, &vp);
        if boite.plancher.is_some_and(|bord| wy > bord) {
            return None;
        }
        let at = (
            (wx - boite.origin.0) as f32 - boite.marge.0,
            (wy - boite.origin.1) as f32 - boite.marge.1,
        );
        Some(offset_at(
            &self.renderer.typography,
            &layout,
            text,
            at,
            &corps,
        ))
    }
}
