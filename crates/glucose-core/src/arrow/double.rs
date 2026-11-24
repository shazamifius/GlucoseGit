//! **La flèche en double** (sa note du 24/11, `text-pxtg-2317` ; fiche 149 § 2) : deux flèches
//! aux mêmes bouts suivent le même itinéraire, et la seconde se cache exactement sous la première.

use crate::types::{Annotation, ArrowPredicate, TextSelection};

/// Un bout de flèche : son nœud, et le passage qu'elle y désigne, s'il y en a un.
type Bout<'a> = (&'a str, Option<&'a str>, Option<&'a TextSelection>);

/// Les deux bouts d'une flèche accrochée à deux nœuds ; `None` si l'un est libre.
fn bouts(fleche: &Annotation) -> Option<(Bout<'_>, Bout<'_>)> {
    let Annotation::Arrow {
        source_id: Some(source),
        target_id: Some(cible),
        source_block_id,
        target_block_id,
        source_text_sel,
        target_text_sel,
        ..
    } = fleche
    else {
        return None;
    };
    Some((
        (
            source.as_str(),
            source_block_id.as_deref(),
            source_text_sel.as_ref(),
        ),
        (
            cible.as_str(),
            target_block_id.as_deref(),
            target_text_sel.as_ref(),
        ),
    ))
}

/// **La flèche qui relie déjà les bouts de `fleche`** parmi `annotations` : mêmes nœuds, mêmes
/// passages, dans un sens ou dans l'autre — leurs traits se confondent. Une flèche dont un bout
/// est libre n'en double aucune ; `fleche` elle-même n'est pas comptée.
pub fn qui_relie_deja<'a>(
    annotations: &'a [Annotation],
    fleche: &'a Annotation,
) -> Option<&'a Annotation> {
    memes_bouts(annotations, fleche).next()
}

/// **La flèche qu'un agent reprend au lieu d'en tracer une seconde** : l'une de celles qui
/// relient déjà ces bouts et ne disent pas autre chose — la même relation d'abord, sinon une qui
/// n'en dit aucune, ou n'importe laquelle si l'agent n'en donne pas. Deux relations différentes
/// sont deux affirmations (« inspire » et « contredit » entre les deux mêmes textes) : chacune
/// a sa flèche.
pub fn a_reprendre<'a>(annotations: &'a [Annotation], fleche: &'a Annotation) -> Option<&'a str> {
    let dit = relation(fleche);
    let memes = || memes_bouts(annotations, fleche);
    memes()
        .find(|autre| relation(autre) == dit)
        .or_else(|| memes().find(|autre| dit.is_none() || relation(autre).is_none()))
        .map(Annotation::id)
}

fn relation(fleche: &Annotation) -> Option<ArrowPredicate> {
    match fleche {
        Annotation::Arrow { predicate, .. } => *predicate,
        _ => None,
    }
}

fn memes_bouts<'a>(
    annotations: &'a [Annotation],
    fleche: &'a Annotation,
) -> impl Iterator<Item = &'a Annotation> {
    let ab = bouts(fleche);
    annotations.iter().filter(move |autre| {
        autre.id() != fleche.id()
            && ab.is_some_and(|(a, b)| {
                bouts(autre).is_some_and(|(c, d)| (c, d) == (a, b) || (c, d) == (b, a))
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fleche(id: &str, source: &str, cible: &str) -> Annotation {
        let mut f = Annotation::arrow(id, 0.0, 0.0, 10.0, 0.0);
        if let Annotation::Arrow {
            source_id,
            target_id,
            ..
        } = &mut f
        {
            *source_id = Some(source.into());
            *target_id = Some(cible.into());
        }
        f
    }

    #[test]
    fn test_double_memes_bouts_dans_les_deux_sens() {
        let tableau = vec![fleche("f", "A", "B"), fleche("g", "B", "C")];
        let id = |f: &Annotation| qui_relie_deja(&tableau, f).map(|a| a.id().to_string());
        assert_eq!(id(&fleche("n", "A", "B")).as_deref(), Some("f"));
        assert_eq!(
            id(&fleche("n", "B", "A")).as_deref(),
            Some("f"),
            "l'autre sens"
        );
        assert_eq!(id(&fleche("n", "A", "C")), None, "d'autres bouts");
        assert_eq!(id(&fleche("f", "A", "B")), None, "elle-même");
    }

    #[test]
    fn test_double_un_agent_reprend_la_meme_affirmation_pas_une_autre() {
        let avec = |mut f: Annotation, p: Option<ArrowPredicate>| {
            if let Annotation::Arrow { predicate, .. } = &mut f {
                *predicate = p;
            }
            f
        };
        let contredit = Some(ArrowPredicate::Contredit);
        let inspire = Some(ArrowPredicate::Inspire);
        let tableau = vec![
            avec(fleche("f", "A", "B"), contredit),
            avec(fleche("g", "B", "A"), inspire),
        ];
        let reprise = |p| {
            let nouvelle = avec(fleche("n", "A", "B"), p);
            a_reprendre(&tableau, &nouvelle).map(str::to_string)
        };
        assert_eq!(
            reprise(None).as_deref(),
            Some("f"),
            "sans relation : la première"
        );
        assert_eq!(
            reprise(inspire).as_deref(),
            Some("g"),
            "la même affirmation"
        );
        assert_eq!(reprise(Some(ArrowPredicate::Illustre)), None, "une autre");
    }

    #[test]
    fn test_double_un_autre_passage_ou_un_bout_libre_n_en_double_aucune() {
        let tableau = vec![fleche("f", "A", "B")];
        let mut passage = fleche("n", "A", "B");
        if let Annotation::Arrow {
            source_text_sel, ..
        } = &mut passage
        {
            *source_text_sel = Some(TextSelection::Legacy("un passage".into()));
        }
        assert!(qui_relie_deja(&tableau, &passage).is_none());
        let mut libre = fleche("n", "A", "B");
        if let Annotation::Arrow { target_id, .. } = &mut libre {
            *target_id = None;
        }
        assert!(qui_relie_deja(&tableau, &libre).is_none());
    }
}
