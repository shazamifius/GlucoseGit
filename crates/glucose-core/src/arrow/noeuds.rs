//! **Ce qu'une flèche demande à ce qu'elle relie** : la boîte d'un nœud, la hauteur d'un
//! passage (FLECHE-4), les obstacles d'une zone et la mémoire de ses itinéraires (FLECHE-5).

use crate::geometry::Rect;
use crate::types::TextAnchor;

/// **Ce qu'une flèche demande à ce qu'elle relie** (FLECHE-4) : la boîte d'un nœud — et, si
/// l'on sait mesurer son texte, la hauteur d'un passage dans ce nœud.
///
/// Une flèche ancrée à un passage précis d'une carte part de ce passage, à sa hauteur, et non
/// du milieu de la carte (Tauri : `findTextSelPosition`). Mesurer un texte demande sa mise en
/// page, qui vit avec les polices, hors du noyau : le noyau la **demande**, par ce trait, et
/// le dessin comme le clic passent par le même — un clic ne vise jamais une flèche ailleurs
/// que là où elle est dessinée (loi L4).
///
/// Une simple fonction `id → boîte` en est un : elle ne sait mesurer aucun texte, et une flèche
/// ancrée à un passage part alors du milieu, comme avant.
pub trait Noeuds {
    /// La boîte du nœud, s'il existe.
    fn boite(&self, id: &str) -> Option<Rect>;

    /// La hauteur, depuis le haut du nœud, du passage que ces ancres désignent — `None` si on
    /// ne sait pas la mesurer, ou si le passage a disparu.
    fn hauteur_du_passage(&self, _id: &str, _ancres: &[TextAnchor]) -> Option<f64> {
        None
    }

    /// **Les obstacles qu'une flèche rencontre dans cette zone** (FLECHE-5) : la boîte de
    /// chaque nœud qui la touche et qu'une flèche contourne — ni une flèche, ni une membrane
    /// (une flèche en sort pour relier deux domaines). La source et la cible n'ont pas à être
    /// écartées : une flèche part de l'intérieur de sa source et arrive dans sa cible, et un
    /// obstacle qui contient un bout ne se contourne pas. Aucun par défaut : sans index pour
    /// les trouver, une flèche va droit.
    fn obstacles(&self, _zone: Rect, _sortie: &mut Vec<Rect>) {}

    /// **Un itinéraire retenu d'un appel à l'autre** : `calcul` ne se fait que si cet
    /// itinéraire n'est pas déjà connu. Le dessin, le clic, les poignées et l'étiquette lisent
    /// tous le tracé d'une flèche ; il n'a pas à se chercher quatre fois par image. Sans mémoire
    /// par défaut.
    fn itineraire(
        &self,
        _cle: &Itineraire<'_>,
        calcul: &mut dyn FnMut() -> Vec<(f64, f64)>,
    ) -> Vec<(f64, f64)> {
        calcul()
    }
}

/// **Ce dont l'itinéraire d'une flèche dépend**, hors du reste du tableau : la flèche, ses deux
/// bouts, et si elle est courbe. Le reste — les obstacles — ne change qu'avec le document, et
/// celui qui retient les itinéraires les oublie alors tous.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Itineraire<'a> {
    pub fleche: &'a str,
    pub depart: Bout,
    pub arrivee: Bout,
    pub courbe: bool,
}

impl<F: Fn(&str) -> Option<Rect>> Noeuds for F {
    fn boite(&self, id: &str) -> Option<Rect> {
        self(id)
    }
}

/// Ce que le débogage montre d'un interlocuteur : son rôle, pas son contenu — qui peut être
/// tout un tableau.
impl std::fmt::Debug for dyn Noeuds + '_ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Noeuds")
    }
}

impl Noeuds for &dyn Noeuds {
    fn boite(&self, id: &str) -> Option<Rect> {
        (**self).boite(id)
    }

    fn hauteur_du_passage(&self, id: &str, ancres: &[TextAnchor]) -> Option<f64> {
        (**self).hauteur_du_passage(id, ancres)
    }

    fn obstacles(&self, zone: Rect, sortie: &mut Vec<Rect>) {
        (**self).obstacles(zone, sortie);
    }

    fn itineraire(
        &self,
        cle: &Itineraire<'_>,
        calcul: &mut dyn FnMut() -> Vec<(f64, f64)>,
    ) -> Vec<(f64, f64)> {
        (**self).itineraire(cle, calcul)
    }
}

/// Un bout de flèche : le point qu'elle vise, et la boîte du nœud qui le porte. L'itinéraire ne
/// dépend que du point ; la courbe qui sort de la boîte, resserrée sur lui, dépend aussi de la
/// boîte.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bout {
    pub point: (f64, f64),
    pub boite: Option<Rect>,
}
