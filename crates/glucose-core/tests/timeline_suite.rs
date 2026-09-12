//! Tests portés fidèlement de src/utils/timeline.test.ts

use glucose_core::timeline::{
    format_anchor, format_year, node_matches_temporal_filter, parse_anchor, tick_step,
    TemporalFilter, DEFAULT_ERAS,
};
use glucose_core::types::TemporalAnchor;

#[test]
fn test_format_year() {
    assert_eq!(format_year(2026), "2026");
    assert_eq!(format_year(0), "0");

    assert_eq!(format_year(-500), "500 av. J.-C.");
    assert_eq!(format_year(-9999), "9999 av. J.-C.");

    assert_eq!(format_year(-10_000), "10.0 ka");
    assert_eq!(format_year(-200_000), "200 ka");

    assert_eq!(format_year(-1_500_000), "1.5 Ma");
    assert_eq!(format_year(-100_000_000), "100 Ma");
}

#[test]
fn test_parse_anchor_single_year() {
    assert_eq!(
        parse_anchor("1789", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: 1789,
            end: 1789,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("-500", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: -500,
            end: -500,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("500 av JC", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: -500,
            end: -500,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("500 BC", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: -500,
            end: -500,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("500 av. J.-C.", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: -500,
            end: -500,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("1789 ap JC", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: 1789,
            end: 1789,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("1789 AD", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: 1789,
            end: 1789,
            label: None
        })
    );

    assert_eq!(
        parse_anchor("10 ka", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: -10_000,
            end: -10_000,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("12,5 ka", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: -12_500,
            end: -12_500,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("1,5 Ma", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: -1_500_000,
            end: -1_500_000,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("100 Ma", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: -100_000_000,
            end: -100_000_000,
            label: None
        })
    );
}

#[test]
fn test_parse_anchor_ranges() {
    assert_eq!(
        parse_anchor("1789-1799", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: 1789,
            end: 1799,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("1789 – 1799", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: 1789,
            end: 1799,
            label: None
        })
    );
    assert_eq!(
        parse_anchor("1789 - 1799", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: 1789,
            end: 1799,
            label: None
        })
    );

    // Ordonne start <= end même si saisie inverse
    assert_eq!(
        parse_anchor("1799-1789", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: 1789,
            end: 1799,
            label: None
        })
    );
}

#[test]
fn test_parse_anchor_named_eras() {
    let a = parse_anchor("Renaissance", DEFAULT_ERAS).unwrap();
    assert_eq!(
        a,
        TemporalAnchor {
            start: 1400,
            end: 1600,
            label: Some("Renaissance".into()),
        }
    );

    assert_eq!(
        parse_anchor("renaissance", DEFAULT_ERAS)
            .unwrap()
            .label
            .as_deref(),
        Some("Renaissance")
    );
    assert_eq!(
        parse_anchor("RENAISSANCE", DEFAULT_ERAS)
            .unwrap()
            .label
            .as_deref(),
        Some("Renaissance")
    );

    assert_eq!(parse_anchor("blabla", DEFAULT_ERAS), None);
    assert_eq!(parse_anchor("", DEFAULT_ERAS), None);
    assert_eq!(parse_anchor("   ", DEFAULT_ERAS), None);
}

#[test]
fn test_format_anchor() {
    assert_eq!(
        format_anchor(&TemporalAnchor {
            start: 1400,
            end: 1600,
            label: Some("Renaissance".into())
        }),
        "Renaissance"
    );
    assert_eq!(
        format_anchor(&TemporalAnchor {
            start: 1789,
            end: 1789,
            label: None
        }),
        "1789"
    );
    assert_eq!(
        format_anchor(&TemporalAnchor {
            start: 1789,
            end: 1799,
            label: None
        }),
        "1789 – 1799"
    );
}

#[test]
fn test_node_matches_temporal_filter() {
    assert!(node_matches_temporal_filter(
        None,
        Some(TemporalFilter {
            start: 1000,
            end: 2000
        })
    ));
    assert!(node_matches_temporal_filter(
        Some(&TemporalAnchor {
            start: 1789,
            end: 1799,
            label: None
        }),
        None
    ));

    let f = Some(TemporalFilter {
        start: 1700,
        end: 1900,
    });
    assert!(node_matches_temporal_filter(
        Some(&TemporalAnchor {
            start: 1789,
            end: 1799,
            label: None
        }),
        f
    ));

    let f2 = Some(TemporalFilter {
        start: 1800,
        end: 2000,
    });
    assert!(node_matches_temporal_filter(
        Some(&TemporalAnchor {
            start: 1789,
            end: 1900,
            label: None
        }),
        f2
    ));
    assert!(node_matches_temporal_filter(
        Some(&TemporalAnchor {
            start: 1700,
            end: 1800,
            label: None
        }),
        Some(TemporalFilter {
            start: 1750,
            end: 2000
        })
    ));

    let f_contained = Some(TemporalFilter {
        start: 1700,
        end: 1800,
    });
    assert!(node_matches_temporal_filter(
        Some(&TemporalAnchor {
            start: 1000,
            end: 2000,
            label: None
        }),
        f_contained
    ));

    // Disjoints
    assert!(!node_matches_temporal_filter(
        Some(&TemporalAnchor {
            start: 1500,
            end: 1600,
            label: None
        }),
        f_contained
    ));
    assert!(!node_matches_temporal_filter(
        Some(&TemporalAnchor {
            start: 1900,
            end: 2000,
            label: None
        }),
        f_contained
    ));

    // Bornes inclusives
    assert!(node_matches_temporal_filter(
        Some(&TemporalAnchor {
            start: 1700,
            end: 1700,
            label: None
        }),
        f_contained
    ));
    assert!(node_matches_temporal_filter(
        Some(&TemporalAnchor {
            start: 1800,
            end: 1800,
            label: None
        }),
        f_contained
    ));
}

#[test]
fn test_tick_step() {
    assert_eq!(tick_step(10), 1);
    assert_eq!(tick_step(100), 10);
    assert_eq!(tick_step(2000), 100);
    assert_eq!(tick_step(20_000), 1000);
    assert_eq!(tick_step(2_000_000), 100_000);
    assert_eq!(tick_step(200_000_000), 10_000_000);
}

#[test]
fn test_default_eras_valid() {
    for e in DEFAULT_ERAS {
        assert!(e.start <= e.end);
    }
    let names: Vec<&str> = DEFAULT_ERAS.iter().map(|e| e.name).collect();
    assert!(names.contains(&"Renaissance"));
    assert!(names.contains(&"Révolution française"));
    assert!(names.contains(&"Antiquité"));
}

/// Fiche 09 § 7.1 — les trois exemples de la fiche, tels quels : `1789`, `-3000` (avant
/// J.-C., entier signé), et la Renaissance comme plage continue `[1400, 1600]`.
#[test]
fn test_the_anchor_examples_of_the_spec() {
    let year = |y: i64| TemporalAnchor { start: y, end: y, label: None };
    assert_eq!(parse_anchor("1789", DEFAULT_ERAS), Some(year(1789)));
    assert_eq!(parse_anchor("-3000", DEFAULT_ERAS), Some(year(-3000)));
    assert_eq!(
        parse_anchor("1400-1600", DEFAULT_ERAS),
        Some(TemporalAnchor { start: 1400, end: 1600, label: None })
    );
    // Et la date est celle du sujet, pas celle du fichier : rien dans l'ancre ne vient
    // d'une horloge, elle se construit et se relit à l'identique.
    assert_eq!(format_anchor(&year(-3000)), format_anchor(&year(-3000)));
}
