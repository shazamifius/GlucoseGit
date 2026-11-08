//! **Chercher dans le document** (`Ctrl+F`, fiche 89) : la palette, dont les rangées sont les nœuds
//! que [`glucose_core::recherche::chercher`] trouve. Entrée y va : l'onglet qui le porte, le nœud
//! choisi, la caméra qui vole jusqu'à lui. `F3` va au résultat suivant sans rouvrir la palette, et
//! `Maj+F3` au précédent ; au doigt, « Chercher… » au menu du vide.
//!
//! # TROUVE-2 — aller à un résultat ne change pas le document
//!
//! On change d'onglet, on choisit le nœud, on vole : ni le journal ni le fichier n'en savent rien.
//! Un `Ctrl+Z` après une recherche défait le dernier geste, pas le voyage.

use super::Etat;
use crate::app::GlucoseApp;
use glucose_core::recherche::{passage_du, Genre, Index, Trouve};
use glucose_core::store::agents::Sorte;

/// Ce que la palette dit tant qu'on n'a rien écrit.
pub(super) const AIDE: &str =
    "Textes, post-it, membranes, photos, tableaux… Entrée y va ; F3 va au suivant.";

/// La longueur, en lettres, de la phrase qu'une rangée montre : au-delà, elle se coupe.
const LETTRES: usize = 64;

/// **Ce que la palette dit de ses résultats.**
pub(super) fn compte(etat: &Etat) -> String {
    match (etat.filtre.trim().is_empty(), etat.trouves.len()) {
        (true, _) => AIDE.to_string(),
        (false, 0) => format!("Aucun résultat pour « {} ».", etat.filtre.trim()),
        (false, 1) => "1 résultat. Entrée y va.".to_string(),
        (false, n) => format!("{n} résultats. Entrée va au premier en évidence ; F3 au suivant."),
    }
}

/// La phrase, coupée à [`LETTRES`] lettres avec « … ».
fn coupee(phrase: &str) -> String {
    let phrase = phrase.trim();
    if phrase.chars().count() <= LETTRES {
        return phrase.to_string();
    }
    let court: String = phrase.chars().take(LETTRES - 1).collect();
    format!("{}…", court.trim_end())
}

impl GlucoseApp {
    /// **Ce que cette recherche trouve dans le document tel qu'il est**, par l'index : refait si le
    /// document a changé depuis, gardé sinon.
    pub(crate) fn chercher_dans_le_document(&mut self, requete: &str) -> Vec<Trouve> {
        let cle = (self.store.identite.clone(), self.store.version);
        let a_jour = matches!(&self.ui.index_de_recherche, Some((c, _)) if *c == cle);
        if !a_jour {
            let index = Index::du(&self.store.project);
            self.ui.index_de_recherche = Some((cle, index));
        }
        match &self.ui.index_de_recherche {
            Some((_, index)) => index.chercher(requete),
            None => Vec::new(),
        }
    }

    /// **Une rangée de la recherche** : la phrase trouvée, ce qu'est le nœud, et son onglet quand le
    /// document en a plusieurs.
    pub(super) fn dire_le_trouve(&self, t: &Trouve, requete: &str) -> String {
        let phrase = coupee(&passage_du(&self.store.project, t, requete));
        let genre = t.genre.nom();
        // L'onglet qui le porte : un tableau rangé dans un dossier se dit par son onglet racine.
        let plusieurs = self.store.onglets().nth(1).is_some();
        let racine = self.store.racine_de(&t.tableau);
        match self.store.onglets().find(|(id, _, _)| *id == racine) {
            Some((_, nom, _)) if plusieurs && t.genre != Genre::Tableau => {
                format!("{phrase}   — {genre}, {nom}")
            }
            _ => format!("{phrase}   — {genre}"),
        }
    }

    /// **Va à ce résultat**, et s'en souvient pour `F3` (TROUVE-2).
    pub(crate) fn aller_au_resultat(&mut self, requete: &str, trouves: &[Trouve], rang: usize) {
        let Some(t) = trouves.get(rang) else {
            return;
        };
        self.ui.derniere_recherche = Some((requete.to_string(), rang));
        // Son tableau, même rangé dans un dossier : la pile des dossiers se refait. Un tableau qui
        // n'existe plus n'emmène nulle part.
        if self.store.project.active_board_id != t.tableau
            && self
                .store
                .try_set_active_board_id(t.tableau.clone())
                .is_err()
        {
            return;
        }
        // L'onglet a pu changer, la sélection va changer : l'image suivante les montre.
        self.mark_dirty();
        let Some(id) = &t.id else {
            return;
        };
        self.store.clear_selection();
        match t.genre {
            Genre::Noeud(Sorte::Photo) => self.store.select_image(id.clone(), false),
            Genre::Dossier => self.store.select_folder(id.clone()),
            _ => self.store.select_annotation(id.clone(), false),
        }
        self.voler_vers_le_noeud(id);
    }

    /// **Le résultat suivant (`pas` = 1) ou précédent (-1)** de la dernière recherche, cherchée à
    /// nouveau dans le document tel qu'il est. Sans recherche, ou plus rien à trouver : la palette
    /// s'ouvre.
    pub(crate) fn resultat_voisin(&mut self, pas: isize) {
        let Some((requete, rang)) = self.ui.derniere_recherche.clone() else {
            self.ouvrir_la_palette(super::But::Chercher);
            return;
        };
        let trouves = self.chercher_dans_le_document(&requete);
        if trouves.is_empty() {
            self.ouvrir_la_palette(super::But::Chercher);
            return;
        }
        let n = trouves.len() as isize;
        let suivant = (rang as isize + pas).rem_euclid(n) as usize;
        self.aller_au_resultat(&requete, &trouves, suivant);
    }
}

#[cfg(test)]
mod tests;
