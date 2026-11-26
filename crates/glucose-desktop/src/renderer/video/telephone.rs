//! **Une vidéo jouée par Android** (VIDEO-ANDROID-1, fiche 172) : le lecteur du système, par la
//! porte que `glucose-android` branche (`plateforme::video`). Les mêmes gestes que celui de
//! GStreamer (`lecteur_gst`), pour que le joueur ne sache pas lequel il tient.

use super::Fin;
use crate::plateforme::video::Attente;
use crate::renderer::gif::Toile;

/// **Ce qu'une attente rend au joueur** : une image prêtée qu'on n'a pas su lire est une faute,
/// jamais une image sautée — le système la rendrait pareille à la suivante.
fn issue(attente: Attente, toile: Option<Toile>) -> Result<Option<Toile>, Fin> {
    match attente {
        Attente::Image => toile.map(Some).ok_or(Fin::Faute),
        Attente::Rien => Ok(None),
        Attente::AuBout => Err(Fin::AuBout),
        Attente::Faute => Err(Fin::Faute),
    }
}

#[cfg(target_os = "android")]
pub(crate) use android::{image_fixe, lisible_ici, LecteurDuTelephone};

#[cfg(target_os = "android")]
mod android {
    use super::{issue, Fin};
    use crate::plateforme::video::{porte, Lecteur};
    use crate::renderer::gif::Toile;
    use crate::renderer::video::{yuv, Lieu};
    use std::time::Duration;

    /// **Une vidéo ouverte par Android**, prête à jouer.
    pub(crate) struct LecteurDuTelephone(Box<dyn Lecteur>);

    impl LecteurDuTelephone {
        /// **Ouvre la vidéo** de ce lieu ; `None` si le système ne sait pas la lire.
        pub(crate) fn ouvrir(lieu: Lieu, avec_le_son: bool) -> Option<Self> {
            porte()?.ouvrir(&lieu.ou(), avec_le_son).map(Self)
        }

        /// **L'image suivante**, attendue au plus `delai` : `Ok(None)` si elle n'est pas encore
        /// là, `Err` à la fin de la vidéo ou à la première faute.
        pub(crate) fn suivante(&self, delai: Duration) -> Result<Option<Toile>, Fin> {
            let mut toile = None;
            let attente = self
                .0
                .attendre(delai, &mut |plans| toile = yuv::en_rgba(plans));
            issue(attente, toile)
        }

        pub(crate) fn revenir_au_debut(&mut self) -> bool {
            self.0.revenir_au_debut()
        }

        pub(crate) fn jouer(&self, oui: bool) {
            self.0.jouer(oui);
        }

        pub(crate) fn couper_le_son(&self, oui: bool) {
            self.0.couper_le_son(oui);
        }
    }

    /// **La première image** que le système rend, déjà tournée.
    pub(crate) fn image_fixe(lieu: Lieu) -> Option<Toile> {
        let image = porte()?.image_fixe(&lieu.ou())?;
        Some(Toile {
            largeur: image.largeur,
            hauteur: image.hauteur,
            rgba: image.rgba,
            delai: Duration::ZERO,
        })
    }

    /// La porte est branchée.
    pub(crate) fn lisible_ici() -> bool {
        porte().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn video_android_1_une_image_illisible_est_une_faute() {
        let toile = || Toile {
            largeur: 1,
            hauteur: 1,
            rgba: vec![0; 4],
            delai: Duration::ZERO,
        };
        assert!(matches!(issue(Attente::Image, Some(toile())), Ok(Some(_))));
        assert_eq!(issue(Attente::Image, None).err(), Some(Fin::Faute));
        assert!(matches!(issue(Attente::Rien, None), Ok(None)));
        assert_eq!(issue(Attente::AuBout, None).err(), Some(Fin::AuBout));
        assert_eq!(issue(Attente::Faute, None).err(), Some(Fin::Faute));
    }
}
