//! **REVENIR-1, le document entier** (fiche 146 § 4) : la fusion à trois de chaque liste et de chaque
//! valeur du document, et les éditions du journal qui la refont dans la salle.
//!
//! Un tableau présent des deux côtés **ne se compare pas en bloc** : changé ici dans un nœud et là-bas
//! dans un autre, ce serait un conflit, et le travail d'ici serait rendu au lieu d'être gardé. Il se
//! fusionne liste par liste (photos, textes et flèches, dossiers, cases et storyboards ; son nom, ses
//! zones). Seul un tableau d'un seul côté — posé ici, ou retiré — passe par la liste des tableaux.
//! Le point de vue et l'onglet regardé restent à chacun (ils ne partent jamais, PARTAGE-1).

use super::fusion::{fondre_la_liste, fondre_la_valeur, FusionDeListe, FusionDeValeur};
use super::partage::{GestePartage, Nomme};
use super::{Edit, Slot, Whole};
use crate::types::{Board, Project};

/// **Un document fusionné.**
#[derive(Debug, Clone)]
pub struct FusionDuDocument {
    /// Le document de la salle, plus ce qui n'a changé qu'ici.
    pub projet: Project,
    /// Ce qui n'a changé qu'ici : les éditions du journal à appliquer dans l'ordre au document de
    /// la salle — ce qui y part.
    pub a_envoyer: Vec<Edit>,
    /// Le nom de chaque élément changé des deux côtés : la salle a gagné.
    pub conflits: Vec<String>,
}

/// **Fusionne le document d'ici avec celui de la salle**, depuis leur ancêtre commun.
pub fn fondre_le_document(ancetre: &Project, ici: &Project, la_bas: &Project) -> FusionDuDocument {
    let mut f = FusionDuDocument {
        projet: la_bas.clone(),
        a_envoyer: Vec::new(),
        conflits: Vec::new(),
    };
    let nom = fondre_la_valeur(&ancetre.name, &ici.name, &la_bas.name);
    f.projet.name = f.valeur(nom, "le nom du document", |whole| Edit::ProjectName {
        whole,
    });
    let domaines = fondre_la_liste(&ancetre.domains, &ici.domains, &la_bas.domains);
    f.projet.domains = f.liste(domaines, |slot| Edit::Domain { slot });
    let preglages = fondre_la_liste(&ancetre.presets, &ici.presets, &la_bas.presets);
    f.projet.presets = f.liste(preglages, |slot| Edit::Preset { slot });
    f.fondre_les_tableaux(ancetre, ici, la_bas);
    f
}

/// **Revenir dans sa salle** (fiche 146 § 4, le 2) : la salle rebâtie depuis sa base — au numéro
/// `retenu`, le dernier que le fichier d'ici contient (au moins : un numéro trop bas ne gêne pas), et
/// maintenant —, puis fusionnée avec le document d'ici. `gestes` : ce que la salle garde après sa base,
/// avec leur numéro, dans son ordre.
pub fn revenir(
    ici: &Project,
    base: &Project,
    gestes: &[(u64, GestePartage)],
    retenu: u64,
) -> FusionDuDocument {
    let mut ancetre = base.clone();
    let mut la_bas = base.clone();
    for (n, geste) in gestes {
        if *n <= retenu {
            geste.appliquer(&mut ancetre);
        }
        geste.appliquer(&mut la_bas);
    }
    fondre_le_document(&ancetre, ici, &la_bas)
}

/// Le tableau de ce nom dans ce document.
fn tableau<'a>(projet: &'a Project, id: &str) -> Option<&'a Board> {
    projet.boards.iter().find(|b| b.id == id)
}

impl FusionDuDocument {
    fn liste<T: Nomme>(
        &mut self,
        f: FusionDeListe<T>,
        edition: impl Fn(Slot<T>) -> Edit,
    ) -> Vec<T> {
        self.a_envoyer.extend(f.a_envoyer.into_iter().map(edition));
        self.conflits.extend(f.conflits.into_iter().map(|c| c.nom));
        f.liste
    }

    fn valeur<T>(
        &mut self,
        f: FusionDeValeur<T>,
        nom: &str,
        edition: impl Fn(Whole<T>) -> Edit,
    ) -> T {
        self.a_envoyer.extend(f.a_envoyer.map(edition));
        if f.conflit.is_some() {
            self.conflits.push(nom.to_string());
        }
        f.valeur
    }

    fn fondre_les_tableaux(&mut self, ancetre: &Project, ici: &Project, la_bas: &Project) {
        // Un tableau des deux côtés est pris tel que là-bas pour la liste des tableaux (il n'y
        // change rien), puis fusionné liste par liste.
        let tel_la_bas = |b: &Board| match (tableau(ici, &b.id), tableau(la_bas, &b.id)) {
            (Some(_), Some(r)) => r.clone(),
            _ => b.clone(),
        };
        let a: Vec<Board> = ancetre.boards.iter().map(tel_la_bas).collect();
        let l: Vec<Board> = ici.boards.iter().map(tel_la_bas).collect();
        let f = fondre_la_liste(&a, &l, &la_bas.boards);
        self.projet.boards = self.liste(f, |slot| Edit::Board { slot });
        for l in &ici.boards {
            let Some(r) = tableau(la_bas, &l.id) else {
                continue;
            };
            // Sans ancêtre (posé des deux côtés sous le même nom) : un tableau vide.
            let a = tableau(ancetre, &l.id).cloned().unwrap_or_else(|| vide(r));
            self.fondre_le_tableau(&a, l, r);
        }
    }

    fn fondre_le_tableau(&mut self, a: &Board, l: &Board, r: &Board) {
        let Some(rang) = self.projet.boards.iter().position(|b| b.id == r.id) else {
            return;
        };
        let mut t = self.projet.boards[rang].clone();
        let id = || r.id.clone();
        let images = fondre_la_liste(&a.images, &l.images, &r.images);
        t.images = self.liste(images, |slot| Edit::Image { board: id(), slot });
        let notes = fondre_la_liste(&a.annotations, &l.annotations, &r.annotations);
        t.annotations = self.liste(notes, |slot| Edit::Annotation { board: id(), slot });
        let dossiers = fondre_la_liste(&a.folders, &l.folders, &r.folders);
        t.folders = self.liste(dossiers, |slot| Edit::Folder { board: id(), slot });
        let cases = fondre_la_liste(&a.panels, &l.panels, &r.panels);
        t.panels = self.liste(cases, |slot| Edit::Panel { board: id(), slot });
        let histoires = fondre_la_liste(&a.storyboards, &l.storyboards, &r.storyboards);
        t.storyboards = self.liste(histoires, |slot| Edit::Storyboard { board: id(), slot });
        let nom = fondre_la_valeur(&a.name, &l.name, &r.name);
        t.name = self.valeur(nom, "le nom d'un tableau", |whole| Edit::BoardName {
            board: id(),
            whole,
        });
        let zones = fondre_la_valeur(&a.zones, &l.zones, &r.zones);
        t.zones = self.valeur(zones, "les zones d'un tableau", |whole| Edit::Zones {
            board: id(),
            whole,
        });
        self.projet.boards[rang] = t;
    }
}

/// Un tableau du même nom, sans rien dedans.
fn vide(r: &Board) -> Board {
    let mut t = r.clone();
    t.images.clear();
    t.annotations.clear();
    t.folders.clear();
    t.panels.clear();
    t.storyboards.clear();
    t.zones.clear();
    t
}

#[cfg(test)]
#[path = "fusion_projet_tests.rs"]
mod tests;
