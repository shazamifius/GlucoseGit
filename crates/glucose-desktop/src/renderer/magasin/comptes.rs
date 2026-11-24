//! **Ce que le magasin compte** : son chantier, ses évictions, ses mouvements, ses octets — ce que
//! la trace, les bancs et la mémoire lisent.

use super::{Magasin, Mouvements};
use crate::renderer::photo::Etat;

impl Magasin {
    /// Combien de décodages et de reprises sont encore en chantier.
    pub fn en_travail(&self) -> usize {
        self.atelier.en_travail()
    }

    /// Combien d'images ont été rendues à la machine depuis le début.
    pub fn evincees(&self) -> usize {
        self.evincees
    }

    /// Ce que l'étagement a fait depuis le début.
    pub fn mouvements(&self) -> Mouvements {
        self.mouvements
    }

    /// Attend que tout le chantier soit rentré.
    ///
    /// # Le seul endroit où attendre un décodage est juste
    ///
    /// Les **témoins visuels et les bancs**, qui produisent une image de référence et non une
    /// cadence : un témoin rendu avant l'arrivée des photos ne montrerait que des cadres, et
    /// ne prouverait rien. Partout ailleurs, l'invariant DECODE-1 l'interdit — le rendu
    /// n'attend jamais, il dessine ce qu'il a.
    ///
    /// Ne coûte rien quand le chantier est vide, ce qui est le cas de toutes les images sauf
    /// la première. La borne de temps n'est pas un réglage : c'est un garde-fou pour qu'un
    /// fil perdu fasse échouer un test au lieu de le faire tourner sans fin.
    pub fn attendre_le_chantier(&mut self) {
        let depart = std::time::Instant::now();
        while self.atelier.en_route() > 0 && depart.elapsed() < std::time::Duration::from_secs(60) {
            self.recolter();
            std::thread::yield_now();
        }
        self.recolter();
    }

    /// Les octets que les images décodées représentent, où qu'ils soient.
    pub fn octets(&self) -> usize {
        self.cache.octets()
    }

    /// Les octets de leurs niveaux dans cet état — tenus, offerts, en chemin.
    pub fn octets_en(&self, etat: Etat) -> usize {
        self.cache
            .values()
            .map(|e| e.pyramide.octets_en(etat))
            .sum()
    }

    /// **Ce que les images occupent en mémoire** : leurs niveaux tenus, et ceux qu'un ouvrier
    /// emporte pour les offrir ou les reprendre — encore là, dans sa file (MEMOIRE-4, fiche 82).
    /// Compter les seuls tenus laissait 2,7 Go « inconnus » au journal du téléphone virtuel.
    pub fn octets_en_memoire(&self) -> usize {
        self.octets_en(Etat::Tenu) + self.octets_en(Etat::EnChemin) + self.atelier.octets_a_jeter()
    }
}
