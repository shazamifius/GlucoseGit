//! **DOCUMENTS-2 — renommer, dupliquer, supprimer un document du téléphone** (fiche 57).
//!
//! Un appui long sur un document de la liste (APPUI-1) demande ce qu'on en fait — comme dans
//! Infinite Painter et Concepts —, et la liste revient ensuite : le résultat se voit là où on
//! l'a demandé. Supprimer est définitif, comme dans Procreate : la question le dit, une fois.
//!
//! # Le document ouvert
//!
//! Son scribe y écrit, geste après geste. Le renommer passe donc par « Enregistrer sous » —
//! copier le fichier, histoire comprise, et continuer dans la copie (HISTOIRE-1) —, puis
//! effacer l'ancien ; le dupliquer attend que le scribe ait tout posé sur le disque ; le
//! supprimer ouvre d'abord un document vierge, qui le remplace sous les yeux.

use super::nom_de;
use crate::app::GlucoseApp;
use crate::ui::question::{Champ, Question, Reponse, Suite};
use glucose_core::persist::FILE_EXTENSION;
use glucose_core::text::Selection;
use std::path::{Path, PathBuf};

/// Ce qu'un nom de document ne porte pas : `/`, qu'aucun système n'accepte, et ce que Windows
/// refuse — un document part un jour vers un autre appareil.
const INTERDITS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// **La longueur d'un nom, en caractères** : un nom de fichier tient en 255 octets partout, son
/// extension et son point compris, et un caractère en prend jusqu'à quatre.
const LONGUEUR_MAX: usize = (255 - 1 - FILE_EXTENSION.len()) / 4;

/// **Le nom qu'on peut donner à un document**, tiré de ce qu'on a tapé — ou pourquoi non.
pub(crate) fn nom_valide(brut: &str) -> Result<String, String> {
    let nom = brut.trim();
    let extension = format!(".{FILE_EXTENSION}");
    let nom = nom.strip_suffix(extension.as_str()).unwrap_or(nom).trim();
    if nom.is_empty() {
        return Err("Un document a besoin d'un nom.".into());
    }
    if let Some(c) = nom.chars().find(|c| INTERDITS.contains(c)) {
        return Err(format!("Un nom ne peut pas contenir « {c} »."));
    }
    if nom.chars().any(char::is_control) {
        return Err("Un nom ne peut pas contenir de caractère invisible.".into());
    }
    // « . » et « .. » désignent des dossiers ; un point au bout, Windows le retire en silence.
    if nom.starts_with('.') || nom.ends_with('.') {
        return Err("Un nom ne commence ni ne finit par un point.".into());
    }
    if nom.chars().count() > LONGUEUR_MAX {
        return Err(format!("Un nom tient en {LONGUEUR_MAX} caractères."));
    }
    Ok(nom.to_string())
}

/// **Le premier nom libre d'une copie** : « X copie », puis « X copie 2 »…
pub(crate) fn copie_libre(chemin: &Path) -> PathBuf {
    let nom = nom_de(chemin);
    let dossier = chemin.parent().unwrap_or(Path::new(""));
    (1u32..)
        .map(|n| match n {
            1 => format!("{nom} copie.{FILE_EXTENSION}"),
            n => format!("{nom} copie {n}.{FILE_EXTENSION}"),
        })
        .map(|fichier| dossier.join(fichier))
        .find(|p| !p.exists())
        .unwrap_or_else(|| chemin.to_path_buf())
}

impl GlucoseApp {
    /// **Ce qu'on fait de ce document** : la question de l'appui long.
    pub(crate) fn gerer_un_document(&mut self, chemin: PathBuf) {
        let question = Question {
            titre: format!("« {} »", nom_de(&chemin)),
            choix: vec![
                ("Renommer…".into(), Reponse::Choix(0)),
                ("Dupliquer".into(), Reponse::Choix(1)),
                ("Supprimer…".into(), Reponse::Choix(2)),
                ("Annuler".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::Gerer(chemin));
    }

    /// La réponse à [`Self::gerer_un_document`] ; « Annuler » rend la liste.
    pub(crate) fn suivre_la_gestion(&mut self, chemin: PathBuf, reponse: Reponse) {
        match reponse {
            Reponse::Choix(0) => {
                let nom = nom_de(&chemin);
                self.demander_un_nom(chemin, nom, String::new());
            }
            Reponse::Choix(1) => self.dupliquer_le_document(&chemin),
            Reponse::Choix(2) => self.demander_la_suppression(chemin),
            _ => self.choisir_un_document(),
        }
    }

    /// Le nom à donner, dans un champ que le clavier remplit — tout sélectionné, comme partout :
    /// la première lettre tapée remplace l'ancien nom. `pourquoi` dit, au-dessus du champ, ce
    /// qui n'allait pas dans le nom d'avant.
    fn demander_un_nom(&mut self, chemin: PathBuf, nom: String, pourquoi: String) {
        let selection = Selection::all(&nom);
        let question = Question {
            titre: "Renommer".into(),
            texte: pourquoi,
            champ: Some(Champ {
                texte: nom,
                selection,
            }),
            choix: vec![
                ("Renommer".into(), Reponse::Oui),
                ("Annuler".into(), Reponse::Annuler),
            ],
            ..Default::default()
        };
        self.demander(question, Suite::Renommer(chemin));
    }

    /// **Renomme ce document** du nom tapé. Un nom refusé, ou déjà pris, se redemande avec ce
    /// qu'on avait tapé — rien n'est perdu de la saisie.
    pub(crate) fn renommer_le_document(&mut self, chemin: &Path, brut: &str) {
        let nom = match nom_valide(brut) {
            Ok(nom) => nom,
            Err(pourquoi) => {
                self.demander_un_nom(chemin.to_path_buf(), brut.to_string(), pourquoi);
                return;
            }
        };
        let cible = chemin.with_file_name(format!("{nom}.{FILE_EXTENSION}"));
        if cible == chemin {
            self.montrer_les_documents(None);
            return;
        }
        if cible.exists() {
            let pourquoi = format!("Un document s'appelle déjà « {nom} ».");
            self.demander_un_nom(chemin.to_path_buf(), nom, pourquoi);
            return;
        }
        let arrive = match self.changer_de_nom(chemin, &cible) {
            Ok(()) => format!("Renommé « {nom} »."),
            Err(e) => format!("« {} » n'a pas pu être renommé : {e}", nom_de(chemin)),
        };
        self.montrer_les_documents(Some(arrive));
    }

    /// Le fichier change de nom ; s'il est ouvert, son écriture le suit.
    fn changer_de_nom(&mut self, chemin: &Path, cible: &Path) -> Result<(), String> {
        if self.project_path.as_deref() != Some(chemin) {
            return crate::persist::atomic::renommer_sans_ecraser(chemin, cible)
                .map_err(|e| e.to_string());
        }
        self.save_to(cible.to_path_buf());
        if self.project_path.as_deref() != Some(cible) {
            return Err("sa copie n'a pas pu s'écrire".into());
        }
        std::fs::remove_file(chemin).map_err(|e| format!("l'ancien fichier reste : {e}"))
    }

    /// **Duplique ce document** sous « X copie », rangé à côté.
    pub(crate) fn dupliquer_le_document(&mut self, chemin: &Path) {
        // Le document ouvert : ce que son scribe tient encore se pose d'abord sur le disque.
        if self.project_path.as_deref() == Some(chemin) {
            self.consigner();
            if let Some(ecriture) = &self.disque.ecriture {
                let _ = ecriture.synchroniser();
            }
        }
        let cible = copie_libre(chemin);
        let arrive = match crate::persist::atomic::copier_d_un_bloc(chemin, &cible) {
            Ok(()) => format!("« {} » est rangé dans les documents.", nom_de(&cible)),
            Err(e) => format!("« {} » n'a pas pu être dupliqué : {e}", nom_de(chemin)),
        };
        self.montrer_les_documents(Some(arrive));
    }

    /// La question de la suppression : définitive, et elle le dit.
    fn demander_la_suppression(&mut self, chemin: PathBuf) {
        let question = Question {
            titre: format!("Supprimer « {} » ?", nom_de(&chemin)),
            texte: "C'est définitif : le document, son histoire et ses images disparaissent de \
                    ce téléphone."
                .into(),
            choix: vec![
                ("Supprimer".into(), Reponse::Oui),
                ("Annuler".into(), Reponse::Annuler),
            ],
            // Entrée ne détruit rien : l'évidence va à « Annuler » (POPUP-1).
            focus: 1,
            ..Default::default()
        };
        self.demander(question, Suite::Supprimer(chemin));
    }

    /// **Supprime ce document.** Celui qu'on regarde ne s'efface pas sous les yeux : un
    /// document vierge prend d'abord sa place.
    pub(crate) fn supprimer_le_document(&mut self, chemin: &Path) {
        let nom = nom_de(chemin);
        if self.project_path.as_deref() == Some(chemin) {
            self.adopter_un_document_vierge();
        }
        let arrive = if self.project_path.as_deref() == Some(chemin) {
            format!("« {nom} » est encore ouvert : il n'est pas supprimé.")
        } else {
            match std::fs::remove_file(chemin) {
                Ok(()) => format!("« {nom} » est supprimé."),
                Err(e) => format!("« {nom} » n'a pas pu être supprimé : {e}"),
            }
        };
        self.montrer_les_documents(Some(arrive));
    }
}

#[cfg(test)]
mod tests;
