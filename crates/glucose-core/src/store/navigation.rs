//! Navigation : board actif, viewport, pan/zoom, dossiers.
//!
//! INVARIANT UNDO-1 — aucune de ces opérations ne pousse d'entrée d'undo (règle 3.5 des
//! standards) : la navigation n'est pas une mutation du document.

use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Board, FolderTreeNode, TemporalAnchor, Viewport};

/// Reconstruit la pile de dossiers UI menant au board actif.
pub fn build_folder_stack(boards: &[Board], active_board_id: &str) -> Vec<(String, String)> {
    let mut parent_map = std::collections::HashMap::new();
    for b in boards {
        for f in &b.folders {
            parent_map.insert(f.child_board_id.clone(), (b.id.clone(), f.id.clone()));
        }
    }
    let mut stack = Vec::new();
    let mut curr = active_board_id.to_string();
    let mut guard = 256;
    while let Some(parent) = parent_map.get(&curr) {
        if guard == 0 {
            break;
        }
        guard -= 1;
        stack.insert(0, parent.clone());
        curr = parent.0.clone();
    }
    stack
}

impl Store {
    pub fn active_board(&self) -> Option<&Board> {
        self.project
            .boards
            .iter()
            .find(|b| b.id == self.project.active_board_id)
    }

    pub fn active_board_mut(&mut self) -> Option<&mut Board> {
        let id = self.project.active_board_id.clone();
        self.project.boards.iter_mut().find(|b| b.id == id)
    }

    /// Change de board actif, ou dit pourquoi il ne l'a pas fait (R-35).
    pub fn try_set_active_board_id(&mut self, board_id: impl Into<String>) -> CoreResult<()> {
        let bid = board_id.into();
        if !self.project.boards.iter().any(|b| b.id == bid) {
            return Err(CoreError::BoardNotFound(bid));
        }
        self.project.active_board_id = bid.clone();
        self.folder_stack = build_folder_stack(&self.project.boards, &bid);
        self.clear_selection();
        Ok(())
    }

    /// Enveloppe silencieuse de [`Store::try_set_active_board_id`].
    ///
    /// NON marquée `#[deprecated]` — contrairement aux autres enveloppes R-35 — parce que
    /// `glucose-desktop` l'appelle et que le dépôt exige `clippy -D warnings` à zéro
    /// avertissement. À migrer vers `try_set_active_board_id` en même temps que ses appelants.
    pub fn set_active_board_id(&mut self, board_id: impl Into<String>) {
        drop(self.try_set_active_board_id(board_id));
    }

    /// Pose la caméra d'un board. Le viewport est ramené dans le domaine du modèle
    /// ([`Viewport::normalized`]) : c'est ici, et dans [`Store::zoom`], que la borne d'échelle
    /// de la fiche 09 § 1 devient une propriété du store plutôt qu'une précaution d'appelant.
    pub fn set_viewport(&mut self, board_id: &str, vp: Viewport) {
        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            b.viewport = vp.normalized();
        }
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        if let Some(b) = self.active_board_mut() {
            b.viewport.x += dx;
            b.viewport.y += dy;
        }
    }

    /// Zoom ancré sous `(cursor_x, cursor_y)` sur le board actif — la formule est celle de
    /// [`Viewport::zoom_at`], et `range` la borne de l'appelant (le geste molette a la
    /// sienne, fiche 07 § 7.1) ; celle du modèle s'applique toujours.
    pub fn zoom(&mut self, factor: f64, cursor_x: f64, cursor_y: f64, range: (f64, f64)) {
        if let Some(b) = self.active_board_mut() {
            b.viewport.zoom_at(factor, cursor_x, cursor_y, range);
        }
    }

    /// Entre dans un dossier du board actif, ou dit pourquoi il n'a pas pu (R-35).
    pub fn try_enter_folder(&mut self, folder_id: &str) -> CoreResult<()> {
        let current_board_id = self.project.active_board_id.clone();
        let target = self
            .active_board()
            .and_then(|b| b.folders.iter().find(|f| f.id == folder_id))
            .map(|f| f.child_board_id.clone())
            .ok_or_else(|| CoreError::FolderNotFound(folder_id.to_string()))?;

        self.folder_stack
            .push((current_board_id, folder_id.to_string()));
        self.project.active_board_id = target;
        self.clear_selection();
        Ok(())
    }

    /// Enveloppe silencieuse de [`Store::try_enter_folder`].
    #[deprecated(note = "R-35 : l'échec est silencieux. Utiliser `try_enter_folder`.")]
    pub fn enter_folder(&mut self, folder_id: &str) {
        drop(self.try_enter_folder(folder_id));
    }

    pub fn exit_folder(&mut self) {
        if let Some((parent_board_id, _)) = self.folder_stack.pop() {
            self.project.active_board_id = parent_board_id;
            self.clear_selection();
        }
    }

    /// Le chemin courant, de la racine au dossier ouvert : `["Projet", "Recherches", "Notes"]`.
    ///
    /// C'est ce que le fil d'Ariane affiche. Le premier segment est le projet lui-même — on est
    /// toujours quelque part —, les suivants sont les dossiers traversés, dans l'ordre. Un
    /// dossier dont le nom a disparu du tableau parent devient `"?"` plutôt que de faire
    /// disparaître le segment : un chemin à trou serait pire qu'un chemin incertain.
    pub fn folder_path(&self) -> Vec<String> {
        let mut chemin = vec![self.project.name.clone()];
        for (parent_board_id, folder_id) in &self.folder_stack {
            let nom = self
                .project
                .boards
                .iter()
                .find(|b| &b.id == parent_board_id)
                .and_then(|b| b.folders.iter().find(|f| &f.id == folder_id))
                .map(|f| f.name.clone())
                .unwrap_or_else(|| "?".to_string());
            chemin.push(nom);
        }
        chemin
    }

    /// Remonte jusqu'à la profondeur `depth` : 0 est la racine, 1 le premier dossier.
    ///
    /// Rend `true` si le tableau actif a changé. Remonter à une profondeur égale ou supérieure
    /// à la profondeur courante ne fait rien — cliquer sur le segment où l'on est déjà n'est
    /// pas une erreur, c'est un geste sans effet.
    pub fn exit_to_depth(&mut self, depth: usize) -> bool {
        let mut bouge = false;
        while self.folder_stack.len() > depth {
            let avant = self.project.active_board_id.clone();
            self.exit_folder();
            bouge |= self.project.active_board_id != avant;
        }
        bouge
    }

    pub fn exit_to_root(&mut self) {
        if let Some((root_board_id, _)) = self.folder_stack.first().cloned() {
            self.folder_stack.clear();
            self.project.active_board_id = root_board_id;
            self.clear_selection();
        }
    }

    pub fn expand_folder(
        &mut self,
        _parent_board_id: &str,
        folder_id: &str,
        level: FolderTreeNode,
    ) {
        let child_board_id = self
            .project
            .boards
            .iter()
            .find_map(|b| b.folders.iter().find(|f| f.id == folder_id))
            .map(|f| f.child_board_id.clone());

        if let Some(child_id) = child_board_id {
            if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == child_id) {
                b.annotations.extend(level.annotations);
                b.images.extend(level.images);
            }
        }
    }

    pub fn set_temporal_filter(&mut self, filter: Option<TemporalAnchor>) {
        self.temporal_filter = filter;
    }

    /// La boîte englobante de tout ce qu'un tableau contient — images, annotations et dossiers
    /// — en unités monde. `None` si le tableau est vide ou n'existe pas.
    ///
    /// # Pourquoi cette méthode appartient au `Store`
    ///
    /// Trois endroits en avaient besoin et se la calculaient chacun : le cadrage d'un banc, la
    /// mise à l'échelle de la minimap, et la touche `F` — qui, faute de l'avoir, se contentait
    /// de remettre la caméra à l'origine sans rien cadrer (fiche 03 § 1.6). Savoir où est le
    /// contenu est une question qu'on pose au document, pas une géométrie qu'on refait.
    ///
    /// Chaque nœud compte par sa boîte telle que le modèle la définit — une image par son
    /// rectangle centré, une carte par sa taille de naissance si le document n'en fixe pas,
    /// une flèche par l'enveloppe de son tracé. Ce qui est dessiné est dedans, rien de plus.
    ///
    /// # Invariant BORNES-1 — la réponse est gardée tant que le document ne bouge pas
    ///
    /// La minimap la demande **à chaque image** pour cadrer une vignette de 180 × 120 pixels,
    /// et la réponse ne dépend que du document. Mesuré à un million de nœuds : 16,9 ms sur les
    /// 18,3 d'une image de navigation, pendant que le culling et les quatre passes de scène
    /// réunis tenaient dans 0,01 ms. Autrement dit, tout le coût de se déplacer dans Glucose
    /// était ici, et nulle part ailleurs.
    ///
    /// Le repère est `(version, tableau)` : toute mutation fait avancer `version` (R-39), donc
    /// aucune réponse périmée ne peut être rendue. Le parcours reste O(n) — il a simplement
    /// cessé d'être par image pour redevenir par mutation.
    ///
    /// Pourquoi pas un maintien incrémental : une union de rectangles s'étend en O(1) mais ne
    /// se rétracte pas, faute d'inverse. Après avoir tout supprimé sauf un nœud, la minimap
    /// resterait cadrée sur un document disparu. Maintenir les quatre extrema sous suppression
    /// demanderait un multiset ordonné — un O(log n) par mutation pour économiser un O(n) par
    /// mutation, ce qui n'est pas un gain. Le calcul paresseux est exact et n'invente rien.
    pub fn content_bounds(&self, board_id: &str) -> Option<crate::geometry::Rect> {
        let mut cache = self.bornes.borrow_mut();
        if !cache.repond_pour(self.version, board_id) {
            cache.parcours += 1;
            cache.repere = Some((self.version, board_id.to_string()));
            cache.valeur = self.calculer_les_bornes(board_id);
        }
        cache.valeur
    }

    /// Le nombre de parcours réels du document faits par [`Self::content_bounds`].
    ///
    /// Sans ce compteur, la garde de BORNES-1 ne se prouverait qu'en chronométrant — donc pas
    /// de façon déterministe. Avec lui, « se déplacer ne reparcourt pas le document » est une
    /// égalité.
    pub fn parcours_des_bornes(&self) -> usize {
        self.bornes.borrow().parcours
    }

    fn calculer_les_bornes(&self, board_id: &str) -> Option<crate::geometry::Rect> {
        let board = self.project.boards.iter().find(|b| b.id == board_id)?;
        board
            .images
            .iter()
            .map(|img| img.rect())
            .chain(board.folders.iter().map(|f| f.rect()))
            .chain(board.annotations.iter().map(|a| a.bounds()))
            .reduce(|acc, r| acc.union(r))
    }
}

/// La dernière réponse de [`Store::content_bounds`], et le repère qui la rend valable.
///
/// Voir l'invariant BORNES-1 sur [`Store::content_bounds`].
#[derive(Debug, Clone, Default)]
pub(super) struct BornesDuContenu {
    /// `(version du document, tableau interrogé)`. `None` tant que rien n'a été demandé.
    repere: Option<(u64, String)>,
    valeur: Option<crate::geometry::Rect>,
    /// Compte les parcours réels : ce qui rend la garde testable au lieu de supposée.
    parcours: usize,
}

impl BornesDuContenu {
    fn repond_pour(&self, version: u64, board_id: &str) -> bool {
        matches!(&self.repere, Some((v, id)) if *v == version && id == board_id)
    }
}
