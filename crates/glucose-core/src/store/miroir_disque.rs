//! **Un dossier miroir du disque reçoit ce que le disque a rendu** (DISQUE-1 et DISQUE-2, fiche 164) :
//! la première lecture le remplit, les suivantes le réconcilient — ce qui entre, ce qui suit son
//! fichier, ce qui part (la loi : [`crate::miroir_disque::reconcilier`]) — et le dossier se dit lu.
//!
//! **Un geste comme les autres**, et non une lecture hors du journal comme `expand_folder` (l'héritage
//! de Tauri) : le document est une histoire de gestes. Ce qui n'y entre pas ne s'enregistre pas,
//! ne se partage pas, ne s'annule pas. Il est **signé « Glucose »** : la lecture arrive d'un autre
//! fil, après l'entrée — personne ne l'a faite à la main. Un `Ctrl+Z` juste après la première
//! lecture vide le dossier, qui redevient « à lire » ; y revenir le relit.
//!
//! **La disposition est celle de COLONNES-1** ([`crate::layout::colonnes`]) : les sous-dossiers,
//! les photos, les textes, puis les tuiles, chacun en bloc, dans l'ordre du tri, en colonnes qui
//! approchent la proportion de l'écran. Aucune constante à elle : la grille en croix de Glucose
//! Tauri (des cellules de 220, des écarts de 160, une rangée de 1 500) a disparu dans la loi qui
//! dispose déjà les membranes.

use super::journal::{Edit, Main, Slot};
use super::redresser::GLUCOSE;
use super::Store;
use crate::geometry::Rect;
use crate::layout::colonnes::{disposer, Bloc, Marges};
use crate::miroir_disque::reconcilier::Noeud;
use crate::types::{Annotation, BoardImage, CanvasFolder, FolderMirrorSource};
use std::collections::HashMap;

/// **Un niveau lu**, dans l'ordre du tri : ses sous-dossiers (chacun une source en attente), ses
/// photos, ses textes et ses tuiles. Neufs, les positions qu'ils portent ne comptent pas : le niveau
/// se dispose ici ; suivis, chacun est la nouvelle version entière d'un nœud du tableau.
#[derive(Debug, Default, Clone)]
pub struct Niveau {
    pub dossiers: Vec<CanvasFolder>,
    pub photos: Vec<BoardImage>,
    pub textes: Vec<Annotation>,
    pub tuiles: Vec<Annotation>,
}

/// **Le miroir où l'on se trouve** : le tableau qui le porte, son identifiant, sa source, et le
/// tableau qu'il montre.
#[derive(Debug, Clone, PartialEq)]
pub struct MiroirOuvert {
    pub parent: String,
    pub dossier: String,
    pub tableau: String,
    pub source: FolderMirrorSource,
}

/// **Ce qu'une lecture du disque change à un miroir** (la loi : [`crate::miroir_disque::reconcilier`]) :
/// les nœuds neufs (disposés ici, sous ce qu'il y a déjà), la nouvelle version de ceux qui suivent
/// leur fichier, et ceux qui partent avec lui.
#[derive(Debug, Default, Clone)]
pub struct Reconciliation {
    pub neufs: Niveau,
    pub suivis: Niveau,
    pub partis: Vec<Noeud>,
}

impl Reconciliation {
    fn est_vide(&self) -> bool {
        let n = |v: &Niveau| {
            v.dossiers.is_empty()
                && v.photos.is_empty()
                && v.textes.is_empty()
                && v.tuiles.is_empty()
        };
        n(&self.neufs) && n(&self.suivis) && self.partis.is_empty()
    }
}

impl Store {
    /// **Le dossier miroir où l'on se trouve**, lu ou non. Le haut de la pile des dossiers suffit —
    /// quelle que soit la porte par laquelle on est entré (le double-clic, le fil d'Ariane, un retour
    /// en arrière).
    pub fn miroir_ouvert(&self) -> Option<MiroirOuvert> {
        let (parent, id) = self.folder_stack.last()?;
        let f = self
            .project
            .boards
            .iter()
            .find(|b| &b.id == parent)?
            .folders
            .iter()
            .find(|f| &f.id == id)?;
        Some(MiroirOuvert {
            parent: parent.clone(),
            dossier: id.clone(),
            tableau: f.child_board_id.clone(),
            source: f.mirror_source.clone()?,
        })
    }

    /// **Applique une lecture du disque à ce miroir**, en un seul geste signé « Glucose » — personne
    /// ne l'a fait à la main —, défaisable, et le dit lu à l'heure `maintenant` (secondes depuis
    /// 1970). Les nœuds neufs se disposent pour approcher `proportion` (largeur sur hauteur), sous
    /// ce que le tableau montre déjà : la place qu'il a donnée aux autres ne bouge pas.
    ///
    /// Rend `Some(true)` quand ce geste a rempli un miroir vide qui attendait sa lecture (sa
    /// première vue reste à poser), `Some(false)` pour un autre geste, `None` si rien ne change — un
    /// dossier introuvable, une lecture sans rien de neuf, ou un geste ouvert : la lecture attend
    /// qu'il se ferme plutôt que de s'y fondre (un `Échap` défairait les deux).
    pub fn reconcilier_le_miroir(
        &mut self,
        parent: &str,
        dossier: &str,
        r: Reconciliation,
        proportion: f64,
        maintenant: i64,
    ) -> Option<bool> {
        let f = self
            .project
            .boards
            .iter()
            .find(|b| b.id == parent)?
            .folders
            .iter()
            .find(|f| f.id == dossier)?;
        let attendait = f.mirror_source.as_ref()?.pending_scan;
        let enfant = f.child_board_id.clone();
        if self.in_live_edit() || (r.est_vide() && !attendait) {
            return None;
        }
        let vide = self.content_bounds(&enfant).is_none();
        let main = std::mem::replace(&mut self.journal.main, Main::Agent(GLUCOSE.into()));
        self.begin_live_edit();
        self.suivre_et_retirer(&enfant, r.suivis, r.partis);
        let neufs = disposer_le_niveau(r.neufs, proportion);
        let neufs = self.sous_le_contenu(&enfant, neufs);
        for f in neufs.dossiers {
            self.create_folder_with_content(&enfant, f, Vec::new());
        }
        let noeuds = neufs.textes.into_iter().chain(neufs.tuiles);
        let edits = self.poser_dans(&enfant, neufs.photos, noeuds.collect());
        self.record_as_one_gesture(edits);
        self.update_folder(parent, dossier, |f| {
            if let Some(s) = f.mirror_source.as_mut() {
                s.pending_scan = false;
                s.last_scanned_at = maintenant;
            }
        });
        self.end_live_edit();
        self.journal.main = main;
        Some(attendait && vide && self.content_bounds(&enfant).is_some())
    }

    /// Les nœuds qui suivent leur fichier prennent leur nouvelle version ; ceux qui partent partent.
    fn suivre_et_retirer(&mut self, tableau: &str, suivis: Niveau, partis: Vec<Noeud>) {
        for img in suivis.photos {
            let id = img.id.clone();
            self.update_image(tableau, &id, |i| *i = img);
        }
        for a in suivis.textes.into_iter().chain(suivis.tuiles) {
            let id = a.id().to_owned();
            self.update_annotation(tableau, &id, |n| *n = a);
        }
        for d in suivis.dossiers {
            let id = d.id.clone();
            self.update_folder(tableau, &id, |f| *f = d);
        }
        let (mut photos, mut notes, mut dossiers) = (Vec::new(), Vec::new(), Vec::new());
        for n in &partis {
            match n {
                Noeud::Photo(id) => photos.push(id.as_str()),
                Noeud::Note(id) => notes.push(id.as_str()),
                Noeud::Dossier(id) => dossiers.push(id.as_str()),
            }
        }
        if !photos.is_empty() {
            self.remove_images(tableau, &photos);
        }
        if !notes.is_empty() {
            self.remove_annotations(tableau, &notes);
        }
        if !dossiers.is_empty() {
            self.remove_folders(tableau, &dossiers);
        }
    }

    /// Les nœuds neufs d'un tableau qui en montre déjà : le bloc qu'ils forment se pose sous le
    /// contenu, à son bord gauche, à l'écart de Glucose.
    fn sous_le_contenu(&self, tableau: &str, mut niveau: Niveau) -> Niveau {
        let Some(c) = self.content_bounds(tableau) else {
            return niveau;
        };
        let (dx, dy) = (c.left, c.top + c.height + super::zones::ECART);
        for f in &mut niveau.dossiers {
            (f.x, f.y) = (f.x + dx, f.y + dy);
        }
        for i in &mut niveau.photos {
            (i.x, i.y) = (i.x + dx, i.y + dy);
        }
        for a in niveau.textes.iter_mut().chain(niveau.tuiles.iter_mut()) {
            a.translate(dx, dy);
        }
        niveau
    }

    /// Pose ces photos et ces nœuds dans ce tableau, sans toucher à la sélection, et rend ce qui
    /// s'est fait.
    fn poser_dans(
        &mut self,
        board: &str,
        photos: Vec<BoardImage>,
        noeuds: Vec<Annotation>,
    ) -> Vec<Edit> {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board) else {
            return Vec::new();
        };
        let mut edits = Vec::with_capacity(photos.len() + noeuds.len());
        for img in photos {
            edits.push(Edit::Image {
                board: board.to_string(),
                slot: Slot::inserted(b.images.len(), img.clone()),
            });
            b.images.push(img);
        }
        for ann in noeuds {
            edits.push(Edit::Annotation {
                board: board.to_string(),
                slot: Slot::inserted(b.annotations.len(), ann.clone()),
            });
            b.annotations.push(ann);
        }
        edits
    }
}

/// **Dispose un niveau** : un bloc par sorte, non vide, dans l'ordre du niveau ; chaque nœud reçoit
/// sa place, l'origine au coin haut-gauche du tableau.
pub fn disposer_le_niveau(mut niveau: Niveau, proportion: f64) -> Niveau {
    let feuille = |id: &str, largeur: f64, hauteur: f64| Bloc {
        id: id.to_string(),
        largeur,
        hauteur,
        enfants: Vec::new(),
        titree: false,
    };
    let groupes: [Vec<Bloc>; 4] = [
        niveau
            .dossiers
            .iter()
            .map(|f| feuille(&f.id, f.width, f.height))
            .collect(),
        niveau
            .photos
            .iter()
            .map(|i| feuille(&i.id, i.width, i.height))
            .collect(),
        blocs_de(&niveau.textes, feuille),
        blocs_de(&niveau.tuiles, feuille),
    ];
    let racine = Bloc {
        enfants: groupes
            .into_iter()
            .enumerate()
            .filter(|(_, g)| !g.is_empty())
            // Un identifiant qu'aucun nœud ne porte : un caractère nul ne s'écrit dans aucun.
            .map(|(k, enfants)| Bloc {
                enfants,
                ..feuille(&format!("\0{k}"), 0.0, 0.0)
            })
            .collect(),
        ..feuille("\0", 0.0, 0.0)
    };
    if racine.enfants.is_empty() {
        return niveau;
    }
    let proportion = if proportion.is_finite() && proportion > 0.0 {
        proportion
    } else {
        1.0
    };
    // Les marges d'une membrane sans titre : l'écart de Glucose partout.
    let ecart = super::zones::ECART;
    let places: HashMap<String, Rect> = disposer(
        &racine,
        proportion,
        Marges {
            ecart,
            titre: ecart,
        },
    )
    .into_iter()
    .collect();
    for f in &mut niveau.dossiers {
        if let Some(r) = places.get(&f.id) {
            (f.x, f.y) = (r.left, r.top);
        }
    }
    // Une photo est ancrée en son centre (R-11).
    for i in &mut niveau.photos {
        if let Some(r) = places.get(&i.id) {
            (i.x, i.y) = (r.left + r.width / 2.0, r.top + r.height / 2.0);
        }
    }
    for a in niveau.textes.iter_mut().chain(niveau.tuiles.iter_mut()) {
        if let (Some(r), Some(ici)) = (places.get(a.id()), a.rect()) {
            a.translate(r.left - ici.left, r.top - ici.top);
        }
    }
    niveau
}

fn blocs_de(noeuds: &[Annotation], feuille: impl Fn(&str, f64, f64) -> Bloc) -> Vec<Bloc> {
    noeuds
        .iter()
        .filter_map(|a| {
            let r = a.rect()?;
            Some(feuille(a.id(), r.width, r.height))
        })
        .collect()
}

#[cfg(test)]
mod tests;
