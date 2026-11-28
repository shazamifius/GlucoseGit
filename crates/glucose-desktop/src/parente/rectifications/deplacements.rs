//! **RECTIF-2 — ses déplacements** (fiche 190) : quand l'humain déplace à la main ce qu'un agent a
//! posé, l'IA de Glucose écrit d'où à où, et de quelle membrane à quelle autre.
//!
//! Sa note du 27/11 (`text-n3rk-2495`) : *« des outils […] pour dire en gros pourquoi l'IA s'est
//! trompée, mais sans user de mots — juste avec tes intentions, tes clics […] pris en compte à vie
//! par l'IA »*. Ses clics sont déjà des gestes du document : l'écrit les nomme, et l'IA suivante le
//! lit dans `glucose_histoire` avant de poser ; il voyage avec le partage, comme l'histoire.
//!
//! # Ce qui compte comme son intention
//!
//! **Un geste, un écrit** : il nomme chaque nœud déplacé, sans seuil (RECTIF-1). En sont exclus :
//!
//! * **ce qu'une membrane emporte** — sa place dans sa membrane n'a pas changé ; seule la membrane
//!   déplacée, si un agent l'a posée, est nommée ;
//! * **ce que pousse un texte qu'il écrit** (POUSSER-145) : un pas de saisie (`etiquette`), pas un
//!   avis sur la place ;
//! * les flèches, qui suivent leurs bouts ; ce que l'humain a déjà fait sien (MAIN-1) ; le geste
//!   d'un agent.

use glucose_core::store::journal::{Ecrit, Edit, Main};
use glucose_core::types::{Annotation, Project};
use std::collections::{HashMap, HashSet};

/// Au plus tant de nœuds décrits en toutes lettres ; les suivants sont comptés — l'écrit les
/// nomme tous dans ses sujets.
const DECRITS: usize = 5;

/// Ce qu'un nœud a encore bougé après un geste, dans les gestes suivants du même lot.
type Decalages = HashMap<String, (f64, f64)>;

/// **Ce que ces gestes déplacent de ce que des agents ont posé** : le rang de chaque geste, ses
/// nœuds, et ce qu'en dit l'écrit. Le projet et la provenance sont ceux d'après le lot : la place
/// d'un nœud à un geste se retrouve en défaisant les déplacements qui l'ont suivi, et ce qui était
/// à un agent en lui rendant ce que l'humain a repris depuis — un nœud glissé dans une membrane y
/// est rattaché par le même geste, qui le fait sien (MAIN-1). D'où le lot lu à rebours.
pub(super) fn des_gestes(
    ecrits: &[Ecrit],
    projet: &Project,
    des_agents: &HashSet<String>,
) -> Vec<(usize, Vec<String>, String)> {
    let mut decalages = Decalages::new();
    let mut repris_depuis: HashSet<&str> = HashSet::new();
    let mut faits = Vec::new();
    for (geste, ecrit) in ecrits.iter().enumerate().rev() {
        repris_depuis.extend(ecrit.repris.iter().map(String::as_str));
        let d_un_agent = |id: &str| des_agents.contains(id) || repris_depuis.contains(id);
        if let Some((sujets, dit)) = d_un_geste(ecrit, (projet, &decalages), d_un_agent) {
            faits.push((geste, sujets, dit));
        }
        for (_, a, (dx, dy), _) in emportes(ecrit, projet) {
            let d = decalages.entry(a.id().to_string()).or_default();
            (d.0, d.1) = (d.0 + dx, d.1 + dy);
        }
    }
    faits.reverse();
    faits
}

/// Ce que les translations de ce geste emportent, tel que le projet le porte : le tableau, le
/// nœud, le vecteur, et si sa membrane bougeait avec lui.
fn emportes<'a>(
    ecrit: &'a Ecrit,
    projet: &'a Project,
) -> Vec<(&'a str, &'a Annotation, (f64, f64), bool)> {
    let mut tout = Vec::new();
    for edit in &ecrit.transaction.edits {
        let Edit::Translation {
            board,
            delta,
            annotations,
            ..
        } = edit
        else {
            continue;
        };
        let les: Vec<&Annotation> = (annotations.iter())
            .filter_map(|&i| projet.annotation_au_rang(board, i as usize))
            .collect();
        let bougent: HashSet<&str> = les.iter().map(|a| a.id()).collect();
        for a in les {
            let porte = a.membrane_id().is_some_and(|m| bougent.contains(m));
            tout.push((board.as_str(), a, *delta, porte));
        }
    }
    tout
}

/// **Ce que ce geste de l'humain déplace de ce que des agents ont posé** : les nœuds, et ce qu'en
/// dit l'écrit ; rien si aucun.
fn d_un_geste(
    ecrit: &Ecrit,
    (projet, decalages): (&Project, &Decalages),
    d_un_agent: impl Fn(&str) -> bool,
) -> Option<(Vec<String>, String)> {
    if ecrit.main != Main::Humain || ecrit.etiquette.is_some() {
        return None;
    }
    let mut lignes: Vec<(String, String)> = Vec::new();
    for (board, a, delta, porte) in emportes(ecrit, projet) {
        if matches!(a, Annotation::Arrow { .. }) || porte || !d_un_agent(a.id()) {
            continue;
        }
        let apres = decalages.get(a.id()).copied().unwrap_or_default();
        let apres = (a.x() - apres.0, a.y() - apres.1);
        let ligne = decrire(a, (apres, delta), (board, projet), &ecrit.transaction.edits);
        lignes.push((a.id().to_string(), ligne));
    }
    if lignes.is_empty() {
        return None;
    }
    let mut dit = String::from("L'humain a déplacé à la main ce qu'un agent avait posé : ");
    let decrits: Vec<&str> = lignes
        .iter()
        .take(DECRITS)
        .map(|(_, l)| l.as_str())
        .collect();
    dit.push_str(&decrits.join(" ; "));
    if lignes.len() > DECRITS {
        dit.push_str(&format!(" ; et {} autres", lignes.len() - DECRITS));
    }
    dit.push('.');
    Some((lignes.into_iter().map(|(id, _)| id).collect(), dit))
}

/// « « début » de (x, y) à (x', y') », et la membrane quittée et rejointe si ce geste l'a changée.
fn decrire(
    a: &Annotation,
    ((x, y), (dx, dy)): ((f64, f64), (f64, f64)),
    (board, projet): (&str, &Project),
    edits: &[Edit],
) -> String {
    let mut dit = format!(
        "{} de ({:.0}, {:.0}) à ({x:.0}, {y:.0})",
        nom(a),
        x - dx,
        y - dy
    );
    let rattache = edits.iter().find_map(|e| match e {
        Edit::Annotation { board: b, slot } if b == board => {
            let (avant, apres) = (slot.before.as_deref()?, slot.after.as_deref()?);
            (avant.id() == a.id() && avant.membrane_id() != apres.membrane_id())
                .then(|| (avant.membrane_id(), apres.membrane_id()))
        }
        _ => None,
    });
    if let Some((de, vers)) = rattache {
        let titre = |m: Option<&str>| match m.and_then(|m| projet.annotation(board, m)) {
            Some(m) => nom(m),
            None => "aucune membrane".to_string(),
        };
        dit.push_str(&format!(", de {} à {}", titre(de), titre(vers)));
    }
    dit
}

/// Un nœud nommé comme l'humain le reconnaît : le début de son texte ou de son titre, entre
/// guillemets, sinon sa sorte et son identifiant.
fn nom(a: &Annotation) -> String {
    let texte = match a {
        Annotation::Text { text, .. } | Annotation::Sticky { text, .. } => Some(text.as_str()),
        Annotation::Membrane { text, .. } => text.as_deref(),
        Annotation::Arrow { .. } => None,
    };
    match texte.map(super::debut).filter(|t| !t.is_empty()) {
        Some(t) => format!("« {t} »"),
        None => format!("le nœud {}", a.id()),
    }
}
