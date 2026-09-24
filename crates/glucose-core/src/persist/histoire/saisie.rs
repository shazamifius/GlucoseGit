//! **Le texte en cours de frappe** : ce qu'une carte ouverte en édition contient déjà, et que
//! le document ne contient pas encore.
//!
//! # Pourquoi hors de l'histoire
//!
//! Une saisie entière est **un** geste (fiche 09 § 2.2) : elle n'entre dans l'histoire qu'une
//! fois validée. Jusque-là, un arrêt brutal perdait tout ce qui avait été tapé — une note de
//! dix minutes aussi bien qu'un mot. L'écrire dans l'histoire à chaque touche ferait de chaque
//! lettre un geste, que la Time Machine montrerait et que `Ctrl+Z` défairait une à une. Le
//! texte en cours vit donc **à côté** : un petit fichier par document, réécrit à chaque
//! changement, effacé dès que la saisie est devenue un geste.
//!
//! # À quoi une saisie s'accroche — sans horloge
//!
//! Pendant qu'on tape, rien d'autre ne change le document : seule la validation y écrit, et
//! c'est un geste. Une saisie porte donc la somme de la chaîne **juste après le dernier
//! geste** qu'elle a vu ([`super::Ouvert::dernier_geste`]). Au retour, elle ne vaut que si le
//! document n'a pas connu de geste depuis : sinon, c'est que sa validation est déjà écrite.
//! Les autres entrées — une image scellée, une vue, un jalon — ne changent pas ce qu'on
//! tapait, et ne la rendent pas caduque.
//!
//! # La forme du fichier
//!
//! ```text
//! 0..8    "SAISIE01"   la nature du fichier et sa version
//! 8..16   le point     la somme de la chaîne après le dernier geste
//!         document, tableau, annotation, texte   (chaînes préfixées de leur longueur)
//! −8..    le sceau     SHA-256 de tout ce qui précède, 8 octets
//! ```
//!
//! Un fichier dont le sceau ne correspond pas — écrit à moitié, altéré — ne se lit pas.

use super::super::bytes::{Reader, Writer};
use super::Chaine;
use crate::hash::sha256;

const SIGNATURE: [u8; 8] = *b"SAISIE01";
const SCEAU: usize = 8;

/// Ce qu'une carte en édition contient.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Saisie {
    /// Le fichier du document, tel que l'écrivain le connaissait : de quoi le rouvrir au
    /// lancement suivant.
    pub document: String,
    pub tableau: String,
    pub annotation: String,
    pub texte: String,
}

/// Les octets du fichier d'une saisie, accrochée à ce point de l'histoire.
pub fn ecrire(point: Chaine, s: &Saisie) -> Vec<u8> {
    let mut w = Writer::with_capacity(64 + s.texte.len());
    w.raw(&SIGNATURE);
    w.raw(&point.0);
    w.text(&s.document);
    w.text(&s.tableau);
    w.text(&s.annotation);
    w.text(&s.texte);
    let mut octets = w.into_bytes();
    let sceau = sha256(&octets);
    octets.extend_from_slice(&sceau[..SCEAU]);
    octets
}

/// Relit un fichier de saisie : son point, et la saisie. `None` s'il n'est pas intact.
pub fn lire(octets: &[u8]) -> Option<(Chaine, Saisie)> {
    let corps = octets.len().checked_sub(SCEAU).map(|n| &octets[..n])?;
    if sha256(corps)[..SCEAU] != octets[corps.len()..] {
        return None;
    }
    let mut r = Reader::new(corps);
    if r.take(SIGNATURE.len()).ok()? != SIGNATURE {
        return None;
    }
    let mut point = [0u8; 8];
    point.copy_from_slice(r.take(8).ok()?);
    let s = Saisie {
        document: r.text().ok()?,
        tableau: r.text().ok()?,
        annotation: r.text().ok()?,
        texte: r.text().ok()?,
    };
    r.finish().ok()?;
    Some((Chaine(point), s))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saisie() -> Saisie {
        Saisie {
            document: "C:\\projets\\thèse.glucose".into(),
            tableau: "b1".into(),
            annotation: "text-7".into(),
            texte: "une idée\nsur deux lignes — é, 漢".into(),
        }
    }

    #[test]
    fn test_une_saisie_se_relit_telle_quelle() {
        let point = Chaine([7; 8]);
        assert_eq!(lire(&ecrire(point, &saisie())), Some((point, saisie())));
    }

    /// Un octet changé n'importe où — signature, point, texte, sceau — et le fichier ne se lit
    /// plus : une saisie à moitié écrite ne devient jamais un texte faux.
    #[test]
    fn test_une_saisie_abimee_ne_se_lit_pas() {
        let octets = ecrire(Chaine([1; 8]), &saisie());
        for i in 0..octets.len() {
            let mut abime = octets.clone();
            abime[i] ^= 0x20;
            assert_eq!(lire(&abime), None, "octet {i}");
        }
        for n in 0..octets.len() {
            assert_eq!(lire(&octets[..n]), None, "coupée à {n}");
        }
    }
}
