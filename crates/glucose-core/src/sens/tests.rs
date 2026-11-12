//! Les épreuves de SENS-1 : le découpeur, le sens d'un texte, le modèle embarqué. Celles de
//! PARENTE-1 sont avec la loi des familles.

use super::{embarque, ressemblance, textes_du_tableau, Sens, MODELE};

/// La référence : les rangs du découpeur de Hugging Face, et les ressemblances calculées en Python
/// (`outils/sens/reference.py`).
const REFERENCE: &str = include_str!("tests/reference.txt");

fn modele() -> Sens<'static> {
    Sens::lire(MODELE).expect("le modèle se lit")
}

fn hexa(h: &str) -> String {
    let octets: Vec<u8> = (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).expect("hexadécimal"))
        .collect();
    String::from_utf8(octets).expect("UTF-8")
}

/// Les textes de la référence et leurs rangs, puis ses paires (deux rangs, leur ressemblance).
type Reference = (Vec<(String, Vec<u32>)>, Vec<(usize, usize, f32)>);

/// Les textes de la référence et leurs rangs, puis ses paires.
fn reference() -> Reference {
    let (mut textes, mut paires) = (Vec::new(), Vec::new());
    let mut lignes = REFERENCE.lines();
    while let Some(ligne) = lignes.next() {
        if let Some(h) = ligne.strip_prefix("T ") {
            let rangs = lignes.next().and_then(|r| r.strip_prefix('R')).expect("R");
            let rangs = rangs
                .split_whitespace()
                .map(|r| r.parse().expect("rang"))
                .collect();
            textes.push((hexa(h.trim()), rangs));
        } else if let Some(p) = ligne.strip_prefix("P ") {
            let v: Vec<&str> = p.split_whitespace().collect();
            paires.push((
                v[0].parse().unwrap(),
                v[1].parse().unwrap(),
                v[2].parse().unwrap(),
            ));
        }
    }
    (textes, paires)
}

/// **SENS-1 — le découpeur rend les rangs de la référence**, texte piège par texte piège : accents,
/// capitales accentuées, ligatures, guillemets français, insécables, caractères invisibles, chinois,
/// japonais, coréen, cyrillique, grec (un sigma final), arabe, emoji, un mot de cent un caractères,
/// des marques combinantes, l'usage privé, le vide.
#[test]
fn sens_1_le_decoupeur_rend_les_rangs_de_la_reference() {
    let sens = modele();
    let (textes, _) = reference();
    assert!(textes.len() >= 28, "la référence se lit : {}", textes.len());
    for (texte, attendu) in &textes {
        assert_eq!(&sens.decouper(texte), attendu, "{texte:?}");
    }
}

/// **SENS-1 — le sens d'un texte est celui de la référence** : la ressemblance de neuf paires, à
/// 10⁻⁴ près (l'ordre des sommes en `f32`).
#[test]
fn sens_1_les_ressemblances_sont_celles_de_la_reference() {
    let sens = modele();
    let (textes, paires) = reference();
    assert!(paires.len() >= 8);
    for (i, j, attendue) in paires {
        let (a, b) = (sens.plonger(&textes[i].0), sens.plonger(&textes[j].0));
        let r = ressemblance(&a.expect("un sens"), &b.expect("un sens"));
        assert!(
            (r - attendue).abs() < 1e-4,
            "{i}, {j} : {r} contre {attendue}"
        );
    }
    assert_eq!(sens.dimensions(), 128);
}

/// Un texte sans morceau n'a pas de sens ; un modèle tronqué ou étranger se refuse.
#[test]
fn sens_1_le_vide_et_le_faux() {
    let sens = modele();
    assert_eq!(sens.plonger(""), None);
    assert_eq!(sens.plonger(" \t\n"), None);
    assert!(Sens::lire(&MODELE[..MODELE.len() - 1]).is_err(), "tronqué");
    assert!(Sens::lire(b"pas un modele du tout").is_err(), "étranger");
}

/// **Le modèle embarqué se lit**, une seule fois pour tout le programme.
#[test]
fn sens_1_le_modele_embarque_se_lit() {
    let a = embarque().expect("le modèle embarqué se lit");
    let b = embarque().expect("encore");
    assert!(std::ptr::eq(a, b), "lu une seule fois");
    assert_eq!(a.dimensions(), 128);
}

/// **Les textes d'un tableau** : ses nœuds de texte, dans l'ordre du document, et rien d'autre.
#[test]
fn sens_1_les_textes_d_un_tableau() {
    use crate::types::{Annotation, Board};
    let mut tableau = Board::new("t", "t");
    tableau
        .annotations
        .push(Annotation::text("a", 0.0, 0.0, "l'eau"));
    tableau
        .annotations
        .push(Annotation::membrane("m", 0.0, 0.0, 10.0, 10.0));
    tableau
        .annotations
        .push(Annotation::text("b", 0.0, 0.0, "le feu"));
    let lus: Vec<(&str, &str)> = textes_du_tableau(&tableau).collect();
    assert_eq!(lus, vec![("a", "l'eau"), ("b", "le feu")]);
}

/// **SENS-1 — un mot qui bute est inconnu tout entier** : « eau » se trouve, puis un hiéroglyphe que
/// le vocabulaire ne sait pas continuer ; la référence rend « [UNK] » seul, sans le morceau déjà
/// trouvé. Les rangs sont ceux du découpeur de Hugging Face (`tokenizers`, le 11/11/2026) : aucun
/// texte de la référence ne bute au milieu d'un mot, et un découpeur qui gardait ses morceaux
/// passait (fiche 106).
#[test]
fn sens_1_un_mot_qui_bute_est_inconnu_tout_entier() {
    let sens = modele();
    assert_eq!(sens.decouper("l eau\u{13000} bout"), vec![154, 100, 33169]);
    assert_eq!(sens.decouper("eau"), vec![16147], "« eau » seul se trouve");
}

/// **B3 — les mots d'un texte rendent exactement ses morceaux** (fiche 110) : mis bout à bout, les
/// rangs que `decouper_en_mots` range sous chaque mot d'origine sont ceux du découpeur, texte piège
/// par texte piège de la référence — accents, insécables, invisibles, chinois, ponctuation collée.
#[test]
fn pourquoi_1_les_mots_rendent_les_morceaux() {
    let sens = modele();
    let (textes, _) = reference();
    for (texte, attendu) in &textes {
        let mots = super::decoupeur::decouper_en_mots(texte, &sens);
        let bout_a_bout: Vec<u32> = mots.iter().flat_map(|(_, r)| r.iter().copied()).collect();
        assert_eq!(&bout_a_bout, attendu, "{texte:?}");
    }
    // U+0085 est un blanc, mais aussi un contrôle : la référence le retire sans séparer.
    for texte in ["eau\u{85}pluie", "mer\u{200b}océan"] {
        let mots = super::decoupeur::decouper_en_mots(texte, &sens);
        let bout_a_bout: Vec<u32> = mots.iter().flat_map(|(_, r)| r.iter().copied()).collect();
        assert_eq!(bout_a_bout, sens.decouper(texte), "{texte:?}");
    }
    let mots = super::decoupeur::decouper_en_mots("L'eau\u{a0}bout, à 100 °C.", &sens);
    let mots: Vec<&str> = mots.iter().map(|(m, _)| *m).collect();
    assert_eq!(
        mots,
        vec!["L", "'", "eau", "bout", ",", "à", "100", "°C", "."],
        "les mots d'origine, séparés comme la référence les sépare"
    );
}

/// **B3 — les parts font la ressemblance** : la somme des parts des mots de `a` est le cosinus de
/// leurs sens, à 10⁻⁴ près (l'ordre des sommes en `f32`) — et il en va de même de `b` vers `a`.
/// Les paires de la référence, puis deux textes qui partagent des mots et en répètent.
#[test]
fn pourquoi_1_les_parts_font_la_ressemblance() {
    let sens = modele();
    let (textes, paires) = reference();
    let mut cas: Vec<(String, String)> = paires
        .iter()
        .map(|&(i, j, _)| (textes[i].0.clone(), textes[j].0.clone()))
        .collect();
    cas.push((
        "La pluie tombe, la pluie mouille ; l'eau ruisselle.".into(),
        "Les nappes d'eau souterraines se remplissent par la pluie.".into(),
    ));
    for (a, b) in &cas {
        let r = ressemblance(&sens.plonger(a).unwrap(), &sens.plonger(b).unwrap());
        for (x, y) in [(a, b), (b, a)] {
            let somme: f32 = sens.pourquoi(x, y).iter().map(|p| p.part).sum();
            assert!(
                (somme - r).abs() < 1e-4,
                "{x:?} → {y:?} : {somme} contre {r}"
            );
        }
    }
    assert!(sens.pourquoi("", "l'eau").is_empty(), "un texte sans sens");
    assert!(
        sens.pourquoi("l'eau", " ").is_empty(),
        "vers un texte sans sens"
    );
}

/// **B3 — ce qui rapproche deux textes, c'est ce qu'ils partagent** : entre un texte sur la pluie et
/// un autre sur la pluie et les nappes, le mot « pluie » pèse le plus ; un mot répété, aux
/// majuscules près, n'en fait qu'un, qui cumule ses parts — « l'eau » et « Eau » sont un seul
/// « eau » —, le plus fort d'abord.
#[test]
fn pourquoi_1_le_mot_partage_pese_le_plus() {
    let sens = modele();
    let a = "Pluie sur Lyon : la pluie, encore la pluie.";
    let parts = sens.pourquoi(
        a,
        "Les nappes souterraines se remplissent quand il pleut, la pluie.",
    );
    assert_eq!(parts[0].mot, "Pluie", "{parts:?}");
    let pluie: Vec<_> = parts
        .iter()
        .filter(|p| p.mot.to_lowercase() == "pluie")
        .collect();
    assert_eq!(pluie.len(), 1, "un seul « pluie » : {parts:?}");
    let seul = sens.pourquoi(
        "pluie",
        "Les nappes souterraines se remplissent quand il pleut, la pluie.",
    );
    assert!(
        pluie[0].part > seul[0].part * 0.5,
        "ses trois parts s'ajoutent : {parts:?}"
    );
    assert!(
        parts.windows(2).all(|w| w[0].part >= w[1].part),
        "le plus fort d'abord : {parts:?}"
    );
    let eau = sens.pourquoi("l'eau, Eau et d'eau", "la pluie");
    let eau: Vec<_> = eau
        .iter()
        .filter(|p| p.mot.to_lowercase() == "eau")
        .collect();
    assert_eq!(
        eau.len(),
        1,
        "l'eau, Eau, d'eau : un seul « eau » : {eau:?}"
    );
    assert_eq!(eau[0].mot, "eau", "sa première écriture");
}
