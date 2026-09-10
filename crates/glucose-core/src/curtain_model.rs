//! MEMB-3 — Rideaux : données, propriété et permissions (0 dépendance).

use crate::curtain_panel::{normalize_config, CurtainConfig};
use crate::types::{
    Annotation, CurtainEditable, CurtainNote, CurtainVisibility, MembraneCurtain,
};

pub const MAX_NOTE_LENGTH: usize = 2000;

pub mod curtain_board_consts {
    pub const MARGIN: f64 = 40.0;
    pub const BLOCK_W: f64 = 320.0;
    pub const BLOCK_STEP: f64 = 88.0;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurtainOwner {
    pub id: String,
    pub name: String,
    pub color: String,
}

pub fn sanitize_note_text(raw: &str) -> String {
    let replaced = raw.replace("\r\n", "\n");
    let trimmed = replaced.trim();
    if trimmed.len() > MAX_NOTE_LENGTH {
        trimmed[..MAX_NOTE_LENGTH].to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn create_curtain(id: impl Into<String>, owner: CurtainOwner, now: i64) -> MembraneCurtain {
    MembraneCurtain {
        id: id.into(),
        owner_id: owner.id,
        owner_name: owner.name,
        owner_color: owner.color,
        visibility: CurtainVisibility::Private,
        editable: CurtainEditable::Owner,
        collapsed_ratio: None,
        expanded_ratio: None,
        board_id: None,
        notes: Vec::new(),
        created_at: now,
    }
}

pub fn create_note(id: impl Into<String>, text: &str, now: i64) -> CurtainNote {
    CurtainNote {
        id: id.into(),
        text: sanitize_note_text(text),
        created_at: now,
    }
}

pub fn notes_to_annotations(notes: &[CurtainNote]) -> Vec<Annotation> {
    let mut out = Vec::new();
    let mut y = curtain_board_consts::MARGIN;
    for n in notes {
        let text = sanitize_note_text(&n.text);
        if text.is_empty() {
            continue;
        }
        out.push(Annotation::Text {
            id: format!("{}-b", n.id),
            x: curtain_board_consts::MARGIN,
            y,
            width: Some(curtain_board_consts::BLOCK_W),
            height: None,
            text,
            font_size: Some(14.0),
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        });
        y += curtain_board_consts::BLOCK_STEP;
    }
    out
}

pub fn detach_curtain(c: &MembraneCurtain) -> MembraneCurtain {
    MembraneCurtain {
        id: c.id.clone(),
        owner_id: c.owner_id.clone(),
        owner_name: c.owner_name.clone(),
        owner_color: c.owner_color.clone(),
        visibility: c.visibility,
        editable: c.editable,
        collapsed_ratio: c.collapsed_ratio,
        expanded_ratio: c.expanded_ratio,
        board_id: c.board_id.clone(),
        notes: c
            .notes
            .iter()
            .map(|n| CurtainNote {
                id: n.id.clone(),
                text: n.text.clone(),
                created_at: n.created_at,
            })
            .collect(),
        created_at: c.created_at,
    }
}

pub fn detach_curtains(list: &[MembraneCurtain]) -> Vec<MembraneCurtain> {
    list.iter().map(detach_curtain).collect()
}

pub fn can_see(curtain: &MembraneCurtain, user_id: &str) -> bool {
    curtain.owner_id == user_id || curtain.visibility == CurtainVisibility::Shared
}

pub fn can_edit(curtain: &MembraneCurtain, user_id: &str) -> bool {
    if curtain.owner_id == user_id {
        return true;
    }
    curtain.visibility == CurtainVisibility::Shared && curtain.editable == CurtainEditable::Everyone
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurtainKind {
    Carnet,
    Vitrine,
    Atelier,
}

pub fn curtain_kind(curtain: &MembraneCurtain) -> CurtainKind {
    if curtain.visibility == CurtainVisibility::Private {
        CurtainKind::Carnet
    } else if curtain.editable == CurtainEditable::Everyone {
        CurtainKind::Atelier
    } else {
        CurtainKind::Vitrine
    }
}

pub fn visible_curtains(curtains: &[MembraneCurtain], user_id: &str) -> Vec<MembraneCurtain> {
    let mut visible: Vec<MembraneCurtain> = curtains
        .iter()
        .filter(|c| can_see(c, user_id))
        .cloned()
        .collect();

    visible.sort_by(|a, b| {
        let mine_a = a.owner_id == user_id;
        let mine_b = b.owner_id == user_id;
        mine_b
            .cmp(&mine_a)
            .then_with(|| a.created_at.cmp(&b.created_at))
    });

    visible
}

pub fn config_of(curtain: Option<&MembraneCurtain>) -> CurtainConfig {
    normalize_config(curtain.map(|c| CurtainConfig {
        collapsed: c.collapsed_ratio.unwrap_or(0.1),
        expanded: c.expanded_ratio.unwrap_or(0.9),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissions() {
        let owner = CurtainOwner {
            id: "alice".into(),
            name: "Alice".into(),
            color: "#ff0000".into(),
        };
        let mut c = create_curtain("c1", owner, 100);

        // Privé : seul Alice voit et édite
        assert!(can_see(&c, "alice"));
        assert!(can_edit(&c, "alice"));
        assert!(!can_see(&c, "bob"));
        assert!(!can_edit(&c, "bob"));
        assert_eq!(curtain_kind(&c), CurtainKind::Carnet);

        // Vitrine : partagé + owner
        c.visibility = CurtainVisibility::Shared;
        c.editable = CurtainEditable::Owner;
        assert!(can_see(&c, "bob"));
        assert!(!can_edit(&c, "bob"));
        assert_eq!(curtain_kind(&c), CurtainKind::Vitrine);

        // Atelier : partagé + everyone
        c.editable = CurtainEditable::Everyone;
        assert!(can_see(&c, "bob"));
        assert!(can_edit(&c, "bob"));
        assert_eq!(curtain_kind(&c), CurtainKind::Atelier);
    }

    #[test]
    fn test_visible_curtains_sorting() {
        let owner1 = CurtainOwner { id: "alice".into(), name: "A".into(), color: "red".into() };
        let owner2 = CurtainOwner { id: "bob".into(), name: "B".into(), color: "blue".into() };

        let mut c1 = create_curtain("c1", owner1, 100);
        c1.visibility = CurtainVisibility::Shared;
        let mut c2 = create_curtain("c2", owner2, 200);
        c2.visibility = CurtainVisibility::Shared;

        let visible = visible_curtains(&[c1, c2], "bob");
        // Bob voit le sien (c2) en premier même si créé après
        assert_eq!(visible[0].id, "c2");
        assert_eq!(visible[1].id, "c1");
    }
}
