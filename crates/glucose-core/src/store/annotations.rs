//! Mutations réversibles portant sur les annotations, leurs miroirs et les panneaux storyboard.

use super::images::translate_annotation;
use super::journal::{Edit, Slot};
use super::Store;
use crate::error::{CoreError, CoreResult};
use crate::types::{Annotation, StoryboardPanel};
use std::collections::HashSet;

impl Store {
    pub fn sync_annotation_size(&mut self, board_id: &str, ann_id: &str, width: f64, height: f64) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let Some(ann) = b.annotations.iter_mut().find(|a| a.id() == ann_id) else {
            return;
        };
        match ann {
            Annotation::Text {
                width: w,
                height: h,
                ..
            }
            | Annotation::Sticky {
                width: w,
                height: h,
                ..
            } => {
                *w = Some(width);
                *h = Some(height);
            }
            Annotation::Membrane {
                width: w,
                height: h,
                ..
            } => {
                *w = width;
                *h = height;
            }
            Annotation::Arrow { .. } => {}
        }
    }

    /// **Site migre vers le journal.**
    pub fn add_annotation(&mut self, board_id: &str, ann: Annotation) {
        let id = ann.id().to_string();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let index = b.annotations.len();
        b.annotations.push(ann.clone());

        self.record_edit(Edit::Annotation {
            board: board_id.to_string(),
            slot: Slot::inserted(index, ann),
        });
        self.select_annotation(id, false);
    }

    /// Modifie une annotation, et fait suivre les fleches qui y sont attachees (R-12).
    ///
    /// **Site migre vers le journal.** L'annotation modifiee et chaque fleche deplacee
    /// forment un seul geste : un Ctrl+Z remet le tout. Une fleche n'est clonee que si elle
    /// est effectivement attachee, donc le cout suit le nombre de voisins et non la taille
    /// du board.
    pub fn update_annotation<F: FnOnce(&mut Annotation)>(
        &mut self,
        board_id: &str,
        id: &str,
        f: F,
    ) {
        let mut edits = Vec::new();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let bid = b.id.clone();
        let Some(i) = b.annotations.iter().position(|a| a.id() == id) else {
            return;
        };

        let before = b.annotations[i].clone();
        f(&mut b.annotations[i]);
        let after = b.annotations[i].clone();
        let (nx, ny) = (after.x(), after.y());
        edits.push(Edit::Annotation {
            board: bid.clone(),
            slot: Slot::changed(i, before, after),
        });

        // Suivi des fleches connectees (R-12).
        for (j, a) in b.annotations.iter_mut().enumerate() {
            let attached = match a {
                Annotation::Arrow {
                    source_id,
                    target_id,
                    ..
                } => source_id.as_deref() == Some(id) || target_id.as_deref() == Some(id),
                _ => false,
            };
            if !attached {
                continue;
            }
            let before = a.clone();
            if let Annotation::Arrow {
                source_id,
                target_id,
                x,
                y,
                x2,
                y2,
                ..
            } = a
            {
                if source_id.as_deref() == Some(id) {
                    *x = nx;
                    *y = ny;
                }
                if target_id.as_deref() == Some(id) {
                    *x2 = nx;
                    *y2 = ny;
                }
            }
            edits.push(Edit::Annotation {
                board: bid.clone(),
                slot: Slot::changed(j, before, a.clone()),
            });
        }

        self.record_as_one_gesture(edits);
    }

    /// Supprime des annotations, et avec elles les fleches devenues orphelines.
    ///
    /// **Site migre vers le journal.** Les deux passes de filtrage d'origine sont fusionnees
    /// en un seul parcours arriere : une annotation part si elle est visee, ou si c'est une
    /// fleche dont une extremite l'est. Parcourir a l'envers rend les index valides a la
    /// reinsertion (voir `remove_images`).
    pub fn remove_annotations(&mut self, _board_id: &str, ids: &[&str]) {
        let id_set: HashSet<&str> = ids.iter().copied().collect();
        let doomed = |a: &Annotation| {
            id_set.contains(a.id())
                || match a {
                    Annotation::Arrow {
                        source_id,
                        target_id,
                        ..
                    } => {
                        source_id
                            .as_ref()
                            .is_some_and(|s| id_set.contains(s.as_str()))
                            || target_id
                                .as_ref()
                                .is_some_and(|t| id_set.contains(t.as_str()))
                    }
                    _ => false,
                }
        };

        let mut edits = Vec::new();
        for b in &mut self.project.boards {
            for i in (0..b.annotations.len()).rev() {
                if doomed(&b.annotations[i]) {
                    let ann = b.annotations.remove(i);
                    edits.push(Edit::Annotation {
                        board: b.id.clone(),
                        slot: Slot::removed(i, ann),
                    });
                }
            }
        }

        self.record_as_one_gesture(edits);
        self.selected_annotation_ids.clear();
    }

    /// Crée un miroir d'une annotation, ou dit pourquoi il n'a pas pu (R-35).
    pub fn try_mirror_annotation(
        &mut self,
        board_id: &str,
        id: &str,
        x: f64,
        y: f64,
    ) -> CoreResult<String> {
        let source = self
            .project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .ok_or_else(|| CoreError::BoardNotFound(board_id.to_string()))?
            .annotations
            .iter()
            .find(|a| a.id() == id)
            .ok_or_else(|| CoreError::AnnotationNotFound(id.to_string()))?
            .clone();

        let mid = self.generate_id("mirror");
        let mirror = build_mirror(source, &mid, id, x, y);

        if let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) {
            let index = b.annotations.len();
            b.annotations.push(mirror.clone());
            self.record_edit(Edit::Annotation {
                board: board_id.to_string(),
                slot: Slot::inserted(index, mirror),
            });
        }
        Ok(mid)
    }

    /// Enveloppe silencieuse de [`Store::try_mirror_annotation`].
    #[deprecated(note = "R-35 : l'échec est silencieux. Utiliser `try_mirror_annotation`.")]
    pub fn mirror_annotation(
        &mut self,
        board_id: &str,
        id: &str,
        x: f64,
        y: f64,
    ) -> Option<String> {
        self.try_mirror_annotation(board_id, id, x, y).ok()
    }

    // ── Storyboard Panels ───────────────────────────────────────────────────

    /// **Site migre vers le journal.**
    pub fn add_panel(&mut self, board_id: &str, panel: StoryboardPanel) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let index = b.panels.len();
        b.panels.push(panel.clone());
        self.record_edit(Edit::Panel {
            board: board_id.to_string(),
            slot: Slot::inserted(index, panel),
        });
    }

    /// **Site migre vers le journal.**
    pub fn update_panel(&mut self, board_id: &str, id: &str, description: impl Into<String>) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let Some(i) = b.panels.iter().position(|p| p.id == id) else {
            return;
        };
        let before = b.panels[i].clone();
        b.panels[i].description = description.into();
        let after = b.panels[i].clone();
        self.record_edit(Edit::Panel {
            board: board_id.to_string(),
            slot: Slot::changed(i, before, after),
        });
    }

    /// **Site migre vers le journal.**
    pub fn remove_panel(&mut self, board_id: &str, id: &str) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let Some(i) = b.panels.iter().position(|p| p.id == id) else {
            return;
        };
        let panel = b.panels.remove(i);
        self.record_edit(Edit::Panel {
            board: board_id.to_string(),
            slot: Slot::removed(i, panel),
        });
    }
}

/// Copie d'une annotation déposée en (x, y) et reliée à sa source par `mirror_of`.
///
/// Une flèche n'a pas de `mirror_of` : elle est translatée en bloc pour conserver sa forme.
fn build_mirror(
    source: Annotation,
    mirror_id: &str,
    origin_id: &str,
    x: f64,
    y: f64,
) -> Annotation {
    let mut m = source;
    let mut arrow_delta = None;
    match &mut m {
        Annotation::Text {
            id,
            x: ax,
            y: ay,
            mirror_of,
            ..
        }
        | Annotation::Sticky {
            id,
            x: ax,
            y: ay,
            mirror_of,
            ..
        }
        | Annotation::Membrane {
            id,
            x: ax,
            y: ay,
            mirror_of,
            ..
        } => {
            *id = mirror_id.to_string();
            *ax = x;
            *ay = y;
            *mirror_of = Some(origin_id.to_string());
        }
        Annotation::Arrow {
            id, x: ax, y: ay, ..
        } => {
            *id = mirror_id.to_string();
            arrow_delta = Some((x - *ax, y - *ay));
        }
    }
    if let Some((dx, dy)) = arrow_delta {
        translate_annotation(&mut m, dx, dy);
    }
    m
}
