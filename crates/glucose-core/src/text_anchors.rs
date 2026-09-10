//! Ancrage d'une sélection de texte par position et ré-ancrage résilient — 0 dépendance.

use crate::types::{TextAnchor, TextSelection};

pub const LEGACY_SEP: &str = " ‖ ";
pub const CONTEXT_LEN: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedRange {
    pub start: usize,
    pub end: usize,
}

pub fn create_anchor(plain: &str, start: usize, end: usize) -> Option<TextAnchor> {
    let chars: Vec<char> = plain.chars().collect();
    let mut s = start.min(chars.len());
    let mut e = end.min(chars.len());
    if e < s {
        std::mem::swap(&mut s, &mut e);
    }
    while s < e && chars[s].is_whitespace() {
        s += 1;
    }
    while e > s && chars[e - 1].is_whitespace() {
        e -= 1;
    }
    if e <= s {
        return None;
    }

    let quote: String = chars[s..e].iter().collect();
    let prefix_start = s.saturating_sub(CONTEXT_LEN);
    let prefix: String = chars[prefix_start..s].iter().collect();
    let suffix_end = (e + CONTEXT_LEN).min(chars.len());
    let suffix: String = chars[e..suffix_end].iter().collect();

    Some(TextAnchor {
        start: s as i64,
        end: e as i64,
        quote,
        prefix: Some(prefix),
        suffix: Some(suffix),
    })
}

pub fn add_anchor(mut anchors: Vec<TextAnchor>, anchor: TextAnchor) -> Vec<TextAnchor> {
    let overlaps = anchors.iter().any(|a| a.start < anchor.end && anchor.start < a.end);
    if overlaps {
        return anchors;
    }
    anchors.push(anchor);
    anchors.sort_by_key(|a| a.start);
    anchors
}

pub fn normalize_text_sel(sel: Option<&TextSelection>) -> Vec<TextAnchor> {
    match sel {
        None => Vec::new(),
        Some(TextSelection::Anchors(list)) => list.clone(),
        Some(TextSelection::Legacy(s)) => s
            .split(LEGACY_SEP)
            .map(str::trim)
            .filter(|q| !q.is_empty())
            .map(|quote| TextAnchor {
                start: -1,
                end: -1,
                quote: quote.to_string(),
                prefix: None,
                suffix: None,
            })
            .collect(),
    }
}

pub fn has_text_selection(sel: Option<&TextSelection>) -> bool {
    !normalize_text_sel(sel).is_empty()
}

fn indexes_of(hay: &str, needle: &str) -> Vec<usize> {
    if needle.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut start = 0;
    while let Some(pos) = hay[start..].find(needle) {
        let abs_pos = start + pos;
        out.push(abs_pos);
        start = abs_pos + needle.len();
    }
    out
}

fn common_suffix_len(a: &str, b: &str) -> usize {
    a.chars().rev().zip(b.chars().rev()).take_while(|(c1, c2)| c1 == c2).count()
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(c1, c2)| c1 == c2).count()
}

fn reanchor(plain: &str, anchor: &TextAnchor) -> Option<ResolvedRange> {
    let quote = &anchor.quote;
    if quote.is_empty() {
        return None;
    }

    let mut candidates = indexes_of(plain, quote);
    let len = quote.len();
    if candidates.is_empty() {
        let plain_lower = plain.to_lowercase();
        let quote_lower = quote.to_lowercase();
        candidates = indexes_of(&plain_lower, &quote_lower);
        if candidates.is_empty() {
            return None;
        }
    }

    let prefix = anchor.prefix.as_deref().unwrap_or("");
    let suffix = anchor.suffix.as_deref().unwrap_or("");
    let origin = if anchor.start >= 0 { anchor.start as usize } else { 0 };

    let mut best = candidates[0];
    let mut best_score = -1i32;
    let mut best_dist = usize::MAX;

    for &i in &candidates {
        let left_start = i.saturating_sub(CONTEXT_LEN);
        let left = &plain[left_start..i];
        let right_end = (i + len + CONTEXT_LEN).min(plain.len());
        let right = &plain[i + len..right_end];

        let score = (common_suffix_len(left, prefix) + common_prefix_len(right, suffix)) as i32;
        let dist = (i as isize - origin as isize).unsigned_abs();

        if score > best_score || (score == best_score && dist < best_dist) {
            best = i;
            best_score = score;
            best_dist = dist;
        }
    }

    Some(ResolvedRange {
        start: best,
        end: best + len,
    })
}

pub fn resolve_anchors(plain: &str, anchors: &[TextAnchor]) -> Vec<ResolvedRange> {
    let mut ranges = Vec::new();
    for anchor in anchors {
        if anchor.start >= 0
            && (anchor.end as usize) <= plain.len()
            && anchor.end > anchor.start
            && &plain[anchor.start as usize..anchor.end as usize] == anchor.quote
        {
            ranges.push(ResolvedRange {
                start: anchor.start as usize,
                end: anchor.end as usize,
            });
            continue;
        }
        if let Some(found) = reanchor(plain, anchor) {
            ranges.push(found);
        }
    }

    ranges.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.end.cmp(&b.end)));
    let mut merged: Vec<ResolvedRange> = Vec::new();
    for r in ranges {
        if let Some(last) = merged.last_mut() {
            if r.start <= last.end {
                last.end = last.end.max(r.end);
                continue;
            }
        }
        merged.push(r);
    }
    merged
}

pub fn resolve_text_sel(plain: &str, sel: Option<&TextSelection>) -> Vec<ResolvedRange> {
    resolve_anchors(plain, &normalize_text_sel(sel))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_resolve_anchor() {
        let text = "Bonjour le monde magnifique !";
        let anchor = create_anchor(text, 11, 16).expect("should create anchor"); // "monde"
        assert_eq!(anchor.quote, "monde");

        let resolved = resolve_anchors(text, &[anchor]);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0], ResolvedRange { start: 11, end: 16 });
    }

    #[test]
    fn test_reanchor_after_edit() {
        let text1 = "Bonjour le monde magnifique !";
        let anchor = create_anchor(text1, 11, 16).unwrap();

        // On insère du texte au début
        let text2 = "Salut à tous ! Bonjour le monde magnifique !";
        let resolved = resolve_anchors(text2, &[anchor]);
        assert_eq!(resolved.len(), 1);
        assert_eq!(&text2[resolved[0].start..resolved[0].end], "monde");
    }
}
