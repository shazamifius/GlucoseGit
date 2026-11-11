//! **REGION-1 — une membrane se peint comme une région de carte** (fiche 105).
//!
//! # La loi
//!
//! La couleur en un point est celle de **la plus petite région qui le contient**, et non le mélange
//! de toutes — la règle des cartes (GMap, Gansner, Hu et Kobourov, 2010 ; Imhof). Chaque membrane
//! posait un voile de sa teinte sur ses mères : cinq membranes emboîtées faisaient une boue de cinq
//! teintes, sous des lettres qui pâlissaient (fiche 103 § 5 ; sa parole du 10/11/2026, *« c'est
//! vraiment pas beau »*). Les régions se posent désormais de la plus grande à la plus petite, et le
//! corps de chacune est **opaque** : il efface ce qui est dessous.
//!
//! # Le sol, puis les frontières
//!
//! Une région donne deux sortes de champs : son **sol** — ses halos, puis son corps — et sa
//! **frontière**. Tous les sols se posent d'abord, puis ce qui doit rester visible dans une
//! membrane (la grille de points, les zones, les cases d'un storyboard), puis toutes les frontières :
//! une frontière n'est jamais couverte par le halo d'une fille posée contre elle. Les deux voies
//! reçoivent cette liste, et la posent dans cet ordre.
//!
//! # Ce qui ne se voit pas ne se pose pas
//!
//! Un corps opaque qui couvre tout l'écran ([`Membrane::couvre`]) cache tout ce qui le précède :
//! ces champs ne se posent pas. Zoomé dans une région au sixième niveau, la carte graphique
//! composait six fois l'écran entier, une membrane sur l'autre ; elle n'en pose plus qu'un. Une
//! frontière dont l'écran est tout entier loin à l'intérieur ne pose rien non plus
//! ([`Membrane::touche`]). Rien de ce qui se voit ne change : une épreuve le vérifie au bit près.

use super::Membrane;

/// **Les champs des régions d'une image**, prêts à poser : le sol, puis les frontières.
#[derive(Debug, Default, Clone)]
pub struct Regions {
    sol: Vec<Membrane>,
    frontieres: Vec<Membrane>,
}

impl Regions {
    /// **Ajoute une région**, posée après — donc par-dessus — celles qui la précèdent : ses halos,
    /// son corps, sa frontière, pour un écran de `largeur × hauteur`.
    ///
    /// Un corps qui couvre l'écran efface le sol d'avant, ses propres halos compris ; un champ qui
    /// ne peut rien poser sur l'écran ne s'ajoute pas.
    pub fn poser(&mut self, [halos, corps]: [Membrane; 2], frontiere: Membrane, ecran: (f32, f32)) {
        if corps.couvre(ecran) {
            self.sol.clear();
        } else if halos.touche(ecran) {
            self.sol.push(halos);
        }
        if corps.touche(ecran) {
            self.sol.push(corps);
        }
        if frontiere.touche(ecran) {
            self.frontieres.push(frontiere);
        }
    }

    /// Le sol des régions : ce qui se pose avant la grille et les zones.
    pub fn sol(&self) -> &[Membrane] {
        &self.sol
    }

    /// Les frontières des régions : ce qui se pose après la grille et les zones.
    pub fn frontieres(&self) -> &[Membrane] {
        &self.frontieres
    }

    /// Le nombre de champs à poser, sol et frontières.
    pub fn len(&self) -> usize {
        self.sol.len() + self.frontieres.len()
    }

    /// Aucune région ne se voit.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests;
