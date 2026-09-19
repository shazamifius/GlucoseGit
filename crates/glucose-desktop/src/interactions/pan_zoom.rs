//! Interaction de navigation caméra : molette, pavé tactile, pan à la souris.
//!
//! # NAV-2 — un événement de défilement dit **zoom** ou **pan**, jamais les deux
//!
//! La règle est celle de Glucose Tauri (`src/canvas/navigation.ts`), et elle vaut d'être
//! reprise telle quelle : c'est la seule qui rende un pavé tactile utilisable.
//!
//! * pincement — que le pilote livre en `Ctrl` + défilement — : **zoom** ;
//! * cran de molette de souris : **zoom** ;
//! * tout le reste, c'est-à-dire un glissement à deux doigts : **pan**.
//!
//! # Reconnaître un cran de souris sans rien inventer
//!
//! Tauri distingue les deux en testant `wheelDeltaY % 120 == 0` : sous Windows, une souris
//! rend exactement 120 unités par cran, un pavé tactile rend ce que vaut le doigt. Winit
//! livre la **même** grandeur déjà divisée par 120 — donc « multiple de 120 » s'écrit ici
//! « nombre entier ». Aucune constante nouvelle : la même information, mieux dite.
//!
//! Le piège que cela corrige : l'ancienne version décidait sur le **type** winit, `LineDelta`
//! contre `PixelDelta`. Or Windows livre tout en `LineDelta`, pavé tactile compris — alors
//! deux doigts vers le bas zoomaient, et le pan vertical au pavé tactile était littéralement
//! inatteignable.

use crate::app::GlucoseApp;
use winit::event::MouseScrollDelta;

/// Bornes du zoom **au geste** — molette et pincement (fiche 07 § 7.1) : de ×50 dézoomé à
/// ×20 zoomé. Plus étroites que celles du modèle ([`glucose_core::types::Viewport::SCALE_RANGE`]),
/// qu'un signet ou un fichier peuvent atteindre sans que la main y arrive.
pub const WHEEL_SCALE_RANGE: (f64, f64) = (0.02, 20.0);

/// Ce qu'un cran de molette de souris change d'échelle, en **octaves**.
///
/// # Pourquoi l'octave, et pas un facteur
///
/// L'échelle est multiplicative : la dire en octaves, c'est la dire dans son unité naturelle,
/// où l'addition a un sens. Huit crans doublent la taille apparente, huit crans en arrière la
/// divisent par deux, et le geste est exactement réversible — ce que `0.999^(-delta·40)` ne
/// laissait ni lire ni vérifier.
///
/// La valeur d'avant valait 0,058 octave par cran : dix-sept crans pour doubler. C'est ce que
/// l'utilisateur a décrit comme « la sensibilité est un peu faible ».
const OCTAVES_PAR_CRAN: f64 = 0.125;

/// Ce qu'une unité de défilement change d'échelle quand elle vient d'un **doigt**.
///
/// # Deux gestes physiques, deux gains — et la touche n'en fait pas partie
///
/// Un cran de molette est un **déclic** : quantifié, discret, une secousse par encoche. Une
/// course de doigt est **continue** : le pavé en envoie des dizaines d'unités par seconde
/// tant que le doigt bouge. Windows les encode dans la même grandeur numérique, mais leur
/// appliquer le même gain est faux par construction — quarante unités de doigt valaient
/// quarante crans de molette, c'est-à-dire cinq octaves en un geste.
///
/// Le code d'avant ne séparait que le **pincement**. `Ctrl` + glissement à deux doigts, lui,
/// tombait dans la branche du cran de souris et zoomait donc deux fois trop vite : ce sont
/// pourtant les mêmes doigts sur le même pavé, et seule la touche changeait. C'est corrigé
/// ici — **la nature du geste décide du gain, la touche ne décide que du sens.**
///
/// # Ce que vaut ce chiffre, et d'où il vient
///
/// Le journal de ce réglage, pour que le prochain ajustement parte de ce qui a été essayé
/// plutôt que du vide :
///
/// | valeur | verdict de l'utilisateur |
/// |---|---|
/// | 0,058 (avant l'élan) | « la sensibilité est un peu faible » |
/// | 1,25 | « beaucoup beaucoup trop » |
/// | 0,25 | « ça dézoome et ça zoome trop trop vite » |
/// | **0,0625** | à juger |
///
/// Un huitième d'octave par **deux** unités de doigt : il en faut seize pour doubler. Et il
/// faut compter l'élan par-dessus, qui prolonge le geste d'à peu près aussi longtemps qu'il a
/// duré, donc **double l'amplitude ressentie** — l'effet à la main est celui d'un huitième
/// d'octave par unité, exactement ce que l'utilisateur jugeait « un peu faible » avant que
/// l'élan n'existe.
const OCTAVES_PAR_UNITE_DE_DOIGT: f64 = 0.0625;

/// Un cran de molette, en pixels de défilement, là où la plateforme compte en pixels.
///
/// Windows livre tout en crans ; macOS livre tout en pixels. Cette conversion n'existe que
/// pour ramener les seconds aux premiers, et vaut ce qu'elle valait avant.
const ZOOM_LIGNE_PX: f64 = 40.0;

/// Une ligne de défilement, en pixels, pour un **pan**.
///
/// Tauri emploie 40 pour le zoom et 16 pour le pan — deux valeurs pour une seule conversion.
/// C'est une incohérence de la référence, et elle est reprise **délibérément** : c'est cette
/// sensation-là qui est validée à la main. Les unifier est une expérience à mener au doigt,
/// pas une correction à faire sur le papier.
const PAN_LIGNE_PX: f64 = 16.0;

/// Ce qu'un événement de défilement demande à la caméra.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Geste {
    /// Changer l'échelle de tant d'**octaves**, autour du curseur : `+1` double.
    Zoom(f64),
    /// Déplacer le contenu de tant de pixels écran.
    Pan(f64, f64),
}

/// NAV-2 — ce qu'un événement de défilement veut dire.
///
/// Fonction pure : c'est elle qui porte toute la décision, et elle se teste sans fenêtre.
pub fn geste(delta: MouseScrollDelta, ctrl: bool, pincement: bool) -> Geste {
    let (dx, dy, ligne) = deltas(delta);
    // Un pincement est marqué **par le système** : c'est donc certainement un doigt, et le
    // test d'entier — qu'un doigt satisfait parfois par accident — n'a plus à trancher.
    let cran = !pincement && cran_de_souris(dx, dy, ligne);
    if ctrl || pincement || cran {
        let unites = if ligne { dy } else { dy / ZOOM_LIGNE_PX };
        let par_unite = if cran {
            OCTAVES_PAR_CRAN
        } else {
            OCTAVES_PAR_UNITE_DE_DOIGT
        };
        return Geste::Zoom(unites * par_unite);
    }
    let px = if ligne { PAN_LIGNE_PX } else { 1.0 };
    Geste::Pan(dx * px, dy * px)
}

/// Les deux composantes d'un défilement, et s'il s'exprime en lignes.
fn deltas(delta: MouseScrollDelta) -> (f64, f64, bool) {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => (f64::from(x), f64::from(y), true),
        MouseScrollDelta::PixelDelta(p) => (p.x, p.y, false),
    }
}

/// **Pourquoi** ce geste a été décidé — pour la trace, jamais pour le comportement.
///
/// La décision elle-même reste dans [`geste`], à un seul endroit. Celle-ci n'en lit que la
/// raison, parce qu'un défilement pris pour un cran de souris est le cas ambigu : s'il abonde
/// pendant qu'on glisse à deux doigts, chacun coûte un saut d'échelle visible — et aucune
/// mesure de durée ne le montrerait.
pub fn pourquoi(
    delta: MouseScrollDelta,
    ctrl: bool,
    pincement: bool,
) -> crate::chronique::navigation::Decision {
    use crate::chronique::navigation::Decision;
    if pincement {
        return Decision::Pincement;
    }
    let (dx, dy, ligne) = deltas(delta);
    if !ctrl && cran_de_souris(dx, dy, ligne) {
        return Decision::CranDeSouris;
    }
    match geste(delta, ctrl, pincement) {
        Geste::Zoom(_) => Decision::Zoom,
        Geste::Pan(..) => Decision::Pan,
    }
}

/// Un cran de molette de souris : vertical pur, et d'un nombre **entier** de lignes.
///
/// Un pavé tactile ne remplit ces deux conditions ensemble que par accident, et l'accident
/// coûte une image de zoom au milieu d'un pan — invisible. L'inverse, prendre une souris pour
/// un pavé, coûterait tout le zoom à la molette.
fn cran_de_souris(dx: f64, dy: f64, ligne: bool) -> bool {
    ligne && dx == 0.0 && dy != 0.0 && dy.fract() == 0.0
}

impl GlucoseApp {
    /// Gère les événements de molette et gestes tactiles.
    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        // Le `Ctrl` d'un pincement est virtuel : il vit dans le message du systeme, pas dans
        // l'etat du clavier que `winit` rapporte. Les deux disent la meme chose -- « ce
        // defilement veut zoomer » -- et se lisent donc ici, et nulle part ailleurs, pour que
        // la decision elle-meme reste une fonction pure.
        let pincement = super::pincement::zoom_du_systeme();
        let ctrl = self.modifiers.control_key();
        // NAV-3 : ce que le doigt a demande entre dans la trace, avec l'instant ou il l'a
        // demande. C'est de la qu'on saura si l'ecran suit la main.
        self.chronique
            .navigation
            .evenement(pourquoi(delta, ctrl, pincement));
        // L'evenement ne bouge PLUS la camera : il pousse dans l'elan, que l'image videra en
        // une seule fois. Windows livre l'horizontal et le vertical dans deux messages
        // separes -- les appliquer chacun a leur tour faisait d'une diagonale un escalier.
        match geste(delta, ctrl, pincement) {
            Geste::Zoom(octaves) => self.elan.pousser_zoom(octaves, self.ancre_du_zoom()),
            Geste::Pan(dx, dy) => self.elan.pousser_pan(dx, dy),
        }
        self.mark_dirty();
    }

    /// Le point d'écran autour duquel le zoom tourne.
    ///
    /// Le curseur dès qu'il a été posé — c'est la règle de Glucose Tauri, et un pincement ne
    /// déplace pas le curseur, donc le point reste celui qu'on vise. Tant qu'il ne l'a pas
    /// été, le centre de la fenêtre : zoomer vers un coin qu'on n'a pas choisi donne
    /// exactement l'impression d'un « point d'origine » qui aspire la vue.
    fn ancre_du_zoom(&self) -> (f64, f64) {
        if self.curseur_vu {
            return self.mouse_pos;
        }
        let Some(fenetre) = &self.window else {
            return self.mouse_pos;
        };
        let taille = fenetre.inner_size();
        (
            f64::from(taille.width) / 2.0,
            f64::from(taille.height) / 2.0,
        )
    }

    /// Déplacement relatif de la caméra lors d'un pan souris (bouton milieu, droit, ou l'outil
    /// Pan où la barre d'espace fait entrer).
    pub fn handle_pan_move(&mut self, dx: f64, dy: f64) {
        // Protection contre les sauts anormaux du curseur OS
        if dx.hypot(dy) < 300.0 {
            // Par l'elan, comme la molette : deux mouvements de curseur arrives entre deux
            // images se rejoignent, et lacher le bouton en plein geste laisse la vue filer
            // au lieu de s'arreter net.
            self.elan.pousser_pan(dx, dy);
        }
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
