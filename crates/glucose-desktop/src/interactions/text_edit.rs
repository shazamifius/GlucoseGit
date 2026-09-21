//! La session d'édition d'une annotation : ouvrir, écrire, valider.
//!
//! Le **clavier** vit dans [`keys`], la **souris** dans [`super::text_mouse`], et le modèle de
//! sélection — mouvements, frontières de mots, écriture — dans [`glucose_core::text::selection`],
//! où il se teste sans écran.
//!
//! # EDIT-1 — une session ne connaît que sa sélection
//!
//! Il n'y a pas un curseur *et* une sélection : une sélection vide **est** le curseur (SEL-1).
//! Tant que les deux existaient séparément, chaque geste devait penser à mettre les deux à
//! jour, et le premier oubli laissait un surlignage fantôme derrière le curseur.

pub mod geometry;
pub mod keys;

use crate::app::GlucoseApp;
use crate::renderer::TextEditSession;
use glucose_core::text::Selection;
use glucose_core::types::Annotation;

impl GlucoseApp {
    /// Initialise une session d'édition in-place pour une annotation, curseur à la fin.
    pub fn start_text_edit(&mut self, ann_id: String, initial_text: String) {
        let fin = initial_text.len();
        self.start_text_edit_at(ann_id, initial_text, Selection::at(fin));
    }

    /// La même, avec une sélection choisie — ce qu'un clic ou un double-clic vient de désigner.
    pub fn start_text_edit_at(
        &mut self,
        ann_id: String,
        initial_text: String,
        selection: Selection,
    ) {
        let selection = selection.clamped(&initial_text);
        self.editing_session = Some(TextEditSession {
            ann_id,
            buffer: initial_text,
            selection,
            goal_x: None,
            blink_timer: std::time::Instant::now(),
            // Un curseur qui vient de naître est allumé : la phase zéro est la phase
            // visible, et la boucle de réveil prendra le relais à la demi-seconde.
            curseur_visible: true,
        });
        self.mark_dirty();
    }

    /// Valide et persiste le texte édité dans le store.
    ///
    /// La saisie entière est **une** entrée d'undo (fiche 09 § 2.2) : `begin_live_edit`
    /// ouvre le geste, le texte puis la hauteur de la carte (TEXT-FIT-1) s'y écrivent, et
    /// `end_live_edit` le referme. Chaque écriture passe par le store, jamais par le board
    /// directement : tant que l'ouverture prenait un cliché du document, une écriture
    /// directe était couverte ; avec le journal d'éditions, elle est invisible — et c'est
    /// ainsi que la saisie de texte a cessé d'être annulable sans qu'aucun test ne le voie.
    pub fn commit_editing(&mut self) {
        self.text_drag = None;
        let Some(session) = self.editing_session.take() else {
            return;
        };
        let board = self.store.project.active_board_id.clone();
        let is_empty = session.buffer.trim().is_empty();
        let is_text_card = self
            .store
            .active_board()
            .and_then(|b| b.annotations.iter().find(|a| a.id() == session.ann_id))
            .is_some_and(|a| matches!(a, Annotation::Text { .. }));

        self.store.begin_live_edit();
        if is_empty && is_text_card {
            // Une carte de texte vidée disparaît — une suppression comme une autre.
            self.store.remove_annotations(&board, &[&session.ann_id]);
        } else {
            self.store
                .update_annotation(&board, &session.ann_id, |ann| match ann {
                    Annotation::Text { text, .. } | Annotation::Sticky { text, .. } => {
                        *text = session.buffer.clone();
                    }
                    // Une membrane et une flèche portent un texte **optionnel** : le vider,
                    // c'est le retirer, pas y ranger une chaîne vide qui se dessinerait en
                    // pastille creuse.
                    //
                    // Les quatre variantes sont couvertes, et il n'y a donc plus de bras
                    // fourre-tout : une annotation d'un nouveau genre fera échouer la
                    // compilation ici, au lieu de perdre silencieusement ce qu'on y écrit.
                    Annotation::Membrane { text, .. } | Annotation::Arrow { text, .. } => {
                        *text = if is_empty {
                            None
                        } else {
                            Some(session.buffer.clone())
                        };
                    }
                });
            self.fit_text_card_height(&session.ann_id);
        }
        self.store.end_live_edit();
    }
}
