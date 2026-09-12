//! Undo/Redo — la pile, les transactions, et la préservation de la vue.
//!
//! Le stockage est délégué à [`journal::Journal`]. Ce module ne garde que ce qui relève du
//! store : la sémantique des gestes, et l'invariant de vue.
//!
//! # INVARIANT UNDO-1
//!
//! La caméra et le board actif sont préservés à travers un undo/redo : annuler un déplacement
//! ne doit jamais téléporter l'utilisateur ailleurs dans le document.
//!
//! Il ne concerne que les entrées qui **remplacent** le document ([`journal::Step::Replaced`]).
//! Une entrée journalisée ne touche que les éléments qu'elle a modifiés : la caméra n'est pas
//! dans son périmètre, et il n'y a donc rien à rétablir — l'invariant est tenu par
//! construction plutôt que par réparation.
//!
//! # État de la migration
//!
//! Les 34 sites de mutation appellent encore [`Store::push_undo`], qui empile un snapshot.
//! Ce module les laisse intacts : seule la mécanique en dessous a changé. Chaque site migré
//! vers [`journal::Edit`] fait baisser [`Store::snapshot_count`], jusqu'à zéro.

use super::{build_folder_stack, Store};
use crate::store::journal::Step;
use crate::types::{Project, Viewport};

/// Ce qu'il faut retenir de la vue pour la rétablir après un remplacement de document.
///
/// On ne clone pas le projet pour ça : seuls le board actif et les caméras comptent, soit
/// quelques dizaines d'octets au lieu de la totalité du document.
struct View {
    active_board_id: String,
    viewports: Vec<(String, Viewport)>,
}

impl View {
    fn capture(p: &Project) -> Self {
        Self {
            active_board_id: p.active_board_id.clone(),
            viewports: p.boards.iter().map(|b| (b.id.clone(), b.viewport)).collect(),
        }
    }

    /// Réapplique la vue au document restauré (UNDO-1).
    fn restore(&self, p: &mut Project) {
        if p.boards.iter().any(|b| b.id == self.active_board_id) {
            p.active_board_id = self.active_board_id.clone();
        } else if let Some(first) = p.boards.first() {
            p.active_board_id = first.id.clone();
        }
        for b in &mut p.boards {
            if let Some((_, vp)) = self.viewports.iter().find(|(id, _)| *id == b.id) {
                b.viewport = *vp;
            }
        }
    }
}

/// Préserve la caméra et le board actif lors d'un Undo ou Redo.
///
/// Conservée pour les appelants existants ; l'implémentation passe désormais par [`View`].
pub fn preserve_view(restored: &mut Project, cur: &Project) {
    View::capture(cur).restore(restored);
}

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

    /// Nombre de gestes encore stockés sous forme de snapshot — la dette de migration.
    /// Doit atteindre zéro.
    pub fn snapshot_count(&self) -> usize {
        self.journal.snapshot_count()
    }

    /// Enregistre l'état d'avant un geste.
    ///
    /// Appelé par les 34 sites de mutation non encore migrés. Sans effet pendant une
    /// transaction ouverte : un geste continu ne produit qu'une entrée.
    pub fn push_undo(&mut self) {
        if self.journal.is_open() {
            return;
        }
        self.journal.push_snapshot(&self.project);
        self.bump_version();
    }

    /// Ouvre une transaction : tout ce qui suit jusqu'à `end_live_edit` forme un seul geste.
    ///
    /// La version n'avance pas ici : rien n'a encore changé. C'est `end_live_edit` qui la
    /// fait avancer, une fois le geste écrit ; et un geste abandonné (`cancel_live_edit`) ou
    /// resté immobile ne la touche pas — le document n'est pas « modifié » pour un clic.
    ///
    /// Le snapshot pris à l'ouverture couvre les sites pas encore migrés, dont les
    /// `push_undo` seront absorbés par la transaction. Il disparaîtra avec eux.
    pub fn begin_live_edit(&mut self) {
        if self.journal.is_open() {
            return;
        }
        self.journal.push_snapshot(&self.project);
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
        // 1. Défaire ce que la transaction avait déjà écrit.
        self.journal.cancel(&mut self.project);
        // 2. Retirer l'entrée d'ouverture et revenir à l'état d'avant le geste.
        let view = View::capture(&self.project);
        if self.journal.undo(&mut self.project) == Some(Step::Replaced) {
            view.restore(&mut self.project);
        }
        // 3. Ne laisser aucune trace : ce geste n'a pas eu lieu.
        self.journal.forget_redo();
        true
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
    ///
    /// Seul un pas qui **remplace** le document demande de rétablir la vue et de repartir
    /// d'une sélection vide : une entrée journalisée n'a touché que ses propres éléments.
    fn step(&mut self, backward: bool) -> bool {
        let view = View::capture(&self.project);
        let outcome = if backward {
            self.journal.undo(&mut self.project)
        } else {
            self.journal.redo(&mut self.project)
        };
        match outcome {
            None => false,
            Some(Step::Replaced) => {
                view.restore(&mut self.project);
                self.clear_selection();
                self.folder_stack =
                    build_folder_stack(&self.project.boards, &self.project.active_board_id);
                self.bump_version();
                true
            }
            Some(Step::Local) => {
                self.bump_version();
                true
            }
        }
    }
}
