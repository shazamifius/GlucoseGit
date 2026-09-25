//! **Les onglets** : leur mise en page, et ce qu'on est en train d'y faire (BOARDS-1).
//!
//! Un onglet est un tableau racine — le contenu d'un dossier n'en est pas un
//! ([`glucose_core::store::Store::onglets`]). On le choisit d'un clic, on le renomme d'un
//! double-clic, on le range en le glissant, on le supprime par sa croix ; le clic droit ouvre
//! le même menu. C'est ce que faisait `BoardTabs.tsx` dans Glucose Tauri, et ce que font les
//! pages de tldraw ou les feuilles d'un classeur.
//!
//! La géométrie ne se calcule qu'ici (loi L4) : le dessin ([`super::bande`]) et le clic
//! ([`super::handle_ui_click`]) lisent la même liste.

use super::{UiState, TABS_HEIGHT};
use crate::interactions::text_entry::TextEntry;
use crate::typography::{Face, Typography};
use glucose_core::store::Store;
use std::hash::{Hash, Hasher};

/// Le corps du nom d'un onglet, en points.
pub(super) const CORPS: f32 = 12.0;
/// La marge de part et d'autre du nom, en points.
const MARGE: f32 = 14.0;
/// L'espace entre deux onglets, en points.
const ESPACE: f32 = 4.0;
/// La croix d'un onglet et l'écart qui la sépare du nom — ceux de Glucose Tauri (14 et 6).
const CROIX: f32 = 14.0;
const ECART: f32 = 6.0;
/// La largeur du champ de renommage — celle de Glucose Tauri (`width: 100`).
pub(super) const CHAMP: f32 = 100.0;

/// Ce qu'on est en train de faire aux onglets.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EtatDesOnglets {
    /// L'onglet qu'on renomme, et ce qu'on y tape.
    pub renomme: Option<(String, TextEntry)>,
    /// L'onglet que la souris tient.
    pub tenu: Option<OngletTenu>,
    /// L'onglet sur lequel le menu contextuel s'est ouvert.
    pub menu: Option<String>,
}

/// Un onglet pris à la souris : s'il a bougé au-delà d'un tremblement, il glisse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OngletTenu {
    pub id: String,
    /// Où le bouton s'est enfoncé, en pixels.
    pub depuis: (i64, i64),
    pub glisse: bool,
}

/// Un onglet, ou le bouton « + », une fois posé.
#[derive(Debug, Clone, PartialEq)]
pub struct TabButtonLayout {
    pub board_id: String,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub is_active: bool,
    pub is_plus: bool,
    /// La croix qui supprime l'onglet : `(x, y, côté)`. Absente quand il ne reste qu'un
    /// onglet — le dernier ne se supprime pas — et pendant qu'on le renomme.
    pub fermer: Option<(f32, f32, f32)>,
}

/// Ce qu'un point de la barre d'onglets désigne.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CibleOnglet {
    Onglet(String),
    Fermer(String),
    Plus,
}

/// **Pose les onglets**, puis le bouton « + ».
pub fn layout_tabs(store: &Store, ui: &UiState, typo: &Typography) -> Vec<TabButtonLayout> {
    let s = ui.scale();
    let y = ui.topbar_height();
    let hauteur = TABS_HEIGHT * s;
    let plusieurs = store.onglets().nth(1).is_some();
    let renomme = ui.onglets.renomme.as_ref().map(|(id, _)| id.as_str());
    let mut layouts = Vec::new();
    let mut x = 8.0 * s;
    for (id, nom, actif) in store.onglets() {
        let en_renommage = renomme == Some(id);
        let (nom_w, _) = typo.measure_text(nom, CORPS * s, face(actif));
        let contenu = if en_renommage { CHAMP * s } else { nom_w };
        let croix = (plusieurs && !en_renommage).then_some((ECART + CROIX) * s);
        let largeur = contenu + 2.0 * MARGE * s + croix.unwrap_or(0.0);
        layouts.push(TabButtonLayout {
            board_id: id.to_string(),
            name: nom.to_string(),
            x,
            y,
            width: largeur,
            height: hauteur,
            is_active: actif,
            is_plus: false,
            fermer: croix.map(|_| {
                let cote = CROIX * s;
                (
                    x + largeur - MARGE * s - cote,
                    y + (hauteur - cote) / 2.0,
                    cote,
                )
            }),
        });
        x += largeur + ESPACE * s;
    }
    layouts.push(TabButtonLayout {
        board_id: String::new(),
        name: "+".into(),
        x,
        y,
        width: 30.0 * s,
        height: hauteur,
        is_active: false,
        is_plus: true,
        fermer: None,
    });
    layouts
}

/// Le style du nom : gras pour l'onglet actif.
pub(super) fn face(actif: bool) -> Face {
    if actif {
        Face::Bold
    } else {
        Face::Regular
    }
}

/// Ce que ce point désigne parmi ces onglets. La croix passe avant l'onglet qui la porte.
pub fn cible(onglets: &[TabButtonLayout], x: f32, y: f32) -> Option<CibleOnglet> {
    let t = onglets
        .iter()
        .find(|t| x >= t.x && x < t.x + t.width && y >= t.y && y < t.y + t.height)?;
    if t.is_plus {
        return Some(CibleOnglet::Plus);
    }
    let sur_la_croix = t
        .fermer
        .is_some_and(|(cx, cy, c)| x >= cx && x < cx + c && y >= cy && y < cy + c);
    Some(if sur_la_croix {
        CibleOnglet::Fermer(t.board_id.clone())
    } else {
        CibleOnglet::Onglet(t.board_id.clone())
    })
}

/// **Tout ce dont l'aspect des onglets dépend**, réduit à une empreinte : les tableaux — nom,
/// ordre, actif —, le nom qu'on tape et son curseur, l'onglet qui glisse.
pub(super) fn empreinte(onglets: &[TabButtonLayout], etat: &EtatDesOnglets) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for t in onglets {
        t.board_id.hash(&mut h);
        t.name.hash(&mut h);
        t.is_active.hash(&mut h);
        t.fermer.is_some().hash(&mut h);
    }
    if let Some((id, entree)) = &etat.renomme {
        id.hash(&mut h);
        entree.text().hash(&mut h);
        entree.before_cursor().len().hash(&mut h);
    }
    etat.tenu
        .as_ref()
        .filter(|t| t.glisse)
        .map(|t| &t.id)
        .hash(&mut h);
    h.finish()
}

#[cfg(test)]
mod tests;
