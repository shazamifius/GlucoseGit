//! Tests portés fidèlement de src/utils/textAnchors.test.ts

use glucose_core::text_anchors::{
    add_anchor, create_anchor, has_text_selection, normalize_text_sel, resolve_anchors,
    resolve_text_sel, ResolvedRange,
};
use glucose_core::types::{TextAnchor, TextSelection};

const PLAIN: &str = "bonjourstesttestbonjours";
const FIRST: usize = 0;
const SECOND: usize = 16; // lastIndexOf("bonjours") in "bonjourstesttestbonjours"

#[test]
fn test_create_anchor() {
    let a = create_anchor(PLAIN, SECOND, SECOND + 8).unwrap();
    assert_eq!(a.start, SECOND as i64);
    assert_eq!(a.end, (SECOND + 8) as i64);
    assert_eq!(a.quote, "bonjours");
    assert_eq!(a.prefix.as_deref(), Some("bonjourstesttest"));
    assert_eq!(a.suffix.as_deref(), Some(""));

    let text = "a  mot  b";
    let a2 = create_anchor(text, 1, 7).unwrap();
    assert_eq!(a2.start, 3);
    assert_eq!(a2.end, 6);
    assert_eq!(a2.quote, "mot");

    // Rejette sélection vide
    assert!(create_anchor(text, 1, 3).is_none());
}

#[test]
fn test_resolve_anchors_original_bug() {
    let anchor = create_anchor(PLAIN, SECOND, SECOND + 8).unwrap();
    assert_eq!(
        resolve_anchors(PLAIN, &[anchor]),
        vec![ResolvedRange { start: SECOND, end: SECOND + 8 }]
    );

    let first = create_anchor(PLAIN, FIRST, FIRST + 8).unwrap();
    assert_eq!(
        resolve_anchors(PLAIN, &[first]),
        vec![ResolvedRange { start: FIRST, end: FIRST + 8 }]
    );

    let mut anchors = Vec::new();
    anchors = add_anchor(anchors, create_anchor(PLAIN, FIRST, FIRST + 8).unwrap());
    anchors = add_anchor(anchors, create_anchor(PLAIN, SECOND, SECOND + 8).unwrap());
    assert_eq!(anchors.len(), 2);
    assert_eq!(
        resolve_anchors(PLAIN, &anchors),
        vec![
            ResolvedRange { start: FIRST, end: FIRST + 8 },
            ResolvedRange { start: SECOND, end: SECOND + 8 },
        ]
    );

    // Ignore ajout chevauchant
    let overlapping = add_anchor(
        vec![create_anchor(PLAIN, FIRST, FIRST + 8).unwrap()],
        TextAnchor {
            start: (FIRST + 3) as i64,
            end: (FIRST + 6) as i64,
            quote: "jou".into(),
            prefix: None,
            suffix: None,
        },
    );
    assert_eq!(overlapping.len(), 1);

    // Abandonne ancre dont la citation a disparu
    let absent = TextAnchor {
        start: 0,
        end: 5,
        quote: "absent".into(),
        prefix: None,
        suffix: None,
    };
    assert_eq!(resolve_anchors(PLAIN, &[absent]), vec![]);

    // Fusionne les plages qui se recouvrent
    let ranges = resolve_anchors(
        "abcdef",
        &[
            TextAnchor { start: 0, end: 3, quote: "abc".into(), prefix: None, suffix: None },
            TextAnchor { start: 2, end: 5, quote: "cde".into(), prefix: None, suffix: None },
        ],
    );
    assert_eq!(ranges, vec![ResolvedRange { start: 0, end: 5 }]);
}

#[test]
fn test_resolve_anchors_reanchor_after_edit() {
    let anchor = create_anchor(PLAIN, SECOND, SECOND + 8).unwrap();
    let edited = format!("préface {}", PLAIN);
    let shifted = SECOND + "préface ".len();
    assert_eq!(
        resolve_anchors(&edited, &[anchor.clone()]),
        vec![ResolvedRange { start: shifted, end: shifted + 8 }]
    );

    // Tranche entre occurrences homonymes grâce au contexte
    let edited2 = format!("XX{}", PLAIN);
    assert_eq!(
        resolve_anchors(&edited2, &[anchor]),
        vec![ResolvedRange { start: SECOND + 2, end: SECOND + 10 }]
    );

    // Privilégie la position tant qu'elle reste valide
    let first = create_anchor(PLAIN, FIRST, FIRST + 8).unwrap();
    assert_eq!(
        resolve_anchors(PLAIN, &[first]),
        vec![ResolvedRange { start: FIRST, end: FIRST + 8 }]
    );
}

#[test]
fn test_normalize_and_resolve_legacy_text_sel() {
    let legacy = TextSelection::Legacy("bonjours ‖ test".into());
    let normalized = normalize_text_sel(Some(&legacy));
    assert_eq!(
        normalized,
        vec![
            TextAnchor { start: -1, end: -1, quote: "bonjours".into(), prefix: None, suffix: None },
            TextAnchor { start: -1, end: -1, quote: "test".into(), prefix: None, suffix: None },
        ]
    );

    let resolved = resolve_text_sel(PLAIN, Some(&TextSelection::Legacy("bonjours".into())));
    assert_eq!(resolved, vec![ResolvedRange { start: FIRST, end: FIRST + 8 }]);

    assert!(!has_text_selection(None));
    assert!(!has_text_selection(Some(&TextSelection::Anchors(vec![]))));
    assert!(has_text_selection(Some(&TextSelection::Legacy("bonjours".into()))));
}
