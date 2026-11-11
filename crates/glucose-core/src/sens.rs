//! **SENS-1 — à combien ce texte parle de celui-là** (fiche 106).
//!
//! Sa parole du 11/11/2026 : *« une intelligence artificielle semi from scratch, ultra optimisée et
//! minimale, capable de comprendre une seule et unique chose : est-ce que ce texte a un rapport avec
//! ce texte-là, et à combien de % »*.
//!
//! # Le modèle
//!
//! `static-similarity-mrl-multilingual-v1` (sentence-transformers, licence Apache 2.0) : une table qui
//! donne à chaque morceau de mot une position dans un espace du sens, entraînée pour que deux phrases
//! semblables tombent près l'une de l'autre, dans une cinquantaine de langues dont le français. Le sens
//! d'un texte est **la moyenne de ses morceaux** ; deux textes se ressemblent comme le cosinus de leurs
//! positions. Aucun réseau à faire tourner, aucune carte graphique : une lecture de table et une
//! moyenne, sur n'importe quelle machine — le téléphone compris (sa parole : *« aucune machine
//! exclue »*).
//!
//! Glucose en garde les 128 premières dimensions (le modèle a été entraîné « en poupées russes »), en
//! octets signés avec une échelle par morceau : 15 Mo au lieu de 434. **Mesuré sur la carte de
//! Wikipédia** (fiche 106) : deux résumés du même thème, deux articles liés, deux paragraphes du même
//! article ou de la même section se reconnaissent aussi bien qu'avec le modèle entier, et mieux que par
//! les sacs de mots d'avant. La conversion : `outils/sens/convertir.py` ; le format est dit là.
//!
//! # Ce qui est écrit ici
//!
//! Le découpeur ([`decoupeur`], le WordPiece de BERT multilingue, réécrit à l'identique de la
//! référence), la lecture du fichier, le plongement d'un texte, la ressemblance de deux ; et
//! [`familles`], qui tire des ressemblances d'un document ses familles sûres et leurs teintes.

mod decoupeur;
pub mod familles;
mod unicode;

use crate::types::{Annotation, Board};
use std::collections::HashMap;
use std::sync::OnceLock;

/// **Le modèle que Glucose embarque** : 15 Mo, dans le programme même — aucune machine n'a à le
/// télécharger, aucune ne s'en trouve privée hors ligne. Sa provenance, sa licence et sa conversion :
/// `donnees/LISEZ-MOI.md`.
pub static MODELE: &[u8] = include_bytes!("../donnees/sens-multilingue-128.bin");

/// **Le modèle embarqué, lu une seule fois** pour tout le programme (le vocabulaire se range en
/// quelques dizaines de millisecondes) — ou rien s'il ne se lit pas, ce qu'une épreuve interdit.
pub fn embarque() -> Option<&'static Sens<'static>> {
    static LU: OnceLock<Option<Sens<'static>>> = OnceLock::new();
    LU.get_or_init(|| Sens::lire(MODELE).ok()).as_ref()
}

/// **Les textes d'un tableau, dans l'ordre du document** : l'identifiant et le texte de chaque nœud
/// de texte — ce que la parenté lit, et rien d'autre.
pub fn textes_du_tableau(tableau: &Board) -> impl Iterator<Item = (&str, &str)> {
    tableau.annotations.iter().filter_map(|a| match a {
        Annotation::Text { id, text, .. } => Some((id.as_str(), text.as_str())),
        _ => None,
    })
}

/// Le début du fichier : « GSENS1 » et deux octets nuls.
const SIGNATURE: &[u8; 8] = b"GSENS1\0\0";

/// **Le modèle du sens**, lu dans ses octets sans les recopier : le vocabulaire et la table restent
/// dans le fichier que Glucose embarque.
pub struct Sens<'a> {
    /// Les morceaux qui commencent un mot, et leur rang.
    debuts: HashMap<&'a str, u32>,
    /// Les morceaux qui continuent un mot (« ##… » dans le vocabulaire), sans leur « ## ».
    suites: HashMap<&'a str, u32>,
    /// Le rang de « [UNK] » : un mot qu'aucun morceau ne sait écrire.
    inconnu: u32,
    dimensions: usize,
    echelles: Vec<f32>,
    /// Une ligne de `dimensions` octets signés par morceau.
    table: &'a [u8],
}

impl<'a> Sens<'a> {
    /// **Lit le modèle** dans ses octets — le format de `outils/sens/convertir.py`.
    pub fn lire(octets: &'a [u8]) -> Result<Self, String> {
        let mut lu = Lecteur { octets, pos: 0 };
        if lu.prendre(8)? != SIGNATURE {
            return Err("ce n'est pas un modèle du sens (GSENS1)".into());
        }
        let n = lu.u32()? as usize;
        let dimensions = lu.u32()? as usize;
        let (mut debuts, mut suites) = (HashMap::with_capacity(n), HashMap::new());
        let mut inconnu = None;
        for rang in 0..n as u32 {
            let longueur = usize::from(u16::from_le_bytes(lu.tableau()?));
            let mot = std::str::from_utf8(lu.prendre(longueur)?)
                .map_err(|_| "un morceau n'est pas de l'UTF-8")?;
            if mot == "[UNK]" {
                inconnu = Some(rang);
            }
            match mot.strip_prefix("##") {
                Some(suite) => suites.entry(suite).or_insert(rang),
                None => debuts.entry(mot).or_insert(rang),
            };
        }
        let echelles = lu
            .prendre(4 * n)?
            .as_chunks::<4>()
            .0
            .iter()
            .map(|c| f32::from_le_bytes(*c))
            .collect();
        let table = lu.prendre(n * dimensions)?;
        Ok(Self {
            debuts,
            suites,
            inconnu: inconnu.ok_or("le vocabulaire n'a pas de [UNK]")?,
            dimensions,
            echelles,
            table,
        })
    }

    /// Le nombre de dimensions d'un plongement.
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// **Les rangs des morceaux de ce texte**, dans l'ordre — ce que le découpeur de référence donne.
    pub fn decouper(&self, texte: &str) -> Vec<u32> {
        decoupeur::decouper(texte, self)
    }

    /// **Le sens d'un texte** : la moyenne de ses morceaux, ramenée à la longueur 1 — ou rien pour un
    /// texte sans morceau (vide, des blancs).
    pub fn plonger(&self, texte: &str) -> Option<Vec<f32>> {
        let rangs = self.decouper(texte);
        if rangs.is_empty() {
            return None;
        }
        let d = self.dimensions;
        let mut somme = vec![0.0f32; d];
        for &r in &rangs {
            let r = r as usize;
            let echelle = self.echelles[r];
            for (s, &o) in somme.iter_mut().zip(&self.table[r * d..(r + 1) * d]) {
                *s += echelle * f32::from(o as i8);
            }
        }
        let norme = somme.iter().map(|x| x * x).sum::<f32>().sqrt();
        (norme > 0.0).then(|| somme.into_iter().map(|x| x / norme).collect())
    }
}

/// **La ressemblance de deux sens** : leur cosinus (ils sont de longueur 1), de −1 à 1.
pub fn ressemblance(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Ce que le découpeur demande au vocabulaire.
impl decoupeur::Vocabulaire for Sens<'_> {
    fn debut(&self, morceau: &str) -> Option<u32> {
        self.debuts.get(morceau).copied()
    }
    fn suite(&self, morceau: &str) -> Option<u32> {
        self.suites.get(morceau).copied()
    }
    fn inconnu(&self) -> u32 {
        self.inconnu
    }
}

/// Une lecture qui avance dans des octets, et dit où le fichier s'arrête trop tôt.
struct Lecteur<'a> {
    octets: &'a [u8],
    pos: usize,
}

impl<'a> Lecteur<'a> {
    fn prendre(&mut self, n: usize) -> Result<&'a [u8], String> {
        let fin = self.pos.checked_add(n).filter(|&f| f <= self.octets.len());
        let fin = fin.ok_or("le modèle s'arrête trop tôt")?;
        let tranche = &self.octets[self.pos..fin];
        self.pos = fin;
        Ok(tranche)
    }

    fn tableau<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let mut t = [0u8; N];
        t.copy_from_slice(self.prendre(N)?);
        Ok(t)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.tableau()?))
    }
}

#[cfg(test)]
mod tests;
