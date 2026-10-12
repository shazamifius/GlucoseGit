//! **La couleur d'une image telle qu'on la voit, et l'ordre qu'on en tire** (ORDONNER-2).
//!
//! # Ce que Glucose Tauri faisait, et pourquoi ce n'est pas repris
//!
//! Il réduisait chaque image à 32 × 32 pixels, prenait la moyenne de leur teinte HSL pondérée
//! par la saturation, et triait l'arc-en-ciel à partir du rouge. Les images « presque
//! grises » partaient à la fin, désignées par quatre seuils réglés à l'œil (saturation 0,15,
//! luminosité 0,1 et 0,92, poids 0,01) ; et une image qui tardait à se lire recevait une
//! luminosité de 0,5, sans que personne le sache. Trois défauts :
//!
//! * **HSL n'est pas un espace de perception** : un jaune et un bleu de même « luminosité » HSL
//!   n'ont pas du tout la même clarté à l'œil ;
//! * **les seuils n'ont pas de sens** hors de l'œil qui les a réglés ;
//! * **le rouge comme début du cercle** coupe en deux ce qui se ressemble, si le document a des
//!   rouges et des magentas.
//!
//! # Oklab, l'espace où la distance est ce que l'œil voit
//!
//! Björn Ottosson l'a construit en 2020 pour prédire la **clarté**, la **chroma** et la
//! **teinte** perçues ([Oklab](https://bottosson.github.io/posts/oklab/)) ; CSS Color 4 l'a
//! adopté. La clarté `L` va de 0 (noir) à 1 (blanc), et la teinte est l'angle du point dans le
//! plan `(a, b)`, sa chroma la distance à l'axe des gris.
//!
//! # Gris : une couleur qu'on ne distingue pas d'un gris
//!
//! Une image est grise quand sa couleur **ne se distingue pas** du gris de même clarté. La
//! distance entre les deux, dans Oklab, est exactement la chroma ; et la plus petite
//! différence que l'œil voit y vaut 0,02 — la « différence à peine visible » que CSS Color 4
//! emploie pour ramener une couleur dans un écran. Ce n'est pas un réglage : c'est la mesure
//! de ce qu'on voit. Le noir et le blanc s'y rangent d'eux-mêmes, leur chroma tendant vers
//! zéro.
//!
//! # Où le cercle commence : au plus grand vide
//!
//! Les teintes forment un cercle, et un ordre doit le couper quelque part. Le couper là où
//! les images sont **le plus loin** les unes des autres sépare ce qui se ressemble le moins ;
//! partout ailleurs, deux voisines sur le cercle se retrouveraient aux deux bouts du rangement.
//! Aucun angle de départ n'est choisi : les images le donnent.

/// La plus petite différence que l'œil distingue, en distance Oklab (CSS Color 4, la
/// correspondance des gamuts).
pub const DIFFERENCE_A_PEINE_VISIBLE: f64 = 0.02;

/// Une couleur en Oklch : clarté, chroma, teinte (en radians, dans `[-π, π]`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Oklch {
    pub l: f64,
    pub c: f64,
    pub h: f64,
}

/// Un canal sRGB (`0..=1`) ramené à la lumière qu'il représente (IEC 61966-2-1).
fn lineaire(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// **Une couleur sRGB en Oklab**, par les matrices d'Ottosson.
pub fn oklab([r, g, b]: [f64; 3]) -> [f64; 3] {
    let (r, g, b) = (lineaire(r), lineaire(g), lineaire(b));
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}

impl Oklch {
    /// **La couleur d'un pixel prémultiplié** — ce que la pyramide d'une image garde. `None`
    /// pour un pixel entièrement transparent : il n'a pas de couleur.
    pub fn du_pixel([r, g, b, a]: [u8; 4]) -> Option<Self> {
        if a == 0 {
            return None;
        }
        // Prémultiplié : la couleur droite est le rapport au canal alpha.
        let a = f64::from(a);
        let droit = |c: u8| f64::from(c) / a;
        let [l, aa, bb] = oklab([droit(r), droit(g), droit(b)]);
        Some(Self {
            l,
            c: aa.hypot(bb),
            h: bb.atan2(aa),
        })
    }

    /// **On la distingue d'un gris** de même clarté.
    pub fn a_une_teinte(self) -> bool {
        self.c >= DIFFERENCE_A_PEINE_VISIBLE
    }
}

/// Comment ranger des couleurs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    /// L'arc-en-ciel coupé au plus grand vide, puis les gris du sombre au clair.
    Teinte,
    /// Par clarté perçue, du sombre au clair.
    SombreVersClair,
    /// Par clarté perçue, du clair au sombre.
    ClairVersSombre,
}

/// **L'ordre des couleurs selon ce tri** : les indices de `couleurs`, du premier rangé au
/// dernier. Ce qui n'a pas de couleur vient à la fin, dans l'ordre où il était ; à égalité, on
/// garde aussi l'ordre donné — les tris sont stables, et ils reçoivent les couleurs dans
/// l'ordre des indices.
pub fn ordonner(couleurs: &[Option<Oklch>], tri: Tri) -> Vec<usize> {
    let mut connues: Vec<(usize, Oklch)> = couleurs
        .iter()
        .enumerate()
        .filter_map(|(i, c)| Some((i, (*c)?)))
        .collect();
    let rangees = match tri {
        Tri::Teinte => par_teinte(connues),
        Tri::SombreVersClair | Tri::ClairVersSombre => {
            connues.sort_by(|(_, a), (_, b)| {
                if tri == Tri::SombreVersClair {
                    a.l.total_cmp(&b.l)
                } else {
                    b.l.total_cmp(&a.l)
                }
            });
            connues.into_iter().map(|(i, _)| i).collect()
        }
    };
    let sans_couleur = couleurs
        .iter()
        .enumerate()
        .filter(|(_, c)| c.is_none())
        .map(|(i, _)| i);
    rangees.into_iter().chain(sans_couleur).collect()
}

/// Les teintes autour du cercle, coupé au plus grand vide ; puis les gris, du sombre au clair.
fn par_teinte(connues: Vec<(usize, Oklch)>) -> Vec<usize> {
    let (mut teintes, mut gris): (Vec<_>, Vec<_>) =
        connues.into_iter().partition(|(_, c)| c.a_une_teinte());
    teintes.sort_by(|(_, a), (_, b)| a.h.total_cmp(&b.h));
    let depart = apres_le_plus_grand_vide(&teintes);
    teintes.rotate_left(depart);
    gris.sort_by(|(_, a), (_, b)| a.l.total_cmp(&b.l));
    teintes.into_iter().chain(gris).map(|(i, _)| i).collect()
}

/// L'indice, dans des teintes triées, de celle qui suit **le plus grand écart** entre deux
/// voisines du cercle — le dernier écart fait le tour, du dernier angle au premier.
fn apres_le_plus_grand_vide(teintes: &[(usize, Oklch)]) -> usize {
    let n = teintes.len();
    if n < 2 {
        return 0;
    }
    let ecart = |k: usize| {
        let (avant, apres) = (teintes[k].1.h, teintes[(k + 1) % n].1.h);
        let d = apres - avant;
        if k + 1 == n {
            d + std::f64::consts::TAU
        } else {
            d
        }
    };
    // À égalité, l'écart qui fait le tour l'emporte : des teintes régulièrement espacées
    // gardent l'ordre de leurs angles.
    let plus_grand = (0..n).fold(n - 1, |meilleur, k| {
        if ecart(k) > ecart(meilleur) {
            k
        } else {
            meilleur
        }
    });
    (plus_grand + 1) % n
}

#[cfg(test)]
mod tests;
