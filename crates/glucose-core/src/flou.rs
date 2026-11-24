//! **Le flou du reste, autour de ce qu'on a choisi** (Q12, fiche 137) — sa réponse du 23/11/2026,
//! sous l'image D : *« les carrés avec le flou, mais alors au moins 60 % moins que ce qu'il y a
//! actuellement »*.
//!
//! # Une gaussienne, par la loi des variances
//!
//! Flouter, c'est convoluer par une gaussienne d'écart-type σ ; et des convolutions qui se suivent
//! **additionnent leurs variances**. Une grande gaussienne se fait donc à peu de frais en
//! réduisant l'image, en floutant la petite, puis en l'agrandissant — à condition de compter ce que
//! chaque étape floute elle-même. Chaque étage réduit de moitié par le filtre binomial
//! `[1, 3, 3, 1] / 8` (variance 0,75 sur la grille fine), et s'agrandit au retour par
//! l'interpolation linéaire `[¾, ¼]` (variance 0,75) : 1,5 de variance par étage, et ce qui reste
//! se divise par 4 sur la grille deux fois plus grossière. Sur la plus grossière, une gaussienne
//! séparable fait le reste. C'est la loi du rendu de Vello sur processeur
//! (`vello_common/filter/gaussian_blur.rs`, Linebender) ; la réponse à une impulsion a la variance
//! annoncée à la troncature près du noyau à 3σ (l'épreuve la mesure).
//!
//! # Combien d'étages : une loi, pas un seuil
//!
//! Vello réduit tant que la variance dépasse 4, un nombre choisi. Ici, **on réduit tant que le
//! flou qui resterait sur la grille réduite efface encore sa propre fréquence de Nyquist sous un
//! niveau de huit bits** : la gaussienne y vaut `exp(−π²σ²/2)`, qui passe sous `1/255` dès que
//! `σ² ≥ 2 ln 255 / π²` (≈ 1,12). En deçà, réduire encore laisserait passer ce que la grille ne
//! sait plus porter.
//!
//! # Les deux voies
//!
//! Le processeur appelle ces fonctions ; la carte graphique les transcrit (`flou.wgsl` du bureau).
//! Chaque échantillon y tombe aux mêmes poids — une lecture bilinéaire au quart d'un texel rend
//! exactement `¼` et `¾` —, les bords prolongés des deux côtés (le dernier pixel se répète).

/// Un pixel prémultiplié, chaque canal de 0 à 255.
pub type Pixel = [f32; 4];

/// Une image de pixels prémultipliés, rangée par lignes.
#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub largeur: usize,
    pub hauteur: usize,
    pub pixels: Vec<Pixel>,
}

impl Image {
    /// Une image d'octets RVBA prémultipliés, rangée par lignes.
    pub fn d_octets(octets: &[u8], largeur: usize, hauteur: usize) -> Self {
        let pixels = octets
            .as_chunks::<4>()
            .0
            .iter()
            .take(largeur * hauteur)
            .map(|p| p.map(f32::from))
            .collect();
        Self {
            largeur,
            hauteur,
            pixels,
        }
    }

    fn a(&self, x: isize, y: isize) -> Pixel {
        let x = x.clamp(0, self.largeur as isize - 1) as usize;
        let y = y.clamp(0, self.hauteur as isize - 1) as usize;
        self.pixels[y * self.largeur + x]
    }
}

/// **La variance au-dessous de laquelle une grille ne se réduit plus** : `2 ln 255 / π²`.
pub fn variance_de_nyquist() -> f64 {
    2.0 * 255f64.ln() / (std::f64::consts::PI * std::f64::consts::PI)
}

/// **Le plan d'un flou d'écart-type σ** (en pixels) : combien d'étages de réduction, et le noyau
/// de la gaussienne qui reste sur la grille la plus grossière.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub etages: u32,
    /// Les poids de la gaussienne, du centre vers un bord (`noyau[0]` au centre) ; symétriques,
    /// leur somme sur les deux côtés vaut 1.
    pub noyau: Vec<f32>,
}

impl Plan {
    pub fn de(sigma: f64) -> Self {
        let seuil = variance_de_nyquist();
        let mut variance = sigma.max(0.0).powi(2);
        let mut etages = 0;
        while (variance - 1.5) / 4.0 >= seuil {
            variance = (variance - 1.5) / 4.0;
            etages += 1;
        }
        Self {
            etages,
            noyau: noyau(variance.sqrt()),
        }
    }

    /// Le côté d'une maille de la grille la plus grossière, en pixels de l'image.
    pub fn maille(&self) -> usize {
        1 << self.etages
    }
}

/// Les poids d'une gaussienne d'écart-type `s`, du centre jusqu'à `⌈3s⌉`, normalisés.
fn noyau(s: f64) -> Vec<f32> {
    if s <= 0.0 {
        return vec![1.0];
    }
    let r = (3.0 * s).ceil() as usize;
    let poids: Vec<f64> = (0..=r)
        .map(|x| (-((x * x) as f64) / (2.0 * s * s)).exp())
        .collect();
    let total = poids[0] + 2.0 * poids[1..].iter().sum::<f64>();
    poids.iter().map(|p| (p / total) as f32).collect()
}

/// La taille d'une grille réduite de moitié : la dernière maille, si elle est seule, compte.
pub fn moitie(n: usize) -> usize {
    n.div_ceil(2)
}

/// **Réduit de moitié** par le filtre binomial `[1, 3, 3, 1] / 8` sur chaque axe : le pixel `i`
/// de la petite grille couvre les pixels `2i` et `2i + 1` de la grande, et lit `2i − 1` à `2i + 2`.
pub fn reduire(img: &Image) -> Image {
    reduire_par((img.largeur, img.hauteur), |y, rangee| {
        rangee.clear();
        rangee.extend_from_slice(&img.pixels[y * img.largeur..(y + 1) * img.largeur]);
    })
}

/// **Réduit de moitié des octets RVBA prémultipliés** — le premier étage, qui lit le tampon de
/// l'écran tel quel : sans copie flottante de l'image entière, une rangée à la fois.
pub fn reduire_des_octets(octets: &[u8], (largeur, hauteur): (usize, usize)) -> Image {
    reduire_par((largeur, hauteur), |y, rangee| {
        rangee.clear();
        let octets = &octets[y * largeur * 4..(y + 1) * largeur * 4];
        rangee.extend(octets.as_chunks::<4>().0.iter().map(|p| p.map(f32::from)));
    })
}

/// **La réduction**, chaque rangée lue d'un coup par `lire_la_rangee` ; les quatre voisins de chaque
/// pixel de la petite grille, bornés aux bords (le dernier pixel se répète), se calculent une fois
/// par colonne et par rangée. Les nombres mêmes qu'un voisin borné lu pixel par pixel, au bit près —
/// qui coûtaient 8,7 ms à pleine taille en 1080p (fiche 140 § 5).
fn reduire_par(
    (largeur, hauteur): (usize, usize),
    lire_la_rangee: impl Fn(usize, &mut Vec<Pixel>),
) -> Image {
    let (l, h) = (moitie(largeur), moitie(hauteur));
    let colonnes: Vec<[usize; 4]> = (0..l).map(|i| fenetre(i, largeur)).collect();
    // D'abord les lignes (largeur réduite, hauteur entière), puis les colonnes.
    let mut lignes = Vec::with_capacity(l * hauteur);
    let mut rangee = Vec::with_capacity(largeur);
    for y in 0..hauteur {
        lire_la_rangee(y, &mut rangee);
        lignes.extend(colonnes.iter().map(|f| pese(f.map(|k| rangee[k]))));
    }
    let mut pixels = Vec::with_capacity(l * h);
    for j in 0..h {
        let quatre = fenetre(j, hauteur).map(|k| &lignes[k * l..(k + 1) * l]);
        pixels.extend((0..l).map(|x| pese(quatre.map(|r| r[x]))));
    }
    Image {
        largeur: l,
        hauteur: h,
        pixels,
    }
}

/// Les quatre voisins `2i − 1` à `2i + 2` du pixel `i` de la petite grille, dans une grille de `n`
/// cases, bornés aux bords.
fn fenetre(i: usize, n: usize) -> [usize; 4] {
    let x = 2 * i as isize;
    [x - 1, x, x + 1, x + 2].map(|k| k.clamp(0, n as isize - 1) as usize)
}

/// Le filtre binomial `[1, 3, 3, 1] / 8` sur quatre pixels.
fn pese(p: [Pixel; 4]) -> Pixel {
    std::array::from_fn(|c| (p[0][c] + 3.0 * p[1][c] + 3.0 * p[2][c] + p[3][c]) / 8.0)
}

/// La lecture bilinéaire d'une image au point `(cx, cy)` de sa grille (le centre du pixel `i` est
/// en `i`), les bords prolongés — la lecture d'un échantillonneur de carte graphique.
pub fn lire(img: &Image, (cx, cy): (f32, f32)) -> Pixel {
    let (x0, y0) = (cx.floor(), cy.floor());
    let (tx, ty) = (cx - x0, cy - y0);
    let (x0, y0) = (x0 as isize, y0 as isize);
    let (a, b) = (img.a(x0, y0), img.a(x0 + 1, y0));
    let (c, d) = (img.a(x0, y0 + 1), img.a(x0 + 1, y0 + 1));
    std::array::from_fn(|k| {
        let haut = a[k] + (b[k] - a[k]) * tx;
        let bas = c[k] + (d[k] - c[k]) * tx;
        haut + (bas - haut) * ty
    })
}

/// Le point de la grille deux fois plus grossière sous le centre du pixel fin `j`.
fn sous(j: usize) -> f32 {
    (j as f32 + 0.5) / 2.0 - 0.5
}

/// **Agrandit au double**, jusqu'à `(largeur, hauteur)`, par l'interpolation linéaire : chaque
/// pixel fin lit la grille grossière sous son centre — les poids `¾` et `¼`.
///
/// Ses voisins et ses poids se calculent une fois par colonne et par rangée ([`Pas`]) : les nombres
/// mêmes de [`lire`], au bit près (fiche 140 § 5).
pub fn agrandir(img: &Image, (largeur, hauteur): (usize, usize)) -> Image {
    let colonnes: Vec<Pas> = (0..largeur)
        .map(|x| Pas::de(sous(x), img.largeur))
        .collect();
    let mut pixels = Vec::with_capacity(largeur * hauteur);
    for y in 0..hauteur {
        let rangee = Pas::de(sous(y), img.hauteur);
        pixels.extend(colonnes.iter().map(|c| bilineaire(img, (c, &rangee))));
    }
    Image {
        largeur,
        hauteur,
        pixels,
    }
}

/// **La gaussienne séparable**, sur les lignes puis sur les colonnes, les bords prolongés.
pub fn convoluer(img: &Image, noyau: &[f32]) -> Image {
    let r = noyau.len() as isize - 1;
    let passe = |src: &Image, en_x: bool| -> Image {
        let mut pixels = Vec::with_capacity(src.pixels.len());
        for y in 0..src.hauteur as isize {
            for x in 0..src.largeur as isize {
                let mut somme = [0f32; 4];
                for d in -r..=r {
                    let p = if en_x {
                        src.a(x + d, y)
                    } else {
                        src.a(x, y + d)
                    };
                    let w = noyau[d.unsigned_abs()];
                    for c in 0..4 {
                        somme[c] += w * p[c];
                    }
                }
                pixels.push(somme);
            }
        }
        Image {
            largeur: src.largeur,
            hauteur: src.hauteur,
            pixels,
        }
    };
    passe(&passe(img, true), false)
}

/// **Le flou d'octets RVBA prémultipliés, jusqu'à la grille d'un demi-étage** : réduit `etages`
/// fois, floute la plus grossière, puis remonte jusqu'à la grille du premier étage — la dernière
/// montée, jusqu'à l'image entière, se fait en la mêlant à l'image nette ([`meler`]). Sans étage,
/// c'est l'image entière floutée.
pub fn flouter(octets: &[u8], (largeur, hauteur): (usize, usize), plan: &Plan) -> Image {
    if plan.etages == 0 {
        return convoluer(&Image::d_octets(octets, largeur, hauteur), &plan.noyau);
    }
    let mut tailles = vec![(largeur, hauteur)];
    let mut courante = reduire_des_octets(octets, (largeur, hauteur));
    for _ in 1..plan.etages {
        tailles.push((courante.largeur, courante.hauteur));
        courante = reduire(&courante);
    }
    courante = convoluer(&courante, &plan.noyau);
    // Remonter jusqu'au premier étage, sans le dernier pas.
    for &taille in tailles.iter().skip(1).rev() {
        courante = agrandir(&courante, taille);
    }
    courante
}

/// **Le masque de ce qui reste net** : une grille d'octets de la taille de la grille la plus
/// grossière, de 0 (flouté) à 255 (net).
#[derive(Clone, Debug, PartialEq)]
pub struct Masque {
    pub largeur: usize,
    pub hauteur: usize,
    pub octets: Vec<u8>,
}

impl Masque {
    /// La lecture bilinéaire du masque au point `(cx, cy)` de sa grille, de 0 à 1.
    pub fn lire(&self, (cx, cy): (f32, f32)) -> f32 {
        let a = |x: isize, y: isize| {
            let x = x.clamp(0, self.largeur as isize - 1) as usize;
            let y = y.clamp(0, self.hauteur as isize - 1) as usize;
            f32::from(self.octets[y * self.largeur + x]) / 255.0
        };
        let (x0, y0) = (cx.floor(), cy.floor());
        let (tx, ty) = (cx - x0, cy - y0);
        let (x0, y0) = (x0 as isize, y0 as isize);
        let haut = a(x0, y0) + (a(x0 + 1, y0) - a(x0, y0)) * tx;
        let bas = a(x0, y0 + 1) + (a(x0 + 1, y0 + 1) - a(x0, y0 + 1)) * tx;
        haut + (bas - haut) * ty
    }
}

/// La taille de la grille la plus grossière d'une image de `(largeur, hauteur)` : la moitié prise
/// `etages` fois, chaque dernière maille seule comptée.
pub fn grille(plan: &Plan, (largeur, hauteur): (usize, usize)) -> (usize, usize) {
    (0..plan.etages).fold((largeur, hauteur), |(l, h), _| (moitie(l), moitie(h)))
}

/// **Mêle l'image nette et son flou par le masque** : `net × m + flou × (1 − m)`, où `m` dit, de 0
/// à 1, ce qui reste net. Le flou se lit sous le centre de chaque pixel dans la grille du premier
/// étage (la dernière montée de [`flouter`]), le masque dans la grille la plus grossière — la même
/// lecture bilinéaire, les bords prolongés. `net` : des octets RVBA prémultipliés, que le mélange
/// remplace — l'image entière : la chrome, posée ensuite, couvre la bande.
///
/// **Les mêmes nombres, au bit près, que [`lire`] et [`Masque::lire`] pixel par pixel** — les
/// mêmes opérations, dans le même ordre —, mais leurs voisins et leurs poids se calculent une fois
/// par colonne et par rangée, et le flou ne se lit pas là où le masque garde tout net. Lus pixel par
/// pixel, ils coûtaient 50,6 ms par image en 1080p sur son Ryzen (fiche 140 § 5).
pub fn meler(
    net: &mut [u8],
    (largeur, hauteur): (usize, usize),
    (flou, plan): (&Image, &Plan),
    masque: &Masque,
) {
    let maille = plan.maille() as f32;
    let centre = |j: usize| (j as f32 + 0.5) / maille - 0.5;
    let colonnes_du_masque: Vec<Pas> = (0..largeur)
        .map(|x| Pas::de(centre(x), masque.largeur))
        .collect();
    let colonnes_du_flou: Vec<Pas> = (0..largeur)
        .map(|x| Pas::de(sous(x), flou.largeur))
        .collect();
    // Une rangée du masque sert à `maille` rangées de l'écran, une du flou à quatre : chacune se
    // lisse le long de x une fois, et se garde tant qu'on la relit.
    let ligne_du_masque = |r: usize, sortie: &mut Vec<f32>| {
        let o = |k: usize| f32::from(masque.octets[r * masque.largeur + k]) / 255.0;
        sortie.clear();
        sortie.extend(
            colonnes_du_masque
                .iter()
                .map(|c| o(c.avant) + (o(c.apres) - o(c.avant)) * c.t),
        );
    };
    let ligne_du_flou = |r: usize, sortie: &mut Vec<Pixel>| {
        let p = |k: usize| flou.pixels[r * flou.largeur + k];
        sortie.clear();
        sortie.extend(colonnes_du_flou.iter().map(|c| {
            let (a, b) = (p(c.avant), p(c.apres));
            std::array::from_fn(|k| a[k] + (b[k] - a[k]) * c.t)
        }));
    };
    let (mut masques, mut flous) = (Lissees::default(), Lissees::default());
    for y in 0..hauteur {
        let (rm, rf) = (
            Pas::de(centre(y), masque.hauteur),
            Pas::de(sous(y), flou.hauteur),
        );
        let (ma, mb) = masques.deux((rm.avant, rm.apres), &ligne_du_masque);
        let (fa, fb) = if plan.etages == 0 {
            (&[][..], &[][..])
        } else {
            flous.deux((rf.avant, rf.apres), &ligne_du_flou)
        };
        for x in 0..largeur {
            let m = (ma[x] + (mb[x] - ma[x]) * rm.t).clamp(0.0, 1.0);
            if m >= 1.0 {
                continue;
            }
            let f = if plan.etages == 0 {
                flou.a(x as isize, y as isize)
            } else {
                let (haut, bas) = (fa[x], fb[x]);
                std::array::from_fn(|k| haut[k] + (bas[k] - haut[k]) * rf.t)
            };
            let i = (y * largeur + x) * 4;
            for c in 0..4 {
                let v = f32::from(net[i + c]) * m + f[c] * (1.0 - m);
                net[i + c] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// **Deux rangées d'une grille, lissées le long de x, gardées** : la moitié d'une lecture
/// bilinéaire que les rangées voisines de l'écran relisent — calculée une fois, au lieu d'une fois
/// par pixel.
struct Lissees<T> {
    rangs: [Option<usize>; 2],
    rangees: [Vec<T>; 2],
}

impl<T> Default for Lissees<T> {
    fn default() -> Self {
        Self {
            rangs: [None, None],
            rangees: [Vec::new(), Vec::new()],
        }
    }
}

impl<T> Lissees<T> {
    /// Les rangées `avant` et `apres`, lissées par `lisser` si elles ne sont pas déjà gardées.
    fn deux(
        &mut self,
        (avant, apres): (usize, usize),
        lisser: &impl Fn(usize, &mut Vec<T>),
    ) -> (&[T], &[T]) {
        for r in [avant, apres] {
            if !self.rangs.contains(&Some(r)) {
                let libre = (0..2)
                    .find(|&i| self.rangs[i] != Some(avant) && self.rangs[i] != Some(apres))
                    .unwrap_or(0);
                lisser(r, &mut self.rangees[libre]);
                self.rangs[libre] = Some(r);
            }
        }
        let place = |r: usize| (0..2).find(|&i| self.rangs[i] == Some(r)).unwrap_or(0);
        (&self.rangees[place(avant)], &self.rangees[place(apres)])
    }
}

/// **La lecture bilinéaire d'une image, ses pas déjà faits** : le même calcul que [`lire`], dans le
/// même ordre — le long de la rangée, puis entre les deux rangées.
fn bilineaire(img: &Image, (colonne, rangee): (&Pas, &Pas)) -> Pixel {
    let p = |r: usize, c: usize| img.pixels[r * img.largeur + c];
    let (a, b) = (
        p(rangee.avant, colonne.avant),
        p(rangee.avant, colonne.apres),
    );
    let (c, d) = (
        p(rangee.apres, colonne.avant),
        p(rangee.apres, colonne.apres),
    );
    std::array::from_fn(|k| {
        let haut = a[k] + (b[k] - a[k]) * colonne.t;
        let bas = c[k] + (d[k] - c[k]) * colonne.t;
        haut + (bas - haut) * rangee.t
    })
}

/// **Un pas de lecture bilinéaire** le long d'un axe : les deux voisins du point `c` dans une grille
/// de `n` cases, les bords prolongés, et le poids du second — ce que [`lire`] calcule à chaque
/// pixel, calculé une fois par colonne ou par rangée.
struct Pas {
    avant: usize,
    apres: usize,
    t: f32,
}

impl Pas {
    fn de(c: f32, n: usize) -> Self {
        let c0 = c.floor();
        let borne = |i: isize| i.clamp(0, n as isize - 1) as usize;
        Self {
            avant: borne(c0 as isize),
            apres: borne(c0 as isize + 1),
            t: c - c0,
        }
    }
}

#[cfg(test)]
mod tests;
