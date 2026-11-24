//! **La flèche en double, vue du store** (sa note du 24/11 ; fiche 149 § 2).

use super::Store;
use crate::types::Annotation;

impl Store {
    /// **La flèche qui relie déjà les bouts de la flèche `id`** — mêmes nœuds, mêmes passages,
    /// dans un sens ou dans l'autre : son identifiant, ou `None`.
    pub fn fleche_qui_relie_deja(&self, board_id: &str, id: &str) -> Option<String> {
        let fleche = self.project.annotation(board_id, id)?;
        crate::arrow::qui_relie_deja(self.annotations_du_tableau(board_id), fleche)
            .map(|a| a.id().to_string())
    }

    /// Les annotations d'un tableau ; aucune s'il n'existe pas.
    pub(crate) fn annotations_du_tableau(&self, board_id: &str) -> &[Annotation] {
        self.project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .map_or(&[], |b| b.annotations.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::store::agents::Lien;
    use crate::store::Store;
    use crate::types::{Annotation, ArrowPredicate};

    /// **Un agent qui relie deux nœuds déjà reliés reprend la flèche** (fiche 149 § 2) : il y écrit
    /// ce qu'il dit de neuf, et rend son identifiant ; une autre relation a sa propre flèche.
    #[test]
    fn test_double_l_agent_reprend_la_fleche_qui_dit_la_meme_chose() {
        let mut store = Store::new("double");
        let board = store.project.active_board_id.clone();
        for (id, x) in [("A", 0.0), ("B", 600.0)] {
            store.add_annotation(&board, Annotation::text(id, x, 0.0, "un texte"));
        }
        let mut relier = |id: &'static str, relation, texte| {
            store.geste_d_agent(|s| {
                let lien = Lien {
                    id,
                    source: "B",
                    cible: "A",
                    relation,
                    texte,
                    ..Lien::default()
                };
                s.relier_pour_un_agent(&board, lien)
            })
        };
        assert_eq!(relier("f", None, None).as_deref(), Ok("f"));
        assert_eq!(relier("g", None, Some("réponse")).as_deref(), Ok("f"));
        let contredit = Some(ArrowPredicate::Contredit);
        assert_eq!(relier("h", contredit, None).as_deref(), Ok("f"));
        assert_eq!(
            relier("i", Some(ArrowPredicate::Inspire), None).as_deref(),
            Ok("i"),
            "une autre affirmation"
        );
        assert!(matches!(
            store.project.annotation(&board, "f"),
            Some(Annotation::Arrow { text: Some(t), predicate: Some(ArrowPredicate::Contredit), .. })
                if t == "réponse"
        ));
        assert!(store.project.annotation(&board, "g").is_none());
    }
}
