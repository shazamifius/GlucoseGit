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

use std::time::{Duration, Instant};

/// Combien de tranches d'histogramme par doublement de durée (voir le module parent).
const PAR_OCTAVE: usize = 4;

/// De 1 µs à 2^20 µs. Au-delà d'une seconde de latence, la question n'est plus la finesse.
const OCTAVES: usize = 21;

const TRANCHES: usize = PAR_OCTAVE * OCTAVES;

/// Ce qu'un événement de défilement a voulu dire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Le geste a été compris comme un zoom, au pincement ou au `Ctrl`.
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

/// Tout ce que la navigation a vécu pendant la session.
#[derive(Debug)]
pub struct Navigation {
    zooms: u64,
    pans: u64,
    crans: u64,
    /// L'instant du plus ancien événement que l'écran n'a pas encore montré.
    ///
    /// `None` quand tout a été montré : la latence ne se mesure que lorsqu'il y a quelque
    /// chose en attente, sinon on mesurerait le temps où personne ne demandait rien.
    en_attente: Option<Instant>,
    /// La distribution des latences, en microsecondes.
    latences: [u32; TRANCHES],
    mesurees: u64,
    pire_us: u32,
}

impl Default for Navigation {
    fn default() -> Self {
        Self::nouvelle()
    }
}

impl Navigation {
    pub fn nouvelle() -> Self {
        Self {
            zooms: 0,
            pans: 0,
            crans: 0,
            en_attente: None,
            latences: [0; TRANCHES],
            mesurees: 0,
            pire_us: 0,
        }
    }

    /// Note un événement de navigation et ce qu'on a décidé d'en faire.
    ///
    /// Seul le **premier** d'une rafale pose l'instant d'attente : la latence qui compte est
    /// celle du plus ancien geste que l'écran n'a pas encore montré, pas celle du dernier.
    pub fn evenement(&mut self, decision: Decision) {
        match decision {
            Decision::Zoom => self.zooms += 1,
            Decision::Pan => self.pans += 1,
            Decision::CranDeSouris => self.crans += 1,
        }
        self.en_attente.get_or_insert_with(Instant::now);
    }

    /// L'écran vient de montrer ce qui était en attente.
    ///
    /// Rend la latence si quelque chose attendait — une image qui ne montre aucun geste neuf
    /// n'en a aucune, et la compter fausserait la distribution vers le bas.
    pub fn image_presentee(&mut self) -> Option<Duration> {
        let depuis = self.en_attente.take()?;
        let latence = depuis.elapsed();
        let us = latence.as_micros().min(u128::from(u32::MAX)) as u32;
        self.latences[tranche(us)] += 1;
        self.mesurees += 1;
        self.pire_us = self.pire_us.max(us);
        Some(latence)
    }

    /// Combien d'événements de chaque nature : zooms, déplacements, crans supposés.
    pub fn comptes(&self) -> (u64, u64, u64) {
        (self.zooms, self.pans, self.crans)
    }

    /// Combien de latences ont été mesurées, et la pire.
    pub fn mesurees(&self) -> (u64, u32) {
        (self.mesurees, self.pire_us)
    }

    /// La latence sous laquelle tombe la part `p` des images, en microsecondes.
    pub fn centile(&self, p: f64) -> u32 {
        if self.mesurees == 0 {
            return 0;
        }
        let cible = (self.mesurees as f64 * p).ceil() as u64;
        let mut cumul = 0u64;
        for (i, n) in self.latences.iter().enumerate() {
            cumul += u64::from(*n);
            if cumul >= cible {
                return borne_haute(i);
            }
        }
        self.pire_us
    }
}

/// L'indice d'histogramme d'une durée (même découpage que le module parent).
fn tranche(us: u32) -> usize {
    if us == 0 {
        return 0;
    }
    let octave = us.ilog2() as usize;
    let base = 1u64 << octave;
    let reste = u64::from(us) - base;
    let sous = (reste * PAR_OCTAVE as u64 / base) as usize;
    (octave * PAR_OCTAVE + sous).min(TRANCHES - 1)
}

/// La durée maximale que contient cette tranche.
fn borne_haute(i: usize) -> u32 {
    let octave = i / PAR_OCTAVE;
    let sous = i % PAR_OCTAVE;
    let base = 1u64 << octave;
    let borne = base + base * (sous as u64 + 1) / PAR_OCTAVE as u64;
    borne.min(u64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests;
