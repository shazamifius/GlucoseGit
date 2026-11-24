//! **TEXT-FIT-2 — un texte qui grandit pousse sa membrane** (fiche 124).
//!
//! TEXT-FIT-1 fait du texte le plancher de sa carte : quand il demande plus de hauteur — on y écrit,
//! ou une mise en page neuve le coupe autrement —, la carte grandit vers le bas. Sa membrane, elle,
//! restait où elle était. Le 19/11, TABLE-3 a coupé en lignes les tableaux que Claude.ai avait
//! posés dans l'onglet « Textes optimisés » : les textes ont grandi, et sont sortis par le bas de
//! leur membrane — celui du co-working en dépassait déjà de vingt unités.
//!
//! La loi : **le bas d'une membrane suit le bas de ce qui grandit en elle**, avec l'écart du cadre
//! (GROUPER-1) ; et celle qui la contient la suit à son tour, comme MEMB-7 le veut d'une membrane
//! qui grandit. Seul ce qui grandit pousse — un texte qui ne change pas ne touche à rien, même s'il
//! dépasse —, et seul le bas bouge : ni le titre, ni les côtés, ni les voisins (MEMB-7 ne les
//! pousse pas). L'appartenance est celle que le document écrit (`membrane_id`) : un texte qu'on
//! tire hors de sa membrane n'en est plus membre (MEMB-1, au dépôt), il ne l'étire donc jamais.
//!
//! Une loi, deux appelants : le chargement, qui ajuste chaque carte à son texte hors du journal
//! (`ajuster_les_cartes`), et la saisie, dans le geste qu'elle a ouvert ([`Store::pousser_les_membranes`]).

use super::zones::ECART;
use super::Store;
use crate::geometry::Rect;
use crate::types::{Annotation, Board};

/// **Ce que la loi de la fiche 145 lit d'un nœud** : son nom, sa boîte, sa membrane — un nœud, ou un
/// nœud vu à sa place d'origine ([`AOrigine`]), jamais une copie du tableau (une annotation pèse
/// ~512 octets : la cloner pour lire sa boîte coûterait, sur un grand tableau, à chaque saisie).
pub trait Boite {
    fn id(&self) -> &str;
    fn rect(&self) -> Option<Rect>;
    fn membrane_id(&self) -> Option<&str>;
}

impl Boite for Annotation {
    fn id(&self) -> &str {
        Annotation::id(self)
    }
    fn rect(&self) -> Option<Rect> {
        Annotation::rect(self)
    }
    fn membrane_id(&self) -> Option<&str> {
        Annotation::membrane_id(self)
    }
}

/// **Un nœud à sa place d'origine** (fiche 145, la voie sans copie) : la boîte qu'il avait quand
/// l'écriture l'a touché la première fois ; sa boîte d'aujourd'hui s'il n'a pas été touché — il est
/// alors à sa place de départ, ou là où une autre main l'a mis.
pub struct AOrigine<'a> {
    noeud: &'a Annotation,
    cadre: Option<Rect>,
}

impl Boite for AOrigine<'_> {
    fn id(&self) -> &str {
        self.noeud.id()
    }
    fn rect(&self) -> Option<Rect> {
        self.cadre
    }
    fn membrane_id(&self) -> Option<&str> {
        self.noeud.membrane_id()
    }
}

/// Le tableau vu à son départ : chaque nœud touché ramené à son origine.
fn a_l_origine<'a>(
    annotations: &'a [Annotation],
    origines: &[(String, Rect)],
) -> Vec<AOrigine<'a>> {
    (annotations.iter())
        .map(|noeud| {
            let origine = origines.iter().find(|o| o.0 == noeud.id()).map(|o| o.1);
            AOrigine {
                noeud,
                cadre: origine.or_else(|| noeud.rect()),
            }
        })
        .collect()
}

/// **Les membranes que pousse le nœud `id`, son bas descendu à `bas`**, et le bas neuf de chacune :
/// la sienne d'abord, puis chaque membrane qui la contient, jusqu'à la première qui le contient
/// déjà. Le rang est celui de la membrane dans `annotations`.
fn membranes_poussees(annotations: &[Annotation], id: &str, bas: f64) -> Vec<(usize, f64)> {
    let mut poussees: Vec<(usize, f64)> = Vec::new();
    let mut parente = annotations
        .iter()
        .find(|a| a.id() == id)
        .and_then(Annotation::membrane_id);
    let mut bas = bas;
    while let Some(m) = parente {
        let Some(rang) = annotations
            .iter()
            .position(|a| a.id() == m && matches!(a, Annotation::Membrane { .. }))
        else {
            break;
        };
        // Un cycle de membranes ne doit pas exister ; s'il existait, il ne pousserait pas sans fin.
        if poussees.iter().any(|&(r, _)| r == rang) {
            break;
        }
        let Some(cadre) = annotations[rang].rect() else {
            break;
        };
        let voulu = bas + ECART;
        if voulu <= cadre.bottom() {
            break;
        }
        poussees.push((rang, voulu));
        bas = voulu;
        parente = annotations[rang].membrane_id();
    }
    poussees
}

/// **Les voisins que pousse le nœud `id` pendant qu'on l'écrit** (fiche 145 ; sa réponse sur
/// `relais-rep-327`) : `depart` est son tableau au début de l'écriture, `bas` son bas courant. À son
/// niveau (même membrane), ce qui était entièrement dessous et que le bas neuf atteint descend du
/// strict nécessaire — l'écart d'origine gardé, au plus `ECART` —, et pousse à son tour : un
/// balayage par haut croissant (Force Scan, vers le bas seulement), qui garde l'ordre. Une
/// fonction de l'état de départ, jamais une suite de poussées : effacer fait remonter, sans dérive.
/// Rend le rang de chaque voisin qui descend et son haut neuf.
pub fn voisins_pousses<B: Boite>(depart: &[B], id: &str, bas: f64) -> Vec<(usize, f64)> {
    let Some((cadre, niveau)) =
        (depart.iter().find(|a| a.id() == id)).and_then(|a| Some((a.rect()?, a.membrane_id())))
    else {
        return Vec::new();
    };
    let mut dessous: Vec<(usize, Rect)> = (depart.iter().enumerate())
        .filter(|(_, a)| a.id() != id && a.membrane_id() == niveau)
        .filter_map(|(rang, a)| Some((rang, a.rect()?)))
        .collect();
    dessous.sort_by(|a, b| a.1.top.total_cmp(&b.1.top));
    // Chaque pousseur : sa boîte de départ, et son bas neuf.
    let mut pousseurs: Vec<(Rect, f64)> = vec![(cadre, bas)];
    let mut pousses = Vec::new();
    for (rang, c) in dessous {
        let haut = (pousseurs.iter())
            .filter(|(p, _)| p.left < c.right() && c.left < p.right() && p.bottom() <= c.top)
            .map(|(p, bas_neuf)| bas_neuf + ECART.min(c.top - p.bottom()))
            .fold(c.top, f64::max);
        if haut > c.top {
            pousses.push((rang, haut));
        }
        pousseurs.push((c, haut + c.height));
    }
    pousses
}

/// **Ce que pousse le nœud `id` pendant qu'on l'écrit, à chaque étage** (fiche 145) : ses voisins
/// ([`voisins_pousses`]), puis sa membrane, qui grandit jusqu'au plus bas de ce qu'elle contient
/// désormais (+ `ECART`) et pousse à son tour les siens — jusqu'à la première qui le contient déjà.
/// Rend le haut neuf de chaque voisin poussé et le bas neuf de chaque membrane qui grandit, par rang.
pub fn pousser_en_ecrivant<B: Boite>(depart: &[B], id: &str, bas: f64) -> Vec<(usize, Bord, f64)> {
    let mut faits: Vec<(usize, Bord, f64)> = Vec::new();
    let (mut qui, mut bas) = (id.to_string(), bas);
    while faits.len() <= depart.len() {
        let pousses = voisins_pousses(depart, &qui, bas);
        let plus_bas = (pousses.iter())
            .filter_map(|&(r, haut)| Some(haut + depart[r].rect()?.height))
            .fold(bas, f64::max);
        faits.extend(pousses.into_iter().map(|(r, haut)| (r, Bord::Haut, haut)));
        let Some((rang, cadre)) = (depart.iter().find(|a| a.id() == qui))
            .and_then(|a| a.membrane_id())
            .and_then(|m| depart.iter().position(|a| a.id() == m))
            .and_then(|r| Some((r, depart[r].rect()?)))
        else {
            break;
        };
        let voulu = plus_bas + ECART;
        if voulu <= cadre.bottom() {
            break;
        }
        faits.push((rang, Bord::Bas, voulu));
        (qui, bas) = (depart[rang].id().to_string(), voulu);
    }
    faits
}

/// Le bord qu'un geste de [`pousser_en_ecrivant`] déplace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bord {
    /// Le nœud descend, sa taille gardée.
    Haut,
    /// La membrane grandit, son haut gardé.
    Bas,
}

/// Le bas d'une membrane posé à `bas`, son haut gardé.
fn poser_le_bas(membrane: &mut Annotation, bas: f64) {
    if let Annotation::Membrane { y, height, .. } = membrane {
        *height = bas - *y;
    }
}

/// **TEXT-FIT-2, hors du journal** : ce que fait un document qu'on lit, avant qu'il ne devienne un
/// geste — le nœud `id` de ce tableau a descendu son bas à `bas`, ses membranes le suivent.
pub fn pousser_dans_le_tableau(board: &mut Board, id: &str, bas: f64) {
    for (rang, bas) in membranes_poussees(&board.annotations, id, bas) {
        poser_le_bas(&mut board.annotations[rang], bas);
    }
}

impl Store {
    /// **TEXT-FIT-2, par le journal** : le nœud `id` vient de descendre son bas à `bas` — ses
    /// membranes le suivent, dans le geste ouvert, et le même `Ctrl+Z` les défait avec lui.
    pub fn pousser_les_membranes(&mut self, board_id: &str, id: &str, bas: f64) {
        let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) else {
            return;
        };
        let poussees: Vec<(String, Rect)> = membranes_poussees(&b.annotations, id, bas)
            .into_iter()
            .filter_map(|(rang, bas)| {
                let m = &b.annotations[rang];
                let cadre = m.rect()?;
                Some((
                    m.id().to_string(),
                    Rect::new(cadre.left, cadre.top, cadre.width, bas - cadre.top),
                ))
            })
            .collect();
        for (m, cadre) in poussees {
            self.set_annotation_rect(board_id, &m, cadre);
        }
    }
}

impl Store {
    /// **POUSSER-145, par le journal** : le nœud `id` qu'on écrit a son bas à `bas` ; `origines`
    /// garde la boîte d'origine de ce que cette écriture a touché — le nœud écrit dès l'ouverture de
    /// la saisie, chaque voisin et chaque membrane à leur premier déplacement. Chaque voisin prend le
    /// haut que la loi lui donne, en emportant son contenu — ou revient à son origine (effacer fait
    /// remonter), puis est oublié : une autre main peut le bouger. Chaque membrane, son bas. Dans le
    /// geste ouvert : le même `Ctrl+Z` défait la saisie et tout ce qu'elle a poussé.
    pub fn pousser_en_ecrivant(
        &mut self,
        board_id: &str,
        origines: &mut Vec<(String, Rect)>,
        id: &str,
        bas: f64,
    ) {
        let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) else {
            return;
        };
        let depart = a_l_origine(&b.annotations, origines);
        let faits = pousser_en_ecrivant(&depart, id, bas);
        let voulu = |rang: usize, cadre: Rect| {
            let fait = faits.iter().find(|f| f.0 == rang);
            match fait {
                Some(&(_, Bord::Haut, haut)) => {
                    Rect::new(cadre.left, haut, cadre.width, cadre.height)
                }
                Some(&(_, Bord::Bas, b)) => {
                    Rect::new(cadre.left, cadre.top, cadre.width, b - cadre.top)
                }
                None => cadre,
            }
        };
        let touches: Vec<(String, Rect)> = (depart.iter().enumerate())
            .filter(|(_, a)| a.id() != id)
            .filter(|(r, a)| {
                faits.iter().any(|f| f.0 == *r) || origines.iter().any(|o| o.0 == a.id())
            })
            .filter_map(|(r, a)| Some((a.id().to_string(), voulu(r, a.rect()?))))
            .collect();
        // Ce qui revient à son origine est oublié ; ce que la loi touche la première fois, retenu.
        let poussees: Vec<(String, Option<Rect>)> = (faits.iter())
            .map(|f| (depart[f.0].id().to_string(), depart[f.0].rect()))
            .collect();
        origines.retain(|o| o.0 == id || poussees.iter().any(|p| p.0 == o.0));
        for (qui, cadre) in poussees {
            if let Some(c) = cadre.filter(|_| !origines.iter().any(|o| o.0 == qui)) {
                origines.push((qui, c));
            }
        }
        for (qui, cible) in touches {
            let Some(actuel) = (self.project.boards.iter().find(|b| b.id == board_id))
                .and_then(|b| b.annotations.iter().find(|a| a.id() == qui))
                .and_then(Annotation::rect)
            else {
                continue;
            };
            let dy = cible.top - actuel.top;
            if dy != 0.0 {
                let emport = (self.project.boards.iter().find(|b| b.id == board_id))
                    .map(|b| super::membranes::Emport::de(b, &[], std::slice::from_ref(&qui)));
                let t = emport.and_then(|e| {
                    self.translation_de(board_id, &e, &super::choix::Choix::default(), (0.0, dy))
                });
                if let Some(t) = t.filter(|t| t.apply(&mut self.project)) {
                    self.record_as_one_gesture(vec![t]);
                }
            }
            if (cible.height - actuel.height).abs() > 1e-9 {
                let haut = actuel.top + dy;
                self.set_annotation_rect(
                    board_id,
                    &qui,
                    Rect::new(cible.left, haut, cible.width, cible.height),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests;
