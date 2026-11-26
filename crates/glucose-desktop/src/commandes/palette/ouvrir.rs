//! **Ouvrir un document** (`Ctrl+O`, RECENTS-1, fiche 174) : la palette, dont les rangées sont les
//! documents de [`crate::persist::recents`] — les récents, les introuvables en rouge, ceux qu'il n'a
//! jamais ouverts — et, tout en bas, « Parcourir ce PC… ». La même palette, une autre source, comme
//! `Ctrl+F` (fiche 89).

use super::{rang_de, Etat};
use crate::app::GlucoseApp;
use crate::persist::close::Ensuite;
use crate::persist::documents::nom_de;
use crate::persist::recents::{self, Document, Sorte};
use crate::ui::question::{Precision, Question, Reponse, Suite};
use std::path::{Path, PathBuf};

/// **La réponse de la dernière rangée**, « Parcourir ce PC… » : l'ouverture d'avant, au cas où.
pub(super) const PARCOURIR: Reponse = Reponse::Oui;

/// Ce qu'elle dit tant qu'on n'a rien écrit.
pub(super) const AIDE: &str = "Écris un nom ou un dossier ; Entrée ouvre le document en évidence.";

impl GlucoseApp {
    /// **Compose la liste des documents**, avec ce que la fouille a trouvé jusqu'ici. La liste des
    /// connus se corrige si elle a appris quelque chose — un document retrouvé à sa nouvelle
    /// place, un numéro qui manquait. Composer ne relance pas la fouille : chaque trouvaille
    /// recompose, et une fouille relancée à chaque fois ne finirait jamais.
    pub(super) fn composer_les_documents(&mut self) -> Vec<Document> {
        let connus = self.disque.recenseur.la_liste();
        let voisins = connus.iter().filter_map(|c| c.chemin.parent());
        let dossiers = voisins.chain(std::iter::once(self.disque.documents.as_path()));
        let mut trouves = recents::dans_ces_dossiers(dossiers);
        trouves.extend(self.disque.fouille.trouves());
        let (documents, a_recrire) = recents::composer(connus, &trouves);
        if let Some(liste) = a_recrire {
            self.disque.recenseur.ecrire_la_liste(&liste);
        }
        documents
    }

    /// **Ce que la palette dit de ses documents** : combien, et si la fouille cherche encore.
    pub(super) fn compte_des_documents(&self, etat: &Etat) -> String {
        let trouves = trouves_pour(&etat.filtre, &etat.documents).len();
        let cherche = match self.disque.fouille.en_cours() {
            true => " La recherche sur ce PC continue…",
            false => "",
        };
        match (etat.filtre.trim().is_empty(), trouves) {
            (true, _) => format!("{AIDE}{cherche}"),
            (false, 0) => format!("Aucun document pour « {} ».{cherche}", etat.filtre.trim()),
            (false, 1) => format!("1 document.{cherche}"),
            (false, n) => format!("{n} documents.{cherche}"),
        }
    }

    /// **La réponse de la palette des documents** : l'ouvrir ; introuvable, demander s'il faut
    /// l'oublier ; « Parcourir ce PC… », le sélecteur du système.
    pub(super) fn ouvrir_depuis_la_palette(&mut self, etat: &Etat, reponse: Reponse) {
        if reponse == PARCOURIR {
            self.open_project();
            return;
        }
        let Reponse::Choix(i) = reponse else {
            return;
        };
        let Some(document) = trouves_pour(&etat.filtre, &etat.documents).get(i).copied() else {
            return;
        };
        match document.sorte {
            Sorte::Introuvable => self.demander_d_oublier(&document.chemin),
            Sorte::Recent | Sorte::JamaisOuvert => {
                self.laisser_puis(Ensuite::Ouvrir(document.chemin.clone()));
            }
        }
    }

    /// **Un document introuvable** : le dire, et proposer de le retirer de la liste.
    fn demander_d_oublier(&mut self, chemin: &Path) {
        let question = Question {
            titre: format!("« {} » est introuvable", nom_de(chemin)),
            texte: format!(
                "Il n'est plus dans {} : supprimé, ou sur un disque absent. Le retirer de la liste ?",
                chemin.parent().map_or_else(String::new, |d| d.display().to_string())
            ),
            choix: vec![
                ("Retirer de la liste".into(), Reponse::Oui),
                ("Annuler".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::Oublier(chemin.to_path_buf()));
    }

    /// **La réponse** : retiré ou non, la liste revient.
    pub(crate) fn repondre_a_l_oubli(&mut self, chemin: &Path, oublier: bool) {
        if oublier {
            let mut liste = self.disque.recenseur.la_liste();
            liste.retain(|c| c.chemin != chemin);
            self.disque.recenseur.ecrire_la_liste(&liste);
        }
        self.ouvrir_la_palette(super::But::Ouvrir);
    }
}

/// **Les documents que ce filtre trouve**, les meilleurs d'abord, l'ordre de la liste ensuite —
/// par leur nom, ou par leur dossier tel qu'on le lit (`~/Documents/Clients`). Les lettres dans
/// l'ordre ne valent que pour le nom : dans un chemin, presque tout filtre les trouverait.
pub(super) fn trouves_pour<'a>(filtre: &str, documents: &'a [Document]) -> Vec<&'a Document> {
    let personnel = personnel();
    let mut v: Vec<(u8, usize, &Document)> = documents
        .iter()
        .enumerate()
        .filter_map(|(rang, d)| {
            let nom = nom_de(&d.chemin);
            let dossier = recents::dossier_court(&d.chemin, personnel.as_deref());
            let par_le_dossier = rang_de(filtre, &dossier, "").filter(|&q| q < 3);
            let qualite = rang_de(filtre, &nom, "")
                .into_iter()
                .chain(par_le_dossier)
                .min()?;
            Some((qualite, rang, d))
        })
        .collect();
    v.sort_by_key(|(qualite, rang, _)| (*qualite, *rang));
    v.into_iter().map(|(_, _, d)| d).collect()
}

/// **La rangée d'un document** : son nom, et ce qui le situe.
pub(super) fn rangee(document: &Document, personnel: Option<&Path>) -> (String, Precision) {
    let precision = Precision {
        texte: recents::precision(document, personnel, std::time::SystemTime::now()),
        introuvable: document.sorte == Sorte::Introuvable,
    };
    (nom_de(&document.chemin), precision)
}

/// Le dossier personnel, que la précision écrit `~`.
pub(super) fn personnel() -> Option<PathBuf> {
    recents::fouille::racine().or_else(|| std::env::var_os("HOME").map(PathBuf::from))
}

#[cfg(test)]
mod tests;
