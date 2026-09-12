//! Les courbes et les durées des animations — `std` uniquement, sans horloge.
//!
//! # Pourquoi ce module n'attend pas le GPU
//!
//! La fiche 07 § 2 renvoie le mécanisme de tween à la refonte du rendu : *« à concevoir une
//! fois, avec le rendu GPU »*. Les deux n'ont pourtant rien à voir. Une animation est une
//! **valeur qui dépend du temps** ; qui la dessine ensuite, et avec quelle carte graphique, ne
//! change ni la courbe, ni la durée, ni la façon de les tester. Lier les deux n'aurait fait
//! qu'ajourner l'un au nom de l'autre.
//!
//! # Ce module ne connaît pas l'heure
//!
//! [`Tween::at`] reçoit le temps écoulé et rend une valeur. Il n'appelle pas `Instant::now`,
//! ne retient aucun début, ne dépend d'aucune horloge : **toute animation de Glucose est donc
//! une fonction pure, vérifiable image par image sans attendre une seule milliseconde.** C'est
//! l'appelant — la boucle d'événements, qui seule sait quelle heure il est — qui mesure le
//! temps et redemande une frame tant qu'une animation dure.
//!
//! # Les durées sont ici, et nulle part ailleurs
//!
//! [`timing`] rassemble la table chronométrique de la fiche 07 § 1. Celles qui existaient déjà
//! ailleurs y sont **réexportées**, jamais recopiées : une seconde définition finirait par
//! diverger de la première, et c'est précisément la dérive que la relecture des fiches a
//! trouvée partout.

/// La table chronométrique de Glucose (fiche 07 § 1).
///
/// Une durée de ce module est en **millisecondes**, et c'est le seul endroit du programme où
/// elle est écrite.
pub mod timing {
    /// Entrée ou sortie d'une membrane minimisée.
    pub const MEMBRANE_TWEEN_MS: u32 = 200;
    /// Plongée de la caméra à l'entrée ou à la sortie d'un dossier.
    pub const FOLDER_TRANSITION_MS: u32 = 400;
    /// Vol de la caméra vers l'original d'un miroir.
    pub const MIRROR_TELEPORT_MS: u32 = 400;
    /// Glissement d'éviction d'un panneau du dock.
    pub const PANEL_DISMISS_MS: u32 = 200;
    /// Translation de la minimap quand un panneau droit s'ouvre ou se ferme.
    pub const MINIMAP_SLIDE_MS: u32 = 180;
    /// Déploiement du panneau de description d'une flèche.
    pub const ARROW_PANEL_IN_MS: u32 = 180;
    /// Inactivité avant l'écriture disque du projet (fiche 09 § 3.3).
    pub const AUTOSAVE_DEBOUNCE_MS: u32 = 2_000;
    /// Réordonnancement d'un panneau du dock (transition FLIP).
    pub const DOCK_REORDER_MS: u32 = 250;

    /// Fenêtre du double-clic — définie par l'arbitre de clic, réexportée ici pour que la
    /// table soit complète sans être dédoublée.
    pub const DOUBLE_CLICK_MS: i64 = crate::hit_priority::pick_consts::DBLCLICK_MS;
    /// Durée de vie du cycle de profondeur, même raison.
    pub const CYCLE_TTL_MS: i64 = crate::hit_priority::pick_consts::CYCLE_TTL_MS;
    /// Cadrage du mode focus d'une membrane, défini par `membrane_focus`.
    pub const MEMBRANE_FOCUS_MS: i64 = crate::membrane_focus::focus_consts::FIT_ANIM_MS;
    /// Délai avant qu'un second focus soit accepté, même source.
    pub const FOCUS_COOLDOWN_MS: i64 = crate::membrane_focus::focus_consts::COOLDOWN_MS;
}

/// Une courbe d'amortissement : une fonction de `[0, 1]` dans `[0, 1]`, valant 0 en 0 et 1 en 1.
///
/// Les deux Béziers sont celles de la fiche 07 § 2, écrites comme la référence CSS les écrit —
/// deux points de contrôle, le premier et le dernier étant fixés à `(0, 0)` et `(1, 1)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Curve {
    /// Aucun amortissement : la valeur avance à vitesse constante.
    Linear,
    /// L'amorti universel de Glucose : `1 − (1 − t)³`. Départ vif, arrivée posée.
    EaseOutCubic,
    /// Une Bézier cubique CSS, par ses deux points de contrôle.
    ///
    /// Un point de contrôle dont l'ordonnée sort de `[0, 1]` fait **dépasser** la courbe — c'est
    /// le rebond voulu par la fiche pour la préhension du dock, pas une erreur à corriger.
    CubicBezier(f64, f64, f64, f64),
}

impl Curve {
    /// Le rebond de préhension d'un panneau du dock : il dépasse sa place avant de s'y caler.
    pub const DOCK_GRAB: Self = Self::CubicBezier(0.34, 1.56, 0.64, 1.0);
    /// Le réordonnancement d'un panneau du dock (transition FLIP).
    pub const DOCK_REORDER: Self = Self::CubicBezier(0.22, 1.0, 0.36, 1.0);

    /// La valeur de la courbe au temps normalisé `t`.
    ///
    /// `t` est rabattu dans `[0, 1]` : une animation déjà finie rend sa valeur d'arrivée plutôt
    /// que de continuer sa course, et une valeur négative — ce qu'une horloge qui recule peut
    /// produire — rend le départ.
    pub fn at(self, t: f64) -> f64 {
        let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
        match self {
            Self::Linear => t,
            Self::EaseOutCubic => {
                let u = 1.0 - t;
                1.0 - u * u * u
            }
            Self::CubicBezier(x1, y1, x2, y2) => bezier_y_at_x(t, x1, y1, x2, y2),
        }
    }
}

/// Une composante d'une Bézier cubique dont les extrémités valent 0 et 1.
fn bezier(t: f64, a: f64, b: f64) -> f64 {
    let u = 1.0 - t;
    3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
}

/// Sa dérivée, pour la recherche de racine.
fn bezier_prime(t: f64, a: f64, b: f64) -> f64 {
    let u = 1.0 - t;
    3.0 * u * u * a + 6.0 * u * t * (b - a) + 3.0 * t * t * (1.0 - b)
}

/// L'ordonnée d'une Bézier CSS à l'abscisse `x`.
///
/// Une Bézier CSS est paramétrée par un `s` qui n'est **pas** l'abscisse : il faut d'abord
/// résoudre `x(s) = x`. La résolution est faite par Newton-Raphson, avec repli sur une
/// bissection quand la dérivée s'annule — ce qui arrive sur les courbes très plates, où Newton
/// partirait à l'infini.
///
/// Huit itérations suffisent : l'erreur résiduelle est alors de l'ordre de 10⁻¹⁰, soit bien en
/// deçà du dixième de pixel qu'une animation peut faire voir.
fn bezier_y_at_x(x: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let mut s = x;
    for _ in 0..8 {
        let ecart = bezier(s, x1, x2) - x;
        if ecart.abs() < 1e-9 {
            return bezier(s, y1, y2);
        }
        let pente = bezier_prime(s, x1, x2);
        if pente.abs() < 1e-9 {
            break;
        }
        s -= ecart / pente;
    }
    // Repli : la bissection converge toujours, plus lentement mais sans condition.
    let (mut bas, mut haut) = (0.0_f64, 1.0_f64);
    let mut s = x;
    for _ in 0..40 {
        let courant = bezier(s, x1, x2);
        if (courant - x).abs() < 1e-9 {
            break;
        }
        if courant < x {
            bas = s;
        } else {
            haut = s;
        }
        s = (bas + haut) / 2.0;
    }
    bezier(s, y1, y2)
}

/// Une valeur qui va d'un point à un autre en un temps donné, le long d'une courbe.
///
/// Ne retient aucun instant : c'est l'appelant qui mesure le temps écoulé. Un tween est donc
/// copiable, comparable, et testable sans attendre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tween {
    pub from: f64,
    pub to: f64,
    pub duration_ms: u32,
    pub curve: Curve,
}

impl Tween {
    /// Un tween de `from` à `to`, en `duration_ms`, amorti par `curve`.
    pub const fn new(from: f64, to: f64, duration_ms: u32, curve: Curve) -> Self {
        Self { from, to, duration_ms, curve }
    }

    /// La valeur après `elapsed_ms` millisecondes.
    ///
    /// Une durée nulle rend immédiatement l'arrivée : une animation qui ne dure pas est un
    /// changement, et il vaut mieux qu'elle le dise que de diviser par zéro.
    pub fn at(&self, elapsed_ms: f64) -> f64 {
        if self.duration_ms == 0 {
            return self.to;
        }
        let t = self.curve.at(elapsed_ms / self.duration_ms as f64);
        self.from + (self.to - self.from) * t
    }

    /// Vrai tant que l'animation a encore quelque chose à montrer.
    pub fn running(&self, elapsed_ms: f64) -> bool {
        elapsed_ms < self.duration_ms as f64
    }

    /// Le temps restant, en millisecondes — ce qu'une boucle d'événements doit attendre avant
    /// de redemander une frame.
    pub fn remaining_ms(&self, elapsed_ms: f64) -> f64 {
        (self.duration_ms as f64 - elapsed_ms).max(0.0)
    }
}

#[cfg(test)]
mod tests;
