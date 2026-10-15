//! **GRAND-1 — une grande lettre se trace** (fiche 72).
//!
//! # Ce qu'il a vu
//!
//! Le 15/10, ses captures : de près, un texte reste flou « un bon moment », en gros pixels, un
//! carré net au milieu — *« illisible, impossible de travailler dessus »*, et *« comment ça se
//! fait qu'on a besoin d'autant de pixelisation sur un simple texte ? »*. Les photos, elles,
//! restent nettes.
//!
//! # Pourquoi
//!
//! Une lettre se dessinait toujours par son **image** : `fontdue` la rastérise entière, à sa
//! taille à l'écran, puis elle est décalée d'une fraction de pixel et gardée en cache. Son coût
//! est sa surface. De près, une lettre fait mille pixels de haut : chaque carré de la découpe
//! (DE-PRES-1) qui la touche la faisait rastériser **entière** — un million de pixels pour en
//! poser quelques milliers —, et la cascade (CASCADE-2), qui ne rend que ce que le temps libre
//! de chaque image permet, laissait l'ancienne texture agrandie à l'écran pendant tout ce temps.
//! Une borne, `MAX_FONT_SIZE` (512 px), empêchait le pire : elle **réduisait** les lettres au-delà.
//!
//! # La règle
//!
//! Au-delà d'un **demi-carré** ([`glucose_core::tuile::COTE`] / 2, 128 px d'em), la lettre se
//! dessine par son **contour** — celui des tables de la police, lu par `ttf-parser`
//! (`decisions/13`), le même que le SVG écrit (P5) — rempli par le rastériseur de la voie
//! processeur. Son coût est celui des pixels qu'elle couvre **dans ce qu'on dessine**, jamais
//! plus : un carré coûte un carré, quel que soit le zoom. C'est ce que fait Skia, le moteur de
//! Chrome — donc de Glucose Tauri —, au-delà de 256 à 384 pixels selon le système
//! (`GrContextOptions` : « glyphs are drawn as individual paths »). En deçà, l'image de la
//! lettre reste : petite, elle se réutilise d'un carré et d'une image à l'autre, et son cache
//! vaut son coût.
//!
//! # Le seuil, mesuré (`bench_de_pres`, son écran, une carte qui le couvre, à froid)
//!
//! Le seuil est un nombre de mesure, pas une loi : le croisement des deux coûts. Les valeurs
//! essayées, en millisecondes pour tout l'écran (et le pire carré) :
//!
//! ```text
//!   seuil      ×4 (56 px)   ×8 (112 px)   ×16 (224 px)   ×32 (448 px)   ×50 (700 px)
//!   aucun       2,8           16,3          33,0 (7,2)     65,6 (26,4)    53,0, réduites
//!   256 px      4,0           14,7          36,7 (9,0)     19,0 (3,1)     18,8 (2,7)
//!   128 px      2,6           17,5          21,3 (3,0)     19,7 (2,5)     18,7 (3,8)
//!    64 px      2,6           14,9          20,3 (3,5)     22,1 (3,3)     20,5 (5,0)
//!    32 px      5,5           15,8          26,5 (4,1)     18,6 (3,7)     21,3 (3,8)
//! ```
//!
//! L'image gagne nettement en deçà de 64 px (×4 : 2,6 contre 5,5) ; le tracé, au-delà de 128
//! (×16 : 21 contre 37). Entre les deux, le bruit. Le demi-carré est dans le creux, et se dit
//! par la grandeur qui existe déjà.
//!
//! La position n'est plus arrondie au quart de pixel (GLYPH-1) : le contour se pose à sa place
//! exacte.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tiny_skia::{Color, FillRule, Paint, Path, PathBuilder, PixmapMut, Transform};

/// Le contour d'un glyphe, dans les unités de sa police (l'axe des ordonnées vers le haut).
struct Bati(PathBuilder);

impl ttf_parser::OutlineBuilder for Bati {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.0.close();
    }
}

/// **Les contours des polices embarquées**, lus une fois par glyphe et gardés : un contour ne
/// dépend ni de la taille ni de la place, il se transforme à chaque pose.
pub struct Contours {
    polices: Vec<ttf_parser::Face<'static>>,
    cache: RefCell<HashMap<(usize, u16), Contour>>,
}

/// Le contour d'un glyphe, partagé ; absent pour un glyphe qui n'en a pas (une espace).
type Contour = Option<Rc<Path>>;

impl Contours {
    /// Les contours de ces polices, dans l'ordre où la typographie les numérote.
    pub fn de(octets: &[&'static [u8]]) -> Self {
        let polices = octets
            .iter()
            .map(|o| {
                ttf_parser::Face::parse(o, 0).expect(
                    "police intégrée par include_bytes! — corruption du binaire si ceci échoue",
                )
            })
            .collect();
        Self {
            polices,
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Le contour de ce glyphe, s'il en a un (une espace n'en a pas).
    fn contour(&self, police: usize, indice: u16) -> Contour {
        self.cache
            .borrow_mut()
            .entry((police, indice))
            .or_insert_with(|| {
                let mut bati = Bati(PathBuilder::new());
                self.polices[police].outline_glyph(ttf_parser::GlyphId(indice), &mut bati)?;
                bati.0.finish().map(Rc::new)
            })
            .clone()
    }

    /// **Remplit ce glyphe** de cette couleur, sa plume en `x`, sa ligne de base en `base`, à ce
    /// corps — l'em en pixels, comme `fontdue` le compte. Ce qui tombe hors du tampon ne coûte
    /// rien : le rastériseur ne parcourt que ce qu'il couvre.
    pub fn remplir(
        &self,
        pixmap: &mut PixmapMut,
        (police, indice): (usize, u16),
        (x, base): (f32, f32),
        corps: f32,
        couleur: Color,
    ) {
        let Some(contour) = self.contour(police, indice) else {
            return;
        };
        let s = corps / f32::from(self.polices[police].units_per_em());
        let mut paint = Paint::default();
        paint.set_color(couleur);
        paint.anti_alias = true;
        let pose = Transform::from_row(s, 0.0, 0.0, -s, x, base);
        // TrueType et CFF remplissent par l'enroulement (non nul).
        pixmap.fill_path(&contour, &paint, FillRule::Winding, pose, None);
    }
}

/// **Cette lettre se trace-t-elle ?** Plus grande qu'un demi-carré : son image coûte alors
/// plus que son tracé (le journal des mesures est en tête du module).
pub fn se_trace(corps: f32) -> bool {
    corps > (glucose_core::tuile::COTE / 2) as f32
}

#[cfg(test)]
mod tests {
    use crate::typography::{Face, TextStyle, Typography};
    use tiny_skia::{Color, Pixmap};

    /// La boîte des pixels encrés d'un tampon : (gauche, haut, droite, bas), bornes incluses.
    fn encre(p: &Pixmap) -> Option<(u32, u32, u32, u32)> {
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
        for (i, px) in p.pixels().iter().enumerate() {
            if px.alpha() > 127 {
                let (x, y) = (i as u32 % p.width(), i as u32 / p.width());
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
        (x0 <= x1).then_some((x0, y0, x1, y1))
    }

    /// **Une grande lettre tracée tombe dans la boîte que `fontdue` lui donne** — un juge qui
    /// n'est pas le tracé : la même police, sa propre lecture des tables. Même taille, même
    /// place, à l'endroit : un pixel d'écart au plus, celui de l'anticrénelage.
    #[test]
    fn test_grand_1_la_lettre_tracee_tombe_dans_sa_boite() {
        let typo = Typography::new();
        let lettres = [
            ('A', Face::Regular),
            ('g', Face::Italic),
            ('W', Face::Bold),
            ('\u{1F600}', Face::Regular), // par le repli des emojis (EMOJI-1)
        ];
        for (ch, face) in lettres {
            let corps = 300.0;
            assert!(super::se_trace(corps));
            let mut p = Pixmap::new(600, 600).expect("tampon");
            let style = TextStyle {
                size: corps,
                color: Color::BLACK,
                face,
            };
            let (x, y) = (100.0, 50.0);
            typo.draw_text(&mut p.as_mut(), &ch.to_string(), x, y, style);
            let (x0, y0, x1, y1) = encre(&p).expect("de l'encre");
            let (police, indice) = typo.police(ch, face);
            let m = police.metrics_indexed(indice, corps);
            // La boîte de fontdue : `xmin` depuis la plume, `ymin` depuis la ligne de base vers
            // le haut ; la ligne de base est à `y + corps`.
            let gauche = x + m.xmin as f32;
            let bas = y + corps - m.ymin as f32;
            let (droite, haut) = (gauche + m.width as f32, bas - m.height as f32);
            let pres = |a: u32, b: f32| (a as f32 - b).abs() <= 1.5;
            assert!(
                pres(x0, gauche) && pres(x1 + 1, droite),
                "{ch} : {x0}..{x1} contre {gauche}..{droite}"
            );
            assert!(
                pres(y0, haut) && pres(y1 + 1, bas),
                "{ch} : {y0}..{y1} contre {haut}..{bas}"
            );
        }
    }

    /// **Une lettre plus petite que le seuil garde son image** — et une plus grande n'en a pas :
    /// le seuil sépare les deux voies, et lui seul.
    #[test]
    fn test_grand_1_le_seuil_separe_l_image_du_trace() {
        let typo = Typography::new();
        let seuil = (glucose_core::tuile::COTE / 2) as f32;
        assert!(typo.get_glyph('A', seuil, Face::Regular).is_some());
        assert!(typo.get_glyph('A', seuil + 1.0, Face::Regular).is_none());
    }

    /// **Un titre détouré, de près, se trace aussi** : neuf passes d'une lettre de dix millions
    /// de pixels dans un petit tampon se font en un temps borné par le tampon — par son image,
    /// elles demanderaient cent téraoctets. Le H est centré sur le tampon par sa propre boîte :
    /// sa barre le couvre.
    #[test]
    fn test_grand_1_le_texte_detoure_se_trace_aussi() {
        let typo = Typography::new();
        let corps = 1e7;
        let m = typo.font(Face::Bold).metrics('H', corps);
        let x = 32.0 - m.xmin as f32 - m.width as f32 / 2.0;
        let y = 32.0 - corps + m.ymin as f32 + m.height as f32 / 2.0;
        let mut p = Pixmap::new(64, 64).expect("tampon");
        let style = TextStyle {
            size: corps,
            color: Color::BLACK,
            face: Face::Bold,
        };
        let fin = typo.draw_text_with_outline(&mut p.as_mut(), "H", x, y, style, Color::WHITE);
        assert_eq!(fin, x + typo.measure_text("H", corps, Face::Bold).0);
        assert_eq!(
            encre(&p),
            Some((0, 0, 63, 63)),
            "la barre d'un H géant couvre le tampon"
        );
    }

    /// **Un décalage entier déplace la lettre tracée d'autant, au bit près** — celui des
    /// panneaux mis en cache (GLYPH-1) : dessinée décalée de (10, 20), elle est la même que
    /// dessinée dix et vingt pixels plus loin.
    #[test]
    fn test_grand_1_le_decalage_entier_est_exact() {
        let typo = Typography::new();
        let style = TextStyle {
            size: 200.0,
            color: Color::BLACK,
            face: Face::Regular,
        };
        let mut decale = Pixmap::new(300, 300).expect("tampon");
        let mut direct = Pixmap::new(300, 300).expect("tampon");
        typo.draw_text_offset(&mut decale.as_mut(), "Ag", 3.25, 4.5, (10, 20), style);
        typo.draw_text(&mut direct.as_mut(), "Ag", 13.25, 24.5, style);
        assert!(encre(&direct).is_some());
        assert!(decale.data() == direct.data());
    }
}
