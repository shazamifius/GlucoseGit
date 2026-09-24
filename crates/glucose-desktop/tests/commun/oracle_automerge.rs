//! L'oracle : ce que la bibliothèque de référence d'Automerge lit dans les mêmes octets,
//! traduit dans notre [`Valeur`] pour une comparaison arbre contre arbre.
//!
//! Partagé par `tests/tauri_suite.rs` et `examples/oracle_tauri.rs` (`#[path]`).

use automerge::{hydrate, ScalarValue};
use glucose_core::persist::tauri::Valeur;

/// La valeur que la référence tire de ces octets.
pub fn reference(octets: &[u8]) -> Result<Valeur, String> {
    let doc = automerge::Automerge::load(octets).map_err(|e| e.to_string())?;
    Ok(vers_valeur(&doc.hydrate(None)))
}

pub fn vers_valeur(v: &hydrate::Value) -> Valeur {
    match v {
        hydrate::Value::Scalar(s) => match s {
            ScalarValue::Bytes(b) => Valeur::Octets(b.clone()),
            ScalarValue::Str(s) => Valeur::Texte(s.to_string()),
            ScalarValue::Int(n) | ScalarValue::Timestamp(n) => Valeur::Entier(*n),
            ScalarValue::Uint(n) => Valeur::Entier(i64::try_from(*n).unwrap_or(i64::MAX)),
            ScalarValue::F64(x) => Valeur::Flottant(*x),
            ScalarValue::Counter(c) => Valeur::Entier(i64::from(c)),
            ScalarValue::Boolean(b) => Valeur::Booleen(*b),
            ScalarValue::Null | ScalarValue::Unknown { .. } => Valeur::Nul,
        },
        hydrate::Value::Map(m) => Valeur::Carte(
            m.iter()
                .map(|(k, mv)| (k.clone(), vers_valeur(&mv.value)))
                .collect(),
        ),
        hydrate::Value::List(l) => {
            Valeur::Liste(l.iter().map(|lv| vers_valeur(&lv.value)).collect())
        }
        hydrate::Value::Text(t) => Valeur::Texte(String::from(t)),
    }
}

/// Le premier endroit où deux valeurs diffèrent, sous la forme d'un chemin lisible.
pub fn premiere_difference(a: &Valeur, b: &Valeur, chemin: &str) -> Option<String> {
    match (a, b) {
        (Valeur::Carte(x), Valeur::Carte(y)) => {
            let nul = Valeur::Nul;
            for k in x.keys().chain(y.keys()) {
                let (va, vb) = (x.get(k).unwrap_or(&nul), y.get(k).unwrap_or(&nul));
                if let Some(d) = premiere_difference(va, vb, &format!("{chemin}.{k}")) {
                    return Some(d);
                }
            }
            None
        }
        (Valeur::Liste(x), Valeur::Liste(y)) => {
            if x.len() != y.len() {
                return Some(format!(
                    "{chemin} : {} éléments contre {}",
                    x.len(),
                    y.len()
                ));
            }
            x.iter()
                .zip(y)
                .enumerate()
                .find_map(|(i, (va, vb))| premiere_difference(va, vb, &format!("{chemin}[{i}]")))
        }
        _ if a == b => None,
        _ => Some(format!("{chemin} : {a:?} contre {b:?}")),
    }
}
