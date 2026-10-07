//! **Le mode référence, à la PureRef** (fiche 51 § 5) : garder sa référence au-dessus de
//! Blender, et rien d'autre à l'écran qu'elle.
//!
//! # Ce que le mode fait
//!
//! * **plus aucune interface** : ni bande, ni onglets, ni minimap, ni panneaux, ni barre
//!   d'action — le canevas prend toute la fenêtre ;
//! * **plus de cadre** : la fenêtre n'a plus de barre de titre ni de bordure ;
//! * **toujours au premier plan** ;
//! * **retenu d'une session à l'autre** : relancé, Glucose reprend le mode où on l'a laissé ;
//! * **défait par le même geste** : `Ctrl+Maj+A`, ou l'entrée du menu.
//!
//! Le menu contextuel et les messages restent : sans eux, on ne saurait plus sortir du mode.
//!
//! # Les gestes sont ceux de PureRef
//!
//! Son manuel les donne ([raccourcis par défaut](https://www.pureref.com/handbook/shortcuts/all-shortcuts/)) :
//! **glisser au bouton droit déplace la fenêtre**, le bouton gauche sur un bord la redimensionne,
//! et `Ctrl+Maj+A` la met au premier plan — c'est donc le geste de tout le mode, celui que la
//! main d'un utilisateur de PureRef connaît déjà. Un clic droit sans bouger ouvre toujours le
//! menu ; le bouton du milieu déplace toujours la vue. `Alt+T`, l'ancien geste caché qui ne
//! faisait que le premier plan (et l'oubliait à la relance), fait désormais tout le mode.
//!
//! # REFERENCE-2 — au pavé tactile, `Alt`
//!
//! Son essai du 07/10 : il n'a **pas de souris**. Glisser au bouton droit et viser une bordure
//! de huit pixels sont des gestes de souris, presque impossibles au pavé. `Alt` donne donc la
//! main à la fenêtre : **`Alt` + glisser la déplace**, par le geste natif du système (il colle
//! aux bords de l'écran comme toute fenêtre), et **`Alt` + pincer l'agrandit ou la rétrécit**
//! autour de son centre, du même gain que le zoom. `Alt` au clic, ailleurs, fouille une pile
//! de nœuds superposés : en mode référence on regarde plus qu'on ne range, et la fenêtre passe
//! d'abord.
//!
//! # La bordure qui redimensionne
//!
//! Huit pixels logiques : la bordure de redimensionnement que Windows donne à toute fenêtre
//! (`SM_CXSIZEFRAME` + `SM_CXPADDEDBORDER`, quatre et quatre à cent pour cent). Une fenêtre sans
//! cadre se prend donc au même endroit qu'une fenêtre ordinaire.

use crate::app::GlucoseApp;
use winit::window::{ResizeDirection, WindowLevel};

/// Le fichier qui retient le mode, dans le dossier où l'application habite.
const SOUVENIR: &str = "mode-reference";

/// La bordure de redimensionnement, en pixels logiques.
const BORD: f64 = 8.0;
/// Le plus petit côté qu'un pincement laisse à la fenêtre, en pixels logiques : celui sous
/// lequel Windows lui-même refuse de rétrécir une fenêtre ordinaire à la souris.
const COTE_MINIMAL: f64 = 160.0;

/// **REFERENCE-3 — ce que les pincements demandent à la fenêtre**, en attendant l'image.
///
/// La première version redimensionnait la fenêtre **à chaque événement du pavé** — une
/// centaine par seconde. Chaque taille reconstruit la surface de la carte graphique et le
/// tampon de l'écran ; sa session du 07/10 s'est terminée par une rafale d'images à 12 ms dans
/// `present`, puis plus rien, et un PC « qui a failli planter ». Les pincements s'additionnent
/// donc ici, et la fenêtre ne change de taille qu'une fois par image — et seulement quand
/// Windows a appliqué la taille précédente.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PincementDeFenetre {
    /// Les octaves demandées depuis la dernière taille appliquée.
    octaves: f64,
    /// La taille demandée en dernier, et quand : la suivante attend qu'elle soit là.
    demandee: Option<((u32, u32), std::time::Instant)>,
}

impl PincementDeFenetre {
    /// Les octaves demandées et pas encore appliquées.
    pub fn en_attente(&self) -> f64 {
        self.octaves
    }
}

/// **Une nouvelle taille peut-elle partir ?** Quand la précédente est appliquée — ou qu'elle ne
/// le sera plus : Windows borne une fenêtre à sa guise, et une taille refusée ne doit pas
/// bloquer le geste. Le délai est celui d'une image à la cadence la plus basse que la charte
/// admet : au-delà, la demande n'est plus en route, elle est refusée.
pub fn peut_redimensionner(
    demandee: Option<((u32, u32), std::time::Instant)>,
    actuelle: (u32, u32),
    maintenant: std::time::Instant,
) -> bool {
    demandee.is_none_or(|(taille, quand)| {
        taille == actuelle || maintenant.duration_since(quand) > crate::cadence::BUDGET_TOTAL
    })
}

/// **Ce que le mode référence fait à la fenêtre** : le déplacement au bouton droit en cours, et
/// les pincements qui attendent l'image.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FenetreDeReference {
    pub deplacement: Option<Deplacement>,
    pub pincement: PincementDeFenetre,
}

/// Un déplacement de fenêtre au bouton droit : où était le curseur sur l'écran, et la fenêtre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deplacement {
    curseur: (f64, f64),
    fenetre: (i32, i32),
    /// La fenêtre a-t-elle bougé ? Alors le relâchement n'ouvre pas le menu.
    a_bouge: bool,
}

impl GlucoseApp {
    /// `Ctrl+Maj+A`, `Alt+T` et l'entrée du menu.
    pub(crate) fn basculer_le_mode_reference(&mut self) {
        self.poser_le_mode_reference(!self.ui.reference);
        let souvenir = self.souvenir_du_mode_reference();
        // Un souvenir qui ne s'écrit pas coûte la relance, jamais le mode : il ne se dit pas.
        if self.ui.reference {
            let _ = crate::persist::atomic::write_atomic(&souvenir, b"1");
        } else {
            let _ = std::fs::remove_file(&souvenir);
        }
    }

    /// Le mode, appliqué à l'interface et à la fenêtre.
    pub(crate) fn poser_le_mode_reference(&mut self, actif: bool) {
        self.ui.reference = actif;
        self.ui.context_menu_at = None;
        if let Some(fenetre) = &self.window {
            // **Une référence flotte, elle ne couvre pas l'écran** : agrandie et sans cadre, la
            // fenêtre ressemblait exactement à un jeu en plein écran fenêtré, et l'application
            // NVIDIA lui a appliqué RTX HDR et la vibrance — toutes ses couleurs changées, son
            // essai du 07/10 (fiche 52). Elle se désagrandit donc en entrant dans le mode.
            if actif && fenetre.is_maximized() {
                fenetre.set_maximized(false);
            }
            fenetre.set_decorations(!actif);
            fenetre.set_window_level(niveau(actif));
        }
        // La bande part ou revient : toute la vue change de cadre.
        self.mark_dirty();
    }

    /// Le mode était-il actif à la fin de la session précédente ?
    pub(crate) fn mode_reference_retenu(&self) -> bool {
        self.souvenir_du_mode_reference().exists()
    }

    fn souvenir_du_mode_reference(&self) -> std::path::PathBuf {
        self.disque
            .brouillons
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(SOUVENIR)
    }

    /// **Le bouton droit s'enfonce** : en mode référence, un déplacement de fenêtre peut
    /// commencer. Rend `true` s'il a commencé — la vue, alors, ne bouge pas.
    pub(crate) fn commencer_a_deplacer_la_fenetre(&mut self) -> bool {
        if !self.ui.reference {
            return false;
        }
        let Some(fenetre) = &self.window else {
            return false;
        };
        let (Ok(dedans), Ok(dehors)) = (fenetre.inner_position(), fenetre.outer_position()) else {
            return false;
        };
        self.fenetre_de_reference.deplacement = Some(Deplacement {
            curseur: (
                f64::from(dedans.x) + self.mouse_pos.0,
                f64::from(dedans.y) + self.mouse_pos.1,
            ),
            fenetre: (dehors.x, dehors.y),
            a_bouge: false,
        });
        true
    }

    /// Le curseur bouge pendant un déplacement de fenêtre : la fenêtre suit, au pixel.
    ///
    /// Le curseur se lit **sur l'écran** — position de la fenêtre plus position dans la
    /// fenêtre —, parce que la fenêtre bouge sous lui : relu dans la fenêtre, il ne bougerait
    /// presque pas.
    pub(crate) fn deplacer_la_fenetre(&mut self) -> bool {
        let Some(d) = self.fenetre_de_reference.deplacement.as_mut() else {
            return false;
        };
        let Some(fenetre) = &self.window else {
            return true;
        };
        let Ok(dedans) = fenetre.inner_position() else {
            return true;
        };
        let ecran = (
            f64::from(dedans.x) + self.mouse_pos.0,
            f64::from(dedans.y) + self.mouse_pos.1,
        );
        let (dx, dy) = (ecran.0 - d.curseur.0, ecran.1 - d.curseur.1);
        if dx != 0.0 || dy != 0.0 {
            d.a_bouge = true;
            fenetre.set_outer_position(winit::dpi::PhysicalPosition::new(
                d.fenetre.0 + dx.round() as i32,
                d.fenetre.1 + dy.round() as i32,
            ));
        }
        true
    }

    /// Le bouton droit se relâche : le déplacement finit. Rend `true` si la fenêtre a bougé —
    /// le menu, alors, ne s'ouvre pas.
    pub(crate) fn finir_de_deplacer_la_fenetre(&mut self) -> bool {
        self.fenetre_de_reference
            .deplacement
            .take()
            .is_some_and(|d| d.a_bouge)
    }

    /// Le bord de la fenêtre sous ce point, en mode référence.
    pub(crate) fn bord_sous(&self, (x, y): (f64, f64)) -> Option<ResizeDirection> {
        if !self.ui.reference {
            return None;
        }
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        bord(
            (x, y),
            (f64::from(largeur), f64::from(hauteur)),
            BORD * self.scale_factor,
        )
    }

    /// **Le bouton gauche sur un bord** : la fenêtre se redimensionne, par le système.
    pub(crate) fn redimensionner_par_le_bord(&mut self) -> bool {
        let Some(direction) = self.bord_sous(self.mouse_pos) else {
            return false;
        };
        if let Some(fenetre) = &self.window {
            let _ = fenetre.drag_resize_window(direction);
        }
        true
    }
}

impl GlucoseApp {
    /// **`Alt` + glisser, en mode référence** : la fenêtre se déplace, par le système
    /// (REFERENCE-2). Rend `true` si le geste lui revient — même sans fenêtre, il n'est pas
    /// au canevas.
    pub(crate) fn deplacer_la_fenetre_avec_alt(&mut self) -> bool {
        if !(self.ui.reference && self.modifiers.alt_key()) {
            return false;
        }
        if let Some(fenetre) = &self.window {
            let _ = fenetre.drag_window();
        }
        true
    }

    /// **`Alt` + pincer, en mode référence** : la fenêtre grandit ou rétrécit de tant
    /// d'octaves, autour de son centre (REFERENCE-2). Rend `true` si le geste lui revient. Le
    /// pincement s'additionne, et l'image l'appliquera (REFERENCE-3).
    pub(crate) fn redimensionner_au_pincement(&mut self, octaves: f64) -> bool {
        if !(self.ui.reference && self.modifiers.alt_key()) {
            return false;
        }
        self.fenetre_de_reference.pincement.octaves += octaves;
        self.mark_dirty();
        true
    }

    /// **Applique à la fenêtre ce que les pincements ont demandé** — au plus une fois par
    /// image, et quand la taille précédente est là (REFERENCE-3).
    pub(crate) fn appliquer_le_pincement_de_fenetre(&mut self) {
        let octaves = self.fenetre_de_reference.pincement.octaves;
        if octaves == 0.0 {
            return;
        }
        let Some(fenetre) = self.window.clone() else {
            self.fenetre_de_reference.pincement.octaves = 0.0;
            return;
        };
        let actuelle = fenetre.inner_size();
        let maintenant = std::time::Instant::now();
        if !peut_redimensionner(
            self.fenetre_de_reference.pincement.demandee,
            (actuelle.width, actuelle.height),
            maintenant,
        ) {
            // L'image suivante réessaiera : la demande reste due.
            self.mark_dirty();
            return;
        }
        self.fenetre_de_reference.pincement.octaves = 0.0;
        // La taille intérieure, celle qu'on demande et qu'on attend : sans cadre en mode
        // référence, elle est aussi l'extérieure.
        let (Ok(coin), taille) = (fenetre.outer_position(), actuelle) else {
            return;
        };
        let ecran = fenetre
            .current_monitor()
            .map(|m| (f64::from(m.size().width), f64::from(m.size().height)));
        let ((x, y), (l, h)) = cadre_apres_pincement(
            (f64::from(coin.x), f64::from(coin.y)),
            (f64::from(taille.width), f64::from(taille.height)),
            octaves,
            COTE_MINIMAL * self.scale_factor,
            ecran,
        );
        let _ = fenetre.request_inner_size(winit::dpi::PhysicalSize::new(l, h));
        fenetre.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
        self.fenetre_de_reference.pincement.demandee = Some(((l, h), maintenant));
    }
}

/// **Le cadre d'une fenêtre après un pincement** de `octaves` : sa taille multipliée par
/// `2^octaves`, son centre gardé ; jamais un côté sous `minimal`, jamais plus grande que
/// `ecran` — les proportions tenues dans les deux cas.
pub fn cadre_apres_pincement(
    (x, y): (f64, f64),
    (l, h): (f64, f64),
    octaves: f64,
    minimal: f64,
    ecran: Option<(f64, f64)>,
) -> ((i32, i32), (u32, u32)) {
    let mut facteur = octaves.exp2();
    facteur = facteur.max(minimal / l.min(h).max(1.0));
    if let Some((el, eh)) = ecran {
        facteur = facteur.min((el / l.max(1.0)).min(eh / h.max(1.0)));
    }
    let (nl, nh) = (l * facteur, h * facteur);
    let (cx, cy) = (x + l / 2.0, y + h / 2.0);
    (
        (
            (cx - nl / 2.0).round() as i32,
            (cy - nh / 2.0).round() as i32,
        ),
        (nl.round() as u32, nh.round() as u32),
    )
}

fn niveau(actif: bool) -> WindowLevel {
    if actif {
        WindowLevel::AlwaysOnTop
    } else {
        WindowLevel::Normal
    }
}

/// Le bord d'un rectangle `taille` sous ce point, à `epaisseur` près — coins compris.
pub fn bord(
    (x, y): (f64, f64),
    (largeur, hauteur): (f64, f64),
    epaisseur: f64,
) -> Option<ResizeDirection> {
    let gauche = x < epaisseur;
    let droite = x >= largeur - epaisseur;
    let haut = y < epaisseur;
    let bas = y >= hauteur - epaisseur;
    Some(match (gauche, droite, haut, bas) {
        (true, _, true, _) => ResizeDirection::NorthWest,
        (_, true, true, _) => ResizeDirection::NorthEast,
        (true, _, _, true) => ResizeDirection::SouthWest,
        (_, true, _, true) => ResizeDirection::SouthEast,
        (true, ..) => ResizeDirection::West,
        (_, true, ..) => ResizeDirection::East,
        (_, _, true, _) => ResizeDirection::North,
        (_, _, _, true) => ResizeDirection::South,
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
