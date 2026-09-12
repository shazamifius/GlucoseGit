//! Sérialisation des quatre variantes d'annotation et de leurs satellites.
//!
//! Le discriminant de variante est un `u8` en tête (`0` texte, `1` pense-bête, `2` flèche,
//! `3` membrane). Il est **stable pour toujours** : une variante nouvelle prend le numéro
//! suivant, aucune n'est renumérotée. Un discriminant inconnu n'est jamais deviné — c'est une
//! erreur qui dit à l'utilisateur de mettre Glucose à jour.
//!
//! Voir l'INVARIANT PERSIST-1 dans [`super::document`] : chaque écrivain déstructure sa
//! variante sans `..`, donc un champ ajouté casse la build ici. L'aiguillage, lui, matche sur
//! `{ .. }` et délègue — c'est ce qui garde chaque fonction sous les 60 lignes de la règle
//! § 1.1 malgré une flèche à 25 champs, et ce qui rend l'écriture symétrique de la lecture.
//!
//! Le `let … else { return }` de chaque écrivain n'est jamais pris : l'aiguillage a déjà
//! choisi la variante. S'il l'était (aiguillage mal recâblé), rien ne serait écrit pour cette
//! annotation et le compteur de la séquence ne collerait plus — le lecteur rendrait une erreur
//! de troncature. Un défaut de câblage se voit donc, il ne corrompt pas en silence.

use super::bytes::{Reader, Writer};
use super::document::{
    read_domain_assignment, read_temporal_anchor, write_domain_assignment, write_temporal_anchor,
};
use super::tags::{
    editable_from_tag, editable_tag, membrane_mode_from_tag, membrane_mode_tag, operator_from_tag,
    operator_tag, predicate_from_tag, predicate_tag, unknown, visibility_from_tag, visibility_tag,
};
use crate::error::CoreResult;
use crate::types::{Annotation, CurtainNote, MembraneCurtain, Point2D, TextAnchor, TextSelection};

const KIND_TEXT: u8 = 0;
const KIND_STICKY: u8 = 1;
const KIND_ARROW: u8 = 2;
const KIND_MEMBRANE: u8 = 3;

// ── Écriture ────────────────────────────────────────────────────────────────

pub fn write_annotation(w: &mut Writer, ann: &Annotation) {
    match ann {
        Annotation::Text { .. } => write_text(w, ann),
        Annotation::Sticky { .. } => write_sticky(w, ann),
        Annotation::Arrow { .. } => write_arrow(w, ann),
        Annotation::Membrane { .. } => write_membrane(w, ann),
    }
}

fn write_text(w: &mut Writer, ann: &Annotation) {
    let Annotation::Text {
        id,
        x,
        y,
        width,
        height,
        text,
        font_size,
        color,
        cursor_pos,
        source_file,
        membrane_id,
        domains,
        mirror_of,
        temporal_anchor,
    } = ann
    else {
        return;
    };

    w.u8(KIND_TEXT);
    w.text(id);
    w.f64(*x);
    w.f64(*y);
    w.opt_f64(*width);
    w.opt_f64(*height);
    w.text(text);
    w.opt_f64(*font_size);
    w.opt_text(color.as_ref());
    w.opt(*cursor_pos, |ww, pos| ww.count(pos));
    w.opt_text(source_file.as_ref());
    w.opt_text(membrane_id.as_ref());
    w.seq(domains, write_domain_assignment);
    w.opt_text(mirror_of.as_ref());
    w.opt(temporal_anchor.as_ref(), write_temporal_anchor);
}

fn write_sticky(w: &mut Writer, ann: &Annotation) {
    let Annotation::Sticky {
        id,
        x,
        y,
        width,
        height,
        text,
        font_size,
        color,
        bg_color,
        cursor_pos,
        operator,
        source_file,
        membrane_id,
        domains,
        mirror_of,
        temporal_anchor,
    } = ann
    else {
        return;
    };

    w.u8(KIND_STICKY);
    w.text(id);
    w.f64(*x);
    w.f64(*y);
    w.opt_f64(*width);
    w.opt_f64(*height);
    w.text(text);
    w.opt_f64(*font_size);
    w.opt_text(color.as_ref());
    w.opt_text(bg_color.as_ref());
    w.opt(*cursor_pos, |ww, pos| ww.count(pos));
    w.opt(*operator, |ww, op| ww.u8(operator_tag(op)));
    w.opt_text(source_file.as_ref());
    w.opt_text(membrane_id.as_ref());
    w.seq(domains, write_domain_assignment);
    w.opt_text(mirror_of.as_ref());
    w.opt(temporal_anchor.as_ref(), write_temporal_anchor);
}

fn write_arrow(w: &mut Writer, ann: &Annotation) {
    let Annotation::Arrow {
        id,
        x,
        y,
        x2,
        y2,
        text,
        font_size,
        color,
        arrow_type,
        arrow_bidirectional,
        predicate,
        stroke_width,
        waypoints,
        source_id,
        target_id,
        source_block_id,
        target_block_id,
        source_text_sel,
        target_text_sel,
        long_text,
        target_board_id,
        membrane_id,
        domains,
        mirror_of,
        temporal_anchor,
    } = ann
    else {
        return;
    };

    w.u8(KIND_ARROW);
    w.text(id);
    w.f64(*x);
    w.f64(*y);
    w.f64(*x2);
    w.f64(*y2);
    w.opt_text(text.as_ref());
    w.opt_f64(*font_size);
    w.opt_text(color.as_ref());
    w.opt_text(arrow_type.as_ref());
    w.flag(*arrow_bidirectional);
    w.opt(*predicate, |ww, p| ww.u8(predicate_tag(p)));
    w.opt_f64(*stroke_width);
    w.seq(waypoints, write_point);
    w.opt_text(source_id.as_ref());
    w.opt_text(target_id.as_ref());
    w.opt_text(source_block_id.as_ref());
    w.opt_text(target_block_id.as_ref());
    w.opt(source_text_sel.as_ref(), write_text_selection);
    w.opt(target_text_sel.as_ref(), write_text_selection);
    w.opt_text(long_text.as_ref());
    w.opt_text(target_board_id.as_ref());
    w.opt_text(membrane_id.as_ref());
    w.seq(domains, write_domain_assignment);
    w.opt_text(mirror_of.as_ref());
    w.opt(temporal_anchor.as_ref(), write_temporal_anchor);
}

fn write_membrane(w: &mut Writer, ann: &Annotation) {
    let Annotation::Membrane {
        id,
        x,
        y,
        width,
        height,
        color,
        text,
        mode,
        curtains,
        membrane_id,
        domains,
        mirror_of,
        temporal_anchor,
    } = ann
    else {
        return;
    };

    w.u8(KIND_MEMBRANE);
    w.text(id);
    w.f64(*x);
    w.f64(*y);
    w.f64(*width);
    w.f64(*height);
    w.opt_text(color.as_ref());
    w.opt_text(text.as_ref());
    w.u8(membrane_mode_tag(*mode));
    w.seq(curtains, write_curtain);
    w.opt_text(membrane_id.as_ref());
    w.seq(domains, write_domain_assignment);
    w.opt_text(mirror_of.as_ref());
    w.opt(temporal_anchor.as_ref(), write_temporal_anchor);
}

// ── Lecture ─────────────────────────────────────────────────────────────────

pub fn read_annotation(r: &mut Reader<'_>) -> CoreResult<Annotation> {
    match r.u8()? {
        KIND_TEXT => read_text(r),
        KIND_STICKY => read_sticky(r),
        KIND_ARROW => read_arrow(r),
        KIND_MEMBRANE => read_membrane(r),
        other => Err(unknown("variante d'annotation", other)),
    }
}

fn read_text(r: &mut Reader<'_>) -> CoreResult<Annotation> {
    Ok(Annotation::Text {
        id: r.text()?,
        x: r.f64()?,
        y: r.f64()?,
        width: r.opt_f64()?,
        height: r.opt_f64()?,
        text: r.text()?,
        font_size: r.opt_f64()?,
        color: r.opt_text()?,
        cursor_pos: r.opt(|rr| rr.count())?,
        source_file: r.opt_text()?,
        membrane_id: r.opt_text()?,
        domains: r.seq(read_domain_assignment)?,
        mirror_of: r.opt_text()?,
        temporal_anchor: r.opt(read_temporal_anchor)?,
    })
}

fn read_sticky(r: &mut Reader<'_>) -> CoreResult<Annotation> {
    Ok(Annotation::Sticky {
        id: r.text()?,
        x: r.f64()?,
        y: r.f64()?,
        width: r.opt_f64()?,
        height: r.opt_f64()?,
        text: r.text()?,
        font_size: r.opt_f64()?,
        color: r.opt_text()?,
        bg_color: r.opt_text()?,
        cursor_pos: r.opt(|rr| rr.count())?,
        operator: r.opt(|rr| operator_from_tag(rr.u8()?))?,
        source_file: r.opt_text()?,
        membrane_id: r.opt_text()?,
        domains: r.seq(read_domain_assignment)?,
        mirror_of: r.opt_text()?,
        temporal_anchor: r.opt(read_temporal_anchor)?,
    })
}

fn read_arrow(r: &mut Reader<'_>) -> CoreResult<Annotation> {
    Ok(Annotation::Arrow {
        id: r.text()?,
        x: r.f64()?,
        y: r.f64()?,
        x2: r.f64()?,
        y2: r.f64()?,
        text: r.opt_text()?,
        font_size: r.opt_f64()?,
        color: r.opt_text()?,
        arrow_type: r.opt_text()?,
        arrow_bidirectional: r.flag()?,
        predicate: r.opt(|rr| predicate_from_tag(rr.u8()?))?,
        stroke_width: r.opt_f64()?,
        waypoints: r.seq(read_point)?,
        source_id: r.opt_text()?,
        target_id: r.opt_text()?,
        source_block_id: r.opt_text()?,
        target_block_id: r.opt_text()?,
        source_text_sel: r.opt(read_text_selection)?,
        target_text_sel: r.opt(read_text_selection)?,
        long_text: r.opt_text()?,
        target_board_id: r.opt_text()?,
        membrane_id: r.opt_text()?,
        domains: r.seq(read_domain_assignment)?,
        mirror_of: r.opt_text()?,
        temporal_anchor: r.opt(read_temporal_anchor)?,
    })
}

fn read_membrane(r: &mut Reader<'_>) -> CoreResult<Annotation> {
    Ok(Annotation::Membrane {
        id: r.text()?,
        x: r.f64()?,
        y: r.f64()?,
        width: r.f64()?,
        height: r.f64()?,
        color: r.opt_text()?,
        text: r.opt_text()?,
        mode: membrane_mode_from_tag(r.u8()?)?,
        curtains: r.seq(read_curtain)?,
        membrane_id: r.opt_text()?,
        domains: r.seq(read_domain_assignment)?,
        mirror_of: r.opt_text()?,
        temporal_anchor: r.opt(read_temporal_anchor)?,
    })
}

// ── Satellites ──────────────────────────────────────────────────────────────

fn write_point(w: &mut Writer, point: &Point2D) {
    let Point2D { x, y } = point;
    w.f64(*x);
    w.f64(*y);
}

fn read_point(r: &mut Reader<'_>) -> CoreResult<Point2D> {
    Ok(Point2D {
        x: r.f64()?,
        y: r.f64()?,
    })
}

const SEL_LEGACY: u8 = 0;
const SEL_ANCHORS: u8 = 1;

fn write_text_selection(w: &mut Writer, sel: &TextSelection) {
    match sel {
        TextSelection::Legacy(raw) => {
            w.u8(SEL_LEGACY);
            w.text(raw);
        }
        TextSelection::Anchors(anchors) => {
            w.u8(SEL_ANCHORS);
            w.seq(anchors, write_text_anchor);
        }
    }
}

fn read_text_selection(r: &mut Reader<'_>) -> CoreResult<TextSelection> {
    match r.u8()? {
        SEL_LEGACY => Ok(TextSelection::Legacy(r.text()?)),
        SEL_ANCHORS => Ok(TextSelection::Anchors(r.seq(read_text_anchor)?)),
        other => Err(unknown("sélection de texte", other)),
    }
}

fn write_text_anchor(w: &mut Writer, anchor: &TextAnchor) {
    let TextAnchor {
        start,
        end,
        quote,
        prefix,
        suffix,
    } = anchor;

    w.i64(*start);
    w.i64(*end);
    w.text(quote);
    w.opt_text(prefix.as_ref());
    w.opt_text(suffix.as_ref());
}

fn read_text_anchor(r: &mut Reader<'_>) -> CoreResult<TextAnchor> {
    Ok(TextAnchor {
        start: r.i64()?,
        end: r.i64()?,
        quote: r.text()?,
        prefix: r.opt_text()?,
        suffix: r.opt_text()?,
    })
}

fn write_curtain(w: &mut Writer, curtain: &MembraneCurtain) {
    let MembraneCurtain {
        id,
        owner_id,
        owner_name,
        owner_color,
        visibility,
        editable,
        collapsed_ratio,
        expanded_ratio,
        board_id,
        notes,
        created_at,
    } = curtain;

    w.text(id);
    w.text(owner_id);
    w.text(owner_name);
    w.text(owner_color);
    w.u8(visibility_tag(*visibility));
    w.u8(editable_tag(*editable));
    w.opt_f64(*collapsed_ratio);
    w.opt_f64(*expanded_ratio);
    w.opt_text(board_id.as_ref());
    w.seq(notes, write_curtain_note);
    w.i64(*created_at);
}

fn read_curtain(r: &mut Reader<'_>) -> CoreResult<MembraneCurtain> {
    Ok(MembraneCurtain {
        id: r.text()?,
        owner_id: r.text()?,
        owner_name: r.text()?,
        owner_color: r.text()?,
        visibility: visibility_from_tag(r.u8()?)?,
        editable: editable_from_tag(r.u8()?)?,
        collapsed_ratio: r.opt_f64()?,
        expanded_ratio: r.opt_f64()?,
        board_id: r.opt_text()?,
        notes: r.seq(read_curtain_note)?,
        created_at: r.i64()?,
    })
}

fn write_curtain_note(w: &mut Writer, note: &CurtainNote) {
    let CurtainNote {
        id,
        text,
        created_at,
    } = note;
    w.text(id);
    w.text(text);
    w.i64(*created_at);
}

fn read_curtain_note(r: &mut Reader<'_>) -> CoreResult<CurtainNote> {
    Ok(CurtainNote {
        id: r.text()?,
        text: r.text()?,
        created_at: r.i64()?,
    })
}
