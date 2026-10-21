//! **Lire par niveaux** (fiche 79) : un agent ne reprend que ce dont il a besoin.
//!
//! Lu d'un bloc, un tableau rend le texte entier de chacun de ses nœuds : pour un petit modèle de
//! langage, dont la fenêtre se remplit, c'est l'inverse d'une mémoire. Protéo l'a vécu le
//! 21/10/2026 : un manuel de 1 037 lignes lu en entier occupait presque toute sa fenêtre. Le
//! **sommaire** dit de chaque nœud ce qu'il est, où il est, sa première phrase et sa taille ;
//! l'agent lit ensuite **en entier** les nœuds qu'il choisit (`ids`), ou **cherche** une phrase
//! dans tous (le passage trouvé, pas le texte). Aucune longueur n'est fixée ici : la phrase est
//! l'unité, celle du texte lui-même.

use glucose_core::store::agents::Decrit;

/// Ce qu'un agent demande à lire.
pub struct Demande<'a> {
    /// Seulement ce qu'une membrane contient (elle comprise).
    pub dans: Option<&'a str>,
    /// Ces nœuds, en entier.
    pub ids: Option<Vec<&'a str>>,
    /// Les nœuds dont le texte contient ces mots (sans tenir compte des majuscules).
    pub cherche: Option<&'a str>,
    /// Le texte entier de tous les nœuds, comme avant les niveaux.
    pub complet: bool,
}

/// Comment un nœud est rendu.
#[derive(Debug, PartialEq)]
pub enum Rendu {
    /// Sa première phrase et sa taille, pas son texte.
    Sommaire,
    /// Son texte entier.
    Entier,
    /// La phrase qui contient ce qu'on cherche.
    Passage(String),
}

/// Les phrases d'un texte : coupées après `.`, `!` ou `?` suivis d'un blanc, et à chaque ligne.
pub fn phrases(texte: &str) -> Vec<&str> {
    let mut v = Vec::new();
    let mut debut = 0;
    let mut precedent: Option<char> = None;
    for (i, c) in texte.char_indices() {
        let fin_de_phrase = c.is_whitespace() && matches!(precedent, Some('.' | '!' | '?'));
        if c == '\n' || fin_de_phrase {
            v.push(&texte[debut..i]);
            debut = i + c.len_utf8();
        }
        precedent = Some(c);
    }
    v.push(&texte[debut..]);
    v.into_iter()
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect()
}

/// La première phrase d'un texte.
pub fn premiere_phrase(texte: &str) -> Option<&str> {
    phrases(texte).into_iter().next()
}

/// La première phrase qui contient `cherche`, sans tenir compte des majuscules.
pub fn passage(texte: &str, cherche: &str) -> Option<String> {
    let c = cherche.trim().to_lowercase();
    if c.is_empty() {
        return None;
    }
    phrases(texte)
        .into_iter()
        .find(|p| p.to_lowercase().contains(&c))
        .map(str::to_string)
}

/// Les nœuds à rendre, et comment : `ids` d'abord, sinon `cherche`, sinon tous en sommaire (ou
/// en entier, si l'agent l'a demandé).
pub fn choisir<'d>(decrits: &'d [Decrit], d: &Demande<'_>) -> Vec<(&'d Decrit, Rendu)> {
    let dans = |n: &Decrit| {
        d.dans
            .is_none_or(|m| n.membrane.as_deref() == Some(m) || n.id == m)
    };
    let pris = decrits.iter().filter(|n| dans(n));
    if let Some(ids) = &d.ids {
        return pris
            .filter(|n| ids.contains(&n.id.as_str()))
            .map(|n| (n, Rendu::Entier))
            .collect();
    }
    if let Some(c) = d.cherche {
        return pris
            .filter_map(|n| {
                let p = passage(n.texte.as_deref()?, c)?;
                Some((n, Rendu::Passage(p)))
            })
            .collect();
    }
    let rendu = || {
        if d.complet {
            Rendu::Entier
        } else {
            Rendu::Sommaire
        }
    };
    pris.map(|n| (n, rendu())).collect()
}

#[cfg(test)]
mod tests;
