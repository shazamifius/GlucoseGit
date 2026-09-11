//! Instrumentation de performance optionnelle, activée par `GLUCOSE_PERF=1`.
//!
//! Aucune allocation ni aucun appel à `Instant::now()` n'est effectué lorsque la
//! variable d'environnement est absente : le coût est alors une simple lecture
//! d'un booléen mis en cache.

use std::cell::{Cell, RefCell};
use std::sync::OnceLock;
use std::time::Instant;

static LEVEL: OnceLock<u8> = OnceLock::new();

/// Niveau de trace : 0 = désactivé, 1 = une ligne par frame, 2 = chaque étape en direct.
fn level() -> u8 {
    *LEVEL.get_or_init(|| match std::env::var("GLUCOSE_PERF") {
        Ok(value) => {
            let v = value.trim();
            if v.is_empty() || v == "0" || v.eq_ignore_ascii_case("false") {
                0
            } else if v == "2" {
                2
            } else {
                1
            }
        }
        Err(_) => 0,
    })
}

/// Indique si la trace de performance est demandée (`GLUCOSE_PERF=1`).
pub fn enabled() -> bool {
    level() > 0
}

thread_local! {
    static FRAME_START: Cell<Option<Instant>> = const { Cell::new(None) };
    static LAST_MARK: Cell<Option<Instant>> = const { Cell::new(None) };
    static STAGES: RefCell<Vec<(&'static str, f64)>> = const { RefCell::new(Vec::new()) };
    static FRAME_INDEX: Cell<u64> = const { Cell::new(0) };
}

/// Ouvre une nouvelle frame de mesure.
pub fn frame_begin() {
    if !enabled() {
        return;
    }
    let now = Instant::now();
    if level() >= 2 {
        eprintln!("[perf] --- frame begin ---");
    }
    FRAME_START.with(|c| c.set(Some(now)));
    LAST_MARK.with(|c| c.set(Some(now)));
    STAGES.with(|s| s.borrow_mut().clear());
}

/// Enregistre la durée écoulée depuis le repère précédent sous le nom `label`.
pub fn stage(label: &'static str) {
    if !enabled() {
        return;
    }
    let now = Instant::now();
    let previous = LAST_MARK.with(|c| c.replace(Some(now)));
    if let Some(prev) = previous {
        let ms = now.duration_since(prev).as_secs_f64() * 1000.0;
        if level() >= 2 {
            eprintln!("[perf]   {label}={ms:.2}ms");
        }
        STAGES.with(|s| s.borrow_mut().push((label, ms)));
    }
}

/// Clôt la frame courante et écrit la ligne de trace sur stderr.
pub fn frame_end() {
    if !enabled() {
        return;
    }
    let total_ms = FRAME_START
        .with(|c| c.replace(None))
        .map(|start| start.elapsed().as_secs_f64() * 1000.0)
        .unwrap_or(0.0);
    LAST_MARK.with(|c| c.set(None));
    let index = FRAME_INDEX.with(|c| {
        let next = c.get().wrapping_add(1);
        c.set(next);
        next
    });
    let detail = STAGES.with(|s| {
        s.borrow()
            .iter()
            .map(|(label, ms)| format!("{label}={ms:.2}"))
            .collect::<Vec<_>>()
            .join(" ")
    });
    eprintln!("[perf] frame #{index} total={total_ms:.2}ms {detail}");
}

/// Écrit une mesure ponctuelle hors frame (démarrage, chargement, etc.).
pub fn event(label: &str, started: Instant) {
    if !enabled() {
        return;
    }
    let ms = started.elapsed().as_secs_f64() * 1000.0;
    eprintln!("[perf] {label}={ms:.2}ms");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_api_is_safe_whatever_the_level() {
        // Activée ou non, l'API ne doit ni paniquer ni laisser d'état résiduel.
        frame_begin();
        stage("etape-a");
        stage("etape-b");
        frame_end();
        event("evenement", Instant::now());

        // Une étape hors frame est ignorée silencieusement.
        stage("orpheline");
        assert_eq!(enabled(), level() > 0);
    }
}
