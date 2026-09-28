//! **Les itinéraires retenus** (FLECHE-5) : le détour d'une flèche se cherche quand ce dont il
//! dépend change, pas à chaque lecture de son tracé.
//!
//! Le dessin, le survol, le clic, les poignées et l'étiquette lisent tous le tracé d'une flèche
//! — plusieurs fois par image. Un itinéraire dépend de ses deux bouts et des obstacles. Une
//! entrée ne vaut que pour les bouts qu'elle a vus : une flèche en cours de tracé, qui suit la
//! souris sans que le document change, se recherche à chaque bout nouveau. Quand le document
//! est publié à neuf — une nouvelle version —, tout est oublié d'un bloc.
//!
//! # Pendant un geste : n'oublier que ce que le geste concerne
//!
//! Pendant un glisser, la version n'avance pas, mais le document change à chaque mouvement. Tout
//! oublier à chaque image ferait chercher toutes les flèches visibles à chaque image — et une
//! flèche qui longe deux cents cartes coûte deux millisecondes. Or un itinéraire tient encore,
//! **exactement**, tant que :
//!
//! * aucun obstacle qu'il a lu en se cherchant n'a bougé — les autres ne le contraignaient pas :
//!   c'est l'argument même de la recherche paresseuse (`glucose_core::arrow::contour`) ;
//! * aucun nœud déplacé ne barre son chemin.
//!
//! Chaque entrée retient donc les rangs de ce qu'elle a lu, et seules celles que le geste
//! touche sont oubliées. Une flèche s'écarte de la carte qu'on glisse pendant qu'on la glisse,
//! comme chez Tauri, qui recalculait tout à chaque image — et les autres ne coûtent rien.

use glucose_core::arrow::contour;
use glucose_core::arrow::{Bout, Itineraire};
use glucose_core::geometry::Rect;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

type Point = (f64, f64);

/// L'état du document qu'une mémoire décrit : sa version, le numéro du geste en cours (zéro
/// hors geste), combien d'écritures ce geste a faites, et les longueurs des trois listes du
/// tableau — qui disent ce qu'un rang désigne.
pub type Epoque = (u64, u64, usize, [usize; 3]);

/// Un itinéraire retenu : les bouts qu'il a vus, ses étapes, et les rangs des obstacles lus en
/// le cherchant (triés).
struct Entree {
    bouts: (Bout, Bout, bool),
    etapes: Vec<Point>,
    lus: Vec<u32>,
}

/// La mémoire des itinéraires, flèche par flèche.
#[derive(Default)]
pub struct Itineraires {
    epoque: Cell<Epoque>,
    table: RefCell<HashMap<String, Entree>>,
    /// Les rangs lus par l'itinéraire en train de se chercher.
    lecture: RefCell<Option<Vec<u32>>>,
    /// Combien d'itinéraires ont été réellement cherchés — l'instrument des épreuves.
    recherches: Cell<u64>,
}

impl Itineraires {
    /// Oublie tout.
    pub fn oublier(&self) {
        self.table.borrow_mut().clear();
    }

    /// **Se met à l'époque du document.** Une autre version, des listes d'une autre longueur, un
    /// geste qui s'achève sans nouvelle version — annulé, le document revenu à l'état d'avant —
    /// : tout est oublié. Un geste qui commence, ou de nouvelles écritures dans le même geste :
    /// n'est oublié que ce que touchent les nœuds qu'il a touchés, `(rang, boîte d'obstacle)`.
    pub fn suivre(&self, epoque: Epoque, touches: impl FnOnce() -> Vec<(u32, Option<Rect>)>) {
        let (version, numero, ecrits, longueurs) = epoque;
        let (v, n, e, l) = self.epoque.replace(epoque);
        let commence = n == 0 && numero != 0;
        if (v, l) != (version, longueurs) || (n != numero && !commence) {
            self.oublier();
            return;
        }
        if n == numero && e == ecrits {
            return;
        }
        let touches = touches();
        self.table
            .borrow_mut()
            .retain(|_, entree| !entree.touchee(&touches));
    }

    /// Combien d'itinéraires ont été cherchés depuis la création.
    pub fn recherches(&self) -> u64 {
        self.recherches.get()
    }

    /// Note qu'un obstacle a été lu par l'itinéraire en train de se chercher.
    pub(crate) fn noter(&self, rang: u32) {
        if let Some(lus) = self.lecture.borrow_mut().as_mut() {
            lus.push(rang);
        }
    }

    /// L'itinéraire retenu pour cette clé, ou celui que `calcul` trouve — alors retenu, avec ce
    /// qu'il a lu.
    pub(crate) fn retenu(
        &self,
        cle: &Itineraire<'_>,
        calcul: &mut dyn FnMut() -> Vec<Point>,
    ) -> Vec<Point> {
        let bouts = (cle.depart, cle.arrivee, cle.courbe);
        if let Some(entree) = self.table.borrow().get(cle.fleche) {
            if entree.bouts == bouts {
                return entree.etapes.clone();
            }
        }
        *self.lecture.borrow_mut() = Some(Vec::new());
        let etapes = calcul();
        let mut lus = self.lecture.borrow_mut().take().unwrap_or_default();
        lus.sort_unstable();
        lus.dedup();
        self.recherches.set(self.recherches.get() + 1);
        self.table.borrow_mut().insert(
            cle.fleche.to_string(),
            Entree {
                bouts,
                etapes: etapes.clone(),
                lus,
            },
        );
        etapes
    }
}

impl Entree {
    /// Un nœud touché par le geste compte-t-il pour cet itinéraire ? S'il a été lu en le
    /// cherchant, ou s'il barre maintenant son chemin.
    fn touchee(&self, touches: &[(u32, Option<Rect>)]) -> bool {
        let (a, b) = (self.bouts.0.point, self.bouts.1.point);
        touches.iter().any(|(rang, boite)| {
            self.lus.binary_search(rang).is_ok()
                || boite.is_some_and(|r| contour::barre(a, &self.etapes, b, r))
        })
    }
}
