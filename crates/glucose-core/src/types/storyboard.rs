//! **Le storyboard d'un tableau** : ses cases, et la grille qui les range (STORYBOARD-1,
//! fiche 64 ; STORYBOARD-2, fiche 65).
//!
//! # La grille est une donnée du tableau
//!
//! Glucose Tauri la gardait avec le tableau (`board.storyboard` : `aspectRatio`, `panelWidth`,
//! `cols`, `gap`). STORYBOARD-1 l'avait laissée au panneau : un rectangle tracé à six colonnes
//! retombait à quatre au « + Ajouter » suivant, et un document rouvert ne savait plus la sienne.
//! Elle revient au tableau (schéma 3), et **une seule loi** ([`Grille::coin`]) dit où va chaque
//! case : la case de rang `i` est à la colonne `i mod c`, à la rangée `i div c`.

use super::Id;

/// La hauteur de la description sous chaque case, en unités du monde — celle de Glucose Tauri
/// (`StoryboardLayer.ts`, `descH`). Le dessin, le fantôme et la prise d'une case la lisent.
pub const DESCRIPTION: f64 = 40.0;

#[derive(Debug, Clone, PartialEq)]
pub struct StoryboardPanel {
    pub id: Id,
    pub order: i32,
    pub description: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl StoryboardPanel {
    pub fn new(id: impl Into<String>, order: i32, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            id: id.into(),
            order,
            description: String::new(),
            x,
            y,
            width,
            height,
        }
    }

    /// Le cadre contient-il ce point ? La bande de description, sous lui, n'en est pas.
    pub fn contient(&self, (x, y): (f64, f64)) -> bool {
        (self.x..=self.x + self.width).contains(&x) && (self.y..=self.y + self.height).contains(&y)
    }
}

/// **Les cinq formats de Glucose Tauri**, dans l'ordre de son panneau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Cinema,
    Classique,
    Scope,
    Carre,
    Vertical,
}

impl Format {
    pub const TOUS: [Format; 5] = [
        Format::Cinema,
        Format::Classique,
        Format::Scope,
        Format::Carre,
        Format::Vertical,
    ];

    /// La largeur sur la hauteur d'une case.
    pub fn rapport(self) -> f64 {
        match self {
            Self::Cinema => 16.0 / 9.0,
            Self::Classique => 4.0 / 3.0,
            Self::Scope => 2.35,
            Self::Carre => 1.0,
            Self::Vertical => 9.0 / 16.0,
        }
    }
}

/// **La grille d'un storyboard** : le format des cases, leur largeur, les colonnes, l'écart.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Grille {
    pub format: Format,
    pub largeur: f64,
    pub colonnes: usize,
    pub ecart: f64,
}

/// **La grille de Glucose Tauri** quand rien n'est réglé : le cinéma, 280 de large, quatre
/// colonnes, 24 d'écart (`StoryboardControls.tsx`, ses `useState`).
impl Default for Grille {
    fn default() -> Self {
        Self {
            format: Format::Cinema,
            largeur: 280.0,
            colonnes: 4,
            ecart: 24.0,
        }
    }
}

impl Grille {
    /// La hauteur d'une case : sa largeur sur son rapport.
    pub fn hauteur(&self) -> f64 {
        self.largeur / self.format.rapport()
    }

    /// Combien de colonnes, au moins une.
    pub fn colonnes(&self) -> usize {
        self.colonnes.max(1)
    }

    /// Combien de cases une grille neuve porte : deux rangées, comme dans Tauri.
    pub fn nombre_initial(&self) -> usize {
        self.colonnes() * 2
    }

    /// D'une case à sa voisine, en largeur et en hauteur : la case, sa bande, l'écart.
    pub fn pas(&self) -> (f64, f64) {
        (
            self.largeur + self.ecart,
            self.hauteur() + DESCRIPTION + self.ecart,
        )
    }

    /// **Le coin de la case de rang `rang`**, la grille partant de `origine` — la seule loi
    /// qui place une case.
    pub fn coin(&self, origine: (f64, f64), rang: usize) -> (f64, f64) {
        let (pas_x, pas_y) = self.pas();
        let colonnes = self.colonnes();
        (
            origine.0 + (rang % colonnes) as f64 * pas_x,
            origine.1 + (rang / colonnes) as f64 * pas_y,
        )
    }

    /// **L'étendue d'une grille de `nombre` cases** : sa largeur et sa hauteur, bandes
    /// comprises, sans l'écart d'après la dernière.
    pub fn etendue(&self, nombre: usize) -> (f64, f64) {
        let (pas_x, pas_y) = self.pas();
        let colonnes = self.colonnes().min(nombre.max(1));
        let rangees = nombre.max(1).div_ceil(self.colonnes());
        (
            colonnes as f64 * pas_x - self.ecart,
            rangees as f64 * pas_y - self.ecart,
        )
    }

    /// **Combien de colonnes et de rangées pour `(largeur, hauteur)`** : le nombre de cases
    /// dont l'étendue est la plus proche — au moins une de chaque.
    ///
    /// `n` cases et leurs `n − 1` écarts s'étendent sur `n·p − e` (`p` le pas) ; la plus proche
    /// de `L` est `n = round((L + e) / p)`. Pas la partie entière — « celles qui tiennent » :
    /// sur une étendue exacte, `(n·p − e + e) / p` peut valoir `n − 1 ulp` (sept rangées au
    /// format Scope donnent 6,999999999999999), et saisir la poignée au coin aurait retiré une
    /// rangée sans que la main bouge. L'arrondi n'a de bord qu'à mi-case.
    pub fn tient_dans(&self, (largeur, hauteur): (f64, f64)) -> (usize, usize) {
        let (pas_x, pas_y) = self.pas();
        let nombre = |l: f64, pas: f64| ((l + self.ecart) / pas).round().max(1.0) as usize;
        (nombre(largeur, pas_x), nombre(hauteur, pas_y))
    }

    /// Le coin de chaque case d'une grille de `nombre` cases centrée sur `centre`, dans
    /// l'ordre de lecture.
    pub fn coins(&self, nombre: usize, centre: (f64, f64)) -> Vec<(f64, f64)> {
        let (largeur, hauteur) = self.etendue(nombre);
        let origine = (centre.0 - largeur / 2.0, centre.1 - hauteur / 2.0);
        (0..nombre).map(|i| self.coin(origine, i)).collect()
    }
}
