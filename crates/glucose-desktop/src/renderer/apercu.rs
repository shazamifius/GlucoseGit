//! **Les aperçus sur le disque** : la vue d'ensemble de chaque image, pour qu'un document
//! s'ouvre déjà montré (ETAGES-4, fiche 32).
//!
//! # Ce que la mesure a dit
//!
//! Ouvrir le document de l'utilisateur demande de décoder ses 243 photos : 0,43 s sur quinze
//! fils avant que la dernière paraisse, seize millisecondes par mégapixel et par fil. À dix
//! mille images, dix-huit secondes de cadres vides. C'est ce que Lightroom règle par ses
//! aperçus, et ce que le disque — le troisième étage — est là pour porter.
//!
//! # Ce qu'on garde, et pourquoi cela ne coûte qu'un écran
//!
//! Pas l'image : sa **vue d'ensemble**, les niveaux de la pyramide depuis la taille qu'elle
//! prend quand le tableau entier tient dans l'écran jusqu'au pixel. Plus un document a
//! d'images, plus chacune y est petite : la somme des aperçus d'un document vaut environ **un
//! écran de pixels** — quinze mégaoctets sur l'écran de l'utilisateur —, qu'il compte cent
//! images ou dix millions. La place disque est bornée par l'écran, pas par le document, et
//! aucune borne n'a eu à être choisie.
//!
//! # Ce qu'un aperçu devient en mémoire
//!
//! Une pyramide **partielle** : les niveaux gardés sont tenus, les plus grands, jamais
//! décodés, se disent **perdus**. L'écran qui les voudra les fera redécoder depuis le fichier
//! — exactement le chemin d'une page que Windows a jetée (ETAGES-1). Rien de neuf à inventer :
//! le disque rend ce que la mémoire aurait rendu, en moins bien, tout de suite.

use super::photo::dimensions_au_rang;
use glucose_core::hash::{hex_of, sha256};
use std::path::{Path, PathBuf};

/// La vue d'ensemble d'une image, telle qu'elle se garde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Apercu {
    /// Les dimensions de l'image entière : elles disent celles de tous les niveaux.
    pub natives: (u32, u32),
    pub opaque: bool,
    /// Le rang du premier niveau gardé — la vue d'ensemble.
    pub rang: usize,
    /// Les octets des niveaux, de ce rang au dernier.
    pub niveaux: Vec<Vec<u8>>,
}

/// La signature d'un aperçu : un autre fichier dans le dossier ne se lit pas comme tel.
const MAGIE: &[u8; 8] = b"GLAPERCU";
/// Un aperçu d'une autre version ne se lit pas : il se refait.
const VERSION: u8 = 1;
/// Magie, version, opacité, rang, largeur, hauteur.
const ENTETE: usize = 8 + 1 + 1 + 1 + 4 + 4;

/// **Le fichier de l'aperçu de cette source, telle qu'elle est maintenant.**
///
/// Sa taille et sa date entrent dans le nom : une source modifiée a un autre aperçu, et
/// l'ancien ne se relit jamais — aucune invalidation à tenir.
pub fn chemin(dossier: &Path, src: &str) -> Option<PathBuf> {
    let meta = std::fs::metadata(src).ok()?;
    let date = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    let cle = format!("{src}\0{}\0{date}", meta.len());
    Some(dossier.join(format!("{}.apercu", hex_of(&sha256(cle.as_bytes())))))
}

/// Écrit un aperçu, d'un bloc : un fichier à moitié écrit ne doit jamais se lire.
pub fn ecrire(chemin: &Path, a: &Apercu) -> std::io::Result<()> {
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier)?;
    }
    let mut octets = Vec::with_capacity(ENTETE + a.niveaux.iter().map(Vec::len).sum::<usize>());
    octets.extend_from_slice(MAGIE);
    octets.push(VERSION);
    octets.push(u8::from(a.opaque));
    octets.push(u8::try_from(a.rang).unwrap_or(u8::MAX));
    octets.extend_from_slice(&a.natives.0.to_le_bytes());
    octets.extend_from_slice(&a.natives.1.to_le_bytes());
    for niveau in &a.niveaux {
        octets.extend_from_slice(niveau);
    }
    let provisoire = chemin.with_extension("ecriture");
    std::fs::write(&provisoire, &octets)?;
    std::fs::rename(&provisoire, chemin)
}

/// **Lit un aperçu**, ou rien s'il manque ou ne se tient pas : chaque niveau doit avoir
/// exactement les octets que ses dimensions demandent, et le dernier doit être le pixel.
pub fn lire(chemin: &Path) -> Option<Apercu> {
    let octets = std::fs::read(chemin).ok()?;
    let entete = octets.get(..ENTETE)?;
    if &entete[..8] != MAGIE || entete[8] != VERSION {
        return None;
    }
    let lire_u32 = |i: usize| Some(u32::from_le_bytes(entete.get(i..i + 4)?.try_into().ok()?));
    let natives = (lire_u32(11)?, lire_u32(15)?);
    let rang = usize::from(entete[10]);
    let mut reste = &octets[ENTETE..];
    let mut niveaux = Vec::new();
    let mut k = rang;
    loop {
        let (l, h) = dimensions_au_rang(natives, k)?;
        let taille = (l as usize).checked_mul(h as usize)?.checked_mul(4)?;
        let (niveau, suite) = (reste.get(..taille)?, reste.get(taille..)?);
        niveaux.push(niveau.to_vec());
        reste = suite;
        if l <= 1 && h <= 1 {
            break;
        }
        k += 1;
    }
    reste.is_empty().then_some(Apercu {
        natives,
        opaque: entete[9] != 0,
        rang,
        niveaux,
    })
}

#[cfg(test)]
mod tests;
