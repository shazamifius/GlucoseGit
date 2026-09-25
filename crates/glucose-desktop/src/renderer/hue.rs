//! Cache des teintes symbiotiques.
//!
//! La teinte d'une carte dépend de son **voisinage** : la teinte de biome de sa position, sa
//! variation propre, et la moyenne vectorielle des cartes à moins de
//! [`RAYON_SYMBIOTIQUE`](glucose_core::symbiotic_hue::RAYON_SYMBIOTIQUE) d'elle (cf. `halo.rs`,
//! HALO-2).
//!
//! # HALO-4 — le voisinage se demande à l'index, il ne se cherche plus
//!
//! Le calcul parcourait **tout le tableau pour chaque carte**. À un million de nœuds, la seule
//! façon de tenir était de ne jamais recalculer : d'où un cache qui gardait une copie des
//! positions de tout le document, un ensemble de tous ses identifiants, une liste des points
//! qui avaient bougé, et un rayon d'invalidation — une constante qui devait rester égale à
//! celle du calcul sans que rien ne l'impose. Mesuré : 250 à 270 ms à chaque mutation,
//! premier poste de l'application, pour entretenir un évitement.
//!
//! Or l'index spatial sait rendre les voisines d'un point, et le calcul accepte n'importe quel
//! sur-ensemble des cartes du rayon. Le voisinage se demande donc en O(ce qu'il y a autour),
//! et toute la machinerie d'invalidation disparaît : une teinte n'est plus qu'une
//! mémoïsation, oubliée quand le document change.

use glucose_core::quadtree::{SpatialHash, Visibles};
use glucose_core::symbiotic_hue::RAYON_SYMBIOTIQUE;
use glucose_core::types::{Annotation, Board};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CachedHue {
    pub hue: f64,
    pub rgb: (u8, u8, u8),
}

pub struct SymbioticHueCache {
    entries: HashMap<String, CachedHue>,
    /// Le document et le tableau pour lesquels les teintes gardées valent encore.
    ///
    /// Une teinte est une fonction des positions et des identifiants : une version de document
    /// inchangée signifie des teintes inchangées (R-39). Quand elle change, on ne cherche plus
    /// **lesquelles** ont changé — les recalculer coûte désormais une requête de voisinage
    /// chacune, et seules les cartes à l'écran en demandent une.
    repere: Option<(u64, String)>,
    /// Tampon des rangs voisins, réutilisé d'une carte à l'autre : une requête par carte
    /// visible et par image ne doit pas allouer une fois par carte.
    voisins: Vec<u32>,
    /// Combien de teintes ont réellement été calculées. Rend la mémoïsation testable : sans ce
    /// compteur, un cache qui ne cache rien rend exactement les mêmes teintes.
    calculs: usize,
}

/// `new` ne prend aucun argument : `Default` est donc exactement le même constructeur.
/// Le déclarer évite qu'un appelant générique ait à connaître le nom `new`.
impl Default for SymbioticHueCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbioticHueCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            repere: None,
            voisins: Vec::new(),
            calculs: 0,
        }
    }

    /// Combien de teintes ont été calculées depuis le début.
    pub fn calculs(&self) -> usize {
        self.calculs
    }

    /// Accorde le cache sur l'état courant du document, et oublie tout s'il a changé.
    ///
    /// À appeler une fois par image, avant toute demande de teinte.
    pub fn suivre(&mut self, version: u64, board_id: &str) {
        if self
            .repere
            .as_ref()
            .is_some_and(|(v, b)| *v == version && b == board_id)
        {
            return;
        }
        self.repere = Some((version, board_id.to_string()));
        self.entries.clear();
    }

    /// La teinte d'une carte, calculée depuis son voisinage si elle n'est pas déjà connue.
    ///
    /// L'index doit être synchronisé sur `board` — c'est le cas au moment du rendu, qui le met
    /// à jour avant toute passe. Un index en retard ne rendrait pas une teinte fausse mais un
    /// voisinage incomplet : le sur-ensemble serait un sous-ensemble, et la teinte s'écarterait
    /// silencieusement. C'est la seule précondition de cette fonction.
    pub fn get_or_compute(
        &mut self,
        ann: &Annotation,
        index: &SpatialHash,
        board: &Board,
    ) -> (f64, (u8, u8, u8)) {
        if let Some(entry) = self.entries.get(ann.id()) {
            return (entry.hue, entry.rgb);
        }
        self.calculs += 1;

        let (x, y) = (ann.x(), ann.y());
        index.query_rect_ranks_into(x, y, x, y, RAYON_SYMBIOTIQUE, &mut self.voisins);
        let voisines = Visibles::nouvelles(&self.voisins, board).annotations();

        let hue = glucose_core::symbiotic_hue::get_symbiotic_hue(ann, voisines);
        let rgb = glucose_core::symbiotic_hue::hsl_to_rgb(hue, 0.75, 0.65);
        self.entries
            .insert(ann.id().to_string(), CachedHue { hue, rgb });
        (hue, rgb)
    }

    /// **La teinte qu'aurait `id` posé en `point`** — celle du bout libre d'une flèche
    /// (FLECHE-1). Elle n'est pas gardée : un bout libre suit la main, et ne se demande qu'une
    /// fois par image.
    pub fn au_point(
        &mut self,
        id: &str,
        point: (f64, f64),
        index: &SpatialHash,
        board: &Board,
    ) -> f64 {
        index.query_rect_ranks_into(
            point.0,
            point.1,
            point.0,
            point.1,
            RAYON_SYMBIOTIQUE,
            &mut self.voisins,
        );
        let voisines = Visibles::nouvelles(&self.voisins, board).annotations();
        glucose_core::symbiotic_hue::teinte_au_point(id, point, voisines)
    }
}

#[cfg(test)]
mod tests;
