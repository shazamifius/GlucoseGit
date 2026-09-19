//! Le vol de caméra : aller quelque part sans y être téléporté.
//!
//! # Ce que la téléportation coûte, et pourquoi ce module existe
//!
//! Poser la vue ailleurs d'une image à l'autre fait perdre le fil : rien ne dit d'où l'on
//! vient, et l'œil doit reconstruire la carte entière. Un déplacement continu, lui, se suit —
//! c'est la même information, rendue lisible par le mouvement.
//!
//! L'utilisateur l'a dit de la minimap : « il faut cliquer et ça nous téléporte instantanément
//! là où on a cliqué, or sur Glucose tu peux maintenir directement la minimap et tu voyages
//! comme ça, en plus d'avoir un smooth ».
//!
//! # Deux mouvements indépendants, parce que ce sont deux grandeurs différentes
//!
//! Un vol n'interpole **pas** les trois nombres du cadrage. Il interpole :
//!
//! * le **point du monde au centre de l'écran** — un déplacement, en unités du monde ;
//! * l'**altitude**, c'est-à-dire l'échelle **en octaves** — une grandeur multiplicative.
//!
//! Interpoler l'échelle elle-même donnerait un vol qui se précipite puis rampe : passer de ×1
//! à ×20 en ligne droite passe la moitié du trajet au-dessus de ×10. En octaves, chaque
//! fraction du trajet double ou dédouble d'autant — le mouvement est uniforme **à l'œil**,
//! qui perçoit les rapports et non les différences.
//!
//! C'est la même unité que la molette et le pincement ([`super::pan_zoom`]), pour la même
//! raison. Le zoom est multiplicatif partout dans Glucose, donc il s'additionne partout.
//!
//! # Il n'y a pas de point d'origine, et c'est le fond de l'affaire
//!
//! La touche `F` remettait le cadrage à sa valeur par défaut — l'origine, à l'échelle un.
//! Dans un canva infini cette origine **n'existe pas** : ce n'est le centre de rien, et rien
//! ne garantit qu'il reste quoi que ce soit à y voir. Ce module ne connaît que des
//! destinations calculées à partir du contenu.

use glucose_core::geometry::Rect;
use glucose_core::membrane_focus::{fit_viewport, focus_consts, ScreenSize};
use glucose_core::types::Viewport;

/// Constante de temps du vol, en secondes.
///
/// Ce qu'elle vaut se lit ainsi : après `TAU`, il reste 37 % du chemin ; après trois `TAU`,
/// 5 %. Un vol se **voit** donc durer à peu près une demi-seconde, sans jamais s'arrêter net.
///
/// Elle est plus courte que l'amortissement d'un glissement ([`super::elan`]) parce qu'elle
/// répond à une demande **ponctuelle** et non à un geste continu : on a dit où aller, et on
/// veut y être. Traîner davantage donnerait l'impression que le logiciel hésite.
const TAU: f64 = 0.18;

/// Un vol en cours, ou rien.
///
/// L'état tient en une destination. Il n'y a ni vitesse, ni durée écoulée, ni point de
/// départ mémorisé : à chaque image on regarde où l'on est et où l'on va, et on comble une
/// fraction de l'écart. Redéfinir la destination en plein vol est donc gratuit et sans
/// discontinuité — c'est exactement ce que demande une minimap qu'on maintient.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vol {
    cible: Option<Viewport>,
}

impl Vol {
    /// Décide d'aller là. Remplace une destination précédente sans secousse.
    pub fn viser(&mut self, cible: Viewport) {
        self.cible = Some(cible);
    }

    /// Abandonne le vol en cours : la vue reste où elle est.
    ///
    /// Appelé dès que la main reprend la navigation — un vol est une intention passée, et
    /// elle ne doit jamais discuter avec le geste présent.
    pub fn poser(&mut self) {
        self.cible = None;
    }

    pub fn en_cours(&self) -> bool {
        self.cible.is_some()
    }

    /// Rapproche `vue` de la destination du temps qui s'est écoulé.
    ///
    /// Rend le nouveau cadrage, ou `None` s'il n'y a rien à faire. La fraction comblée est
    /// `1 − e^(−dt/τ)`, l'intégrale **exacte** de l'amortissement sur la durée de l'image :
    /// le trajet ne dépend donc pas de la cadence, et une image longue rattrape d'elle-même
    /// ce que trois images courtes auraient fait.
    pub fn avancer(&mut self, vue: Viewport, ecran: ScreenSize, dt: f64) -> Option<Viewport> {
        let cible = self.cible?;
        // Moins d'un demi-pixel d'écart où que ce soit : on **pose** exactement la cible,
        // plutôt que de s'en approcher indéfiniment sans jamais l'atteindre.
        if ecart_max_en_pixels(vue, cible, ecran) < 0.5 {
            self.cible = None;
            return Some(cible);
        }
        let part = 1.0 - (-dt / TAU).exp();
        let (depart, altitude_depart) = decomposer(vue, ecran);
        let (arrivee, altitude_arrivee) = decomposer(cible, ecran);
        let centre = (
            depart.0 + (arrivee.0 - depart.0) * part,
            depart.1 + (arrivee.1 - depart.1) * part,
        );
        let altitude = altitude_depart + (altitude_arrivee - altitude_depart) * part;
        Some(composer(centre, altitude, ecran))
    }
}

/// Le point du monde au centre de l'écran, et l'altitude en octaves.
fn decomposer(vue: Viewport, ecran: ScreenSize) -> ((f64, f64), f64) {
    let centre = (
        (ecran.width / 2.0 - vue.x) / vue.scale,
        (ecran.height / 2.0 - vue.y) / vue.scale,
    );
    (centre, vue.scale.log2())
}

/// L'opération inverse de [`decomposer`].
fn composer(centre: (f64, f64), altitude: f64, ecran: ScreenSize) -> Viewport {
    let scale = altitude.exp2();
    Viewport {
        scale,
        x: ecran.width / 2.0 - centre.0 * scale,
        y: ecran.height / 2.0 - centre.1 * scale,
    }
}

/// De combien de pixels un point de l'écran bouge, au pire, en passant de `a` à `b`.
///
/// # Pourquoi les quatre coins suffisent, et pourquoi c'est exact
///
/// Le déplacement d'un point du monde `p` vaut `p·(s_b − s_a) + (t_b − t_a)` : il est
/// **affine** en `p`. Une fonction affine sur un rectangle atteint ses extrêmes à un sommet —
/// examiner les quatre coins donne donc le maximum exact, sans échantillonner ni majorer.
///
/// C'est ce qui permet un critère d'arrêt en pixels, la seule unité dans laquelle « c'est
/// arrivé » veut dire quelque chose : un dixième d'octave ne se voit pas de la même façon
/// selon qu'on regarde une vignette ou un mur d'images.
fn ecart_max_en_pixels(a: Viewport, b: Viewport, ecran: ScreenSize) -> f64 {
    let coins = [
        (0.0, 0.0),
        (ecran.width, 0.0),
        (0.0, ecran.height),
        (ecran.width, ecran.height),
    ];
    coins
        .into_iter()
        .map(|(sx, sy)| {
            // Le coin, exprimé dans le monde tel que `a` le voit — puis regardé par `b`.
            let monde = ((sx - a.x) / a.scale, (sy - a.y) / a.scale);
            let vu_par_b = (monde.0 * b.scale + b.x, monde.1 * b.scale + b.y);
            (vu_par_b.0 - sx).hypot(vu_par_b.1 - sy)
        })
        .fold(0.0, f64::max)
}

/// Le cadrage qui montre **tout le contenu**, sous le bandeau, avec sa marge.
///
/// # Ce que cette fonction remplace
///
/// La touche `F` posait [`Viewport::default`] : l'origine, à l'échelle un. Dans un canva
/// infini, cette origine n'est le centre de rien — rien ne garantit qu'il y ait quoi que ce
/// soit à y voir, et s'y retrouver après coup demande autant de travail qu'avant. Cadrer sur
/// le contenu, en revanche, répond toujours à la question qu'on se pose en appuyant : « où
/// sont mes affaires ? ».
///
/// # Trois soins, et chacun répare une façon de rater le cadrage
///
/// * la hauteur utile **exclut le bandeau**, et le résultat est décalé d'autant : sinon le
///   contenu se centre sur la fenêtre, donc trop haut, et sa première ligne passe dessous ;
/// * la marge est [`focus_consts::FIT_PADDING`], celle du mode focus — le contenu ne colle
///   pas aux bords, et aucune constante nouvelle n'apparaît ici ;
/// * l'échelle est bornée par [`Viewport::SCALE_RANGE`], puis le centrage est **refait** à
///   partir d'elle. Un seul nœud dans un tableau vide demanderait sinon un grossissement que
///   le modèle refuse, et le cadrage se retrouverait à côté de sa propre cible.
pub fn cadrage_du_contenu(contenu: Rect, ecran: ScreenSize, bandeau: f64) -> Viewport {
    let utile = ScreenSize {
        width: ecran.width.max(1.0),
        height: (ecran.height - bandeau).max(1.0),
    };
    let (min, max) = Viewport::SCALE_RANGE;
    let scale = fit_viewport(contenu, utile, focus_consts::FIT_PADDING)
        .scale
        .clamp(min, max);
    let centre = (
        contenu.left + contenu.width / 2.0,
        contenu.top + contenu.height / 2.0,
    );
    Viewport {
        scale,
        x: utile.width / 2.0 - centre.0 * scale,
        y: bandeau + utile.height / 2.0 - centre.1 * scale,
    }
}

impl crate::app::GlucoseApp {
    /// Cadre la vue sur tout le contenu du tableau — la touche `F`.
    ///
    /// Un vol, pas un saut : d'où l'on revient fait partie de ce qu'on apprend en appuyant.
    pub fn cadrer_sur_le_contenu(&mut self, ecran: ScreenSize, bandeau: f64) {
        let Some(tableau) = self.store.active_board() else {
            return;
        };
        let Some(contenu) = self.store.content_bounds(&tableau.id.clone()) else {
            // Rien à cadrer, et surtout pas l'origine : dans un canva infini elle ne vaut pas
            // mieux que l'endroit où l'on est déjà.
            self.ui.show_toast("Rien a cadrer : ce tableau est vide");
            return;
        };
        self.vol.viser(cadrage_du_contenu(contenu, ecran, bandeau));
        self.mark_dirty();
    }

    /// Vise le point du monde que la minimap désigne, en gardant l'échelle courante.
    ///
    /// Appelé à l'appui **et à chaque mouvement tant que le bouton tient** : la destination
    /// suit le curseur, et le vol la poursuit. C'est le voyage continu que l'utilisateur
    /// décrit, au lieu d'une téléportation par clic.
    pub fn viser_par_la_minimap(&mut self, monde: (f64, f64), ecran: ScreenSize, bandeau: f64) {
        let Some(tableau) = self.store.active_board() else {
            return;
        };
        let scale = tableau.viewport.scale;
        let utile = (ecran.height - bandeau).max(1.0);
        self.vol.viser(Viewport {
            scale,
            x: ecran.width / 2.0 - monde.0 * scale,
            y: bandeau + utile / 2.0 - monde.1 * scale,
        });
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
