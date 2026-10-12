//! Sortir un tableau vers le monde extérieur : une image, du Markdown, ou une image vectorielle.
//!
//! # Ce module n'écrit aucun format — il les branche
//!
//! Les deux moteurs vivent dans [`glucose_core::export`], sans aucune dépendance, et sont
//! testés depuis longtemps : 334 lignes qui tournaient à vide, faute d'un geste pour les
//! appeler et d'une écriture pour les poser sur le disque. C'est exactement ce que ce module
//! ajoute, et rien d'autre.
//!
//! # Le format ne se demande pas : il se lit dans le nom du fichier
//!
//! Aucune boîte de dialogue ne demande « Markdown ou SVG ? », et il n'y a pas deux raccourcis.
//! Le sélecteur natif propose ses filtres, le système met l'extension qui va avec, et
//! [`Format::du_chemin`] la relit. Un utilisateur qui tape `carte.svg` à la main obtient du
//! SVG sans avoir eu à répondre à une question de plus.
//!
//! Cela supprime aussi le seul cas où l'application pouvait mentir : le format écrit **est**
//! celui que le nom annonce, et le message de confirmation le nomme.
//!
//! # Ce qui reste à faire ici, nommé pour ne pas le croire fait
//!
//! Le SVG sorti par le noyau porte les cartes et les flèches, pas les images ni les
//! membranes ; le Markdown porte la structure. Ce sont les moteurs qui en décident, pas ce
//! module — les compléter se fait dans [`glucose_core::export`], et se teste sans fenêtre.

use super::atomic::write_atomic;
use super::human_size;
use crate::app::GlucoseApp;
use crate::error::{DesktopError, DesktopResult};
use glucose_core::types::Board;
use std::path::{Path, PathBuf};

/// Un format de sortie.
///
/// L'énumération porte **tout** ce qui distingue un format : son extension, son nom lisible,
/// et le moteur qui le produit. Ajouter le HTML ou le PNG (défaut `I.2` de la fiche 14) est
/// donc une variante et trois lignes — jamais un `match` de plus ailleurs dans le code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Le tableau, ou la sélection, dessiné en image (EXPORT-PNG-1, [`png`]).
    Png,
    Markdown,
    Svg,
}

impl Format {
    /// Tous les formats, dans l'ordre où le sélecteur les propose. Le **premier** sert de
    /// repli quand le nom choisi ne porte aucune extension connue.
    ///
    /// L'image d'abord : c'est ce qu'on exporte le plus souvent, et le repli d'un nom sans
    /// extension.
    pub const TOUS: [Self; 3] = [Self::Png, Self::Markdown, Self::Svg];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Markdown => "md",
            Self::Svg => "svg",
        }
    }

    pub fn nom(self) -> &'static str {
        match self {
            Self::Png => "Image",
            Self::Markdown => "Markdown",
            Self::Svg => "Image vectorielle",
        }
    }

    /// Le format qu'annonce ce nom de fichier, s'il en annonce un.
    ///
    /// Insensible à la casse : Windows rend volontiers `CARTE.SVG`.
    pub fn du_chemin(chemin: &Path) -> Option<Self> {
        let extension = chemin.extension()?.to_str()?;
        Self::TOUS
            .into_iter()
            .find(|f| extension.eq_ignore_ascii_case(f.extension()))
    }

    /// Le texte de ce tableau dans ce format. C'est le seul endroit qui appelle les moteurs de
    /// texte ; une image n'en est pas un — elle se dessine ([`png`]).
    pub fn rendre(self, tableau: &Board) -> Option<String> {
        match self {
            Self::Png => None,
            Self::Markdown => Some(glucose_core::export::to_markdown(tableau)),
            Self::Svg => Some(glucose_core::export::to_svg(tableau)),
        }
    }
}

/// Le chemin et le format à écrire, l'extension complétée si elle manquait.
///
/// Le repli sur le premier format n'est jamais silencieux : le message de confirmation nomme
/// le format écrit, donc un repli qui ne convient pas se voit à l'instant où il arrive.
pub fn destination(choisi: PathBuf) -> (PathBuf, Format) {
    match Format::du_chemin(&choisi) {
        Some(format) => (choisi, format),
        None => {
            let repli = Format::TOUS[0];
            (choisi.with_extension(repli.extension()), repli)
        }
    }
}

/// Le sélecteur qui demande où exporter, chaque format en filtre.
fn la_sortie(nom: &str) -> crate::dialogue::Fichier {
    let mut dialogue = crate::dialogue::fichier();
    for format in Format::TOUS {
        dialogue = dialogue.filtre(format.nom(), &[format.extension()]);
    }
    dialogue.nom(format!("{nom}.{}", Format::TOUS[0].extension()))
}

impl GlucoseApp {
    /// Exporte le tableau courant : demande un nom ; le choix revenu, écrit, et dit ce qui a
    /// été écrit ([`Self::exporter_vers`]).
    pub fn export_board(&mut self) {
        let sortie = la_sortie(&self.document_label());
        let demande = super::choix::Demande::Exporter;
        self.demander_un_fichier(sortie, crate::dialogue::Mode::Enregistrer, demande);
    }

    /// Le choix revenu : le tableau s'écrit là, au format que son nom dit.
    pub fn exporter_vers(&mut self, choisi: PathBuf) {
        let (chemin, format) = destination(choisi);
        if format == Format::Png {
            self.exporter_en_image(chemin);
            return;
        }
        let message = match self.try_export(&chemin, format) {
            Ok(octets) => format!(
                "{} exporté en {} — {}",
                nom_du_fichier(&chemin),
                format.nom(),
                human_size(octets)
            ),
            Err(err) => err.to_string(),
        };
        self.ui.show_toast(message);
    }

    /// Rendre puis écrire, atomiquement comme un enregistrement : un plantage en pleine
    /// écriture ne laisse jamais un export à moitié fait à la place d'un précédent valide.
    fn try_export(&self, chemin: &Path, format: Format) -> DesktopResult<usize> {
        let tableau = self
            .store
            .active_board()
            .ok_or_else(|| DesktopError::SaveFailed {
                path: chemin.display().to_string(),
                reason: "aucun tableau ouvert".into(),
            })?;
        let texte = format
            .rendre(tableau)
            .ok_or_else(|| DesktopError::SaveFailed {
                path: chemin.display().to_string(),
                reason: "une image se dessine, elle ne s'écrit pas en texte".into(),
            })?;
        write_atomic(chemin, texte.as_bytes())?;
        Ok(texte.len())
    }
}

fn nom_du_fichier(chemin: &Path) -> String {
    chemin
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("export")
        .to_string()
}

pub mod png;

#[cfg(test)]
mod tests;
