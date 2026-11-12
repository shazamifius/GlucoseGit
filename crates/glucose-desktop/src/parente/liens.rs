//! **PARENTE-4 — ce que l'IA dit des flèches du tableau** (fiche 107) : la sûreté que chaque flèche
//! relie deux choses qui parlent de la même chose ([`glucose_core::sens::liens`]).
//!
//! Une flèche de la carte de Wikipédia va du texte d'un article à la **membrane** d'un autre : une
//! membrane qu'une flèche touche se juge par ce qu'elle porte — ses textes, ceux de ses membranes
//! filles — et par son nom. Une flèche dont un bout n'est ni un texte lu ni une telle membrane (une
//! photo, un post-it, rien) n'est pas jugée.
//!
//! **Seules se jugent les flèches qu'un agent ou un script a posées** et que l'humain n'a pas
//! touchées depuis (MAIN-1, fiche 108) : sa parole du 12/11/2026, *« il ne faut pas que l'IA juge
//! les flèches, mais qu'elle comprenne leur importance, afin de capter la méta derrière un
//! document »*. Une flèche qu'il tire est sa parole ; l'IA la laisse entière.

use glucose_core::sens::liens::{liens_du_tableau, surete_des_liens, Bout, Region};
use glucose_core::types::Board;
use std::collections::{HashMap, HashSet};

/// **Les flèches d'un tableau** dont les deux bouts sont nommés — la flèche, sa source, sa cible —,
/// et les membranes qu'elles touchent : leur nom, et les textes qu'elles portent, filles comprises.
#[derive(Debug, Default, Hash)]
pub(super) struct Liens {
    fleches: Vec<[String; 3]>,
    regions: Vec<(String, Option<String>, Vec<String>)>,
}

impl Liens {
    /// **Ceux du tableau** qu'un agent a posés (`des_agents`), dans l'ordre du document
    /// ([`liens_du_tableau`]).
    pub(super) fn du_tableau(tableau: &Board, des_agents: &HashSet<String>) -> Self {
        let (fleches, regions) = liens_du_tableau(tableau, des_agents);
        Self { fleches, regions }
    }

    /// Les noms des membranes touchées : l'IA les lit comme des textes.
    pub(super) fn noms(&self) -> impl Iterator<Item = &str> {
        self.regions.iter().filter_map(|r| r.1.as_deref())
    }

    /// **La sûreté de chaque flèche jugée**, par son identité : `ids` et `vecteurs` sont les textes
    /// lus et leur sens, `nom` le sens d'un nom de membrane.
    pub(super) fn juger(
        &self,
        (ids, vecteurs): (&[String], &[Vec<f32>]),
        nom: impl Fn(&str) -> Option<Vec<f32>>,
    ) -> Vec<(String, f32)> {
        let rang: HashMap<&str, usize> = ids
            .iter()
            .enumerate()
            .map(|(k, i)| (i.as_str(), k))
            .collect();
        let mut regions = Vec::new();
        let mut region_de: HashMap<&str, usize> = HashMap::new();
        for (id, titre, porte) in &self.regions {
            let porte = porte.iter().filter_map(|t| rang.get(t.as_str()).copied());
            let nom = titre.as_deref().and_then(&nom);
            if let Some(r) = Region::de(vecteurs, porte.collect(), nom.as_deref()) {
                region_de.insert(id, regions.len());
                regions.push(r);
            }
        }
        let bout = |id: &str| {
            let texte = rang.get(id).map(|&k| Bout::Texte(k));
            texte.or_else(|| region_de.get(id).map(|&r| Bout::Region(r)))
        };
        let (juges, paires): (Vec<&String>, Vec<(Bout, Bout)>) = self
            .fleches
            .iter()
            .filter_map(|[f, s, c]| Some((f, (bout(s)?, bout(c)?))))
            .unzip();
        let surete = surete_des_liens(vecteurs, &regions, &paires);
        juges.into_iter().cloned().zip(surete).collect()
    }
}
