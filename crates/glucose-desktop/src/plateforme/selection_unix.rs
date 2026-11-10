//! **Ce que la sélection offre sous Linux, en X11 comme en Wayland** : chaque type MIME, et les
//! octets qui le remplissent, fabriqués quand une autre application le demande (COPIER-1,
//! fiche 61 ; Wayland, fiche 100).
//!
//! | forme | types | ce qu'on y lit |
//! |---|---|---|
//! | le lot | `application/x-glucose-lot` | le `.glucose` tel quel : la longueur d'un transfert est exacte, rien ne l'enveloppe |
//! | le texte | `UTF8_STRING`, `text/plain;charset=utf-8`, `text/plain` | de l'UTF-8 — ce que Chromium offre aussi sous ces trois noms, sous X11 comme sous Wayland |
//! | l'image seule | `image/png` | le PNG, ce que Chromium lit d'abord (`ClipboardOzone::ReadPng`) |
//! | les fichiers | `text/uri-list`, `x-special/gnome-copied-files` | des URI `file://` : la première pour Chromium (`ReadFilenames`) et KDE, la seconde pour les gestionnaires de fichiers de GNOME |
//!
//! Aucun HTML : il porterait le lot en base64 dans tout ce qui colle du HTML (un courrier, un
//! traitement de texte) — des mégaoctets illisibles. Le lot a son type à lui, que seul Glucose
//! demande.
//!
//! **Une seule table pour les deux systèmes.** X11 nomme ses cibles par des atomes, Wayland par les
//! types eux-mêmes : chacun traduit les noms d'ici ([`Offre::nouvelle`] reçoit la traduction), et
//! aucun ne peut offrir une forme que l'autre ne connaîtrait pas.

use crate::interactions::clipboard::{Forme, Formes, Pour};
use crate::plateforme::copies::EnFichiers;
use std::borrow::Cow;
use std::fmt::Write as _;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Le type du lot : seul un Glucose le demande.
pub(super) const LOT: &str = "application/x-glucose-lot";
/// Le texte en UTF-8, sous le nom que X11 lui a donné — que Chromium offre aussi sous Wayland.
pub(super) const UTF8_STRING: &str = "UTF8_STRING";
/// Le texte en UTF-8, sous son type.
pub(super) const TEXTE_UTF8: &str = "text/plain;charset=utf-8";
/// Le texte sans jeu de caractères dit.
pub(super) const TEXTE: &str = "text/plain";
/// Les noms du texte, de l'UTF-8 sûr au plus vague — l'ordre où l'on préfère les lire.
pub(super) const TEXTES: [&str; 3] = [UTF8_STRING, TEXTE_UTF8, TEXTE];
/// L'image seule.
pub(super) const PNG: &str = "image/png";
/// Les fichiers, pour Chromium et KDE.
pub(super) const LISTE_D_URI: &str = "text/uri-list";
/// Les fichiers, pour les gestionnaires de fichiers de GNOME.
pub(super) const FICHIERS_GNOME: &str = "x-special/gnome-copied-files";

/// Ce qui remplit un type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Rendu {
    Lot,
    Texte,
    Png,
    ListeDUri,
    FichiersGnome,
}

/// **Les types d'une forme, et ce qui les remplit** — la table que les deux systèmes traduisent.
fn types_de(forme: Forme) -> Vec<(&'static str, Rendu)> {
    match forme {
        Forme::Lot => vec![(LOT, Rendu::Lot)],
        Forme::Texte => TEXTES.iter().map(|t| (*t, Rendu::Texte)).collect(),
        Forme::Image => vec![(PNG, Rendu::Png)],
        Forme::Fichiers => vec![
            (LISTE_D_URI, Rendu::ListeDUri),
            (FICHIERS_GNOME, Rendu::FichiersGnome),
        ],
    }
}

/// La sélection confiée au presse-papiers, et ses fichiers, écrits quand on les demande. `C` est
/// la façon dont le système nomme un type : un atome sous X11, le type lui-même sous Wayland.
pub(super) struct Offre<C> {
    formes: Formes,
    fichiers: EnFichiers,
    cibles: Vec<(C, Rendu)>,
}

impl<C: Clone + PartialEq> Offre<C> {
    /// **Les formes qui partent ([`Formes::offertes`]), dites dans les noms du système** — pour le
    /// presse-papiers ou pour un glisser, où elles diffèrent. Sur le fil qui copie : c'est lui qui
    /// efface le dossier de la copie d'avant.
    pub fn nouvelle(formes: Formes, pour: Pour, nommer: impl Fn(&'static str) -> C) -> Self {
        let cibles = formes
            .offertes(pour)
            .into_iter()
            .flat_map(types_de)
            .map(|(nom, rendu)| (nommer(nom), rendu))
            .collect();
        Self {
            fichiers: EnFichiers::pour(formes.date, pour),
            formes,
            cibles,
        }
    }

    /// **Un texte seul** : ce que `Ctrl+C` copie d'un texte qu'on écrit — ses mots, et rien
    /// d'autre ; Discord, un navigateur, un terminal les collent.
    pub fn du_texte(texte: String, nommer: impl Fn(&'static str) -> C) -> Self {
        let formes = Formes {
            lot: Vec::new(),
            texte: Some(texte),
            image: None,
            fichiers: Vec::new(),
            date: 0,
        };
        let cibles = types_de(Forme::Texte)
            .into_iter()
            .map(|(nom, rendu)| (nommer(nom), rendu))
            .collect();
        Self {
            fichiers: EnFichiers::pour(0, Pour::PressePapiers),
            formes,
            cibles,
        }
    }

    /// Les cibles offertes, dans l'ordre de préférence.
    pub fn cibles(&self) -> impl Iterator<Item = C> + '_ {
        self.cibles.iter().map(|(c, _)| c.clone())
    }

    /// Ce qui remplit cette cible, si elle est offerte.
    pub fn rendu(&self, cible: &C) -> Option<Rendu> {
        self.cibles
            .iter()
            .find(|(c, _)| c == cible)
            .map(|(_, r)| *r)
    }

    /// Le lot, tel quel — vide pour un texte seul.
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

/// **Le chemin d'une URI `file://`** — l'inverse exact de [`uri_de`] : chaque `%XX` redevient son
/// octet. Une URI d'un autre hôte (`file://machine/…`) ne désigne pas un fichier d'ici : rien.
pub(super) fn chemin_de(uri: &str) -> Option<PathBuf> {
    let reste = uri.strip_prefix("file://")?;
    let reste = reste.strip_prefix("localhost").unwrap_or(reste);
    if !reste.starts_with('/') {
        return None;
    }
    let mut octets = Vec::with_capacity(reste.len());
    let mut i = 0;
    let brut = reste.as_bytes();
    while i < brut.len() {
        let hex = |o: u8| char::from(o).to_digit(16);
        match (brut[i], brut.get(i + 1).copied(), brut.get(i + 2).copied()) {
            (b'%', Some(a), Some(b)) if hex(a).is_some() && hex(b).is_some() => {
                octets.push((hex(a)? * 16 + hex(b)?) as u8);
                i += 3;
            }
            (o, ..) => {
                octets.push(o);
                i += 1;
            }
        }
    }
    Some(PathBuf::from(std::ffi::OsStr::from_bytes(&octets)))
}

/// **`text/uri-list`** (RFC 2483) : une URI par ligne, chaque ligne finie par `\r\n`. Chromium
/// coupe la liste **sur `\r\n` seulement** (`URIListToFileInfos`) : séparées par `\n`, plusieurs
/// images n'y feraient qu'un seul chemin, faux.
pub(super) fn liste_d_uri(chemins: &[PathBuf]) -> String {
    chemins.iter().map(|c| uri_de(c) + "\r\n").collect()
}

/// **Ce qu'une `text/uri-list` désigne** : les fichiers d'ici, puis les adresses du web. Les
/// lignes de commentaire (`#`) et les autres schémas ne désignent rien qu'on sache poser. On lit
/// `\n` comme `\r\n` : la RFC dit l'un, des programmes écrivent l'autre.
pub(super) fn lire_la_liste(liste: &str) -> (Vec<PathBuf>, Vec<String>) {
    let (mut chemins, mut adresses) = (Vec::new(), Vec::new());
    for ligne in liste.lines().map(str::trim) {
        if ligne.starts_with("http://") || ligne.starts_with("https://") {
            adresses.push(ligne.to_string());
        } else if let Some(chemin) = chemin_de(ligne) {
            chemins.push(chemin);
        }
    }
    (chemins, adresses)
}

/// **`x-special/gnome-copied-files`** : « copy », puis une URI par ligne, séparées par `\n` —
/// ce que Nautilus, Nemo et Caja posent et relisent pour coller des fichiers.
pub(super) fn fichiers_gnome(chemins: &[PathBuf]) -> String {
    std::iter::once("copy".to_string())
        .chain(chemins.iter().map(|c| uri_de(c)))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Une URI relue rend le chemin écrit, octet pour octet** : espaces, accents, `%`, `#`, et un
    /// octet qui n'est pas de l'UTF-8 — ce qu'un nom de fichier sous Linux peut porter.
    #[test]
    fn test_un_chemin_traverse_son_uri_intact() {
        let brut = b"/tmp/Mes photos/\xc3\xa9t\xc3\xa9 100% #1 ?\xff.png";
        let chemin = PathBuf::from(std::ffi::OsStr::from_bytes(brut));
        let uri = uri_de(&chemin);
        assert!(uri.is_ascii() && !uri.contains(' '), "{uri}");
        assert_eq!(chemin_de(&uri), Some(chemin));
        assert_eq!(chemin_de("file://localhost/a%20b"), Some("/a b".into()));
        assert_eq!(chemin_de("file://ailleurs/a"), None, "un autre hôte");
        assert_eq!(chemin_de("https://x/a"), None);
    }

    /// **Une liste mêlée se trie** : les fichiers d'ici, les adresses du web, et rien d'autre —
    /// que les lignes finissent par `\r\n` (la RFC) ou par `\n` (des programmes).
    #[test]
    fn test_une_liste_d_uri_se_lit_en_fichiers_et_en_adresses() {
        let liste = "# un commentaire\r\nfile:///a/b%20c.png\r\nhttps://ex.org/i.jpg\nftp://non\n\
                     file:///d.txt";
        let (chemins, adresses) = lire_la_liste(liste);
        assert_eq!(
            chemins,
            vec![PathBuf::from("/a/b c.png"), PathBuf::from("/d.txt")]
        );
        assert_eq!(adresses, vec!["https://ex.org/i.jpg".to_string()]);
        let ecrite = liste_d_uri(&chemins);
        assert_eq!(lire_la_liste(&ecrite).0, chemins, "l'aller-retour");
    }

    /// **Un texte seul n'offre que ses mots**, sous les trois noms du texte : ni lot vide, ni
    /// fichiers.
    #[test]
    fn test_un_texte_seul_n_offre_que_le_texte() {
        let offre = Offre::du_texte("bonjour".into(), str::to_string);
        let cibles: Vec<String> = offre.cibles().collect();
        assert_eq!(cibles, TEXTES.map(str::to_string).to_vec());
        assert_eq!(
            offre.octets(Rendu::Texte).unwrap().as_ref(),
            b"bonjour".as_slice()
        );
        assert_eq!(offre.rendu(&LOT.to_string()), None);
    }
}
