//! **Le cran commun des photos sur la carte** (ETAGES-3, fiche 32).
//!
//! # Ce que le budget ne bornait pas
//!
//! VRAM-1 borne ce que la carte garde **hors** de l'écran. Ce qu'elle montre, rien ne le
//! bornait : sur une carte qu'un Blender remplit, les textures de l'écran dépassaient le
//! budget sans rien dire — et Windows, alors, *« gèle le processus par intermittence pour
//! laisser tourner les autres »* (Microsoft, *Residency*).
//!
//! # La règle, et elle ne choisit aucun nombre
//!
//! Toutes les photos à l'écran perdent **le même nombre de crans** — un cran divise chaque
//! côté par deux, donc la place par quatre — et ce nombre est le plus petit qui fasse tenir ce
//! qu'elles demandent dans ce que le budget leur laisse. C'est la pixelisation assumée de la
//! charte, pilotée par le budget ; répartie sur toutes, elle est la plus petite possible pour
//! chacune. Les cartes de texte n'y entrent pas : un texte flou se lit mal, une photo réduite
//! d'un cran ne se voit presque pas.
//!
//! Et si aucun cran ne suffit — même réduites à un pixel, les photos ne tiennent pas, parce
//! que la carte est prise —, l'image se compose sur le processeur : *« se mettre là où il y a
//! de la place, pour surtout jamais gêner l'utilisateur »*.

use super::super::magasin::Magasin;
use super::super::{Cadrage, Renderer};
use glucose_core::quadtree::Visibles;
use glucose_core::store::Store;
use glucose_core::types::{BoardImage, Viewport};

/// **Ce que la carte laisse aux photos, et ce qu'on en a décidé.**
#[derive(Debug, Default)]
pub struct EtatDeLaCarte {
    /// Ce que les photos à l'écran peuvent occuper, dit par la présentation ; `None` quand
    /// la plateforme ne dit pas de budget.
    pub part_des_photos: Option<u64>,
    /// Le cran de cette image : zéro tant que tout tient.
    pub cran: u32,
    /// Le plus grand cran de la session.
    pub cran_pire: u32,
    /// Aucun cran ne suffit : l'image se compose sur le processeur.
    pub debordee: bool,
    /// Combien d'images l'ont été.
    pub images_debordees: u64,
}

/// Au-delà, toute pyramide est à son dernier niveau : une image large de 2³² pixels — la plus
/// grande qu'un `u32` décrive — en a trente-trois.
const CRAN_MAX: u32 = 32;

/// **Le plus petit cran qui fait tenir la demande dans la part**, ou `None` si aucun.
///
/// `demande` décroît avec le cran (chaque cran divise la place par quatre, jusqu'au pixel) :
/// le premier qui tient est donc le bon, et on s'arrête dès qu'on l'a.
pub(crate) fn cran_qui_tient(part: u64, demande: impl Fn(u32) -> u64) -> Option<u32> {
    (0..=CRAN_MAX).find(|&c| demande(c) <= part)
}

/// Sur quelle largeur la **source entière** d'une photo se pose à cette vue — ce qu'un
/// recadrage rend plus grand que la boîte (RECADRAGE-1). C'est elle qui choisit le niveau.
pub(super) fn largeur_source(img: &BoardImage, vp: &Viewport) -> f64 {
    let boite = (0.0, 0.0, img.width * vp.scale, img.height * vp.scale);
    img.crop.source_pour(boite).2
}

/// **Ce que les photos visibles demandent à la carte** au cran `cran`, en octets : pour
/// chacune que le magasin tient, le niveau qui couvre sa largeur divisée par `2^cran`.
pub(super) fn demande_des_photos(
    magasin: &Magasin,
    store: &Store,
    (vp, rangs): (&Viewport, &[u32]),
    cran: u32,
) -> u64 {
    let Some(board) = store.active_board() else {
        return 0;
    };
    let diviseur = 2f64.powi(cran as i32);
    Visibles::nouvelles(rangs, board)
        .images()
        .filter_map(|img| {
            let pyramide = &magasin.cache.get(img.src.as_deref()?)?.pyramide;
            let facteur = pyramide.facteur_pour((largeur_source(img, vp) / diviseur) as f32);
            let (l, h) = pyramide.dimensions(facteur);
            Some(u64::from(l) * u64::from(h) * 4)
        })
        .sum()
}

impl EtatDeLaCarte {
    /// **Décide le cran de cette image**, sur la vue qu'elle montre.
    ///
    /// Sur l'état et le magasin seulement, et non sur tout le renderer : la passe qui
    /// l'appelle tient déjà l'index en lecture.
    pub(super) fn juger(
        &mut self,
        magasin: &Magasin,
        store: &Store,
        (vp, rangs): (&Viewport, &[u32]),
    ) {
        let verdict = match self.part_des_photos {
            None => Some(0),
            Some(part) => cran_qui_tient(part, |c| {
                demande_des_photos(magasin, store, (vp, rangs), c)
            }),
        };
        self.cran = verdict.unwrap_or(CRAN_MAX);
        self.cran_pire = self.cran_pire.max(self.cran);
        self.debordee = verdict.is_none();
        self.images_debordees += u64::from(self.debordee);
    }
}

impl Renderer {
    /// **Rejuge depuis la voie du processeur** : cadrer la vue, puis juger — ce qui dira
    /// quand la carte peut reprendre les photos.
    pub fn rejuger_la_carte(&mut self, store: &Store, taille: (u32, u32), header_h: f32) {
        let (vp, rangs) = self.cadrer(store, taille, header_h, Cadrage::plein());
        self.carte.juger(&self.magasin, store, (&vp, &rangs));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::magasin::Entree;
    use crate::renderer::photo::Pyramide;

    /// **Le plus petit cran qui tient**, et rien de plus : chaque cran divise la demande par
    /// quatre, et la part décide où l'on s'arrête.
    #[test]
    fn test_le_cran_est_le_plus_petit_qui_tient() {
        let demande = |c: u32| 1_000_000u64 >> (2 * c);
        assert_eq!(cran_qui_tient(u64::MAX, demande), Some(0), "tout tient : aucun cran");
        assert_eq!(cran_qui_tient(1_000_000, demande), Some(0), "juste ce qu'il faut");
        assert_eq!(cran_qui_tient(999_999, demande), Some(1));
        assert_eq!(cran_qui_tient(62_500, demande), Some(2));
        assert_eq!(cran_qui_tient(62_499, demande), Some(3));
    }

    /// **Ce qui ne tient à aucun cran le dit** — les photos réduites au pixel pèsent encore.
    #[test]
    fn test_ce_qui_ne_tient_a_aucun_cran_deborde() {
        let demande = |c: u32| 4 + (1_000_000u64 >> (2 * c));
        assert_eq!(cran_qui_tient(3, demande), None);
        assert_eq!(cran_qui_tient(4, demande), Some(10));
    }

    /// **La demande d'une photo vue à 60 pixels, cran après cran** : le niveau qui la couvre,
    /// puis sa moitié, puis son quart — lus sur la vraie pyramide, par la vraie vue.
    #[test]
    fn test_la_demande_suit_le_niveau_de_chaque_photo() {
        let mut store = Store::new("cran");
        let board = store.project.active_board_id.clone();
        let mut img = BoardImage::new("p", 0.0, 0.0, 60.0, 60.0);
        img.src = Some("p.png".into());
        store.add_image(&board, img);
        let mut magasin = Magasin::nouveau();
        let pixmap = tiny_skia::Pixmap::new(512, 512).expect("une image");
        magasin
            .cache
            .insert("p.png".into(), Entree::pour_test(Pyramide::nouvelle(pixmap)));
        let vp = Viewport::default();
        let demande = |c| demande_des_photos(&magasin, &store, (&vp, &[0]), c);
        assert_eq!(demande(0), 64 * 64 * 4, "soixante pixels : le niveau de 64");
        assert_eq!(demande(1), 32 * 32 * 4, "un cran : le niveau de 32");
        assert_eq!(demande(2), 16 * 16 * 4);
    }
}
