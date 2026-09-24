//! Les annotations de Tauri — cartes, pense-bêtes, flèches, membranes — traduites une à une.
//!
//! Une annotation de Tauri est une union discriminée par son champ `type` ; celle de Rust est
//! une énumération. Un `type` inconnu n'est pas deviné : l'annotation est omise, et comptée.

use super::{ancre_temporelle, assignations, option, texte, Rapport};
use crate::persist::tauri::Valeur;
use crate::types::{
    Annotation, ArrowPredicate, CurtainEditable, CurtainNote, CurtainVisibility, MembraneCurtain,
    MembraneMode, Point2D, StickyOperator, TextAnchor, TextSelection,
};

pub fn annotation(a: &Valeur, r: &mut Rapport) -> Option<Annotation> {
    let Some(id) = a.texte("id") else {
        r.omettre("annotation sans identifiant");
        return None;
    };
    let id = id.to_string();
    let (x, y) = (a.nombre("x").unwrap_or(0.0), a.nombre("y").unwrap_or(0.0));
    Some(match a.texte("type") {
        Some("text") => Annotation::Text {
            id,
            x,
            y,
            width: a.nombre("width"),
            height: a.nombre("height"),
            text: texte(a, "text"),
            font_size: a.nombre("fontSize"),
            color: option(a, "color"),
            cursor_pos: curseur(a),
            source_file: option(a, "sourceFile"),
            membrane_id: option(a, "membraneId"),
            domains: assignations(a),
            mirror_of: option(a, "mirrorOf"),
            temporal_anchor: a.champ("temporalAnchor").map(ancre_temporelle),
        },
        Some("sticky") => Annotation::Sticky {
            id,
            x,
            y,
            width: a.nombre("width"),
            height: a.nombre("height"),
            text: texte(a, "text"),
            font_size: a.nombre("fontSize"),
            color: option(a, "color"),
            bg_color: option(a, "bgColor"),
            cursor_pos: curseur(a),
            operator: a.texte("operator").and_then(operateur),
            source_file: option(a, "sourceFile"),
            membrane_id: option(a, "membraneId"),
            domains: assignations(a),
            mirror_of: option(a, "mirrorOf"),
            temporal_anchor: a.champ("temporalAnchor").map(ancre_temporelle),
        },
        Some("arrow") => fleche(id, x, y, a),
        Some("membrane") => membrane(id, x, y, a),
        _ => {
            r.omettre("annotation d'un type inconnu");
            return None;
        }
    })
}

fn curseur(a: &Valeur) -> Option<usize> {
    a.entier("cursorPos").and_then(|n| usize::try_from(n).ok())
}

fn operateur(s: &str) -> Option<StickyOperator> {
    Some(match s {
        "AND" => StickyOperator::And,
        "OR" => StickyOperator::Or,
        "BUT" => StickyOperator::But,
        "BECAUSE" => StickyOperator::Because,
        _ => return None,
    })
}

fn predicat(s: &str) -> Option<ArrowPredicate> {
    ArrowPredicate::ALL.into_iter().find(|p| p.as_str() == s)
}

fn fleche(id: String, x: f64, y: f64, a: &Valeur) -> Annotation {
    Annotation::Arrow {
        id,
        x,
        y,
        x2: a.nombre("x2").unwrap_or(x),
        y2: a.nombre("y2").unwrap_or(y),
        text: option(a, "text"),
        font_size: a.nombre("fontSize"),
        color: option(a, "color"),
        arrow_type: option(a, "arrowType"),
        arrow_bidirectional: a.booleen("arrowBidirectional").unwrap_or(false),
        predicate: a.texte("predicate").and_then(predicat),
        stroke_width: a.nombre("strokeWidth"),
        waypoints: a
            .liste("waypoints")
            .iter()
            .filter_map(|p| {
                Some(Point2D {
                    x: p.nombre("x")?,
                    y: p.nombre("y")?,
                })
            })
            .collect(),
        source_id: option(a, "sourceId"),
        target_id: option(a, "targetId"),
        source_block_id: option(a, "sourceBlockId"),
        target_block_id: option(a, "targetBlockId"),
        source_text_sel: a.champ("sourceTextSel").and_then(selection),
        target_text_sel: a.champ("targetTextSel").and_then(selection),
        long_text: option(a, "longText"),
        target_board_id: option(a, "targetBoardId"),
        membrane_id: option(a, "membraneId"),
        domains: assignations(a),
        mirror_of: option(a, "mirrorOf"),
        temporal_anchor: a.champ("temporalAnchor").map(ancre_temporelle),
    }
}

/// Une sélection de texte : une chaîne (l'encodage d'avant les ancres) ou des ancres.
fn selection(v: &Valeur) -> Option<TextSelection> {
    match v {
        Valeur::Texte(s) => Some(TextSelection::Legacy(s.clone())),
        Valeur::Liste(ancres) => Some(TextSelection::Anchors(
            ancres
                .iter()
                .map(|a| TextAnchor {
                    start: a.entier("start").unwrap_or(-1),
                    end: a.entier("end").unwrap_or(-1),
                    quote: texte(a, "quote"),
                    prefix: option(a, "prefix"),
                    suffix: option(a, "suffix"),
                })
                .collect(),
        )),
        _ => None,
    }
}

fn membrane(id: String, x: f64, y: f64, a: &Valeur) -> Annotation {
    Annotation::Membrane {
        id,
        x,
        y,
        width: a.nombre("width").unwrap_or(0.0),
        height: a.nombre("height").unwrap_or(0.0),
        color: option(a, "color"),
        text: option(a, "text"),
        mode: match a.texte("mode") {
            Some("minimized") => MembraneMode::Minimized,
            Some("stretched") => MembraneMode::Stretched,
            _ => MembraneMode::Classic,
        },
        curtains: a.liste("curtains").iter().map(rideau).collect(),
        membrane_id: option(a, "membraneId"),
        domains: assignations(a),
        mirror_of: option(a, "mirrorOf"),
        temporal_anchor: a.champ("temporalAnchor").map(ancre_temporelle),
    }
}

fn rideau(c: &Valeur) -> MembraneCurtain {
    MembraneCurtain {
        id: texte(c, "id"),
        owner_id: texte(c, "ownerId"),
        owner_name: texte(c, "ownerName"),
        owner_color: texte(c, "ownerColor"),
        visibility: match c.texte("visibility") {
            Some("shared") => CurtainVisibility::Shared,
            _ => CurtainVisibility::Private,
        },
        editable: match c.texte("editable") {
            Some("everyone") => CurtainEditable::Everyone,
            _ => CurtainEditable::Owner,
        },
        collapsed_ratio: c.nombre("collapsedRatio"),
        expanded_ratio: c.nombre("expandedRatio"),
        board_id: option(c, "boardId"),
        notes: c
            .liste("notes")
            .iter()
            .map(|n| CurtainNote {
                id: texte(n, "id"),
                text: texte(n, "text"),
                created_at: n.entier("createdAt").unwrap_or(0),
            })
            .collect(),
        created_at: c.entier("createdAt").unwrap_or(0),
    }
}
