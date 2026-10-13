//! **Les champs de nombre du dock** (CHAMP-1) : la taille, l'espacement et les colonnes
//! d'Ordonner ; la largeur, les colonnes et l'espacement du Storyboard.
//!
//! Ils étaient dessinés et ne répondaient à rien : un clic dessus ne faisait rien, et Ordonner
//! rangeait toujours à 280 et 16. Glucose Tauri en faisait des `<input type="number">`.
//!
//! # Glisser pour régler, cliquer pour taper
//!
//! Les outils de création (Photoshop, Figma, Blender) prêtent deux gestes au même champ :
//! **glisser** horizontalement fait défiler la valeur, et l'écran suit ; **cliquer** sans
//! bouger ouvre la frappe, la valeur entière choisie — la première touche la remplace. Le
//! glisser seul n'est ni découvrable ni à la portée de tous : la frappe reste toujours là.
//! Entrée ou un clic ailleurs valident, Échap renonce, les flèches montent et descendent d'un
//! pas.
//!
//! # Le panneau entier est la règle
//!
//! Glisser sur toute la largeur du panneau parcourt toutes les valeurs du champ, de sa borne
//! basse à sa borne haute. Aucune sensibilité à choisir : chaque champ se règle au même
//! geste, qu'il compte douze colonnes ou deux mille points.

use super::{panneau_sous, DockManager, LayoutMode, TabId, WidgetRect};
use crate::interactions::text_entry::TextEntry;
use crate::params::{Pointer, ScaledRect, ScreenFrame};
use crate::typography::Typography;
use winit::keyboard::{Key, NamedKey};

pub mod paint;

/// Un champ de nombre du dock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Champ {
    OrdonnerTaille,
    OrdonnerEcart,
    OrdonnerColonnes,
    StoryLargeur,
    StoryColonnes,
    StoryEcart,
}

/// Les bornes d'un champ et son pas — ceux de Glucose Tauri.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bornes {
    pub min: f64,
    pub max: f64,
    pub pas: f64,
}

impl Champ {
    pub const TOUS: [Self; 6] = [
        Self::OrdonnerTaille,
        Self::OrdonnerEcart,
        Self::OrdonnerColonnes,
        Self::StoryLargeur,
        Self::StoryColonnes,
        Self::StoryEcart,
    ];

    pub fn panneau(self) -> TabId {
        match self {
            Self::OrdonnerTaille | Self::OrdonnerEcart | Self::OrdonnerColonnes => TabId::Organize,
            Self::StoryLargeur | Self::StoryColonnes | Self::StoryEcart => TabId::Storyboard,
        }
    }

    pub fn bornes(self) -> Bornes {
        let (min, max, pas) = match self {
            Self::OrdonnerTaille => (50.0, 2000.0, 10.0),
            Self::OrdonnerEcart => (0.0, 200.0, 4.0),
            Self::OrdonnerColonnes => (0.0, 20.0, 1.0),
            Self::StoryLargeur => (80.0, 800.0, 10.0),
            Self::StoryColonnes => (1.0, 12.0, 1.0),
            Self::StoryEcart => (0.0, 100.0, 4.0),
        };
        Bornes { min, max, pas }
    }

    /// **Le champ est-il là ?** Les colonnes d'Ordonner ne comptent que pour la grille et la
    /// mosaïque : ailleurs, le champ n'existe pas — un champ sans effet mentirait.
    pub fn present(self, dock: &DockManager) -> bool {
        self != Self::OrdonnerColonnes
            || matches!(dock.organize.layout, LayoutMode::Grid | LayoutMode::Masonry)
    }

    /// Ce que le champ dit au-dessus de lui. La taille d'Ordonner est la **hauteur** d'une
    /// rangée, ou la **largeur** d'une colonne, selon la disposition.
    pub fn libelle(self, dock: &DockManager) -> &'static str {
        match self {
            Self::OrdonnerTaille => match dock.organize.layout {
                LayoutMode::Grid | LayoutMode::Masonry => "LARGEUR",
                _ => "HAUTEUR",
            },
            Self::OrdonnerEcart | Self::StoryEcart => "ESPACEMENT",
            Self::OrdonnerColonnes | Self::StoryColonnes => "COLONNES",
            Self::StoryLargeur => "LARGEUR",
        }
    }

    pub fn lire(self, dock: &DockManager) -> f64 {
        match self {
            Self::OrdonnerTaille => dock.organize.size,
            Self::OrdonnerEcart => dock.organize.gap,
            Self::OrdonnerColonnes => dock.organize.cols as f64,
            Self::StoryLargeur => dock.storyboard.panel_width,
            Self::StoryColonnes => dock.storyboard.cols as f64,
            Self::StoryEcart => dock.storyboard.gap,
        }
    }

    /// Écrit une valeur, **bornée et entière** — ce qui sort d'ici est toujours permis.
    pub fn ecrire(self, dock: &mut DockManager, v: f64) {
        let b = self.bornes();
        let v = if v.is_finite() {
            v.round()
        } else {
            self.lire(dock)
        };
        let v = v.clamp(b.min, b.max);
        match self {
            Self::OrdonnerTaille => dock.organize.size = v,
            Self::OrdonnerEcart => dock.organize.gap = v,
            Self::OrdonnerColonnes => dock.organize.cols = v as usize,
            Self::StoryLargeur => dock.storyboard.panel_width = v,
            Self::StoryColonnes => dock.storyboard.cols = v as usize,
            Self::StoryEcart => dock.storyboard.gap = v,
        }
    }

    /// La valeur telle qu'on la lit : `auto` pour des colonnes qui se comptent seules.
    pub fn montre(self, v: f64) -> String {
        if self == Self::OrdonnerColonnes && v == 0.0 {
            "auto".to_string()
        } else {
            format!("{}", v as i64)
        }
    }

    /// Le rectangle du champ dans le cadre de son panneau — la géométrie du panneau, la seule.
    pub fn rect(self, frame: ScaledRect, typo: &Typography) -> WidgetRect {
        match self {
            Self::OrdonnerTaille => super::organize::layout_organize_panel(frame, typo).size_rect,
            Self::OrdonnerEcart => super::organize::layout_organize_panel(frame, typo).gap_rect,
            Self::OrdonnerColonnes => super::organize::layout_organize_panel(frame, typo).cols_rect,
            Self::StoryLargeur => super::storyboard::champs(frame)[0],
            Self::StoryColonnes => super::storyboard::champs(frame)[1],
            Self::StoryEcart => super::storyboard::champs(frame)[2],
        }
    }
}

/// **Ce qu'on fait d'un champ.**
#[derive(Debug, Clone, PartialEq)]
pub enum Saisie {
    /// Le champ est tenu : glisser règle ; relâcher sans avoir bougé ouvre la frappe.
    Tenu {
        champ: Champ,
        /// L'abscisse de l'appui et la valeur d'alors ; la règle — la largeur du panneau — ;
        /// le jeu sous lequel l'appui reste un clic.
        depart: (f32, f64),
        regle: f32,
        jeu: f32,
        bouge: bool,
    },
    /// La valeur se tape. `entiere` : toute la valeur est choisie, la première touche la
    /// remplace.
    Frappe {
        champ: Champ,
        entree: TextEntry,
        entiere: bool,
    },
}

impl Saisie {
    pub fn champ(&self) -> Champ {
        match self {
            Self::Tenu { champ, .. } | Self::Frappe { champ, .. } => *champ,
        }
    }
}

/// **Le champ sous ce point**, s'il y en a un : dans le panneau qui prend le clic.
pub fn champ_sous(
    dock: &DockManager,
    typo: &Typography,
    screen: ScreenFrame,
    p: Pointer,
) -> Option<(Champ, ScaledRect)> {
    let (tab, frame) = panneau_sous(dock, screen, p)?;
    Champ::TOUS
        .into_iter()
        .filter(|c| c.panneau() == tab && c.present(dock))
        .find(|c| c.rect(frame, typo).contains(p.x, p.y))
        .map(|c| (c, frame))
}

impl DockManager {
    /// **Prend un champ** à l'abscisse `x` ; tout le panneau est sa règle. Sous `jeu` — le
    /// tremblement d'une main —, un appui reste un clic.
    pub fn tenir_le_champ(&mut self, champ: Champ, frame: ScaledRect, (x, jeu): (f32, f32)) {
        self.champ = Some(Saisie::Tenu {
            champ,
            depart: (x, champ.lire(self)),
            regle: frame.w.max(1.0),
            jeu,
            bouge: false,
        });
    }

    /// **Le champ tenu suit la main** : rend vrai si un champ est tenu. Le glisser se compte
    /// depuis le bord du jeu, pour que la valeur ne saute pas quand il commence.
    pub fn glisser_le_champ(&mut self, x: f32) -> bool {
        let Some(Saisie::Tenu {
            champ,
            depart,
            regle,
            jeu,
            bouge,
        }) = self.champ.as_mut()
        else {
            return false;
        };
        if !*bouge {
            if (x - depart.0).abs() < *jeu {
                return true;
            }
            (*bouge, depart.0) = (true, depart.0 + jeu.copysign(x - depart.0));
        }
        let (champ, depart, regle) = (*champ, *depart, *regle);
        let b = champ.bornes();
        let pas = (f64::from((x - depart.0) / regle) * (b.max - b.min) / b.pas).round();
        champ.ecrire(self, depart.1 + pas * b.pas);
        true
    }

    /// **Le champ est lâché** : glissé, il garde sa valeur ; cliqué, il s'ouvre à la frappe.
    /// Rend vrai si un champ était tenu.
    pub fn lacher_le_champ(&mut self) -> bool {
        let Some(Saisie::Tenu { champ, bouge, .. }) = self.champ else {
            return false;
        };
        self.champ = (!bouge).then(|| Saisie::Frappe {
            champ,
            entree: TextEntry::new(format!("{}", champ.lire(self) as i64)),
            entiere: true,
        });
        true
    }

    /// **La valeur tapée devient celle du champ**, bornée ; une frappe qui n'est pas un nombre
    /// laisse la valeur d'avant.
    pub fn valider_le_champ(&mut self) {
        if let Some(Saisie::Frappe { champ, entree, .. }) = self.champ.take() {
            if let Ok(v) = entree.text().trim().parse::<f64>() {
                champ.ecrire(self, v);
            }
        }
    }

    /// **Une touche dans le champ qu'on tape** : elle est pour lui, toute — une lettre ne
    /// doit pas devenir un raccourci. Rend vrai si une frappe était ouverte.
    pub fn frapper_le_champ(&mut self, key: &Key) -> bool {
        // Le panneau refermé emporte sa frappe : elle ne doit pas garder le clavier.
        if self
            .champ
            .as_ref()
            .is_some_and(|c| !self.is_open(c.champ().panneau()))
        {
            self.champ = None;
        }
        let Some(Saisie::Frappe {
            champ,
            entree,
            entiere,
        }) = self.champ.as_mut()
        else {
            return false;
        };
        let champ = *champ;
        match key {
            Key::Named(NamedKey::Enter) => self.valider_le_champ(),
            Key::Named(NamedKey::Tab) => {
                self.valider_le_champ();
                self.ouvrir_le_suivant(champ);
            }
            Key::Named(NamedKey::Escape) => self.champ = None,
            Key::Named(fleche @ (NamedKey::ArrowUp | NamedKey::ArrowDown)) => {
                let signe = if *fleche == NamedKey::ArrowUp {
                    1.0
                } else {
                    -1.0
                };
                let tape = entree.text().trim().parse::<f64>().ok();
                let b = champ.bornes();
                let v = (tape.unwrap_or(champ.lire(self)) + signe * b.pas).clamp(b.min, b.max);
                if let Some(Saisie::Frappe {
                    entree, entiere, ..
                }) = self.champ.as_mut()
                {
                    (*entree, *entiere) = (TextEntry::new(format!("{}", v as i64)), true);
                }
            }
            // Une valeur choisie entière s'efface, ou se remplace, d'une touche.
            Key::Named(NamedKey::Backspace | NamedKey::Delete) if *entiere => {
                (*entree, *entiere) = (TextEntry::default(), false);
            }
            Key::Character(t) if t.chars().all(|c| c.is_ascii_digit()) => {
                if std::mem::take(entiere) {
                    *entree = TextEntry::default();
                }
                entree.insert(t);
            }
            Key::Character(_) => {}
            autre => *entiere &= !entree.editer(autre),
        }
        true
    }

    /// `Tab` : le champ suivant du même panneau s'ouvre à la frappe.
    fn ouvrir_le_suivant(&mut self, champ: Champ) {
        let suivant = Champ::TOUS
            .into_iter()
            .cycle()
            .skip_while(|c| *c != champ)
            .skip(1)
            .take(Champ::TOUS.len())
            .find(|c| c.panneau() == champ.panneau() && c.present(self));
        if let Some(c) = suivant {
            self.champ = Some(Saisie::Frappe {
                champ: c,
                entree: TextEntry::new(format!("{}", c.lire(self) as i64)),
                entiere: true,
            });
        }
    }

    /// La saisie de ce panneau, s'il en a une — ce dont son image dépend.
    pub fn saisie_du(&self, tab: TabId) -> Option<Saisie> {
        self.champ.clone().filter(|s| s.champ().panneau() == tab)
    }
}

#[cfg(test)]
mod tests;
