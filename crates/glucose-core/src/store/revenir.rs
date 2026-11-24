//! **REVENIR-1, dans le document** (fiche 146 § 4 ; fiche 147 § 6) : un document partagé revient
//! dans sa salle après un arrêt net ou un « Quitter », sans rien perdre d'un côté ni de l'autre.
//!
//! Trois temps. **Le document d'ici devient la salle** — hors partage, donc rien ne repart : les
//! éditions qui le mènent là sont celles de la fusion prise comme un diff (l'ancêtre est ici, et
//! « ici » est la salle). **On participe** au numéro où la salle en est. **Ce qui n'a changé qu'ici
//! devient mon geste**, qui part comme les autres et se défait par `Ctrl+Z`. Le document montré est
//! alors exactement *la salle, plus mes gestes en attente* (CLIENT-1).

use super::journal::fusion_projet::{fondre_le_document, rebatir};
use super::journal::partage::GestePartage;
use super::journal::Transaction;
use super::Store;
use crate::types::Project;

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
        let vers_la_salle = fondre_le_document(&self.project, &la_bas, &self.project).a_envoyer;
        self.appliquer_comme_un_geste(Transaction {
            edits: vers_la_salle,
        });
        self.resync_next_id();
        let entre = self.rejoindre(marque, ecrivain, salle.n);
        self.appliquer_comme_un_geste(Transaction {
            edits: fusion.a_envoyer,
        });
        entre.then_some(fusion.conflits)
    }
}
