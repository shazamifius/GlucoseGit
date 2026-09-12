//! Navigation : board actif, viewport, pan/zoom, dossiers.
//!
//! INVARIANT UNDO-1 — aucune de ces opérations ne pousse d'entrée d'undo (règle 3.5 des
//! standards) : la navigation n'est pas une mutation du document.

use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Annotation, Board, FolderTreeNode, TemporalAnchor, Viewport};

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
    /// Une flèche compte par ses **deux** extrémités : son point d'ancrage ne dit rien de
    /// l'endroit qu'elle occupe.
    pub fn content_bounds(&self, board_id: &str) -> Option<crate::geometry::Rect> {
        let board = self.project.boards.iter().find(|b| b.id == board_id)?;
        let mut bounds: Option<(f64, f64, f64, f64)> = None;
        let mut etendre = |x0: f64, y0: f64, x1: f64, y1: f64| {
            bounds = Some(match bounds {
                None => (x0, y0, x1, y1),
                Some((ax, ay, bx, by)) => (ax.min(x0), ay.min(y0), bx.max(x1), by.max(y1)),
            });
        };

        for img in &board.images {
            etendre(img.x, img.y, img.x + img.width, img.y + img.height);
        }
        for f in &board.folders {
            etendre(f.x, f.y, f.x + f.width, f.y + f.height);
        }
        for a in &board.annotations {
            match a {
                Annotation::Arrow { x, y, x2, y2, .. } => {
                    etendre(x.min(*x2), y.min(*y2), x.max(*x2), y.max(*y2));
                }
                Annotation::Membrane { x, y, width, height, .. } => {
                    etendre(*x, *y, x + width, y + height);
                }
                Annotation::Text { x, y, width, height, .. }
                | Annotation::Sticky { x, y, width, height, .. } => {
                    // Une carte sans taille explicite occupe au moins son point : mieux vaut
                    // une boîte un peu petite qu'un cadrage qui invente des dimensions.
                    etendre(*x, *y, x + width.unwrap_or(0.0), y + height.unwrap_or(0.0));
                }
            }
        }

        bounds.map(|(x0, y0, x1, y1)| crate::geometry::Rect::new(x0, y0, x1 - x0, y1 - y0))
    }
}
