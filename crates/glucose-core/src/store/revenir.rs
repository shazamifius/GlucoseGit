//! **REVENIR-1, dans le document** (fiche 146 § 4 ; fiche 147 § 6) : un document partagé revient
//! dans sa salle après un arrêt net ou un « Quitter », sans rien perdre d'un côté ni de l'autre.
//!
//! Trois temps. **Le document d'ici devient la salle** — hors partage, donc rien ne repart : les
//! éditions qui le mènent là sont celles de la fusion prise comme un diff (l'ancêtre est ici, et
//! « ici » est la salle). **On participe** au numéro où la salle en est. **Ce qui n'a changé qu'ici
//! devient mon geste**, qui part comme les autres et se défait par `Ctrl+Z`. Le document montré est
//! alors exactement *la salle, plus mes gestes en attente* (CLIENT-1).

use super::images::ECART_D_UNE_COPIE;
use super::journal::fusion_projet::{fondre_le_document, rebatir};
use super::journal::partage::GestePartage;
use super::journal::{Edit, Slot, Transaction};
use super::Store;
use crate::types::{Annotation, BoardImage, Project};

/// **Ce que la salle garde**, reçu depuis son début.
pub struct CeQueLaSalleGarde<'a> {
    pub base: &'a Project,
    /// Ses gestes après la base, avec leur numéro, dans son ordre.
    pub gestes: &'a [(u64, GestePartage)],
    /// Le numéro où elle en est.
    pub n: u64,
}

impl Store {
    /// **Revient dans la salle** : `retenu` est le dernier numéro que ce document contient, au moins.
    /// Rend le nom de chaque élément changé des deux côtés (la salle l'a emporté) — `None` si l'on
    /// n'a pas pu participer : le document garde alors la fusion, à soi seul. Jamais pendant un
    /// geste ouvert : rien ne se fait.
    pub fn revenir_dans_la_salle(
        &mut self,
        salle: CeQueLaSalleGarde,
        retenu: u64,
        marque: &str,
        ecrivain: bool,
    ) -> Option<Vec<String>> {
        if self.journal.is_open() {
            return None;
        }
        let ancetre = rebatir(salle.base, salle.gestes, retenu);
        let la_bas = rebatir(salle.base, salle.gestes, salle.n);
        let fusion = fondre_le_document(&ancetre, &self.project, &la_bas);
        let versions = versions_d_ici(&self.project, &fusion.conflits);
        let vers_la_salle = fondre_le_document(&self.project, &la_bas, &self.project).a_envoyer;
        self.appliquer_comme_un_geste(Transaction {
            edits: vers_la_salle,
        });
        self.resync_next_id();
        let entre = self.rejoindre(marque, ecrivain, salle.n);
        let mut edits = fusion.a_envoyer;
        edits.extend(self.poser_a_cote(versions, &fusion.projet));
        self.appliquer_comme_un_geste(Transaction { edits });
        entre.then_some(fusion.conflits)
    }

    /// **Les versions d'ici, posées à côté** — sous un nom neuf (celui de ma marque : rien ne s'y
    /// mêle), à la fin de leur liste dans le document fusionné, à droite du nœud que la salle a
    /// emporté, de l'écart d'un nœud dupliqué.
    fn poser_a_cote(&mut self, versions: Vec<VersionDIci>, fusionne: &Project) -> Vec<Edit> {
        let mut poses: Vec<(String, bool)> = Vec::new();
        let mut edits = Vec::new();
        for (board, version) in versions {
            let Some(b) = fusionne.boards.iter().find(|b| b.id == board) else {
                continue;
            };
            let photo = matches!(version, Version::Photo(_));
            let deja = poses
                .iter()
                .filter(|p| p.0 == board && p.1 == photo)
                .count();
            let index = deja
                + if photo {
                    b.images.len()
                } else {
                    b.annotations.len()
                };
            poses.push((board.clone(), photo));
            edits.push(match version {
                Version::Photo(mut img) => {
                    img.id = self.generate_id("img");
                    img.x += img.width + ECART_D_UNE_COPIE;
                    Edit::Image {
                        board,
                        slot: Slot::inserted(index, img),
                    }
                }
                Version::Note(mut note) => {
                    note.set_id(self.generate_id("ann"));
                    note.translate(note.bounds().width + ECART_D_UNE_COPIE, 0.0);
                    Edit::Annotation {
                        board,
                        slot: Slot::inserted(index, note),
                    }
                }
            });
        }
        edits
    }
}

/// **La version d'ici d'un nœud que la salle a emporté.**
enum Version {
    Photo(BoardImage),
    Note(Annotation),
}

/// Le tableau où elle vit, et elle.
type VersionDIci = (String, Version);

/// **Les versions d'ici des conflits** : les photos et les notes. Pas les flèches : une flèche tient
/// à ses deux bouts, sa copie ne serait pas « à côté » ; ni un nom de tableau ou du document — dits
/// seulement.
fn versions_d_ici(ici: &Project, conflits: &[String]) -> Vec<VersionDIci> {
    let mut versions = Vec::new();
    for b in &ici.boards {
        let photos = b.images.iter().filter(|i| conflits.contains(&i.id));
        versions.extend(photos.map(|i| (b.id.clone(), Version::Photo(i.clone()))));
        let notes = (b.annotations.iter())
            .filter(|a| conflits.iter().any(|c| c == a.id()))
            .filter(|a| !matches!(a, Annotation::Arrow { .. }));
        versions.extend(notes.map(|a| (b.id.clone(), Version::Note(a.clone()))));
    }
    versions
}
