//! **L'éditeur d'ancres d'une flèche** (FLECHE-4) — son état, et le panneau qui le guide.
//!
//! # Ce que Tauri faisait, et ce qui change
//!
//! `ArrowTextEditor.tsx` ouvrait une fenêtre par-dessus le canevas, y **recopiait** le texte de
//! la carte rendu une seconde fois, et l'on y sélectionnait le passage. Deux rendus d'un même
//! texte : c'est là que les positions divergeaient, et qu'un mot pouvait en désigner un autre.
//!
//! Ici, on sélectionne **dans la carte elle-même**, sur le canevas : la caméra vole jusqu'à
//! elle, et le panneau ne fait que guider — l'étape, ce qui est choisi, les boutons. Il n'y a
//! qu'une mise en page, celle que la carte montre, donc qu'une vérité sur les positions.
//!
//! Le panneau se pose là où se pose la barre d'options, qu'il remplace le temps de l'édition.

use super::action_bar::{
    dans, fond_arrondi, BTN_PAD_X, BTN_PAD_Y, BTN_RADIUS, FONT, GAP, PAD_X, PAD_Y, RADIUS,
};
use crate::theme::Theme;
use crate::typography::{Face, TextStyle, Typography};
use glucose_core::types::TextAnchor;
use tiny_skia::PixmapMut;

/// L'écart entre le panneau et le bas de la fenêtre, comme la barre d'options au-dessus de la
/// barre d'action.
const EN_BAS: f32 = 48.0;

/// Le côté de la flèche qu'on ancre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Etape {
    Source,
    Cible,
}

/// L'édition en cours des ancres d'une flèche.
#[derive(Debug, Clone, PartialEq)]
pub struct Ancrage {
    pub fleche: String,
    pub etape: Etape,
    /// Les cartes de texte de ses deux bouts : `None` d'un côté qui n'en est pas une.
    pub cartes: (Option<String>, Option<String>),
    /// Ce qui est choisi de chaque côté.
    pub source: Vec<TextAnchor>,
    pub cible: Vec<TextAnchor>,
    /// Le glisser en cours : l'octet où il a commencé, celui où il en est, et s'il **ajoute**
    /// au choix (`Ctrl`) au lieu de le remplacer.
    pub glisse: Option<(usize, usize, bool)>,
}

impl Ancrage {
    /// La carte de l'étape en cours.
    pub fn carte(&self) -> Option<&str> {
        match self.etape {
            Etape::Source => self.cartes.0.as_deref(),
            Etape::Cible => self.cartes.1.as_deref(),
        }
    }

    /// Ce qui est choisi à l'étape en cours.
    pub fn ancres(&self) -> &[TextAnchor] {
        match self.etape {
            Etape::Source => &self.source,
            Etape::Cible => &self.cible,
        }
    }

    pub fn ancres_mut(&mut self) -> &mut Vec<TextAnchor> {
        match self.etape {
            Etape::Source => &mut self.source,
            Etape::Cible => &mut self.cible,
        }
    }

    /// Reste-t-il une étape après celle-ci ?
    pub fn a_une_suite(&self) -> bool {
        self.etape == Etape::Source && self.cartes.1.is_some()
    }
}

/// Ce qu'un bouton du panneau demande.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionDAncrage {
    /// Vider le choix de l'étape.
    Effacer,
    /// Passer à la cible — en gardant ce qui est choisi, s'il y a quelque chose.
    Suivant,
    /// Écrire les deux côtés, en un geste.
    Terminer,
    /// Tout laisser comme avant.
    Annuler,
}

/// Le panneau, une fois placé.
#[derive(Debug, Clone, PartialEq)]
pub struct Panneau {
    pub rect: (f32, f32, f32, f32),
    /// « SOURCE » ou « CIBLE », et la consigne.
    pub titre: (&'static str, f32),
    pub consigne: (&'static str, f32),
    pub boutons: Vec<BoutonDAncrage>,
}

/// Un bouton du panneau : ce qu'il demande, ce qu'il écrit, où il est.
#[derive(Debug, Clone, PartialEq)]
pub struct BoutonDAncrage {
    pub action: ActionDAncrage,
    pub libelle: &'static str,
    pub rect: (f32, f32, f32, f32),
}

const CONSIGNE: &str = "Sélectionnez le texte exact · Ctrl pour en ajouter";

/// Le panneau de l'édition en cours. Fonction pure : le dessin et le clic lisent la même
/// mise en page (loi L4).
pub fn layout_ancrage(
    ancrage: &Ancrage,
    typographie: &Typography,
    ecran: (f32, f32),
    scale: f32,
) -> Panneau {
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    let mesure = |t: &str| typographie.measure_text(t, font, Face::Regular).0;
    let titre = match ancrage.etape {
        Etape::Source => "SOURCE",
        Etape::Cible => "CIBLE",
    };
    let mut boutons: Vec<(ActionDAncrage, &'static str)> = Vec::new();
    if !ancrage.ancres().is_empty() {
        boutons.push((ActionDAncrage::Effacer, "Effacer"));
    }
    if ancrage.a_une_suite() {
        let libelle = if ancrage.ancres().is_empty() {
            "Passer →"
        } else {
            "Valider → Cible"
        };
        boutons.push((ActionDAncrage::Suivant, libelle));
    } else {
        boutons.push((ActionDAncrage::Terminer, "Terminer"));
    }
    boutons.push((ActionDAncrage::Annuler, "Annuler"));

    let hauteur_btn = font + BTN_PAD_Y * s * 2.0;
    let hauteur = hauteur_btn + PAD_Y * s * 2.0;
    let largeur_btn = |t: &str| mesure(t) + BTN_PAD_X * s * 2.0;
    let largeur = PAD_X * s * 2.0
        + mesure(titre)
        + GAP * s * 2.0
        + mesure(CONSIGNE)
        + boutons
            .iter()
            .map(|(_, t)| GAP * s + largeur_btn(t))
            .sum::<f32>()
        + GAP * s;
    let (x, y) = ((ecran.0 - largeur) / 2.0, ecran.1 - EN_BAS * s - hauteur);
    let mut curseur = x + PAD_X * s;
    let titre = (titre, curseur);
    curseur += mesure(titre.0) + GAP * s * 2.0;
    let consigne = (CONSIGNE, curseur);
    curseur += mesure(CONSIGNE) + GAP * s;
    let boutons = boutons
        .into_iter()
        .map(|(action, libelle)| {
            curseur += GAP * s;
            let w = largeur_btn(libelle);
            let rect = (curseur, y + PAD_Y * s, w, hauteur_btn);
            curseur += w;
            BoutonDAncrage {
                action,
                libelle,
                rect,
            }
        })
        .collect();
    Panneau {
        rect: (x, y, largeur, hauteur),
        titre,
        consigne,
        boutons,
    }
}

/// Ce qu'un clic en `(px, py)` demande au panneau.
pub fn action_sous(panneau: &Panneau, px: f32, py: f32) -> Option<ActionDAncrage> {
    panneau
        .boutons
        .iter()
        .find(|b| dans(b.rect, px, py))
        .map(|b| b.action)
}

/// Le clic tombe-t-il sur le panneau ?
pub fn couvre(panneau: &Panneau, px: f32, py: f32) -> bool {
    dans(panneau.rect, px, py)
}

/// Dessine le panneau de l'édition en cours.
pub fn draw_ancrage(
    pixmap: &mut PixmapMut,
    ancrage: &Ancrage,
    typographie: &Typography,
    theme: &Theme,
    (ecran, scale): ((f32, f32), f32),
) {
    let p = layout_ancrage(ancrage, typographie, ecran, scale);
    let s = crate::theme::clamp_ui_scale(scale);
    let font = FONT * s;
    fond_arrondi(
        pixmap,
        p.rect,
        RADIUS * s,
        theme.btn_bg,
        Some(theme.border_medium),
    );
    let y = p.rect.1 + (p.rect.3 - font) / 2.0;
    let style = |color, face| TextStyle {
        size: font,
        color,
        face,
    };
    typographie.draw_text(
        pixmap,
        p.titre.0,
        p.titre.1,
        y,
        style(theme.text_primary, Face::Bold),
    );
    typographie.draw_text(
        pixmap,
        p.consigne.0,
        p.consigne.1,
        y,
        style(theme.text_muted, Face::Regular),
    );
    for BoutonDAncrage {
        action,
        libelle,
        rect,
    } in &p.boutons
    {
        let principal = matches!(action, ActionDAncrage::Suivant | ActionDAncrage::Terminer);
        if principal {
            fond_arrondi(
                pixmap,
                *rect,
                BTN_RADIUS * s,
                theme.bg_active,
                Some(theme.border_medium),
            );
        }
        let encre = if principal {
            theme.text_primary
        } else {
            theme.text_muted
        };
        typographie.draw_text(
            pixmap,
            libelle,
            rect.0 + BTN_PAD_X * s,
            rect.1 + (rect.3 - font) / 2.0,
            style(encre, Face::Regular),
        );
    }
}

#[cfg(test)]
mod tests;
