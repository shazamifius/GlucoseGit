//! Traitement des événements de souris (clics, survol, menus, outils, sélection).

use crate::app::{GlucoseApp, LastClickInfo};
use crate::canvas::screen_to_world;
use crate::dock::{compute_panel_layouts, handle_dock_click, DragSession, PanelClickResult, TabId};
use crate::animation::{fly_into_folder, fly_out_to_depth};
use crate::params::{Pointer, ScreenFrame};
use glucose_core::membrane_focus::ScreenSize;
use crate::renderer::card::text_card_fit_height;
use crate::ui::{handle_ui_click, ActiveTool, UiAction};
use glucose_core::hit_priority::{collect_candidates_indexed, pick_consts, PickInput, PickOwner};
use glucose_core::types::{Annotation, CanvasFolder, Viewport};
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

/// Ce que disent les boutons dont la fonction n'existe pas encore. Un bouton qui annonce ce
/// qu'il n'a pas fait est un bouton qui ment ; celui-ci dit ce qu'il en est.
/// Taille d'un dossier créé à la main, en unités monde.
///
/// La même que celle d'une membrane créée au clic : ce sont les deux conteneurs du canevas, et
/// rien ne justifierait qu'ils naissent de tailles différentes. Le minimum de la fiche 06 § 8.1
/// est 180 × 120 ; celle-ci laisse de quoi poser quelque chose dedans.
pub const FOLDER_DEFAULT_SIZE: (f64, f64) = (320.0, 240.0);

pub const NOT_YET_EXPORT: &str = "Export : pas encore disponible";
pub const NOT_YET_STORYBOARD: &str = "Storyboard : pas encore disponible";
pub const NOT_YET_AI: &str = "IA locale : pas encore disponible";

impl GlucoseApp {
    /// Mouvement continu de la souris (pan, drag d'élément, drag de dock, ou mise à jour de boîte élastique).
    pub fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        let prev_pos = self.mouse_pos;
        self.mouse_pos = (position.x, position.y);
        let dx = position.x - prev_pos.0;
        let dy = position.y - prev_pos.1;

        if self.dock_manager.drag.is_some() {
            self.dock_manager.update_drag(position.x as f32, position.y as f32);
            self.mark_dirty();
            return;
        }

        if self.is_panning {
            self.handle_pan_move(dx, dy);
        } else if self.resize_session.is_some() {
            self.handle_resize_move(position.x, position.y);
        } else if self.is_dragging_item {
            self.handle_item_drag_move(position.x, position.y);
        } else if self.selection_box.is_some() {
            self.update_selection_box(position.x, position.y);
            self.mark_dirty();
        } else if position.y < self.ui.header_height() as f64 {
            self.mark_dirty();
        }

        self.update_cursor();
    }

    /// Enfoncement d'un bouton de la souris (gauche, droit, milieu).
    pub fn handle_mouse_down(&mut self, button: MouseButton, screen_w: f32, screen_h: f32) {
        match button {
            MouseButton::Right | MouseButton::Middle => {
                self.right_or_middle_down = true;
                self.is_panning = true;
                self.update_cursor();
                self.mark_dirty();
            }
            MouseButton::Left => {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;

                // 0. Un clic pendant une plongée l'abrège : on arrive tout de suite. Le clic
                // est consommé — le viser dans le tableau d'arrivée, à une position qui n'a
                // rien à voir avec celle qu'on visait au départ, serait pire que de l'ignorer.
                if self.animator.skip(&mut self.store) {
                    self.update_cursor();
                    self.mark_dirty();
                    return;
                }

                // 0 bis. Clic sur le fil d'Ariane : il occupe une bande sous les onglets, donc
                // avant tout le reste. Remonter change le tableau, plus rien de ce clic ne
                // vaut ensuite.
                if let Some(depth) = crate::ui::breadcrumb::hit_breadcrumb(
                    &self.store,
                    &self.renderer.typography,
                    self.ui.header_height(),
                    self.ui.scale_factor,
                    (mx, my),
                ) {
                    let ecran = ScreenSize { width: screen_w as f64, height: screen_h as f64 };
                    if fly_out_to_depth(&mut self.store, &mut self.animator, depth, ecran) {
                        self.update_cursor();
                        self.mark_dirty();
                    }
                    return;
                }

                // 1. Clic sur l'interface (Header / TopBar / Tabs / Minimap)
                if let Some(action) = handle_ui_click(
                    mx,
                    my,
                    screen_w,
                    screen_h,
                    &self.store,
                    &mut self.ui,
                    &self.renderer.typography,
                ) {
                    match action {
                        UiAction::SelectTool(tool) => {
                            self.ui.active_tool = tool;
                        }
                        UiAction::AddImages => {
                            self.pick_and_import_images();
                        }
                        UiAction::Organize => {
                            self.dock_manager.toggle_tab(TabId::Organize);
                        }
                        UiAction::ToggleTimer => {
                            self.dock_manager.toggle_tab(TabId::Pomodoro);
                        }
                        UiAction::ToggleStoryboard => {
                            self.dock_manager.toggle_tab(TabId::Storyboard);
                        }
                        UiAction::ToggleMagnet => {}
                        UiAction::ToggleTransDomain => {
                            self.ui.show_toast(if self.ui.trans_domain {
                                "Trans-domaines activé"
                            } else {
                                "Trans-domaines désactivé"
                            });
                        }
                        UiAction::ToggleCollab => {}
                        UiAction::ExportMenu => {
                            // Fiche 09 § 5 : aucun export n'est branché. Le bouton reste,
                            // parce que la barre d'outils le prévoit (fiche 10) ; il dit la
                            // vérité plutôt que d'annoncer une exportation qui n'a pas lieu.
                            self.ui.show_toast(NOT_YET_EXPORT);
                        }
                        UiAction::TogglePlugins => {
                            self.dock_manager.toggle_tab(TabId::Plugins);
                        }
                        UiAction::TogglePreset => {
                            self.dock_manager.toggle_tab(TabId::Preset);
                        }
                        UiAction::ToggleDomains => {
                            self.dock_manager.toggle_tab(TabId::Domains);
                        }
                        UiAction::SelectBoard(id) => {
                            self.store.set_active_board_id(&id);
                        }
                        UiAction::AddBoard => {
                            let new_name = format!("Board {}", self.store.project.boards.len() + 1);
                            let new_id = self.store.add_board(new_name);
                            self.store.set_active_board_id(new_id);
                            self.ui.show_toast("Nouveau board créé");
                        }
                        UiAction::MinimapPan(wx, wy) => {
                            if let Some(b) = self.store.active_board() {
                                let (bid, scale) = (b.id.clone(), b.viewport.scale);
                                self.store.set_viewport(
                                    &bid,
                                    Viewport {
                                        x: screen_w as f64 / 2.0 - wx * scale,
                                        y: screen_h as f64 / 2.0 - wy * scale,
                                        scale,
                                    },
                                );
                            }
                        }
                    }
                    self.update_cursor();
                    self.mark_dirty();
                    return;
                }

                // 2. Clic sur les panneaux déroulants & poignées (Dock)
                let dock_layouts = compute_panel_layouts(
                    &self.dock_manager,
                    screen_w,
                    screen_h,
                    self.ui.header_height(),
                    self.ui.scale_factor,
                );

                let mut grip_hit = None;
                for layout in dock_layouts.iter().rev() {
                    if layout.grip_contains_point(mx, my) {
                        grip_hit = Some(layout.tab);
                        break;
                    }
                }
                if let Some(tab) = grip_hit {
                    self.dock_manager.drag = Some(DragSession {
                        tab,
                        start_x: mx,
                        start_y: my,
                        current_x: mx,
                        current_y: my,
                    });
                    self.mark_dirty();
                    return;
                }

                let screen = ScreenFrame {
                    width: screen_w,
                    height: screen_h,
                    header_h: self.ui.header_height(),
                    scale: self.ui.scale_factor,
                };
                if let Some(action) = handle_dock_click(
                    &mut self.dock_manager,
                    &self.store,
                    &self.renderer.typography,
                    screen,
                    Pointer { x: mx, y: my },
                ) {
                    match action {
                        PanelClickResult::ApplyLayout(state) => {
                            self.apply_dock_layout(&state);
                        }
                        PanelClickResult::Domain(intent) => {
                            self.apply_domain_intent(intent);
                        }
                        // Fiche 09 § 8 : le storyboard n'a pas d'effet sur le canevas. Le
                        // panneau ne doit pas laisser croire le contraire.
                        PanelClickResult::ToggleStoryboard | PanelClickResult::SelectFormat(_) => {
                            self.dock_manager.storyboard.active = false;
                            self.ui.show_toast(NOT_YET_STORYBOARD);
                        }
                        // Fiche 09 § 10.3 : pas de moteur, pas de téléchargement.
                        PanelClickResult::DownloadModel => {
                            self.ui.show_toast(NOT_YET_AI);
                        }
                        _ => {}
                    }
                    self.mark_dirty();
                    return;
                }

                // 3. Clic sur le canvas
                if self.space_pressed || self.ui.active_tool == ActiveTool::Pan {
                    self.is_panning = true;
                    self.update_cursor();
                    return;
                }

                let active_bid = self.store.project.active_board_id.clone();
                let vp = self.store.active_board().map(|b| b.viewport).unwrap_or_default();
                let (wx, wy) = screen_to_world(self.mouse_pos.0, self.mouse_pos.1, &vp);

                // Outils interactifs de création
                match self.ui.active_tool {
                    ActiveTool::Pan => unreachable!(),
                    ActiveTool::Text => {
                        let aid = self.store.generate_id("text");
                        let initial_str = "Nouveau texte".to_string();
                        // TEXT-FIT-1 : la hauteur d'une carte est celle de son texte.
                        let height = text_card_fit_height(&self.renderer.typography, &self.renderer.math, &initial_str, 240.0);
                        let ann = Annotation::Text {
                            id: aid.clone(),
                            x: wx,
                            y: wy,
                            width: Some(240.0),
                            height: Some(height),
                            text: initial_str.clone(),
                            font_size: Some(14.0),
                            color: None,
                            cursor_pos: None,
                            source_file: None,
                            membrane_id: None,
                            domains: Vec::new(),
                            mirror_of: None,
                            temporal_anchor: None,
                        };
                        self.store.add_annotation(&active_bid, ann);
                        self.start_text_edit(aid, initial_str);
                        self.ui.show_toast("Édition du texte");
                        self.ui.active_tool = ActiveTool::Select;
                        self.update_cursor();
                        return;
                    }
                    ActiveTool::Sticky => {
                        let aid = self.store.generate_id("sticky");
                        let initial_str = "Nouvelle note".to_string();
                        let ann = Annotation::Sticky {
                            id: aid.clone(),
                            x: wx,
                            y: wy,
                            width: Some(180.0),
                            height: Some(130.0),
                            text: initial_str.clone(),
                            font_size: Some(12.0),
                            color: Some("#1c1917".into()),
                            bg_color: Some("#fef08a".into()),
                            cursor_pos: None,
                            operator: None,
                            source_file: None,
                            membrane_id: None,
                            domains: Vec::new(),
                            mirror_of: None,
                            temporal_anchor: None,
                        };
                        self.store.add_annotation(&active_bid, ann);
                        self.start_text_edit(aid, initial_str);
                        self.ui.show_toast("Édition du sticky");
                        self.ui.active_tool = ActiveTool::Select;
                        self.update_cursor();
                        return;
                    }
                    ActiveTool::Arrow => {
                        let aid = self.store.generate_id("arrow");
                        let ann = Annotation::Arrow {
                            id: aid,
                            x: wx,
                            y: wy,
                            x2: wx + 120.0,
                            y2: wy + 80.0,
                            text: None,
                            font_size: None,
                            color: Some("#94a3b8".into()),
                            arrow_type: None,
                            arrow_bidirectional: false,
                            predicate: None,
                            stroke_width: Some(2.0),
                            waypoints: Vec::new(),
                            source_id: None,
                            target_id: None,
                            source_block_id: None,
                            target_block_id: None,
                            source_text_sel: None,
                            target_text_sel: None,
                            long_text: None,
                            target_board_id: None,
                            membrane_id: None,
                            domains: Vec::new(),
                            mirror_of: None,
                            temporal_anchor: None,
                        };
                        self.store.add_annotation(&active_bid, ann);
                        self.ui.show_toast("Flèche ajoutée");
                        self.ui.active_tool = ActiveTool::Select;
                        self.update_cursor();
                        self.mark_dirty();
                        return;
                    }
                    ActiveTool::Membrane => {
                        let aid = self.store.generate_id("membrane");
                        let ann = Annotation::Membrane {
                            id: aid,
                            x: wx,
                            y: wy,
                            width: 320.0,
                            height: 240.0,
                            color: Some("#60a5fa".into()),
                            text: Some("Groupe".into()),
                            mode: glucose_core::types::MembraneMode::Classic,
                            curtains: Vec::new(),
                            membrane_id: None,
                            domains: Vec::new(),
                            mirror_of: None,
                            temporal_anchor: None,
                        };
                        self.store.add_annotation(&active_bid, ann);
                        self.ui.show_toast("Membrane créée (rx=60)");
                        self.ui.active_tool = ActiveTool::Select;
                        self.update_cursor();
                        self.mark_dirty();
                        return;
                    }
                    ActiveTool::Folder => {
                        // Le dossier capture ce qui se trouve sous lui : `create_folder` le
                        // fait, crée le tableau enfant, et enregistre le tout comme UN geste
                        // annulable. L'outil se contentait d'un toast, ce que la fiche 11 § A.2
                        // interdit — un bouton qui annonce ce qu'il ne fait pas.
                        let mut folder = CanvasFolder::new(
                            self.store.generate_id("folder"),
                            "Dossier",
                            String::new(),
                        );
                        folder.x = wx;
                        folder.y = wy;
                        folder.width = FOLDER_DEFAULT_SIZE.0;
                        folder.height = FOLDER_DEFAULT_SIZE.1;
                        self.store.create_folder(&active_bid, folder);
                        self.ui.show_toast("Dossier créé");
                        self.ui.active_tool = ActiveTool::Select;
                        self.update_cursor();
                        self.mark_dirty();
                        return;
                    }
                    ActiveTool::Select => {}
                }

                // 4. Une poignée sous le clic : le geste de redimensionnement (RESIZE-1).
                // La priorité poignée > nœud > canevas est celle de `hit_priority`.
                if self.begin_resize_at(wx, wy) {
                    self.update_cursor();
                    self.mark_dirty();
                    return;
                }

                // 5. Sélection par clic (PICK-1)
                // L'index doit refléter le document AVANT qu'on l'interroge : sans cela, un
                // nœud créé au clic précédent serait introuvable jusqu'à la frame suivante.
                self.renderer.sync_spatial_index(&self.store);
                let mut selected = false;
                let mut entrer_dans: Option<String> = None;
                if let Some(b) = self.store.active_board() {
                    let input = PickInput {
                        wx,
                        wy,
                        scale: vp.scale,
                        images: &b.images,
                        annotations: &b.annotations,
                        folders: &b.folders,
                        selected_image_ids: &self.store.selected_image_ids,
                        selected_annotation_ids: &self.store.selected_annotation_ids,
                        selected_folder_id: self.store.selected_folder_id.as_deref(),
                        arrow_id: None,
                        dom_hint: None,
                    };
                    let candidates = collect_candidates_indexed(&input, &self.renderer.spatial_hash);
                    if let Some(top) = candidates.first() {
                        let is_dbl_click = if let Some(ref lc) = self.last_click {
                            lc.id == top.id
                                && (lc.time.elapsed().as_millis() as i64) < pick_consts::DBLCLICK_MS
                                && (lc.pos.0 - self.mouse_pos.0).hypot(lc.pos.1 - self.mouse_pos.1) < 8.0
                        } else {
                            false
                        };

                        self.last_click = Some(LastClickInfo {
                            time: std::time::Instant::now(),
                            pos: self.mouse_pos,
                            id: top.id.clone(),
                        });

                        if is_dbl_click && top.owner == PickOwner::Folder {
                            // Entrer demande `&mut self.store`, et `b` emprunte ce même store :
                            // l'intention est notée ici et exécutée une fois l'emprunt rendu.
                            entrer_dans = Some(top.id.clone());
                        }
                        if is_dbl_click {
                            if let Some(ann) = b.annotations.iter().find(|a| a.id() == top.id) {
                                let initial_text = match ann {
                                    Annotation::Text { text, .. } => text.clone(),
                                    Annotation::Sticky { text, .. } => text.clone(),
                                    Annotation::Membrane { text, .. } => text.clone().unwrap_or_default(),
                                    _ => String::new(),
                                };
                                self.start_text_edit(top.id.clone(), initial_text);
                                self.mark_dirty();
                                return;
                            }
                        }

                        if let Some(ref session) = self.editing_session {
                            if session.ann_id != top.id {
                                self.commit_editing();
                            }
                        }

                        match top.owner {
                            PickOwner::Image => {
                                self.store.select_image(top.id.clone(), self.modifiers.shift_key());
                                selected = true;
                            }
                            PickOwner::Annotation | PickOwner::Membrane | PickOwner::Arrow => {
                                self.store.select_annotation(top.id.clone(), self.modifiers.shift_key());
                                selected = true;
                            }
                            PickOwner::Folder => {
                                self.store.select_folder(top.id.clone());
                                selected = true;
                            }
                        }
                    } else {
                        self.commit_editing();
                    }
                }

                // Entrer dans un dossier remplace le tableau : plus rien de ce clic n'a de
                // sens ensuite, ni sélection ni début de glisser.
                if let Some(folder_id) = entrer_dans {
                    let ecran = ScreenSize { width: screen_w as f64, height: screen_h as f64 };
                    // La caméra plonge, et la bascule attend l'arrivée. Si le dossier a
                    // disparu entre-temps, on entre sans cérémonie plutôt que de ne rien faire.
                    if !fly_into_folder(&self.store, &mut self.animator, &folder_id, ecran) {
                        drop(self.store.try_enter_folder(&folder_id));
                    }
                    self.update_cursor();
                    self.mark_dirty();
                    return;
                }

                if selected {
                    self.init_item_drag(wx, wy);
                } else {
                    self.store.clear_selection();
                    self.start_selection_box(self.mouse_pos.0, self.mouse_pos.1);
                }

                self.update_cursor();
                self.mark_dirty();
            }
            _ => {}
        }
    }

    /// Relâchement d'un bouton de souris.
    pub fn handle_mouse_up(&mut self, button: MouseButton) {
        match button {
            MouseButton::Right | MouseButton::Middle => {
                self.right_or_middle_down = false;
                self.is_panning = false;
                self.update_cursor();
                self.mark_dirty();
            }
            MouseButton::Left => {
                if let Some(dismissed) = self.dock_manager.finish_drag() {
                    self.ui.show_toast(format!("Panneau {} fermé", dismissed.title()));
                    self.mark_dirty();
                    return;
                }
                if !self.right_or_middle_down {
                    self.is_panning = false;
                }
                self.finish_resize();
                self.finish_item_drag();
                self.finish_selection_box();
                self.update_cursor();
                self.mark_dirty();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
