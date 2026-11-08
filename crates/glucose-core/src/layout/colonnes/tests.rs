//! COLONNES-1 — l'optimum exact, l'ordre de lecture, la proportion, l'emboîtement.

use super::*;

const M: Marges = Marges {
    ecart: 30.0,
    titre: 60.0,
};

/// Une suite de hauteurs, pseudo-aléatoire mais toujours la même.
fn hauteurs(graine: u64, n: usize) -> Vec<f64> {
    let mut x = graine;
    (0..n)
        .map(|_| {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            40.0 + ((x >> 33) % 400) as f64
        })
        .collect()
}

/// La plus haute colonne de ces coupes.
fn plus_haute(h: &[f64], coupes: &[(usize, usize)]) -> f64 {
    coupes
        .iter()
        .map(|&(a, z)| colonne(&h[a..z], M.ecart))
        .fold(0.0, f64::max)
}

/// **La force brute** : la plus basse « plus haute colonne » sur toutes les découpes en au plus k.
fn optimum(h: &[f64], k: usize) -> f64 {
    fn rec(h: &[f64], k: usize) -> f64 {
        if h.is_empty() {
            return 0.0;
        }
        if k == 1 {
            return colonne(h, M.ecart);
        }
        (1..=h.len())
            .map(|i| colonne(&h[..i], M.ecart).max(rec(&h[i..], k - 1)))
            .fold(f64::INFINITY, f64::min)
    }
    rec(h, k)
}

/// **La bissection rend l'optimum exact** de la partition linéaire, comparé à la force brute, sur
/// des suites de toutes les tailles jusqu'à dix et tous les k. Les coupes couvrent la suite, dans
/// l'ordre, sans trou.
#[test]
fn colonnes_1_la_bissection_rend_l_optimum_exact() {
    for graine in 0..40 {
        for n in 1..=10 {
            let h = hauteurs(graine, n);
            for k in 1..=n {
                let c = coupes_pour(&h, k, M.ecart);
                assert!(c.len() <= k, "au plus {k} colonnes : {c:?}");
                assert_eq!(c.first().map(|x| x.0), Some(0));
                assert_eq!(c.last().map(|x| x.1), Some(n));
                assert!(c.windows(2).all(|w| w[0].1 == w[1].0 && w[0].0 < w[0].1));
                let attendu = optimum(&h, k);
                assert!(
                    (plus_haute(&h, &c) - attendu).abs() < 1e-9,
                    "graine {graine}, n {n}, k {k} : {} au lieu de {attendu}",
                    plus_haute(&h, &c)
                );
            }
        }
    }
}

fn feuille(id: &str, largeur: f64, hauteur: f64) -> Bloc {
    Bloc {
        id: id.into(),
        largeur,
        hauteur,
        enfants: Vec::new(),
    }
}

/// **Le nombre de colonnes approche la proportion visée** : aucun autre k ne fait mieux.
#[test]
fn colonnes_1_la_proportion_la_plus_proche() {
    for (graine, proportion) in [(1, 1.6), (2, 0.5), (3, 3.0), (4, 1.0)] {
        let h = hauteurs(graine, 30);
        let enfants: Vec<Bloc> = h
            .iter()
            .enumerate()
            .map(|(i, &x)| feuille(&format!("n{i}"), 240.0, x))
            .collect();
        let fils: Vec<Dispose> = enfants
            .iter()
            .map(|e| disposer_bloc(e, proportion, M))
            .collect();
        let choisi = meilleures_coupes(&fils, &h, proportion, M);
        let ecart = |c: &[(usize, usize)]| {
            let (l, hh) = taille(&fils, &h, c, M);
            ((l / hh) / proportion).ln().abs()
        };
        for k in 1..=h.len() {
            let autre = coupes_pour(&h, k, M.ecart);
            assert!(
                ecart(&choisi) <= ecart(&autre) + 1e-12,
                "proportion {proportion} : {k} colonnes feraient mieux"
            );
        }
    }
}

/// **L'ordre de lecture se garde, et une membrane emboîtée contient ce qu'elle porte** : ses
/// enfants dans sa boîte, sous son bandeau ; deux frères ne se chevauchent jamais ; la racine
/// contient tout.
#[test]
fn colonnes_1_l_ordre_et_l_emboitement() {
    let section = |id: &str, n: usize, graine: u64| Bloc {
        id: id.into(),
        largeur: 300.0,
        hauteur: 100.0,
        enfants: hauteurs(graine, n)
            .into_iter()
            .enumerate()
            .map(|(i, h)| feuille(&format!("{id}-p{i}"), 240.0, h))
            .collect(),
    };
    let mut racine = section("article", 5, 9);
    racine.enfants.insert(2, section("s1", 12, 7));
    racine.enfants.push(section("s2", 3, 8));
    racine.enfants.push(Bloc {
        id: "vide".into(),
        largeur: 320.0,
        hauteur: 140.0,
        enfants: Vec::new(),
    });
    let boites: std::collections::HashMap<String, Rect> =
        disposer(&racine, 1.6, M).into_iter().collect();
    let dedans = |a: &Rect, b: &Rect| {
        a.left <= b.left && a.top <= b.top && a.right() >= b.right() && a.bottom() >= b.bottom()
    };
    assert_eq!(
        boites["vide"].width, 320.0,
        "une membrane vide garde sa taille"
    );
    fn verifier(
        b: &Bloc,
        boites: &std::collections::HashMap<String, Rect>,
        dedans: &dyn Fn(&Rect, &Rect) -> bool,
    ) {
        let parent = boites[&b.id];
        let mut avant: Option<Rect> = None;
        for e in &b.enfants {
            let r = boites[&e.id];
            assert!(dedans(&parent, &r), "{} dans {}", e.id, b.id);
            assert!(r.top >= parent.top + M.titre, "{} sous le bandeau", e.id);
            if let Some(a) = avant {
                // L'ordre de lecture : le suivant est dessous, ou dans une colonne à droite.
                assert!(
                    (r.left == a.left && r.top > a.top) || r.left > a.left,
                    "{} vient après",
                    e.id
                );
            }
            avant = Some(r);
            verifier(e, boites, dedans);
        }
        for (i, a) in b.enfants.iter().enumerate() {
            for c in &b.enfants[i + 1..] {
                let (x, y) = (boites[&a.id], boites[&c.id]);
                let disjoints = x.right() <= y.left
                    || y.right() <= x.left
                    || x.bottom() <= y.top
                    || y.bottom() <= x.top;
                assert!(disjoints, "{} et {} se chevauchent", a.id, c.id);
            }
        }
    }
    verifier(&racine, &boites, &dedans);
}
