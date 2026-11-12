//! **B3 — pourquoi l'IA rapproche deux textes** (fiche 110), mot par mot.
//!
//! Sa réponse à la page « Le cercle de l'IA » (12/11/2026) : l'IA de Glucose doit dire **pourquoi**
//! elle propose — *« ce texte est de la famille Eau, il parle de pluie et de nappes »* ; on corrige
//! bien mieux une IA qui s'explique (Kulesza et al., *Principles of Explanatory Debugging*, IUI 2015).
//!
//! # Une explication exacte, sans heuristique
//!
//! Le sens d'un texte est la somme des vecteurs de ses morceaux, ramenée à la longueur 1
//! ([`Sens::plonger`]). La ressemblance de deux textes `a` et `b` est donc
//!
//! ```text
//! cos(a, b) = (Σₘ vₘ) · Sᵦ / (|Sₐ| |Sᵦ|)        (m : les mots de a, vₘ : la somme de leurs morceaux)
//! ```
//!
//! — une **somme** : chaque mot de `a` y porte la part `vₘ · Sᵦ / (|Sₐ| |Sᵦ|)`, et les parts
//! s'additionnent à la ressemblance, au flottant près. Ce n'est pas une approximation par
//! perturbation ni un score d'attention (Opitz et al., *Interpretable Text Embeddings and Text
//! Similarity Explanation: A Survey*, EMNLP 2025) : pour un modèle statique, l'attribution additive
//! est la ressemblance elle-même, découpée. Un mot qui revient cumule ses parts ; un mot dont la
//! part est négative éloigne les deux textes.
//!
//! Sa limite, celle du modèle : une moyenne ne voit pas l'ordre des mots — « pas bon » ressemble à
//! « bon ».

use super::{decoupeur, norme, Sens};

/// **La part d'un mot** dans la ressemblance de deux textes : le mot tel que le texte l'écrit
/// (sa première écriture, s'il revient), et ce qu'il y pèse.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub mot: String,
    pub part: f32,
}

impl Sens<'_> {
    /// **Pourquoi `a` ressemble à `b`** : chaque mot de `a` avec sa part de leur ressemblance, la
    /// plus forte d'abord — les mots comme le modèle les sépare, aux blancs et à la ponctuation ; les
    /// écritures d'un même mot aux majuscules près n'en font qu'un (« l'eau », « Eau » : « eau »). La
    /// somme des parts est la ressemblance de [`Sens::plonger`] (`a`) et de [`Sens::plonger`] (`b`).
    /// Rien si l'un des deux n'a pas de sens.
    pub fn pourquoi(&self, a: &str, b: &str) -> Vec<Part> {
        let mots = decoupeur::decouper_en_mots(a, self);
        let tous: Vec<u32> = mots.iter().flat_map(|(_, r)| r.iter().copied()).collect();
        let vers = self.somme(&self.decouper(b));
        let diviseur = norme(&self.somme(&tous)) * norme(&vers);
        if diviseur <= 0.0 {
            return Vec::new();
        }
        let mut parts: Vec<Part> = Vec::new();
        let mut rang_de: std::collections::HashMap<String, usize> = Default::default();
        for (mot, rangs) in mots {
            let v = self.somme(&rangs);
            let part = v.iter().zip(&vers).map(|(x, y)| x * y).sum::<f32>() / diviseur;
            let rang = *rang_de.entry(mot.to_lowercase()).or_insert_with(|| {
                parts.push(Part {
                    mot: mot.to_string(),
                    part: 0.0,
                });
                parts.len() - 1
            });
            parts[rang].part += part;
        }
        parts.sort_by(|x, y| y.part.total_cmp(&x.part));
        parts
    }
}
