//! **Peindre les lueurs en bandes horizontales paralleles.**
//!
//! # Pourquoi ce module existe a part
//!
//! Sur une session reelle, `halos` est le premier poste des trois gestes dont l'utilisateur
//! se plaint, et de loin : 23,04 ms pour selectionner, 19,48 pour editer du texte, 18,08 pour
//! glisser un noeud. La raison est dans HALO-1 : une ombre de boite deborde de sa carte de la
//! portee du flou, donc elle touche bien plus de pixels qu'elle n'en designe -- 109 000 pour
//! une carte de 240 x 60. Plusieurs cartes, et la passe couvre l'ecran plusieurs fois.
//!
//! Le decoupage porte sur la DESTINATION, jamais sur les lueurs : chaque bande possede ses
//! lignes, une lueur a cheval est peinte par les deux, et le compilateur verifie lui-meme
//! qu'aucun fil ne voit les pixels d'un autre.
//!
//! # Ce que la mesure donne, et la limite qu'elle revele
//!
//! Sur le banc de la salissure -- vingt cartes, 2560 x 1600 -- la passe tombe de **8,86 a
//! 4,43 ms**. Un facteur deux la ou la composition des tuiles en gagnait huit, et la raison
//! est geometrique : les vingt cartes du banc tiennent sur trois lignes verticales, donc
//! **trois bandes sur seize font tout le travail**. Un decoupage horizontal ne partage que ce
//! qui est reparti verticalement.
//!
//! Ce qu'il faudrait pour aller plus loin, et qui reste a faire : des bandes plus fines que
//! le nombre de fils, distribuees a mesure qu'un fil se libere.

use super::{
    first_pixel_at_or_after, index_du_sommet, peindre_par_segments, EdgeProfile, HaloBox,
    LevelSource, Sens, HALO_ALPHA,
};
use tiny_skia::PixmapMut;

/// Compose la lueur d'une boîte sur `dst` (HALO-1).
/// **Une lueur dont tout ce qui ne dépend pas de la ligne est déjà calculé.**
///
/// # Pourquoi ce préambule sort de la boucle, et sort même de la bande
///
/// `draw_halo` calculait, avant de peindre : le profil de la gaussienne, sa portée, le profil
/// horizontal colonne par colonne, les `alpha` niveaux de couleur, et le sommet du profil.
/// Aucun de ces cinq ne dépend de la **ligne** — le commentaire le disait déjà pour les
/// colonnes — et, comme le découpage en bandes est horizontal, aucun ne dépend de la
/// **bande** non plus.
///
/// Les laisser dedans faisait refaire vingt préambules par bande, soit trois cent vingt pour
/// vingt cartes sur seize fils, dont deux allocations chacun. Le découpage gagnait encore un
/// facteur deux là où il aurait dû en donner davantage.
pub(super) struct LueurPrete {
    boite: HaloBox,
    profile: EdgeProfile,
    /// Les bornes en pixels d'écran, avant tout découpage.
    ecran: (i32, i32, i32, i32),
    columns: Vec<f32>,
    levels: Vec<LevelSource>,
    sommet: usize,
    alpha: u8,
}

impl LueurPrete {
    /// `None` si la lueur ne peut rien écrire — invisible, dégénérée, ou hors de l'écran.
    pub(super) fn nouvelle(
        halo: HaloBox,
        rgb: (u8, u8, u8),
        alpha: u8,
        largeur: i32,
        hauteur: i32,
    ) -> Option<Self> {
        if alpha == 0
            || !halo.left.is_finite()
            || !halo.top.is_finite()
            || !halo.right.is_finite()
            || !halo.bottom.is_finite()
        {
            return None;
        }
        let profile = EdgeProfile::new(halo.sigma);
        let reach = profile.reach(alpha);
        let x0 = first_pixel_at_or_after(halo.left - reach, largeur);
        let x1 = first_pixel_at_or_after(halo.right + reach, largeur);
        let y0 = first_pixel_at_or_after(halo.top - reach, hauteur);
        let y1 = first_pixel_at_or_after(halo.bottom + reach, hauteur);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        // Le profil horizontal ne dépend que de la colonne : il se calcule une fois pour
        // toutes les lignes, et la boucle chaude n'y fait plus qu'une lecture.
        let columns: Vec<f32> = (x0..x1)
            .map(|x| profile.band(x as f32 + 0.5, halo.left, halo.right))
            .collect();
        // Le profil horizontal est unimodal : il monte, plafonne, puis redescend. Son SOMMET
        // coupe la ligne en deux morceaux monotones, ce qui est tout ce dont la suite a besoin.
        let sommet = index_du_sommet(&columns);
        Some(Self {
            boite: halo,
            profile,
            ecran: (x0, y0, x1, y1),
            columns,
            // La lueur ne prend que `alpha` niveaux distincts : ils se précalculent tous.
            levels: (0..=alpha).map(|a| LevelSource::new(rgb, a)).collect(),
            sommet,
            alpha,
        })
    }

    /// Les ordonnées d'écran que cette lueur touche, bornes comprises.
    pub(super) fn lignes(&self) -> (i32, i32) {
        (self.ecran.1, self.ecran.3)
    }

    /// Peint la part de cette lueur qui tombe dans `dst`, une vue qui commence à `decalage`.
    pub(super) fn peindre(&self, dst: &mut PixmapMut, decalage: i32) {
        let (x0, y0, x1, y1) = self.ecran;
        let largeur = dst.width() as usize;
        let hauteur = dst.height() as i32;
        // Ce que cette vue voit de la lueur, en SES ordonnées.
        let (haut, bas) = ((y0 - decalage).max(0), (y1 - decalage).min(hauteur));
        if bas <= haut {
            return;
        }
        let (pixels, _) = dst.data_mut().as_chunks_mut::<4>();
        let peak = f32::from(self.alpha);
        for y in haut..bas {
            // Le poids de la ligne se lit en ordonnées d'ÉCRAN : c'est la seule grandeur du
            // calcul qui dépende de la bande, et l'y ramener est tout ce qu'il faut faire.
            let ecran_y = (y + decalage) as f32 + 0.5;
            let row_weight = self
                .profile
                .band(ecran_y, self.boite.top, self.boite.bottom)
                * peak;
            if row_weight < 0.5 {
                continue;
            }
            let debut = y as usize * largeur + x0 as usize;
            let ligne = &mut pixels[debut..debut + (x1 - x0) as usize];
            let (gauche, droite) = ligne.split_at_mut(self.sommet);
            peindre_par_segments(
                gauche,
                row_weight,
                &self.columns[..self.sommet],
                &self.levels,
                Sens::Montant,
            );
            peindre_par_segments(
                droite,
                row_weight,
                &self.columns[self.sommet..],
                &self.levels,
                Sens::Descendant,
            );
        }
    }
}

/// **Peint toutes les lueurs, en bandes horizontales parallèles.**
///
/// # Pourquoi les lueurs avant tout le reste
///
/// Sur une session réelle, `halos` est le premier poste des trois gestes dont l'utilisateur
/// se plaint, et de loin :
///
/// ```text
///     selectionner        23,04 ms
///     editer du texte     19,48 ms
///     glisser un noeud    18,08 ms
/// ```
///
/// La raison est dans HALO-1 : une ombre de boîte déborde de sa carte de la portée du flou,
/// donc elle touche bien plus de pixels qu'elle n'en désigne — 109 000 pour une carte de
/// 240 × 60. Plusieurs cartes sélectionnées, et la passe couvre l'écran plusieurs fois.
///
/// # Pourquoi des bandes, et pourquoi c'est sûr
///
/// Une bande possède ses lignes et personne d'autre n'y écrit : le découpage porte sur la
/// **destination**, jamais sur les lueurs. Une lueur à cheval est peinte par les deux bandes,
/// chacune sur sa part — et comme la peinture borne deja son parcours a la hauteur qu'on lui
/// donne, il suffit de lui présenter la bande et de descendre la boîte d'autant.
///
/// Rien ne se partage, donc rien ne se synchronise, et le compilateur le vérifie lui-même.
pub(super) fn peindre_en_bandes(pixmap: &mut PixmapMut, halos: &[(HaloBox, (u8, u8, u8))]) {
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    // Le préambule une fois, pas une fois par bande (voir [`LueurPrete`]).
    let pretes: Vec<LueurPrete> = halos
        .iter()
        .filter_map(|(halo, rgb)| {
            LueurPrete::nouvelle(*halo, *rgb, HALO_ALPHA, largeur as i32, hauteur as i32)
        })
        .collect();
    if pretes.is_empty() {
        return;
    }
    // L'etendue verticale que les lueurs touchent, toutes ensemble : c'est CELA qui se
    // partage, et non leur nombre.
    let lignes = pretes
        .iter()
        .map(|p| p.lignes())
        .fold((i32::MAX, i32::MIN), |(h, b), (ph, pb)| {
            (h.min(ph), b.max(pb))
        });
    let touchees = u32::try_from(lignes.1 - lignes.0).unwrap_or(0);
    let fils = crate::renderer::fils::bandes_utiles(touchees.min(hauteur));
    peindre_en(fils, pixmap, &pretes);
}

/// La même peinture, en un nombre de bandes **imposé**.
///
/// # Pourquoi ce paramètre existe, alors que personne ne le choisit en production
///
/// La charte demande que deux voies d'une même opération produisent les mêmes pixels, au bit
/// près. Ici les voies sont « un fil » et « seize », et rien ne le prouverait si le nombre de
/// bandes venait toujours de la machine — un test dirait seize sur l'une, deux sur l'autre,
/// un sur l'intégration continue.
pub(super) fn peindre_en(fils: usize, pixmap: &mut PixmapMut, pretes: &[LueurPrete]) {
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    if fils <= 1 {
        for prete in pretes {
            prete.peindre(pixmap, 0);
        }
        return;
    }
    let par_ligne = largeur as usize * 4;
    let par_bande = (hauteur as usize).div_ceil(fils) * par_ligne;
    std::thread::scope(|portee| {
        let mut haut = 0i32;
        for bande in pixmap.data_mut().chunks_mut(par_bande) {
            let h = (bande.len() / par_ligne) as u32;
            portee.spawn(move || {
                let Some(mut vue) = PixmapMut::from_bytes(bande, largeur, h) else {
                    return;
                };
                for prete in pretes {
                    prete.peindre(&mut vue, haut);
                }
            });
            haut += h as i32;
        }
    });
}
