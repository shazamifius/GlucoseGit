//! **Ce qu'on sait d'un geste** : la distribution de ses images, et celle de chacun de ses
//! postes. Sorti de [`super`] quand la boîte noire a reçu les postes (POSTES-1, fiche 63) :
//! le fichier passait ses six cents lignes, et cette structure a son histoire à elle.

use super::{Histogramme, Instantane, POSTES};

/// Ce qu'on sait d'un geste : combien d'images, et comment elles se distribuent.
#[derive(Debug, Clone)]
pub(super) struct Poste {
    /// La distribution des durées de ses images, en microsecondes.
    pub(super) durees: Histogramme,
    /// La distribution de chaque poste de rendu, pour savoir **où** va le temps de ce geste —
    /// un geste lent ne l'est pas pour la même raison qu'un autre.
    ///
    /// # Pourquoi une distribution, alors qu'une somme tenait en huit octets
    ///
    /// C'était une somme, et le rapport en tirait un pourcentage. Sur une session réelle de
    /// trente et une images, il a annoncé **`blit` 62,4 %** du temps du repos. La vérité :
    /// trente images à 0,94 ms, et **une** à 335 ms — le gel d'initialisation du pilote, à la
    /// deuxième seconde. Une seule image aberrante décidait donc du portrait de tout un geste,
    /// et je l'ai crue au point de l'écrire dans une fiche d'architecture.
    ///
    /// C'est exactement ce que le module d'à côté interdit dans sa première phrase — *« cent
    /// images à 2 ms et une à 200 ms donnent une moyenne de 4 ms, alors que l'utilisateur a vu
    /// un gel »*. La leçon avait été tirée pour les durées d'image et jamais appliquée à leur
    /// décomposition, si bien que le tableau des gestes se lisait au centile pendant que la
    /// ligne du dessous se lisait en moyenne.
    ///
    /// Le coût est de 356 octets par poste et par geste, soit 142 Ko pour la session entière,
    /// quelle que soit sa durée. Le tableau vit dans un `Vec`, donc rien ne passe par la pile.
    pub(super) postes: Vec<Histogramme>,
}

impl Default for Poste {
    fn default() -> Self {
        Self {
            durees: Histogramme::nouveau(),
            postes: vec![Histogramme::nouveau(); POSTES],
        }
    }
}

impl Poste {
    /// Compte cette image : sa durée, et celle de chacun de ses postes.
    pub(super) fn ajouter(&mut self, vu: &Instantane) {
        self.durees.ajouter(vu.duree_us);
        // Les zéros comptent : un poste qui ne travaille qu'une image sur dix ne coûte rien à
        // une image typique, et c'est précisément ce qu'on veut lire.
        for (distribution, us) in self.postes.iter_mut().zip(vu.postes_us.iter()) {
            distribution.ajouter(*us);
        }
    }
}
