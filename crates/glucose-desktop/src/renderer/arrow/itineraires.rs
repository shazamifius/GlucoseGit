//! **Les itinéraires retenus** (FLECHE-5) : le détour d'une flèche se cherche une fois par
//! version du document, pas à chaque lecture de son tracé.
//!
//! Le dessin, le survol, le clic, les poignées et l'étiquette lisent tous le tracé d'une flèche
//! — plusieurs fois par image. Un itinéraire ne dépend que de ses deux bouts et des obstacles ;
//! les obstacles ne bougent qu'avec le document. La table est donc **oubliée d'un bloc** quand
//! le document change (le rendu le fait en resynchronisant son index), et une entrée ne vaut
//! que pour les bouts qu'elle a vus : une flèche en cours de tracé, qui suit la souris sans que
//! le document change, se recherche à chaque bout nouveau.
//!
//! **Pendant un geste**, la version du document n'avance pas — elle ne se publie qu'au
//! relâchement —, mais le document change à chaque mouvement : l'époque d'un itinéraire est
//! donc la version **et** le nombre d'écritures du geste en cours. Une flèche s'écarte de la
//! carte qu'on glisse pendant qu'on la glisse, comme chez Tauri, qui recalculait tout à chaque
//! image.

use glucose_core::arrow::contour::Bout;
use glucose_core::arrow::Itineraire;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

type Point = (f64, f64);

/// Ce qu'une entrée a vu : les deux bouts, et si la flèche est courbe.
type Vu = (Bout, Bout, bool);

/// L'état du document qu'une mémoire décrit : sa version, le numéro du geste en cours (zéro
/// hors geste), et combien d'écritures ce geste a faites.
pub type Epoque = (u64, u64, usize);

/// La mémoire des itinéraires, flèche par flèche.
#[derive(Default)]
pub struct Itineraires {
    epoque: Cell<Epoque>,
    table: RefCell<HashMap<String, (Vu, Vec<Point>)>>,
    /// Combien d'itinéraires ont été réellement cherchés — l'instrument de l'épreuve.
    recherches: Cell<u64>,
}

impl Itineraires {
    /// Oublie tout : le document a changé.
    pub fn oublier(&self) {
        self.table.borrow_mut().clear();
    }

    /// Se met à l'époque du document : ce qui a été retenu pour une autre est oublié.
    pub fn suivre(&self, epoque: Epoque) {
        if self.epoque.replace(epoque) != epoque {
            self.oublier();
        }
    }

    /// Combien d'itinéraires ont été cherchés depuis la création.
    pub fn recherches(&self) -> u64 {
        self.recherches.get()
    }

    /// L'itinéraire retenu pour cette clé, ou celui que `calcul` trouve — alors retenu.
    pub(crate) fn retenu(
        &self,
        cle: &Itineraire<'_>,
        calcul: &mut dyn FnMut() -> Vec<Point>,
    ) -> Vec<Point> {
        let vu = (cle.depart, cle.arrivee, cle.courbe);
        if let Some((connu, etapes)) = self.table.borrow().get(cle.fleche) {
            if *connu == vu {
                return etapes.clone();
            }
        }
        let etapes = calcul();
        self.recherches.set(self.recherches.get() + 1);
        self.table
            .borrow_mut()
            .insert(cle.fleche.to_string(), (vu, etapes.clone()));
        etapes
    }
}
