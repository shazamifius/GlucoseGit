//! Undo/Redo — la sémantique des gestes, par-dessus le journal d'éditions.
//!
//! Le stockage est délégué à [`journal::Journal`]. Ce module ne garde que ce qui relève du
//! store : ce qui fait un geste, et les invariants qu'un pas d'annulation doit tenir.
//!
//! # INVARIANT UNDO-1 — la caméra ne bouge pas
//!
//! Annuler un déplacement ne doit jamais téléporter l'utilisateur ailleurs dans le document.
//!
//! Il n'y a **rien à faire** pour le tenir : une entrée du journal ne touche que les éléments
//! qu'elle a modifiés, et la caméra n'est pas dans son périmètre. Tant que l'undo restaurait
//! un document complet, il fallait au contraire capturer la vue avant chaque pas et la
//! réappliquer après — c'est ce que faisait `preserve_view`, partie avec le dernier snapshot.
//! L'invariant a cessé d'être une réparation pour devenir une propriété.
//!
//! # INVARIANT NAV-1 — la navigation ne pointe pas dans le vide
//!
//! Celui-là, en revanche, demande un geste : voir [`Store::settle_navigation`].

use super::{build_folder_stack, Store};

impl Store {
    /// Profondeur de la pile d'annulation, en nombre de gestes.
    pub fn undo_depth(&self) -> usize {
        self.journal.depth()
    }

    /// Nombre de gestes rétablissables.
    pub fn redo_depth(&self) -> usize {
        self.journal.redo_depth()
    }

    /// Vrai pendant un geste continu (glisser, frappe de texte).
    pub fn in_live_edit(&self) -> bool {
        self.journal.is_open()
    }

    /// Ouvre une transaction : tout ce qui suit jusqu'à `end_live_edit` forme un seul geste.
    ///
    /// La version n'avance pas ici : rien n'a encore changé. C'est `end_live_edit` qui la
    /// fait avancer, une fois le geste écrit ; et un geste abandonné (`cancel_live_edit`) ou
    /// resté immobile ne la touche pas — le document n'est pas « modifié » pour un clic.
    ///
    /// **Saisir une carte ne coûte plus rien.** Tant qu'un site savait seulement se décrire
    /// par un état complet, l'ouverture d'un geste posait un filet : un clone du document,
    /// soit 333 ms sur un million de nœuds, payées au moment même où la main se ferme sur
    /// l'objet. Tous les sites décrivant désormais ce qu'ils changent, le filet n'a plus
    /// d'objet et l'ouverture est en temps constant.
    pub fn begin_live_edit(&mut self) {
        self.journal.begin();
    }

    pub fn end_live_edit(&mut self) {
        self.journal.end();
        self.bump_version();
    }

    /// Abandonne la transaction live en cours (`Échap` pendant un geste).
    ///
    /// Le document revient à l'état d'avant `begin_live_edit`, et l'entrée posée à
    /// l'ouverture disparaît avec lui : un geste annulé ne laisse **aucune** trace dans la
    /// pile, ni à annuler ni à rétablir. La sélection est conservée — les nœuds existent
    /// toujours — et la caméra aussi (UNDO-1). La version ne bouge pas : le document est
    /// exactement celui de la version courante, que les index connaissent déjà. Rend `false`
    /// hors transaction.
    pub fn cancel_live_edit(&mut self) -> bool {
        if !self.journal.is_open() {
            return false;
        }
        // La transaction porte tout ce que le geste a écrit : la défaire suffit, et il n'y a
        // rien d'autre à retirer de la pile puisqu'un geste ouvert n'y a encore rien déposé.
        //
        // Tant qu'un filet était posé à l'ouverture, il fallait aussi le dépiler ici. Le
        // faire aujourd'hui défaireait le geste *précédent* — c'est exactement ce qu'a
        // attrapé test_cancel_live_edit_restores_the_document_and_leaves_no_undo_entry.
        self.journal.cancel(&mut self.project);
        self.settle_navigation();
        true
    }

    /// Consigne une édition, et publie la nouvelle version du document.
    ///
    /// **La version n'avance pas pendant un geste continu.** Elle dit à l'extérieur — index
    /// spatiaux, rendu, autosave — que le document a changé ; pendant un glisser, le
    /// changement n'est publié qu'au relâchement, par `end_live_edit`. Un geste abandonné en
    /// route ne doit donc laisser aucune trace, pas même un numéro de version consommé.
    pub(super) fn record_edit(&mut self, edit: crate::store::journal::Edit) {
        self.journal.record(edit);
        if !self.journal.is_open() {
            self.bump_version();
        }
    }

    /// Enregistre plusieurs éditions comme **un seul geste** annulable.
    ///
    /// Respecte une transaction déjà ouverte : pendant un glisser, tout reste un geste unique.
    /// Une liste vide ne laisse aucune trace — une suppression qui ne supprime rien n'est pas
    /// un geste.
    pub(super) fn record_as_one_gesture(&mut self, edits: Vec<crate::store::journal::Edit>) {
        if edits.is_empty() {
            return;
        }
        let already_open = self.journal.is_open();
        if !already_open {
            self.journal.begin();
        }
        for edit in edits {
            self.journal.record(edit);
        }
        if !already_open {
            self.journal.end();
            self.bump_version();
        }
    }

    /// Applique une transformation en bloc au board, et consigne ce qu'elle a changé.
    ///
    /// Les algorithmes de mise en page (`organize_board_grid`, panneau ORDONNER) déplacent
    /// des dizaines d'éléments d'un coup sans dire lesquels. Plutôt que de leur demander de
    /// se décrire, on photographie les listes avant, on les laisse travailler, et on compare.
    /// Seuls les éléments réellement modifiés entrent au journal.
    ///
    /// Le coût de la comparaison est en O(n) sur le board — ce qui est conforme à JRN-1,
    /// puisque le geste peut légitimement toucher tout le board.
    ///
    /// Si la transformation change le **nombre** d'éléments, elle sort du cadre de cette
    /// méthode : le journal est alors vidé (JRN-2) plutôt que de décrire faussement le geste.
    pub fn mutate_board_layout<F: FnOnce(&mut crate::types::Board)>(
        &mut self,
        board_id: &str,
        f: F,
    ) {
        use crate::store::journal::{Edit, Slot};

        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let bid = b.id.clone();
        let images_before = b.images.clone();
        let anns_before = b.annotations.clone();

        f(b);

        if b.images.len() != images_before.len() || b.annotations.len() != anns_before.len() {
            self.journal.clear();
            self.bump_version();
            return;
        }

        let mut edits = Vec::new();
        for (i, (was, now)) in images_before.iter().zip(b.images.iter()).enumerate() {
            if was != now {
                edits.push(Edit::Image {
                    board: bid.clone(),
                    slot: Slot::changed(i, was.clone(), now.clone()),
                });
            }
        }
        for (i, (was, now)) in anns_before.iter().zip(b.annotations.iter()).enumerate() {
            if was != now {
                edits.push(Edit::Annotation {
                    board: bid.clone(),
                    slot: Slot::changed(i, was.clone(), now.clone()),
                });
            }
        }
        self.record_as_one_gesture(edits);
    }

    pub fn can_undo(&self) -> bool {
        self.journal.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.journal.can_redo()
    }

    pub fn undo(&mut self) -> bool {
        self.step(true)
    }

    pub fn redo(&mut self) -> bool {
        self.step(false)
    }

    /// Un pas d'annulation, dans un sens ou dans l'autre.
    fn step(&mut self, backward: bool) -> bool {
        let moved = if backward {
            self.journal.undo(&mut self.project)
        } else {
            self.journal.redo(&mut self.project)
        };
        if !moved {
            return false;
        }
        self.settle_navigation();
        self.bump_version();
        true
    }

    /// INVARIANT NAV-1 — après un pas d'annulation, la navigation ne pointe jamais dans le
    /// vide : le board actif existe, et la pile de dossiers décrit le chemin qui y mène.
    ///
    /// Un snapshot restaurait le document entier, caméra et board actif compris, et tenait
    /// donc cet invariant sans avoir à le nommer. Une entrée journalisée ne touche que les
    /// listes qu'elle a modifiées : annuler la création d'un dossier dans lequel on est
    /// entre-temps descendu supprime le sous-board sans rien dire à la navigation. C'est ce
    /// que ce rétablissement couvre — et c'est un invariant du store, pas une réparation
    /// propre au journal.
    fn settle_navigation(&mut self) {
        let active_exists = self
            .project
            .boards
            .iter()
            .any(|b| b.id == self.project.active_board_id);
        if !active_exists {
            if let Some(first) = self.project.boards.first() {
                self.project.active_board_id = first.id.clone();
            }
        }
        self.folder_stack =
            build_folder_stack(&self.project.boards, &self.project.active_board_id);
    }
}
