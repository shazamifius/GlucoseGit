//! **LOT-1 — tant que rien ne change, les poses restent valables à une similitude près** (fiche 87).
//!
//! # Ce que ça fait disparaître
//!
//! À la vue d'ensemble, une image qui ne change que la caméra refaisait pour chaque photo la même
//! réclamation, le même niveau, la même pose, le même examen. Le coût suivait le nombre de photos
//! alors que rien ne changeait. Or si `p` est une pose écrite à la vue `(x₀, y₀, s₀)`, sa place à la
//! vue `(x₁, y₁, s₁)` est **exactement** `r · p + t`, avec `r = s₁ / s₀` et `t = v₁ − r · v₀` : un seul
//! rapport et une seule translation, pour toutes les photos.
//!
//! La carte garde donc les poses de la dernière image complète, et le nuanceur applique la
//! similitude. **Aucune coordonnée du monde** : les poses restent celles d'une image ordinaire, à
//! l'écran, là où `f32` est le plus précis, et il n'y a qu'une loi — une image complète est le cas
//! `r = 1, t = 0`.
//!
//! # Quand c'est permis
//!
//! Rien n'a à être décidé par un seuil : on compare ce dont une pose dépend. Le document (sa
//! version, et aucun geste ouvert : elle n'avance qu'à la fin d'un geste), le tableau, ce qui est
//! visible, ce que le magasin tient, le cran et le plafond de la carte, et **l'échelle tant qu'elle
//! reste dans l'intervalle où chaque photo garde son niveau**, relevé photo par photo.
//!
//! Une photo qui change de niveau, entre à l'écran ou arrive de chez un ouvrier refait une image
//! complète — et rien d'autre ne change : ce chemin-là reste le seul qui décide.

use super::cran::EtatDeLaCarte;
use glucose_core::store::Store;
use glucose_core::types::Viewport;

/// **Ce qui mène les poses de la carte à la vue de cette image** : `p ↦ r · p + t`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Similitude {
    pub rapport: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Similitude {
    /// Rien ne bouge : ce que porte une image complète.
    pub const IDENTITE: Self = Self {
        rapport: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    /// De la vue où les poses ont été écrites à celle de cette image.
    fn entre(avant: (f64, f64, f64), vp: &Viewport) -> Self {
        let (x, y, s) = avant;
        let rapport = vp.scale / s;
        Self {
            rapport,
            tx: vp.x - rapport * x,
            ty: vp.y - rapport * y,
        }
    }
}

/// **Les photos de cette image sont celles que la carte tient déjà** : combien, et la similitude.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LotEnPlace {
    pub nombre: usize,
    pub similitude: Similitude,
}

/// **L'intervalle d'échelles où toutes les photos gardent leur niveau**, ouvert en bas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Intervalle {
    pub bas: f64,
    pub haut: f64,
}

impl Default for Intervalle {
    fn default() -> Self {
        Self {
            bas: 0.0,
            haut: f64::INFINITY,
        }
    }
}

impl Intervalle {
    /// Resserré par ce que cette photo permet.
    pub fn resserrer(&mut self, bas: f64, haut: f64) {
        self.bas = self.bas.max(bas);
        self.haut = self.haut.min(haut);
    }

    fn contient(&self, echelle: f64) -> bool {
        self.bas < echelle && echelle <= self.haut
    }
}

/// Ce dont dépendent les poses de la dernière image complète.
#[derive(Debug, Clone, PartialEq)]
struct Fige {
    numero: u64,
    version: u64,
    tableau: String,
    magasin: u64,
    rangs: Vec<u32>,
    /// Combien de photos ces rangs ont posées : les poses du lot.
    nombre: usize,
    cran: u32,
    plafond: Option<u32>,
    /// La part de la carte où le cran resterait celui-là (ETAGES-3) : au moins le premier, moins
    /// que le second.
    part: (u64, u64),
    echelles: Intervalle,
    vue: (f64, f64, f64),
    /// La période que la frise chronologique garde : l'opacité de chaque pose en dépend
    /// (ESTOMPE-1).
    filtre: Option<glucose_core::timeline::TemporalFilter>,
}

/// **Le lot que le rendu a proposé à la carte.**
#[derive(Debug, Default)]
pub struct Lot {
    dernier: u64,
    fige: Option<Fige>,
}

impl Lot {
    /// **Peut-on reprendre les poses que la carte tient ?** Elle doit tenir ce lot-là, et rien de
    /// ce dont elles dépendent ne doit avoir changé.
    pub(super) fn reconduire(
        &self,
        store: &Store,
        (vp, rangs): (&Viewport, &[u32]),
        magasin: u64,
        carte: &EtatDeLaCarte,
    ) -> Option<LotEnPlace> {
        let f = self.fige.as_ref()?;
        let part_valable = carte
            .part_des_photos
            .is_none_or(|p| f.part.0 <= p && p < f.part.1);
        let permis = carte.lot_de_la_carte == Some(f.numero)
            && f.version == store.version
            && store.journal.en_cours().is_none()
            && f.tableau == store.project.active_board_id
            && f.magasin == magasin
            && f.cran == carte.cran
            && f.plafond == carte.plafond_des_textures
            && part_valable
            && f.echelles.contient(vp.scale)
            && f.filtre == store.temporal_filter
            && f.rangs == rangs;
        permis.then(|| LotEnPlace {
            nombre: f.nombre,
            similitude: Similitude::entre(f.vue, vp),
        })
    }

    /// **Retient ce dont dépendent les poses de cette image complète**, et rend le numéro à
    /// proposer à la carte. Rien pendant un geste : le document bouge sous la main, et le lot
    /// ne servirait pas à l'image suivante.
    pub(super) fn proposer(
        &mut self,
        store: &Store,
        (vp, rangs): (&Viewport, &[u32]),
        (magasin, echelles, nombre): (u64, Intervalle, usize),
        carte: &EtatDeLaCarte,
    ) -> Option<u64> {
        self.fige = None;
        if store.journal.en_cours().is_some() || carte.debordee {
            return None;
        }
        self.dernier += 1;
        self.fige = Some(Fige {
            numero: self.dernier,
            version: store.version,
            tableau: store.project.active_board_id.clone(),
            magasin,
            rangs: rangs.to_vec(),
            nombre,
            cran: carte.cran,
            plafond: carte.plafond_des_textures,
            part: carte.part_valable,
            echelles,
            vue: (vp.x, vp.y, vp.scale),
            filtre: store.temporal_filter,
        });
        Some(self.dernier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vue(x: f64, y: f64, scale: f64) -> Viewport {
        Viewport { x, y, scale }
    }

    /// **La similitude redonne exactement la pose qu'on aurait calculée** : c'est tout l'objet de
    /// LOT-1, pour n'importe quel point du monde.
    #[test]
    fn test_lot_1_la_similitude_mene_d_une_vue_a_l_autre() {
        let (a, b) = (vue(13.5, -7.25, 0.0125), vue(-400.0, 91.0, 0.0173));
        let s = Similitude::entre((a.x, a.y, a.scale), &b);
        for (wx, wy) in [(0.0, 0.0), (1000.0, -2000.0), (54_321.0, 12_345.5)] {
            let (ax, ay) = (wx * a.scale + a.x, wy * a.scale + a.y);
            let (bx, by) = (wx * b.scale + b.x, wy * b.scale + b.y);
            assert!((ax * s.rapport + s.tx - bx).abs() < 1e-9);
            assert!((ay * s.rapport + s.ty - by).abs() < 1e-9);
        }
        assert_eq!(s.rapport, b.scale / a.scale);
    }

    /// Un lot proposé, et la carte qui dit le tenir.
    fn un_lot(store: &Store) -> (Lot, EtatDeLaCarte, Viewport, Vec<u32>) {
        let mut lot = Lot::default();
        let carte = EtatDeLaCarte {
            part_valable: (0, u64::MAX),
            ..EtatDeLaCarte::default()
        };
        let (vp, rangs) = (vue(10.0, 20.0, 0.5), vec![0, 1, 2]);
        let echelles = Intervalle {
            bas: 0.25,
            haut: 1.0,
        };
        let numero = lot
            .proposer(store, (&vp, &rangs), (7, echelles, 3), &carte)
            .expect("un lot");
        let carte = EtatDeLaCarte {
            lot_de_la_carte: Some(numero),
            ..carte
        };
        (lot, carte, vp, rangs)
    }

    /// **Rien de ce dont une pose dépend n'a changé : le lot se reprend**, et il dit combien de
    /// poses il porte, pas combien de rangs : une photo qui n'a rien à poser n'en a pas.
    #[test]
    fn test_lot_1_rien_n_a_change_le_lot_se_reprend() {
        let store = Store::new("lot");
        let (lot, carte, vp, rangs) = un_lot(&store);
        let repris = lot.reconduire(&store, (&vp, &rangs), 7, &carte);
        assert_eq!(
            repris.map(|r| (r.nombre, r.similitude)),
            Some((3, Similitude::IDENTITE))
        );
        let ailleurs = vue(-50.0, 3.0, 0.75);
        let repris = lot.reconduire(&store, (&ailleurs, &rangs), 7, &carte);
        assert_eq!(repris.map(|r| r.similitude.rapport), Some(0.75 / 0.5));
    }

    /// **Chaque chose dont une pose dépend, changée seule, refait une image complète.** C'est toute
    /// la sûreté de LOT-1 : un seul oubli, et des photos resteraient à la mauvaise place.
    #[test]
    fn test_lot_1_ce_dont_une_pose_depend_defait_le_lot() {
        let store = Store::new("lot");
        let (lot, carte, vp, rangs) = un_lot(&store);
        let refuse =
            |store: &Store, vp: &Viewport, rangs: &[u32], magasin, carte: &EtatDeLaCarte| {
                lot.reconduire(store, (vp, rangs), magasin, carte).is_none()
            };
        assert!(
            !refuse(&store, &vp, &rangs, 7, &carte),
            "le témoin : il se reprend"
        );

        // La frise chronologique change l'opacité des poses (ESTOMPE-1).
        let mut frise = store.clone();
        frise.set_temporal_filter(Some(glucose_core::timeline::TemporalFilter {
            start: 1914,
            end: 1918,
        }));
        assert!(refuse(&frise, &vp, &rangs, 7, &carte), "la frise a changé");

        let mut autre = Store::new("lot");
        autre.project.active_board_id = store.project.active_board_id.clone();
        autre.version = store.version + 1;
        assert!(
            refuse(&autre, &vp, &rangs, 7, &carte),
            "le document a changé"
        );

        let mut en_geste = Store::new("lot");
        en_geste.project.active_board_id = store.project.active_board_id.clone();
        en_geste.version = store.version;
        en_geste.journal.begin();
        assert!(
            refuse(&en_geste, &vp, &rangs, 7, &carte),
            "un geste est ouvert"
        );

        let mut tableau = Store::new("lot");
        tableau.version = store.version;
        tableau.project.active_board_id = "un autre tableau".to_string();
        assert!(refuse(&tableau, &vp, &rangs, 7, &carte), "un autre tableau");

        assert!(
            refuse(&store, &vp, &rangs, 8, &carte),
            "le magasin a changé"
        );
        assert!(
            refuse(&store, &vp, &[0, 1], 7, &carte),
            "une photo a quitté l'écran"
        );
        assert!(
            refuse(&store, &vp, &[0, 1, 3], 7, &carte),
            "une autre y est entrée"
        );

        let par = |c: EtatDeLaCarte| refuse(&store, &vp, &rangs, 7, &c);
        let base = || EtatDeLaCarte {
            lot_de_la_carte: carte.lot_de_la_carte,
            part_valable: carte.part_valable,
            ..EtatDeLaCarte::default()
        };
        assert!(
            par(EtatDeLaCarte {
                lot_de_la_carte: None,
                ..base()
            }),
            "la carte ne le tient plus"
        );
        assert!(
            par(EtatDeLaCarte {
                lot_de_la_carte: Some(99),
                ..base()
            }),
            "elle en tient un autre"
        );
        assert!(par(EtatDeLaCarte { cran: 1, ..base() }), "le cran a changé");
        assert!(
            par(EtatDeLaCarte {
                plafond_des_textures: Some(64),
                ..base()
            }),
            "le plafond aussi"
        );

        assert!(
            refuse(&store, &vue(0.0, 0.0, 0.25), &rangs, 7, &carte),
            "le bas : un autre niveau"
        );
        assert!(
            !refuse(&store, &vue(0.0, 0.0, 0.2501), &rangs, 7, &carte),
            "juste au-dessus"
        );
        assert!(
            !refuse(&store, &vue(0.0, 0.0, 1.0), &rangs, 7, &carte),
            "le haut lui appartient"
        );
        assert!(
            refuse(&store, &vue(0.0, 0.0, 1.0001), &rangs, 7, &carte),
            "juste au-delà"
        );
    }

    /// **Le cran ne se rejuge que si la part sort de ce qui le garde** (ETAGES-3) : juger à
    /// chaque image reparcourrait toutes les photos, alors que la part bouge sans cesse d'un rien.
    #[test]
    fn test_lot_1_la_part_de_la_carte_garde_le_cran() {
        let store = Store::new("lot");
        let (mut lot, carte, vp, rangs) = un_lot(&store);
        let carte = EtatDeLaCarte {
            part_valable: (100, 400),
            ..carte
        };
        let numero = lot
            .proposer(&store, (&vp, &rangs), (7, Intervalle::default(), 3), &carte)
            .expect("un lot");
        let essai = |part| {
            let c = EtatDeLaCarte {
                part_des_photos: Some(part),
                lot_de_la_carte: Some(numero),
                part_valable: (100, 400),
                ..EtatDeLaCarte::default()
            };
            lot.reconduire(&store, (&vp, &rangs), 7, &c).is_some()
        };
        assert!(essai(100), "la demande tient juste");
        assert!(essai(399), "le cran d'avant ne tient pas encore");
        assert!(!essai(99), "ce cran ne tient plus");
        assert!(!essai(400), "le cran d'avant tiendrait : plus net");
    }

    /// **Rien ne se propose pendant un geste, ni quand la carte déborde** : la prochaine image ne
    /// pourrait de toute façon rien reprendre.
    #[test]
    fn test_lot_1_rien_ne_se_propose_pendant_un_geste() {
        let mut store = Store::new("lot");
        let mut lot = Lot::default();
        let carte = EtatDeLaCarte::default();
        let (vp, rangs) = (vue(0.0, 0.0, 1.0), vec![0]);
        store.journal.begin();
        let i = (1, Intervalle::default(), 1);
        assert_eq!(lot.proposer(&store, (&vp, &rangs), i, &carte), None);
        let mut store = Store::new("lot");
        let debordee = EtatDeLaCarte {
            debordee: true,
            ..EtatDeLaCarte::default()
        };
        assert_eq!(lot.proposer(&store, (&vp, &rangs), i, &debordee), None);
        // Et un lot refusé défait le précédent : ses poses ne sont plus celles de la carte.
        assert!(lot.proposer(&store, (&vp, &rangs), i, &carte).is_some());
        store.journal.begin();
        assert_eq!(lot.proposer(&store, (&vp, &rangs), i, &carte), None);
        assert!(lot.fige.is_none());
    }

    #[test]
    fn test_lot_1_la_meme_vue_ne_bouge_rien() {
        let a = vue(13.5, -7.25, 0.0125);
        let s = Similitude::entre((a.x, a.y, a.scale), &a);
        assert_eq!(s, Similitude::IDENTITE);
    }

    /// **L'intervalle est le plus étroit de ceux des photos**, et sa borne haute lui appartient :
    /// `rang_pour` garde son niveau jusqu'à la largeur du niveau incluse.
    #[test]
    fn test_lot_1_l_intervalle_garde_le_plus_etroit() {
        let mut i = Intervalle::default();
        assert!(i.contient(1e9), "aucune photo : aucune borne");
        i.resserrer(1.0, 8.0);
        i.resserrer(2.0, 16.0);
        i.resserrer(0.5, 4.0);
        assert!(!i.contient(2.0));
        assert!(i.contient(2.5));
        assert!(i.contient(4.0));
        assert!(!i.contient(4.5));
    }
}
