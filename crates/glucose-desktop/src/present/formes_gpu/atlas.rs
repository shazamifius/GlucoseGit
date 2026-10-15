//! **L'atlas des formes** (fiche 73) : chaque forme rangée une fois — ses courbes, ses bandes —,
//! mise bout à bout avec les autres dans deux tableaux que la carte lit.
//!
//! # Une lettre se range une fois, en em
//!
//! Un glyphe ne dépend ni de sa taille ni de sa place : une transformation le pose. Il se range
//! **dans l'em de sa police** (ses unités divisées par ses unités par em), comme chez Lengyel —
//! les deux seuils de la loi sont faits pour des coordonnées de cet ordre —, et ses cubiques
//! s'approchent à une demi-unité de sa grille (CUBIQUE-1).
//!
//! # Ce qui ne sert plus s'oublie — et l'atlas se resserre
//!
//! Comme les textures (VRAM-1) : ce qu'une image n'a pas demandé peut partir. Mais un atlas est
//! un seul tableau, et l'on ne retire pas une forme du milieu : il se **resserre** — tout ce qui a
//! servi à la dernière image recopié au début — dès que ce qui ne sert plus pèse plus que ce qui
//! sert. C'est la règle du tableau qui double : chaque resserrement coûte au plus ce que les
//! formes oubliées ont coûté à ranger, et aucun nombre n'est choisi.

use crate::typography::Police;
use glucose_core::courbes::emballage::{emballer, Emballage};
use glucose_core::courbes::{Contours, Quadratique};
use std::collections::HashMap;
use std::sync::Arc;

/// **Ce qu'une forme est**, indépendamment de sa place.
#[derive(Debug, Clone)]
pub enum Source {
    /// Un glyphe d'une police embarquée, par son indice.
    Glyphe(Police, u16),
    /// Des contours déjà bâtis, nommés par leur empreinte : une forme de formule, un trait.
    Contours(u64, Arc<Vec<Vec<Quadratique>>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Cle {
    Glyphe(&'static str, u16),
    Contours(u64),
}

impl Source {
    fn cle(&self) -> Cle {
        match self {
            Self::Glyphe(p, i) => Cle::Glyphe(p.nom, *i),
            Self::Contours(e, _) => Cle::Contours(*e),
        }
    }
}

/// **Une forme rangée** : où ses courbes et ses bandes commencent, et ce qu'il faut pour les lire.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rangee {
    pub boite: [f32; 4],
    pub nombre_de_bandes: [u32; 2],
    pub transformation: [f32; 4],
    pub courbes: u32,
    pub bandes: u32,
    taille: (u32, u32),
}

struct Entree {
    /// `None` : une forme sans contour (une espace).
    rangee: Option<Rangee>,
    /// La dernière image qui l'a demandée.
    vue: u64,
}

/// Les formes rangées, et les deux tableaux qui les portent.
#[derive(Default)]
pub struct Atlas {
    entrees: HashMap<Cle, Entree>,
    pub courbes: Vec<[f32; 4]>,
    pub bandes: Vec<u32>,
    /// Ce que la carte a déjà reçu de chaque tableau : le reste est à lui envoyer.
    pub envoyees: (usize, usize),
    /// Les polices déjà lues.
    polices: HashMap<&'static str, Option<ttf_parser::Face<'static>>>,
    image: u64,
}

impl Atlas {
    /// Ouvre une image : ce qu'elle demande sera gardé au prochain resserrement.
    pub fn ouvrir(&mut self) {
        self.image += 1;
    }

    /// **Où est cette forme**, rangée à sa première demande — `None` si elle n'a pas de contour.
    pub fn ranger(&mut self, source: &Source) -> Option<Rangee> {
        let image = self.image;
        let cle = source.cle();
        if let Some(e) = self.entrees.get_mut(&cle) {
            e.vue = image;
            return e.rangee;
        }
        let emballage = match source {
            Source::Glyphe(police, indice) => self.contours_du_glyphe(*police, *indice),
            Source::Contours(_, contours) => emballer(contours),
        };
        let rangee = emballage.map(|e| self.ajouter(&e));
        self.entrees.insert(cle, Entree { rangee, vue: image });
        rangee
    }

    /// Ajoute une forme au bout des deux tableaux.
    fn ajouter(&mut self, e: &Emballage) -> Rangee {
        let rangee = Rangee {
            boite: e.boite,
            nombre_de_bandes: e.nombre_de_bandes,
            transformation: e.transformation,
            courbes: self.courbes.len() as u32,
            bandes: self.bandes.len() as u32,
            taille: (e.courbes.len() as u32, e.bandes.len() as u32),
        };
        self.courbes.extend_from_slice(&e.courbes);
        self.bandes.extend_from_slice(&e.bandes);
        rangee
    }

    /// Le contour d'un glyphe, dans l'em de sa police, rangé.
    fn contours_du_glyphe(&mut self, police: Police, indice: u16) -> Option<Emballage> {
        let face = self
            .polices
            .entry(police.nom)
            .or_insert_with(|| ttf_parser::Face::parse(police.octets, 0).ok())
            .as_ref()?;
        let unite = 1.0 / f64::from(face.units_per_em());
        // CUBIQUE-1 : une demi-unité de la grille de la police.
        let mut bati = Bati(Contours::nouveaux(unite / 2.0), unite);
        face.outline_glyph(ttf_parser::GlyphId(indice), &mut bati)?;
        emballer(&bati.0.finir())
    }

    /// **Resserre l'atlas** si ce qui n'a pas servi à cette image pèse plus que ce qui a servi :
    /// le vivant est recopié au début, et tout repart vers la carte. Rend `true` s'il l'a fait.
    pub fn resserrer(&mut self) -> bool {
        let image = self.image;
        let vivant: usize = self
            .entrees
            .values()
            .filter(|e| e.vue == image)
            .filter_map(|e| e.rangee)
            .map(|r| (r.taille.0 + r.taille.1) as usize)
            .sum();
        let total = self.courbes.len() + self.bandes.len();
        if total - vivant <= vivant {
            return false;
        }
        let (anciennes, anciennes_bandes) = (
            std::mem::take(&mut self.courbes),
            std::mem::take(&mut self.bandes),
        );
        self.entrees.retain(|_, e| e.vue == image);
        for e in self.entrees.values_mut() {
            if let Some(r) = e.rangee.as_mut() {
                let (c, b) = (r.courbes as usize, r.bandes as usize);
                let (nc, nb) = (r.taille.0 as usize, r.taille.1 as usize);
                r.courbes = self.courbes.len() as u32;
                r.bandes = self.bandes.len() as u32;
                self.courbes.extend_from_slice(&anciennes[c..c + nc]);
                self.bandes.extend_from_slice(&anciennes_bandes[b..b + nb]);
            }
        }
        self.envoyees = (0, 0);
        true
    }

    /// Combien de formes l'atlas connaît.
    #[cfg(test)]
    pub fn combien(&self) -> usize {
        self.entrees.len()
    }

    /// **La forme rangée, telle que la carte la lit** — ses cases relues dans les deux tableaux :
    /// ce que la loi du noyau compare à la carte.
    #[cfg(test)]
    pub fn emballage(&mut self, source: &Source) -> Option<Emballage> {
        let r = self.ranger(source)?;
        let (c, b) = (r.courbes as usize, r.bandes as usize);
        Some(Emballage {
            courbes: self.courbes[c..c + r.taille.0 as usize].to_vec(),
            bandes: self.bandes[b..b + r.taille.1 as usize].to_vec(),
            boite: r.boite,
            nombre_de_bandes: r.nombre_de_bandes,
            transformation: r.transformation,
        })
    }
}

/// Les gestes d'un contour de police, en em.
struct Bati(Contours, f64);

impl ttf_parser::OutlineBuilder for Bati {
    fn move_to(&mut self, x: f32, y: f32) {
        let u = self.1;
        self.0.aller([f64::from(x) * u, f64::from(y) * u]);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let u = self.1;
        self.0.ligne([f64::from(x) * u, f64::from(y) * u]);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let u = self.1;
        self.0.quadratique(
            [f64::from(x1) * u, f64::from(y1) * u],
            [f64::from(x) * u, f64::from(y) * u],
        );
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let u = self.1;
        self.0.cubique(
            [f64::from(x1) * u, f64::from(y1) * u],
            [f64::from(x2) * u, f64::from(y2) * u],
            [f64::from(x) * u, f64::from(y) * u],
        );
    }

    fn close(&mut self) {
        self.0.fermer();
    }
}
