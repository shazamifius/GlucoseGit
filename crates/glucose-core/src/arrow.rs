//! La géométrie d'une flèche : le chemin qu'elle suit, et ce qui la désigne sous un curseur.
//!
//! # ARROW-1 — une flèche est une polyligne, et une seule fonction la décrit
//!
//! Elle part de `(x, y)`, passe par ses `waypoints` dans l'ordre, et finit en `(x2, y2)`.
//! Tout ce qui a besoin de la suivre — le test de clic, le dessin, le déplacement d'une
//! extrémité, l'étiquette posée au milieu — lit cette même suite de points. C'est la leçon
//! que la rotation a coûtée : deux endroits qui recalculent la même forme finissent par en
//! dessiner deux différentes.
//!
//! # Pourquoi une flèche n'était pas cliquable
//!
//! Glucose Tauri désigne la flèche sous le curseur par le **DOM** : une bande invisible de
//! 24 pixels d'épaisseur la double, et le navigateur dit laquelle a été touchée
//! (`ArrowSvgLayer.tsx`). L'arbitre de clic reçoit ce nom tout fait, par un champ nommé
//! `arrow_id`.
//!
//! Le portage a gardé le champ mais pas le navigateur : personne ne le remplissait, et
//! **aucune flèche n'était donc jamais sélectionnable**. Elle se voyait, elle ne se prenait
//! pas. Ici, c'est la géométrie qui répond, et le champ disparaît avec le trou.
//!
//! # ARROW-2 — une flèche s'accroche **au dessin**, pas après coup
//!
//! Glucose Tauri ne donne aucune poignée pour déplacer le bout d'une flèche : les deux
//! disques qu'il pose aux extrémités d'une flèche sélectionnée sont décoratifs, sans
//! aucun gestionnaire. Ce qui relie une flèche à un nœud, c'est **l'aimantation pendant
//! le tracé** (`snapToNearest`, `GlucoseCanvas.tsx`) : à l'appui pour l'origine, en continu
//! pendant le glisser pour la cible.
//!
//! C'est la différence entre une flèche qui **relie** et une flèche qui **flotte**, et
//! c'est ce qui manquait : DRAW-1 posait une flèche, mais elle ne s'accrochait à rien.
//!
//! # ARROW-3 — le coude se pose, se déplace et se retire au même endroit
//!
//! Une flèche sélectionnée montre deux sortes de poignées (`ArrowSvgLayer.tsx`) : un
//! disque sur chaque **coude** existant, qu'on glisse pour le déplacer et qu'un double-clic
//! retire, et un losange au **milieu de chaque tronçon**, dont l'appui insère un coude à
//! cet endroit. Poser, déplacer et retirer sont donc trois gestes sur un même objet.
//!
//! Les deux sortes vivent dans la même liste et se désignent par la même fonction, pour la
//! raison d'ARROW-1 : deux fonctions qui placent des poignées finiraient par les placer
//! ailleurs l'une que l'autre, et le clic manquerait ce que l'œil voit.

pub mod aspect;
pub mod champ;
pub mod trace;

use crate::arrow_anchor::{arrow_endpoints, ArrowAnchor};
use crate::geometry::{distance_to_segment, Rect};
use crate::quadtree::{noeud_au_rang, SpatialHash};
use crate::types::{TextAnchor, TextSelection};

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
}
use crate::types::Annotation;
use crate::types::{Board, CanvasFolder};

/// Épaisseur de la bande qui désigne une flèche, en pixels **écran**.
///
/// C'est la valeur de Glucose Tauri (`strokeWidth={24}` sur un tracé transparent, en
/// `non-scaling-stroke`, donc constante à tout zoom). Un trait de deux pixels ne se vise pas :
/// c'est la bande qui se vise, et elle vaut la même chose qu'une poignée.
pub const BAND_PX: f64 = 24.0;

/// Les points par lesquels une flèche passe, dans l'ordre.
///
/// Rend `None` pour tout ce qui n'est pas une flèche, plutôt qu'une suite vide : « ce nœud
/// n'est pas une flèche » et « cette flèche n'a pas de point » ne sont pas la même chose.
pub fn path(arrow: &Annotation) -> Option<Vec<(f64, f64)>> {
    let Annotation::Arrow {
        x,
        y,
        x2,
        y2,
        waypoints,
        ..
    } = arrow
    else {
        return None;
    };
    let mut points = Vec::with_capacity(waypoints.len() + 2);
    points.push((*x, *y));
    points.extend(waypoints.iter().map(|p| (p.x, p.y)));
    points.push((*x2, *y2));
    Some(points)
}

/// L'ancre d'une extrémité : la boîte du nœud qu'elle désigne, ou le point brut.
///
/// Une flèche qui vise un nœud ne vise pas son centre : elle s'arrête sur son **bord**, du
/// côté d'où elle vient. C'est ce que [`crate::arrow_anchor`] sait faire depuis toujours —
/// cent soixante-dix-sept lignes écrites, testées, et que personne n'appelait.
///
/// Ancrée à un passage que le nœud sait mesurer, elle vise ce passage : même boîte, mais un
/// point de visée à sa hauteur — elle sort donc du bord qui fait face, à cette hauteur.
fn anchor_of(
    noeuds: &impl Noeuds,
    (id, passage): (Option<&String>, Option<&TextSelection>),
    fallback: (f64, f64),
) -> ArrowAnchor {
    let Some((id, rect)) = id.and_then(|id| Some((id, noeuds.boite(id)?))) else {
        return ArrowAnchor::point(fallback.0, fallback.1);
    };
    let centre = rect.center();
    let ancres = crate::text_anchors::normalize_text_sel(passage);
    let y = (!ancres.is_empty())
        .then(|| noeuds.hauteur_du_passage(id, &ancres))
        .flatten()
        .map_or(centre.y, |h| rect.top + h);
    ArrowAnchor::with_box(
        centre.x,
        y,
        rect.left,
        rect.right(),
        rect.top,
        rect.bottom(),
    )
}

/// La boîte d'un nœud du tableau, quelle que soit sa nature.
pub fn node_rect(board: &Board, id: &str) -> Option<Rect> {
    if let Some(img) = board.images.iter().find(|i| i.id == id) {
        return Some(img.rect());
    }
    if let Some(ann) = board.annotations.iter().find(|a| a.id() == id) {
        return ann.rect();
    }
    board
        .folders
        .iter()
        .find(|f| f.id == id)
        .map(CanvasFolder::rect)
}

/// **La boîte d'un nœud, retrouvée par l'index spatial** — en temps constant, là où
/// [`node_rect`] parcourt le tableau.
///
/// L'identifiant trouvé au rang est vérifié : un index qui aurait une passe de retard ne fait
/// pas viser le mauvais nœud, il fait seulement chercher par le tableau.
pub fn node_rect_indexe(board: &Board, index: &SpatialHash, id: &str) -> Option<Rect> {
    if let Some(noeud) = index.rang_de(id).and_then(|r| noeud_au_rang(board, r)) {
        if noeud.id() == id {
            return noeud.rect();
        }
    }
    node_rect(board, id)
}

/// Les points par lesquels une flèche passe **une fois ancrée** à ce qu'elle relie.
///
/// C'est la même suite que [`path`], sauf que les deux bouts sont ramenés sur le périmètre
/// des nœuds visés. Le brancher **ici** le propage partout d'un coup : le test de clic, la
/// sélection élastique et le dessin lisent tous cette fonction, donc aucun d'eux ne peut
/// voir une flèche ailleurs que là où elle est (ARROW-1).
pub fn path_with(arrow: &Annotation, noeuds: impl Noeuds) -> Option<Vec<(f64, f64)>> {
    let Annotation::Arrow {
        x,
        y,
        x2,
        y2,
        waypoints,
        source_id,
        target_id,
        source_text_sel,
        target_text_sel,
        ..
    } = arrow
    else {
        return None;
    };
    if source_id.is_none() && target_id.is_none() {
        return path(arrow);
    }
    let depart = anchor_of(
        &noeuds,
        (source_id.as_ref(), source_text_sel.as_ref()),
        (*x, *y),
    );
    let arrivee = anchor_of(
        &noeuds,
        (target_id.as_ref(), target_text_sel.as_ref()),
        (*x2, *y2),
    );
    // `Point2D` est le point du **document**, `Point` celui de la géométrie : deux types
    // pour une même notion, dont la fusion dépasse ce chantier. La conversion est ici, à
    // l'unique frontière où les deux se rencontrent.
    let etapes: Vec<crate::geometry::Point> = waypoints
        .iter()
        .map(|p| crate::geometry::Point::new(p.x, p.y))
        .collect();
    let bouts = arrow_endpoints(depart, arrivee, &etapes);
    let mut points = Vec::with_capacity(waypoints.len() + 2);
    points.push((bouts.start.x, bouts.start.y));
    points.extend(waypoints.iter().map(|p| (p.x, p.y)));
    points.push((bouts.end.x, bouts.end.y));
    Some(points)
}

/// La distance d'un point du monde à une flèche, ou `None` si ce n'en est pas une.
///
/// C'est la plus courte distance à l'un de ses segments : une polyligne n'est pas plus loin
/// que son tronçon le plus proche.
pub fn distance_to(arrow: &Annotation, point: (f64, f64)) -> Option<f64> {
    distance_along(&path(arrow)?, point)
}

/// La même distance, à une flèche **ancrée** et telle qu'elle se dessine — courbe comprise :
/// c'est là où on la voit qu'on la vise (FLECHE-1).
///
/// `tolerance` est la précision, en unités du monde, avec laquelle une courbe se remplace
/// par une ligne brisée pour la mesurer ; un segment se mesure exactement.
pub fn distance_to_with(
    arrow: &Annotation,
    noeuds: impl Noeuds,
    point: (f64, f64),
    tolerance: f64,
) -> Option<f64> {
    let brisee = trace::aplatir(&morceaux_with(arrow, noeuds)?, tolerance);
    distance_along(&brisee, point)
}

/// Une flèche est-elle courbe ? Le champ de Tauri, `arrowType: "curved"`.
pub fn est_courbe(arrow: &Annotation) -> bool {
    matches!(arrow, Annotation::Arrow { arrow_type: Some(t), .. } if t == "curved")
}

/// **Les morceaux qu'une flèche ancrée dessine** — la description que le dessin, le clic,
/// les poignées et l'étiquette lisent tous (FLECHE-1, loi L4).
pub fn morceaux_with(arrow: &Annotation, noeuds: impl Noeuds) -> Option<Vec<trace::Morceau>> {
    Some(trace::morceaux(
        &path_with(arrow, noeuds)?,
        est_courbe(arrow),
    ))
}

/// Les mêmes morceaux, pour une flèche de ce tableau.
pub fn morceaux_in(arrow: &Annotation, board: &Board) -> Option<Vec<trace::Morceau>> {
    morceaux_with(arrow, |id: &str| node_rect(board, id))
}

/// La plus courte distance d'un point à une polyligne.
fn distance_along(points: &[(f64, f64)], point: (f64, f64)) -> Option<f64> {
    points
        .windows(2)
        .map(|seg| distance_to_segment(point, seg[0], seg[1]))
        .fold(None, |acc: Option<f64>, d| {
            Some(acc.map_or(d, |m| m.min(d)))
        })
}

/// La flèche la plus proche de `point`, si l'une d'elles est à portée de la bande.
///
/// À égalité, la **dernière** l'emporte : c'est celle qui est dessinée au-dessus, donc celle
/// que la main croit viser. Le même arbitrage que pour les nœuds empilés.
pub fn at(
    annotations: &[Annotation],
    noeuds: impl Noeuds + Copy,
    point: (f64, f64),
    scale: f64,
) -> Option<(&Annotation, f64)> {
    // La bande garde une épaisseur **écran** : une flèche ne devient pas plus dure à viser
    // parce qu'on s'est éloigné. C'est la même exception que les poignées (SCALE-1).
    let portee = BAND_PX / 2.0 / scale.max(1e-6);
    // Un quart de pixel : la ligne brisée qui remplace une courbe ne se distingue pas d'elle
    // à l'écran, donc on vise exactement ce qu'on voit.
    let tolerance = 0.25 / scale.max(1e-6);
    annotations
        .iter()
        .filter_map(|a| distance_to_with(a, noeuds, point, tolerance).map(|d| (a, d)))
        .filter(|(_, d)| *d <= portee)
        .reduce(|meilleur, courant| {
            if courant.1 <= meilleur.1 {
                courant
            } else {
                meilleur
            }
        })
}

/// La distance, en unités **monde**, à laquelle un bout de flèche s'aimante à un nœud.
///
/// C'est le `HIT_DIST` de Glucose Tauri, et c'est bien une longueur monde et non écran :
/// l'aimantation décrit un voisinage **du document** — « assez près de cette carte pour
/// vouloir la relier » — et non une tolérance de main comme la bande de sélection.
pub const SNAP_DIST: f64 = 120.0;

/// Où un bout de flèche se pose, et à quoi il s'accroche.
#[derive(Debug, Clone, PartialEq)]
pub struct Snap {
    /// Le point retenu : le curseur lui-même, ou le point du bord le plus proche.
    pub point: (f64, f64),
    /// Le nœud auquel ce bout s'accroche, s'il y en a un assez près.
    pub node: Option<String>,
}

impl Snap {
    /// Un bout qui ne s'accroche à rien : le curseur, tel quel.
    pub fn free(point: (f64, f64)) -> Self {
        Self { point, node: None }
    }
}

/// Le point d'une boîte le plus proche de `point` — le point lui-même s'il est dedans.
fn closest_on(rect: Rect, (x, y): (f64, f64)) -> (f64, f64) {
    (
        x.clamp(rect.left, rect.right()),
        y.clamp(rect.top, rect.bottom()),
    )
}

/// Le nœud auquel un bout de flèche s'aimante à cet endroit, et où il s'y pose (ARROW-2).
///
/// Chaque nœud est jugé sur la distance au **point de sa boîte le plus proche**, pas à son
/// centre : viser le bord d'une grande carte l'accroche, alors qu'une distance au centre
/// ferait préférer une petite carte lointaine. À l'intérieur d'une boîte, la distance est
/// nulle et le point retenu est le curseur — on pointe où l'on veut dans la carte visée.
///
/// `exclude` écarte les identifiants qui ne doivent pas être visés : la flèche en cours de
/// tracé, et le nœud dont elle part — sans quoi elle se refermerait sur son origine dès le
/// premier pixel de glisser.
pub fn snap_to_nearest(board: &Board, point: (f64, f64), exclude: &[&str]) -> Snap {
    let mut best = Snap::free(point);
    let mut best_dist = SNAP_DIST;

    let boxes = board
        .images
        .iter()
        .map(|img| (img.id.as_str(), img.rect()))
        .chain(
            board
                .annotations
                .iter()
                .filter_map(|a| Some((a.id(), a.rect()?))),
        )
        .chain(board.folders.iter().map(|f| (f.id.as_str(), f.rect())));

    for (id, rect) in boxes {
        if exclude.contains(&id) {
            continue;
        }
        let edge = closest_on(rect, point);
        let distance = (edge.0 - point.0).hypot(edge.1 - point.1);
        if distance < best_dist {
            best_dist = distance;
            best = Snap {
                point: edge,
                node: Some(id.to_string()),
            };
        }
    }
    best
}

/// Où la **pointe** d'une flèche en cours de tracé se pose, et à quoi elle s'accroche.
///
/// C'est [`snap_to_nearest`] avec les seules exclusions qui aient un sens pendant un tracé :
/// la flèche elle-même, et le nœud dont elle part. Les déduire ici plutôt que de les faire
/// assembler par l'appelant lui évite de lire le modèle pour savoir ce qu'il doit écarter —
/// et surtout évite que deux appelants n'en écartent pas les mêmes.
pub fn snap_for_tip(board: &Board, arrow_id: &str, point: (f64, f64)) -> Snap {
    let source = board.annotations.iter().find_map(|a| match a {
        Annotation::Arrow { id, source_id, .. } if id == arrow_id => source_id.as_deref(),
        _ => None,
    });
    let mut exclude = vec![arrow_id];
    exclude.extend(source);
    snap_to_nearest(board, point, &exclude)
}

/// Rayon d'une poignée de flèche, en pixels **écran** (Glucose Tauri : `6 / vpScale`).
pub const HANDLE_RADIUS_PX: f64 = 6.0;

/// Rayon de **saisie** d'une poignée, en pixels écran (Glucose Tauri : `16 / vpScale`).
///
/// Plus large que ce qu'on voit, comme toute affordance : on vise un disque de six pixels,
/// on l'attrape à seize. C'est la même exception à SCALE-1 que la bande d'ARROW-1.
pub const HANDLE_GRAB_PX: f64 = 16.0;

/// Ce qu'une poignée de flèche désigne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleKind {
    /// Le coude d'index donné : on le glisse pour le déplacer, un double-clic le retire.
    Bend(usize),
    /// Le milieu du tronçon d'index donné : un appui y **insère** un coude.
    Midpoint(usize),
}

/// Une poignée de flèche : ce qu'elle désigne, et où elle se trouve dans le monde.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowHandle {
    pub kind: HandleKind,
    pub at: (f64, f64),
}

/// Les poignées d'une flèche, dans l'ordre où elles se dessinent.
///
/// Les milieux d'abord, les coudes ensuite : un coude se dessine **par-dessus** le milieu
/// du tronçon qu'il vient de couper, et c'est aussi l'ordre dans lequel on veut les
/// attraper — voir [`handle_at`].
pub fn handles(arrow: &Annotation, noeuds: impl Noeuds) -> Vec<ArrowHandle> {
    let Some(points) = path_with(arrow, noeuds) else {
        return Vec::new();
    };
    // Le milieu d'un tronçon est pris **sur le tracé** : sur une flèche courbe, le milieu de
    // la corde tomberait à côté du trait, et la poignée flotterait dans le vide.
    let troncons = trace::morceaux(&points, est_courbe(arrow));
    let milieux = troncons.iter().enumerate().map(|(index, m)| ArrowHandle {
        kind: HandleKind::Midpoint(index),
        at: m.milieu(),
    });
    // Les coudes sont les points **intérieurs** du chemin : les deux bouts n'en sont pas,
    // et Glucose Tauri ne leur donne aucune poignée non plus.
    let coudes = points
        .iter()
        .enumerate()
        .skip(1)
        .take(points.len().saturating_sub(2))
        .map(|(index, at)| ArrowHandle {
            kind: HandleKind::Bend(index - 1),
            at: *at,
        });
    milieux.chain(coudes).collect()
}

/// La poignée sous ce point, si l'une d'elles est à portée de saisie (ARROW-3).
///
/// Un **coude** l'emporte sur un milieu à égalité de distance, et il l'emporte même de
/// justesse : c'est celui qui se dessine au-dessus, et surtout c'est le geste qu'on refait
/// le plus. Sans cette préférence, redéplacer un coude qu'on vient de poser au milieu d'un
/// tronçon insérerait un second coude par-dessus le premier.
pub fn handle_at(
    arrow: &Annotation,
    noeuds: impl Noeuds,
    point: (f64, f64),
    scale: f64,
) -> Option<ArrowHandle> {
    let portee = HANDLE_GRAB_PX / scale.max(1e-6);
    handles(arrow, noeuds)
        .into_iter()
        .map(|h| {
            let distance = (h.at.0 - point.0).hypot(h.at.1 - point.1);
            (h, distance)
        })
        .filter(|(_, d)| *d <= portee)
        // Un rang explicite plutôt qu'une condition composée : « le coude gagne, puis le
        // plus proche » se lit, alors qu'un `si préféré ou plus proche` laissait un milieu
        // très proche battre un coude — ce qui aurait inséré un coude par-dessus un coude.
        .min_by(|(a, da), (b, db)| {
            let rang = |h: &ArrowHandle| u8::from(matches!(h.kind, HandleKind::Midpoint(_)));
            rang(a).cmp(&rang(b)).then(da.total_cmp(db))
        })
        .map(|(h, _)| h)
}

/// Le point du chemin où une flèche porte ce qu'elle dit — étiquette et prédicat.
///
/// Le **milieu du tronçon médian**, et non le milieu de la corde : sur une flèche coudée,
/// le milieu de la corde tombe souvent à côté du trait, parfois très loin, et l'étiquette
/// s'y détacherait de ce qu'elle nomme. Glucose Tauri place le sien exactement là
/// (`ArrowSvgLayer.tsx`, `midSeg = Math.floor(n / 2)`).
///
/// Sur une flèche courbe, c'est le milieu du morceau médian **sur la courbe** (FLECHE-1).
pub fn label_anchor(arrow: &Annotation, noeuds: impl Noeuds) -> Option<(f64, f64)> {
    milieu_du_trace(&morceaux_with(arrow, noeuds)?)
}

/// Le point où une flèche porte ce qu'elle dit, lu sur ses morceaux déjà calculés.
pub fn milieu_du_trace(morceaux: &[trace::Morceau]) -> Option<(f64, f64)> {
    // `n` points font `n − 1` morceaux ; le tronçon médian de Tauri, entre les points
    // `⌊n/2⌋ − 1` et `⌊n/2⌋`, est le morceau `⌊n/2⌋ − 1`.
    let n = morceaux.len() + 1;
    Some(morceaux.get((n / 2).saturating_sub(1))?.milieu())
}

/// Le même point, pour une flèche de ce tableau.
pub fn label_anchor_in(arrow: &Annotation, board: &Board) -> Option<(f64, f64)> {
    label_anchor(arrow, |id: &str| node_rect(board, id))
}

/// Le chemin d'une flèche ancrée dans son tableau — le raccourci courant de [`path_with`].
pub fn path_in(arrow: &Annotation, board: &Board) -> Option<Vec<(f64, f64)>> {
    path_with(arrow, |id: &str| node_rect(board, id))
}

#[cfg(test)]
mod tests;
