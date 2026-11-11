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
