//! **Les mots d'une commande, devenus les arguments de son outil** — lus par le schéma de l'outil,
//! jamais par une liste à part (TERMINAL-1).
//!
//! * `clé=valeur` : la valeur prend le type que le schéma donne à la clé — un texte, un nombre,
//!   `true` ou `false`, ou un élément d'une liste ; répéter la clé ajoute un élément
//!   (`ids=a ids=b`). Une liste ou un objet s'écrit aussi en JSON (`domaines='[{"nouveau":"eau"}]'`).
//! * **un mot sans `=`** va à l'argument que l'outil exige, s'il n'en exige qu'un :
//!   `glucose chercher eau chaude`, `glucose poser-texte "Bonjour"`, `glucose retirer n1 n2`.
//! * **`-`** comme valeur lit l'entrée standard : `cat notes.md | glucose poser-texte -`.
//!
//! Une clé que l'outil ne connaît pas est refusée, avec la liste de celles qu'il connaît : deviner
//! ce qu'on a voulu dire écrirait autre chose que ce qu'on a demandé.

use crate::agents::json::Json;

/// Le type qu'un schéma donne à une clé.
fn type_de(schema: &Json, cle: &str) -> Option<String> {
    schema
        .get("properties")?
        .get(cle)?
        .get("type")?
        .as_str()
        .map(str::to_string)
}

/// Les clés que l'outil connaît, dans l'ordre de son schéma.
pub fn cles(schema: &Json) -> Vec<String> {
    match schema.get("properties") {
        Some(Json::Objet(paires)) => paires.iter().map(|(k, _)| k.clone()).collect(),
        _ => Vec::new(),
    }
}

/// L'argument que l'outil exige, s'il n'en exige qu'un : celui des mots sans `=`.
fn requis_unique(schema: &Json) -> Option<String> {
    match schema.get("required").and_then(Json::as_liste) {
        Some([seul]) => seul.as_str().map(str::to_string),
        _ => None,
    }
}

/// **Une valeur écrite au terminal, au type de sa clé.**
fn valeur(type_: &str, cle: &str, ecrit: &str) -> Result<Json, String> {
    match type_ {
        "number" => ecrit
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .map(Json::from)
            .ok_or_else(|| format!("« {cle} » attend un nombre, pas « {ecrit} »")),
        "boolean" => match ecrit.trim() {
            "true" | "oui" => Ok(Json::Bool(true)),
            "false" | "non" => Ok(Json::Bool(false)),
            _ => Err(format!("« {cle} » attend true ou false, pas « {ecrit} »")),
        },
        _ => Ok(Json::from(ecrit)),
    }
}

/// Pose `ecrit` sous `cle`, à son type : un élément de plus pour une liste.
fn poser(
    args: &mut Vec<(String, Json)>,
    schema: &Json,
    cle: &str,
    ecrit: &str,
) -> Result<(), String> {
    let Some(type_) = type_de(schema, cle) else {
        return Err(format!(
            "« {cle} » n'est pas un argument de cette commande ; elle connaît : {}",
            cles(schema).join(", ")
        ));
    };
    if type_ != "array" {
        let v = valeur(&type_, cle, ecrit)?;
        args.retain(|(k, _)| k != cle);
        args.push((cle.to_string(), v));
        return Ok(());
    }
    let elements = match Json::lire(ecrit.trim()) {
        Ok(Json::Liste(l)) => l,
        _ => vec![Json::from(ecrit)],
    };
    match args.iter_mut().find(|(k, _)| k == cle) {
        Some((_, Json::Liste(l))) => l.extend(elements),
        _ => args.push((cle.to_string(), Json::Liste(elements))),
    }
    Ok(())
}

/// **Les arguments de l'outil**, lus des mots de la commande ; `entree` rend l'entrée standard,
/// lue une seule fois, pour une valeur `-`.
pub fn lire(
    schema: &Json,
    mots: &[String],
    entree: &mut dyn FnMut() -> Result<String, String>,
) -> Result<Json, String> {
    let mut args: Vec<(String, Json)> = Vec::new();
    let mut nus: Vec<&str> = Vec::new();
    let mut lue: Option<String> = None;
    for mot in mots {
        let Some((cle, ecrit)) = mot.split_once('=') else {
            nus.push(mot);
            continue;
        };
        let ecrit = if ecrit == "-" {
            if lue.is_none() {
                lue = Some(entree()?);
            }
            lue.clone().unwrap_or_default()
        } else {
            ecrit.to_string()
        };
        poser(&mut args, schema, cle, &ecrit)?;
    }
    if !nus.is_empty() {
        let Some(cle) = requis_unique(schema) else {
            return Err(format!(
                "« {} » : cette commande ne prend que des clé=valeur ({})",
                nus.join(" "),
                cles(schema).join(", ")
            ));
        };
        if type_de(schema, &cle).as_deref() == Some("array") {
            for mot in nus {
                poser(&mut args, schema, &cle, mot)?;
            }
        } else {
            let ecrit = match nus.as_slice() {
                ["-"] => entree()?,
                _ => nus.join(" "),
            };
            poser(&mut args, schema, &cle, &ecrit)?;
        }
    }
    Ok(Json::Objet(args))
}
