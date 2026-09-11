//! PICK-1 — cycle de selection : re-cliquer au meme endroit descend dans la pile.

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct CycleState {
    pub sx: f64,
    pub sy: f64,
    pub sig: String,
    pub index: usize,
    pub t: i64,
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PickOptions {
    pub alt: bool,
    pub multi: bool,
}

fn signature_of(cands: &[PickCandidate]) -> String {
    let mut parts = Vec::with_capacity(cands.len());
    for c in cands {
        parts.push(format!("{}:{}:{}", c.owner.as_str(), c.id, c.kind.as_str()));
    }
    parts.join("|")
}

fn cyclable_of(candidates: &[PickCandidate]) -> Vec<PickCandidate> {
    candidates
        .iter()
        .filter(|c| c.rank != PICK_RANK_HANDLE)
        .cloned()
        .collect()
}

pub fn pick_at_down(
    candidates: &[PickCandidate],
    prev: Option<&CycleState>,
    sx: f64,
    sy: f64,
    now: i64,
    opts: PickOptions,
) -> (Option<PickCandidate>, Option<CycleState>) {
    if candidates.is_empty() {
        return (None, None);
    }

    let handles: Vec<&PickCandidate> = candidates
        .iter()
        .filter(|c| c.rank == PICK_RANK_HANDLE)
        .collect();
    let cyclable = cyclable_of(candidates);
    let sig = signature_of(&cyclable);

    let dt = match prev {
        Some(p) => now - p.t,
        None => i64::MAX,
    };

    let same_spot = if let Some(p) = prev {
        p.sig == sig
            && dt <= pick_consts::CYCLE_TTL_MS
            && f64::hypot(sx - p.sx, sy - p.sy) <= pick_consts::CYCLE_RADIUS_PX
            && !opts.multi
    } else {
        false
    };

    let repeat = same_spot && dt >= pick_consts::DBLCLICK_MS && !opts.alt;

    let mut index = if same_spot {
        let p = prev.unwrap();
        p.index.min(cyclable.len().saturating_sub(1))
    } else {
        0
    };

    if opts.alt && same_spot && cyclable.len() > 1 {
        index = (index + 1) % cyclable.len();
    }

    let cycle = CycleState {
        sx,
        sy,
        sig,
        index,
        t: now,
        repeat,
    };

    if !handles.is_empty() && !opts.alt {
        return (Some((*handles[0]).clone()), Some(cycle));
    }

    if cyclable.is_empty() {
        let picked = handles.first().map(|&h| h.clone());
        return (picked, Some(cycle));
    }

    let picked = cyclable.get(index).or_else(|| cyclable.first()).cloned();
    (picked, Some(cycle))
}

pub fn advance_on_release(
    candidates: &[PickCandidate],
    cycle: Option<&CycleState>,
    now: i64,
) -> (Option<PickCandidate>, Option<CycleState>) {
    let c = match cycle {
        Some(c) => c,
        None => return (None, None),
    };

    let settled = CycleState {
        t: now,
        repeat: false,
        ..c.clone()
    };

    if !c.repeat {
        return (None, Some(settled));
    }

    let cyclable = cyclable_of(candidates);
    if signature_of(&cyclable) != c.sig || cyclable.len() < 2 {
        return (None, Some(settled));
    }

    if let Some(target) = cyclable.get(c.index) {
        if target.terminal {
            return (None, Some(settled));
        }
    }

    let index = (c.index + 1) % cyclable.len();
    let picked = cyclable.get(index).cloned();
    let next_cycle = CycleState {
        index,
        ..settled
    };
    (picked, Some(next_cycle))
}
