//! **Le toucher** (fiches 50 et 54) : les doigts sur un écran — un téléphone, une tablette.
//!
//! Jusqu'ici Glucose ignorait les doigts : sur un téléphone, rien n'aurait bougé. Trois gestes,
//! et aucun n'invente sa propre logique :
//!
//! * **Un doigt** se comporte comme le bouton gauche de la souris — l'interface, les menus, les
//!   nœuds le reçoivent par le chemin qu'ils connaissent. Sauf **sur le vide** : la souris y
//!   trace un cadre de sélection, le doigt y **déplace le canevas**, avec son élan — sur un
//!   téléphone, le canevas glisse sous le doigt.
//! * **Deux doigts** font subir au canevas la similitude exacte qu'ils décrivent : l'échelle est
//!   le rapport de leurs écarts, et le point entre eux suit leur milieu. C'est exactement le
//!   [`super::pave::Mouvement`] du pavé tactile, appliqué par le même code — un pincement et un
//!   déplacement à la fois, sans rien classer (fiche 54).
//! * Un **troisième** doigt ne change rien : les deux premiers mènent.
//! * **Les gestes à plusieurs doigts** (GESTES-1, fiche 58, [`plusieurs`]) : deux doigts touchés
//!   annulent, trois rétablissent ; après l'appui long, lever ouvre le menu, glisser trace un
//!   rectangle, un autre doigt ajoute des nœuds à la sélection.
//!
//! Le deuxième doigt **termine** ce que le premier faisait — un glisser se pose où il est — :
//! on ne pince pas en déplaçant un nœud.
//!
//! # Hors de Windows
//!
//! Sous Windows, `winit` livre les doigts **et** la souris que Windows simule à partir du
//! premier : tout arriverait deux fois. Les doigts n'y passent donc pas encore par ici — il faudra
//! d'abord reconnaître la souris simulée (`GetMessageExtraInfo`) pour l'écarter.

mod plusieurs;

use super::pave::{Mouvement, Pave};
use crate::app::GlucoseApp;
use winit::event::{MouseButton, Touch, TouchPhase};

/// Les entrées au doigt : le pavé de précision, et les doigts posés sur l'écran.
#[derive(Default)]
pub struct Toucher {
    /// **Le pavé de précision par *Direct Manipulation*** (fiche 53), quand le système le donne.
    pub pave: Option<Box<dyn Pave>>,
    pub doigts: Doigts,
    /// **Tout ce que les doigts et le pavé ont fait depuis l'image précédente**, en une seule
    /// similitude, composée exactement ([`composer`]) : l'image l'applique d'un coup. Deux
    /// doigts qui bougent l'un après l'autre en donnent deux ; les additionner (les
    /// déplacements sommés, les échelles multipliées) faisait dériver le point entre les doigts
    /// de 12,5 unités sur un pincement du double — l'épreuve l'a vu.
    pub(crate) attente: Option<Mouvement>,
    /// **Un doigt posé qui pourrait devenir un appui long** (APPUI-1, fiche 57) : où il s'est
    /// posé, et quand il aura tenu assez — tant qu'il ne bouge pas plus qu'une main qui tremble.
    pub(crate) appui: Option<Appui>,
    /// **Un toucher à plusieurs doigts qui se décide** (GESTES-1) : annuler, rétablir.
    pub(crate) bref: Option<plusieurs::Bref>,
    /// **L'appui long a pris**, et ce qui le suit décide (GESTES-1).
    pub(crate) pris: Option<plusieurs::Pris>,
}

/// Un appui qui se décide.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Appui {
    pub depart: (f64, f64),
    pub echeance: std::time::Instant,
}

impl Toucher {
    /// Ajoute une similitude à ce qui attend l'image.
    pub(crate) fn attendre(&mut self, m: Mouvement) {
        self.attente = Some(match self.attente {
            Some(avant) => composer(avant, m),
            None => m,
        });
    }
}

/// **`a` puis `b`, en une similitude** : `q ↦ r_b · (r_a · q + b_a) + b_b`, soit l'échelle
/// `r_a · r_b` et le décalage `r_b · b_a + b_b`.
pub fn composer(a: Mouvement, b: Mouvement) -> Mouvement {
    Mouvement {
        echelle: a.echelle * b.echelle,
        decalage: (
            b.echelle * a.decalage.0 + b.decalage.0,
            b.echelle * a.decalage.1 + b.decalage.1,
        ),
    }
}

/// Les doigts posés, dans l'ordre où ils se sont posés.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Doigts {
    poses: Vec<(u64, (f64, f64))>,
    /// Le premier doigt mène-t-il encore un geste de souris ? Faux dès que le deuxième s'est
    /// posé, et jusqu'à ce que tous se soient levés.
    pub(crate) seul: bool,
}

/// **La similitude que deux doigts décrivent** en passant de `avant` à `apres` : chaque point `q`
/// va en `r · q + b`, avec `r` le rapport des écarts et `b` tel que le milieu des doigts suive
/// leur milieu. Aucune quand les deux doigts étaient au même point — l'échelle n'y a pas de
/// sens.
pub fn similitude(avant: [(f64, f64); 2], apres: [(f64, f64); 2]) -> Option<Mouvement> {
    let ecart = |[a, b]: [(f64, f64); 2]| (b.0 - a.0).hypot(b.1 - a.1);
    let milieu = |[a, b]: [(f64, f64); 2]| ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    let (e0, e1) = (ecart(avant), ecart(apres));
    if e0 == 0.0 || e1 == 0.0 {
        return None;
    }
    let r = e1 / e0;
    let (c0, c1) = (milieu(avant), milieu(apres));
    Some(Mouvement {
        echelle: r,
        decalage: (c1.0 - r * c0.0, c1.1 - r * c0.1),
    })
}

// Sous Windows, les doigts ne sont pas encore branchés (en tête du module) : rien ne les lit.
#[cfg_attr(windows, allow(dead_code))]
impl Doigts {
    /// Les deux doigts qui mènent, s'ils sont là.
    fn les_deux(&self) -> Option<[(f64, f64); 2]> {
        match self.poses.as_slice() {
            [(_, a), (_, b), ..] => Some([*a, *b]),
            _ => None,
        }
    }

    fn ou(&mut self, id: u64) -> Option<&mut (f64, f64)> {
        self.poses
            .iter_mut()
            .find(|(i, _)| *i == id)
            .map(|(_, p)| p)
    }
}

#[cfg_attr(windows, allow(dead_code))]
impl GlucoseApp {
    /// **Un doigt** se pose, bouge, se lève — ou le système l'annule.
    pub(crate) fn toucher(&mut self, t: &Touch) {
        let ici = (t.location.x, t.location.y);
        match t.phase {
            TouchPhase::Started => self.poser_un_doigt(t.id, ici),
            TouchPhase::Moved => self.bouger_un_doigt(t.id, ici),
            TouchPhase::Ended | TouchPhase::Cancelled => self.lever_un_doigt(t.id),
        }
        self.mark_dirty();
    }

    fn poser_un_doigt(&mut self, id: u64, ici: (f64, f64)) {
        self.plusieurs_a_la_pose(id, ici);
        let doigts = &mut self.toucher.doigts;
        doigts.poses.push((id, ici));
        match doigts.poses.len() {
            1 => {
                doigts.seul = true;
                self.handle_cursor_moved(winit::dpi::PhysicalPosition::new(ici.0, ici.1));
                let (largeur, hauteur) = self.taille_de_la_fenetre();
                self.handle_mouse_down(MouseButton::Left, largeur, hauteur);
                self.toucher.appui = Some(Appui {
                    depart: ici,
                    echeance: std::time::Instant::now() + crate::plateforme::doigt::appui_long(),
                });
            }
            2 if doigts.seul => {
                // Le deuxième doigt termine ce que le premier faisait.
                doigts.seul = false;
                self.toucher.appui = None;
                self.handle_mouse_up(MouseButton::Left);
            }
            _ => self.toucher.appui = None,
        }
    }

    fn bouger_un_doigt(&mut self, id: u64, ici: (f64, f64)) {
        let avant = self.toucher.doigts.les_deux();
        let Some(place) = self.toucher.doigts.ou(id) else {
            return;
        };
        *place = ici;
        if self.plusieurs_au_mouvement(id, ici) {
            return;
        }
        if self.toucher.doigts.seul {
            // Un doigt qui part n'était pas un appui long : il glisse.
            let tremblement = self.tremblement();
            if let Some(a) = self.toucher.appui {
                if (ici.0 - a.depart.0).hypot(ici.1 - a.depart.1) > tremblement {
                    self.toucher.appui = None;
                }
            }
            self.handle_cursor_moved(winit::dpi::PhysicalPosition::new(ici.0, ici.1));
            return;
        }
        let apres = self.toucher.doigts.les_deux();
        if let (Some(avant), Some(apres)) = (avant, apres) {
            if let Some(m) = similitude(avant, apres) {
                self.toucher.attendre(m);
            }
        }
    }

    fn lever_un_doigt(&mut self, id: u64) {
        // Levé avant l'échéance : c'était un toucher.
        self.toucher.appui = None;
        self.plusieurs_au_lever(id);
        let doigts = &mut self.toucher.doigts;
        doigts.poses.retain(|(i, _)| *i != id);
        if doigts.poses.is_empty() {
            let seul = std::mem::take(&mut doigts.seul);
            if seul {
                self.handle_mouse_up(MouseButton::Left);
            }
            self.plusieurs_tous_leves();
        }
    }
}

impl GlucoseApp {
    /// **L'appui long attend son échéance** (APPUI-1) : la boucle se réveille à l'instant où il
    /// prend, et le fait prendre. Sans doigt posé, rien n'attend.
    pub(crate) fn attente_de_l_appui(&mut self) -> Option<u64> {
        let echeance = self.toucher.appui?.echeance;
        let reste = echeance.saturating_duration_since(std::time::Instant::now());
        if reste.is_zero() {
            self.appui_long();
            return None;
        }
        // Arrondi au-dessus : un réveil d'une milliseconde trop tôt en coûterait un second.
        Some(u64::try_from(reste.as_micros().div_ceil(1000)).unwrap_or(u64::MAX))
    }

    /// **L'appui long a pris : c'est le clic droit du doigt** (APPUI-1, fiche 57). Le geste
    /// que le doigt avait commencé se termine sur place — un toucher —, puis :
    ///
    /// * sur une question, rien ne répond ;
    /// * dans le texte qu'on écrit, le mot sous le doigt se sélectionne, comme partout sous
    ///   Android ;
    /// * ailleurs, **ce qui suit décide** (GESTES-1, fiche 58) : lever le doigt ouvre le menu
    ///   du clic droit, à la taille du doigt — celui du nœud que le toucher vient de choisir, ou
    ///   celui du vide ; le glisser trace un rectangle ; un autre doigt ajoute des nœuds.
    pub(crate) fn appui_long(&mut self) {
        if self.toucher.appui.take().is_none() {
            return;
        }
        // Le doigt ne mène plus de geste : se lever ne fera plus rien.
        self.toucher.doigts.seul = false;
        crate::plateforme::doigt::sentir_l_appui();
        if self.appui_long_sur_la_question() {
            return;
        }
        let ici = self.mouse_pos;
        let tenu = self.toucher.doigts.poses.first().map(|(i, _)| *i);
        self.toucher.bref = None;
        let dans_le_texte = self.text_drag.is_some();
        // **Un appui long n'est pas un clic** : le relâchement ne réduit pas une sélection
        // multiple au nœud touché (SEL-MULTI-1) — le menu s'ouvre sur toute la sélection,
        // comme le clic droit au bureau.
        self.reduire_a_la_relache = None;
        self.handle_mouse_up(MouseButton::Left);
        if dans_le_texte {
            self.click_text_at(ici, 2, false);
            self.end_text_drag();
            return;
        }
        if let Some(tenu) = tenu {
            self.toucher.pris = Some(plusieurs::Pris::nouveau(tenu, ici));
        }
    }

    /// Le geste de souris en cours vient-il d'un doigt ? Sur le vide, il déplace alors le
    /// canevas au lieu d'y tracer un cadre.
    pub(crate) fn au_doigt(&self) -> bool {
        self.toucher.doigts.seul
    }
}

#[cfg(test)]
mod tests;
