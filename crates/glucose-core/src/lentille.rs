//! **LENTILLE-1 — ce que les lentilles laissent voir d'un nœud** (fiche 115).
//!
//! Une lentille change ce qu'on regarde, jamais le document ni le journal — comme la caméra :
//! la **frise chronologique** garde une période (`timeline::presence`, ESTOMPE-1), **le domaine
//! regardé** garde ce qui lui appartient (DOMAINE-1). Ce qu'une lentille écarte ne se cache pas, il
//! pâlit — jusqu'à [`ESTOMPE`] et jamais en dessous (VISION § 2, *rien ne se cache* ; la « vitrine »
//! de Kumu fait de même : ce qui n'est pas choisi s'estompe au lieu de disparaître).
//!
//! * **Le domaine regardé est gradué par l'appartenance** : un nœud porte chaque domaine avec un
//!   poids, de 0 à 1 ; sa présence va de [`ESTOMPE`] (il ne le porte pas) à 1 (il le porte
//!   pleinement), en ligne droite. C'est sa parole du 9/11 sur les membranes — *l'opacité dit
//!   l'appartenance* : un sujet qui entre pile dans sa matière est opaque, un sujet qui en dérive
//!   plus transparent. Aucun seuil : Tauri ne comptait un domaine qu'au-delà d'un poids de 0,1.
//! * **Deux lentilles se combinent par le minimum** — le « et » de la logique floue. Un nœud que
//!   les deux écartent reste à [`ESTOMPE`] ; leur produit l'aurait mis à 1,4 % : caché.
//! * **Une relation** — une flèche — suit ce qui lui appartient en propre (sa date, le domaine
//!   regardé si elle le porte), sinon ce qu'elle relie : le moins présent de ses deux bouts
//!   (ESTOMPE-2). Un bout libre ne retire rien.

use crate::timeline::{self, TemporalFilter, ESTOMPE};
use crate::types::{Annotation, BoardImage, DomainAssignment, TemporalAnchor};

/// **Les lentilles posées** : la période que la frise garde, le domaine regardé.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Lentilles<'a> {
    pub periode: Option<TemporalFilter>,
    pub domaine: Option<&'a str>,
}

/// **Ce qu'un nœud montre aux lentilles** : sa date et ses domaines.
#[derive(Debug, Clone, Copy)]
pub struct Regard<'a> {
    pub date: Option<&'a TemporalAnchor>,
    pub domaines: &'a [DomainAssignment],
}

impl Annotation {
    /// Ce que ce nœud montre aux lentilles.
    pub fn regard(&self) -> Regard<'_> {
        Regard {
            date: self.temporal_anchor(),
            domaines: self.domains(),
        }
    }
}

impl BoardImage {
    /// Ce que cette photo montre aux lentilles.
    pub fn regard(&self) -> Regard<'_> {
        Regard {
            date: self.temporal_anchor.as_ref(),
            domaines: &self.domains,
        }
    }
}

/// **La présence de ces domaines sous le domaine regardé** : entière sans lentille ; sinon de
/// [`ESTOMPE`] à 1, selon le poids qu'ils lui donnent.
pub fn dans_un_domaine(domaines: &[DomainAssignment], regarde: Option<&str>) -> f32 {
    let Some(regarde) = regarde else {
        return 1.0;
    };
    let poids = domaines
        .iter()
        .filter(|a| a.domain_id == regarde)
        .map(|a| a.weight)
        .fold(0.0, f64::max)
        .clamp(0.0, 1.0) as f32;
    ESTOMPE + (1.0 - ESTOMPE) * poids
}

impl Lentilles<'_> {
    /// **La présence de ce nœud** : le minimum de ce que chaque lentille lui laisse.
    pub fn presence(&self, n: Regard<'_>) -> f32 {
        timeline::presence(n.date, self.periode).min(dans_un_domaine(n.domaines, self.domaine))
    }

    /// **La présence d'une relation** : pour chaque lentille, ce qui lui appartient en propre,
    /// sinon le moins présent de ses bouts (un bout libre : `None`) ; puis le minimum des deux.
    pub fn presence_d_une_relation(
        &self,
        propre: Regard<'_>,
        bouts: [Option<Regard<'_>>; 2],
    ) -> f32 {
        let temps = timeline::presence_d_une_relation(
            propre.date,
            bouts.map(|b| b.and_then(|b| b.date)),
            self.periode,
        );
        let domaine = match self.domaine {
            None => 1.0,
            Some(d) if propre.domaines.iter().any(|a| a.domain_id == d) => {
                dans_un_domaine(propre.domaines, Some(d))
            }
            Some(d) => bouts
                .iter()
                .map(|b| b.map_or(1.0, |b| dans_un_domaine(b.domaines, Some(d))))
                .fold(1.0, f32::min),
        };
        temps.min(domaine)
    }
}

#[cfg(test)]
mod tests;
