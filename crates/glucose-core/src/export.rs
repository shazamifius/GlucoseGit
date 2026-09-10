//! Moteurs d'exportation (SVG, Markdown) en 100% Rust Standard Library (0 dépendance).
//! Ports stricts de toMarkdown.ts, scene.ts et toSvg.ts.

use crate::types::{Annotation, Board, Project};

// ── Types de scène d'export ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoxRect {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneCard {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub text: String,
    pub font_size: f64,
    pub text_color: String,
    pub aura_color: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneSticky {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub text: String,
    pub color: String,
    pub bg_color: String,
    pub operator: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneMembrane {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub color: String,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneArrow {
    pub id: String,
    pub source_id: Option<String>,
    pub target_id: Option<String>,
    pub points: Vec<Pt>,
    pub stroke_width: f64,
    pub col_start: String,
    pub col_end: String,
    pub label: Option<String>,
    pub long_text: Option<String>,
    pub predicate: Option<String>,
    pub mid: Pt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportScene {
    pub project_name: String,
    pub board_name: String,
    pub background: String,
    pub bbox: BoxRect,
    pub width: f64,
    pub height: f64,
    pub cards: Vec<SceneCard>,
    pub stickies: Vec<SceneSticky>,
    pub membranes: Vec<SceneMembrane>,
    pub arrows: Vec<SceneArrow>,
}

pub const SCENE_BG: &str = "#0d0d0d";

#[derive(Debug, Clone, PartialEq)]
pub struct SvgOptions {
    pub transparent: bool,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self { transparent: false }
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn strip_inline_markdown(s: &str) -> String {
    s.replace("**", "").replace('*', "").replace('_', "").replace('`', "")
}

/// Titre d'affichage d'une carte : 1er titre `#`/`##`/`###`, sinon 1re ligne.
pub fn card_title(text: &str) -> String {
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(stripped) = t.strip_prefix('#') {
            let title = stripped.trim_start_matches('#').trim();
            if !title.is_empty() {
                return strip_inline_markdown(title);
            }
        }
        return strip_inline_markdown(t);
    }
    "(sans titre)".to_string()
}

/// Corps d'une carte sans son premier titre.
fn card_body_without_title(text: &str) -> String {
    let mut dropped = false;
    let mut out = Vec::new();
    for line in text.lines() {
        if !dropped && !line.trim().is_empty() {
            dropped = true;
            continue;
        }
        out.push(line);
    }
    out.join("\n").trim().to_string()
}

fn card_in_membrane(c: &SceneCard, m: &SceneMembrane) -> bool {
    let cx = c.x + c.w / 2.0;
    let cy = c.y + c.h / 2.0;
    cx >= m.x && cx <= m.x + m.w && cy >= m.y && cy <= m.y + m.h
}

pub fn build_scene(project: &Project) -> ExportScene {
    let active_id = &project.active_board_id;
    let board = project
        .boards
        .iter()
        .find(|b| &b.id == active_id)
        .or_else(|| project.boards.first());

    let mut cards = Vec::new();
    let mut stickies = Vec::new();
    let mut membranes = Vec::new();
    let mut arrows = Vec::new();

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    let mut extend = |l: f64, t: f64, r: f64, b: f64| {
        min_x = min_x.min(l);
        min_y = min_y.min(t);
        max_x = max_x.max(r);
        max_y = max_y.max(b);
    };

    if let Some(b) = board {
        for ann in &b.annotations {
            match ann {
                Annotation::Text { id, x, y, width, height, text, font_size, color, .. } => {
                    let w = width.unwrap_or(340.0);
                    let h = height.unwrap_or(120.0);
                    extend(*x, *y, *x + w, *y + h);
                    cards.push(SceneCard {
                        id: id.clone(),
                        x: *x,
                        y: *y,
                        w,
                        h,
                        text: text.clone(),
                        font_size: font_size.unwrap_or(14.0),
                        text_color: color.clone().unwrap_or_else(|| "#ffffff".into()),
                        aura_color: "#60a5fa".into(),
                    });
                }
                Annotation::Sticky { id, x, y, width, height, text, color, bg_color, operator, .. } => {
                    let w = width.unwrap_or(160.0);
                    let h = height.unwrap_or(120.0);
                    extend(*x, *y, *x + w, *y + h);
                    stickies.push(SceneSticky {
                        id: id.clone(),
                        x: *x,
                        y: *y,
                        w,
                        h,
                        text: text.clone(),
                        color: color.clone().unwrap_or_else(|| "#1a1a1a".into()),
                        bg_color: bg_color.clone().unwrap_or_else(|| "#fde68a".into()),
                        operator: operator.map(|o| format!("{:?}", o)),
                    });
                }
                Annotation::Membrane { id, x, y, width, height, color, text, .. } => {
                    extend(*x, *y, *x + width, *y + height);
                    membranes.push(SceneMembrane {
                        id: id.clone(),
                        x: *x,
                        y: *y,
                        w: *width,
                        h: *height,
                        color: color.clone().unwrap_or_else(|| "#8b5cf6".into()),
                        text: text.clone(),
                    });
                }
                Annotation::Arrow { id, x, y, x2, y2, source_id, target_id, text, long_text, predicate, .. } => {
                    extend(*x, *y, *x2, *y2);
                    let mid = Pt { x: (x + x2) / 2.0, y: (y + y2) / 2.0 };
                    let pred_str = predicate.map(|p| p.as_str().to_string());
                    arrows.push(SceneArrow {
                        id: id.clone(),
                        source_id: source_id.clone(),
                        target_id: target_id.clone(),
                        points: vec![Pt { x: *x, y: *y }, Pt { x: *x2, y: *y2 }],
                        stroke_width: 2.0,
                        col_start: "#60a5fa".into(),
                        col_end: "#a855f7".into(),
                        label: text.clone(),
                        long_text: long_text.clone(),
                        predicate: pred_str,
                        mid,
                    });
                }
            }
        }
    }

    if !min_x.is_finite() {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 800.0;
        max_y = 600.0;
    }

    let margin = 120.0;
    let bbox = BoxRect {
        left: min_x - margin,
        top: min_y - margin,
        right: max_x + margin,
        bottom: max_y + margin,
    };

    ExportScene {
        project_name: project.name.clone(),
        board_name: board.map(|b| b.name.clone()).unwrap_or_else(|| "Board".into()),
        background: SCENE_BG.into(),
        bbox,
        width: bbox.right - bbox.left,
        height: bbox.bottom - bbox.top,
        cards,
        stickies,
        membranes,
        arrows,
    }
}

pub fn scene_to_markdown(scene: &ExportScene) -> String {
    let mut out = Vec::new();
    out.push(format!("# {} — {}", scene.project_name, scene.board_name));
    out.push("".into());

    let mut title_by_id = std::collections::HashMap::new();
    for c in &scene.cards {
        title_by_id.insert(c.id.clone(), card_title(&c.text));
    }

    let mut used = std::collections::HashSet::new();

    // Membranes
    for m in &scene.membranes {
        let inside: Vec<&SceneCard> = scene
            .cards
            .iter()
            .filter(|c| !used.contains(&c.id) && card_in_membrane(c, m))
            .collect();
        if inside.is_empty() {
            continue;
        }
        let zone_title = m.text.as_deref().unwrap_or("Zone");
        out.push(format!("## {}", strip_inline_markdown(zone_title)));
        out.push("".into());
        for c in inside {
            used.insert(c.id.clone());
            out.push(format!("### {}", card_title(&c.text)));
            out.push("".into());
            let body = card_body_without_title(&c.text);
            if !body.is_empty() {
                out.push(body);
                out.push("".into());
            }
        }
    }

    // Cartes hors zone (orphelins)
    let orphans: Vec<&SceneCard> = scene
        .cards
        .iter()
        .filter(|c| !used.contains(&c.id))
        .collect();
    if !orphans.is_empty() {
        if !scene.membranes.is_empty() {
            out.push("## Autres".into());
            out.push("".into());
        }
        for c in orphans {
            out.push(format!("### {}", card_title(&c.text)));
            out.push("".into());
            let body = card_body_without_title(&c.text);
            if !body.is_empty() {
                out.push(body);
                out.push("".into());
            }
        }
    }

    // Notes (stickies)
    if !scene.stickies.is_empty() {
        out.push("## Notes".into());
        out.push("".into());
        for s in &scene.stickies {
            let op = s.operator.as_deref().map(|o| format!("_({})_ ", o)).unwrap_or_default();
            out.push(format!("- {}{}", op, strip_inline_markdown(&s.text)));
        }
        out.push("".into());
    }

    // Liens (flèches)
    let links: Vec<&SceneArrow> = scene
        .arrows
        .iter()
        .filter(|a| a.source_id.is_some() && a.target_id.is_some())
        .collect();
    if !links.is_empty() {
        out.push("## Liens".into());
        out.push("".into());
        for a in links {
            let src = a.source_id.as_ref().and_then(|id| title_by_id.get(id)).cloned().unwrap_or_else(|| "?".into());
            let tgt = a.target_id.as_ref().and_then(|id| title_by_id.get(id)).cloned().unwrap_or_else(|| "?".into());
            let rel = a.predicate.as_deref().unwrap_or("→");
            let mut line = format!("- **{}** {} **{}**", src, rel, tgt);
            if let Some(ref lt) = a.long_text {
                line.push_str(&format!(" — {}", strip_inline_markdown(lt)));
            }
            out.push(line);
        }
        out.push("".into());
    }

    out.join("\n").trim_end().to_string() + "\n"
}

pub fn project_to_markdown(project: &Project) -> String {
    let scene = build_scene(project);
    scene_to_markdown(&scene)
}

pub fn scene_to_svg(scene: &ExportScene, opts: &SvgOptions) -> String {
    let mut arrow_defs = Vec::new();
    let mut arrow_bodies = Vec::new();

    for a in &scene.arrows {
        if a.points.len() >= 2 {
            let grad_id = format!("grad-{}", a.id);
            arrow_defs.push(format!(
                r##"<linearGradient id="{}" gradientUnits="userSpaceOnUse" x1="{}" y1="{}" x2="{}" y2="{}"><stop offset="0" stop-color="{}"/><stop offset="1" stop-color="{}"/></linearGradient>"##,
                grad_id, a.points[0].x, a.points[0].y, a.points[1].x, a.points[1].y, a.col_start, a.col_end
            ));
            let d = format!("M {} {} L {} {}", a.points[0].x, a.points[0].y, a.points[1].x, a.points[1].y);
            arrow_bodies.push(format!(
                r##"<path d="{}" fill="none" stroke="url(#{})" stroke-width="{}"/>"##,
                d, grad_id, a.stroke_width
            ));
        }
    }

    let defs = arrow_defs.join("\n");
    let bg = if opts.transparent {
        String::new()
    } else {
        format!(
            r##"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"##,
            scene.bbox.left, scene.bbox.top, scene.width, scene.height, scene.background
        )
    };

    let mut cards_svg = Vec::new();
    for c in &scene.cards {
        cards_svg.push(format!(
            r##"<g class="card"><rect x="{}" y="{}" width="{}" height="{}" rx="12" fill="{}" fill-opacity="0.1"/><text x="{}" y="{}" fill="{}" font-size="14">{}</text></g>"##,
            c.x, c.y, c.w, c.h, c.aura_color,
            c.x + 12.0, c.y + 24.0, c.text_color, html_escape(&c.text)
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{} {} {} {}">
<defs>{}</defs>
{}
{}
{}
</svg>"#,
        scene.width.round(), scene.height.round(),
        scene.bbox.left, scene.bbox.top, scene.width, scene.height,
        defs, bg, cards_svg.join("\n"), arrow_bodies.join("\n")
    )
}

pub fn to_markdown(board: &Board) -> String {
    let mut p = Project::new("Export");
    p.boards = vec![board.clone()];
    project_to_markdown(&p)
}

pub fn to_svg(board: &Board) -> String {
    let mut p = Project::new("Export");
    p.boards = vec![board.clone()];
    let scene = build_scene(&p);
    scene_to_svg(&scene, &SvgOptions::default())
}
