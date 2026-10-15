//! **Le peintre vectoriel** (P5, fiche 69) : ce que le rendu dessine en pixels, écrit en SVG.
//!
//! # Une sœur pour chaque dessin, et aucune géométrie de plus
//!
//! L'export SVG de Glucose Tauri recomposait sa scène de son côté (`scene.ts`, `toSvg.ts`) : un
//! second jeu de marges, de rayons et de couleurs, qui divergeait du canevas à chaque retouche
//! de l'un sans l'autre. Ici, chaque module du rendu qui dessine un nœud porte, à côté de son
//! dessin, **sa sœur vectorielle** — la même mise en page, les mêmes constantes, la même passe —,
//! qui écrit dans ce peintre au lieu de poser des pixels. Le SVG d'un tableau est donc ce que
//! le canevas montre, dans le cadre de l'export en image (EXPORT-PNG-1), au même 100 %.
//!
//! # Ce qu'un lecteur doit trouver
//!
//! Du SVG 1.1 sans script : des `<rect>`, des `<path>`, des glyphes en tracés posés là où le
//! canevas les pose ([`glyphes`], fiche 70 — aucune police à demander au lecteur), des
//! `<image>` qui portent leurs octets d'origine en `data:` — un fichier qui se suffit.

use std::collections::HashMap;
use tiny_skia::Color;

mod glyphes;

/// **Un SVG en cours d'écriture** : ses définitions (dégradés, filtres) et ses éléments, dans
/// l'ordre où ils se peignent.
#[derive(Debug, Default)]
pub struct Svg {
    defs: String,
    corps: String,
    /// De quoi nommer chaque définition d'un nom à elle.
    suivant: usize,
    /// Les glyphes déjà définis, par leur police et leur indice — `None` pour un glyphe sans
    /// contour (fiche 70, [`glyphes`]).
    glyphes: HashMap<(&'static str, u16), Option<glyphes::Defini>>,
}

/// `#rrggbb` d'une couleur, son opacité à part.
pub fn hex((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// Une couleur de `tiny-skia` : `#rrggbb` et son opacité.
pub fn couleur(c: Color) -> (String, f32) {
    let u = c.to_color_u8();
    (hex((u.red(), u.green(), u.blue())), c.alpha())
}

/// Ce qu'un texte devient dans du XML.
pub fn echapper(texte: &str) -> String {
    let mut s = String::with_capacity(texte.len());
    for ch in texte.chars() {
        match ch {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            '"' => s.push_str("&quot;"),
            // Un caractère de contrôle n'a pas de place dans du XML 1.0.
            c if (c as u32) < 0x20 && c != '\t' => {}
            c => s.push(c),
        }
    }
    s
}

/// Un nombre, sans décimales inutiles : `12`, `12.5`, `0.333`.
pub fn n(v: f32) -> String {
    let arrondi = (v * 1000.0).round() / 1000.0;
    if arrondi == arrondi.trunc() {
        format!("{}", arrondi as i64)
    } else {
        format!("{arrondi}")
    }
}

impl Svg {
    pub fn new() -> Self {
        Self::default()
    }

    /// Un nom neuf pour une définition.
    pub fn nom(&mut self, prefixe: &str) -> String {
        self.suivant += 1;
        format!("{prefixe}-{}", self.suivant)
    }

    /// Ajoute une définition telle quelle.
    pub fn definir(&mut self, definition: &str) {
        self.defs.push_str(definition);
    }

    /// Ajoute un élément tel quel.
    pub fn poser(&mut self, element: &str) {
        self.corps.push_str(element);
        self.corps.push('\n');
    }

    /// **Un rectangle aux coins circulaires**, rempli d'une couleur à cette opacité.
    pub fn rectangle(
        &mut self,
        (x, y, l, h): (f32, f32, f32, f32),
        rayon: f32,
        (c, a): (&str, f32),
    ) {
        let e = format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{c}" fill-opacity="{}"/>"#,
            n(x),
            n(y),
            n(l),
            n(h),
            n(rayon_tenu(rayon, (l, h))),
            n(a)
        );
        self.poser(&e);
    }

    /// **Une lueur** : ce rectangle arrondi, de cette teinte, flouté d'une gaussienne
    /// d'écart-type `sigma` — l'ombre de boîte de Glucose Tauri, exactement (HALO-1).
    ///
    /// **La région du filtre est la boîte élargie de `portee`** — la distance au-delà de
    /// laquelle cette lueur ne change plus aucun niveau ([`super::halo::portee_du_flou`],
    /// HALO-2), propre à chaque lueur. La fiche 70 déclarait ±100 000 unités pour toutes : un
    /// lecteur floutait un carré de 200 000 de côté par lueur — 28 s pour `resvg` sur la
    /// vitrine, et autant pour quiconque ouvrait le fichier (fiche 71). Aucune constante.
    pub fn lueur(
        &mut self,
        (x, y, l, h): (f32, f32, f32, f32),
        rayon: f32,
        (teinte, alpha): (&str, f32),
        (sigma, portee): (f32, f32),
    ) {
        let nom = self.nom("flou");
        self.defs.push_str(&format!(
            r#"<filter id="{nom}" filterUnits="userSpaceOnUse" x="{}" y="{}" width="{}" height="{}"><feGaussianBlur stdDeviation="{}"/></filter>"#,
            n(x - portee),
            n(y - portee),
            n(l + 2.0 * portee),
            n(h + 2.0 * portee),
            n(sigma)
        ));
        self.poser(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{teinte}" fill-opacity="{}" filter="url(#{nom})"/>"#,
            n(x), n(y), n(l), n(h), n(rayon_tenu(rayon, (l, h))), n(alpha)
        ));
    }

    /// **Le document entier**, de `largeur` × `hauteur` pixels.
    pub fn finir(self, (largeur, hauteur): (u32, u32)) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" \
             xmlns:xlink=\"http://www.w3.org/1999/xlink\" width=\"{largeur}\" height=\"{hauteur}\" \
             viewBox=\"0 0 {largeur} {hauteur}\">\n<defs>{}</defs>\n{}</svg>\n",
            self.defs, self.corps
        )
    }
}

/// **Le rayon qu'un coin peut tenir** : borné par la plus petite demi-dimension, comme le
/// canevas le borne (`push_rounded_rect`) — un coin reste un quart de cercle. Un `rx` seul
/// laisserait le lecteur borner chaque axe à part, et un rectangle bas aurait des coins
/// d'ellipse.
pub fn rayon_tenu(rayon: f32, (l, h): (f32, f32)) -> f32 {
    rayon.min(l / 2.0).min(h / 2.0).max(0.0)
}

/// **Du base64** (RFC 4648, avec remplissage) — les octets d'une image dans son `data:`.
pub fn base64(octets: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(octets.len().div_ceil(3) * 4);
    for bloc in octets.chunks(3) {
        let b = [
            bloc[0],
            *bloc.get(1).unwrap_or(&0),
            *bloc.get(2).unwrap_or(&0),
        ];
        let v = u32::from(b[0]) << 16 | u32::from(b[1]) << 8 | u32::from(b[2]);
        for i in 0..4 {
            if i <= bloc.len() {
                s.push(ALPHABET[(v >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                s.push('=');
            }
        }
    }
    s
}

#[cfg(test)]
mod tests;
