//! **FRISE-1 — les gestes de la frise chronologique** (fiche 114) : l'ouvrir et la fermer
//! (`Maj+R`, la palette, sa croix), glisser une poignée ou la fenêtre, zoomer et faire glisser
//! l'échelle — la molette et le pavé par **la loi du canevas** (`pan_zoom::geste` : un cran de
//! molette zoome d'un tiers d'octave autour de la main, deux doigts font glisser).
//!
//! La fenêtre est une **lentille** : elle change ce qu'on regarde, jamais le document ni le
//! journal — comme la caméra. Fermer la frise la retire.

use crate::app::GlucoseApp;
use crate::params::{Pointer, ScreenFrame};
use crate::ui::frise::{an_de, endroit, placer, Endroit, Frise, Placee, Prise, Tenue};
use glucose_core::store::Store;
use glucose_core::timeline::{etendue, tick_step, TemporalFilter, YEAR_MAX, YEAR_MIN};
use winit::event::MouseScrollDelta;

/// **Ce que la frise montre quand rien n'est daté** : la vue que Glucose Tauri ouvrait.
const VUE_DE_TAURI: TemporalFilter = TemporalFilter {
    start: -1_000,
    end: 2_050,
};

/// Une vue dans les bornes du modèle, jamais à l'envers.
fn borner(v: TemporalFilter) -> TemporalFilter {
    let start = v.start.clamp(YEAR_MIN, YEAR_MAX);
    TemporalFilter {
        start,
        end: v.end.clamp(start, YEAR_MAX),
    }
}

/// **La vue d'ouverture** : les dates du document, et une graduation majeure de part et d'autre
/// — cinq pas de leurs graduations ; la vue de Tauri quand rien n'est daté. Une plage choisie
/// d'avance (`-1000..2050`) ne montrait pas un document de géologie.
pub(crate) fn vue_initiale(store: &Store) -> TemporalFilter {
    let Some(e) = etendue(store.dates_du_tableau()) else {
        return VUE_DE_TAURI;
    };
    let marge = 5 * tick_step(e.end - e.start);
    borner(TemporalFilter {
        start: e.start - marge,
        end: e.end + marge,
    })
}

/// L'étendue d'une vue, en années (une année compte pour une).
fn annees(v: TemporalFilter) -> f64 {
    (v.end - v.start) as f64 + 1.0
}

/// **La vue après un geste** : `octaves` de zoom autour de l'année sous `x` (un tiers d'octave par
/// cran : zoomer resserre l'échelle), puis `glisse` points vers la droite. Jamais moins d'une
/// année, jamais plus que le modèle.
pub(crate) fn vue_apres(
    vue: TemporalFilter,
    piste: crate::ui::frise::Boite,
    x: f32,
    (octaves, glisse): (f64, f32),
) -> TemporalFilter {
    let ici = an_de(x, vue, piste);
    let tout = (YEAR_MAX - YEAR_MIN) as f64 + 1.0;
    let neuve = (annees(vue) * 2f64.powf(-octaves)).clamp(1.0, tout);
    let par_point = neuve / f64::from(piste.2.max(1.0));
    let debut = ici - f64::from(x - piste.0) * par_point - f64::from(glisse) * par_point;
    let start = debut.round() as i64;
    let end = start + neuve.round() as i64 - 1;
    // Une vue qui touche un bord du modèle s'y appuie au lieu de se rétrécir.
    let (start, end) = if start < YEAR_MIN {
        (YEAR_MIN, YEAR_MIN + (end - start))
    } else if end > YEAR_MAX {
        (YEAR_MAX - (end - start), YEAR_MAX)
    } else {
        (start, end)
    };
    borner(TemporalFilter { start, end })
}

impl GlucoseApp {
    /// **Ouvre ou ferme la frise** (`Maj+R`) : ouverte, elle montre les dates du document et
    /// garde toute sa vue — rien ne s'estompe encore ; fermée, rien n'est plus écarté.
    pub(crate) fn basculer_la_frise(&mut self) {
        if self.ui.frise.take().is_some() {
            self.store.set_temporal_filter(None);
        } else {
            let vue = vue_initiale(&self.store);
            self.ui.frise = Some(Frise { vue, tenue: None });
            self.store.set_temporal_filter(Some(vue));
        }
        self.mark_dirty();
    }

    /// La frise posée, si elle est ouverte et l'interface montrée.
    fn frise_placee(&self, screen: ScreenFrame) -> Option<Placee> {
        if self.ui.sans_interface() || self.ui.frise.is_none() {
            return None;
        }
        let ecran = self.ui.ecran_du_systeme((screen.width, screen.height));
        Some(placer(ecran, self.ui.scale()))
    }

    /// **Un appui sur la frise** : sa croix la ferme, une poignée ou la fenêtre se prend ; elle
    /// garde tout ce qui tombe sur elle — rien ne descend au canevas qu'elle recouvre.
    pub fn click_frise(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        let (Some(p), Some(r)) = (self.frise_placee(screen), self.ui.frise) else {
            return false;
        };
        let fenetre = self.store.temporal_filter.unwrap_or(r.vue);
        match endroit(&p, (r.vue, fenetre), (pointer.x, pointer.y)) {
            None => return false,
            Some(Endroit::Fermer) => self.basculer_la_frise(),
            Some(Endroit::Prise(prise)) => {
                let par_point = annees(r.vue) / f64::from(p.piste.2.max(1.0));
                let tenue = Tenue {
                    prise,
                    depuis: pointer.x,
                    fenetre,
                    par_point,
                };
                self.ui.frise = Some(Frise {
                    tenue: Some(tenue),
                    ..r
                });
            }
            Some(Endroit::Ailleurs) => {}
        }
        self.mark_dirty();
        true
    }

    /// **La main qui tient une poignée ou la fenêtre** la suit, à l'année près. Rend `false` si la
    /// frise ne tient rien.
    pub fn glisser_la_frise(&mut self, x: f32) -> bool {
        let Some(Frise { tenue: Some(t), .. }) = self.ui.frise else {
            return false;
        };
        let d = (f64::from(x - t.depuis) * t.par_point).round() as i64;
        let f = t.fenetre;
        let neuve = match t.prise {
            Prise::Debut => TemporalFilter {
                start: (f.start + d).clamp(YEAR_MIN, f.end),
                ..f
            },
            Prise::Fin => TemporalFilter {
                end: (f.end + d).clamp(f.start, YEAR_MAX),
                ..f
            },
            Prise::Fenetre => {
                let d = d.clamp(YEAR_MIN - f.start, YEAR_MAX - f.end);
                TemporalFilter {
                    start: f.start + d,
                    end: f.end + d,
                }
            }
        };
        if self.store.temporal_filter != Some(neuve) {
            self.store.set_temporal_filter(Some(neuve));
            self.mark_dirty();
        }
        true
    }

    /// **La main lâche** ce que la frise tenait.
    pub fn relacher_la_frise(&mut self) -> bool {
        match &mut self.ui.frise {
            Some(r) if r.tenue.is_some() => {
                r.tenue = None;
                self.mark_dirty();
                true
            }
            _ => false,
        }
    }

    /// **La molette ou le pavé sur la frise** : son échelle zoome ou glisse, par la loi du
    /// canevas ; le canevas, sous elle, ne bouge pas. Rend `false` hors d'elle.
    pub(crate) fn molette_vers_la_frise(&mut self, delta: MouseScrollDelta) -> bool {
        let (l, h) = self.taille_de_la_fenetre();
        let ici = (self.mouse_pos.0 as f32, self.mouse_pos.1 as f32);
        self.molette_sur_la_frise(delta, ici, self.screen_frame(l, h))
    }

    /// La même, sur un écran donné.
    pub fn molette_sur_la_frise(
        &mut self,
        delta: MouseScrollDelta,
        (x, y): (f32, f32),
        screen: ScreenFrame,
    ) -> bool {
        let (Some(p), Some(r)) = (self.frise_placee(screen), self.ui.frise) else {
            return false;
        };
        let fenetre = self.store.temporal_filter.unwrap_or(r.vue);
        if endroit(&p, (r.vue, fenetre), (x, y)).is_none() {
            return false;
        }
        let pincement = super::pincement::zoom_du_systeme();
        let doigt = self.defilement_au_doigt || super::pan_zoom::source_continue(delta);
        let ctrl = self.modifiers.control_key();
        let geste = match super::pan_zoom::geste(delta, ctrl, pincement, doigt) {
            super::pan_zoom::Geste::Zoom(o) => (o, 0.0),
            // Deux doigts : l'échelle suit le doigt, dans un sens ou dans l'autre.
            super::pan_zoom::Geste::Pan(dx, dy) => {
                (0.0, if dx.abs() >= dy.abs() { dx } else { dy } as f32)
            }
        };
        let vue = vue_apres(r.vue, p.piste, x, geste);
        self.ui.frise = Some(Frise { vue, ..r });
        self.mark_dirty();
        true
    }
}

#[cfg(test)]
mod tests;
