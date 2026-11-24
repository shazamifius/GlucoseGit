//! **La flèche en double** (sa note du 24/11, `text-pxtg-2317` ; fiche 149 § 2) : deux flèches
//! aux mêmes bouts suivent le même itinéraire, et la seconde se cache exactement sous la première.

use crate::types::{Annotation, ArrowPredicate, TextSelection};
use std::collections::HashMap;

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

/// **La hauteur d'une pastille d'étiquette**, en unités du monde : le rendu la dessine de cette
/// hauteur (`arrow_label`, `LABEL_HALF_H` en est la moitié).
pub const HAUTEUR_DE_PASTILLE: f64 = 22.0;

/// **L'écart de deux flèches parallèles**, à leur sommet : deux hauteurs de pastille. Leurs
/// étiquettes, posées au milieu du premier morceau (`milieu_du_trace`, la règle de Tauri), en
/// gardent les trois quarts — plus d'une pastille : elles se lisent l'une au-dessus de l'autre.
pub const ECART_DES_PARALLELES: f64 = 2.0 * HAUTEUR_DE_PASTILLE;

/// **Le rang de chaque flèche parmi ses parallèles** — celles qui relient les mêmes bouts —, et
/// combien elles sont : de quoi les écarter l'une de l'autre au lieu de les cacher l'une sous
/// l'autre (fiche 149 § 2). Calculé une fois par version du document : jamais un balayage du
/// tableau par flèche et par image. Une flèche seule n'y figure pas.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Paralleles(HashMap<String, (usize, usize)>);

impl Paralleles {
    /// Range les flèches de ce tableau, dans l'ordre du tableau : la plus ancienne d'abord.
    pub fn du_tableau(annotations: &[Annotation]) -> Self {
        let mut paires: HashMap<(&str, &str), Vec<&Annotation>> = HashMap::new();
        for fleche in annotations {
            if let Some(((a, ..), (b, ..))) = bouts(fleche) {
                paires.entry((a.min(b), a.max(b))).or_default().push(fleche);
            }
        }
        let mut rangs = HashMap::new();
        for paire in paires.values().filter(|p| p.len() > 1) {
            // Les mêmes nœuds, mais peut-être d'autres passages : chaque tracé a sa classe.
            let mut classes: Vec<Vec<&Annotation>> = Vec::new();
            for &fleche in paire {
                match classes.iter_mut().find(|c| memes(c[0], fleche)) {
                    Some(classe) => classe.push(fleche),
                    None => classes.push(vec![fleche]),
                }
            }
            for classe in classes.iter().filter(|c| c.len() > 1) {
                for (rang, fleche) in classe.iter().enumerate() {
                    rangs.insert(fleche.id().to_string(), (rang, classe.len()));
                }
            }
        }
        Self(rangs)
    }

    /// Range les flèches d'un tableau.
    pub fn du_board(board: &crate::types::Board) -> Self {
        Self::du_tableau(&board.annotations)
    }

    /// Le rang de cette flèche parmi ses parallèles, et leur nombre ; `None` si elle est seule.
    pub fn de(&self, id: &str) -> Option<(usize, usize)> {
        self.0.get(id).copied()
    }
}

/// Deux flèches aux mêmes bouts, dans un sens ou dans l'autre.
fn memes(f: &Annotation, g: &Annotation) -> bool {
    matches!((bouts(f), bouts(g)), (Some((a, b)), Some(cd)) if cd == (a, b) || cd == (b, a))
}

/// **Le point par où passe une flèche droite parmi ses parallèles** : son milieu, écarté de
/// son rang, perpendiculairement à la direction qu'elles partagent — celle qui va du plus petit
/// nœud au plus grand, la même quel que soit le sens de chacune. `None` pour celle du milieu
/// d'un nombre impair : elle reste droite.
pub(super) fn ecartee(
    fleche: &Annotation,
    (rang, nombre): (usize, usize),
    (depart, arrivee): ((f64, f64), (f64, f64)),
) -> Option<(f64, f64)> {
    let decalage = (rang as f64 - (nombre - 1) as f64 / 2.0) * ECART_DES_PARALLELES;
    let ((a, ..), (b, ..)) = bouts(fleche)?;
    let (de, vers) = if a <= b {
        (depart, arrivee)
    } else {
        (arrivee, depart)
    };
    let (dx, dy) = (vers.0 - de.0, vers.1 - de.1);
    let longueur = dx.hypot(dy);
    if decalage == 0.0 || longueur < f64::EPSILON {
        return None;
    }
    let milieu = ((depart.0 + arrivee.0) / 2.0, (depart.1 + arrivee.1) / 2.0);
    Some((
        milieu.0 - dy / longueur * decalage,
        milieu.1 + dx / longueur * decalage,
    ))
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
    fn test_paralleles_rangees_dans_l_ordre_du_tableau_et_ecartees_de_part_et_d_autre() {
        let tableau = vec![
            fleche("f", "A", "B"),
            fleche("seule", "A", "C"),
            fleche("g", "B", "A"),
        ];
        let p = Paralleles::du_tableau(&tableau);
        assert_eq!(
            (p.de("f"), p.de("g"), p.de("seule")),
            (Some((0, 2)), Some((1, 2)), None)
        );
        let (d, a) = ((0.0, 0.0), (100.0, 0.0));
        let f = ecartee(&tableau[0], (0, 2), (d, a)).expect("écartée");
        // « g » va de B à A : la direction commune reste de A à B, donc l'autre côté.
        let g = ecartee(&tableau[2], (1, 2), (a, d)).expect("écartée");
        assert_eq!((f.0, g.0), (50.0, 50.0));
        assert_eq!(
            f.1 - g.1,
            -ECART_DES_PARALLELES,
            "un écart entre leurs milieux"
        );
        assert!(
            ecartee(&tableau[0], (1, 3), (d, a)).is_none(),
            "celle du milieu reste droite"
        );
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
