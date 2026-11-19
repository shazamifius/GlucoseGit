//! **La convolution au pas de un, par tuiles** (DETOURER-1) — toutes celles d'u2netp, toutes sauf
//! une d'ISNet.
//!
//! La méthode classique d'une convolution directe : l'entrée **bordée de zéros** une fois pour
//! toutes (plus aucun test de bord dans la boucle), puis, pour chaque ligne de sortie, des **tuiles**
//! de [`TUILE`] colonnes sur [`BLOC`] canaux, tenues dans les registres du processeur pendant qu'on
//! parcourt tous les canaux d'entrée et tout le noyau — chaque valeur lue sert à quatre canaux, et
//! rien ne repart en mémoire avant la fin de la tuile. Les fils se partagent **les lignes** : un
//! modèle dont les couches n'ont que seize canaux (u2netp) ne laisserait sinon que quatre blocs à
//! sept fils.
//!
//! Le dernier bloc d'une convolution dont les canaux ne sont pas un multiple de [`BLOC`] calcule des
//! canaux de poids nuls, qu'il n'écrit pas : une couche à un canal (les sorties de côté) en coûte
//! quatre, quelques millièmes du modèle.

use super::super::calcul::Tenseur;
use super::super::modele::Conv;

/// Les colonnes de sortie d'une tuile.
const TUILE: usize = 16;

/// Les canaux de sortie d'une tuile.
const BLOC: usize = 4;

/// **L'entrée bordée de zéros** — et, au bout de chaque ligne, de quoi lire une tuile entière.
struct Bordee {
    v: Vec<f32>,
    h: usize,
    l: usize,
}

impl Bordee {
    fn de(x: &Tenseur, p: usize) -> Self {
        let (h, l) = (x.h + 2 * p, x.l + 2 * p + TUILE);
        let mut v = vec![0.0; x.c * h * l];
        for (c, plan) in x.v.chunks_exact(x.h * x.l).enumerate() {
            for (y, ligne) in plan.chunks_exact(x.l).enumerate() {
                v[(c * h + y + p) * l + p..][..x.l].copy_from_slice(ligne);
            }
        }
        Self { v, h, l }
    }

    fn ligne(&self, c: usize, y: usize) -> &[f32] {
        &self.v[(c * self.h + y) * self.l..][..self.l]
    }
}

/// Les poids rangés comme la tuile les lit, `[bloc][entrée][ky][kx][canal du bloc]`, complétés de
/// zéros au dernier bloc.
fn ranger(c: &Conv) -> Vec<f32> {
    let blocs = c.cout.div_ceil(BLOC);
    let noyau = c.kh * c.kl;
    let mut w = vec![0.0; blocs * c.cin * noyau * BLOC];
    for co in 0..c.cout {
        for ci in 0..c.cin {
            for k in 0..noyau {
                let dest = (((co / BLOC) * c.cin + ci) * noyau + k) * BLOC + co % BLOC;
                w[dest] = c.poids[(co * c.cin + ci) * noyau + k];
            }
        }
    }
    w
}

/// La convolution `c` (au pas de un) de `x`, son ReLU compris, sur `fils` fils.
pub fn conv(x: &Tenseur, c: &Conv, relu: bool, fils: usize) -> Tenseur {
    let (ho, lo) = (
        c.sortie(x.h, c.kh).expect("vérifié"),
        c.sortie(x.l, c.kl).expect("vérifié"),
    );
    let tache = Tache {
        bordee: Bordee::de(x, c.remplissage),
        poids: ranger(c),
        c,
        lo,
        relu,
    };
    let mut v = vec![0.0; c.cout * ho * lo];
    let fils = fils.clamp(1, ho);
    let par_fil = ho.div_ceil(fils);
    // Chaque fil reçoit, de chaque plan de sortie, sa tranche de lignes.
    let mut parts: Vec<Vec<&mut [f32]>> = (0..fils).map(|_| Vec::with_capacity(c.cout)).collect();
    for plan in v.chunks_mut(ho * lo) {
        for (k, tranche) in plan.chunks_mut(par_fil * lo).enumerate() {
            parts[k].push(tranche);
        }
    }
    let travail = |(k, mut plans): (usize, Vec<&mut [f32]>)| {
        if !plans.is_empty() {
            tache.lancer(k * par_fil, &mut plans);
        }
    };
    let parts = parts.into_iter().enumerate();
    // Un seul fil : rien à lancer — et là où l'on ne peut pas en lancer (le web), rien ne l'est.
    if fils == 1 {
        parts.for_each(travail);
    } else {
        std::thread::scope(|s| {
            for p in parts {
                s.spawn(move || travail(p));
            }
        });
    }
    Tenseur {
        c: c.cout,
        h: ho,
        l: lo,
        v,
    }
}

struct Tache<'a> {
    bordee: Bordee,
    poids: Vec<f32>,
    c: &'a Conv,
    lo: usize,
    relu: bool,
}

impl Tache<'_> {
    /// Les lignes de sortie d'un fil, à partir de `y0`, dans ses tranches de chaque plan.
    fn lancer(&self, y0: usize, plans: &mut [&mut [f32]]) {
        #[cfg(target_arch = "x86_64")]
        if super::avx2_et_fma() {
            // SÛR : le processeur a dit avoir AVX2 et FMA, qu'exige seulement cette fonction.
            unsafe { self.calculer_avx2(y0, plans) };
            return;
        }
        self.calculer::<{ cfg!(target_arch = "aarch64") }>(y0, plans);
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,fma")]
    unsafe fn calculer_avx2(&self, y0: usize, plans: &mut [&mut [f32]]) {
        self.calculer::<true>(y0, plans);
    }

    #[inline(always)]
    fn calculer<const FUSION: bool>(&self, y0: usize, plans: &mut [&mut [f32]]) {
        let (c, lo) = (self.c, self.lo);
        for dy in 0..plans[0].len() / lo {
            // Tous les blocs d'une même tuile à la suite : sa fenêtre d'entrée (les canaux, les
            // lignes du noyau, une tuile de large) reste dans le cache le plus proche.
            for x0 in (0..lo).step_by(TUILE) {
                for bloc in 0..c.cout.div_ceil(BLOC) {
                    let acc = self.tuile::<FUSION>(y0 + dy, bloc, x0);
                    let n = (lo - x0).min(TUILE);
                    for (b, rang) in acc.iter().zip(bloc * BLOC..c.cout) {
                        let dest = &mut plans[rang][dy * lo + x0..][..n];
                        for (d, &a) in dest.iter_mut().zip(b) {
                            *d = if self.relu { a.max(0.0) } else { a };
                        }
                    }
                }
            }
        }
    }

    /// **Une tuile** : [`TUILE`] colonnes de la ligne `yo`, sur les [`BLOC`] canaux du bloc, tout le
    /// noyau et toutes les entrées parcourus dans les registres.
    #[inline(always)]
    fn tuile<const FUSION: bool>(&self, yo: usize, bloc: usize, x0: usize) -> [[f32; TUILE]; BLOC] {
        let c = self.c;
        let mut acc: [[f32; TUILE]; BLOC] =
            std::array::from_fn(|b| [c.biais.get(bloc * BLOC + b).copied().unwrap_or(0.0); TUILE]);
        // Aucun chemin de panique dans la boucle : chaque tranche se prend une fois par ligne du
        // noyau, et ce qui ne s'y trouverait pas se saute. Une vérification qui peut paniquer
        // obligeait le compilateur à remettre les accumulateurs en mémoire après chaque FMA —
        // mesuré : deux fois moins vite.
        let par_ligne = c.kl * BLOC;
        let poids = self
            .poids
            .get(bloc * c.cin * c.kh * par_ligne..)
            .unwrap_or_default();
        for (ci, poids) in poids.chunks_exact(c.kh * par_ligne).take(c.cin).enumerate() {
            for (ky, poids) in poids.chunks_exact(par_ligne).enumerate() {
                let ligne = self.bordee.ligne(ci, yo + ky * c.dilatation);
                let fenetre = ligne.get(x0..).unwrap_or_default();
                for (kx, w) in poids.as_chunks::<BLOC>().0.iter().enumerate() {
                    let Some(s) = fenetre
                        .get(kx * c.dilatation..)
                        .and_then(<[f32]>::first_chunk::<TUILE>)
                    else {
                        continue;
                    };
                    for (a, &w) in acc.iter_mut().zip(w) {
                        for (a, &s) in a.iter_mut().zip(s) {
                            *a = if FUSION { w.mul_add(s, *a) } else { *a + w * s };
                        }
                    }
                }
            }
        }
        acc
    }
}
