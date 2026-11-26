//! **Les octets d'une vidéo, tels qu'AVFoundation les demande** (VIDEO-MAC-1, fiche 175 § 7) : un
//! `AVAssetResourceLoaderDelegate` à nous. La vidéo porte une adresse d'un schéma qu'AVFoundation ne
//! connaît pas (`glucose-video:`) : il demande donc chaque plage d'octets à ce chargeur, qui la lit
//! dans le [`Lieu`](crate::renderer::video::Lieu) — un fichier, la tranche d'un document, la mémoire
//! —, là où elle est, comme le flot de Windows et l'`appsrc` de GStreamer.

use crate::renderer::video::lieu::Octets;
use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol};
use objc2::{define_class, msg_send, AllocAnyThread, DefinedClass};
use objc2_av_foundation::{
    AVAssetResourceLoader, AVAssetResourceLoaderDelegate, AVAssetResourceLoadingRequest,
};
use objc2_foundation::{NSData, NSString};

/// Ce qu'une réponse porte au plus : AVFoundation demande souvent tout le reste du fichier.
const MORCEAU: usize = 256 * 1024;

/// **Le type d'une vidéo pour AVFoundation** (un UTI), d'après l'extension que sa signature dit ;
/// `None` pour ce qu'AVFoundation ne lit pas (Matroska, WebM).
pub(super) fn uti(extension: &str) -> Option<&'static str> {
    Some(match extension {
        "mp4" => "public.mpeg-4",
        "m4v" => "com.apple.m4v-video",
        "mov" => "com.apple.quicktime-movie",
        "3gp" => "public.3gpp",
        "avi" => "public.avi",
        _ => return None,
    })
}

/// Ses octets, et leur type.
pub(super) struct Source {
    octets: Octets,
    uti: &'static str,
}

define_class!(
    // SAFETY : `NSObject` n'exige rien d'une sous-classe ; `Chargeur` n'implémente pas `Drop`.
    #[unsafe(super(NSObject))]
    #[name = "GlucoseChargeurDeVideo"]
    #[ivars = Source]
    pub(super) struct Chargeur;

    unsafe impl NSObjectProtocol for Chargeur {}

    unsafe impl AVAssetResourceLoaderDelegate for Chargeur {
        /// Toute demande se sert ici, sur la file du chargeur : AVFoundation attend la réponse.
        #[unsafe(method(resourceLoader:shouldWaitForLoadingOfRequestedResource:))]
        fn doit_attendre(
            &self,
            _chargeur: &AVAssetResourceLoader,
            demande: &AVAssetResourceLoadingRequest,
        ) -> bool {
            servir(self.ivars(), demande);
            true
        }
    }
);

impl Chargeur {
    pub(super) fn new(octets: Octets, uti: &'static str) -> Retained<Self> {
        let this = Self::alloc().set_ivars(Source { octets, uti });
        // SAFETY : l'`init` de `NSObject`, après les variables d'instance.
        unsafe { msg_send![super(this), init] }
    }
}

/// **Répond à une demande** : le type et la longueur si elle les demande, puis la plage d'octets
/// voulue, morceau par morceau, tant qu'elle n'est pas annulée.
fn servir(source: &Source, demande: &AVAssetResourceLoadingRequest) {
    let octets = &source.octets;
    // SAFETY : la demande vit pendant l'appel ; chaque réponse est copiée par `NSData`.
    unsafe {
        if let Some(info) = demande.contentInformationRequest() {
            info.setContentType(Some(&NSString::from_str(source.uti)));
            info.setContentLength(octets.longueur() as i64);
            info.setByteRangeAccessSupported(true);
        }
        let Some(donnees) = demande.dataRequest() else {
            demande.finishLoading();
            return;
        };
        let debut = donnees.requestedOffset().max(0) as u64;
        let fin = match donnees.requestsAllDataToEndOfResource() {
            true => octets.longueur(),
            false => debut + donnees.requestedLength().max(0) as u64,
        };
        let mut tampon = vec![0; MORCEAU];
        let mut a = debut;
        while a < fin && !demande.isCancelled() {
            let n = ((fin - a) as usize).min(MORCEAU);
            match octets.lire(a, &mut tampon[..n]) {
                Ok(0) => break,
                Ok(lus) => {
                    donnees.respondWithData(&NSData::with_bytes(&tampon[..lus]));
                    a += lus as u64;
                }
                Err(_) => return demande.finishLoadingWithError(None),
            }
        }
        demande.finishLoading();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn video_mac_1_chaque_extension_a_son_type() {
        assert_eq!(super::uti("mov"), Some("com.apple.quicktime-movie"));
        assert_eq!(super::uti("mp4"), Some("public.mpeg-4"));
        assert_eq!(super::uti("mkv"), None, "AVFoundation ne lit pas Matroska");
    }
}
