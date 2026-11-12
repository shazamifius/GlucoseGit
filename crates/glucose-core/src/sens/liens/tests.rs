//! Les épreuves de PARENTE-4 : des documents plantés où chaque garde de la loi change la réponse —
//! la langue commune qui fait ressembler une grande région à tout, une région qui porte la plupart
//! de ses semblables.

use super::{surete_des_liens, Bout, Region};
use crate::sens::familles::{efron, Hasard};
use crate::sens::ressemblance;

/// Un hasard qui se rejoue (Knuth, MMIX) : des sens de longueur 1, autour d'une direction commune.
struct Tirages(u64);

impl Tirages {
    fn normal(&mut self) -> f32 {
        (0..4)
            .map(|_| {
                self.0 = self
                    .0
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                (self.0 >> 40) as f32 / (1u64 << 24) as f32
            })
            .sum::<f32>()
            - 2.0
    }

    /// **Un texte** : la direction commune de la langue (l'axe 0, que tous les textes partagent,
    /// comme les sens d'un vrai modèle), son sujet — le thème `axe`, ou un sujet à lui, de même
    /// force : sur une vraie carte, chaque texte parle de quelque chose —, et du bruit.
    fn texte(&mut self, axe: Option<usize>) -> Vec<f32> {
        let sujet = match axe {
            Some(a) => {
                let mut s = vec![0.0; 70];
                s[a] = 1.0;
                s
            }
            None => {
                let mut s: Vec<f32> = (0..70).map(|_| self.normal()).collect();
                s[0] = 0.0;
                unitaire(s)
            }
        };
        let mut v: Vec<f32> = (0..70).map(|_| 0.12 * self.normal()).collect();
        v[0] += 0.6;
        for (x, s) in v.iter_mut().zip(sujet) {
            *x += s;
        }
        unitaire(v)
    }
}

fn unitaire(v: Vec<f32>) -> Vec<f32> {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    v.into_iter().map(|x| x / n).collect()
}

/// **PARENTE-4 — un lien de sens est sûr, un lien d'habitude ne l'est pas** (fiche 107) : vingt
/// textes de deux thèmes et cinquante sans thème. Sûrs : un texte vers la région de son thème qui
/// ne le porte pas, deux textes du même thème. Pas sûrs : un texte d'un autre thème vers cette
/// région, un texte sans thème vers elle ou vers une **grande région** de quarante textes sans thème
/// — qui, par la langue commune, ressemble à tout. **La preuve à l'envers** : contre le hasard du
/// document (deux textes pris au hasard), ces liens vers la grande région paraissaient sûrs.
#[test]
fn parente_4_un_lien_de_sens_est_sur_un_lien_d_habitude_non() {
    let mut t = Tirages(7);
    let mut sens = Vec::new();
    for axe in [Some(5), Some(9)] {
        for _ in 0..10 {
            sens.push(t.texte(axe));
        }
    }
    for _ in 0..50 {
        sens.push(t.texte(None));
    }
    let theme = Region::de(&sens, (0..5).collect(), None).expect("un sens");
    let grande = Region::de(&sens, (20..60).collect(), None).expect("un sens");
    let regions = [theme, grande];
    let mut de_sens: Vec<(Bout, Bout)> =
        (5..10).map(|k| (Bout::Texte(k), Bout::Region(0))).collect();
    de_sens.extend((10..15).map(|k| (Bout::Texte(k), Bout::Texte(k + 5))));
    let vers_la_grande: Vec<(Bout, Bout)> = (60..70)
        .map(|k| (Bout::Texte(k), Bout::Region(1)))
        .collect();
    let mut habitudes = vers_la_grande.clone();
    habitudes.extend((10..15).map(|k| (Bout::Texte(k), Bout::Region(0))));
    habitudes.extend((60..70).map(|k| (Bout::Texte(k), Bout::Region(0))));
    let liens: Vec<(Bout, Bout)> = de_sens.iter().chain(&habitudes).copied().collect();
    let surete = surete_des_liens(&sens, &regions, &liens);
    let (surs, habitude) = surete.split_at(de_sens.len());
    assert!(
        surs.iter().all(|&s| s > 0.9),
        "les liens de sens : {surs:?}"
    );
    let a_tort = habitude.iter().filter(|&&s| s > 0.5).count();
    assert!(a_tort <= 1, "{a_tort} liens d'habitude sûrs : {habitude:?}");
    // La preuve à l'envers : toute la loi, contre le hasard de deux textes du document.
    let mut paires: Vec<f32> = (0..sens.len())
        .flat_map(|i| (i + 1..sens.len()).map(move |j| (i, j)))
        .map(|(i, j)| ressemblance(&sens[i], &sens[j]))
        .collect();
    let du_document = Hasard::de(&mut paires).expect("des paires");
    let vecteur = |b: Bout| match b {
        Bout::Texte(k) => sens[k].as_slice(),
        Bout::Region(r) => regions[r].sens.as_slice(),
    };
    let chances: Vec<f64> = liens
        .iter()
        .map(|&(s, c)| du_document.chance(ressemblance(vecteur(s), vecteur(c)), 1))
        .collect();
    let avant = efron(&chances);
    let a_tort = avant[de_sens.len()..de_sens.len() + vers_la_grande.len()]
        .iter()
        .filter(|&&s| s > 0.5)
        .count();
    assert!(
        a_tort >= 5,
        "contre le hasard du document, la grande région ressemble à tout : {avant:?}"
    );
}

/// **PARENTE-4 — le hasard d'une région se lit hors d'elle** : « Océan → Eau ». La région de l'eau
/// porte cinquante textes de l'eau, la majorité du document ; un texte de l'océan, hors d'elle, en
/// parle, et vingt textes sans thème non. **La preuve à l'envers** : les textes qu'elle porte,
/// comptés dans son hasard, le haussent jusqu'à rendre le lien de l'océan ordinaire.
#[test]
fn parente_4_le_hasard_d_une_region_se_lit_hors_d_elle() {
    let mut t = Tirages(13);
    let mut sens: Vec<Vec<f32>> = (0..50).map(|_| t.texte(Some(3))).collect();
    let ocean = sens.len();
    let mut v = t.texte(Some(3));
    v[7] += 0.5;
    sens.push(unitaire(v));
    sens.extend((0..20).map(|_| t.texte(None)));
    let eau = Region::de(&sens, (0..50).collect(), None).expect("un sens");
    let liens: Vec<(Bout, Bout)> = (ocean..sens.len())
        .map(|k| (Bout::Texte(k), Bout::Region(0)))
        .collect();
    let surete = surete_des_liens(&sens, std::slice::from_ref(&eau), &liens);
    assert!(surete[0] > 0.9, "l'océan vers l'eau : {surete:?}");
    assert!(
        surete[1..].iter().filter(|&&s| s > 0.5).count() <= 1,
        "{surete:?}"
    );
    // La preuve à l'envers : tout le document dans le hasard de la région.
    let mut tous: Vec<f32> = sens.iter().map(|s| ressemblance(s, &eau.sens)).collect();
    let pollue = Hasard::de(&mut tous).expect("des textes");
    let chance = pollue.chance(ressemblance(&sens[ocean], &eau.sens), 1);
    assert!(chance > 0.1, "le lien de l'océan, ordinaire : {chance}");
}

/// **Le sens d'une région** : la somme de ce qu'elle porte et de son nom, ramenée à 1 — chaque
/// texte une fois ; rien sans rien.
#[test]
fn parente_4_le_sens_d_une_region() {
    let sens = vec![vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]];
    let nom = [0.0, 0.0, 1.0];
    let r = Region::de(&sens, vec![1, 0, 1], Some(&nom)).expect("un sens");
    assert_eq!(r.porte, vec![0, 1], "rangés, chacun une fois");
    let attendu = 1.0 / 3f32.sqrt();
    assert!(
        r.sens.iter().all(|x| (x - attendu).abs() < 1e-6),
        "{:?}",
        r.sens
    );
    assert_eq!(Region::de(&sens, Vec::new(), None), None);
    let seul_le_nom = Region::de(&sens, Vec::new(), Some(&nom)).expect("son nom");
    assert_eq!(seul_le_nom.sens, nom.to_vec());
}

/// **Sans hasard, aucun lien n'est sûr** : un document d'un seul texte, qui se porte lui-même — le
/// hasard d'une cible n'a aucun autre texte où se lire.
#[test]
fn parente_4_trop_peu() {
    let sens = vec![vec![1.0, 0.0]];
    let liens = [(Bout::Texte(0), Bout::Texte(0))];
    assert_eq!(surete_des_liens(&sens, &[], &liens), vec![0.0]);
}

/// **Une membrane porte aussi les textes de ses filles** (fiche 107) : « Eau » contient « Océans »,
/// qui contient un texte ; la flèche de l'agent vers « Eau » la juge par ses deux textes, celui de
/// sa fille compris, et par son nom. La flèche que l'humain tire vers la fille n'est pas collectée
/// (MAIN-1, fiche 108) — la fille, que seule elle touche, n'est donc pas une région à juger.
#[test]
fn parente_4_une_region_porte_les_textes_de_ses_filles() {
    use super::liens_du_tableau;
    use crate::types::{Annotation, Board};
    let mut tableau = Board::new("main", "Tableau");
    let dans = |mut a: Annotation, mere: &str| {
        a.set_membrane_id(Some(mere.into()));
        a
    };
    let mut eau = Annotation::membrane("eau", 0.0, 0.0, 100.0, 100.0);
    if let Annotation::Membrane { text, .. } = &mut eau {
        *text = Some("Eau".into());
    }
    let fleche = |id: &str, (de, vers): (&str, &str)| {
        let mut f = Annotation::arrow(id, 0.0, 0.0, 1.0, 1.0);
        if let Annotation::Arrow {
            source_id,
            target_id,
            ..
        } = &mut f
        {
            (*source_id, *target_id) = (Some(de.into()), Some(vers.into()));
        }
        f
    };
    tableau.annotations = vec![
        eau,
        dans(Annotation::membrane("oceans", 0.0, 0.0, 50.0, 50.0), "eau"),
        dans(Annotation::text("vague", 0.0, 0.0, "La vague"), "oceans"),
        dans(Annotation::text("pluie", 0.0, 0.0, "La pluie"), "eau"),
        Annotation::text("nuage", 0.0, 0.0, "Le nuage"),
        fleche("de-l-agent", ("nuage", "eau")),
        fleche("de-l-humain", ("nuage", "oceans")),
    ];
    let des_agents = std::collections::HashSet::from(["de-l-agent".to_string()]);
    let (fleches, regions) = liens_du_tableau(&tableau, &des_agents);
    assert_eq!(fleches, [["de-l-agent", "nuage", "eau"].map(String::from)]);
    assert_eq!(
        regions,
        [(
            "eau".to_string(),
            Some("Eau".to_string()),
            vec!["vague".to_string(), "pluie".to_string()]
        )],
        "la vague, dans la fille, compte pour la mère"
    );
}
