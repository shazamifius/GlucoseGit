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
//!
//! # TEINTES-1 — le rendu n'attend pas les teintes (fiche 86)
//!
//! « Ce qu'il y a autour » peut être tout le tableau : quatre mille textes serrés dans le rayon,
//! et chaque image d'un glisser coûtait 1,8 s (son essai du 6/11 : 716 ms sur une image, le fil
//! qui dessine saturé). Le coût est celui d'un carré, et aucun index ne le fait disparaître : le
//! voisinage **est** grand. La charte dit quoi faire : *une action lourde a le droit de durer,
//! en cascade ; le rendu n'attend jamais le travail*.
//!
//! Le document qui change n'oublie donc plus rien : il rend les teintes **périmées**. Une image
//! recalcule ce que sa tranche permet, et montre la teinte d'avant pour le reste, au pire celle
//! de son biome sans voisinage. Une teinte glisse lentement avec ses voisines : l'œil ne suit
//! pas cet écart, et il se referme en quelques images, tant que [`SymbioticHueCache::en_retard`]
//! redemande une image. Puis Glucose se rendort.
//!
//! **Sans tranche**, le calcul reste entier : les épreuves et les bancs restent déterministes,
//! et les deux voies rendent les mêmes pixels. Seul le vrai lancement en pose une.
//!
//! # REPOS-2 — au repos, le cache paie ses dettes sans image (fiche 98)
//!
//! La tranche courait depuis le début de l'image ; sur une carte de Wikipédia, les membranes
//! (17 ms) la mangeaient avant que le premier nœud demande sa teinte : **une** teinte par image,
//! et chaque image redessinait toute la carte pour l'avancer d'une seule. Trois cent cinquante
//! nœuds, trois cent cinquante images de 22 ms, trente secondes d'un cœur occupé après chaque
//! « tout cadrer » — pour un travail de quelques millisecondes.
//!
//! Le cache **retient ce qu'il doit** : le rang de chaque nœud qu'une image a montré d'une
//! teinte d'avant ([`SymbioticHueCache::en_retard`] ne dit plus que « il y a des dettes »).
//! Quand la vue ne bouge pas, le réveil les paie hors de toute image
//! ([`SymbioticHueCache::rattraper`]), par tranches, et ne demande **qu'une** image : celle qui
//! montre les teintes une fois toutes refaites. C'est la réparation « quand le système est
//! inactif » de Pad++, et l'état de repos de niri : on ne redessine pas pour travailler.

use glucose_core::quadtree::{SpatialHash, Visibles};
use glucose_core::symbiotic_hue::RAYON_SYMBIOTIQUE;
use glucose_core::types::{Annotation, Board};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CachedHue {
    pub hue: f64,
    pub rgb: (u8, u8, u8),
    /// Le document pour lequel elle a été calculée : différent du courant, elle est périmée.
    version: u64,
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
    /// Le temps qu'une image a pour recalculer (TEINTES-1). `None` : tout le temps qu'il faut.
    tranche: Option<Duration>,
    /// Quand cette image doit cesser de calculer.
    echeance: Option<Instant>,
    /// Une teinte a-t-elle déjà été calculée dans cette image ? La première l'est toujours : la
    /// cascade avance, même sur une machine où la tranche ne suffit à rien.
    une_faite: bool,
    /// **Ce que le cache doit** (REPOS-2) : le rang de chaque nœud que cette image a montré
    /// d'une teinte qu'elle n'a pas eu le temps de refaire. Un rang de l'index, pas un
    /// identifiant : noter une dette n'alloue rien, et elle se paie en temps constant.
    dettes: Vec<u32>,
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
            tranche: None,
            echeance: None,
            une_faite: false,
            dettes: Vec::new(),
        }
    }

    /// Combien de teintes ont été calculées depuis le début.
    pub fn calculs(&self) -> usize {
        self.calculs
    }

    /// **Le temps qu'une image a pour recalculer** (TEINTES-1) : posé par le vrai lancement.
    pub fn accorder_la_tranche(&mut self, tranche: Option<Duration>) {
        self.tranche = tranche;
    }

    /// Combien de teintes la dernière image doit encore.
    #[cfg(test)]
    pub(crate) fn dettes(&self) -> usize {
        self.dettes.len()
    }

    /// **Reste-t-il des teintes à refaire ?** Tant que oui, le réveil les rattrape.
    pub fn en_retard(&self) -> bool {
        !self.dettes.is_empty()
    }

    /// Accorde le cache sur l'état courant du document : une autre version rend les teintes
    /// périmées sans les oublier, un autre tableau les oublie. Ouvre la tranche de cette image.
    ///
    /// À appeler une fois par image, avant toute demande de teinte.
    pub fn suivre(&mut self, version: u64, board_id: &str) {
        self.echeance = self.tranche.map(|t| Instant::now() + t);
        self.une_faite = false;
        // L'image refait le compte de ce qu'elle montre : ce qu'elle doit, elle le notera.
        self.dettes.clear();
        let meme_tableau = self.repere.as_ref().is_some_and(|(_, b)| b == board_id);
        if !meme_tableau {
            self.entries.clear();
        }
        if !self
            .repere
            .as_ref()
            .is_some_and(|(v, _)| *v == version && meme_tableau)
        {
            self.repere = Some((version, board_id.to_string()));
        }
    }

    fn version(&self) -> u64 {
        self.repere.as_ref().map_or(0, |(v, _)| *v)
    }

    /// La tranche de cette image est-elle épuisée ?
    fn plus_de_temps(&self) -> bool {
        self.une_faite && self.echeance.is_some_and(|e| Instant::now() >= e)
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
        let version = self.version();
        let gardee = self
            .entries
            .get(ann.id())
            .map(|e| (e.hue, e.rgb, e.version));
        if let Some((hue, rgb, v)) = gardee {
            if v == version {
                return (hue, rgb);
            }
        }
        if self.plus_de_temps() {
            if let Some(rang) = index.rang_de(ann.id()) {
                self.dettes.push(rang);
            }
            return gardee.map_or_else(|| teinte_sans_voisinage(ann), |(hue, rgb, _)| (hue, rgb));
        }
        self.calculer(ann, index, board)
    }

    /// **REPOS-2 — payer hors d'une image ce que la dernière a dû.**
    ///
    /// `None` : rien n'est dû. `Some(false)` : la tranche est épuisée, il reste à payer —
    /// revenir, sans image. `Some(true)` : une image est due, une seule — toutes les teintes
    /// sont refaites, ou le document a changé depuis l'image et c'est à la suivante de refaire
    /// le compte. La première dette d'un appel se paie toujours : la cascade finit.
    ///
    /// L'index et le tableau doivent être ceux de la dernière image : c'est ce que la version
    /// inchangée garantit, puisque l'index ne se resynchronise que lorsqu'elle change.
    pub fn rattraper(&mut self, index: &SpatialHash, board: &Board, version: u64) -> Option<bool> {
        if self.dettes.is_empty() {
            return None;
        }
        let meme_document = self
            .repere
            .as_ref()
            .is_some_and(|(v, b)| *v == version && *b == board.id);
        if !meme_document {
            self.dettes.clear();
            return Some(true);
        }
        self.echeance = self.tranche.map(|t| Instant::now() + t);
        self.une_faite = false;
        while let Some(&rang) = self.dettes.last() {
            if self.plus_de_temps() {
                return Some(false);
            }
            self.dettes.pop();
            let rangs = [rang];
            let Some(ann) = Visibles::nouvelles(&rangs, board).annotations().next() else {
                continue;
            };
            // Un nœud montré deux fois dans l'image (sa carte, le bout d'une flèche) ne doit
            // qu'une teinte.
            if self
                .entries
                .get(ann.id())
                .is_none_or(|e| e.version != version)
            {
                self.calculer(ann, index, board);
            }
        }
        Some(true)
    }

    /// La teinte d'une carte depuis son voisinage, gardée pour la version courante.
    fn calculer(
        &mut self,
        ann: &Annotation,
        index: &SpatialHash,
        board: &Board,
    ) -> (f64, (u8, u8, u8)) {
        let version = self.version();
        self.une_faite = true;
        self.calculs += 1;

        let (x, y) = (ann.x(), ann.y());
        index.query_rect_ranks_into(x, y, x, y, RAYON_SYMBIOTIQUE, &mut self.voisins);
        let voisines = Visibles::nouvelles(&self.voisins, board).annotations();

        let hue = glucose_core::symbiotic_hue::get_symbiotic_hue(ann, voisines);
        let rgb = glucose_core::symbiotic_hue::hsl_to_rgb(hue, 0.75, 0.65);
        // Une chaîne neuve seulement pour une teinte neuve : la réécriture n'en alloue aucune.
        if let Some(entree) = self.entries.get_mut(ann.id()) {
            *entree = CachedHue { hue, rgb, version };
        } else {
            self.entries
                .insert(ann.id().to_string(), CachedHue { hue, rgb, version });
        }
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

/// **La teinte d'un nœud jamais vu, quand le temps manque** : celle de son biome et de son
/// identifiant, sans voisinage. Elle n'est pas gardée : la vraie viendra.
fn teinte_sans_voisinage(ann: &Annotation) -> (f64, (u8, u8, u8)) {
    let hue = glucose_core::symbiotic_hue::teinte_au_point(
        ann.id(),
        (ann.x(), ann.y()),
        std::iter::empty(),
    );
    (
        hue,
        glucose_core::symbiotic_hue::hsl_to_rgb(hue, 0.75, 0.65),
    )
}

#[cfg(test)]
mod tests;
