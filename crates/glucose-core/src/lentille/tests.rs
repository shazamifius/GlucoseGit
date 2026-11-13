//! LENTILLE-1 et DOMAINE-1 : la présence d'un nœud sous le domaine regardé, graduée par son poids ;
//! deux lentilles combinées par le minimum ; une relation qui suit ce qu'elle relie.

use super::*;

fn eau(poids: f64) -> Vec<DomainAssignment> {
    vec![DomainAssignment {
        domain_id: "eau".into(),
        weight: poids,
    }]
}

/// Un nœud sans date, de ces domaines.
fn noeud(domaines: &[DomainAssignment]) -> Regard<'_> {
    Regard {
        date: None,
        domaines,
    }
}

/// **DOMAINE-1 — la présence suit l'appartenance** : pleine pour un poids de 1, [`ESTOMPE`] pour
/// un nœud qui ne porte pas le domaine regardé (ou en porte un autre), en ligne droite entre les
/// deux ; un poids de 0,05 — sous le seuil de Tauri (0,1) — compte déjà : aucun seuil. Sans
/// lentille, tout est plein.
#[test]
fn domaine_1_la_presence_suit_l_appartenance() {
    let regarde = Some("eau");
    assert_eq!(dans_un_domaine(&eau(1.0), regarde), 1.0);
    assert_eq!(dans_un_domaine(&[], regarde), ESTOMPE);
    let moitie = dans_un_domaine(&eau(0.5), regarde);
    assert!(
        (moitie - (ESTOMPE + (1.0 - ESTOMPE) * 0.5)).abs() < 1e-6,
        "{moitie}"
    );
    assert!(dans_un_domaine(&eau(0.2), regarde) < dans_un_domaine(&eau(0.4), regarde));
    assert!(
        dans_un_domaine(&eau(0.05), regarde) > ESTOMPE,
        "aucun seuil"
    );
    let feu = vec![DomainAssignment {
        domain_id: "feu".into(),
        weight: 1.0,
    }];
    assert_eq!(dans_un_domaine(&feu, regarde), ESTOMPE, "un autre domaine");
    assert_eq!(dans_un_domaine(&[], None), 1.0, "sans lentille");
}

/// **LENTILLE-1 — deux lentilles se combinent par le minimum** : un nœud de la Révolution, hors
/// de l'eau, sous la frise de la Grande Guerre et le domaine de l'eau, reste à [`ESTOMPE`] — leur
/// produit l'aurait mis sous 2 % : caché. Un nœud de l'eau sans date est plein ; de l'eau mais de
/// la Révolution, la frise l'écarte ; à moitié de l'eau, il est entre les deux.
#[test]
fn lentille_1_le_minimum_des_lentilles() {
    let guerre = Some(TemporalFilter {
        start: 1914,
        end: 1918,
    });
    let revolution = TemporalAnchor {
        start: 1789,
        end: 1799,
        label: None,
    };
    let l = Lentilles {
        periode: guerre,
        domaine: Some("eau"),
    };
    let ecarte = Regard {
        date: Some(&revolution),
        domaines: &[],
    };
    assert_eq!(l.presence(ecarte), ESTOMPE);
    const { assert!(ESTOMPE * ESTOMPE < 0.02, "le produit l'aurait caché") };
    let dedans = eau(1.0);
    assert_eq!(l.presence(noeud(&dedans)), 1.0);
    let date_ecartee = Regard {
        date: Some(&revolution),
        domaines: &dedans,
    };
    assert_eq!(l.presence(date_ecartee), ESTOMPE);
    let moitie = eau(0.5);
    let p = l.presence(noeud(&moitie));
    assert!(p > ESTOMPE && p < 1.0, "{p}");
    assert_eq!(Lentilles::default().presence(ecarte), 1.0, "sans lentille");
}

/// **Une relation suit ce qu'elle relie** sous le domaine regardé : entre deux nœuds de l'eau,
/// pleine ; vers un nœud hors de l'eau, estompée ; un bout libre ne retire rien ; celle qui porte
/// l'eau elle-même suit son propre poids, quels que soient ses bouts. La preuve à l'envers : sa
/// seule appartenance la disait estompée.
#[test]
fn lentille_1_une_relation_suit_ce_qu_elle_relie() {
    let l = Lentilles {
        periode: None,
        domaine: Some("eau"),
    };
    let (dedans, dehors, peu) = (eau(1.0), Vec::new(), eau(0.25));
    let relation = |propre: &[DomainAssignment],
                    a: Option<&[DomainAssignment]>,
                    b: Option<&[DomainAssignment]>| {
        l.presence_d_une_relation(noeud(propre), [a.map(noeud), b.map(noeud)])
    };
    assert_eq!(relation(&dehors, Some(&dedans), Some(&dedans)), 1.0);
    assert_eq!(relation(&dehors, Some(&dedans), Some(&dehors)), ESTOMPE);
    assert_eq!(relation(&dehors, Some(&dedans), None), 1.0, "un bout libre");
    assert_eq!(
        relation(&dedans, Some(&dehors), Some(&dehors)),
        1.0,
        "la sienne d'abord"
    );
    assert_eq!(
        relation(&peu, Some(&dedans), Some(&dedans)),
        dans_un_domaine(&peu, Some("eau"))
    );
    assert_eq!(l.presence(noeud(&dehors)), ESTOMPE, "sa seule appartenance");
}
