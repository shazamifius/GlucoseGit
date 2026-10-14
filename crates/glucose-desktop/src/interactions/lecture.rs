//! **L'animatique** (P2, proposition 4, fiche 67) : « ▶ Lire » montre le storyboard case après
//! case, chacune le temps de sa durée, en plein écran.
//!
//! # Le canevas lui-même, et aucun second moteur
//!
//! Storyboarder, Krita et Boords ont un lecteur à part. Ici, la lecture **cadre le canevas** sur
//! la case — elle remplit l'écran, à ses proportions —, cache l'interface comme le mode
//! référence (`UiState::sans_interface`), voile de noir tout ce qui n'est pas elle et pose ses
//! textes en sous-titre (`renderer::lecture`). Ce qui se voit est exactement ce que la case
//! porte : les images, à leur netteté, par les deux voies.
//!
//! # Une coupe, pas un vol
//!
//! Une animatique **coupe** d'une case à l'autre : la vue saute, sans glisser. Entre deux
//! coupes, Glucose dort — la boucle se réveille à la suivante (`Raison::Animation`), et le
//! repos reste à zéro image.
//!
//! # La main
//!
//! Échap (ou le retour du téléphone) et la croix en haut à droite rendent la vue d'avant ; les
//! flèches passent d'une case à l'autre ; Espace suspend et reprend. Un clic — un toucher —
//! sur le tiers gauche revient d'une case, ailleurs avance. Rien d'autre ne passe : la lecture
//! prend toute la main, comme une question.

use crate::app::GlucoseApp;
use glucose_core::types::Viewport;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

/// **Une lecture en cours.**
#[derive(Debug, Clone, PartialEq)]
pub struct Lecture {
    pub board: String,
    /// Le storyboard qu'elle lit (fiche 68).
    pub storyboard: String,
    /// Le rang de la case montrée, dans l'ordre des numéros.
    pub rang: usize,
    pub temps: Temps,
    /// La vue d'avant, rendue à la fin.
    pub vue_d_avant: Viewport,
}

/// **Le temps de la case montrée**, sur l'horloge de l'application (`now_ms`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Temps {
    /// Elle se montre depuis cet instant.
    Depuis(i64),
    /// En pause, après s'être montrée tant de millisecondes.
    EnPause(i64),
}

impl Temps {
    /// Combien elle s'est montrée, à `maintenant`.
    pub fn ecoule(self, maintenant: i64) -> i64 {
        match self {
            Self::Depuis(t) => maintenant - t,
            Self::EnPause(e) => e,
        }
    }

    /// **Ce qui reste d'une case de `duree_ms`** — `None` en pause : rien ne presse.
    pub fn reste(self, duree_ms: u32, maintenant: i64) -> Option<i64> {
        match self {
            Self::Depuis(_) => Some(i64::from(duree_ms) - self.ecoule(maintenant)),
            Self::EnPause(_) => None,
        }
    }

    /// Suspendre, ou reprendre là où l'on en était.
    pub fn basculer(self, maintenant: i64) -> Self {
        match self {
            Self::Depuis(_) => Self::EnPause(self.ecoule(maintenant)),
            Self::EnPause(e) => Self::Depuis(maintenant - e),
        }
    }
}

/// **La vue qui montre `cadre` en entier dans `ecran`**, aussi grand qu'il tient, centré :
/// `(x, y, largeur, hauteur)` du monde dans `(gauche, haut, largeur, hauteur)` de l'écran.
pub fn vue_sur_la_case(cadre: (f64, f64, f64, f64), ecran: (f64, f64, f64, f64)) -> Viewport {
    let (x, y, w, h) = cadre;
    let (ex, ey, ew, eh) = ecran;
    let scale = (ew / w).min(eh / h);
    Viewport {
        scale,
        x: ex + ew / 2.0 - (x + w / 2.0) * scale,
        y: ey + eh / 2.0 - (y + h / 2.0) * scale,
    }
}

impl GlucoseApp {
    /// **Lit le storyboard de cette case, depuis elle** — ou le storyboard courant depuis sa
    /// première (fiche 68). Sans case, rien.
    pub(crate) fn lire_le_storyboard(&mut self, depuis: Option<&str>) {
        let board = self.store.project.active_board_id.clone();
        let de_la_case = depuis.and_then(|id| self.store.storyboard_de_la_case(&board, id));
        let Some(storyboard) = de_la_case
            .map(|s| s.id.clone())
            .or_else(|| self.storyboard_courant())
        else {
            return;
        };
        let cases = self.store.cases(&board, &storyboard);
        if cases.is_empty() {
            return;
        }
        let rang = depuis
            .and_then(|id| cases.iter().position(|c| c.id == id))
            .unwrap_or(0);
        self.ui.context_menu_at = None;
        self.ui.lecture = Some(Lecture {
            board,
            storyboard,
            rang,
            temps: Temps::Depuis(self.now_ms()),
            vue_d_avant: self.store.viewport(),
        });
        let (w, h) = self.taille_de_la_fenetre();
        self.cadrer_la_lecture(w as u32, h as u32);
        self.mark_dirty();
    }

    /// **La fin de la lecture** : la vue d'avant revient, et l'interface.
    pub(crate) fn arreter_la_lecture(&mut self) {
        let Some(lecture) = self.ui.lecture.take() else {
            return;
        };
        self.store.set_viewport(&lecture.board, lecture.vue_d_avant);
        self.mark_dirty();
    }

    /// **Pendant une lecture, ce qu'on demande à la caméra se perd** (fiche 68) — l'élan qu'on
    /// avait lancé, ce que la molette, le pavé ou deux doigts demandent, un vol en route. Rien
    /// ne reste dû : sans quoi la vue d'avant, à peine rendue, partirait de tout ce qu'on a
    /// pincé pendant la lecture. Une seule règle, à chaque image, au lieu d'une garde à chaque
    /// entrée.
    pub(crate) fn oublier_ce_qu_on_demande_a_la_camera(&mut self) {
        self.elan = Default::default();
        self.toucher.attente = None;
        self.vol.poser();
    }

    /// **Passe de `pas` cases** : la suivante commence maintenant. Au-delà de la dernière, la
    /// lecture finit ; avant la première, elle reste sur elle.
    pub(crate) fn passer_la_case(&mut self, pas: isize) {
        let maintenant = self.now_ms();
        let Some(lecture) = self.ui.lecture.as_mut() else {
            return;
        };
        let nombre = self.store.cases(&lecture.board, &lecture.storyboard).len();
        let vise = lecture.rang as isize + pas;
        if vise >= nombre as isize {
            self.arreter_la_lecture();
            return;
        }
        lecture.rang = vise.max(0) as usize;
        lecture.temps = match lecture.temps {
            Temps::Depuis(_) => Temps::Depuis(maintenant),
            Temps::EnPause(_) => Temps::EnPause(0),
        };
        let (w, h) = self.taille_de_la_fenetre();
        self.cadrer_la_lecture(w as u32, h as u32);
        self.mark_dirty();
    }

    /// **La caméra de la lecture**, à chaque image : la case montrée remplit ce que l'écran
    /// laisse voir — les marges du système ôtées. Rend vrai s'il y a une lecture.
    pub(crate) fn cadrer_la_lecture(&mut self, largeur: u32, hauteur: u32) -> bool {
        let Some(lecture) = &self.ui.lecture else {
            return false;
        };
        let Some(cadre) = self
            .store
            .cases(&lecture.board, &lecture.storyboard)
            .get(lecture.rang)
            .map(|c| (c.x, c.y, c.width, c.height))
        else {
            return true;
        };
        let m = self.ui.marges;
        let ecran = (
            f64::from(m.gauche),
            f64::from(m.haut),
            f64::from(largeur as f32 - m.gauche - m.droite),
            f64::from(hauteur as f32 - m.haut - m.sous()),
        );
        let vue = vue_sur_la_case(cadre, ecran);
        let board = lecture.board.clone();
        if self.store.viewport() != vue {
            self.store.set_viewport(&board, vue);
        }
        true
    }

    /// **L'attente de la lecture** : ce qui reste de la case montrée ; à son terme, la coupe.
    pub(crate) fn attente_de_la_lecture(&mut self) -> Option<u64> {
        let reste = {
            let lecture = self.ui.lecture.as_ref()?;
            let duree = self.duree_lue(lecture)?;
            lecture.temps.reste(duree, self.now_ms())?
        };
        if reste > 0 {
            return Some(reste as u64);
        }
        self.passer_la_case(1);
        let lecture = self.ui.lecture.as_ref()?;
        self.duree_lue(lecture).map(u64::from)
    }

    /// La durée de la case que cette lecture montre.
    fn duree_lue(&self, lecture: &Lecture) -> Option<u32> {
        let storyboard = self.store.storyboard(&lecture.board, &lecture.storyboard)?;
        let cases = self.store.cases(&lecture.board, &lecture.storyboard);
        Some(storyboard.duree(cases.get(lecture.rang)?))
    }

    /// **Une touche pendant la lecture** : elle est pour elle, toute. Rend vrai s'il y en a une.
    pub(crate) fn touche_de_la_lecture(&mut self, key: &Key, appuyee: bool) -> bool {
        if self.ui.lecture.is_none() {
            return false;
        }
        if !appuyee {
            return true;
        }
        match key {
            Key::Named(NamedKey::Escape | NamedKey::BrowserBack) => self.arreter_la_lecture(),
            Key::Named(NamedKey::ArrowRight | NamedKey::ArrowDown | NamedKey::PageDown) => {
                self.passer_la_case(1)
            }
            Key::Named(NamedKey::ArrowLeft | NamedKey::ArrowUp | NamedKey::PageUp) => {
                self.passer_la_case(-1)
            }
            Key::Named(NamedKey::Home) => self.passer_la_case(isize::MIN / 2),
            Key::Named(NamedKey::Space) => self.suspendre_la_lecture(),
            _ => {}
        }
        true
    }

    /// Espace : suspendre, ou reprendre.
    fn suspendre_la_lecture(&mut self) {
        let maintenant = self.now_ms();
        if let Some(lecture) = self.ui.lecture.as_mut() {
            lecture.temps = lecture.temps.basculer(maintenant);
            self.mark_dirty();
        }
    }

    /// **Un bouton pendant la lecture** : il est pour elle, tout. La croix finit ; le tiers
    /// gauche revient d'une case, le reste avance. Rend vrai s'il y en a une.
    pub(crate) fn bouton_pendant_la_lecture(&mut self, bouton: MouseButton) -> bool {
        if self.ui.lecture.is_none() {
            return false;
        }
        if bouton != MouseButton::Left {
            return true;
        }
        let (w, _) = self.taille_de_la_fenetre();
        let p = self.pointer();
        let croix = crate::renderer::lecture::croix(w, self.ui.marges.haut, self.ui.scale());
        if croix.0 <= p.x && p.x <= croix.0 + croix.2 && croix.1 <= p.y && p.y <= croix.1 + croix.3
        {
            self.arreter_la_lecture();
        } else if p.x < w / 3.0 {
            self.passer_la_case(-1);
        } else {
            self.passer_la_case(1);
        }
        true
    }
}

#[cfg(test)]
mod tests;
