//! **Ce qu'un geste fait aux nœuds** (MCP-NEUF, fiche 144) : pour « quoi de neuf », chaque nœud
//! qu'un geste touche — son tableau, son identifiant, sa sorte, s'il existait avant et après —,
//! et le reste en mots. Comparer le premier « avant » et le dernier « après » d'un nœud sur une
//! suite de gestes dit ce qui a changé sans rejouer l'histoire.

use super::{Edit, Slot, Transaction};
use crate::types::Annotation;

/// **Un nœud qu'un geste touche.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Touche {
    pub board: String,
    pub id: String,
    /// « texte », « post-it », « flèche », « membrane », « photo », « dossier ».
    pub sorte: &'static str,
    /// Il existait avant ce geste.
    pub avant: bool,
    /// Il existe après ce geste.
    pub apres: bool,
    /// Son texte avant et après ce geste — un texte, un post-it, le titre d'une membrane
    /// (MCP-CHANGE, fiche 180) : de quoi dire ce qui y a changé sans le relire entier.
    pub textes: [Option<String>; 2],
}

/// **Un effet d'un geste** : sur un nœud, ou autre chose, en mots (« déplace des nœuds »).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effet {
    Noeud(Touche),
    Autre(String),
}

impl Transaction {
    /// **Un effet par édition**, dans l'ordre du geste.
    pub fn effets(&self) -> Vec<Effet> {
        self.edits.iter().map(effet).collect()
    }
}

fn effet(edit: &Edit) -> Effet {
    let touche = match edit {
        Edit::Image { board, slot } => toucher(board, slot, |i| ("photo", i.id.clone()), |_| None),
        Edit::Annotation { board, slot } => toucher(
            board,
            slot,
            |a| (sorte(a), a.id().to_string()),
            Annotation::own_text,
        ),
        Edit::Folder { board, slot } => {
            toucher(board, slot, |d| ("dossier", d.id.clone()), |_| None)
        }
        _ => None,
    };
    match touche {
        Some(t) => Effet::Noeud(t),
        None => Effet::Autre(super::en_mots::une_edition(edit)),
    }
}

fn toucher<T>(
    board: &str,
    slot: &Slot<T>,
    nommer: impl Fn(&T) -> (&'static str, String),
    ecrit: impl Fn(&T) -> Option<String>,
) -> Option<Touche> {
    let (sorte, id) = slot
        .after
        .as_deref()
        .or(slot.before.as_deref())
        .map(nommer)?;
    Some(Touche {
        board: board.to_string(),
        id,
        sorte,
        avant: slot.before.is_some(),
        apres: slot.after.is_some(),
        textes: [&slot.before, &slot.after].map(|n| n.as_deref().and_then(&ecrit)),
    })
}

fn sorte(a: &Annotation) -> &'static str {
    match a {
        Annotation::Text { .. } => "texte",
        Annotation::Sticky { .. } => "post-it",
        Annotation::Arrow { .. } => "flèche",
        Annotation::Membrane { .. } => "membrane",
    }
}

#[cfg(test)]
mod tests {
    use super::super::Whole;
    use super::*;

    fn case(avant: Option<&Annotation>, apres: Option<&Annotation>) -> Edit {
        Edit::Annotation {
            board: "b".to_string(),
            slot: Slot {
                index: 0,
                before: avant.cloned().map(Box::new),
                after: apres.cloned().map(Box::new),
            },
        }
    }

    /// **Un geste dit quels nœuds il touche**, s'ils existaient avant et après ; le reste en mots.
    #[test]
    fn mcp_neuf_un_geste_dit_les_noeuds_qu_il_touche() {
        let texte = Annotation::text("a-12", 0.0, 0.0, "Eau");
        let fleche = Annotation::arrow("f-3", 0.0, 0.0, 1.0, 1.0);
        let t = Transaction {
            edits: vec![
                case(None, Some(&texte)),
                case(Some(&fleche), None),
                case(Some(&texte), Some(&texte)),
                Edit::ActiveBoard {
                    whole: Whole {
                        before: "b".to_string(),
                        after: "c".to_string(),
                    },
                },
            ],
        };
        let eau = || Some("Eau".to_string());
        let touche = |id: &str, sorte, avant, apres, textes| {
            Effet::Noeud(Touche {
                board: "b".to_string(),
                id: id.to_string(),
                sorte,
                avant,
                apres,
                textes,
            })
        };
        assert_eq!(
            t.effets(),
            vec![
                touche("a-12", "texte", false, true, [None, eau()]),
                touche("f-3", "flèche", true, false, [None, None]),
                touche("a-12", "texte", true, true, [eau(), eau()]),
                Effet::Autre("change de tableau".to_string()),
            ]
        );
    }
}
