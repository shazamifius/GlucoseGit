//! **L'éditeur du texte lié d'une flèche** (FLECHE-4, ANCRE-UX) — son état, et la fenêtre qui
//! le porte.
//!
//! # Ce qu'il a demandé
//!
//! La première version guidait par une petite barre, et l'on choisissait le passage dans la
//! carte, sur le canevas. Il l'a jugée à l'écran : un bouton qu'on ne reconnaît pas comme un
//! bouton, un libellé qui semble tronqué — et en face, la fenêtre de Glucose Tauri, *« une
//! vraie popup complète qui permet de bien surligner ce qu'on souhaite »*. Celle-ci en reprend
//! l'habit et le déroulé : un voile, une fenêtre, l'étape à la couleur de la carte, le texte
//! à sélectionner, ce qui est choisi en puces qu'on retire une à une, et les boutons.
//!
//! # Ce qui ne change pas : une seule loi pour les positions
//!
//! `ArrowTextEditor.tsx` recopiait le texte **rendu** par le navigateur et comptait ses
//! positions dans ce rendu-là : deux espaces de positions, et c'est là que ses ancres
//! divergeaient. Ici, la fenêtre met le texte en page par la fonction même de la carte
//! (`card_text_layout`), sur la même source : un clic y rend un octet de la source, comme dans
//! la carte. Seul le retour à la ligne change — la fenêtre a sa largeur —, jamais ce qu'une
//! position désigne.
//!
//! # Une unité monde, un point
//!
//! Dans la fenêtre, le texte se pose à l'échelle où une unité du monde vaut un pixel logique :
//! le corps d'une carte, quatorze, y reste quatorze points. Aucun facteur n'est choisi.

mod dessin;
mod fenetre;

pub use dessin::dessiner;
pub use fenetre::{layout_ancrage, BoutonDAncrage, Choisi, Entete, Fenetre, Puce, Zone};

use glucose_core::types::TextAnchor;

/// Le côté de la flèche qu'on ancre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Etape {
    Source,
    Cible,
}

/// L'édition en cours des ancres d'une flèche.
#[derive(Debug, Clone, PartialEq)]
pub struct Ancrage {
    pub fleche: String,
    pub etape: Etape,
    /// Les cartes de texte de ses deux bouts : `None` d'un côté qui n'en est pas une.
    pub cartes: (Option<String>, Option<String>),
    /// Ce qui est choisi de chaque côté.
    pub source: Vec<TextAnchor>,
    pub cible: Vec<TextAnchor>,
    /// Le glisser en cours : l'octet où il a commencé, celui où il en est, et s'il **ajoute**
    /// au choix (`Ctrl`) au lieu de le remplacer.
    pub glisse: Option<(usize, usize, bool)>,
    /// De combien le texte de la fenêtre a défilé, en points, quand il est plus haut qu'elle.
    pub defilement: f32,
}

impl Ancrage {
    /// La carte de l'étape en cours.
    pub fn carte(&self) -> Option<&str> {
        match self.etape {
            Etape::Source => self.cartes.0.as_deref(),
            Etape::Cible => self.cartes.1.as_deref(),
        }
    }

    /// Ce qui est choisi à l'étape en cours.
    pub fn ancres(&self) -> &[TextAnchor] {
        match self.etape {
            Etape::Source => &self.source,
            Etape::Cible => &self.cible,
        }
    }

    pub fn ancres_mut(&mut self) -> &mut Vec<TextAnchor> {
        match self.etape {
            Etape::Source => &mut self.source,
            Etape::Cible => &mut self.cible,
        }
    }

    /// Reste-t-il une étape après celle-ci ?
    pub fn a_une_suite(&self) -> bool {
        self.etape == Etape::Source && self.cartes.1.is_some()
    }

    /// Les étapes de cette flèche : une seule si un seul de ses bouts est une carte.
    pub fn etapes(&self) -> usize {
        usize::from(self.cartes.0.is_some()) + usize::from(self.cartes.1.is_some())
    }
}

/// Ce qu'un bouton de la fenêtre demande.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionDAncrage {
    /// Vider le choix de l'étape.
    Effacer,
    /// Retirer un passage du choix, par son rang.
    Retirer(usize),
    /// Passer à la cible — en gardant ce qui est choisi, s'il y a quelque chose.
    Suivant,
    /// Écrire les deux côtés, en un geste.
    Terminer,
    /// Tout laisser comme avant.
    Annuler,
}

#[cfg(test)]
mod tests;
