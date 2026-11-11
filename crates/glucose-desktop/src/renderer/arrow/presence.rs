//! **LIEN-4 — la présence d'un lien : leur nombre à l'écran, et la souris** (fiche 102).
//!
//! Sa parole du 10/11 au soir, devant la carte de Wikipédia lue de près : *« s'il y a une abondance
//! de liens au sein d'un espace, même zoomé, alors en fonction du placement de la souris — si elle
//! se rapproche d'un nœud possédant des liens… »*, et ses deux formules :
//!
//! * **la densité** : un lien seul à l'écran est plein ; vingt sont à 30 % — sa droite,
//!   `100 − 70 (n − 1) / 19` ;
//! * **la proximité** : `100 / (1 + d)`, d la distance de la souris au nœud qui porte le lien ;
//! * *« je n'aimerais jamais tomber en dessous de 8 % »* — un lien ne disparaît jamais.
//!
//! Ses nombres sont les siens : des nombres de ressenti, dits par lui, et gardés tels quels.
//!
//! # Ce que la session y a mis, à contredire
//!
//! * **L'unité de la distance** : en pixels, sa formule tomberait à moitié à un pixel du nœud. La
//!   distance se compte **en tailles du nœud** — le côté du carré de même aire, `√(l × h)`, la loi
//!   même qui fait grandir le titre d'une membrane (TITRE-1) : à une taille de nœud de distance,
//!   ses liens sont à moitié. Elle ne dépend ni du zoom ni de l'écran ; la souris dans le nœud, 0.
//! * **Les blocs, pas les membranes** (fiche 103 § 3) : la souris rapproche un lien des **blocs**
//!   qu'il relie — une carte, une note, une photo, un dossier, ce qu'une flèche contourne. Une
//!   membrane est une région qu'on habite : la première loi comptait sa boîte, et sur la carte de
//!   Wikipédia, dont les liens arrivent sur des articles de mille unités, la souris était partout
//!   « tout près » — chaque lien vif, une pelote sur le texte, regardée au pilote. Un lien sans
//!   bloc à ses bouts (une flèche libre, ou entre deux membranes) se rapproche par lui-même.
//! * **Les deux formules ensemble** : un lien prend **la plus forte** — ce que la densité lui laisse,
//!   ou ce que la souris lui rend —, puis LIEN-1 (de loin, il pâlit avec la taille réelle), puis le
//!   plancher. Ce qu'on regarde (choisi, survolé, la fenêtre des liens) reste plein.

use super::NoeudsDuRendu;
use glucose_core::geometry::Rect;
use glucose_core::quadtree::Visibles;
use glucose_core::types::{Annotation, Board};

/// **Ce que la main désigne** pour les liens : le nœud sous la souris (LIEN-1), et la souris
/// elle-même, en coordonnées du monde (LIEN-4) — rien au doigt, ni hors du canevas.
#[derive(Clone, Copy, Default)]
pub(crate) struct Main<'a> {
    pub survole: Option<&'a str>,
    pub souris: Option<(f64, f64)>,
}

impl<'a> Main<'a> {
    /// Ce que la scène dit de la main.
    pub(crate) fn de(overlay: &crate::params::SceneOverlay<'a>) -> Self {
        Self {
            survole: overlay.survole,
            souris: overlay.souris,
        }
    }
}

/// *« Je n'aimerais jamais tomber en dessous de 8 % »* (sa parole du 10/11).
pub(crate) const PLANCHER: f32 = 0.08;

/// **Un niveau d'écran** : la plus petite différence d'opacité qu'un écran à huit bits par canal
/// sache montrer. Un pixel vaut le fond plus `(couleur − fond) × opacité`, et l'écart de deux
/// couleurs ne dépasse pas 255 niveaux : deux opacités plus proches qu'un 255ᵉ ne changent aucun
/// pixel de plus d'un arrondi. Le seuil n'est pas choisi : c'est l'écran.
pub(crate) const UN_NIVEAU: f32 = 1.0 / 255.0;

/// **L'opacité du trait d'un lien** (LIEN-1, LIEN-4) : pleine quand on le regarde (`en_lumiere`) ;
/// sinon sa présence, pâlie avec la taille réelle (`lecture`), jamais sous le plancher. Une seule
/// loi, que le dessin et la main lisent.
pub(crate) fn opacite(en_lumiere: bool, presence: f32, lecture: f32) -> f32 {
    if en_lumiere {
        1.0
    } else {
        (lecture * presence).max(PLANCHER)
    }
}

/// **La présence qui compte** (SURVOL-2, fiche 106) : celle du lien, ou 1 sans la calculer quand
/// aucune présence ne peut changer son trait.
///
/// Une présence ne dépasse jamais 1 (la densité et la proximité y sont bornées). Plus pâle que le
/// plancher à pleine présence — `lecture ≤ PLANCHER`, la carte vue de loin —, un lien hors lumière
/// reste donc **au plancher** où que soit la souris : [`opacite`] y rend le plancher pour toute
/// présence, celle-ci comprise. La calculer demandait les boîtes des deux nœuds de chaque lien, à
/// chaque image et à chaque mouvement de la souris : sur la carte de Wikipédia vue de loin, mille
/// cent vingt-huit liens, pour une opacité qui ne bougeait pas.
pub(crate) fn presence_qui_compte(
    ann: &Annotation,
    noeuds: NoeudsDuRendu<'_>,
    souris: Option<(f64, f64)>,
    (n, lecture): (usize, f32),
) -> f32 {
    if lecture <= PLANCHER {
        1.0
    } else {
        presence(ann, noeuds, souris, n)
    }
}

/// **La main, passant d'`avant` à `apres`, change-t-elle l'un de ces liens à l'écran ?** Le nœud
/// qu'elle survole les allume, la souris les rapproche. Au moins d'un niveau ([`UN_NIVEAU`]) : en
/// deçà, l'image refaite serait la même, et la refaire coûterait une scène entière pour rien
/// (fiche 103 § 3 : la souris seule faisait redessiner toute la carte de Wikipédia trente-sept
/// fois par seconde). `n` est le nombre de liens à l'écran, `lecture` le pâlissement de LIEN-1.
pub(crate) fn la_main_change_un_lien<'a>(
    store: &glucose_core::store::Store,
    liens: impl Iterator<Item = &'a Annotation>,
    noeuds: NoeudsDuRendu<'_>,
    [avant, apres]: [Main<'_>; 2],
    (n, lecture): (usize, f32),
) -> bool {
    let vue = |a: &Annotation, main: Main<'_>| {
        let lumiere = super::en_lumiere(store, main.survole, a);
        opacite(
            lumiere,
            presence_qui_compte(a, noeuds, main.souris, (n, lecture)),
            lecture,
        )
    };
    liens
        .filter(|a| matches!(a, Annotation::Arrow { .. }))
        .any(|a| (vue(a, avant) - vue(a, apres)).abs() >= UN_NIVEAU)
}
/// Sa droite : combien un lien de plus à l'écran retire, jusqu'au vingtième (30 %).
const PENTE: f32 = 0.70 / 19.0;

/// **La densité** : la part qu'un lien garde quand `n` liens sont à l'écran.
pub(crate) fn densite(n: usize) -> f32 {
    (1.0 - PENTE * (n.max(1) - 1) as f32).clamp(0.0, 1.0)
}

/// **La proximité de la souris à un nœud** : `1 / (1 + d / taille)`, d sa distance à la boîte
/// (nulle dedans), la taille le côté du carré de même aire.
pub(crate) fn proximite(souris: (f64, f64), boite: Rect) -> f32 {
    let dx = (boite.left - souris.0)
        .max(souris.0 - boite.right())
        .max(0.0);
    let dy = (boite.top - souris.1)
        .max(souris.1 - boite.bottom())
        .max(0.0);
    // Le côté du carré de même aire ; un trait sans aire (une flèche libre, droite) a sa longueur.
    let aire = (boite.width * boite.height).abs();
    let taille = if aire > 0.0 {
        aire.sqrt()
    } else {
        boite.width.abs().max(boite.height.abs())
    };
    if taille <= 0.0 || !taille.is_finite() {
        return 0.0;
    }
    (1.0 / (1.0 + dx.hypot(dy) / taille)) as f32
}

/// **Combien de liens cette image montre** : les flèches parmi ce que la requête a retenu.
pub(crate) fn liens_a_l_ecran(visibles: &[u32], board: &Board) -> usize {
    Visibles::nouvelles(visibles, board)
        .annotations()
        .filter(|a| matches!(a, Annotation::Arrow { .. }))
        .count()
}

/// **La présence d'un lien** : la plus forte de la densité et de la proximité de la souris à l'un
/// des blocs qu'il relie — à la flèche elle-même, s'il n'en relie aucun.
pub(crate) fn presence(
    ann: &Annotation,
    noeuds: NoeudsDuRendu<'_>,
    souris: Option<(f64, f64)>,
    n: usize,
) -> f32 {
    let Annotation::Arrow {
        source_id,
        target_id,
        ..
    } = ann
    else {
        return 1.0;
    };
    let Some(souris) = souris else {
        return densite(n);
    };
    let mut boites: Vec<Rect> = [source_id, target_id]
        .into_iter()
        .flatten()
        .filter_map(|id| noeuds.bloc(id))
        .collect();
    if boites.is_empty() {
        boites.push(ann.encre());
    }
    boites
        .into_iter()
        .map(|b| proximite(souris, b))
        .fold(densite(n), f32::max)
}
