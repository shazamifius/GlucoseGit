//! Les images prêtes à poser, une par nœud du canevas (MIP-2).
//!
//! # Pourquoi par nœud, et non par fichier
//!
//! Le décodage et les niveaux de réduction appartiennent au **fichier** : deux nœuds qui
//! montrent la même photo la décodent une fois. La taille à l'écran et la phase sous-pixel,
//! elles, appartiennent au **nœud** : deux copies d'une même image posées à deux endroits ne
//! sont pas les mêmes pixels.
//!
//! Avoir mis la vignette dans la pyramide revenait donc à n'en garder qu'une pour tout un
//! groupe de copies, qui se la reprenaient l'une à l'autre à chaque image. Le banc l'a montré
//! tout de suite : trente-six nœuds sur un même fichier, et la vignette ne servait jamais.
//!
//! # Ce que ce cache promet de ne pas faire
//!
//! Une vignette ne se construit qu'à la **deuxième** image consécutive où la même forme est
//! demandée. Pendant un zoom, aucune forme ne se répète : rien n'est construit, et le chemin
//! général sert. Dès que le geste s'arrête, la deuxième image la construit, et toutes les
//! suivantes ne font plus qu'un report sans transformation.
//!
//! Et une vignette dont le nœud n'a pas été dessiné à la dernière image est oubliée. Le cache
//! est ainsi borné par ce qui est à l'écran, sans qu'aucun nombre ait été choisi.

use super::photo::{Forme, Pyramide};
use std::collections::HashMap;
use tiny_skia::Pixmap;

/// Ce qu'on garde pour un nœud donné.
struct Entree {
    /// La forme demandée à l'image précédente, honorée ou non : c'est elle qui dit si une
    /// forme se répète, donc si elle mérite une vignette.
    demandee: Option<Forme>,
    prete: Option<(Forme, Pixmap)>,
    /// L'image à laquelle ce nœud a été dessiné pour la dernière fois.
    vue: u64,
}

/// Les vignettes de tous les nœuds dessinés récemment.
#[derive(Default)]
pub struct Vignettes {
    par_noeud: HashMap<String, Entree>,
    image: u64,
    faites: usize,
}

impl Vignettes {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ouvre une image : ce qui sera demandé ensuite appartient à celle-ci.
    pub fn ouvrir(&mut self) {
        self.image += 1;
    }

    /// Ferme l'image et oublie les nœuds qui n'ont pas été dessinés.
    pub fn fermer(&mut self) {
        let courante = self.image;
        self.par_noeud.retain(|_, e| e.vue == courante);
    }

    /// Combien de vignettes ont été construites depuis le début. Rend MIP-2 observable.
    pub fn faites(&self) -> usize {
        self.faites
    }

    /// Combien de nœuds sont actuellement suivis.
    pub fn suivis(&self) -> usize {
        self.par_noeud.len()
    }

    /// La vignette prête pour ce nœud à cette forme, si elle existe ou mérite d'être faite.
    ///
    /// Rend `None` la première fois qu'une forme est demandée pour ce nœud : l'appelant dessine
    /// alors par le chemin général.
    pub fn pour(&mut self, noeud: &str, forme: Forme, pyramide: &mut Pyramide) -> Option<&Pixmap> {
        let image = self.image;
        let entree = match self.par_noeud.get_mut(noeud) {
            Some(e) => e,
            None => self.par_noeud.entry(noeud.to_string()).or_insert(Entree {
                demandee: None,
                prete: None,
                vue: image,
            }),
        };
        let repetee = entree.demandee == Some(forme);
        entree.demandee = Some(forme);
        entree.vue = image;

        match &entree.prete {
            Some((faite, _)) if *faite == forme => {}
            _ if repetee => {
                entree.prete = Some((forme, pyramide.rendre(forme)));
                self.faites += 1;
            }
            _ => return None,
        }
        self.par_noeud
            .get(noeud)
            .and_then(|e| e.prete.as_ref())
            .map(|(_, p)| p)
    }
}

#[cfg(test)]
mod tests;
