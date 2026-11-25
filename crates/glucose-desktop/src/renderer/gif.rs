//! **Les GIF qui bougent** (GIF-1, fiche 159 ; les WebP et PNG animés aussi, § 9) : le lecteur — des
//! octets d'un GIF à la suite de ses
//! toiles, déjà composées (chacune par-dessus la précédente), avec le temps que chacune reste, tour
//! après tour. Il ne garde que la toile en cours : la mémoire d'un GIF est celle d'une image, quelle
//! que soit sa longueur (fiche 159 § 3).

use image::metadata::LoopCount;
use image::AnimationDecoder;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

/// **Un délai de 10 ms ou moins se joue à 100 ms** : la règle d'Internet Explorer, que Chromium,
/// Firefox et WebKit ont reprise — les GIF du monde sont faits pour elle (fiche 159 § 2).
const PLANCHER: Duration = Duration::from_millis(10);
const REMPLACANT: Duration = Duration::from_millis(100);

/// **Le temps qu'une toile reste**, selon la règle des navigateurs.
pub(crate) fn delai(d: image::Delay) -> Duration {
    let (ms, par) = d.numer_denom_ms();
    let delai = Duration::from_secs_f64(f64::from(ms) / f64::from(par.max(1)) / 1000.0);
    if delai <= PLANCHER {
        REMPLACANT
    } else {
        delai
    }
}

/// **Une toile du GIF** : ses pixels (RGBA droits, à la taille du GIF), et le temps qu'elle reste.
pub(crate) struct Toile {
    pub largeur: u32,
    pub hauteur: u32,
    pub rgba: Vec<u8>,
    pub delai: Duration,
}

/// **Le lecteur d'un GIF** : ses octets, les toiles du tour en cours, les tours qui restent.
pub(crate) struct Lecteur {
    octets: Arc<[u8]>,
    toiles: image::Frames<'static>,
    /// Les tours qui restent après celui-ci ; `None` : sans fin.
    tours: Option<u32>,
    /// Les toiles lues depuis l'ouverture : une seule au bout du premier tour, rien à jouer.
    lues: u32,
}

impl Lecteur {
    /// **Un lecteur**, ou rien si ces octets ne sont pas un GIF lisible.
    pub(crate) fn nouveau(octets: Arc<[u8]>) -> Option<Self> {
        let (toiles, tours) = ouvrir(&octets)?;
        Some(Self {
            octets,
            toiles,
            tours,
            lues: 0,
        })
    }

    /// **La toile suivante** ; un tour fini, le suivant reprend du début. `None` quand les tours
    /// sont joués, à la première faute, ou quand le GIF n'a qu'une image : il n'y a rien à jouer.
    pub(crate) fn suivante(&mut self) -> Option<Toile> {
        let toile = match self.toiles.next() {
            Some(toile) => toile.ok()?,
            None => {
                if self.lues < 2 {
                    return None;
                }
                match &mut self.tours {
                    Some(0) => return None,
                    Some(n) => *n -= 1,
                    None => {}
                }
                self.toiles = ouvrir(&self.octets)?.0;
                self.toiles.next()?.ok()?
            }
        };
        self.lues = self.lues.saturating_add(1);
        let delai = delai(toile.delay());
        let rgba = toile.into_buffer();
        Some(Toile {
            largeur: rgba.width(),
            hauteur: rgba.height(),
            rgba: rgba.into_raw(),
            delai,
        })
    }
}

/// Les toiles d'un tour, et les tours qui restent après lui : un GIF, un WebP animé, un PNG animé
/// (fiche 159 § 9) — `image` les compose tous trois.
fn ouvrir(octets: &Arc<[u8]>) -> Option<(image::Frames<'static>, Option<u32>)> {
    use image::codecs::{gif::GifDecoder, png::PngDecoder, webp::WebPDecoder};
    let lecture = Cursor::new(Arc::clone(octets));
    if octets.starts_with(b"GIF8") {
        return Some(tour(GifDecoder::new(lecture).ok()?));
    }
    if octets.starts_with(b"RIFF") {
        let decodeur = WebPDecoder::new(lecture).ok()?;
        return decodeur.has_animation().then(|| tour(decodeur));
    }
    let decodeur = PngDecoder::new(lecture).ok()?;
    if !decodeur.is_apng().ok()? {
        return None;
    }
    Some(tour(decodeur.apng().ok()?))
}

/// Les toiles d'un décodeur, et les tours qui restent après le premier ; `None` : sans fin.
fn tour<D: AnimationDecoder<'static>>(decodeur: D) -> (image::Frames<'static>, Option<u32>) {
    let tours = match decodeur.loop_count() {
        LoopCount::Infinite => None,
        LoopCount::Finite(n) => Some(n.get() - 1),
    };
    (decodeur.into_frames(), tours)
}

pub(crate) mod joueur;
pub(crate) mod signature;

#[cfg(test)]
mod tests;
