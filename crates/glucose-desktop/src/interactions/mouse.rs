//! Traitement des événements de souris (clics, survol, menus, outils, sélection).

use crate::app::{GlucoseApp, LastClickInfo};
use crate::canvas::screen_to_world;
use crate::dock::{compute_panel_layouts, handle_dock_click, DragSession, PanelClickResult, TabId};
use crate::ui::{handle_ui_click, ActiveTool, UiAction, TOTAL_HEADER_HEIGHT};
use glucose_core::hit_priority::{collect_candidates, PickInput, PickOwner};
use glucose_core::types::Annotation;
use winit::dpi::PhysicalPosition;
use winit::event::MouseButton;

impl GlucoseApp {
    /// Mouvement continu de la souris (pan, drag d'élément, drag de dock, ou mise à jour de boîte élastique).
    pub fn handle_cursor_moved(&mut self, position: PhysicalPosition<f64>) {
        let prev_pos = self.mouse_pos;
        self.mouse_pos = (position.x, position.y);
        let dx = position.x - prev_pos.0;
        let dy = position.y - prev_pos.1;

        if self.dock_manager.drag.is_some() {
            self.dock_manager.update_drag(position.x as f32, position.y as f32);
            self.redraw();
            return;
        }

        if self.is_panning {
            self.handle_pan_move(dx, dy);
        } else if self.is_dragging_item {
            self.handle_item_drag_move(position.x, position.y);
        } else if self.selection_box.is_some() {
            self.update_selection_box(position.x, position.y);
            self.redraw();
        } else if position.y < TOTAL_HEADER_HEIGHT as f64 {
            self.redraw();
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
                self.redraw();
            }
            MouseButton::Left => {
                let mx = self.mouse_pos.0 as f32;
                let my = self.mouse_pos.1 as f32;

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
                            if let Some(files) = rfd::FileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
                                .pick_files()
                            {
                                self.import_image_files(&files);
                            }
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
                                "🌌 Trans-domaines activé"
                            } else {
                                "Trans-domaines désactivé"
                            });
                        }
                        UiAction::ToggleCollab => {}
                        UiAction::ExportMenu => {
                            self.ui.show_toast("💾 Exportation du canvas");
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
                            self.ui.show_toast("📋 Nouveau board créé");
                        }
                        UiAction::MinimapPan(wx, wy) => {
                            if let Some(b) = self.store.active_board_mut() {
                                b.viewport.x = screen_w as f64 / 2.0 - wx * b.viewport.scale;
                                b.viewport.y = screen_h as f64 / 2.0 - wy * b.viewport.scale;
                            }
                        }
                    }
                    self.update_cursor();
                    self.redraw();
                    return;
                }

                // 2. Clic sur les panneaux déroulants & poignées (Dock)
                let dock_layouts = compute_panel_layouts(
                    &self.dock_manager,
                    screen_w,
                    screen_h,
                    TOTAL_HEADER_HEIGHT,
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
                    self.redraw();
                    return;
                }

                if let Some(action) = handle_dock_click(
                    &mut self.dock_manager,
                    &self.store,
                    mx,
                    my,
                    screen_w,
                    screen_h,
                    TOTAL_HEADER_HEIGHT,
                ) {
                    match action {
                        PanelClickResult::ApplyLayout(state) => {
                            self.apply_dock_layout(&state);
                        }
                        PanelClickResult::AddDomain => {
                            let did = self.store.generate_id("domain");
                            let name = format!("Domaine {}", self.dock_manager.domains.domains.len() + 1);
                            self.dock_manager.domains.domains.push(crate::dock::DomainItem {
                                id: did,
                                name,
                                color: tiny_skia::Color::from_rgba8(168, 85, 247, 255),
                            });
                        }
                        _ => {}
                    }
                    self.redraw();
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
                        let ann = Annotation::Text {
                            id: aid.clone(),
                            x: wx,
                            y: wy,
                            width: Some(240.0),
                            height: Some(48.0),
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
                        self.ui.show_toast("📝 Édition du texte");
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
                        self.ui.show_toast("📌 Édition du sticky");
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
                        self.ui.show_toast("↗️ Flèche ajoutée");
                        self.ui.active_tool = ActiveTool::Select;
                        self.update_cursor();
                        self.redraw();
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
                        self.ui.show_toast("🧊 Membrane créée (rx=60)");
                        self.ui.active_tool = ActiveTool::Select;
                        self.update_cursor();
                        self.redraw();
                        return;
                    }
                    ActiveTool::Folder => {
                        self.ui.show_toast("📁 Dossier");
                        self.ui.active_tool = ActiveTool::Select;
                        self.update_cursor();
                        return;
                    }
                    ActiveTool::Select => {}
                }

                // 4. Sélection par clic (PICK-1)
                let mut selected = false;
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
                    let candidates = collect_candidates(&input);
                    if let Some(top) = candidates.first() {
                        let is_dbl_click = if let Some(ref lc) = self.last_click {
                            lc.id == top.id
                                && lc.time.elapsed().as_millis() < 350
                                && (lc.pos.0 - self.mouse_pos.0).hypot(lc.pos.1 - self.mouse_pos.1) < 8.0
                        } else {
                            false
                        };

                        self.last_click = Some(LastClickInfo {
                            time: std::time::Instant::now(),
                            pos: self.mouse_pos,
                            id: top.id.clone(),
                        });

                        if is_dbl_click {
                            if let Some(ann) = b.annotations.iter().find(|a| a.id() == top.id) {
                                let initial_text = match ann {
                                    Annotation::Text { text, .. } => text.clone(),
                                    Annotation::Sticky { text, .. } => text.clone(),
                                    Annotation::Membrane { text, .. } => text.clone().unwrap_or_default(),
                                    _ => String::new(),
                                };
                                self.start_text_edit(top.id.clone(), initial_text);
                                self.redraw();
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

                if selected {
                    self.init_item_drag(wx, wy);
                } else {
                    self.store.clear_selection();
                    self.start_selection_box(self.mouse_pos.0, self.mouse_pos.1);
                }

                self.update_cursor();
                self.redraw();
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
                self.redraw();
            }
            MouseButton::Left => {
                if let Some(dismissed) = self.dock_manager.finish_drag() {
                    self.ui.show_toast(format!("👋 Panneau {} fermé", dismissed.title()));
                    self.redraw();
                    return;
                }
                if !self.right_or_middle_down {
                    self.is_panning = false;
                }
                self.finish_item_drag();
                self.finish_selection_box();
                self.update_cursor();
                self.redraw();
            }
            _ => {}
        }
    }
}
