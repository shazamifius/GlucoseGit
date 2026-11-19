//! **La convolution du détourage** (DETOURER-1) — presque tout le coût d'un modèle : 19,8 milliards
//! de multiplications pour u2netp, 160 pour ISNet (fiche 121 § 3).
//!
//! **Au pas de un** — toutes les convolutions d'u2netp —, [`tuiles`] : l'entrée bordée de zéros, des
//! tuiles tenues dans les registres, les fils sur les lignes. **La voie générale** (ici, tout pas) :
//! chaque fil prend une part des canaux de sortie, par blocs de [`BLOC`] ; une ligne de sortie de
//! chacun s'accumule à part, et chaque ligne d'entrée lue sert aux quatre.
//!
//! **Le jeu d'instructions se choisit à l'exécution** (la charte, VISION § 2) : la même boucle,
//! compilée une seconde fois pour AVX2 et FMA, sert les processeurs qui les ont ; les autres
//! gardent la version de base. Une multiplication-addition fusionnée arrondit une fois au lieu de
//! deux : les deux versions diffèrent au dernier bit, jamais davantage.

use super::calcul::Tenseur;
use super::modele::Conv;

mod tuiles;

/// Les canaux de sortie calculés ensemble.
const BLOC: usize = 4;

/// La convolution `c` de `x`, son ReLU compris, sur `fils` fils.
pub fn conv(x: &Tenseur, c: &Conv, relu: bool, fils: usize) -> Tenseur {
    if c.pas == 1 {
        return tuiles::conv(x, c, relu, fils);
    }
    let (ho, lo) = (
        c.sortie(x.h, c.kh).expect("vérifié"),
        c.sortie(x.l, c.kl).expect("vérifié"),
    );
    let plan = ho * lo;
    let mut v = vec![0.0; c.cout * plan];
    let blocs = c.cout.div_ceil(BLOC);
    let fils = fils.clamp(1, blocs);
    let par_fil = blocs.div_ceil(fils);
    // Une part des canaux de sortie, dans sa tranche de `v`.
    let travail = move |(k, part): (usize, &mut [f32])| {
        let premier = k * par_fil * BLOC;
        let mut acc = vec![0.0; BLOC * lo];
        for (j, sortie) in part.chunks_mut(BLOC * plan).enumerate() {
            let co = premier + j * BLOC;
            let tache = Tache {
                x,
                c,
                co,
                ho,
                lo,
                relu,
            };
            if (c.cout - co).min(BLOC) == BLOC {
                tache.lancer::<BLOC>(sortie, &mut acc);
            } else {
                tache.lancer::<1>(sortie, &mut acc);
            }
        }
    };
    let parts = v.chunks_mut(par_fil * BLOC * plan).enumerate();
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

/// Un bloc de canaux de sortie, de `co` à `co + B` (ou un seul, au bout d'une convolution dont le
/// nombre de canaux n'est pas multiple de [`BLOC`]).
#[derive(Clone, Copy)]
struct Tache<'a> {
    x: &'a Tenseur,
    c: &'a Conv,
    co: usize,
    ho: usize,
    lo: usize,
    relu: bool,
}

impl Tache<'_> {
    fn lancer<const B: usize>(self, sortie: &mut [f32], acc: &mut [f32]) {
        let plan = self.ho * self.lo;
        let n = sortie.len() / plan;
        if B == 1 && n > 1 {
            for (k, s) in sortie.chunks_mut(plan).enumerate() {
                Tache {
                    co: self.co + k,
                    ..self
                }
                .lancer::<1>(s, acc);
            }
            return;
        }
        #[cfg(target_arch = "x86_64")]
        if avx2_et_fma() {
            // SÛR : le processeur a dit avoir AVX2 et FMA, qu'exige seulement cette fonction.
            unsafe { self.bloc_avx2::<B>(sortie, acc) };
            return;
        }
        self.bloc::<B, { cfg!(target_arch = "aarch64") }>(sortie, acc);
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,fma")]
    unsafe fn bloc_avx2<const B: usize>(self, sortie: &mut [f32], acc: &mut [f32]) {
        self.bloc::<B, true>(sortie, acc);
    }

    /// Les `B` canaux, ligne de sortie après ligne de sortie. `FUSION` : la multiplication-addition
    /// fusionnée, que le processeur fait en une instruction (FMA sur x86, toujours sur ARM).
    #[inline(always)]
    fn bloc<const B: usize, const FUSION: bool>(self, sortie: &mut [f32], acc: &mut [f32]) {
        let (x, c, lo) = (self.x, self.c, self.lo);
        let noyau = c.kh * c.kl;
        let acc = &mut acc[..B * lo];
        for yo in 0..self.ho {
            for (b, ligne) in acc.chunks_exact_mut(lo).enumerate() {
                ligne.fill(c.biais[self.co + b]);
            }
            for ci in 0..c.cin {
                let entree = x.plan(ci);
                for ky in 0..c.kh {
                    let Some(iy) = (yo * c.pas + ky * c.dilatation).checked_sub(c.remplissage)
                    else {
                        continue;
                    };
                    let Some(ligne) = entree.get(iy * x.l..(iy + 1) * x.l) else {
                        continue;
                    };
                    for kx in 0..c.kl {
                        let w: [f32; B] = std::array::from_fn(|b| {
                            c.poids[((self.co + b) * c.cin + ci) * noyau + ky * c.kl + kx]
                        });
                        let decalage = (kx * c.dilatation) as isize - c.remplissage as isize;
                        accumuler::<B, FUSION>(acc, ligne, w, decalage, c.pas, lo);
                    }
                }
            }
            for (b, ligne) in acc.chunks_exact(lo).enumerate() {
                let dest = &mut sortie[b * self.ho * lo + yo * lo..][..lo];
                if self.relu {
                    dest.iter_mut()
                        .zip(ligne)
                        .for_each(|(d, &a)| *d = a.max(0.0));
                } else {
                    dest.copy_from_slice(ligne);
                }
            }
        }
    }
}

/// `acc[b][xo] += w[b] · ligne[xo·pas + decalage]`, pour chaque `xo` dont la colonne d'entrée
/// tombe dans la ligne.
#[inline(always)]
fn accumuler<const B: usize, const FUSION: bool>(
    acc: &mut [f32],
    ligne: &[f32],
    w: [f32; B],
    decalage: isize,
    pas: usize,
    lo: usize,
) {
    let l = ligne.len() as isize;
    // Les colonnes de sortie `xo` telles que 0 ≤ xo·pas + decalage < l.
    let debut = if decalage < 0 {
        (-decalage) as usize
    } else {
        0
    }
    .div_ceil(pas);
    let fin = if l > decalage {
        ((l - decalage - 1) as usize / pas + 1).min(lo)
    } else {
        0
    };
    if debut >= fin {
        return;
    }
    let lignes: [&mut [f32]; B] = {
        let mut reste = &mut acc[..];
        std::array::from_fn(|_| {
            let (tete, queue) = std::mem::take(&mut reste).split_at_mut(lo);
            reste = queue;
            &mut tete[debut..fin]
        })
    };
    let premiere = (debut as isize * pas as isize + decalage) as usize;
    if pas == 1 {
        let source = &ligne[premiere..premiere + (fin - debut)];
        for (i, &s) in source.iter().enumerate() {
            for b in 0..B {
                let a = &mut lignes[b][i];
                *a = if FUSION {
                    w[b].mul_add(s, *a)
                } else {
                    *a + w[b] * s
                };
            }
        }
    } else {
        for i in 0..fin - debut {
            let s = ligne[premiere + i * pas];
            for b in 0..B {
                let a = &mut lignes[b][i];
                *a = if FUSION {
                    w[b].mul_add(s, *a)
                } else {
                    *a + w[b] * s
                };
            }
        }
    }
}

/// Le processeur a-t-il AVX2 et FMA ? Demandé une fois.
#[cfg(target_arch = "x86_64")]
fn avx2_et_fma() -> bool {
    static OUI: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *OUI.get_or_init(|| is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma"))
}
