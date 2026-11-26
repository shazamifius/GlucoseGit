//! **Une vidéo jouée par le Mac** (VIDEO-MAC-1, fiche 175 § 7) : `AVPlayer` démultiplexe, décode (par
//! la carte graphique), joue le son et cadence ; une `AVPlayerItemVideoOutput` rend chaque image
//! en BGRA à l'heure où elle doit paraître, lue en RGBA — les mêmes gestes que les lecteurs de
//! GStreamer et de Windows, pour que le joueur ne sache pas lequel il tient. Ses octets passent
//! par un chargeur à nous (`chargeur.rs`), sans copie sur le disque.
//!
//! **Hors du fil principal.** Depuis le SDK de macOS 15, Apple marque `AVPlayer` et
//! `AVPlayerItem` `@MainActor` pour Swift, et `objc2` en tire `MainThreadOnly`. L'Objective-C les
//! appelle de tout fil depuis toujours, et le système ne le vérifie pas : seul le code Swift
//! contrôle son acteur. Le fil principal de Glucose est celui de la fenêtre ; y passer demanderait
//! une file que `cargo test` ne sert jamais (ses épreuves attendraient toujours). Chaque lecteur
//! reste donc sur **un seul fil**, le sien (celui du joueur, ou de l'atelier pour l'image fixe) :
//! le jeton du fil principal est un mensonge assumé, écrit ici et nulle part ailleurs. Si un macOS
//! à venir le vérifiait, les épreuves du Mac tomberaient à la CI, avant toute publication.
//!
//! **Aucune exception ne traverse Rust.** Une exception d'Objective-C levée sous Rust tue le
//! processus (« Rust cannot catch foreign exceptions » : vu à la première CI du Mac, le 26/11).
//! Chaque entrée dans AVFoundation passe donc par [`sans_exception`] : l'exception se dit dans la
//! sortie, et la vidéo reste un lanceur.

mod chargeur;

use super::lieu::Octets;
use super::{bgra, Fin, Lieu, PREPARATION, REGARD};
use crate::renderer::gif::Toile;
use chargeur::{uti, Chargeur};
use dispatch2::{DispatchQueue, DispatchRetained};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AllocAnyThread, MainThreadMarker};
use objc2_av_foundation::{
    AVPlayer, AVPlayerActionAtItemEnd, AVPlayerItem, AVPlayerItemStatus, AVPlayerItemVideoOutput,
    AVURLAsset, AVVideoComposition,
};
use objc2_core_media::kCMTimeZero;
use objc2_core_video::{
    kCVPixelBufferPixelFormatTypeKey, kCVPixelFormatType_32BGRA, CVPixelBuffer,
    CVPixelBufferGetBaseAddress, CVPixelBufferGetBytesPerRow, CVPixelBufferGetHeight,
    CVPixelBufferGetWidth, CVPixelBufferLockBaseAddress, CVPixelBufferLockFlags,
    CVPixelBufferUnlockBaseAddress,
};
use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};
use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

/// L'adresse de toute vidéo : son schéma est inconnu d'AVFoundation, qui demande donc ses octets
/// au chargeur.
const ADRESSE: &str = "glucose-video://lieu/video";

/// **Une vidéo ouverte**, en pause au début.
pub(crate) struct LecteurVideo {
    joueur: Retained<AVPlayer>,
    element: Retained<AVPlayerItem>,
    sortie: Retained<AVPlayerItemVideoOutput>,
    /// Muette pour toujours : l'image fixe n'a que faire du son.
    muette: bool,
    /// Le chargeur d'AVFoundation ne le tient que faiblement.
    _chargeur: Retained<Chargeur>,
    _file: DispatchRetained<DispatchQueue>,
}

impl LecteurVideo {
    /// **Ouvre la vidéo** de ce lieu, prête à jouer, son coupé ; `None` si AVFoundation ne sait pas
    /// la lire, ou ne l'a pas préparée à temps.
    pub(crate) fn ouvrir(lieu: Lieu, avec_le_son: bool) -> Option<Self> {
        let octets = lieu.ouvrir()?;
        let uti = uti(octets.extension()?)?;
        let lecteur = sans_exception("l'ouverture", || Self::preparer(octets, uti, avec_le_son))??;
        lecteur.prete().then_some(())?;
        lecteur.redresser();
        Some(lecteur)
    }

    /// Le joueur, sa sortie et son chargeur, en pause au début ; pas encore prêt.
    fn preparer(octets: Octets, uti: &'static str, avec_le_son: bool) -> Option<Self> {
        // SAFETY : le mensonge assumé de l'en-tête — un seul fil, le nôtre.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };
        // SAFETY : chaque objet vit autant que le lecteur ; le chargeur, retenu ici, sert ses
        // demandes sur sa propre file, jamais sur ce fil.
        unsafe {
            let adresse = NSURL::URLWithString(&NSString::from_str(ADRESSE))?;
            let video = AVURLAsset::URLAssetWithURL_options(&adresse, None);
            let chargeur = Chargeur::new(octets, uti);
            let file = DispatchQueue::new("glucose.video.octets", None);
            let delegue = ProtocolObject::from_ref(&*chargeur);
            (video.resourceLoader()).setDelegate_queue(Some(delegue), Some(&file));
            let element = AVPlayerItem::playerItemWithAsset(&video, mtm);
            let sortie = AVPlayerItemVideoOutput::initWithPixelBufferAttributes(
                AVPlayerItemVideoOutput::alloc(),
                Some(&en_bgra()),
            );
            element.addOutput(&sortie);
            let joueur = AVPlayer::playerWithPlayerItem(Some(&element), mtm);
            joueur.setMuted(true);
            joueur.setActionAtItemEnd(AVPlayerActionAtItemEnd::Pause);
            Some(Self {
                joueur,
                element,
                sortie,
                muette: !avec_le_son,
                _chargeur: chargeur,
                _file: file,
            })
        }
    }

    /// **Attend qu'elle soit prête**, au plus le temps de préparation ; sinon, dit pourquoi.
    fn prete(&self) -> bool {
        let debut = Instant::now();
        let statut = || sans_exception("le statut", || unsafe { self.element.status() });
        loop {
            // SAFETY : l'élément vit autant que le lecteur.
            match statut() {
                Some(AVPlayerItemStatus::ReadyToPlay) => return true,
                Some(AVPlayerItemStatus::Failed) | None => break,
                Some(_) if debut.elapsed() < PREPARATION => std::thread::sleep(REGARD),
                Some(_) => break,
            }
        }
        // SAFETY : comme plus haut.
        let erreur = sans_exception("l'erreur", || unsafe { self.element.error() }).flatten();
        let pourquoi = erreur.map(|e| e.localizedDescription().to_string());
        eprintln!(
            "vidéo (Mac) : pas prête après {:?} — statut {:?}, {}",
            debut.elapsed(),
            statut(),
            pourquoi.as_deref().unwrap_or("sans erreur")
        );
        false
    }

    /// **Redresse une vidéo filmée debout** : un téléphone la range souvent couchée, avec l'ordre de
    /// la tourner ; la composition « d'après ses propriétés » l'applique, comme `videoflip` sous
    /// Linux. Posée une fois la vidéo prête — ses pistes lues ; si AVFoundation la refuse, la
    /// vidéo joue telle qu'elle est rangée. Sa remplaçante (`…completionHandler:`) n'existe que
    /// depuis macOS 15.
    fn redresser(&self) {
        // SAFETY : l'élément vit autant que le lecteur.
        sans_exception("la composition", || unsafe {
            #[allow(deprecated)]
            let composition =
                AVVideoComposition::videoCompositionWithPropertiesOfAsset(&self.element.asset());
            self.element.setVideoComposition(Some(&composition));
        });
    }

    /// **L'image où la vidéo est arrêtée** : la première. La vidéo joue, muette, jusqu'à elle,
    /// puis s'arrête (une sortie n'a d'image qu'une fois la lecture partie).
    pub(crate) fn image_arretee(&self) -> Option<Toile> {
        self.jouer(true);
        let toile = self.suivante(PREPARATION).ok().flatten();
        self.jouer(false);
        toile
    }

    /// **L'image suivante**, attendue au plus `delai` : `Ok(None)` si elle n'est pas encore là,
    /// `Err` à la fin de la vidéo ou à la première faute.
    pub(crate) fn suivante(&self, delai: Duration) -> Result<Option<Toile>, Fin> {
        let limite = Instant::now() + delai;
        loop {
            if let Some(fin) = sans_exception("l'image", || self.regarder()).ok_or(Fin::Faute)? {
                return fin;
            }
            let reste = limite.saturating_duration_since(Instant::now());
            if reste.is_zero() {
                return Ok(None);
            }
            std::thread::sleep(reste.min(REGARD));
        }
    }

    /// **Un regard** : `Some` si l'image neuve est là, si la vidéo est au bout ou en faute ;
    /// `None` s'il faut regarder encore.
    fn regarder(&self) -> Option<Result<Option<Toile>, Fin>> {
        // SAFETY : l'élément et la sortie vivent autant que le lecteur ; l'image copiée est à nous.
        unsafe {
            if self.element.status() == AVPlayerItemStatus::Failed {
                return Some(Err(Fin::Faute));
            }
            let maintenant = self.element.currentTime();
            if self.sortie.hasNewPixelBufferForItemTime(maintenant) {
                let image = (self.sortie).copyPixelBufferForItemTime_itemTimeForDisplay(
                    maintenant,
                    std::ptr::null_mut(),
                );
                return Some(image.and_then(|i| toile(&i)).map(Some).ok_or(Fin::Faute));
            }
            // Au bout, le joueur s'arrête là (`AVPlayerActionAtItemEnd::Pause`).
            (maintenant.compare(self.element.duration()) >= 0).then_some(Err(Fin::AuBout))
        }
    }

    /// **Revient au début**, pour une vidéo qui joue en boucle (VIDEO-2).
    pub(crate) fn revenir_au_debut(&mut self) -> bool {
        // SAFETY : le joueur vit autant que le lecteur.
        sans_exception("le retour au début", || unsafe {
            self.joueur.seekToTime(kCMTimeZero);
            self.joueur.play();
        })
        .is_some()
    }

    pub(crate) fn jouer(&self, oui: bool) {
        // SAFETY : comme `revenir_au_debut`.
        sans_exception("la lecture", || unsafe {
            match oui {
                true => self.joueur.play(),
                false => self.joueur.pause(),
            }
        });
    }

    pub(crate) fn couper_le_son(&self, oui: bool) {
        // SAFETY : comme `revenir_au_debut`.
        sans_exception("le son", || unsafe {
            self.joueur.setMuted(oui || self.muette)
        });
    }
}

/// **Un appel à AVFoundation, sans qu'une exception le traverse** : elle se dit dans la sortie
/// (`quoi` la situe), et l'appel rend `None`. Rien n'est laissé à moitié : l'appelant abandonne
/// ce qu'il faisait, et un lecteur en faute se ferme.
pub(super) fn sans_exception<R>(quoi: &str, f: impl FnOnce() -> R) -> Option<R> {
    match objc2::exception::catch(AssertUnwindSafe(f)) {
        Ok(r) => Some(r),
        Err(exception) => {
            let dit = exception.map_or("une exception sans objet".into(), |e| e.to_string());
            eprintln!("vidéo (Mac) : {quoi} a levé {dit}");
            None
        }
    }
}

/// Les images en BGRA, 8 bits par canal : ce que `bgra::en_rgba` lit.
fn en_bgra() -> Retained<NSDictionary<NSString, AnyObject>> {
    // SAFETY : une `CFString` et une `NSString` sont le même objet (« toll-free bridged »).
    let cle: &NSString = unsafe { &*(kCVPixelBufferPixelFormatTypeKey as *const _ as *const _) };
    let format = NSNumber::new_u32(kCVPixelFormatType_32BGRA);
    let valeur: &AnyObject = &format;
    NSDictionary::from_slices(&[cle], &[valeur])
}

/// **L'image d'un `CVPixelBuffer`**, lue sous son verrou.
fn toile(image: &CVPixelBuffer) -> Option<Toile> {
    let (l, h) = (CVPixelBufferGetWidth(image), CVPixelBufferGetHeight(image));
    let lecture = CVPixelBufferLockFlags::ReadOnly;
    // SAFETY : les octets ne se lisent que sous le verrou, rendu avant la sortie.
    unsafe {
        if CVPixelBufferLockBaseAddress(image, lecture) != 0 {
            return None;
        }
        let pas = CVPixelBufferGetBytesPerRow(image);
        let base = CVPixelBufferGetBaseAddress(image).cast::<u8>();
        let rgba = (!base.is_null())
            .then(|| {
                bgra::en_rgba(
                    std::slice::from_raw_parts(base, pas * h),
                    pas,
                    l as u32,
                    h as u32,
                )
            })
            .flatten();
        CVPixelBufferUnlockBaseAddress(image, lecture);
        Some(Toile {
            largeur: l as u32,
            hauteur: h as u32,
            rgba: rgba?,
            delai: Duration::ZERO,
        })
    }
}

/// **Le Mac sait-il lire une vidéo ?** AVFoundation est toujours là.
pub(crate) fn lisible_ici() -> bool {
    true
}
