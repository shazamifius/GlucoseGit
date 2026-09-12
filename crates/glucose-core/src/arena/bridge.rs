//! Le pont entre le modèle historique et l'arène — `std` uniquement.
//!
//! # À quoi il sert
//!
//! L'arène ne remplace rien tant qu'elle ne sait pas porter ce que le modèle porte déjà. Ce
//! module fait la traduction dans les deux sens, et le test d'aller-retour
//! `test_un_tableau_traverse_le_pont_sans_rien_perdre` est la preuve de complétude : un tableau
//! qui entre et ressort identique prouve que l'arène n'a rien laissé tomber.
//!
//! C'est aussi le premier pas de la substitution. Chaque consommateur du modèle pourra passer
//! à l'arène l'un après l'autre, le pont tenant les deux représentations en regard.
//!
//! # Les noms ne vivent que le temps du passage
//!
//! Un [`NodeId`] est un indice ; « img-42 » n'est qu'un nom de fichier. Le pont tient une table
//! des noms **parce qu'il fait l'import et l'export**, et elle disparaît avec lui : le document
//! en service, lui, n'en porte aucun. C'est ce qui économise une trentaine d'octets et une
//! allocation par nœud.
//!
//! # Les trois normalisations, écrites pour qu'elles ne surprennent personne
//!
//! Le pont n'est pas l'identité, et il vaut mieux le dire que le découvrir :
//!
//! | Ce qui entre | Ce qui ressort | Pourquoi |
//! |---|---|---|
//! | `#ABC`, `#aabbcc`, `#AABBCC` | `#aabbcc` | Une couleur est quatre octets ; sa forme écrite est une convention, pas une donnée. |
//! | `Some("")` comme texte optionnel | `None` | Un libellé vide et un libellé absent sont la même chose à l'écran, et l'arène ne distingue pas un texte vide d'un texte absent. |
//! | Une coordonnée hors du domaine, ou `NaN` | La borne, ou zéro | C'est la clôture du domaine de [`crate::fixed::Fx`], et c'est voulu : un fichier abîmé ne doit pas produire un document invisible. |
//!
//! Tout le reste passe à l'identique, y compris ce que l'arène ne comprend pas — une couleur
//! qui n'est pas un hexadécimal est conservée à la lettre.

mod export;
mod import;

use super::doc::Doc;
use super::NodeId;
use std::collections::HashMap;

/// La largeur posée à une carte dont la largeur est calculée (fiche 06 § 5).
///
/// Une taille par défaut plutôt que zéro : le culling doit pouvoir situer le nœud avant que la
/// mise en page ait tourné. Le drapeau `AUTO_WIDTH` dit que cette valeur est une estimation, et
/// le renderer la corrige dès qu'il mesure vraiment.
pub const DEFAULT_CARD_WIDTH: f64 = 260.0;
/// La hauteur posée à une carte dont la hauteur est calculée.
pub const DEFAULT_CARD_HEIGHT: f64 = 80.0;

/// Un document sur l'arène, plus ce qu'il faut pour retrouver les noms d'origine.
#[derive(Debug, Clone, Default)]
pub struct Bridge {
    /// Le document. C'est lui qui est destiné à rester.
    pub doc: Doc,
    /// Le nom d'origine de chaque nœud, indexé par [`NodeId`].
    names: Vec<Box<str>>,
}

impl Bridge {
    /// Le nom d'origine d'un nœud, tel que le modèle historique l'écrivait.
    pub fn name_of(&self, id: NodeId) -> &str {
        self.names.get(id.index()).map(|s| &**s).unwrap_or("")
    }

    /// Le nombre de nœuds portés.
    pub fn len(&self) -> usize {
        self.doc.len()
    }

    /// Vrai si le pont ne porte aucun nœud.
    pub fn is_empty(&self) -> bool {
        self.doc.is_empty()
    }

    /// Vérifie l'invariant BRG-1 : un nom par emplacement de l'arène, et le document cohérent.
    pub fn check(&self) -> Result<(), String> {
        self.doc.check()?;
        if self.names.len() != self.doc.nodes.slots() {
            return Err(format!(
                "BRG-1 : {} noms pour {} emplacements",
                self.names.len(),
                self.doc.nodes.slots()
            ));
        }
        Ok(())
    }
}

/// La table qui résout un nom d'origine vers son nœud, le temps de l'import.
type ByName<'a> = HashMap<&'a str, NodeId>;

/// Traduit une référence par nom en [`NodeId`], ou l'absence de nœud.
///
/// Une référence vers un nœud absent — ce qu'un fichier incomplet peut porter — devient
/// [`NodeId::NONE`] plutôt qu'une erreur : le document s'ouvre, le lien est simplement rompu,
/// et [`Doc::check`] ne signalera rien puisqu'il n'y a plus de lien.
fn resolve(by_name: &ByName<'_>, name: Option<&str>) -> NodeId {
    name.and_then(|n| by_name.get(n)).copied().unwrap_or(NodeId::NONE)
}

#[cfg(test)]
mod tests;
