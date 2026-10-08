//! **BORD-1 — ce que le système réserve au bord de l'écran** (fiche 57).
//!
//! Sur un téléphone, Glucose dessine jusqu'au bord : sous la barre d'état, l'encoche de la
//! caméra, la barre de navigation — Android 15 et 16 l'imposent à une application qui vise
//! l'API 36, et Glucose le fait partout, pour n'avoir qu'un chemin. Le canevas peut aller sous
//! ces barres ; rien de ce qu'on touche ne doit s'y poser. Et le clavier, quand il sort, ne
//! rétrécit plus la fenêtre : il la recouvre.
//!
//! Le système dit ces marges à l'activité Java (`onApplyWindowInsets`), qui les confie ici ;
//! l'application les relève à chaque tour de boucle, et l'interface s'y cale. Ailleurs qu'au
//! téléphone, elles valent zéro.

use std::sync::{Mutex, PoisonError};

/// **Les marges du système**, en pixels de la fenêtre : la barre d'état et l'encoche en haut,
/// la navigation en bas, une encoche sur un côté en paysage — et le clavier, à part, parce
/// qu'il va et vient.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Marges {
    pub gauche: f32,
    pub haut: f32,
    pub droite: f32,
    pub bas: f32,
    pub clavier: f32,
}

impl Marges {
    /// **Ce que le bas de l'écran cache** : la navigation, ou le clavier s'il est sorti — il
    /// la recouvre.
    pub fn sous(&self) -> f32 {
        self.bas.max(self.clavier)
    }
}

/// La boîte aux lettres : les dernières marges dites, et de quoi réveiller la boucle.
struct Boite {
    neuves: Option<Marges>,
    reveil: Option<super::Reveil>,
}

static BOITE: Mutex<Boite> = Mutex::new(Boite {
    neuves: None,
    reveil: None,
});

/// **Le système dit ses marges** — depuis le fil de l'interface d'Android. Elles attendent la
/// boucle, qu'on réveille : elle dort peut-être.
pub fn recevoir(marges: Marges) {
    let mut boite = BOITE.lock().unwrap_or_else(PoisonError::into_inner);
    boite.neuves = Some(marges);
    if let Some(reveil) = &boite.reveil {
        reveil();
    }
}

/// De quoi réveiller la boucle quand les marges changent, donné au lancement.
pub fn brancher(reveil: super::Reveil) {
    BOITE.lock().unwrap_or_else(PoisonError::into_inner).reveil = Some(reveil);
}

/// Les marges dites depuis la dernière fois, s'il y en a.
pub fn nouvelles() -> Option<Marges> {
    BOITE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .neuves
        .take()
}

#[cfg(test)]
mod tests;
