//! **Ouvrir un document** (`Ctrl+O`, RECENTS-1, fiche 174) : la palette, dont les rangées sont les
//! documents de [`crate::persist::recents`] — les récents, les introuvables en rouge, ceux qu'il n'a
//! jamais ouverts — et, tout en bas, « Parcourir ce PC… ». La même palette, une autre source, comme
//! `Ctrl+F` (fiche 89).
//!
//! **Au téléphone** (RECENTS-2), ses rangées sont les documents que Glucose range (DOCUMENTS-1),
//! chacun avec son âge ; le clavier attend qu'on touche le champ, l'appui long sur un document
//! demande ce qu'on en fait (DOCUMENTS-2), et « Annuler » ferme, en bas.

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

/// Ce qu'elle dit au téléphone : l'appui long ne se devine pas (DOCUMENTS-2).
pub(super) const AIDE_AU_DOIGT: &str = "Touche le champ pour chercher. Un appui long sur un \
    document : le renommer, le dupliquer, le supprimer.";

impl GlucoseApp {
    /// **Compose la liste des documents**, avec ce que la fouille a trouvé jusqu'ici. La liste des
    /// connus se corrige si elle a appris quelque chose — un document retrouvé à sa nouvelle
    /// place, un numéro qui manquait. Composer ne relance pas la fouille : chaque trouvaille
    /// recompose, et une fouille relancée à chaque fois ne finirait jamais.
    pub(super) fn composer_les_documents(&mut self) -> Vec<Document> {
        if self.ui.documents_ranges {
            return recents::ranges(&self.disque.documents);
        }
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

    /// **Ce que la palette dit de ses documents** : ce qui vient d'arriver à l'un d'eux, puis
    /// combien, et si la fouille cherche encore.
    pub(super) fn compte_des_documents(&self, etat: &Etat) -> String {
        let trouves = trouves_pour(&etat.filtre, &etat.documents, &self.lecteur()).len();
        let cherche = match self.disque.fouille.en_cours() {
            true => " La recherche sur ce PC continue…",
            false => "",
        };
        let aide = match self.ui.documents_ranges {
            true => AIDE_AU_DOIGT,
            false => AIDE,
        };
        let compte = match (etat.filtre.trim().is_empty(), trouves) {
            (true, 0) if etat.documents.is_empty() => "Aucun document pour l'instant.".into(),
            (true, _) => format!("{aide}{cherche}"),
            (false, 0) => format!("Aucun document pour « {} ».{cherche}", etat.filtre.trim()),
            (false, 1) => format!("1 document.{cherche}"),
            (false, n) => format!("{n} documents.{cherche}"),
        };
        etat.note
            .as_ref()
            .map_or(compte.clone(), |n| format!("{n}\n{compte}"))
    }

    /// **La liste des documents**, et en tête ce qui vient d'arriver à l'un d'eux — renommé,
    /// dupliqué, supprimé (DOCUMENTS-2) : la liste revient après chaque geste, et le dit là où
    /// l'on regarde, plutôt que dans un message posé par-dessus.
    pub(crate) fn montrer_les_documents(&mut self, note: Option<String>) {
        self.poser_une_palette(Etat {
            but: super::But::Ouvrir,
            note,
            ..Etat::default()
        });
    }

    /// **Le document sous le doigt** d'un appui long, au téléphone : celui dont on demande ce
    /// qu'on en fait (DOCUMENTS-2). Au bureau, aucun — supprimer y est définitif, hors de la
    /// corbeille du système.
    pub(crate) fn document_a_gerer(&self, etat: &Etat, rang: usize) -> Option<PathBuf> {
        let documents = trouves_pour(&etat.filtre, &etat.documents, &self.lecteur());
        let document = documents.get(rang).filter(|_| self.ui.documents_ranges)?;
        Some(document.chemin.clone())
    }

    /// **Le dossier d'un document, tel qu'on le lit** : `~/…` sous le dossier personnel. Au
    /// téléphone, aucun : ses documents vivent dans le dossier privé de Glucose, que personne n'y
    /// voit (DOCUMENTS-1) — ce chemin ne s'écrit pas, et ne filtre rien.
    pub(super) fn lecteur(&self) -> impl Fn(&Path) -> Option<String> {
        let personnel = (!self.ui.documents_ranges).then(personnel);
        move |chemin| (personnel.as_ref()).map(|p| recents::dossier_court(chemin, p.as_deref()))
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
        let documents = trouves_pour(&etat.filtre, &etat.documents, &self.lecteur());
        let Some(document) = documents.get(i).copied() else {
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
pub(super) fn trouves_pour<'a>(
    filtre: &str,
    documents: &'a [Document],
    dossier_de: &impl Fn(&Path) -> Option<String>,
) -> Vec<&'a Document> {
    let mut v: Vec<(u8, usize, &Document)> = documents
        .iter()
        .enumerate()
        .filter_map(|(rang, d)| {
            let nom = nom_de(&d.chemin);
            let dossier = dossier_de(&d.chemin);
            let par_le_dossier = dossier
                .and_then(|d| rang_de(filtre, &d, ""))
                .filter(|&q| q < 3);
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
pub(super) fn rangee(document: &Document, dossier: Option<String>) -> (String, Precision) {
    let precision = Precision {
        texte: recents::precision(document, dossier, std::time::SystemTime::now()),
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
