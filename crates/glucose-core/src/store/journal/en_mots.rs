//! **Ce qu'un geste a fait, en mots** (NOTE-1, fiche 108) : la Time Machine relue par une IA dit,
//! pour chaque geste, ce qu'il a posé, changé ou retiré — de quoi comprendre un geste avant d'en
//! écrire quelque chose (sa parole du 12/11/2026 : « des écrits de ce qu'une IA comprend de ce
//! geste, et pourquoi c'est important »).

use super::{Edit, Slot, Transaction};
use crate::types::Annotation;

impl Transaction {
    /// **Une ligne par édition** : « pose le texte « a-12 » », « retire la flèche « f-3 » »,
    /// « déplace 4 nœuds »…
    pub fn en_mots(&self) -> Vec<String> {
        self.edits.iter().map(en_mots).collect()
    }
}

/// Le verbe d'une case : un élément qui apparaît est posé, un qui disparaît est retiré.
fn verbe<T>(slot: &Slot<T>) -> &'static str {
    match (&slot.before, &slot.after) {
        (None, Some(_)) => "pose",
        (Some(_), None) => "retire",
        _ => "change",
    }
}

/// La case d'une liste, en mots : son verbe, la sorte de l'élément, son identifiant.
fn la_case<T>(slot: &Slot<T>, nommer: impl Fn(&T) -> (&'static str, String)) -> String {
    match slot.after.as_deref().or(slot.before.as_deref()).map(nommer) {
        Some((sorte, id)) => format!("{} {sorte} « {id} »", verbe(slot)),
        None => format!("{} un élément", verbe(slot)),
    }
}

/// La sorte d'une annotation, avec son article.
fn sorte(a: &Annotation) -> &'static str {
    match a {
        Annotation::Text { .. } => "le texte",
        Annotation::Sticky { .. } => "le post-it",
        Annotation::Arrow { .. } => "la flèche",
        Annotation::Membrane { .. } => "la membrane",
    }
}

fn en_mots(edit: &Edit) -> String {
    match edit {
        Edit::Image { slot, .. } => la_case(slot, |i| ("la photo", i.id.clone())),
        Edit::Annotation { slot, .. } => la_case(slot, |a| (sorte(a), a.id().to_string())),
        Edit::Folder { slot, .. } => la_case(slot, |d| ("le dossier", d.name.clone())),
        Edit::Panel { slot, .. } => la_case(slot, |c| ("la case", c.id.clone())),
        Edit::Storyboard { slot, .. } => la_case(slot, |s| ("le storyboard", s.id.clone())),
        Edit::Translation { .. } => "déplace des nœuds".to_string(),
        Edit::Zones { .. } => "pose les zones d'un gabarit".to_string(),
        Edit::BoardName { .. } => "renomme un tableau".to_string(),
        Edit::Board { .. } => "pose ou retire un tableau".to_string(),
        Edit::Domain { .. } => "change un domaine".to_string(),
        Edit::Preset { .. } => "change le gabarit".to_string(),
        Edit::ProjectName { .. } => "renomme le document".to_string(),
        Edit::ActiveBoard { .. } => "change de tableau".to_string(),
        Edit::BoardOrder { .. } => "range les tableaux".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Edit, Slot, Transaction};
    use crate::types::Annotation;

    /// **Un geste se dit en mots** : poser, changer, retirer, avec la sorte et l'identifiant.
    #[test]
    fn note_1_un_geste_se_dit_en_mots() {
        let texte = Annotation::text("a-12", 0.0, 0.0, "Eau");
        let fleche = Annotation::arrow("f-3", 0.0, 0.0, 1.0, 1.0);
        let case = |avant: Option<&Annotation>, apres: Option<&Annotation>| Edit::Annotation {
            board: "b".to_string(),
            slot: Slot {
                index: 0,
                before: avant.cloned().map(Box::new),
                after: apres.cloned().map(Box::new),
            },
        };
        let geste = Transaction {
            edits: vec![
                case(None, Some(&texte)),
                case(Some(&texte), Some(&texte)),
                case(Some(&fleche), None),
            ],
        };
        assert_eq!(
            geste.en_mots(),
            [
                "pose le texte « a-12 »",
                "change le texte « a-12 »",
                "retire la flèche « f-3 »"
            ]
        );
    }
}
