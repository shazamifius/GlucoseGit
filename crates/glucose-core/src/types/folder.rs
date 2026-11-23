//! Les dossiers-sous-canvas et leur miroir éventuel d'un dossier du disque.

use super::Id;
use crate::geometry::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderSortMode {
    NameAsc,
    NameDesc,
    Type,
    SizeDesc,
    SizeAsc,
    ModifiedDesc,
    ModifiedAsc,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FolderMirrorSource {
    pub root_path: String,
    pub mode: String, // "snapshot" | "live"
    pub last_scanned_at: i64,
    pub pattern: Option<String>,
    pub recursive: bool,
    pub sort_by: Option<FolderSortMode>,
    pub pending_scan: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasFolder {
    pub id: Id,
    pub name: String,
    pub color: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub child_board_id: Id,
    pub mirror_of: Option<Id>,
    pub mirror_source: Option<FolderMirrorSource>,
}

/// **La couleur d'un dossier qui n'en a pas reçu** : celle de sa naissance, et ce que « Par
/// défaut » lui rend (COULEUR-1, fiche 98).
pub const COULEUR_DE_NAISSANCE: &str = "#888888";

/// **L'opacité du corps d'un dossier qui n'en a pas reçu**, sur 255 : 2,5 %, celle de Glucose
/// Tauri — *« un cadre qui contient, pas un objet qui s'affirme »* (fiche 06 § 8).
pub const OPACITE_DE_NAISSANCE: u8 = 6;

/// **OPACITE-1 — la couleur d'un dossier porte son opacité** (sa note du 23/11 : *« les dossiers
/// sont très transparents par défaut ; ce serait bien, avec le système de couleur, d'avoir une
/// option pour changer son opacité »* ; fiche 133).
///
/// Elle s'écrit `#rrggbb`, ou `#rrggbbaa` — la forme de CSS Color 4 — quand l'opacité n'est pas
/// celle de la naissance. **Pas de champ neuf** : le format du document est positionnel (PERSIST-1),
/// et un champ de plus rendrait le document illisible à la 2.0.4-beta.4, qui partage des canevas
/// avec les versions d'après ; elle lit `#rrggbbaa` comme une couleur qu'elle ne connaît pas, et
/// dessine ce dossier au gris de sa naissance — rien ne se perd. Un dossier à l'opacité de sa
/// naissance garde `#rrggbb`, que toutes les versions lisent.
///
/// Seules ces fonctions lisent les deux derniers chiffres : la teinte d'un dossier ([`teinte_de`])
/// est ce que la roue montre, ce que la pipette prend et ce que « Par défaut » compare.
pub fn teinte_de(couleur: &str) -> &str {
    match opacite_ecrite(couleur) {
        Some(_) => &couleur[..7],
        None => couleur,
    }
}

/// L'opacité qu'une couleur de dossier porte, ou celle de la naissance (OPACITE-1).
pub fn opacite_de(couleur: &str) -> u8 {
    opacite_ecrite(couleur).unwrap_or(OPACITE_DE_NAISSANCE)
}

/// Les deux derniers chiffres d'un `#rrggbbaa`, s'il en est un.
fn opacite_ecrite(couleur: &str) -> Option<u8> {
    let chiffres = hexadecimale(couleur, 8)?;
    u8::from_str_radix(&chiffres[6..], 16).ok()
}

/// Les chiffres de `#…` s'il y en a exactement `n`, tous hexadécimaux.
fn hexadecimale(couleur: &str, n: usize) -> Option<&str> {
    let chiffres = couleur.strip_prefix('#')?;
    let ok = chiffres.len() == n && chiffres.bytes().all(|c| c.is_ascii_hexdigit());
    ok.then_some(chiffres)
}

/// **La couleur d'un dossier de cette teinte à cette opacité** (OPACITE-1) : `#rrggbb` à l'opacité
/// de la naissance, `#rrggbbaa` sinon. Une teinte qui n'est pas un `#rrggbb` (un `#rgb` de Glucose
/// Tauri, un texte faux) ne reçoit pas de chiffres qu'aucun lecteur ne saurait séparer d'elle : elle
/// reste telle quelle, à l'opacité de la naissance.
pub fn couleur_de_dossier(teinte: &str, opacite: u8) -> String {
    let teinte = teinte_de(teinte);
    if opacite == OPACITE_DE_NAISSANCE || hexadecimale(teinte, 6).is_none() {
        return teinte.to_string();
    }
    format!("{teinte}{opacite:02x}")
}

impl CanvasFolder {
    /// Sa teinte, sans l'opacité qu'elle peut porter (OPACITE-1).
    pub fn teinte(&self) -> &str {
        teinte_de(&self.color)
    }

    /// L'opacité de son corps, sur 255 (OPACITE-1).
    pub fn opacite(&self) -> u8 {
        opacite_de(&self.color)
    }

    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        child_board_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            color: COULEUR_DE_NAISSANCE.into(),
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 150.0,
            child_board_id: child_board_id.into(),
            mirror_of: None,
            mirror_source: None,
        }
    }

    /// La boîte du dossier, ancrée en haut à gauche comme les annotations.
    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

#[cfg(test)]
mod tests;
