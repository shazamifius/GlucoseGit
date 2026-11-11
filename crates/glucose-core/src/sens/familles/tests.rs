//! Les épreuves de PARENTE-1 : des documents plantés — des familles connues, noyées dans des textes
//! du hasard —, et les outils de la loi un par un.

use super::{efron, erfc, jacobi, parente, Hasard, Parente};
use std::f64::consts::TAU;

/// **Un hasard qui se rejoue** : un générateur congruentiel (Knuth, MMIX), et des tirages presque
/// normaux (la somme de quatre uniformes, centrée).
struct Tirages(u64);

impl Tirages {
    fn uniforme(&mut self) -> f32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    fn normal(&mut self) -> f32 {
        (0..4).map(|_| self.uniforme()).sum::<f32>() - 2.0
    }

    /// Un sens tiré au hasard, de longueur 1.
    fn sens(&mut self, d: usize) -> Vec<f32> {
        unitaire((0..d).map(|_| self.normal()).collect())
    }

    /// Un membre d'une famille : son centre, et un peu de bruit à lui.
    fn membre(&mut self, centre: &[f32], bruit: f32) -> Vec<f32> {
        unitaire(centre.iter().map(|c| c + bruit * self.normal()).collect())
    }
}

fn unitaire(v: Vec<f32>) -> Vec<f32> {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    v.into_iter().map(|x| x / n).collect()
}

/// Un vecteur unitaire de `d` dimensions : `poids` sur les axes donnés.
fn axes(d: usize, poids: &[(usize, f32)]) -> Vec<f32> {
    let mut v = vec![0.0; d];
    for &(i, p) in poids {
        v[i] += p;
    }
    unitaire(v)
}

/// L'écart de deux teintes sur le cercle.
fn ecart(a: f64, b: f64) -> f64 {
    let e = (a - b).rem_euclid(TAU);
    e.min(TAU - e)
}

/// **PARENTE-1 — les familles plantées se retrouvent, et le hasard n'y est pas sûr** : trois
/// groupes de cinq (le deuxième un peu parent du premier), noyés dans soixante textes du hasard.
/// Chaque groupe est une famille, et ses membres y sont sûrs ; les deux parents prennent des teintes
/// plus proches qu'avec le troisième. Un texte du hasard peut y entrer — le seuil d'une paire n'est
/// pas corrigé du nombre de comparaisons, il propose —, mais il n'y est pas sûr : son halo, pâle, ne
/// lie pas (sa règle : *« si l'IA n'est pas sûre, pas de liaison de couleur »*).
#[test]
fn parente_1_les_familles_plantees_se_retrouvent() {
    let d = 70;
    let mut t = Tirages(7);
    let centres = [
        axes(d, &[(0, 1.0)]),
        axes(d, &[(0, 0.15), (1, 0.989)]),
        axes(d, &[(2, 1.0)]),
    ];
    let mut sens = Vec::new();
    for c in &centres {
        for _ in 0..5 {
            sens.push(t.membre(c, 0.03));
        }
    }
    for _ in 0..60 {
        sens.push(t.sens(d));
    }
    let p = parente(&sens);
    let plantees: Vec<usize> = (0..3)
        .map(|g| p.famille[g * 5].expect("un membre a sa famille"))
        .collect();
    for (g, &plantee) in plantees.iter().enumerate() {
        for k in 0..5 {
            assert_eq!(
                p.famille[g * 5 + k],
                Some(plantee),
                "le membre {k} du groupe {g}"
            );
        }
    }
    assert!(plantees[0] != plantees[1] && plantees[1] != plantees[2]);
    assert!(
        p.surete[..15].iter().all(|&s| s > 0.9),
        "{:?}",
        &p.surete[..15]
    );
    for (f, &s) in p.famille[15..].iter().zip(&p.surete[15..]) {
        if f.is_some_and(|f| plantees.contains(&f)) {
            assert!(s < 0.5, "un intrus de la famille {f:?} est sûr : {s}");
        }
    }
    let surs = p.surete[15..].iter().filter(|&&s| s > 0.5).count();
    assert!(
        surs <= 1,
        "au plus un texte du hasard paraît sûr : {:?}",
        &p.surete[15..]
    );
    let h = |g: usize| p.teintes[plantees[g]];
    assert!(
        ecart(h(0), h(1)) < ecart(h(0), h(2)),
        "des familles parentes plus proches : {:?}",
        p.teintes
    );
}

/// **PARENTE-1 — le hasard pur n'est pas sûr** : deux cents textes tirés au hasard. Leur chance
/// de Šidák seule est uniforme — la moitié d'entre eux paraîtraient à moitié sûrs ; le taux local
/// de fausses découvertes les ramène à zéro. **Au plus un** peut rester : le plus extrême du hasard
/// a, sous le seul hasard, une sûreté uniforme (le théorème de Daniels, 1945) — aucune loi ne le
/// distingue d'un vrai parent faible.
#[test]
fn parente_1_le_hasard_pur_n_est_pas_sur() {
    let mut t = Tirages(11);
    let sens: Vec<Vec<f32>> = (0..200).map(|_| t.sens(64)).collect();
    let p = parente(&sens);
    let surs = p.surete.iter().filter(|&&s| s > 0.5).count();
    assert!(surs <= 1, "{surs} textes du hasard paraissent sûrs");
    // La preuve à l'envers : la seule chance de Šidák, sans Efron, en fait paraître la moitié sûrs.
    let mut paires: Vec<f32> = (0..200)
        .flat_map(|i| (i + 1..200).map(move |j| (i, j)))
        .map(|(i, j)| super::super::ressemblance(&sens[i], &sens[j]))
        .collect();
    let hasard = Hasard::de(&mut paires).expect("des paires");
    let mi_surs = (0..200)
        .filter(|&i| {
            let parent = (0..200)
                .filter(|&j| j != i)
                .map(|j| super::super::ressemblance(&sens[i], &sens[j]))
                .fold(f32::NEG_INFINITY, f32::max);
            hasard.chance(parent, 199) < 0.5
        })
        .count();
    assert!(
        mi_surs > 60,
        "Šidák seul : {mi_surs} textes sur deux cents à plus de 0,5"
    );
}

/// **PARENTE-1 — le hasard se lit sans les vraies parentés** : une famille de trente textes, un
/// quart des paires du document, parmi trente textes du hasard. La médiane et l'écart absolu médian
/// ne bougent pas, et la famille se retrouve. **La preuve à l'envers** : la moyenne et l'écart-type,
/// tirés vers le haut par elle, mettaient le seuil au-dessus de ses propres paires.
#[test]
fn parente_1_le_hasard_se_lit_sans_les_vraies_parentes() {
    let d = 64;
    let mut t = Tirages(3);
    let centre = t.sens(d);
    let mut sens: Vec<Vec<f32>> = (0..30).map(|_| t.membre(&centre, 0.06)).collect();
    sens.extend((0..30).map(|_| t.sens(d)));
    let p = parente(&sens);
    let famille = p.famille[0].expect("une famille");
    assert!(
        p.famille[..30].iter().all(|&f| f == Some(famille)),
        "{:?}",
        p.famille
    );
    let mut paires: Vec<f32> = (0..60)
        .flat_map(|i| (i + 1..60).map(move |j| (i, j)))
        .map(|(i, j)| super::super::ressemblance(&sens[i], &sens[j]))
        .collect();
    let n = paires.len() as f32;
    let moyenne = paires.iter().sum::<f32>() / n;
    let ecart_type = (paires.iter().map(|x| (x - moyenne).powi(2)).sum::<f32>() / n).sqrt();
    let plus_faible_des_siennes = (0..30)
        .flat_map(|i| (i + 1..30).map(move |j| (i, j)))
        .map(|(i, j)| super::super::ressemblance(&sens[i], &sens[j]))
        .fold(f32::INFINITY, f32::min);
    assert!(
        moyenne + 2.0 * ecart_type > plus_faible_des_siennes,
        "par la moyenne, le seuil passerait au-dessus de paires de la famille"
    );
    let hasard = Hasard::de(&mut paires).expect("des paires");
    assert!(hasard.seuil() < plus_faible_des_siennes);
}

/// **PARENTE-1 — un anneau de familles se retrouve sur le cercle** : six familles dont chacune n'est
/// parente que de ses deux voisines ; sur le cercle des teintes, les voisines d'une famille sont ses
/// voisines de l'anneau. Le document les pose **dans le désordre** : posées dans l'ordre de l'anneau,
/// leur ordre d'arrivée était déjà le bon, et un cercle qui ne cherchait rien passait (fiche 106).
#[test]
fn parente_1_l_anneau_des_familles_se_retrouve_sur_le_cercle() {
    let d = 70;
    let mut t = Tirages(5);
    let dans_le_document = [0, 3, 1, 4, 2, 5];
    let mut sens = Vec::new();
    for &f in &dans_le_document {
        let theta = std::f32::consts::TAU * f as f32 / 6.0;
        let centre = axes(
            d,
            &[
                (0, 0.55 * theta.cos()),
                (1, 0.55 * theta.sin()),
                (20 + f, 0.84),
            ],
        );
        for _ in 0..5 {
            sens.push(t.membre(&centre, 0.03));
        }
    }
    for _ in 0..40 {
        sens.push(t.sens(d));
    }
    let p = parente(&sens);
    let mut familles = [0usize; 6];
    for (bloc, &f) in dans_le_document.iter().enumerate() {
        familles[f] = p.famille[bloc * 5].expect("une famille");
    }
    let mut ordre: Vec<usize> = (0..6).collect();
    ordre.sort_by(|&a, &b| p.teintes[familles[a]].total_cmp(&p.teintes[familles[b]]));
    for (i, &f) in ordre.iter().enumerate() {
        let voisines = [ordre[(i + 1) % 6], ordre[(i + 5) % 6]];
        let anneau = [(f + 1) % 6, (f + 5) % 6];
        assert!(
            voisines.iter().all(|v| anneau.contains(v)),
            "la famille {f} : voisines {voisines:?} sur le cercle, {anneau:?} sur l'anneau"
        );
    }
}

/// **PARENTE-1 — le cercle s'oriente par le document** : la famille du premier texte qui en a une
/// prend la teinte d'origine, et la première autre famille tombe du côté où les angles croissent.
#[test]
fn parente_1_le_cercle_s_oriente_par_le_document() {
    let mut p = Parente {
        famille: vec![None, Some(1), Some(0), Some(1), Some(2)],
        teintes: vec![1.0, 2.0, 4.0],
        surete: vec![0.0; 5],
    };
    p.orienter(0.5);
    let attendues = [1.5, 0.5, 0.5 - 2.0 + TAU];
    for (t, a) in p.teintes.iter().zip(attendues) {
        assert!((t - a).abs() < 1e-12, "{:?}", p.teintes);
    }
    // Déjà du bon côté : une rotation seule.
    let mut q = Parente {
        famille: vec![Some(0), Some(1)],
        teintes: vec![1.0, 2.0],
        surete: vec![0.0; 2],
    };
    q.orienter(3.0);
    assert!((q.teintes[0] - 3.0).abs() < 1e-12 && (q.teintes[1] - 4.0).abs() < 1e-12);
}

/// **La fonction d'erreur complémentaire** rend les valeurs des tables à 2 × 10⁻⁷ près, en relatif
/// — loin dans la queue comprise.
#[test]
fn parente_1_erfc() {
    for (x, attendue) in [
        (0.0, 1.0),
        (0.5, 0.479_500_122_186_953_5),
        (1.0, 0.157_299_207_050_285_1),
        (2.0, 0.004_677_734_981_047_266),
        (3.5, 7.430_983_723_414_128e-7),
        (5.0, 1.537_459_794_428_034_8e-12),
        (-1.0, 1.842_700_792_949_715),
    ] {
        let r = erfc(x);
        assert!(((r - attendue) / attendue).abs() < 2e-7, "erfc({x}) = {r}");
    }
}

/// **Le hasard robuste** : la médiane et l'écart absolu médian, ramené à un écart-type — une valeur
/// aberrante ne les déplace pas.
#[test]
fn parente_1_la_mediane_et_l_ecart_absolu_median() {
    let mut v = [1.0f32, 2.0, 3.0, 4.0, 100.0];
    let h = Hasard::de(&mut v).expect("des valeurs");
    assert_eq!(h.mediane, 3.0);
    assert!((h.ecart - 1.482_602_218_505_602).abs() < 1e-12);
    let mut paire = [1.0f32, 2.0, 3.0, 4.0];
    assert_eq!(Hasard::de(&mut paire).expect("des valeurs").mediane, 2.5);
    assert_eq!(Hasard::de(&mut []), None);
}

/// **Efron sur des chances connues** : des chances régulières, celles du seul hasard, ne rendent
/// personne sûr ; des chances nulles rendent sûr.
#[test]
fn parente_1_efron() {
    let regulieres: Vec<f64> = (1..=20).map(|k| f64::from(k) / 20.0).collect();
    assert!(efron(&regulieres).iter().all(|&s| s < 1e-6));
    let s = efron(&[0.0, 0.0, 0.5, 0.75, 1.0]);
    assert_eq!(&s[..2], &[1.0, 1.0]);
    assert!(s[2..].iter().all(|&x| x < 1e-6), "{s:?}");
}

/// Les rotations de Jacobi rendent les valeurs propres d'une matrice symétrique connue.
#[test]
fn parente_1_jacobi() {
    let mut a = vec![2.0, -1.0, 0.0, -1.0, 2.0, -1.0, 0.0, -1.0, 2.0];
    let (mut valeurs, _) = jacobi(&mut a, 3);
    valeurs.sort_by(f64::total_cmp);
    let r2 = 2f64.sqrt();
    for (v, attendue) in valeurs.iter().zip([2.0 - r2, 2.0, 2.0 + r2]) {
        assert!((v - attendue).abs() < 1e-10, "{valeurs:?}");
    }
}

/// Un document d'un texte, ou sans texte : ni famille, ni teinte, ni sûreté.
#[test]
fn parente_1_trop_peu() {
    assert_eq!(parente(&[]).famille, Vec::<Option<usize>>::new());
    let seul = parente(&[axes(4, &[(0, 1.0)])]);
    assert_eq!(seul.famille, vec![None]);
    assert!(seul.teintes.is_empty());
    assert_eq!(seul.surete, vec![0.0]);
}
