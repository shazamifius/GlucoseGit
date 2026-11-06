//! **La fenêtre sans la barre du système** (BARRE-1, fiche 85) : sous Windows, plus de barre
//! blanche ; c'est la bande de Glucose qui déplace la fenêtre et porte ses trois boutons.
//!
//! # Une fenêtre ordinaire, dont la barre n'occupe plus de place
//!
//! Retirer le cadre (`set_decorations(false)`) est le geste facile, et il a déjà coûté : agrandie
//! et sans cadre, la fenêtre ressemble à un jeu en plein écran, et l'application NVIDIA lui a
//! appliqué RTX HDR, toutes ses couleurs changées (fiche 52). Elle perdrait aussi son ombre et ses
//! animations. Glucose garde donc une **fenêtre ordinaire** (`WS_CAPTION`), et répond à Windows,
//! comme Chrome et le Terminal, que sa zone cliente monte jusqu'en haut (`WM_NCCALCSIZE`) et que
//! sa bande est sa barre de titre (`WM_NCHITTEST` rend `HTCAPTION`). Glisser, coller aux bords,
//! double-cliquer, `Win + flèches` : tout reste le geste de Windows, sans une ligne à nous.
//!
//! # Si Windows refuse
//!
//! [`habiller`] rend `false`, la barre du système reste, et aucun bouton n'est dessiné : on ne
//! peut jamais se retrouver avec une fenêtre qu'on ne sait plus fermer.

/// Ce que la bande occupe à l'écran, en pixels de la fenêtre.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Zones {
    /// La hauteur de ce qui déplace la fenêtre. Zéro : rien ne la déplace (un menu est ouvert, le
    /// mode référence, ou une épreuve).
    pub bande: i32,
    /// Ce qui, dans la bande, attend un clic : `[x, y, largeur, hauteur]`.
    pub tenus: Vec<[i32; 4]>,
}

/// Ce qu'un point de la fenêtre est pour Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zone {
    /// Le bord du haut, qui redimensionne : sans lui, il n'y en aurait plus aucun.
    BordDuHaut,
    /// La bande, qui déplace.
    Deplace,
    /// Le reste : le clic va à Glucose.
    Client,
}

impl Zones {
    /// **Ce qu'est ce point.** Un bouton passe avant le bord : celui de fermeture est dans le coin.
    pub fn sous(&self, x: i32, y: i32, agrandie: bool, bord: i32) -> Zone {
        if self.bande <= 0 || y >= self.bande || y < 0 {
            return Zone::Client;
        }
        let tenu = self
            .tenus
            .iter()
            .any(|&[zx, zy, l, h]| x >= zx && x < zx + l && y >= zy && y < zy + h);
        if tenu {
            Zone::Client
        } else if !agrandie && y < bord {
            Zone::BordDuHaut
        } else {
            Zone::Deplace
        }
    }
}

#[cfg(windows)]
mod cadre_windows;

/// **Retire la barre du système de cette fenêtre.** Rend `true` si c'est à Glucose de dessiner
/// ses boutons. `GLUCOSE_CADRE` les montre ailleurs : un instrument, pour regarder leur dessin.
pub fn habiller(fenetre: &winit::window::Window) -> bool {
    #[cfg(windows)]
    {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let Ok(poignee) = fenetre.window_handle() else {
            return false;
        };
        let RawWindowHandle::Win32(w) = poignee.as_raw() else {
            return false;
        };
        cadre_windows::habiller(w.hwnd.get())
    }
    #[cfg(not(windows))]
    {
        let _ = fenetre;
        std::env::var_os("GLUCOSE_CADRE").is_some()
    }
}

/// **Ce que la bande occupe maintenant**, pour la prochaine question de Windows.
pub fn poser_les_zones(zones: Zones) {
    #[cfg(windows)]
    cadre_windows::poser(zones);
    #[cfg(not(windows))]
    let _ = zones;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bande() -> Zones {
        Zones {
            bande: 40,
            tenus: vec![[100, 0, 30, 40], [900, 0, 100, 40]],
        }
    }

    #[test]
    fn test_barre_1_le_vide_de_la_bande_deplace_et_ses_boutons_recoivent_le_clic() {
        let z = bande();
        assert_eq!(z.sous(50, 20, false, 8), Zone::Deplace);
        assert_eq!(z.sous(110, 20, false, 8), Zone::Client, "un bouton");
        assert_eq!(z.sous(50, 40, false, 8), Zone::Client, "sous la bande");
    }

    #[test]
    fn test_barre_1_le_haut_redimensionne_sauf_sur_un_bouton_et_une_fois_agrandie() {
        let z = bande();
        assert_eq!(z.sous(50, 3, false, 8), Zone::BordDuHaut);
        assert_eq!(
            z.sous(950, 3, false, 8),
            Zone::Client,
            "fermer, dans le coin"
        );
        assert_eq!(
            z.sous(50, 3, true, 8),
            Zone::Deplace,
            "agrandie, elle n'a plus de bord"
        );
    }

    #[test]
    fn test_barre_1_sans_bande_rien_ne_deplace() {
        let z = Zones {
            bande: 0,
            tenus: Vec::new(),
        };
        assert_eq!(z.sous(50, 3, false, 8), Zone::Client);
    }
}
