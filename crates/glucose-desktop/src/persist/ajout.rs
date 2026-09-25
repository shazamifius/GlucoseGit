//! **Ajouter un document dans de nouveaux onglets** (BOARDS-2).
//!
//! Un geste à part de `Ctrl+O`, qui reste « ouvrir » : on l'appelle du menu d'un onglet, ou
//! en lâchant un document sur la barre d'onglets. C'est le `Fichier › Ajouter` de Blender —
//! une copie dans le document courant, reliée à rien —, et ce que Figma fait d'un `.fig`
//! réimporté : son présent, sans son historique.
//!
//! Le noyau renomme tout et en fait **un** geste ([`glucose_core::store::Store::importer_un_document`]).
//! Ici, deux choses que le noyau ne peut pas savoir :
//!
//! * **ses images** se lisent dans leur document d'origine — une tranche vérifiée par son
//!   empreinte — jusqu'à ce que le scribe les copie dans celui-ci, au geste. Une clé déjà
//!   prise ici par d'autres octets est renommée d'après leur empreinte : deux images ne se
//!   confondent jamais, et une même image n'entre qu'une fois ;
//! * **ses cartes** se mesurent avant le geste, pour qu'il porte déjà leur hauteur.

use super::objets::Source;
use crate::app::GlucoseApp;
use crate::error::{DesktopError, DesktopResult};
use glucose_core::persist::histoire::{self, Ouvert};
use glucose_core::persist::tauri;
use glucose_core::types::Project;
use std::io::Read;
use std::path::Path;

impl GlucoseApp {
    /// Le geste du menu : choisir un document, puis l'ajouter.
    pub(crate) fn choisir_un_document_a_ajouter(&mut self) {
        if let Some(chemin) = self.sous_un_dialogue(super::commands::pick_open_path) {
            self.ajouter_un_document(&chemin);
        }
    }

    /// **Ajoute ce document dans de nouveaux onglets**, en un geste, et le dit.
    pub(crate) fn ajouter_un_document(&mut self, chemin: &Path) {
        let nom = chemin.file_stem().map_or_else(
            || "Document".to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let message = match self.lire_un_document_a_ajouter(chemin) {
            Ok((mut projet, manquantes)) => {
                let (typo, math) = (&self.renderer.typography, &self.renderer.math);
                crate::interactions::resize::ajuster_les_cartes(&mut projet.boards, typo, math);
                let onglets = self.store.importer_un_document(projet, &nom);
                let mut m = format!("« {nom} » ajouté — {} onglet(s)", onglets.len());
                if manquantes > 0 {
                    m.push_str(&format!(
                        ", {manquantes} image(s) introuvable(s) sur cette machine"
                    ));
                }
                m + " — Ctrl+Z pour le retirer"
            }
            Err(e) => format!("« {nom} » n'a pas pu s'ajouter : {e}"),
        };
        self.ui.show_toast(message);
        self.mark_dirty();
    }

    /// Lit le présent d'un document — de Glucose Rust ou de Glucose Tauri — et place ses
    /// images. Rend aussi le nombre d'images dont les octets manquent.
    fn lire_un_document_a_ajouter(&mut self, chemin: &Path) -> DesktopResult<(Project, usize)> {
        let ouvrir_err = |e: std::io::Error| DesktopError::OpenFailed {
            path: chemin.display().to_string(),
            reason: e.to_string(),
        };
        let mut f = std::fs::File::open(chemin).map_err(ouvrir_err)?;
        let mut tete = [0u8; 16];
        let lus = f.read(&mut tete).map_err(ouvrir_err)?;
        if tauri::reconnaitre(&tete[..lus]).is_some() {
            let octets = std::fs::read(chemin).map_err(ouvrir_err)?;
            let lu = tauri::lire(&octets).map_err(DesktopError::from)?;
            let (projet, importe) = self.placer_un_document_tauri(chemin, lu);
            return Ok((projet, importe.manquantes));
        }
        // Lu sans verrou d'écriture : un document qu'une autre fenêtre tient s'ajoute aussi,
        // et le document courant peut s'ajouter à lui-même.
        let f = std::fs::File::open(chemin).map_err(ouvrir_err)?;
        let ouvert = histoire::ouvrir(&mut std::io::BufReader::new(f))?;
        let mut projet = ouvert.projet.clone();
        self.placer_les_images(&mut projet, &ouvert, chemin);
        Ok((projet, 0))
    }

    /// **Chaque image se lira dans son document d'origine** jusqu'à ce que le scribe la copie
    /// ici. Une clé que ce document-ci donne déjà à d'autres octets est renommée.
    ///
    /// Une image que le document d'origine n'avait pas scellée garde sa clé : elle se lit
    /// comme là-bas, par son chemin.
    fn placer_les_images(&self, projet: &mut Project, ouvert: &Ouvert, chemin: &Path) {
        for img in projet.toutes_les_images_mut() {
            let Some(cle) = img.src.clone() else {
                continue;
            };
            let Some((empreinte, t)) = ouvert
                .liens
                .get(&cle)
                .and_then(|e| ouvert.objets.get(e).map(|t| (*e, *t)))
            else {
                continue;
            };
            let cle = match self.disque.objets.source(&cle) {
                None => cle,
                Some(
                    Source::Tranche { empreinte: e, .. } | Source::Ailleurs { empreinte: e, .. },
                ) if e == empreinte => {
                    continue;
                }
                Some(_) => {
                    let court = glucose_core::hash::hex_of(&empreinte);
                    format!("{cle}#{}", &court[..16])
                }
            };
            self.disque.objets.poser(
                &cle,
                Source::Ailleurs {
                    fichier: chemin.to_path_buf(),
                    empreinte,
                    offset: t.offset,
                    longueur: t.longueur,
                },
            );
            img.src = Some(cle);
        }
    }
}

#[cfg(test)]
mod tests;
