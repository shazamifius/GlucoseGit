//! Ce que la navigation vit : ce que le doigt demande, ce qu'on en fait, et le temps qu'il
//! faut pour que l'écran le montre (NAV-3).
//!
//! # La grandeur qui manquait, et c'est elle qui dit « fluide »
//!
//! La chronique sait dire ce qu'une image coûte. Elle ne sait rien dire du **délai entre le
//! geste et ce qu'on en voit**, et c'est pourtant lui qui se ressent : une application à cent
//! images par seconde dont chaque image montre l'état d'il y a cinquante millisecondes paraît
//! molle, et aucune mesure de durée d'image ne l'explique.
//!
//! On mesure donc, pour chaque image, l'âge du **plus ancien** événement de navigation qu'elle
//! montre pour la première fois. C'est exactement ce que la main attend.
//!
//! # Et ce qu'on décide de son geste
//!
//! Un même événement de défilement peut vouloir dire zoom ou déplacement, et se tromper coûte
//! un saut d'échelle au milieu d'un glissement — ce qui se voit bien plus qu'une image lente.
//! La trace compte donc les deux, plus les fois où le défilement a été **pris pour un cran de
//! souris**, qui est le cas ambigu.

use super::histogramme::Histogramme;
use std::time::{Duration, Instant};

/// Ce qu'un événement de défilement a voulu dire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Le **système** a marqué ce défilement comme un zoom : un pincement à deux doigts.
    ///
    /// Compté à part des autres zooms, et c'est tout l'intérêt : tant que ce compte restait
    /// nul pendant qu'on pinçait, le geste n'arrivait tout simplement pas jusqu'ici.
    Pincement,
    /// Le geste a été compris comme un zoom, `Ctrl` étant tenu au clavier.
    Zoom,
    /// Le geste a été compris comme un déplacement de la vue.
    Pan,
    /// Le geste a été pris pour un **cran de molette de souris**, donc un zoom.
    ///
    /// C'est le cas ambigu : un pavé tactile qui livre un défilement vertical pur d'un nombre
    /// entier de lignes est indiscernable d'une souris. S'il abonde pendant qu'on glisse à
    /// deux doigts, chacun coûte un saut d'échelle visible.
    CranDeSouris,
}

/// Ce que la main a demandé, par nature de geste.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Comptes {
    pub pincements: u64,
    pub zooms: u64,
    pub pans: u64,
    pub crans: u64,
    /// **Dont par *Direct Manipulation*** (fiche 53) : le pincement, puis le déplacement. Le
    /// 07/10, tous les pincements y passaient — et rien ne disait si les déplacements aussi,
    /// ou s'ils arrivaient encore en molette, par paquets.
    pub par_le_pave: (u64, u64),
}

impl Comptes {
    /// Combien d'événements en tout — zéro veut dire que personne n'a navigué.
    pub fn total(&self) -> u64 {
        self.pincements + self.zooms + self.pans + self.crans
    }
}

/// Tout ce que la navigation a vécu pendant la session.
#[derive(Debug)]
pub struct Navigation {
    pincements: u64,
    zooms: u64,
    pans: u64,
    crans: u64,
    par_le_pave: (u64, u64),
    /// Les gestes du pavé passés d'un déplacement à un pincement, et le plus petit écart
    /// d'échelle qui l'a fait (fiche 53 § 8).
    bascules: (u64, f64),
    /// L'instant du plus ancien événement que l'écran n'a pas encore montré.
    ///
    /// `None` quand tout a été montré : la latence ne se mesure que lorsqu'il y a quelque
    /// chose en attente, sinon on mesurerait le temps où personne ne demandait rien.
    en_attente: Option<Instant>,
    /// La distribution des latences, en microsecondes.
    latences: Histogramme,
}

impl Default for Navigation {
    fn default() -> Self {
        Self::nouvelle()
    }
}

impl Navigation {
    pub fn nouvelle() -> Self {
        Self {
            pincements: 0,
            zooms: 0,
            pans: 0,
            crans: 0,
            par_le_pave: (0, 0),
            bascules: (0, f64::INFINITY),
            en_attente: None,
            latences: Histogramme::nouveau(),
        }
    }

    /// Note un événement de navigation et ce qu'on a décidé d'en faire.
    ///
    /// Seul le **premier** d'une rafale pose l'instant d'attente : la latence qui compte est
    /// celle du plus ancien geste que l'écran n'a pas encore montré, pas celle du dernier.
    pub fn evenement(&mut self, decision: Decision) {
        match decision {
            Decision::Pincement => self.pincements += 1,
            Decision::Zoom => self.zooms += 1,
            Decision::Pan => self.pans += 1,
            Decision::CranDeSouris => self.crans += 1,
        }
        self.en_attente.get_or_insert_with(Instant::now);
    }

    /// Un geste attend-il d'être montré ?
    pub fn en_attente(&self) -> bool {
        self.en_attente.is_some()
    }

    /// Un événement du pavé reçu par *Direct Manipulation* : compté comme les autres, et à part.
    pub fn evenement_du_pave(&mut self, decision: Decision) {
        match decision {
            Decision::Pincement => self.par_le_pave.0 += 1,
            _ => self.par_le_pave.1 += 1,
        }
        self.evenement(decision);
    }

    /// Un geste du pavé vient de passer d'un déplacement à un pincement, à cet écart d'échelle.
    pub fn bascule(&mut self, ecart: f64) {
        self.bascules.0 += 1;
        self.bascules.1 = self.bascules.1.min(ecart);
    }

    /// Combien de gestes ont basculé, et le plus petit écart d'échelle qui l'a fait.
    pub fn bascules(&self) -> (u64, f64) {
        self.bascules
    }

    /// L'écran vient de montrer ce qui était en attente.
    ///
    /// Rend la latence si quelque chose attendait — une image qui ne montre aucun geste neuf
    /// n'en a aucune, et la compter fausserait la distribution vers le bas.
    pub fn image_presentee(&mut self) -> Option<Duration> {
        let depuis = self.en_attente.take()?;
        let latence = depuis.elapsed();
        let us = latence.as_micros().min(u128::from(u32::MAX)) as u32;
        self.latences.ajouter(us);
        Some(latence)
    }

    /// Combien d'événements de chaque nature : pincements, zooms clavier, déplacements,
    /// crans supposés.
    pub fn comptes(&self) -> Comptes {
        Comptes {
            pincements: self.pincements,
            zooms: self.zooms,
            pans: self.pans,
            crans: self.crans,
            par_le_pave: self.par_le_pave,
        }
    }

    /// Combien de latences ont été mesurées, et la pire.
    pub fn mesurees(&self) -> (u64, u32) {
        (self.latences.compte(), self.latences.pire())
    }

    /// La latence sous laquelle tombe la part `p` des images, en microsecondes.
    pub fn centile(&self, p: f64) -> u32 {
        self.latences.centile(p)
    }
}

#[cfg(test)]
mod tests;
