//! Le panneau POMODORO : un anneau qui décompte pour de bon, trois durées, démarrer et
//! remettre à zéro.

use super::WidgetRect;
use crate::params::{Pointer, ScaledRect};
use crate::typography::{Face, Typography};
use std::time::Instant;

pub mod paint;

/// Les trois durées proposées, en secondes, dans l'ordre du panneau (fiche 10 § 5.2).
pub const PRESETS: [(&str, u32); 3] = [("25 min", 25 * 60), ("15 min", 15 * 60), ("5 min", 5 * 60)];

#[derive(Debug, Clone)]
pub struct PomodoroState {
    pub total_seconds: u32,
    pub left_seconds: u32,
    pub running: bool,
    pub last_tick: Instant,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self {
            total_seconds: 25 * 60,
            left_seconds: 25 * 60,
            running: false,
            last_tick: Instant::now(),
        }
    }
}

impl PomodoroState {
    /// Fait avancer le décompte d'autant de secondes qu'il s'en est écoulé. Rend `true` si
    /// l'affichage doit changer.
    pub fn tick(&mut self) -> bool {
        if !self.running {
            return false;
        }
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_tick).as_secs();
        if elapsed == 0 {
            return false;
        }
        let elapsed = u32::try_from(elapsed).unwrap_or(u32::MAX);
        if self.left_seconds <= elapsed {
            self.left_seconds = 0;
            self.running = false;
        } else {
            self.left_seconds -= elapsed;
        }
        self.last_tick = now;
        true
    }

    fn toggle(&mut self) {
        self.running = !self.running;
        self.last_tick = Instant::now();
    }

    fn reset(&mut self) {
        self.left_seconds = self.total_seconds;
        self.running = false;
    }

    fn set_duration(&mut self, seconds: u32) {
        self.total_seconds = seconds;
        self.left_seconds = seconds;
        self.running = false;
    }

    /// Ce qu'il reste, entre 0 (fini) et 1 (pas commencé).
    pub fn progress(&self) -> f32 {
        if self.total_seconds == 0 {
            return 1.0;
        }
        1.0 - self.left_seconds as f32 / self.total_seconds as f32
    }

    pub fn start_label(&self) -> &'static str {
        if self.running {
            Self::START_LABELS[0]
        } else {
            Self::START_LABELS[1]
        }
    }

    /// Les deux libellés du bouton de départ. Ils sont déclarés ensemble parce que la mise en
    /// page a besoin des **deux** : elle réserve la largeur du plus long (voir
    /// [`layout_pomodoro_panel`]).
    const START_LABELS: [&'static str; 2] = ["Pause", "Démarrer"];
}

#[derive(Debug, Clone)]
pub struct PomodoroPresetButton {
    pub label: &'static str,
    pub seconds: u32,
    pub rect: WidgetRect,
}

#[derive(Debug, Clone)]
pub struct PomodoroPanelLayout {
    pub ring_center: (f32, f32),
    pub ring_radius: f32,
    pub start_button: WidgetRect,
    pub reset_button: WidgetRect,
    pub presets: Vec<PomodoroPresetButton>,
}

/// La géométrie du panneau — la seule, lue par le dessin et par le clic.
///
/// # Le bouton de départ ne change pas de taille
///
/// Son libellé, lui, change : « Démarrer » puis « Pause ». Si sa largeur suivait, le bouton
/// `↺` se déplacerait de dix pixels sous le curseur à chaque clic — et un utilisateur qui
/// lance puis remet à zéro cliquerait à côté. La mise en page réserve donc la largeur du
/// **plus long** des deux libellés : rien ne bouge, et le panneau ne dépend plus de l'état
/// du minuteur pour sa géométrie.
pub fn layout_pomodoro_panel(frame: ScaledRect, typo: &Typography) -> PomodoroPanelLayout {
    let s = crate::theme::clamp_ui_scale(frame.scale);
    let (px, py, pw) = (frame.x, frame.y, frame.w);

    let by = py + 124.0 * s;
    let widest = PomodoroState::START_LABELS
        .iter()
        .map(|l| typo.measure_text(l, 11.0 * s, Face::Regular).0)
        .fold(0.0_f32, f32::max);
    let bw = widest + 22.0 * s;
    let start_button = WidgetRect::new(px + 22.0 * s, by, bw, 22.0 * s);
    let reset_button = WidgetRect::new(px + 22.0 * s + bw + 6.0 * s, by, 22.0 * s, 22.0 * s);

    let row_y = by + 28.0 * s;
    let preset_w = (pw - 28.0 * s - 8.0 * s) / 3.0;
    let presets = PRESETS
        .into_iter()
        .enumerate()
        .map(|(i, (label, seconds))| PomodoroPresetButton {
            label,
            seconds,
            rect: WidgetRect::new(
                px + 14.0 * s + i as f32 * (preset_w + 4.0 * s),
                row_y,
                preset_w,
                18.0 * s,
            ),
        })
        .collect();

    PomodoroPanelLayout {
        ring_center: (px + pw / 2.0, py + 72.0 * s),
        ring_radius: 32.0 * s,
        start_button,
        reset_button,
        presets,
    }
}

/// Ce qu'un clic dans le panneau a fait à l'état.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PomodoroClick {
    Handled,
    Toggled,
    Reset,
    DurationSet(u32),
}

pub fn click_pomodoro_panel(
    state: &mut PomodoroState,
    layout: &PomodoroPanelLayout,
    pointer: Pointer,
) -> PomodoroClick {
    let hit = |r: WidgetRect| r.contains(pointer.x, pointer.y);
    if hit(layout.start_button) {
        state.toggle();
        return PomodoroClick::Toggled;
    }
    if hit(layout.reset_button) {
        state.reset();
        return PomodoroClick::Reset;
    }
    if let Some(preset) = layout.presets.iter().find(|p| hit(p.rect)) {
        state.set_duration(preset.seconds);
        return PomodoroClick::DurationSet(preset.seconds);
    }
    PomodoroClick::Handled
}
