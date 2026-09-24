//! **Un niveau de pyramide, et où vivent ses octets** (ETAGES-1, fiche 32).
//!
//! Un niveau n'est plus toujours lisible. Tant que l'écran le montre, il est **tenu** ; quand
//! il ne sert plus, il est **offert** au système, qui peut le jeter si une autre application
//! manque de place ; entre les deux, il est **en chemin** chez un ouvrier de l'atelier, parce
//! qu'offrir et reprendre coûtent des millisecondes que le fil qui dessine n'a pas. Et s'il a
//! été jeté, il est **perdu** : l'image se redécode depuis son fichier.
//!
//! Ce module ne décide rien de tout cela — c'est le magasin qui sait ce que l'écran demande. Il
//! garantit une seule chose, et le type la tient : **on ne lit que ce qui est tenu.**

use crate::plateforme::offre::{Offerte, Tenue};
use tiny_skia::PixmapRef;

/// Où sont les octets d'un niveau.
pub(crate) enum Octets {
    Tenus(Tenue),
    Offerts(Offerte),
    EnChemin,
    Perdus,
}

/// Ce que le magasin a besoin de savoir d'un niveau pour décider, sans ses octets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Etat {
    Tenu,
    Offert,
    EnChemin,
    Perdu,
}

/// Un niveau : ses dimensions, toujours connues, et ses octets, là où ils sont.
pub(super) struct Niveau {
    pub(super) largeur: u32,
    pub(super) hauteur: u32,
    pub(super) octets: Octets,
}

impl Niveau {
    /// Un niveau tenu, mis à zéro, ou `None` si le système refuse la place.
    pub(super) fn vide(largeur: u32, hauteur: u32) -> Option<Self> {
        let octets = (largeur as usize)
            .checked_mul(hauteur as usize)?
            .checked_mul(4)?;
        Some(Self {
            largeur,
            hauteur,
            octets: Octets::Tenus(Tenue::nouvelle(octets)?),
        })
    }

    /// Ses pixels, s'il est tenu.
    pub(super) fn vue(&self) -> Option<PixmapRef<'_>> {
        match &self.octets {
            Octets::Tenus(t) => PixmapRef::from_bytes(t.octets(), self.largeur, self.hauteur),
            _ => None,
        }
    }

    /// Ses octets, pour les écrire, s'il est tenu.
    pub(super) fn octets_mut(&mut self) -> Option<&mut [u8]> {
        match &mut self.octets {
            Octets::Tenus(t) => Some(t.octets_mut()),
            _ => None,
        }
    }

    /// Ce qu'il pèse, où qu'il soit.
    pub(super) fn taille(&self) -> usize {
        self.largeur as usize * self.hauteur as usize * 4
    }

    pub(super) fn etat(&self) -> Etat {
        match self.octets {
            Octets::Tenus(_) => Etat::Tenu,
            Octets::Offerts(_) => Etat::Offert,
            Octets::EnChemin => Etat::EnChemin,
            Octets::Perdus => Etat::Perdu,
        }
    }
}

/// **Ce qu'un niveau emporte chez un ouvrier** : des octets à offrir, ou à reprendre.
pub enum Transit {
    AOffrir(Tenue),
    AReprendre(Offerte),
}

/// Ce qui en revient.
pub enum Retour {
    Offerts(Offerte),
    /// Repris intacts, ou `None` si le système les avait jetés.
    Repris(Option<Tenue>),
}

impl Transit {
    /// **Le geste lui-même**, sur le fil de l'ouvrier : trois à cinq millisecondes pour dix
    /// mégaoctets, mesurées par `bench_offre`.
    pub fn accomplir(self) -> Retour {
        match self {
            Self::AOffrir(t) => Retour::Offerts(t.offrir()),
            Self::AReprendre(o) => Retour::Repris(o.reprendre()),
        }
    }
}

impl Retour {
    /// Ce que le niveau devient en rentrant.
    pub(super) fn en_octets(self) -> Octets {
        match self {
            Self::Offerts(o) => Octets::Offerts(o),
            Self::Repris(Some(t)) => Octets::Tenus(t),
            Self::Repris(None) => Octets::Perdus,
        }
    }
}
