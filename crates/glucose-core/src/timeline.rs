//! Phase 6 — Réglette Temporelle Sémantique (0 dépendance).

use crate::types::TemporalAnchor;

pub const YEAR_MIN: i64 = -100_000_000;
pub const YEAR_MAX: i64 = 3_000;

#[derive(Debug, Clone, PartialEq)]
pub struct NamedEra {
    pub name: &'static str,
    pub start: i64,
    pub end: i64,
    pub description: Option<&'static str>,
}

pub const DEFAULT_ERAS: &[NamedEra] = &[
    NamedEra { name: "Crétacé", start: -145_000_000, end: -66_000_000, description: Some("Dinosaures, premiers oiseaux") },
    NamedEra { name: "Paléogène", start: -66_000_000, end: -23_000_000, description: Some("Émergence des mammifères") },
    NamedEra { name: "Néogène", start: -23_000_000, end: -2_580_000, description: Some("Hominidés primitifs") },
    NamedEra { name: "Pléistocène", start: -2_580_000, end: -11_700, description: Some("Glaciations, Homo sapiens") },
    NamedEra { name: "Holocène", start: -11_700, end: 2026, description: Some("Notre époque géologique") },

    NamedEra { name: "Paléolithique", start: -3_300_000, end: -10_000, description: Some("Pierre taillée") },
    NamedEra { name: "Néolithique", start: -10_000, end: -3_300, description: Some("Agriculture, sédentarité") },

    NamedEra { name: "Antiquité", start: -3_300, end: 476, description: Some("De l'écriture à la chute de Rome") },
    NamedEra { name: "Égypte ancienne", start: -3_150, end: -30, description: Some("Des premières dynasties à Cléopâtre") },
    NamedEra { name: "Grèce antique", start: -800, end: -146, description: Some("Cités-États, Alexandre") },
    NamedEra { name: "République romaine", start: -509, end: -27, description: None },
    NamedEra { name: "Empire romain", start: -27, end: 476, description: None },

    NamedEra { name: "Moyen Âge", start: 476, end: 1453, description: Some("De la chute de Rome à celle de Constantinople") },
    NamedEra { name: "Haut Moyen Âge", start: 476, end: 1000, description: None },
    NamedEra { name: "Moyen Âge central", start: 1000, end: 1300, description: None },
    NamedEra { name: "Bas Moyen Âge", start: 1300, end: 1453, description: None },

    NamedEra { name: "Renaissance", start: 1400, end: 1600, description: Some("Humanisme, redécouverte de l'antique") },
    NamedEra { name: "Lumières", start: 1715, end: 1789, description: Some("Raison, encyclopédie, droits naturels") },
    NamedEra { name: "Révolution française", start: 1789, end: 1799, description: None },
    NamedEra { name: "Empire napoléonien", start: 1804, end: 1815, description: None },
    NamedEra { name: "Révolution industrielle", start: 1760, end: 1840, description: None },

    NamedEra { name: "Belle Époque", start: 1871, end: 1914, description: None },
    NamedEra { name: "Première Guerre mondiale", start: 1914, end: 1918, description: None },
    NamedEra { name: "Entre-deux-guerres", start: 1918, end: 1939, description: None },
    NamedEra { name: "Seconde Guerre mondiale", start: 1939, end: 1945, description: None },
    NamedEra { name: "Guerre froide", start: 1947, end: 1991, description: None },
    NamedEra { name: "Ère numérique", start: 1990, end: 2026, description: Some("Web, mobile, IA générative") },
];

pub fn clamp_year(year: i64) -> i64 {
    year.clamp(YEAR_MIN, YEAR_MAX)
}

pub fn format_year(year: i64) -> String {
    if year >= 0 {
        return year.to_string();
    }
    let abs = -year;
    if abs < 10_000 {
        format!("{} av. J.-C.", abs)
    } else if abs < 1_000_000 {
        let ka = abs as f64 / 1_000.0;
        if abs >= 100_000 {
            format!("{:.0} ka", ka)
        } else {
            format!("{:.1} ka", ka)
        }
    } else {
        let ma = abs as f64 / 1_000_000.0;
        if abs >= 100_000_000 {
            format!("{:.0} Ma", ma)
        } else {
            format!("{:.1} Ma", ma)
        }
    }
}

pub fn format_anchor(a: &TemporalAnchor) -> String {
    if let Some(ref label) = a.label {
        return label.clone();
    }
    if a.start == a.end {
        format_year(a.start)
    } else {
        format!("{} – {}", format_year(a.start), format_year(a.end))
    }
}

fn parse_single_year(input: &str) -> Option<i64> {
    let s = input.trim().to_lowercase();
    let s = s.split_whitespace().collect::<Vec<&str>>().join(" ");

    if s.ends_with("ma") {
        let num_part = s[..s.len() - 2].trim().replace(',', ".");
        if let Ok(val) = num_part.parse::<f64>() {
            return Some(-(val * 1_000_000.0).round() as i64);
        }
    }

    if s.ends_with("ka") {
        let num_part = s[..s.len() - 2].trim().replace(',', ".");
        if let Ok(val) = num_part.parse::<f64>() {
            return Some(-(val * 1_000.0).round() as i64);
        }
    }

    let bc_suffixes = [
        "av. j.-c.", "av. j-c", "av jc", "av. jc", "av j.-c.", "av. j.-c", "bc",
    ];
    for suf in &bc_suffixes {
        if s.ends_with(suf) {
            let num_part = s[..s.len() - suf.len()].trim();
            if let Ok(val) = num_part.parse::<i64>() {
                return Some(-val);
            }
        }
    }

    let ad_suffixes = [
        "ap. j.-c.", "ap. j-c", "ap jc", "ap. jc", "ap j.-c.", "ad",
    ];
    for suf in &ad_suffixes {
        if s.ends_with(suf) {
            let num_part = s[..s.len() - suf.len()].trim();
            if let Ok(val) = num_part.parse::<i64>() {
                return Some(val);
            }
        }
    }

    if let Ok(val) = s.parse::<i64>() {
        return Some(val);
    }

    None
}

pub fn parse_anchor(input: &str, eras: &[NamedEra]) -> Option<TemporalAnchor> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lc = trimmed.to_lowercase();
    if let Some(era) = eras.iter().find(|e| e.name.to_lowercase() == lc) {
        return Some(TemporalAnchor {
            start: era.start,
            end: era.end,
            label: Some(era.name.to_string()),
        });
    }

    let range_seps = [" – ", " - ", "—", "‒", "..", "–", "-"];
    for sep in &range_seps {
        let search_start = if trimmed.starts_with('-') && (*sep == "-" || *sep == "–") { 1 } else { 0 };
        if let Some(pos) = trimmed[search_start..].find(sep) {
            let abs_pos = search_start + pos;
            let left = &trimmed[..abs_pos];
            let right = &trimmed[abs_pos + sep.len()..];
            if !left.is_empty() && !right.is_empty() {
                if let (Some(a), Some(b)) = (parse_single_year(left), parse_single_year(right)) {
                    return Some(TemporalAnchor {
                        start: a.min(b),
                        end: a.max(b),
                        label: None,
                    });
                }
            }
        }
    }

    if let Some(y) = parse_single_year(trimmed) {
        return Some(TemporalAnchor {
            start: y,
            end: y,
            label: None,
        });
    }

    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemporalFilter {
    pub start: i64,
    pub end: i64,
}

pub fn node_matches_temporal_filter(
    anchor: Option<&TemporalAnchor>,
    filter: Option<TemporalFilter>,
) -> bool {
    let f = match filter {
        Some(f) => f,
        None => return true,
    };
    let a = match anchor {
        Some(a) => a,
        None => return true,
    };
    a.start <= f.end && a.end >= f.start
}

pub fn tick_step(span_years: i64) -> i64 {
    if span_years > 50_000_000 {
        10_000_000
    } else if span_years > 5_000_000 {
        1_000_000
    } else if span_years > 500_000 {
        100_000
    } else if span_years > 50_000 {
        10_000
    } else if span_years > 5_000 {
        1_000
    } else if span_years > 500 {
        100
    } else if span_years > 50 {
        10
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_year() {
        assert_eq!(format_year(1789), "1789");
        assert_eq!(format_year(-500), "500 av. J.-C.");
        assert_eq!(format_year(-50_000), "50.0 ka");
        assert_eq!(format_year(-1_500_000), "1.5 Ma");
    }

    #[test]
    fn test_format_anchor() {
        let single = TemporalAnchor { start: 1789, end: 1789, label: None };
        assert_eq!(format_anchor(&single), "1789");

        let span = TemporalAnchor { start: 1789, end: 1799, label: None };
        assert_eq!(format_anchor(&span), "1789 – 1799");

        let labeled = TemporalAnchor { start: 1789, end: 1799, label: Some("Révolution".into()) };
        assert_eq!(format_anchor(&labeled), "Révolution");
    }

    #[test]
    fn test_parse_anchor_era() {
        let anchor = parse_anchor("Renaissance", DEFAULT_ERAS).expect("Renaissance");
        assert_eq!(anchor.start, 1400);
        assert_eq!(anchor.end, 1600);
        assert_eq!(anchor.label.as_deref(), Some("Renaissance"));
    }
}
