//! **Ce que la sélection offre sous X11** : chaque cible (un atome, presque toujours un type
//! MIME), et les octets qui la remplissent, fabriqués quand on la demande.
//!
//! | forme | cibles | ce qu'on y lit |
//! |---|---|---|
//! | le lot | `application/x-glucose-lot` | le `.glucose` tel quel : la longueur d'une propriété X est exacte, rien ne l'enveloppe |
//! | le texte | `UTF8_STRING`, `text/plain;charset=utf-8`, `text/plain` | de l'UTF-8 — ce que Chromium offre aussi sous ces trois noms |
//! | l'image seule | `image/png` | le PNG, ce que Chromium lit d'abord (`ClipboardOzone::ReadPng`) |
//! | les fichiers | `text/uri-list`, `x-special/gnome-copied-files` | des URI `file://` : la première pour Chromium (`ReadFilenames`) et KDE, la seconde pour les gestionnaires de fichiers de GNOME |
//!
//! Aucun HTML : il porterait le lot en base64 dans tout ce qui colle du HTML (un courrier, un
//! traitement de texte) — des mégaoctets illisibles. Le lot a son type à lui, que seul Glucose
//! demande.

use super::Atomes;
use crate::interactions::clipboard::{Forme, Formes, Pour};
use crate::plateforme::copies::EnFichiers;
use std::borrow::Cow;
use std::fmt::Write as _;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use x11rb::protocol::xproto::Atom;

/// Ce qui remplit une cible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Rendu {
    Lot,
    Texte,
    Png,
    ListeDUri,
    FichiersGnome,
}

/// La sélection confiée au presse-papiers, et ses fichiers, écrits quand on les demande.
pub(super) struct Offre {
    formes: Formes,
    fichiers: EnFichiers,
    cibles: Vec<(Atom, Rendu)>,
}

impl Offre {
    /// **Les formes qui partent ([`Formes::offertes`]), dites dans les cibles de X11.** Sur le
    /// fil de la fenêtre : c'est lui qui efface le dossier de la copie d'avant.
    pub fn nouvelle(formes: Formes, a: &Atomes) -> Self {
        let mut cibles = Vec::new();
        for forme in formes.offertes(Pour::PressePapiers) {
            match forme {
                Forme::Lot => cibles.push((a.LOT, Rendu::Lot)),
                Forme::Texte => {
                    cibles.extend([a.UTF8_STRING, a.TEXTE_UTF8, a.TEXTE].map(|c| (c, Rendu::Texte)))
                }
                Forme::Image => cibles.push((a.PNG, Rendu::Png)),
                Forme::Fichiers => {
                    cibles.push((a.LISTE_D_URI, Rendu::ListeDUri));
                    cibles.push((a.FICHIERS_GNOME, Rendu::FichiersGnome));
                }
            }
        }
        Self {
            fichiers: EnFichiers::pour(formes.date, Pour::PressePapiers),
            formes,
            cibles,
        }
    }

    /// Les cibles offertes, dans l'ordre de préférence.
    pub fn cibles(&self) -> impl Iterator<Item = Atom> + '_ {
        self.cibles.iter().map(|(c, _)| *c)
    }

    /// Ce qui remplit cette cible, si elle est offerte.
    pub fn rendu(&self, cible: Atom) -> Option<Rendu> {
        self.cibles
            .iter()
            .find(|(c, _)| *c == cible)
            .map(|(_, r)| *r)
    }

    /// Le lot, tel quel.
    pub fn lot(&self) -> &[u8] {
        &self.formes.lot
    }

    /// **Les octets d'une cible, fabriqués maintenant** — les fichiers s'écrivent ici, au premier
    /// collage qui les veut. Le lot et l'image se prêtent sans copie.
    pub fn octets(&self, rendu: Rendu) -> Result<Cow<'_, [u8]>, String> {
        Ok(match rendu {
            Rendu::Lot => Cow::Borrowed(&self.formes.lot),
            Rendu::Texte => Cow::Borrowed(self.formes.texte.as_deref().unwrap_or("").as_bytes()),
            Rendu::Png => match &self.formes.image {
                Some(image) => Cow::Borrowed(&image.png),
                None => return Err("aucune image seule".into()),
            },
            Rendu::ListeDUri => Cow::Owned(liste_d_uri(&self.ecrits()?).into_bytes()),
            Rendu::FichiersGnome => Cow::Owned(fichiers_gnome(&self.ecrits()?).into_bytes()),
        })
    }

    fn ecrits(&self) -> Result<Vec<PathBuf>, String> {
        self.fichiers.ecrits(&self.formes.fichiers)
    }
}

/// **Un chemin en URI `file://`** (RFC 8089) : tout octet hors des caractères non réservés
/// (RFC 3986 § 2.3) et de la barre se code `%XX` — les espaces, les accents en UTF-8, `#`, `%`,
/// `?`… Chromium (`URLToPath`), GLib et Qt décodent tous `%XX`.
pub(super) fn uri_de(chemin: &Path) -> String {
    let mut uri = String::from("file://");
    for &octet in chemin.as_os_str().as_bytes() {
        if octet.is_ascii_alphanumeric() || b"-._~/".contains(&octet) {
            uri.push(char::from(octet));
        } else {
            let _ = write!(uri, "%{octet:02X}");
        }
    }
    uri
}

/// **`text/uri-list`** (RFC 2483) : une URI par ligne, chaque ligne finie par `\r\n`. Chromium
/// coupe la liste **sur `\r\n` seulement** (`URIListToFileInfos`) : séparées par `\n`, plusieurs
/// images n'y feraient qu'un seul chemin, faux.
pub(super) fn liste_d_uri(chemins: &[PathBuf]) -> String {
    chemins.iter().map(|c| uri_de(c) + "\r\n").collect()
}

/// **`x-special/gnome-copied-files`** : « copy », puis une URI par ligne, séparées par `\n` —
/// ce que Nautilus, Nemo et Caja posent et relisent pour coller des fichiers.
pub(super) fn fichiers_gnome(chemins: &[PathBuf]) -> String {
    std::iter::once("copy".to_string())
        .chain(chemins.iter().map(|c| uri_de(c)))
        .collect::<Vec<_>>()
        .join("\n")
}
