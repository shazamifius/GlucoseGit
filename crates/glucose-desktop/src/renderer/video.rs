//! **Les vidéos** (VIDEO-1, fiche 169 ; l'étude, fiche 160) : un nœud photo dont les octets sont
//! une vidéo (`is_video`, déjà dans le format). Son image fixe est sa première image — pyramide,
//! aperçu, export la traitent en photo ; elle joue au clic sur ▶, le son coupé jusqu'au clic sur
//! son icône (sa réponse du 25/11, `relais-q-videos`).
//!
//! Les décodeurs sont ceux du système (la voie A de la fiche 160) : aucun poids livré, aucune
//! licence de codec à porter, la carte graphique décode. Linux d'abord (GStreamer) ; ailleurs,
//! en attendant leur porte, une vidéo n'a pas d'image fixe et ne joue pas.

use crate::persist::objets::Objets;

pub(crate) mod commandes;
pub(crate) mod joueur;
pub(crate) mod signature;

#[cfg(target_os = "linux")]
mod gst;
#[cfg(target_os = "linux")]
mod lecteur_gst;
#[cfg(target_os = "linux")]
pub(crate) use lecteur_gst::LecteurVideo;
#[cfg(target_os = "linux")]
mod lieu;
#[cfg(target_os = "linux")]
pub(crate) use lieu::Lieu;

/// **Ce système sait-il lire une vidéo ?** Sous Linux, si GStreamer est là ; ailleurs, pas
/// encore (leur porte suit, fiche 169).
pub(crate) fn lisible_ici() -> bool {
    #[cfg(target_os = "linux")]
    {
        gst::gst().is_some()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// **L'image fixe d'une vidéo** : sa première image, en RGBA ; `None` si ce système ne sait pas
/// la lire.
pub(crate) fn image_fixe(
    src: &str,
    objets: Option<&Objets>,
) -> Option<crate::renderer::gif::Toile> {
    #[cfg(target_os = "linux")]
    {
        LecteurVideo::ouvrir(Lieu::de(src, objets)?, false)?.image_arretee()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (src, objets);
        None
    }
}
