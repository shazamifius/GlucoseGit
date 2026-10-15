//! **Le relevé** (P8, fiche 73) : le dessin vectoriel d'un nœud, relevé en primitives que la carte
//! graphique dessine ([`crate::present::formes_gpu`]).
//!
//! Tout y est à l'écran, à l'échelle exacte de la vue : il n'y a ni palier ni phase, puisque rien
//! ne se repeint — la carte dessine chaque image depuis les formes.
//!
//! # Ce qui ne se voit pas ne part pas
//!
//! Un nœud dessiné de près est plus grand que l'écran (c'est ce qui l'a fait dessiner ainsi) :
//! la plupart de ses lettres sont hors de lui. Une primitive dont la boîte ne touche pas l'écran
//! n'est pas relevée. La boîte d'un glyphe est celle de **toute sa police** (la table `head`),
//! jamais une estimation : aucun glyphe n'en sort.

use super::{sur_la_grille_des_pixels, Vectoriel};
use crate::present::formes_gpu::{Genre, Primitive, Source, PARTOUT};
use crate::typography::Police;
use glucose_core::courbes::{Contours, Quadratique};
use glucose_core::membrane_forme::Arrondi;
use glucose_math::{Commande, Forme};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tiny_skia::{Color, PathSegment};

/// **La tolérance d'une forme posée à l'écran** (CUBIQUE-1) : la moitié de ce que la sortie peut
/// dire — un niveau de couverture sur 255. Un bord déplacé de moins d'un demi-niveau ne change
/// aucun pixel écrit.
const DEMI_NIVEAU: f64 = 1.0 / 510.0;

/// Les primitives d'une image, relevées nœud après nœud.
pub(crate) struct Releve {
    pub primitives: Vec<Primitive>,
    /// L'encre des groupes ouverts, prémultipliée.
    encres: Vec<[f32; 4]>,
    ecran: (f32, f32),
    /// La boîte de chaque police, en em : gauche, bas, droite, haut.
    bornes: HashMap<&'static str, Option<[f32; 4]>>,
}

/// Une couleur prémultipliée.
fn premultipliee(c: Color) -> [f32; 4] {
    let a = c.alpha();
    [c.red() * a, c.green() * a, c.blue() * a, a]
}

impl Releve {
    /// Un relevé pour un écran de cette taille, en pixels.
    pub fn nouveau(ecran: (f32, f32)) -> Self {
        Self {
            primitives: Vec::new(),
            encres: Vec::new(),
            ecran,
            bornes: HashMap::new(),
        }
    }

    /// La boîte `(gauche, haut, droite, bas)` touche-t-elle l'écran ?
    fn voit(&self, [g, h, d, b]: [f32; 4]) -> bool {
        d >= 0.0 && b >= 0.0 && g <= self.ecran.0 && h <= self.ecran.1
    }

    fn arrondi(&mut self, forme: Arrondi, demi: f32, c: Color) {
        let e = forme.dilate(demi + 0.5);
        if c.alpha() <= 0.0 || !self.voit([e.gauche, e.haut, e.droite, e.bas]) {
            return;
        }
        self.primitives.push(Primitive {
            genre: Genre::Arrondi { forme, demi },
            couleur: premultipliee(c),
            rogne: PARTOUT,
        });
    }

    fn courbes(
        &mut self,
        source: Source,
        matrice: [f32; 6],
        (couleur, rogne): ([f32; 4], [f32; 4]),
    ) {
        self.primitives.push(Primitive {
            genre: Genre::Courbes { source, matrice },
            couleur,
            rogne,
        });
    }

    /// **Un chemin de `tiny-skia`, rempli** à l'écran par la règle des enroulements — ses
    /// cubiques à un demi-niveau près (CUBIQUE-1), nommé par ses points.
    pub fn chemin(&mut self, chemin: &tiny_skia::Path, c: Color) {
        let b = chemin.bounds();
        if c.alpha() <= 0.0
            || !self.voit([
                b.left() - 1.0,
                b.top() - 1.0,
                b.right() + 1.0,
                b.bottom() + 1.0,
            ])
        {
            return;
        }
        let contours = contours_d_un_chemin(chemin);
        let source = Source::Contours(empreinte(&contours), Arc::new(contours));
        self.courbes(source, IDENTITE, (premultipliee(c), PARTOUT));
    }

    /// La boîte de toute une police, en em — lue une fois par relevé.
    fn bornes_de(&mut self, police: Police) -> Option<[f32; 4]> {
        *self.bornes.entry(police.nom).or_insert_with(|| {
            let face = ttf_parser::Face::parse(police.octets, 0).ok()?;
            let (b, u) = (face.global_bounding_box(), f32::from(face.units_per_em()));
            Some([b.x_min, b.y_min, b.x_max, b.y_max].map(|v| f32::from(v) / u))
        })
    }
}

impl Vectoriel for Releve {
    fn voit_la_bande(&self, haut: f32, bas: f32) -> bool {
        bas >= 0.0 && haut <= self.ecran.1
    }

    fn rectangle(&mut self, (x, y, l, h): (f32, f32, f32, f32), rayon: f32, c: Color) {
        self.arrondi(Arrondi::nouveau(x, y, l, h, rayon), 0.0, c);
    }

    fn rectangle_net(&mut self, boite: (f32, f32, f32, f32), c: Color) {
        if let Some(r) = sur_la_grille_des_pixels(boite) {
            self.rectangle(r, 0.0, c);
        }
    }

    fn anneau(&mut self, milieu: Arrondi, demi: f32, c: Color) {
        self.arrondi(milieu, demi, c);
    }

    fn disque(&mut self, (cx, cy): (f32, f32), rayon: f32, c: Color) {
        let forme = Arrondi::nouveau(cx - rayon, cy - rayon, 2.0 * rayon, 2.0 * rayon, rayon);
        self.arrondi(forme, 0.0, c);
    }

    fn groupe(&mut self, _mots: &str, encre: Color) {
        self.encres.push(premultipliee(encre));
    }

    fn fin_du_groupe(&mut self) {
        self.encres.pop();
    }

    fn glyphe(&mut self, (police, indice): (Police, u16), (x, base): (f32, f32), corps: f32) {
        let Some(encre) = self.encres.last().copied().filter(|e| e[3] > 0.0) else {
            return;
        };
        let Some([g, b, d, h]) = self.bornes_de(police) else {
            return;
        };
        // La police est en em, l'ordonnée vers le haut ; l'écran en pixels, vers le bas.
        let boite = [
            x + g * corps,
            base - h * corps,
            x + d * corps,
            base - b * corps,
        ];
        if !self.voit(boite) {
            return;
        }
        let matrice = [corps, 0.0, 0.0, -corps, x, base];
        self.courbes(Source::Glyphe(police, indice), matrice, (encre, PARTOUT));
    }

    fn forme(
        &mut self,
        forme: &Forme,
        (bx, by, bl, bh): (f32, f32, f32, f32),
        trait_px: Option<f32>,
        c: Color,
    ) {
        let rogne = [bx, by, bx + bl, by + bh];
        if c.alpha() <= 0.0 || !self.voit(rogne) {
            return;
        }
        let (sx, sy, dx, dy) = forme.transformation((f64::from(bl), f64::from(bh)));
        let (x0, y0) = (f64::from(bx) + dx, f64::from(by) + dy);
        let encre = premultipliee(c);
        match trait_px {
            None => {
                let Some((source, m)) = forme_pleine(forme) else {
                    return;
                };
                // Le tracé, ramené à l'unité `m` de la forme, puis porté à l'écran.
                let (vx, vy, _, _) = forme.boite_de_vue;
                let matrice = [
                    (m * sx) as f32,
                    0.0,
                    0.0,
                    (m * sy) as f32,
                    (x0 + vx * sx) as f32,
                    (y0 + vy * sy) as f32,
                ];
                self.courbes(source, matrice, (encre, rogne));
            }
            Some(largeur) => {
                let point = |x: f64, y: f64| [x0 + x * sx, y0 + y * sy];
                if let Some(source) = forme_tracee(forme, point, largeur) {
                    self.courbes(source, IDENTITE, (encre, rogne));
                }
            }
        }
    }
}

/// **Une forme pleine de KaTeX, dans son unité** : son tracé ramené à la plus grande dimension
/// de sa boîte de vue (`m`), ses cubiques à une demi-unité du tracé près (CUBIQUE-1) — et
/// nommée par ce tracé, donc rangée une fois quelle que soit sa boîte.
fn forme_pleine(forme: &Forme) -> Option<(Source, f64)> {
    let (vx, vy, vl, vh) = forme.boite_de_vue;
    let m = vl.max(vh);
    if m.is_nan() || m <= 0.0 {
        return None;
    }
    let point = |x: f64, y: f64| [(x - vx) / m, (y - vy) / m];
    let mut c = Contours::nouveaux(0.5 / m);
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for commande in &forme.commandes {
        let (genre, nombres): (u8, &[f64]) = match commande {
            Commande::Aller(x, y) => {
                c.aller(point(*x, *y));
                (0, &[*x, *y])
            }
            Commande::Ligne(x, y) => {
                c.ligne(point(*x, *y));
                (1, &[*x, *y])
            }
            Commande::Cubique(a, b, c2, d, e, f) => {
                c.cubique(point(*a, *b), point(*c2, *d), point(*e, *f));
                (2, &[*a, *b, *c2, *d, *e, *f])
            }
            Commande::Fermer => {
                c.fermer();
                (3, &[])
            }
        };
        genre.hash(&mut h);
        for v in nombres {
            v.to_bits().hash(&mut h);
        }
    }
    for v in [vx, vy, vl, vh] {
        v.to_bits().hash(&mut h);
    }
    Some((Source::Contours(h.finish(), Arc::new(c.finir())), m))
}

/// **Une forme tracée de KaTeX** (les ratures de `\cancel`), à l'écran : son tracé, son trait
/// élargi par le traceur même du canevas, puis rempli.
fn forme_tracee(
    forme: &Forme,
    point: impl Fn(f64, f64) -> [f64; 2],
    largeur: f32,
) -> Option<Source> {
    let mut pb = tiny_skia::PathBuilder::new();
    let f = |p: [f64; 2]| (p[0] as f32, p[1] as f32);
    for commande in &forme.commandes {
        match *commande {
            Commande::Aller(x, y) => {
                let (x, y) = f(point(x, y));
                pb.move_to(x, y);
            }
            Commande::Ligne(x, y) => {
                let (x, y) = f(point(x, y));
                pb.line_to(x, y);
            }
            Commande::Cubique(a, b, c2, d, e, g) => {
                let ((a, b), (c2, d), (e, g)) = (f(point(a, b)), f(point(c2, d)), f(point(e, g)));
                pb.cubic_to(a, b, c2, d, e, g);
            }
            Commande::Fermer => pb.close(),
        }
    }
    let trait_ = tiny_skia::Stroke {
        width: largeur,
        ..Default::default()
    };
    let rempli = pb.finish()?.stroke(&trait_, 1.0)?;
    let contours = contours_d_un_chemin(&rempli);
    Some(Source::Contours(empreinte(&contours), Arc::new(contours)))
}

/// L'empreinte de contours : ce qui les nomme dans l'atlas, par chacun de leurs points.
fn empreinte(contours: &[Vec<Quadratique>]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for q in contours.iter().flatten() {
        for v in q.debut.iter().chain(&q.controle).chain(&q.fin) {
            v.to_bits().hash(&mut h);
        }
    }
    h.finish()
}

/// La matrice qui laisse une forme à l'écran telle qu'elle est.
const IDENTITE: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// Les contours d'un chemin de `tiny-skia`, à l'écran (CUBIQUE-1 : un demi-niveau).
fn contours_d_un_chemin(chemin: &tiny_skia::Path) -> Vec<Vec<Quadratique>> {
    let mut c = Contours::nouveaux(DEMI_NIVEAU);
    let p = |q: tiny_skia::Point| [f64::from(q.x), f64::from(q.y)];
    for s in chemin.segments() {
        match s {
            PathSegment::MoveTo(a) => c.aller(p(a)),
            PathSegment::LineTo(a) => c.ligne(p(a)),
            PathSegment::QuadTo(a, b) => c.quadratique(p(a), p(b)),
            PathSegment::CubicTo(a, b, d) => c.cubique(p(a), p(b), p(d)),
            PathSegment::Close => c.fermer(),
        }
    }
    c.finir()
}
