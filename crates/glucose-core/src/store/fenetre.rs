//! **LIEN-3 — la fenêtre des liens** (fiche 102) : le « show » de sa parole du 9/11, le Bring & Go
//! de Moscovich et al. (CHI 2009).
//!
//! *« Une fenêtre temporaire qui rassemble le nœud et tous ses liens, les blocs de texte placés à
//! côté, automatiquement, même s'ils sont à des kilomètres, gardant leur couleur d'origine, avec
//! l'explication de leur provenance (quelle membrane) ; on y zoome, on y lit ; un clic sur un nœud
//! défait la fenêtre et vole vers son emplacement dans le canevas. »*
//!
//! # Un tableau de copies, à côté du document
//!
//! Ce qu'on y voit doit se dessiner, se viser, se zoomer et se lire **exactement** comme le canevas :
//! un tableau le fait déjà. La fenêtre est donc un tableau de copies, que le magasin montre à la
//! place du tableau actif tant qu'elle est ouverte ([`Store::active_board`]). Il vit **à côté du
//! projet** : aucune écriture du document ne l'atteint (elles nomment toutes un tableau du projet),
//! aucun enregistrement ne le lit (la vue s'écrit depuis le projet), et ses copies portent des
//! identités à elles — un geste qui s'y égarerait ne trouverait rien à toucher dans le document.
//!
//! # Ce qu'elle porte
//!
//! * le nœud montré, à sa place, et **chaque nœud qui lui est lié**, amené près de lui sur la
//!   demi-droite de sa vraie place ([`crate::amener`]) ;
//! * **sa provenance** : chacun dans une membrane titrée du chemin de ses membranes (« Sciences et
//!   médecine › Chimie › Eau »), à la couleur de la plus proche — la grammaire même de Glucose pour
//!   « dans quelle région », et la couleur d'origine de la région ;
//! * une **membrane** liée vient par sa propre membrane, son chemin et son nom en titre, et le
//!   premier texte qu'elle porte (ce qu'elle dit d'elle-même) — la membrane entière serait une
//!   région entière, à des kilomètres de taille ;
//! * **les liens**, du nœud à ce qui est amené, sans leurs coudes : ils contournent ce qu'ils
//!   traverseraient (FLECHE-5), comme partout, plutôt que de barrer un texte qu'on vient lire.

use super::Store;
use crate::geometry::Rect;
use crate::types::{Annotation, Board, BoardImage};
use std::collections::HashMap;

/// Le début de l'identité du tableau de la fenêtre ; aucun tableau du document ne la porte.
pub const TABLEAU: &str = "⟨fenêtre des liens⟩ ";
/// Ce qu'une copie ajoute devant l'identité de son original.
const COPIE: &str = "⟨copie⟩ ";
/// Ce qu'une membrane de provenance ajoute devant l'identité du nœud qu'elle entoure.
const PROVENANCE: &str = "⟨provenance⟩ ";

/// **La fenêtre ouverte** : ses copies, et ce que chacune désigne dans le document.
#[derive(Debug, Clone, PartialEq)]
pub struct FenetreDesLiens {
    /// Le tableau des copies, et la vue de la fenêtre.
    pub tableau: Board,
    /// Le nœud montré, dans le document.
    pub centre: String,
    /// Le tableau du document d'où elle vient.
    pub d_ou: String,
    /// Chaque identité de la fenêtre, et le nœud du document qu'elle désigne.
    designe: HashMap<String, String>,
    /// Combien de fois elle s'est refaite depuis son ouverture : son identité change à chaque fois,
    /// pour que ce qui garde le tableau actif (l'index des nœuds) le voie changer.
    ouverture: u64,
}

impl FenetreDesLiens {
    /// **Le nœud du document que désigne cette identité de la fenêtre** — une copie, sa membrane
    /// de provenance, un lien copié.
    pub fn original(&self, id: &str) -> Option<&str> {
        self.designe.get(id).map(String::as_str)
    }
}

impl Store {
    /// **Ouvre la fenêtre des liens de ce nœud** du tableau actif ; ouverte, elle se refait
    /// autour d'un nœud qu'elle montre (celui qu'on y désigne), **là où elle le montre** — le
    /// Bring & Go qui s'enchaîne. Rend faux si le nœud n'a aucun lien : rien à montrer.
    pub fn montrer_les_liens(&mut self, id: &str) -> bool {
        if !self.peut_montrer_les_liens(id) {
            return false;
        }
        let (original, ancre, vue, ouverture) = match &self.fenetre_des_liens {
            Some(f) => {
                let original = f.original(id).unwrap_or(id).to_string();
                // Là où la fenêtre montre ce nœud : sa copie, pas la membrane de sa provenance.
                let ancre =
                    crate::arrow::node_rect(&f.tableau, &format!("{COPIE}{original}")).map(milieu);
                (original, ancre, f.tableau.viewport, f.ouverture + 1)
            }
            None => (id.to_string(), None, self.viewport(), 1),
        };
        let d_ou = self.project.active_board_id.clone();
        let Some(board) = self.project.boards.iter().find(|b| b.id == d_ou) else {
            return false;
        };
        let Some(mut fenetre) = construire(board, &original, ancre) else {
            return false;
        };
        fenetre.tableau.viewport = vue;
        fenetre.ouverture = ouverture;
        fenetre.tableau.id = format!("{TABLEAU}{ouverture}");
        self.clear_selection();
        self.fenetre_des_liens = Some(fenetre);
        true
    }

    /// **Ferme la fenêtre**, et rend la vue où elle était — le tableau du document reprend la
    /// place d'où on la regardait, pour que rien ne saute. Rend faux si elle n'était pas ouverte.
    pub fn fermer_la_fenetre_des_liens(&mut self) -> bool {
        let Some(f) = self.fenetre_des_liens.take() else {
            return false;
        };
        self.clear_selection();
        self.set_viewport(&f.d_ou, f.tableau.viewport);
        true
    }

    /// **APPLIQUE-1 : la fenêtre des liens de ce nœud s'ouvrirait-elle ?** Un nœud du tableau
    /// actif — ou, fenêtre ouverte, le nœud qu'une copie désigne — lié par une flèche à un autre.
    /// Rien ne se construit : le menu pose la question à chaque image où il est ouvert, et
    /// [`Self::montrer_les_liens`] la pose avant de construire.
    pub fn peut_montrer_les_liens(&self, id: &str) -> bool {
        let original = self
            .fenetre_des_liens
            .as_ref()
            .and_then(|f| f.original(id))
            .unwrap_or(id);
        let actif = &self.project.active_board_id;
        self.project
            .boards
            .iter()
            .find(|b| &b.id == actif)
            .is_some_and(|b| !liens_de(b, original).is_empty())
    }

    /// La fenêtre des liens, si elle est ouverte.
    pub fn fenetre_des_liens(&self) -> Option<&FenetreDesLiens> {
        self.fenetre_des_liens.as_ref()
    }
}

/// Ce qu'un nœud devient dans la fenêtre : ses copies, la boîte qu'elles occupent, le milieu de la
/// copie de son nœud (le point qui glisse sur la demi-droite), et le milieu de sa vraie place.
struct Amene {
    annotations: Vec<Annotation>,
    images: Vec<BoardImage>,
    cadre: Rect,
    accroche: (f64, f64),
    vrai_milieu: (f64, f64),
}

impl Amene {
    fn deplacer(&mut self, (dx, dy): (f64, f64)) {
        for a in &mut self.annotations {
            a.translate(dx, dy);
        }
        for i in &mut self.images {
            i.x += dx;
            i.y += dy;
        }
        self.cadre = Rect::new(
            self.cadre.left + dx,
            self.cadre.top + dy,
            self.cadre.width,
            self.cadre.height,
        );
        self.accroche = (self.accroche.0 + dx, self.accroche.1 + dy);
    }
}

/// **La fenêtre des liens de `centre`**, ou rien s'il n'a aucun lien vers un nœud qui a une boîte.
/// `ancre`, s'il est donné, est le point où le centre se montre ; sinon, sa vraie place.
fn construire(board: &Board, centre: &str, ancre: Option<(f64, f64)>) -> Option<FenetreDesLiens> {
    let liens = liens_de(board, centre);
    let mut voisins: Vec<&str> = Vec::new();
    for (_, autre) in &liens {
        if !voisins.contains(autre) {
            voisins.push(autre);
        }
    }
    if voisins.is_empty() {
        return None;
    }
    let mut designe = HashMap::new();
    // Ce que tous partagent ne se répète pas : le début commun de leurs chemins (la carte entière,
    // une région qui les porte tous) se tait, et chacun dit ce qui le distingue.
    let communs = debut_commun(
        std::iter::once(centre)
            .chain(voisins.iter().copied())
            .map(|id| chemin_de(board, id).map(|c| c.0).unwrap_or_default()),
    );
    let mut au_centre = amener(board, (centre, communs), &mut designe)?;
    let mut amenes: Vec<Amene> = voisins
        .iter()
        .filter_map(|v| amener(board, (v, communs), &mut designe))
        .collect();
    // Le centre montre son nœud sur sa vraie place — une membrane, sa copie resserrée centrée sur
    // elle —, et les demi-droites en partent.
    let c = au_centre.vrai_milieu;
    au_centre.deplacer((c.0 - au_centre.accroche.0, c.1 - au_centre.accroche.1));
    let voisins: Vec<crate::amener::Voisin> = amenes
        .iter()
        .map(|a| crate::amener::Voisin {
            boite: a.cadre,
            accroche: a.accroche,
            vraie_place: a.vrai_milieu,
        })
        .collect();
    let places = crate::amener::disposer(au_centre.cadre, c, &voisins, super::agents::ECART);
    for (a, place) in amenes.iter_mut().zip(places) {
        a.deplacer((place.left - a.cadre.left, place.top - a.cadre.top));
    }
    if let Some(ancre) = ancre {
        let pas = (ancre.0 - c.0, ancre.1 - c.1);
        au_centre.deplacer(pas);
        for a in &mut amenes {
            a.deplacer(pas);
        }
    }
    let titre = titre_de(board, centre);
    let mut tableau = Board::new(TABLEAU, format!("Les liens de « {titre} »"));
    for a in std::iter::once(au_centre).chain(amenes) {
        tableau.annotations.extend(a.annotations);
        tableau.images.extend(a.images);
    }
    // Les membranes dessous, puis ce qu'elles portent, puis les liens : l'ordre du dessin.
    tableau
        .annotations
        .sort_by_key(|a| !matches!(a, Annotation::Membrane { .. }));
    for (lien, _) in &liens {
        if let Some(copie) = copier_le_lien(board, lien, &designe) {
            designe.insert(copie.id().to_string(), lien.to_string());
            tableau.annotations.push(copie);
        }
    }
    Some(FenetreDesLiens {
        tableau,
        centre: centre.to_string(),
        d_ou: board.id.clone(),
        designe,
        ouverture: 1,
    })
}

/// Les flèches qui touchent `centre`, et le nœud au bout de chacune ; un bout sans boîte (un nœud
/// disparu) ne s'amène pas, et son lien ne se copie pas.
fn liens_de<'a>(board: &'a Board, centre: &str) -> Vec<(&'a str, &'a str)> {
    board
        .annotations
        .iter()
        .filter_map(|a| {
            let Annotation::Arrow {
                id,
                source_id,
                target_id,
                ..
            } = a
            else {
                return None;
            };
            let autre = match (source_id.as_deref(), target_id.as_deref()) {
                (Some(s), Some(t)) if s == centre && t != centre => t,
                (Some(s), Some(t)) if t == centre && s != centre => s,
                _ => return None,
            };
            Some((id.as_str(), autre))
        })
        .collect()
}

/// **Un nœud amené** : sa copie (pour une membrane, sa membrane et le premier texte qu'elle porte),
/// dans une membrane de provenance s'il en a une.
fn amener(
    board: &Board,
    (id, communs): (&str, usize),
    designe: &mut HashMap<String, String>,
) -> Option<Amene> {
    let vraie = crate::arrow::node_rect(board, id)?;
    let copie = |original: &str, designe: &mut HashMap<String, String>| {
        let nom = format!("{COPIE}{original}");
        designe.insert(nom.clone(), original.to_string());
        nom
    };
    let mut amene = Amene {
        annotations: Vec::new(),
        images: Vec::new(),
        cadre: vraie,
        accroche: milieu(vraie),
        vrai_milieu: milieu(vraie),
    };
    let chemin = chemin_de(board, id);
    if let Some(a) = board.annotations.iter().find(|a| a.id() == id) {
        if let Annotation::Membrane { .. } = a {
            // Une membrane garde toujours son nom : son chemin peut se taire entier.
            let titre = chemin.map(|(noms, _)| sans_le_debut(&noms, communs, 0));
            return Some(membrane_amenee(board, a, titre, designe));
        }
        let mut c = a.clone();
        c.set_id(copie(id, designe));
        c.set_membrane_id(None);
        amene.annotations.push(c);
    } else {
        let mut c = board.images.iter().find(|i| i.id == id)?.clone();
        c.id = copie(id, designe);
        c.membrane_id = None;
        amene.images.push(c);
    }
    // Un nœud garde toujours la membrane la plus proche : on dit toujours d'où il vient.
    if let Some((noms, couleur)) = chemin {
        let titre = sans_le_debut(&noms, communs, 1);
        let cadre = Store::cadre_autour(vraie);
        let nom = format!("{PROVENANCE}{id}");
        designe.insert(nom.clone(), id.to_string());
        let mut m = Annotation::membrane(
            nom.clone(),
            cadre.left,
            cadre.top,
            cadre.width,
            cadre.height,
        );
        if let Annotation::Membrane { text, color, .. } = &mut m {
            (*text, *color) = (Some(titre), couleur);
        }
        for a in &mut amene.annotations {
            a.set_membrane_id(Some(nom.clone()));
        }
        for i in &mut amene.images {
            i.membrane_id = Some(nom.clone());
        }
        amene.annotations.insert(0, m);
        amene.cadre = cadre;
    }
    Some(amene)
}

/// **Une membrane amenée** : sa copie, son chemin et son nom en titre, autour du premier texte
/// qu'elle porte (dans l'ordre de lecture) — ou, sans texte, à la taille d'une membrane neuve.
fn membrane_amenee(
    board: &Board,
    m: &Annotation,
    chemin: Option<String>,
    designe: &mut HashMap<String, String>,
) -> Amene {
    let id = m.id();
    let vraie = m.rect().unwrap_or(Rect::ZERO);
    let premier = board
        .annotations
        .iter()
        .filter(|a| a.membrane_id() == Some(id) && matches!(a, Annotation::Text { .. }))
        .filter_map(|a| Some((a, crate::arrow::node_rect(board, a.id())?)))
        .min_by(|(_, a), (_, b)| a.top.total_cmp(&b.top).then(a.left.total_cmp(&b.left)));
    let mut annotations = Vec::new();
    let interieur = match premier {
        Some((texte, boite)) => {
            let mut c = texte.clone();
            let nom = format!("{COPIE}{}", texte.id());
            designe.insert(nom.clone(), texte.id().to_string());
            c.set_id(nom);
            c.set_membrane_id(Some(format!("{COPIE}{id}")));
            annotations.push(c);
            boite
        }
        None => {
            let (w, h) = crate::membrane_titre::NAISSANCE;
            centre_en(Rect::new(0.0, 0.0, w, h), milieu(vraie))
        }
    };
    let cadre = Store::cadre_autour(interieur);
    let mut copie = m.clone();
    let nom = format!("{COPIE}{id}");
    designe.insert(nom.clone(), id.to_string());
    copie.set_id(nom);
    copie.set_membrane_id(None);
    if let Annotation::Membrane {
        x,
        y,
        width,
        height,
        text,
        curtains,
        ..
    } = &mut copie
    {
        (*x, *y, *width, *height) = (cadre.left, cadre.top, cadre.width, cadre.height);
        let nom_propre = text.clone().unwrap_or_default();
        *text = Some(match chemin.filter(|c| !c.is_empty()) {
            Some(chemin) if !nom_propre.is_empty() => format!("{chemin} › {nom_propre}"),
            Some(chemin) => chemin,
            None => nom_propre,
        });
        curtains.clear();
    }
    annotations.insert(0, copie);
    Amene {
        annotations,
        images: Vec::new(),
        cadre,
        accroche: milieu(cadre),
        vrai_milieu: milieu(vraie),
    }
}

/// **Le chemin des membranes d'un nœud**, de la plus grande à la plus proche (« Sciences et
/// médecine › Chimie »), et la couleur de la plus proche ; rien s'il n'est dans aucune.
fn chemin_de(board: &Board, id: &str) -> Option<(Vec<String>, Option<String>)> {
    let mut noms = Vec::new();
    let mut couleur = None;
    let mut courant = parent_de(board, id);
    let mut vus = 0;
    while let Some(m) = courant {
        let Some(Annotation::Membrane { text, color, .. }) =
            board.annotations.iter().find(|a| a.id() == m)
        else {
            break;
        };
        if couleur.is_none() {
            couleur = Some(color.clone());
        }
        if let Some(t) = text.as_deref().filter(|t| !t.is_empty()) {
            noms.push(t.to_string());
        }
        courant = parent_de(board, m);
        // Une appartenance qui tournerait en rond (un document abîmé) ne bloque pas la fenêtre.
        vus += 1;
        if vus > board.annotations.len() {
            break;
        }
    }
    let couleur = couleur?;
    noms.reverse();
    Some((noms, couleur))
}

/// **Combien de noms tous ces chemins partagent en tête.**
fn debut_commun(chemins: impl Iterator<Item = Vec<String>>) -> usize {
    let mut commun: Option<Vec<String>> = None;
    for c in chemins {
        commun = Some(match commun {
            None => c,
            Some(p) => p
                .into_iter()
                .zip(c)
                .take_while(|(a, b)| a == b)
                .map(|(a, _)| a)
                .collect(),
        });
    }
    commun.map_or(0, |c| c.len())
}

/// **Le chemin sans son début commun**, qui garde au moins ses `garder` derniers noms.
fn sans_le_debut(noms: &[String], communs: usize, garder: usize) -> String {
    let k = communs.min(noms.len().saturating_sub(garder));
    noms[k..].join(" › ")
}

fn parent_de<'a>(board: &'a Board, id: &str) -> Option<&'a str> {
    match board.annotations.iter().find(|a| a.id() == id) {
        Some(a) => a.membrane_id(),
        None => board
            .images
            .iter()
            .find(|i| i.id == id)
            .and_then(|i| i.membrane_id.as_deref()),
    }
}

/// Le nom d'un nœud pour le titre de la fenêtre : le titre d'une membrane, le début d'un texte.
fn titre_de(board: &Board, id: &str) -> String {
    let texte = match board.annotations.iter().find(|a| a.id() == id) {
        Some(Annotation::Membrane { text, .. }) => text.clone().unwrap_or_default(),
        Some(Annotation::Text { text, .. } | Annotation::Sticky { text, .. }) => text.clone(),
        _ => String::new(),
    };
    texte
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(40)
        .collect()
}

/// **La copie d'un lien** : de la copie de son départ à celle de son arrivée, droite.
fn copier_le_lien(
    board: &Board,
    id: &str,
    designe: &HashMap<String, String>,
) -> Option<Annotation> {
    let mut copie = board.annotations.iter().find(|a| a.id() == id)?.clone();
    let vers_la_copie = |bout: &Option<String>| -> Option<String> {
        let b = bout.as_deref()?;
        let nom = format!("{COPIE}{b}");
        designe.contains_key(&nom).then_some(nom)
    };
    let Annotation::Arrow {
        source_id,
        target_id,
        waypoints,
        membrane_id,
        ..
    } = &mut copie
    else {
        return None;
    };
    let (s, t) = (vers_la_copie(source_id)?, vers_la_copie(target_id)?);
    (*source_id, *target_id) = (Some(s), Some(t));
    waypoints.clear();
    *membrane_id = None;
    copie.set_id(format!("{COPIE}{id}"));
    Some(copie)
}

fn milieu(b: Rect) -> (f64, f64) {
    (b.left + b.width / 2.0, b.top + b.height / 2.0)
}

/// Une boîte de la taille de `b`, centrée en `c`.
fn centre_en(b: Rect, c: (f64, f64)) -> Rect {
    Rect::new(c.0 - b.width / 2.0, c.1 - b.height / 2.0, b.width, b.height)
}

#[cfg(test)]
#[path = "fenetre/tests.rs"]
mod tests;
