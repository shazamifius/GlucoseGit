//! **Importer un document dans de nouveaux onglets** (BOARDS-2).
//!
//! # Ce que l'utilisateur a demandé, et les deux conditions
//!
//! Faire entrer un ancien projet dans le document courant, comme un onglet de plus — un geste
//! à part de `Ctrl+O`, qui reste « ouvrir » (sa réponse du 25/09). Deux conditions le rendent
//! compatible avec `Ctrl+Z` et la Time Machine :
//!
//! * **un seul geste** : un seul `Ctrl+Z` défait tout l'import ;
//! * **seul son présent entre**, pas son histoire : la réglette y verrait deux passés
//!   entremêlés. Le fichier d'origine garde la sienne, intacte. C'est ce que fait Figma d'un
//!   `.fig` réimporté — un nouveau fichier sans historique, relié à rien.
//!
//! # Tous les identifiants sont renommés
//!
//! Deux documents numérotent leurs éléments chacun de leur côté : `text-12` peut exister des
//! deux. Glucose Tauri, pour ses greffons, ne renommait que l'identifiant du tableau — deux
//! imports du même document auraient eu des cartes jumelles. Ici, chaque élément reçoit un
//! identifiant neuf du générateur du document, et chaque référence suit : le propriétaire
//! d'un élément, le miroir et sa source, les deux bouts d'une flèche et son portail, le
//! tableau d'un dossier et celui d'un rideau, les domaines assignés. Une référence vers ce
//! que le document importé ne contient pas **disparaît** au lieu de viser, par hasard, un
//! élément du document courant qui porterait le même nom.
//!
//! Chaque type est déstructuré en entier, sans `..` : un champ ajouté un jour fera échouer la
//! compilation ici, au lieu de laisser passer un identifiant non renommé.

use super::boards::racines;
use super::journal::{Edit, Slot, Whole};
use super::Store;
use crate::types::{
    Annotation, Board, BoardImage, CanvasFolder, Domain, DomainAssignment, Id, MembraneCurtain,
    Project, StoryboardPanel,
};
use std::collections::HashMap;

/// L'ancien identifiant de chaque élément importé, et le nouveau.
#[derive(Default)]
struct Renommage(HashMap<Id, Id>);

impl Renommage {
    /// Donne un nouvel identifiant à `id`, et le retient.
    fn nommer(&mut self, store: &mut Store, prefixe: &str, id: &mut Id) {
        let neuf = store.generate_id(prefixe);
        self.0.insert(std::mem::replace(id, neuf.clone()), neuf);
    }

    /// Une référence suit l'élément qu'elle vise ; vers un absent, elle disparaît.
    fn suivre(&self, r: &mut Option<Id>) {
        *r = r.as_ref().and_then(|id| self.0.get(id).cloned());
    }

    /// Les domaines assignés suivent les leurs ; une assignation vers un domaine absent part.
    fn suivre_les_domaines(&self, domaines: &mut Vec<DomainAssignment>) {
        domaines.retain_mut(|d| {
            let Some(neuf) = self.0.get(&d.domain_id) else {
                return false;
            };
            d.domain_id = neuf.clone();
            true
        });
    }
}

/// Le préfixe d'une annotation, selon sa sorte — celui que le document lui donnerait.
fn prefixe(a: &Annotation) -> &'static str {
    match a {
        Annotation::Text { .. } => "text",
        Annotation::Sticky { .. } => "sticky",
        Annotation::Arrow { .. } => "arrow",
        Annotation::Membrane { .. } => "membrane",
    }
}

impl Store {
    /// **Fait entrer ce document dans des onglets neufs**, en un seul geste, et rend les
    /// identifiants de ses onglets. Le premier devient l'onglet actif.
    ///
    /// Un document d'un seul onglet prend le nom `nom` ; ceux d'un document qui en a plusieurs
    /// s'appellent « `nom` — leur nom ».
    pub fn importer_un_document(&mut self, mut doc: Project, nom: &str) -> Vec<String> {
        let onglets_du_doc = racines(&doc.boards);
        let renommage = self.renommer_le_document(&mut doc);
        let onglets: Vec<String> = onglets_du_doc
            .iter()
            .filter_map(|id| renommage.0.get(id).cloned())
            .collect();
        for b in &mut doc.boards {
            if onglets.contains(&b.id) {
                b.name = if onglets.len() == 1 {
                    nom.to_string()
                } else {
                    format!("{nom} — {}", b.name)
                };
            }
        }
        let mut edits = Vec::new();
        for domaine in doc.domains {
            edits.push(Edit::Domain {
                slot: Slot::inserted(self.project.domains.len(), domaine.clone()),
            });
            self.project.domains.push(domaine);
        }
        for board in doc.boards {
            edits.push(Edit::Board {
                slot: Slot::inserted(self.project.boards.len(), board.clone()),
            });
            self.project.boards.push(board);
        }
        if let Some(premier) = onglets.first() {
            let before = std::mem::replace(&mut self.project.active_board_id, premier.clone());
            edits.push(Edit::ActiveBoard {
                whole: Whole::new(before, premier.clone()),
            });
        }
        self.record_as_one_gesture(edits);
        onglets
    }

    /// Renomme tout ce que le document porte, puis fait suivre chaque référence.
    fn renommer_le_document(&mut self, doc: &mut Project) -> Renommage {
        let mut r = Renommage::default();
        for d in &mut doc.domains {
            let Domain {
                id,
                name: _,
                color: _,
                icon: _,
                created_at: _,
            } = d;
            r.nommer(self, "domain", id);
        }
        for b in &mut doc.boards {
            self.nommer_le_tableau(&mut r, b);
        }
        for b in &mut doc.boards {
            suivre_dans_le_tableau(&r, b);
        }
        r
    }

    /// Premier passage : un nom neuf pour le tableau et pour chaque élément qu'il porte.
    fn nommer_le_tableau(&mut self, r: &mut Renommage, b: &mut Board) {
        r.nommer(self, "board", &mut b.id);
        for i in &mut b.images {
            r.nommer(self, "img", &mut i.id);
        }
        for a in &mut b.annotations {
            let p = prefixe(a);
            let (id, rideaux) = match a {
                Annotation::Membrane { id, curtains, .. } => (id, Some(curtains)),
                Annotation::Text { id, .. }
                | Annotation::Sticky { id, .. }
                | Annotation::Arrow { id, .. } => (id, None),
            };
            r.nommer(self, p, id);
            for rideau in rideaux.into_iter().flatten() {
                r.nommer(self, "curtain", &mut rideau.id);
                for note in &mut rideau.notes {
                    r.nommer(self, "note", &mut note.id);
                }
            }
        }
        for f in &mut b.folders {
            r.nommer(self, "folder", &mut f.id);
        }
        for p in &mut b.panels {
            r.nommer(self, "panel", &mut p.id);
        }
    }
}

/// Second passage : chaque référence du tableau suit l'élément qu'elle vise.
fn suivre_dans_le_tableau(r: &Renommage, b: &mut Board) {
    let Board {
        id: _,
        name: _,
        images,
        annotations,
        folders,
        panels,
        zones: _,
        viewport: _,
        bookmarks: _,
        created_at: _,
        updated_at: _,
    } = b;
    images.iter_mut().for_each(|i| suivre_dans_l_image(r, i));
    annotations
        .iter_mut()
        .for_each(|a| suivre_dans_l_annotation(r, a));
    folders
        .iter_mut()
        .for_each(|f| suivre_dans_le_dossier(r, f));
    panels.iter_mut().for_each(|p| {
        let StoryboardPanel {
            id: _,
            order: _,
            description: _,
            x: _,
            y: _,
            width: _,
            height: _,
        } = p;
    });
}

fn suivre_dans_l_image(r: &Renommage, i: &mut BoardImage) {
    let BoardImage {
        id: _,
        membrane_id,
        asset: _,
        src: _,
        x: _,
        y: _,
        width: _,
        height: _,
        rotation: _,
        locked: _,
        tags: _,
        slot_id: _,
        source_url: _,
        original_width: _,
        original_height: _,
        is_video: _,
        crop: _,
        fit: _,
        domains,
        mirror_of,
        temporal_anchor: _,
    } = i;
    r.suivre(membrane_id);
    r.suivre(mirror_of);
    r.suivre_les_domaines(domains);
}

fn suivre_dans_le_dossier(r: &Renommage, f: &mut CanvasFolder) {
    let CanvasFolder {
        id: _,
        name: _,
        color: _,
        x: _,
        y: _,
        width: _,
        height: _,
        child_board_id,
        mirror_of,
        mirror_source: _,
    } = f;
    // Un dossier dont le tableau manque garde son identifiant : il ne vise rien, ici comme
    // dans le document d'origine.
    if let Some(neuf) = r.0.get(child_board_id.as_str()) {
        *child_board_id = neuf.clone();
    }
    r.suivre(mirror_of);
}

fn suivre_dans_l_annotation(r: &Renommage, a: &mut Annotation) {
    let (membrane_id, domains, mirror_of) = match a {
        Annotation::Text {
            id: _,
            x: _,
            y: _,
            width: _,
            height: _,
            text: _,
            font_size: _,
            color: _,
            cursor_pos: _,
            source_file: _,
            membrane_id,
            domains,
            mirror_of,
            temporal_anchor: _,
        } => (membrane_id, domains, mirror_of),
        Annotation::Sticky {
            id: _,
            x: _,
            y: _,
            width: _,
            height: _,
            text: _,
            font_size: _,
            color: _,
            bg_color: _,
            cursor_pos: _,
            operator: _,
            source_file: _,
            membrane_id,
            domains,
            mirror_of,
            temporal_anchor: _,
        } => (membrane_id, domains, mirror_of),
        Annotation::Arrow { .. } => return suivre_dans_la_fleche(r, a),
        Annotation::Membrane {
            id: _,
            x: _,
            y: _,
            width: _,
            height: _,
            color: _,
            text: _,
            mode: _,
            curtains,
            membrane_id,
            domains,
            mirror_of,
            temporal_anchor: _,
        } => {
            curtains
                .iter_mut()
                .for_each(|c| suivre_dans_le_rideau(r, c));
            (membrane_id, domains, mirror_of)
        }
    };
    r.suivre(membrane_id);
    r.suivre(mirror_of);
    r.suivre_les_domaines(domains);
}

/// Une flèche : ses deux bouts et son portail, en plus de ce que toute annotation porte. Les
/// blocs et les sélections de texte visés sont des positions **dans** une carte : ils
/// suivent la carte, sans nom à changer.
fn suivre_dans_la_fleche(r: &Renommage, a: &mut Annotation) {
    let Annotation::Arrow {
        id: _,
        x: _,
        y: _,
        x2: _,
        y2: _,
        text: _,
        font_size: _,
        color: _,
        arrow_type: _,
        arrow_bidirectional: _,
        predicate: _,
        stroke_width: _,
        waypoints: _,
        source_id,
        target_id,
        source_block_id: _,
        target_block_id: _,
        source_text_sel: _,
        target_text_sel: _,
        long_text: _,
        target_board_id,
        membrane_id,
        domains,
        mirror_of,
        temporal_anchor: _,
    } = a
    else {
        return;
    };
    for reference in [
        source_id,
        target_id,
        target_board_id,
        membrane_id,
        mirror_of,
    ] {
        r.suivre(reference);
    }
    r.suivre_les_domaines(domains);
}

fn suivre_dans_le_rideau(r: &Renommage, c: &mut MembraneCurtain) {
    let MembraneCurtain {
        id: _,
        owner_id: _,
        owner_name: _,
        owner_color: _,
        visibility: _,
        editable: _,
        collapsed_ratio: _,
        expanded_ratio: _,
        board_id,
        notes: _,
        created_at: _,
    } = c;
    r.suivre(board_id);
}

#[cfg(test)]
mod tests;
