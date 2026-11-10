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
//! touche sont refaites. Une flèche s'écarte de la carte qu'on glisse pendant qu'on la glisse,
//! comme chez Tauri, qui recalculait tout à chaque image — et les autres ne coûtent rien.
//!
//! # ITINERAIRE-2 — le rendu n'attend pas les itinéraires (fiche 102)
//!
//! Une nouvelle version oubliait tout d'un bloc, et l'image suivante cherchait d'un coup toutes
//! les flèches visibles. Sur la carte de Wikipédia et ses 1 128 liens, qui longent chacun des
//! centaines de nœuds : **2,2 s pour une image** au début d'un glisser, 0,9 s en zoomant, 0,66 s
//! au repos (sa session du 10/11 : « clairement 0 fps par moment »). La loi est celle des teintes
//! (TEINTES-1, REPOS-2) : ce qui change rend les itinéraires **périmés** sans les oublier ; une
//! image cherche ce que sa tranche permet — au moins un —, et montre le chemin d'avant pour le
//! reste, une ligne droite pour une flèche qui n'en a jamais eu ; au repos, le réveil paie ces
//! dettes hors de toute image ([`crate::renderer::Renderer::rattraper_les_itineraires`]), et une
//! seule image montre le tout. Sans tranche, tout se cherche : les épreuves restent
//! déterministes.

use glucose_core::arrow::contour;
use glucose_core::arrow::{Bout, Itineraire};
use glucose_core::geometry::Rect;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::time::{Duration, Instant};

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
    /// Cherché pour l'état présent du document ; périmé, il se montre en attendant d'être refait.
    frais: bool,
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
    /// Le temps qu'une image a pour chercher (ITINERAIRE-2) ; sans, tout se cherche.
    tranche: Cell<Option<Duration>>,
    echeance: Cell<Option<Instant>>,
    une_faite: Cell<bool>,
    /// Les flèches que la dernière image a montrées d'un chemin d'avant.
    dettes: RefCell<Vec<String>>,
}

impl Itineraires {
    /// Oublie tout : un autre tableau.
    pub fn oublier(&self) {
        self.table.borrow_mut().clear();
    }

    /// **Rend tout périmé**, sans rien oublier : le document a changé, chaque chemin se montre
    /// encore en attendant d'être refait (ITINERAIRE-2).
    pub fn perimer(&self) {
        for entree in self.table.borrow_mut().values_mut() {
            entree.frais = false;
        }
    }

    /// **Le document a changé** : sur le même tableau, une autre version rend tout périmé — les
    /// chemins se montrent en attendant d'être refaits (ITINERAIRE-2) ; un autre tableau oublie.
    pub fn document_change(&self, meme_tableau: bool) {
        if meme_tableau {
            self.perimer();
        } else {
            self.oublier();
        }
    }

    /// **Le temps qu'une image a pour chercher** : posé par le vrai lancement.
    pub fn accorder_la_tranche(&self, tranche: Option<Duration>) {
        self.tranche.set(tranche);
    }

    /// **Ouvre la tranche d'une image** : elle refait le compte de ce qu'elle doit.
    pub fn ouvrir_la_tranche(&self) {
        self.dettes.borrow_mut().clear();
        self.rouvrir();
    }

    /// Une tranche neuve, sans oublier les dettes : le rattrapage hors d'une image.
    fn rouvrir(&self) {
        self.echeance
            .set(self.tranche.get().map(|t| Instant::now() + t));
        self.une_faite.set(false);
    }

    /// La tranche est-elle épuisée ? Jamais avant le premier itinéraire : la cascade avance.
    fn plus_de_temps(&self) -> bool {
        self.une_faite.get() && self.echeance.get().is_some_and(|e| Instant::now() >= e)
    }

    /// **Reste-t-il des itinéraires à refaire ?**
    pub fn en_retard(&self) -> bool {
        !self.dettes.borrow().is_empty()
    }

    /// Les flèches dues, prises : le rattrapage les paie, ou les remet.
    pub(crate) fn prendre_les_dettes(&self) -> Vec<String> {
        let dettes = std::mem::take(&mut *self.dettes.borrow_mut());
        self.rouvrir();
        dettes
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
            self.perimer();
            return;
        }
        if n == numero && e == ecrits {
            return;
        }
        let touches = touches();
        for entree in self.table.borrow_mut().values_mut() {
            if entree.frais && entree.touchee(&touches) {
                entree.frais = false;
            }
        }
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
            if entree.frais && entree.bouts == bouts {
                return entree.etapes.clone();
            }
        }
        // Plus de temps dans cette image : le chemin d'avant, ou droit, et une dette.
        if self.plus_de_temps() {
            let mut dettes = self.dettes.borrow_mut();
            if !dettes.iter().any(|d| d == cle.fleche) {
                dettes.push(cle.fleche.to_string());
            }
            return self
                .table
                .borrow()
                .get(cle.fleche)
                .map(|e| e.etapes.clone())
                .unwrap_or_default();
        }
        self.une_faite.set(true);
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
                frais: true,
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

impl crate::renderer::Renderer {
    /// **ITINERAIRE-2 — payer hors d'une image ce que la dernière a dû** (REPOS-2, pour les
    /// flèches). `None` : rien n'est dû. `Some(false)` : la tranche est épuisée, revenir sans
    /// image. `Some(true)` : une image est due, une seule — tout est refait, ou le document a
    /// changé depuis l'image et c'est à la suivante de refaire le compte.
    pub(crate) fn rattraper_les_itineraires(
        &mut self,
        store: &glucose_core::store::Store,
    ) -> Option<bool> {
        if !self.itineraires.en_retard() {
            return None;
        }
        let board = store.active_board()?;
        if self.spatial_version != store.version || self.active_board_id != board.id {
            self.itineraires.prendre_les_dettes();
            return Some(true);
        }
        let noeuds = super::NoeudsDuRendu {
            board,
            index: Some(&self.spatial_hash),
            typographie: &self.typography,
            math: &self.math,
            contournement: Some(super::Contournement::de(
                &self.itineraires,
                &self.suivi_du_geste,
            )),
        };
        for id in self.itineraires.prendre_les_dettes() {
            let fleche = self
                .spatial_hash
                .rang_de(&id)
                .and_then(|r| glucose_core::quadtree::noeud_au_rang(board, r));
            if let Some(glucose_core::quadtree::Noeud::Annotation(a)) = fleche {
                // Le tracé passe par la mémoire : il refait l'itinéraire, ou le redoit.
                let _ = glucose_core::arrow::path_with(a, noeuds);
            }
        }
        Some(!self.itineraires.en_retard())
    }
}
