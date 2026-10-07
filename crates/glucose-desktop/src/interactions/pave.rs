//! **Le pavé tactile tel que le doigt le fait** (fiches 53 et 54) : le pincement et le
//! déplacement à deux doigts, reçus par *Direct Manipulation* et non plus traduits en molette.
//!
//! # Ce qui n'allait pas, et que trois réglages n'ont pas réparé
//!
//! Une application qui ne prend pas le pavé de précision le reçoit sous la forme que Windows
//! donne à tout le monde : des défilements de molette, `Ctrl` en plus pour un pincement
//! ([`super::pincement`]). Ces paquets arrivent au rythme que Windows choisit, quantifiés, et ont
//! déjà perdu l'échelle du geste. Le 07/10, avec la conduite et la glissade il était « trop
//! smooth » ; tout direct, « strate par strate » ; conduit sans glissade, « légèrement trop
//! lent, et pas fluide, comme s'il sautait ». Aucun gain ni aucun lissage ne rend une continuité
//! perdue en route — alors que dans PureRef ou un navigateur, le pincement est « instantané, de
//! manière fluide ».
//!
//! # Ce que font Chromium, Blender et Flutter
//!
//! Ils prennent le pavé par *Direct Manipulation* (`direct_manipulation_helper_win.cc`,
//! `GHOST_TrackpadWin32.cc`, `direct_manipulation.cc` de Flutter) : un *viewport* fictif, que le
//! système fait glisser et grandir comme le doigt le fait, avec son point de départ, sa cadence
//! et son inertie. L'application lit sa transformation à chaque image.
//!
//! # La transformation entière, et rien d'autre (fiche 54)
//!
//! La première lecture (fiche 53) imitait Chromium et Blender : un geste dont l'échelle bougeait
//! devenait un pincement **jusqu'à la fin**, et son déplacement était jeté. Sa session du 07/10
//! au soir l'a démentie : dix-neuf déplacements pris pour des pincements, le plus petit sur un
//! écart d'échelle de 0,13 % — un biais des doigts —, et le déplacement bloqué jusqu'à ce qu'ils
//! se lèvent : « la multi-direction pose problème ».
//!
//! Il n'y a rien à classer. D'une image à la suivante, le système a fait subir au contenu une
//! **similitude** : chaque point `q` de la fenêtre va en `r · q + b`. C'est exactement ce que les
//! doigts ont fait — l'échelle et le déplacement ensemble, autour du point que le système a
//! choisi —, et c'est ce que Glucose applique ([`Mouvement`]). Un écart d'échelle de 0,13 % est
//! un zoom invisible, et le déplacement continue ; un pincement pur garde fixe le point du
//! système. Le « décalage absurde » de Blender pendant un pincement est le déplacement qui garde
//! ce point fixe : compté **en plus** d'un zoom autour du curseur, il l'était deux fois ; dans la
//! similitude, il n'est compté qu'une fois. Plus de règle, plus de seuil, plus de bascule.
//!
//! Ce que Glucose en fait passe par la porte de la souris ([`super::elan::Elan::placer_pan`]) :
//! montré en entier à l'image suivante. Le lissage et l'inertie sont ceux du système — ceux
//! d'Edge et de Chrome —, et le déplacement à deux doigts garde son élan parce que le système le
//! lui donne. Le pincement n'en a pas : il s'arrête avec les doigts.

use crate::app::GlucoseApp;
use std::time::Instant;

/// **Ce que les doigts ont fait depuis l'image précédente** : chaque point `q` de la fenêtre va
/// en `echelle · q + decalage`, en pixels de la fenêtre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mouvement {
    pub echelle: f64,
    pub decalage: (f64, f64),
}

impl Mouvement {
    /// **La même similitude, dite comme Glucose l'applique** : un déplacement `p`, puis un zoom de
    /// tant d'octaves autour de `ancre` ([`crate::app::GlucoseApp::appliquer_l_elan`] fait dans
    /// cet ordre). De `r · (q + p − a) + a = r · q + b` : `p = (b − (1 − r) · a) / r`.
    pub fn autour_de(&self, (ax, ay): (f64, f64)) -> ((f64, f64), f64) {
        let r = self.echelle;
        let (bx, by) = self.decalage;
        let p = ((bx - (1.0 - r) * ax) / r, (by - (1.0 - r) * ay) / r);
        (p, r.log2())
    }

    /// **Le point que le système a gardé fixe** : `b / (1 − r)` — aucun quand l'échelle n'a pas
    /// bougé. Une mesure (la chronique le compare au curseur) : rien ne s'en sert pour bouger.
    pub fn point_fixe(&self) -> Option<(f64, f64)> {
        let k = 1.0 - self.echelle;
        (k != 0.0).then(|| (self.decalage.0 / k, self.decalage.1 / k))
    }

    /// **Le zoom l'emporte-t-il à l'écran ?** — pour compter, et seulement pour compter : un coin
    /// de la fenêtre bouge de `diagonale / 2 · |r − 1|` sous le zoom, et de `|p|` sous le
    /// déplacement. Le plus grand des deux est ce que l'œil a vu.
    pub fn zoom_domine(&self, deplacement: (f64, f64), diagonale: f64) -> bool {
        diagonale / 2.0 * (self.echelle - 1.0).abs() > deplacement.0.hypot(deplacement.1)
    }
}

/// **Un signe de vie de la voie du pavé**, pour la boîte noire : un contact confié au système,
/// un changement d'état, un geste qui commence ou finit. Sa coupure de 7 à 10 secondes, le
/// 07/10, ne laissait aucune trace : ces signes disent à quel étage elle s'arrête (fiche 54).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signe {
    pub quand: Instant,
    /// Un nom fixé à la compilation : la boîte noire n'écrit jamais rien d'autre.
    pub quoi: &'static str,
    /// Ce qui le précise : l'état d'avant pour un changement d'état, le code d'un refus.
    pub valeur: u64,
}

/// **Tous les noms qu'un [`Signe`] peut porter** — le serveur de la boîte noire n'accepte que
/// ceux-là (`outils/telemetrie/src/valider.js`), et une épreuve tient les deux listes égales.
pub const NOMS_DES_SIGNES: [&str; 12] = [
    "contact",
    "cadre_refuse",
    "interaction_debut",
    "interaction_fin",
    "statut_en_construction",
    "statut_actif",
    "statut_desactive",
    "statut_en_cours",
    "statut_inertie",
    "statut_pret",
    "statut_suspendu",
    "statut_inconnu",
];

/// **Une source de gestes du pavé** : la voie de Windows, ou celle qu'une épreuve fournit.
pub trait Pave {
    /// Un geste est-il en cours — les doigts posés, ou l'inertie qu'ils ont laissée ? C'est la
    /// seule raison de faire avancer le système : hors geste, Glucose dort.
    fn en_geste(&self) -> bool;
    /// Fait avancer le système d'une image, et rend ce qui a bougé — une similitude, deux au
    /// plus quand le geste vient de finir.
    fn avancer(&mut self) -> Vec<Mouvement>;
    /// La fenêtre a changé de taille : le *viewport* fictif la suit.
    fn cadrer(&mut self, _largeur: u32, _hauteur: u32) {}
    /// Les signes de vie reçus depuis le dernier appel.
    fn signes(&mut self) -> Vec<Signe> {
        Vec::new()
    }
}

/// **La lecture de la transformation** que le système applique à son contenu, depuis le début
/// du geste : `(échelle, décalage x, décalage y)` — un point `c` du contenu est montré en
/// `échelle · c + décalage`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lecteur {
    dernier: (f32, f32, f32),
}

/// La transformation qui ne bouge rien : celle d'un geste qui commence.
pub const IDENTITE: (f32, f32, f32) = (1.0, 0.0, 0.0);

impl Default for Lecteur {
    fn default() -> Self {
        Self { dernier: IDENTITE }
    }
}

impl Lecteur {
    /// Le geste est fini et le *viewport* revenu à l'identité : la lecture repart de zéro.
    pub fn remettre(&mut self) {
        *self = Self::default();
    }

    /// **Ce qui a changé** depuis la dernière transformation lue. De `v₁ = s₁ · c + t₁` et
    /// `v₂ = s₂ · c + t₂` : `v₂ = r · v₁ + b`, avec `r = s₂ / s₁` et `b = t₂ − r · t₁` — en `f64`,
    /// pour que les échelles d'un geste se composent exactement en la sienne.
    pub fn lire(&mut self, transformation: (f32, f32, f32)) -> Option<Mouvement> {
        let (s1, x1, y1) = std::mem::replace(&mut self.dernier, transformation);
        let (s2, x2, y2) = transformation;
        // Une échelle nulle ou négative n'est pas une transformation que le système donne.
        if transformation == (s1, x1, y1) || s1 <= 0.0 || s2 <= 0.0 {
            return None;
        }
        let r = f64::from(s2) / f64::from(s1);
        Some(Mouvement {
            echelle: r,
            decalage: (
                f64::from(x2) - r * f64::from(x1),
                f64::from(y2) - r * f64::from(y1),
            ),
        })
    }
}

impl GlucoseApp {
    /// **À chaque image, juste avant que la caméra bouge** ([`GlucoseApp::bouger_la_camera`]) :
    /// si un geste du pavé est en cours, le système avance ; ce qu'il rend rejoint ce que les
    /// doigts ont fait depuis l'image précédente, et le tout se montre dans cette image même, en
    /// **une** similitude ([`crate::interactions::toucher::Toucher::attente`]). Les signes de vie,
    /// eux, partent à chaque image dans la boîte noire.
    pub(crate) fn suivre_le_pave(&mut self) {
        if let Some(pave) = self.toucher.pave.as_mut() {
            let signes = pave.signes();
            let mouvements = if pave.en_geste() {
                pave.avancer()
            } else {
                Vec::new()
            };
            for signe in signes {
                self.chronique.signe_du_pave(signe);
            }
            for mouvement in mouvements {
                self.toucher.attendre(mouvement);
            }
        }
        if let Some(mouvement) = self.toucher.attente.take() {
            self.appliquer_le_pave(mouvement);
            self.mark_dirty();
        }
    }

    /// Les signes de vie qui attendent encore : à la fermeture, avant que la boîte noire se
    /// close — sinon ceux d'une coupure suivie d'aucune image se perdraient.
    pub(crate) fn vider_les_signes_du_pave(&mut self) {
        let signes = self
            .toucher
            .pave
            .as_mut()
            .map(|p| p.signes())
            .unwrap_or_default();
        for signe in signes {
            self.chronique.signe_du_pave(signe);
        }
    }

    /// Une similitude du pavé, montrée en entier à l'image suivante : le système l'a déjà lissée.
    pub(crate) fn appliquer_le_pave(&mut self, mouvement: Mouvement) {
        use crate::chronique::navigation::Decision;
        self.vol.poser();
        self.defilement_au_doigt = true;
        // Les doigts du pavé ne passent pas par `winit` : sans cette ligne, la chronique comptait
        // un geste au pavé comme du repos, et le processeur « au repos » s'en trouvait accusé.
        self.provenance.noter_la_main();
        let ancre = self.ancre_du_zoom();
        let (deplacement, octaves) = mouvement.autour_de(ancre);
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        let diagonale = f64::from(largeur).hypot(f64::from(hauteur));
        let decision = if mouvement.zoom_domine(deplacement, diagonale) {
            Decision::Pincement
        } else {
            Decision::Pan
        };
        self.chronique.navigation.evenement_du_pave(decision);
        if let Some((fx, fy)) = mouvement.point_fixe() {
            let ecart = (fx - ancre.0, fy - ancre.1);
            self.chronique.navigation.point_fixe_du_pave(ecart);
        }
        if deplacement != (0.0, 0.0) {
            self.elan.placer_pan(deplacement.0, deplacement.1);
        }
        // Au mode référence, `Alt` + pincer agrandit la fenêtre (REFERENCE-2) ; le déplacement
        // des doigts, lui, reste au canevas.
        if octaves != 0.0 && !self.redimensionner_au_pincement(octaves) {
            self.elan.placer_zoom(octaves, ancre);
        }
    }
}

#[cfg(test)]
mod tests;
