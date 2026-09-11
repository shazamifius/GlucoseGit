//! Section `document` du format `.glucose` v2 : le projet, ses tableaux et leur mobilier.
//!
//! # INVARIANT PERSIST-1 — aucun champ ne peut être oublié
//!
//! Chaque encodeur **déstructure** la valeur qu'il écrit (`let Board { id, name, .. } = board;`)
//! sans jamais utiliser `..`. Ajouter un champ au modèle casse alors la compilation *ici*, et
//! le décodeur, qui construit par littéral de structure, casse symétriquement. C'est la
//! vérification automatique demandée par la loi L9 (§ 2.6) : un champ arrive avec sa
//! sérialisation, ou la build est rouge.
//!
//! # Ordre stable
//!
//! Les `HashMap` (les signets de tableau) sont écrits **triés par clé**. Deux sauvegardes du
//! même document produisent ainsi des octets identiques, ce qui rend le format diffable et
//! permettra plus tard de sauter l'écriture d'une section inchangée.

use super::bytes::{Reader, Writer};
use super::tags::{sort_mode_from_tag, sort_mode_tag};
use crate::error::CoreResult;
use crate::types::{
    Board, BoardZone, CanvasFolder, Domain, DomainAssignment, FolderMirrorSource, Preset, PresetSlot,
    Project, StoryboardPanel, TemporalAnchor, Viewport,
};
use std::collections::HashMap;

// ── Projet ──────────────────────────────────────────────────────────────────

pub fn write_project(w: &mut Writer, project: &Project) {
    let Project {
        version,
        name,
        boards,
        active_board_id,
        presets,
        domains,
        collab_url,
        asset_channel_url,
        created_at,
        updated_at,
    } = project;

    w.text(version);
    w.text(name);
    w.seq(boards, write_board);
    w.text(active_board_id);
    w.seq(presets, write_preset);
    w.seq(domains, write_domain);
    w.opt_text(collab_url.as_ref());
    w.opt_text(asset_channel_url.as_ref());
    w.i64(*created_at);
    w.i64(*updated_at);
}

pub fn read_project(r: &mut Reader<'_>) -> CoreResult<Project> {
    Ok(Project {
        version: r.text()?,
        name: r.text()?,
        boards: r.seq(read_board)?,
        active_board_id: r.text()?,
        presets: r.seq(read_preset)?,
        domains: r.seq(read_domain)?,
        collab_url: r.opt_text()?,
        asset_channel_url: r.opt_text()?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

// ── Tableau ─────────────────────────────────────────────────────────────────

fn write_board(w: &mut Writer, board: &Board) {
    let Board {
        id,
        images,
        annotations,
        folders,
        panels,
        zones,
        name,
        viewport,
        bookmarks,
        created_at,
        updated_at,
    } = board;

    w.text(id);
    w.text(name);
    w.seq(images, super::image::write_image);
    w.seq(annotations, super::annotation::write_annotation);
    w.seq(folders, write_folder);
    w.seq(panels, write_panel);
    w.seq(zones, write_zone);
    write_viewport(w, viewport);
    write_bookmarks(w, bookmarks);
    w.i64(*created_at);
    w.i64(*updated_at);
}

fn read_board(r: &mut Reader<'_>) -> CoreResult<Board> {
    Ok(Board {
        id: r.text()?,
        name: r.text()?,
        images: r.seq(super::image::read_image)?,
        annotations: r.seq(super::annotation::read_annotation)?,
        folders: r.seq(read_folder)?,
        panels: r.seq(read_panel)?,
        zones: r.seq(read_zone)?,
        viewport: read_viewport(r)?,
        bookmarks: read_bookmarks(r)?,
        created_at: r.i64()?,
        updated_at: r.i64()?,
    })
}

// ── Caméra et signets ───────────────────────────────────────────────────────

fn write_viewport(w: &mut Writer, viewport: &Viewport) {
    let Viewport { x, y, scale } = viewport;
    w.f64(*x);
    w.f64(*y);
    w.f64(*scale);
}

fn read_viewport(r: &mut Reader<'_>) -> CoreResult<Viewport> {
    Ok(Viewport {
        x: r.f64()?,
        y: r.f64()?,
        scale: r.f64()?,
    })
}

fn write_bookmarks(w: &mut Writer, bookmarks: &HashMap<String, Viewport>) {
    let mut entries: Vec<(&String, &Viewport)> = bookmarks.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    w.count(entries.len());
    for (key, vp) in entries {
        w.text(key);
        write_viewport(w, vp);
    }
}

fn read_bookmarks(r: &mut Reader<'_>) -> CoreResult<HashMap<String, Viewport>> {
    let entries = r.seq(|rr| {
        let key = rr.text()?;
        let vp = read_viewport(rr)?;
        Ok((key, vp))
    })?;
    Ok(entries.into_iter().collect())
}

// ── Dossiers de canevas ─────────────────────────────────────────────────────

fn write_folder(w: &mut Writer, folder: &CanvasFolder) {
    let CanvasFolder {
        id,
        name,
        color,
        x,
        y,
        width,
        height,
        child_board_id,
        mirror_of,
        mirror_source,
    } = folder;

    w.text(id);
    w.text(name);
    w.text(color);
    w.f64(*x);
    w.f64(*y);
    w.f64(*width);
    w.f64(*height);
    w.text(child_board_id);
    w.opt_text(mirror_of.as_ref());
    w.opt(mirror_source.as_ref(), write_mirror_source);
}

fn read_folder(r: &mut Reader<'_>) -> CoreResult<CanvasFolder> {
    Ok(CanvasFolder {
        id: r.text()?,
        name: r.text()?,
        color: r.text()?,
        x: r.f64()?,
        y: r.f64()?,
        width: r.f64()?,
        height: r.f64()?,
        child_board_id: r.text()?,
        mirror_of: r.opt_text()?,
        mirror_source: r.opt(read_mirror_source)?,
    })
}

fn write_mirror_source(w: &mut Writer, source: &FolderMirrorSource) {
    let FolderMirrorSource {
        root_path,
        mode,
        last_scanned_at,
        pattern,
        recursive,
        sort_by,
        pending_scan,
    } = source;

    w.text(root_path);
    w.text(mode);
    w.i64(*last_scanned_at);
    w.opt_text(pattern.as_ref());
    w.flag(*recursive);
    w.opt(*sort_by, |ww, mode| ww.u8(sort_mode_tag(mode)));
    w.flag(*pending_scan);
}

fn read_mirror_source(r: &mut Reader<'_>) -> CoreResult<FolderMirrorSource> {
    Ok(FolderMirrorSource {
        root_path: r.text()?,
        mode: r.text()?,
        last_scanned_at: r.i64()?,
        pattern: r.opt_text()?,
        recursive: r.flag()?,
        sort_by: r.opt(|rr| sort_mode_from_tag(rr.u8()?))?,
        pending_scan: r.flag()?,
    })
}

// ── Storyboard et zones ─────────────────────────────────────────────────────

fn write_panel(w: &mut Writer, panel: &StoryboardPanel) {
    let StoryboardPanel {
        id,
        order,
        description,
        x,
        y,
        width,
        height,
    } = panel;

    w.text(id);
    w.i32(*order);
    w.text(description);
    w.f64(*x);
    w.f64(*y);
    w.f64(*width);
    w.f64(*height);
}

fn read_panel(r: &mut Reader<'_>) -> CoreResult<StoryboardPanel> {
    Ok(StoryboardPanel {
        id: r.text()?,
        order: r.i32()?,
        description: r.text()?,
        x: r.f64()?,
        y: r.f64()?,
        width: r.f64()?,
        height: r.f64()?,
    })
}

fn write_zone(w: &mut Writer, zone: &BoardZone) {
    let BoardZone {
        slot_id,
        x,
        y,
        width,
        height,
    } = zone;

    w.text(slot_id);
    w.f64(*x);
    w.f64(*y);
    w.f64(*width);
    w.f64(*height);
}

fn read_zone(r: &mut Reader<'_>) -> CoreResult<BoardZone> {
    Ok(BoardZone {
        slot_id: r.text()?,
        x: r.f64()?,
        y: r.f64()?,
        width: r.f64()?,
        height: r.f64()?,
    })
}

// ── Presets ─────────────────────────────────────────────────────────────────

fn write_preset(w: &mut Writer, preset: &Preset) {
    let Preset {
        id,
        name,
        description,
        slots,
        is_builtin,
        created_at,
    } = preset;

    w.text(id);
    w.text(name);
    w.text(description);
    w.seq(slots, write_preset_slot);
    w.flag(*is_builtin);
    w.i64(*created_at);
}

fn read_preset(r: &mut Reader<'_>) -> CoreResult<Preset> {
    Ok(Preset {
        id: r.text()?,
        name: r.text()?,
        description: r.text()?,
        slots: r.seq(read_preset_slot)?,
        is_builtin: r.flag()?,
        created_at: r.i64()?,
    })
}

fn write_preset_slot(w: &mut Writer, slot: &PresetSlot) {
    let PresetSlot {
        id,
        name,
        color,
        description,
        order,
    } = slot;

    w.text(id);
    w.text(name);
    w.text(color);
    w.text(description);
    w.i32(*order);
}

fn read_preset_slot(r: &mut Reader<'_>) -> CoreResult<PresetSlot> {
    Ok(PresetSlot {
        id: r.text()?,
        name: r.text()?,
        color: r.text()?,
        description: r.text()?,
        order: r.i32()?,
    })
}

// ── Domaines ────────────────────────────────────────────────────────────────

fn write_domain(w: &mut Writer, domain: &Domain) {
    let Domain {
        id,
        name,
        color,
        icon,
        created_at,
    } = domain;

    w.text(id);
    w.text(name);
    w.text(color);
    w.text(icon);
    w.i64(*created_at);
}

fn read_domain(r: &mut Reader<'_>) -> CoreResult<Domain> {
    Ok(Domain {
        id: r.text()?,
        name: r.text()?,
        color: r.text()?,
        icon: r.text()?,
        created_at: r.i64()?,
    })
}

pub fn write_domain_assignment(w: &mut Writer, assignment: &DomainAssignment) {
    let DomainAssignment { domain_id, weight } = assignment;
    w.text(domain_id);
    w.f64(*weight);
}

pub fn read_domain_assignment(r: &mut Reader<'_>) -> CoreResult<DomainAssignment> {
    Ok(DomainAssignment {
        domain_id: r.text()?,
        weight: r.f64()?,
    })
}

// ── Ancrage temporel ────────────────────────────────────────────────────────

pub fn write_temporal_anchor(w: &mut Writer, anchor: &TemporalAnchor) {
    let TemporalAnchor { start, end, label } = anchor;
    w.i64(*start);
    w.i64(*end);
    w.opt_text(label.as_ref());
}

pub fn read_temporal_anchor(r: &mut Reader<'_>) -> CoreResult<TemporalAnchor> {
    Ok(TemporalAnchor {
        start: r.i64()?,
        end: r.i64()?,
        label: r.opt_text()?,
    })
}
