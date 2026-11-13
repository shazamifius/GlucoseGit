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
    let year = |y: i64| TemporalAnchor {
        start: y,
        end: y,
        label: None,
    };
    assert_eq!(parse_anchor("1789", DEFAULT_ERAS), Some(year(1789)));
    assert_eq!(parse_anchor("-3000", DEFAULT_ERAS), Some(year(-3000)));
    assert_eq!(
        parse_anchor("1400-1600", DEFAULT_ERAS),
        Some(TemporalAnchor {
            start: 1400,
            end: 1600,
            label: None
        })
    );
    // Et la date est celle du sujet, pas celle du fichier : rien dans l'ancre ne vient
    // d'une horloge, elle se construit et se relit à l'identique.
    assert_eq!(format_anchor(&year(-3000)), format_anchor(&year(-3000)));
}

// ── ESTOMPE-1 et la frise chronologique (fiche 114) ────────────────────────────────────────────────────

fn date(start: i64, end: i64) -> TemporalAnchor {
    TemporalAnchor {
        start,
        end,
        label: None,
    }
}

/// **Un filtre estompe ce qu'il écarte, jamais ce qui n'est pas daté** ; sans filtre, tout est là.
#[test]
fn estompe_1_la_presence_sous_un_filtre() {
    use glucose_core::timeline::{presence, ESTOMPE};
    let revolution = TemporalFilter {
        start: 1789,
        end: 1799,
    };
    assert_eq!(presence(Some(&date(1792, 1792)), Some(revolution)), 1.0);
    assert_eq!(
        presence(Some(&date(1700, 1789)), Some(revolution)),
        1.0,
        "un bord touche"
    );
    assert_eq!(presence(Some(&date(1914, 1918)), Some(revolution)), ESTOMPE);
    assert_eq!(
        presence(None, Some(revolution)),
        1.0,
        "hors du temps, jamais écarté"
    );
    assert_eq!(presence(Some(&date(1914, 1918)), None), 1.0, "sans filtre");
    const { assert!(ESTOMPE > 0.0 && ESTOMPE < 1.0, "estompé, jamais caché") };
}

/// **L'étendue va du plus ancien début à la plus récente fin** ; rien sans date.
#[test]
fn estompe_1_l_etendue_des_dates() {
    use glucose_core::timeline::etendue;
    let dates = [date(1789, 1799), date(-500, -500), date(1914, 1918)];
    assert_eq!(
        etendue(&dates),
        Some(TemporalFilter {
            start: -500,
            end: 1918
        })
    );
    assert_eq!(etendue(&[]), None);
}

/// **Chaque graduation compte les dates qui la croisent** : une plage compte dans chacune, une
/// date hors de la vue ne compte pas, et les graduations négatives se rangent sans an zéro qui
/// décale (la division euclidienne : -500 tombe dans [-500, -400[).
#[test]
fn estompe_1_la_repartition_par_graduation() {
    use glucose_core::timeline::repartition;
    let vue = TemporalFilter {
        start: -500,
        end: 299,
    };
    let dates = [
        date(-500, -500),
        date(-450, -250),
        date(100, 150),
        date(1789, 1799),
    ];
    // Les graduations : [-500,-400[, [-400,-300[, [-300,-200[, [-200,-100[, [-100,0[, [0,100[,
    // [100,200[, [200,300[.
    assert_eq!(repartition(&dates, vue, 100), vec![2, 1, 1, 0, 0, 0, 1, 0]);
    assert_eq!(repartition(&[], vue, 100).iter().sum::<u32>(), 0);
}

/// **FRISE-2 — chaque ère entière, dans son couloir** (fiche 115). La loi de la plus petite ère
/// (REGION-1 appliquée au temps), rejouée ici, ne montrait la révolution industrielle **qu'à partir
/// de 1800**, en deux morceaux (de 1800 à 1803, puis après l'Empire) : les Lumières, la Révolution et
/// l'Empire, plus courts, la couvraient — une date fausse. Dans son couloir, elle va de 1760 à 1840,
/// entière.
#[test]
fn frise_2_chaque_ere_entiere_dans_son_couloir() {
    use glucose_core::timeline::eres_de_la_vue;
    let vue = TemporalFilter {
        start: 1700,
        end: 2000,
    };
    let bande = eres_de_la_vue(vue, DEFAULT_ERAS);
    let paru = |nom: &str| {
        bande
            .iter()
            .find(|e| e.ere.name == nom)
            .map(|e| (e.paru.start, e.paru.end))
    };
    assert_eq!(paru("Révolution industrielle"), Some((1760, 1839)));
    assert_eq!(paru("Lumières"), Some((1715, 1788)));
    assert_eq!(
        paru("Holocène"),
        Some((1700, 2000)),
        "coupée aux bords de la vue"
    );
    assert_eq!(paru("Renaissance"), None, "hors de la vue");
    // La preuve à l'envers : à chaque année, la plus petite ère qui la contient — la loi d'avant.
    let plus_petite = |an: i64| {
        DEFAULT_ERAS
            .iter()
            .filter(|e| e.start <= an && e.end >= an)
            .min_by_key(|e| e.end - e.start)
            .map(|e| e.name)
    };
    let premiere = (1700..=2000).find(|&an| plus_petite(an) == Some("Révolution industrielle"));
    assert_eq!(
        premiere,
        Some(1800),
        "l'ancienne loi la faisait naître après la Révolution"
    );
}

/// **FRISE-2 — les couloirs** : deux ères qui se chevauchent n'ont jamais le même ; deux qui se
/// touchent (les Lumières finissent en 1789, l'année où la Révolution commence) le partagent ; à
/// début égal, la plus longue passe au-dessus (le Moyen Âge sur le Haut Moyen Âge) ; le rang dans un
/// couloir suit le temps ; et **il y a autant de couloirs que d'ères qui se chevauchent au plus à un
/// même instant** — cinq, de -509 à -146 (l'Holocène, l'Antiquité, l'Égypte, la Grèce et Rome).
#[test]
fn frise_2_les_couloirs_des_eres() {
    use glucose_core::timeline::{couloirs, profondeur};
    let places = couloirs(DEFAULT_ERAS);
    let place = |nom: &str| {
        let i = DEFAULT_ERAS.iter().position(|e| e.name == nom).expect(nom);
        places[i]
    };
    for (i, a) in DEFAULT_ERAS.iter().enumerate() {
        for (j, b) in DEFAULT_ERAS.iter().enumerate().skip(i + 1) {
            if places[i].0 == places[j].0 {
                assert!(
                    a.end <= b.start || b.end <= a.start,
                    "{} et {} partagent un couloir",
                    a.name,
                    b.name
                );
                let (avant, apres) = if a.start < b.start { (i, j) } else { (j, i) };
                assert!(places[avant].1 < places[apres].1, "le rang suit le temps");
            }
        }
    }
    assert_eq!(place("Lumières").0, place("Révolution française").0);
    assert!(place("Moyen Âge").0 < place("Haut Moyen Âge").0);
    let au_plus = DEFAULT_ERAS
        .iter()
        .map(|t| {
            DEFAULT_ERAS
                .iter()
                .filter(|e| e.start <= t.start && t.start < e.end)
                .count()
        })
        .max();
    assert_eq!(au_plus, Some(5));
    assert_eq!(profondeur(DEFAULT_ERAS), 5);
    assert_eq!(profondeur(&[]), 0);
}

/// **ESTOMPE-2 — une relation sans date suit ce qu'elle relie** : la frise garde la Grande Guerre ;
/// une flèche sans date entre deux nœuds de la Révolution s'estompe, vers un nœud sans date aussi
/// (le moins présent de ses bouts), entre deux nœuds de la Guerre non ; une flèche datée suit sa
/// date, quels que soient ses bouts ; un bout libre ne retire rien. La preuve à l'envers : sa seule
/// date (ESTOMPE-1) la disait pleine.
#[test]
fn estompe_2_une_relation_sans_date_suit_ce_qu_elle_relie() {
    use glucose_core::timeline::{presence, presence_d_une_relation, ESTOMPE};
    let guerre = Some(TemporalFilter {
        start: 1914,
        end: 1918,
    });
    let (revolution, grande_guerre) = (date(1789, 1799), date(1914, 1918));
    let (r, g) = (Some(&revolution), Some(&grande_guerre));
    assert_eq!(presence_d_une_relation(None, [r, r], guerre), ESTOMPE);
    assert_eq!(
        presence(None, guerre),
        1.0,
        "sa seule date la disait pleine"
    );
    assert_eq!(presence_d_une_relation(None, [r, None], guerre), ESTOMPE);
    assert_eq!(presence_d_une_relation(None, [g, g], guerre), 1.0);
    assert_eq!(presence_d_une_relation(None, [None, None], guerre), 1.0);
    assert_eq!(
        presence_d_une_relation(g, [r, r], guerre),
        1.0,
        "sa date d'abord"
    );
    assert_eq!(presence_d_une_relation(r, [g, g], guerre), ESTOMPE);
    assert_eq!(
        presence_d_une_relation(None, [r, r], None),
        1.0,
        "sans frise"
    );
}
