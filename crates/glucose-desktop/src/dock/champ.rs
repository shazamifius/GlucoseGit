//! **Les champs de nombre du dock** (CHAMP-1) : la taille, l'espacement et les colonnes
//! d'Ordonner ; la largeur, les colonnes et l'espacement du Storyboard ; la durée d'une case,
//! pour tout le storyboard et pour la case choisie (fiche 67).
//!
//! # Une durée se dit en secondes
//!
//! Elle se garde en millisecondes — au document, comme Storyboarder —, mais se lit « 2,5 s » et
//! se tape en secondes, la virgule ou le point : personne ne pense une case en millisecondes.
//!
//! # Une durée se règle en rapport
//!
//! Passer d'une demi-seconde à une seconde se sent autant que de cinq à dix : une durée se
//! perçoit en **rapport**, pas en écart. Sa règle est donc **logarithmique** — chaque pixel du
//! glisser la multiplie par le même facteur, `(max/min)^(1/largeur)` —, et elle s'arrondit à
//! **deux chiffres significatifs** (0,25 s ; 2,5 s ; 14 s) : la même finesse relative partout,
//! et aucun pas à choisir. Une règle linéaire, à cent millisecondes le pas, faisait passer une
//! case de 2 à 14 s en soixante pixels.
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
    /// La durée d'une case qui n'a pas la sienne, pour tout le storyboard.
    StoryDuree,
    /// La durée de la case choisie.
    CaseDuree,
}

/// Les bornes d'un champ et son pas — ceux de Glucose Tauri.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bornes {
    pub min: f64,
    pub max: f64,
    pub pas: f64,
}

impl Champ {
    pub const TOUS: [Self; 8] = [
        Self::OrdonnerTaille,
        Self::OrdonnerEcart,
        Self::OrdonnerColonnes,
        Self::StoryLargeur,
        Self::StoryColonnes,
        Self::StoryEcart,
        Self::StoryDuree,
        Self::CaseDuree,
    ];

    pub fn panneau(self) -> TabId {
        match self {
            Self::OrdonnerTaille | Self::OrdonnerEcart | Self::OrdonnerColonnes => TabId::Organize,
            Self::StoryLargeur
            | Self::StoryColonnes
            | Self::StoryEcart
            | Self::StoryDuree
            | Self::CaseDuree => TabId::Storyboard,
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
            // Un dixième de seconde à une minute ; le pas d'une durée est un rapport, pas un
            // écart : il ne sert pas (voir `logarithmique`).
            Self::StoryDuree | Self::CaseDuree => (100.0, 60_000.0, 0.0),
        };
        Bornes { min, max, pas }
    }

    /// **Le champ est-il là ?** Les colonnes d'Ordonner ne comptent que pour la grille et la
    /// mosaïque, la durée par case que s'il y a un storyboard, celle d'une case que si une est
    /// choisie : ailleurs, le champ n'existe pas — un champ sans effet mentirait.
    pub fn present(self, dock: &DockManager) -> bool {
        let lecture = dock.storyboard.contenu.lecture.as_ref();
        match self {
            Self::OrdonnerColonnes => {
                matches!(dock.organize.layout, LayoutMode::Grid | LayoutMode::Masonry)
            }
            Self::StoryDuree => lecture.is_some(),
            Self::CaseDuree => lecture.is_some_and(|l| l.case.is_some()),
            _ => true,
        }
    }

    /// Une durée, qui se lit et se tape en secondes.
    fn est_une_duree(self) -> bool {
        matches!(self, Self::StoryDuree | Self::CaseDuree)
    }

    /// **Une règle en rapport** : les durées (voir l'en-tête).
    pub fn logarithmique(self) -> bool {
        self.est_une_duree()
    }

    /// **La valeur arrondie** à ce que le champ montre : une durée à deux chiffres
    /// significatifs, le reste à l'unité.
    fn arrondir(self, v: f64) -> f64 {
        if self.logarithmique() {
            significatifs(v, 2)
        } else {
            v.round()
        }
    }

    /// **La valeur d'un cran** — une flèche du clavier : un pas de plus, ou un dixième de plus
    /// pour une règle en rapport.
    pub fn cran(self, v: f64, signe: f64) -> f64 {
        let b = self.bornes();
        let v = if self.logarithmique() {
            self.arrondir(v * 1.1_f64.powf(signe))
        } else {
            v + signe * b.pas
        };
        v.clamp(b.min, b.max)
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
            Self::StoryDuree => "DURÉE PAR CASE",
            Self::CaseDuree => "DURÉE",
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
            Self::StoryDuree => dock.storyboard.duree_ms,
            Self::CaseDuree => dock.storyboard.duree_case,
        }
    }

    /// Écrit une valeur, **bornée et entière** — ce qui sort d'ici est toujours permis.
    pub fn ecrire(self, dock: &mut DockManager, v: f64) {
        let b = self.bornes();
        let v = if v.is_finite() {
            self.arrondir(v)
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
            Self::StoryDuree => dock.storyboard.duree_ms = v,
            Self::CaseDuree => dock.storyboard.duree_case = v,
        }
    }

    /// La valeur telle qu'on la lit : `auto` pour des colonnes qui se comptent seules, une
    /// durée en secondes.
    pub fn montre(self, v: f64) -> String {
        if self == Self::OrdonnerColonnes && v == 0.0 {
            "auto".to_string()
        } else if self.est_une_duree() {
            format!("{} s", secondes(v))
        } else {
            format!("{}", v as i64)
        }
    }

    /// La valeur telle qu'on la tape : un entier, ou des secondes.
    pub fn a_taper(self, v: f64) -> String {
        if self.est_une_duree() {
            secondes(v)
        } else {
            format!("{}", v as i64)
        }
    }

    /// **Ce qu'on a tapé**, en valeur du champ — des secondes pour une durée, la virgule ou le
    /// point ; `None` si ce n'est pas un nombre.
    pub fn lu(self, texte: &str) -> Option<f64> {
        let v = texte.trim().replace(',', ".").parse::<f64>().ok()?;
        Some(if self.est_une_duree() { v * 1000.0 } else { v })
    }

    /// Ce caractère peut-il s'écrire dans ce champ ? Des chiffres, et la virgule d'une durée.
    fn admet(self, c: char) -> bool {
        c.is_ascii_digit() || (self.est_une_duree() && matches!(c, ',' | '.'))
    }

    /// Le rectangle du champ dans le cadre de son panneau — la géométrie du panneau, la seule.
    pub fn rect(self, frame: ScaledRect, typo: &Typography, dock: &DockManager) -> WidgetRect {
        let lecture = || {
            let s = &dock.storyboard;
            super::storyboard::layout_storyboard_panel(frame, s, &s.contenu).lecture
        };
        let absent = WidgetRect::new(0.0, 0.0, 0.0, 0.0);
        match self {
            Self::OrdonnerTaille => super::organize::layout_organize_panel(frame, typo).size_rect,
            Self::OrdonnerEcart => super::organize::layout_organize_panel(frame, typo).gap_rect,
            Self::OrdonnerColonnes => super::organize::layout_organize_panel(frame, typo).cols_rect,
            Self::StoryLargeur => super::storyboard::champs(frame)[0],
            Self::StoryColonnes => super::storyboard::champs(frame)[1],
            Self::StoryEcart => super::storyboard::champs(frame)[2],
            Self::StoryDuree => lecture().map_or(absent, |l| l.duree),
            Self::CaseDuree => lecture().and_then(|l| l.case).map_or(absent, |c| c.duree),
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
        .find(|c| c.rect(frame, typo, dock).contains(p.x, p.y))
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
        let part = f64::from((x - depart.0) / regle);
        let v = if champ.logarithmique() {
            depart.1 * (b.max / b.min).powf(part)
        } else {
            depart.1 + (part * (b.max - b.min) / b.pas).round() * b.pas
        };
        champ.ecrire(self, v);
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
            entree: TextEntry::new(champ.a_taper(champ.lire(self))),
            entiere: true,
        });
        true
    }

    /// **La valeur tapée devient celle du champ**, bornée ; une frappe qui n'est pas un nombre
    /// laisse la valeur d'avant.
    pub fn valider_le_champ(&mut self) {
        if let Some(Saisie::Frappe { champ, entree, .. }) = self.champ.take() {
            if let Some(v) = champ.lu(entree.text()) {
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
                let tape = champ.lu(entree.text());
                let v = champ.cran(tape.unwrap_or(champ.lire(self)), signe);
                if let Some(Saisie::Frappe {
                    entree, entiere, ..
                }) = self.champ.as_mut()
                {
                    (*entree, *entiere) = (TextEntry::new(champ.a_taper(v)), true);
                }
            }
            // Une valeur choisie entière s'efface, ou se remplace, d'une touche.
            Key::Named(NamedKey::Backspace | NamedKey::Delete) if *entiere => {
                (*entree, *entiere) = (TextEntry::default(), false);
            }
            Key::Character(t) if t.chars().all(|c| champ.admet(c)) => {
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
                entree: TextEntry::new(c.a_taper(c.lire(self))),
                entiere: true,
            });
        }
    }

    /// La saisie de ce panneau, s'il en a une — ce dont son image dépend.
    pub fn saisie_du(&self, tab: TabId) -> Option<Saisie> {
        self.champ.clone().filter(|s| s.champ().panneau() == tab)
    }
}

/// **`v` arrondi à `n` chiffres significatifs** — zéro, et ce qui n'est pas positif, restent.
fn significatifs(v: f64, n: i32) -> f64 {
    if v <= 0.0 {
        return v;
    }
    let echelle = 10f64.powi(v.log10().floor() as i32 - (n - 1));
    (v / echelle).round() * echelle
}

/// **Des millisecondes, en secondes** : au moins un chiffre après la virgule, jamais un zéro
/// de trop — « 2,0 », « 2,5 », « 2,04 ».
fn secondes(ms: f64) -> String {
    let ms = ms.round() as i64;
    let (entier, reste) = (ms / 1000, ms % 1000);
    let mut fraction = format!("{reste:03}");
    while fraction.len() > 1 && fraction.ends_with('0') {
        fraction.pop();
    }
    format!("{entier},{fraction}")
}

#[cfg(test)]
mod tests;
