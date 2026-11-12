//! **RECTIF-1 — le journal des rectifications** (fiche 110) : quand l'humain fait autrement que ce
//! que l'IA de Glucose proposait, elle l'écrit dans l'histoire du document.
//!
//! Sa parole du 11/11/2026 (VISION § 8 bis) : *« un espèce d'énorme journal en arrière-plan, qui
//! est là uniquement pour la rectification […] l'utilisateur a mis cette couleur-là alors que l'IA
//! pensait que c'était ça ; alors elle essaie de chercher, et note son erreur ou tout ce qu'elle
//! pense »* ; et le 12/11 : *« on UTILISE la Time Machine »*. Une rectification est donc un **écrit**
//! (NOTE-1) que l'IA de Glucose attache au geste de l'humain : un fait, un seul endroit —
//! l'histoire —, que la Time Machine et `glucose_histoire` montrent déjà.
//!
//! # Ce qui s'écrit
//!
//! Ses **gestes ordinaires** qui font autrement que l'IA (B3 de sa page, les gestes d'abord) :
//!
//! * il donne sa couleur à un texte que l'IA rangeait dans une famille — le halo montrait la
//!   couleur de l'IA, il montre la sienne ;
//! * il retouche ou retire une flèche qu'un agent avait posée et que l'IA jugeait ;
//! * il suit (`Ctrl`+clic) une telle flèche.
//!
//! **Aucun seuil** : chaque geste sur ce que l'IA jugeait s'écrit, avec sa sûreté — celui qui la
//! contredit comme celui qui la confirme. Couper à « sûre à moins de 50 % » aurait jeté ce qu'une
//! correction apprise (D2) saura peser ; la retouche de l'humain dit « mieux que la proposition »,
//! pas « la bonne réponse » (Tucker et al., ICML 2024), et c'est un fait à garder tel quel.
//!
//! **Les faits, et le pourquoi** : ce que l'IA pensait (la famille et sa sûreté, le texte qui
//! ressemble le plus, et les mots qui les rapprochent — exacts, [`glucose_core::sens::Sens::pourquoi`]),
//! ce que l'humain a fait. Tout le reste se relit : l'état du document à ce geste est dans la Time
//! Machine, et l'IA de Glucose est déterministe.
//!
//! # Se souvenir (D1)
//!
//! Une flèche retouchée devient celle de l'humain, et l'IA ne la juge plus : MAIN-1 le fait, par un
//! geste qui se défait. **Suivre une flèche n'y change rien** : ce n'est pas un geste du document —
//! il ne se défait pas —, et un seul clic est un signal faible (on suit aussi pour voir). L'écrit le
//! garde ; ce qu'il vaut se décidera sur ce que le journal récolte (sa parole : *« on regarde
//! d'abord ce que le journal récolte »*).

use crate::renderer::sens::SensDesTextes;
use glucose_core::store::journal::{Ecrit, Edit, Main};
use glucose_core::types::{Annotation, Project};

/// **Une rectification** : le rang de son geste dans la file, le nœud dont elle parle, et ce
/// qu'elle en dit.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Rectification {
    pub geste: usize,
    pub sujet: String,
    pub texte: String,
}

/// **Ce que ces gestes rectifient** de ce que l'IA proposait — seulement ceux de l'humain : un
/// agent qui recolore ne corrige pas l'IA de Glucose. L'avis lu est celui qu'elle montrait avant
/// eux : le fil ne le refait qu'après.
pub(crate) fn des_gestes(
    ecrits: &[Ecrit],
    projet: &Project,
    sens: &SensDesTextes,
) -> Vec<Rectification> {
    let mut faites = Vec::new();
    for (geste, ecrit) in ecrits.iter().enumerate() {
        if ecrit.main != Main::Humain {
            continue;
        }
        for edit in &ecrit.transaction.edits {
            let Edit::Annotation { board, slot } = edit else {
                continue;
            };
            let (avant, apres) = (slot.before.as_deref(), slot.after.as_deref());
            let dit = match (avant, apres) {
                (Some(Annotation::Text { color: None, .. }), Some(texte)) => {
                    d_un_texte_colore(texte, board, projet, sens)
                }
                (Some(fleche @ Annotation::Arrow { .. }), apres)
                    if ecrit.repris.iter().any(|r| r == fleche.id()) =>
                {
                    d_une_fleche_reprise(fleche.id(), board, apres.is_none(), sens)
                }
                _ => None,
            };
            if let Some((sujet, texte)) = dit {
                faites.push(Rectification {
                    geste,
                    sujet,
                    texte,
                });
            }
        }
    }
    faites
}

/// Un texte que l'IA rangeait dans une famille, et auquel l'humain vient de donner sa couleur.
fn d_un_texte_colore(
    texte: &Annotation,
    board: &str,
    projet: &Project,
    sens: &SensDesTextes,
) -> Option<(String, String)> {
    let Annotation::Text {
        id,
        text,
        color: Some(couleur),
        ..
    } = texte
    else {
        return None;
    };
    let avis = sens.avis_du_tableau(board)?.get(id)?;
    avis.famille?;
    let mut dit = format!(
        "L'IA de Glucose rangeait ce texte dans une famille, sûre à {} %",
        pour_cent(avis.surete)
    );
    let parent = avis
        .parent
        .as_deref()
        .and_then(|p| projet.annotation(board, p));
    if let Some(Annotation::Text { text: voisin, .. }) = parent {
        dit.push_str(&format!(" : il ressemble le plus à « {} »", debut(voisin)));
        let mots = glucose_core::sens::embarque().map(|m| m.pourquoi(text, voisin));
        let mots = mots.map(|m| les_mots_qui_pesent(&m));
        if let Some(mots) = mots.filter(|m| !m.is_empty()) {
            dit.push_str(&format!(", par ses mots {mots}"));
        }
    }
    dit.push_str(&format!(". L'humain lui a donné sa couleur, {couleur}."));
    Some((id.clone(), dit))
}

/// Une flèche qu'un agent avait posée, que l'IA jugeait, et que l'humain retouche ou retire.
fn d_une_fleche_reprise(
    id: &str,
    board: &str,
    retiree: bool,
    sens: &SensDesTextes,
) -> Option<(String, String)> {
    let surete = sens.avis_du_tableau(board)?.get(id)?.surete;
    let fait = if retiree {
        "L'humain l'a retirée."
    } else {
        "L'humain l'a retouchée : elle est désormais la sienne, et l'IA ne la juge plus."
    };
    let dit = format!(
        "Cette flèche, posée par un agent, était jugée par l'IA de Glucose : sûre à {} % que ses \
         deux bouts parlent de la même chose. {fait}",
        pour_cent(surete)
    );
    Some((id.to_string(), dit))
}

/// **Une flèche qu'il suit** (`Ctrl`+clic), posée par un agent et que l'IA jugeait : ce qu'en dit
/// l'écrit. Rien pour une flèche de l'humain, ou que l'IA ne juge pas.
pub(crate) fn d_une_fleche_suivie(
    id: &str,
    board: &str,
    des_agents: &std::collections::HashSet<String>,
    sens: &SensDesTextes,
) -> Option<String> {
    if !des_agents.contains(id) {
        return None;
    }
    let surete = sens.avis_du_tableau(board)?.get(id)?.surete;
    Some(format!(
        "L'humain a suivi cette flèche, posée par un agent ; l'IA de Glucose la jugeait sûre à {} % \
         que ses deux bouts parlent de la même chose.",
        pour_cent(surete)
    ))
}

/// Une sûreté, en pour cent entier.
fn pour_cent(surete: f32) -> String {
    format!("{:.0}", f64::from(surete.clamp(0.0, 1.0)) * 100.0)
}

/// Le début d'un texte : sa première ligne, coupée à soixante caractères.
fn debut(texte: &str) -> String {
    let ligne = texte.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let court: String = ligne.trim().chars().take(60).collect();
    if court.chars().count() < ligne.trim().chars().count() {
        format!("{court}…")
    } else {
        court
    }
}

/// **Les trois mots qui pèsent le plus**, entre guillemets : ceux qui rapprochent (une part
/// positive), d'au moins deux lettres ou chiffres — « l » d'une élision ou une virgule pèsent, mais
/// n'expliquent rien à qui lit.
fn les_mots_qui_pesent(parts: &[glucose_core::sens::Part]) -> String {
    let mots: Vec<String> = parts
        .iter()
        .filter(|p| p.part > 0.0 && p.mot.chars().filter(|c| c.is_alphanumeric()).count() >= 2)
        .take(3)
        .map(|p| format!("« {} »", p.mot))
        .collect();
    match mots.as_slice() {
        [] => String::new(),
        [seul] => seul.clone(),
        [debut @ .., dernier] => format!("{} et {dernier}", debut.join(", ")),
    }
}

#[cfg(test)]
mod tests;
