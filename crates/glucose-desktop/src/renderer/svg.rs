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
//! Du SVG 1.1 sans script : des `<rect>`, des `<path>`, des `<text>` dont chaque ligne est déjà
//! coupée (le lecteur n'a pas à refaire la coupe, qui dépend des mesures d'Inter), des
//! `<image>` qui portent leurs octets d'origine en `data:` — un fichier qui se suffit.

use std::fmt::Write as _;
use tiny_skia::Color;

/// **Un SVG en cours d'écriture** : ses définitions (dégradés, filtres) et ses éléments, dans
/// l'ordre où ils se peignent.
#[derive(Debug, Default)]
pub struct Svg {
    defs: String,
    corps: String,
    /// De quoi nommer chaque définition d'un nom à elle.
    suivant: usize,
    /// Les flous déjà définis, par leur écart-type.
    flous: Vec<String>,
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

/// Un style de texte : son corps, sa graisse, son italique, sa chasse fixe, sa couleur.
#[derive(Debug, Clone, PartialEq)]
pub struct StyleDeTexte {
    pub corps: f32,
    pub gras: bool,
    pub italique: bool,
    pub fixe: bool,
    pub couleur: Color,
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
            n(rayon),
            n(a)
        );
        self.poser(&e);
    }

    /// **Un texte sur une ligne**, son coin haut gauche en `(x, y)` — la ligne de base tombe à
    /// `y + corps`, comme dans la typographie du rendu.
    pub fn texte(&mut self, (x, y): (f32, f32), texte: &str, style: &StyleDeTexte) {
        let mut e = String::new();
        let (c, a) = couleur(style.couleur);
        let _ = write!(
            e,
            r#"<text x="{}" y="{}" font-size="{}" font-family="{}" fill="{c}""#,
            n(x),
            n(y + style.corps),
            n(style.corps),
            if style.fixe {
                "'JetBrains Mono', Consolas, monospace"
            } else {
                "Inter, 'Segoe UI', system-ui, sans-serif"
            }
        );
        if a < 1.0 {
            let _ = write!(e, r#" fill-opacity="{}""#, n(a));
        }
        if style.gras {
            e.push_str(r#" font-weight="700""#);
        }
        if style.italique {
            e.push_str(r#" font-style="italic""#);
        }
        let _ = write!(e, r#" xml:space="preserve">{}</text>"#, echapper(texte));
        self.poser(&e);
    }

    /// **Une lueur** : ce rectangle arrondi, de cette teinte, flouté d'une gaussienne
    /// d'écart-type `sigma` — l'ombre de boîte de Glucose Tauri, exactement (HALO-1).
    pub fn lueur(
        &mut self,
        (x, y, l, h): (f32, f32, f32, f32),
        rayon: f32,
        (teinte, alpha): (&str, f32),
        sigma: f32,
    ) {
        let nom = format!("flou-{}", n(sigma).replace('.', "_"));
        if !self.flous.contains(&nom) {
            // La région du filtre en unités de l'espace : trois écarts-types de chaque côté,
            // au-delà desquels la gaussienne ne change plus aucun niveau.
            self.defs.push_str(&format!(
                r#"<filter id="{nom}" filterUnits="userSpaceOnUse" x="-100000" y="-100000" width="200000" height="200000"><feGaussianBlur stdDeviation="{}"/></filter>"#,
                n(sigma)
            ));
            self.flous.push(nom.clone());
        }
        self.poser(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{teinte}" fill-opacity="{}" filter="url(#{nom})"/>"#,
            n(x), n(y), n(l), n(h), n(rayon), n(alpha)
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
