//! **Une vidéo jouée par Windows** (VIDEO-WINDOWS-1, fiche 175) : le Media Engine de Media
//! Foundation — celui des balises `<video>` d'Edge et des applications de Windows — démultiplexe,
//! décode, joue le son et cadence. Sans fenêtre à lui (le mode « serveur d'images »),
//! `OnVideoStreamTick` dit quand une image neuve doit paraître, et `TransferVideoFrame` la peint
//! dans un bitmap WIC, lu en RGBA : les mêmes gestes que le lecteur de GStreamer, pour que le
//! joueur ne sache pas lequel il tient.
//!
//! **Sans gestionnaire DXGI** : le moteur décode alors par le processeur, mais rend l'image en
//! mémoire, là où le joueur la veut (une pyramide, comme une photo). Avec lui, la carte graphique
//! décoderait, mais WIC est refusé (d'après Wine) : il faudrait une texture Direct3D 11, puis la
//! recopier en mémoire — le pas suivant, s'il manque des images.

mod flot;

use super::mf::{mf, Demarre, Mf};
use super::{bgra, Fin, Lieu, PREPARATION, REGARD};
use crate::renderer::gif::Toile;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::{Duration, Instant};
use windows::core::{implement, Interface, BSTR};
use windows::Win32::Foundation::{RECT, S_OK};
use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM;
use windows::Win32::Graphics::Imaging::{
    CLSID_WICImagingFactory, GUID_WICPixelFormat32bppBGRA, IWICBitmap, IWICImagingFactory,
    WICBitmapCacheOnDemand, WICBitmapLockRead, WICRect,
};
use windows::Win32::Media::MediaFoundation::{
    CLSID_MFMediaEngineClassFactory, IMFMediaEngine, IMFMediaEngineClassFactory, IMFMediaEngineEx,
    IMFMediaEngineNotify, IMFMediaEngineNotify_Impl, MFARGB, MF_MEDIA_ENGINE_CALLBACK,
    MF_MEDIA_ENGINE_EVENT, MF_MEDIA_ENGINE_EVENT_ENDED, MF_MEDIA_ENGINE_EVENT_ERROR,
    MF_MEDIA_ENGINE_EVENT_FIRSTFRAMEREADY, MF_MEDIA_ENGINE_EVENT_LOADEDDATA,
    MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};

/// **Ce que Windows a pour préparer l'image fixe, la vidéo arrêtée**, avant qu'elle joue, muette,
/// jusqu'à elle (Wine ne la prépare jamais : chaque image fixe y attend ce temps de plus).
const AVANT_PLAY: Duration = Duration::from_millis(500);

/// **Ce que le moteur dit de lui-même**, depuis ses fils à lui.
#[derive(Default)]
struct Nouvelles {
    chargee: bool,
    au_bout: bool,
    faute: bool,
}

#[derive(Default)]
struct Etat {
    nouvelles: Mutex<Nouvelles>,
    signal: Condvar,
}

impl Etat {
    fn lire<R>(&self, f: impl FnOnce(&mut Nouvelles) -> R) -> R {
        f(&mut self
            .nouvelles
            .lock()
            .unwrap_or_else(PoisonError::into_inner))
    }

    /// Attend au plus `delai`, tant que `encore` le demande.
    fn attendre(&self, delai: Duration, mut encore: impl FnMut(&Nouvelles) -> bool) {
        let nouvelles = self
            .nouvelles
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let _ = self
            .signal
            .wait_timeout_while(nouvelles, delai, |n| encore(n));
    }
}

/// **L'oreille du moteur** : il exige qu'on l'écoute (`MF_MEDIA_ENGINE_CALLBACK`).
#[implement(IMFMediaEngineNotify)]
struct Ecoute(Arc<Etat>);

impl IMFMediaEngineNotify_Impl for Ecoute_Impl {
    fn EventNotify(&self, evenement: u32, _: usize, _: u32) -> windows::core::Result<()> {
        let evenement = MF_MEDIA_ENGINE_EVENT(evenement as i32);
        self.0.lire(|n| match evenement {
            MF_MEDIA_ENGINE_EVENT_LOADEDDATA | MF_MEDIA_ENGINE_EVENT_FIRSTFRAMEREADY => {
                n.chargee = true;
            }
            MF_MEDIA_ENGINE_EVENT_ENDED => n.au_bout = true,
            MF_MEDIA_ENGINE_EVENT_ERROR => n.faute = true,
            _ => {}
        });
        self.0.signal.notify_all();
        Ok(())
    }
}

/// Le moteur, arrêté quand on le lâche : il tient des fils et la sortie du son.
struct Moteur(IMFMediaEngine);

impl Drop for Moteur {
    fn drop(&mut self) {
        // SAFETY : le moteur vit jusqu'ici ; rien ne s'en sert après.
        let _ = unsafe { self.0.Shutdown() };
    }
}

/// **Une vidéo ouverte**, en pause sur sa première image. Les champs partent dans l'ordre : le
/// moteur, son image, puis Media Foundation.
pub(crate) struct LecteurVideo {
    moteur: Moteur,
    etat: Arc<Etat>,
    image: IWICBitmap,
    taille: (u32, u32),
    /// Ses octets, gardés pour revenir au début par un moteur neuf.
    lieu: Lieu,
    mf: &'static Mf,
    _demarre: Demarre,
}

/// **Un moteur dont la vidéo est chargée**, et ce qu'il dit ; `None` s'il ne sait pas la lire,
/// ou pas à temps.
///
/// # Safety
/// COM et Media Foundation démarrés sur ce fil, jusqu'à la fin du moteur.
unsafe fn preparer(mf: &Mf, lieu: &Lieu) -> Option<(Moteur, Arc<Etat>)> {
    let etat = Arc::new(Etat::default());
    let moteur = Moteur(unsafe { creer(mf, &etat) }.ok()?);
    unsafe { charger(mf, &moteur.0, lieu) }?;
    etat.attendre(PREPARATION, |n| !n.chargee && !n.faute);
    etat.lire(|n| n.chargee && !n.faute)
        .then_some((moteur, etat))
}

impl LecteurVideo {
    /// **Ouvre la vidéo** de ce lieu, prête à jouer, son coupé ; `None` si Media Foundation manque,
    /// ne sait pas la lire, ou ne l'a pas préparée à temps. Avec le son ou sans, le moteur est le
    /// même, muet jusqu'au clic : voir [`creer`].
    pub(crate) fn ouvrir(lieu: Lieu, _avec_le_son: bool) -> Option<Self> {
        let mf = mf()?;
        let demarre = Demarre::de(mf)?;
        // SAFETY : COM et Media Foundation sont démarrés sur ce fil, et le restent jusqu'à la fin
        // de `demarre`, qui part après le moteur (les locales partent à rebours, les champs dans
        // l'ordre).
        let (moteur, etat) = unsafe { preparer(mf, &lieu) }?;
        let taille = unsafe { taille(&moteur.0) }?;
        let image = unsafe { bitmap(taille) }.ok()?;
        Some(Self {
            moteur,
            etat,
            image,
            taille,
            lieu,
            mf,
            _demarre: demarre,
        })
    }

    /// **L'image où la vidéo est arrêtée** : la première. Windows la prépare avant `Play`, comme
    /// le poster d'une balise `<video>` — mais seulement quand on la demande (`OnVideoStreamTick`,
    /// d'après les épreuves de Wine) : peinte sans la demander, elle était noire sur le vrai Windows
    /// de la CI (fiche 175 § 9). Wine ne la prépare jamais : la vidéo joue alors, muette, jusqu'à sa
    /// première image, puis s'arrête.
    pub(crate) fn image_arretee(&self) -> Option<Toile> {
        if let Ok(Some(toile)) = self.suivante(AVANT_PLAY) {
            return Some(toile);
        }
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
            if self.etat.lire(|n| n.faute) {
                return Err(Fin::Faute);
            }
            // Une dernière image peut suivre la fin annoncée : elle passe avant.
            if self.image_neuve() {
                return self.toile().map(Some).ok_or(Fin::Faute);
            }
            if self.etat.lire(|n| n.au_bout) {
                return Err(Fin::AuBout);
            }
            let reste = limite.saturating_duration_since(Instant::now());
            if reste.is_zero() {
                return Ok(None);
            }
            self.etat
                .attendre(reste.min(REGARD), |n| !n.au_bout && !n.faute);
        }
    }

    /// **Revient au début**, pour une vidéo qui joue en boucle (VIDEO-2) : un moteur neuf sur les
    /// mêmes octets, le son tel qu'il était ; `false` si le système refuse. Sous Wine, un moteur
    /// allé au bout ne rend plus d'image — ni par `SetCurrentTime(0)` puis `Play`, ni par
    /// `SetLoop`, ni sa source rechargée — ; un neuf, si (fiche 175 § 2).
    pub(crate) fn revenir_au_debut(&mut self) -> bool {
        // SAFETY : comme `ouvrir` ; l'ancien moteur s'arrête quand le neuf le remplace.
        unsafe {
            let muet = self.moteur.0.GetMuted().as_bool();
            let Some((moteur, etat)) = preparer(self.mf, &self.lieu) else {
                return false;
            };
            self.moteur = moteur;
            self.etat = etat;
            let _ = self.moteur.0.SetMuted(muet);
        }
        self.jouer(true);
        true
    }

    pub(crate) fn jouer(&self, oui: bool) {
        // SAFETY : comme `revenir_au_debut`.
        let _ = unsafe {
            match oui {
                true => self.moteur.0.Play(),
                false => self.moteur.0.Pause(),
            }
        };
    }

    pub(crate) fn couper_le_son(&self, oui: bool) {
        // SAFETY : comme `revenir_au_debut`.
        let _ = unsafe { self.moteur.0.SetMuted(oui) };
    }

    /// **Une image neuve doit-elle paraître ?** L'appel brut : `OnVideoStreamTick` de `windows`
    /// rend `Ok` pour `S_FALSE` (« pas encore ») comme pour `S_OK`.
    fn image_neuve(&self) -> bool {
        let moteur = &self.moteur.0;
        let mut quand = 0i64;
        // SAFETY : le moteur vit autant que le lecteur ; `quand` reçoit l'heure de l'image.
        unsafe { (moteur.vtable().OnVideoStreamTick)(moteur.as_raw(), &mut quand) == S_OK }
    }

    /// **L'image du moment**, peinte dans le bitmap puis lue en RGBA.
    fn toile(&self) -> Option<Toile> {
        let (largeur, hauteur) = self.taille;
        let (l, h) = (largeur as i32, hauteur as i32);
        let cadre = RECT {
            left: 0,
            top: 0,
            right: l,
            bottom: h,
        };
        let noir = MFARGB {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbAlpha: 255,
        };
        let tout = WICRect {
            X: 0,
            Y: 0,
            Width: l,
            Height: h,
        };
        // SAFETY : le bitmap a la taille du cadre ; ses octets ne se lisent que sous le verrou,
        // qui vit jusqu'à la fin du bloc.
        unsafe {
            (self.moteur.0)
                .TransferVideoFrame(&self.image, None, &cadre, Some(&noir))
                .ok()?;
            let verrou = self.image.Lock(&tout, WICBitmapLockRead.0 as u32).ok()?;
            let pas = verrou.GetStride().ok()? as usize;
            let (mut longueur, mut octets) = (0, std::ptr::null_mut());
            verrou.GetDataPointer(&mut longueur, &mut octets).ok()?;
            let octets = std::slice::from_raw_parts(octets, longueur as usize);
            Some(Toile {
                largeur,
                hauteur,
                rgba: bgra::en_rgba(octets, pas, largeur, hauteur)?,
                delai: Duration::ZERO,
            })
        }
    }
}

/// **Le moteur**, en serveur d'images BGRA, à l'écoute. Jamais `MF_MEDIA_ENGINE_FORCEMUTE`, même
/// pour l'image fixe : sur le vrai Windows de la CI, un moteur ainsi muet pour toujours ne jouait
/// plus — aucune image, jamais au bout —, quand le même, muet par `SetMuted` ([`charger`]), jouait
/// (fiche 175 § 9). Wine, lui, jouait les deux.
///
/// # Safety
/// COM et Media Foundation démarrés sur ce fil.
unsafe fn creer(mf: &Mf, etat: &Arc<Etat>) -> windows::core::Result<IMFMediaEngine> {
    unsafe {
        let fabrique: IMFMediaEngineClassFactory =
            CoCreateInstance(&CLSID_MFMediaEngineClassFactory, None, CLSCTX_INPROC_SERVER)?;
        let attributs = mf.attributs(2)?;
        let ecoute: IMFMediaEngineNotify = Ecoute(etat.clone()).into();
        attributs.SetUnknown(&MF_MEDIA_ENGINE_CALLBACK, &ecoute)?;
        let bgra = DXGI_FORMAT_B8G8R8A8_UNORM.0 as u32;
        attributs.SetUINT32(&MF_MEDIA_ENGINE_VIDEO_OUTPUT_FORMAT, bgra)?;
        fabrique.CreateInstance(0, &attributs)
    }
}

/// **Donne au moteur les octets de la vidéo**, son coupé jusqu'au clic sur son icône (sa
/// réponse du 25/11, `relais-q-videos`). Le chargement suit, sur les fils du moteur.
///
/// # Safety
/// Comme [`creer`].
unsafe fn charger(mf: &Mf, moteur: &IMFMediaEngine, lieu: &Lieu) -> Option<()> {
    let (flot, nom) = flot::Flot::ouvrir(lieu)?;
    unsafe {
        let octets = mf.flot_sur(&flot).ok()?;
        let moteur_ex: IMFMediaEngineEx = moteur.cast().ok()?;
        moteur_ex
            .SetSourceFromByteStream(&octets, &BSTR::from(nom))
            .ok()?;
        moteur.SetMuted(true).ok()
    }
}

/// La taille de ses images.
///
/// # Safety
/// Un moteur qui a chargé sa vidéo.
unsafe fn taille(moteur: &IMFMediaEngine) -> Option<(u32, u32)> {
    let (mut l, mut h) = (0, 0);
    unsafe { moteur.GetNativeVideoSize(Some(&mut l), Some(&mut h)) }.ok()?;
    (l > 0 && h > 0).then_some((l, h))
}

/// Un bitmap WIC en mémoire, BGRA, où le moteur peint chaque image.
///
/// # Safety
/// COM démarré sur ce fil.
unsafe fn bitmap((l, h): (u32, u32)) -> windows::core::Result<IWICBitmap> {
    unsafe {
        let wic: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
        wic.CreateBitmap(l, h, &GUID_WICPixelFormat32bppBGRA, WICBitmapCacheOnDemand)
    }
}

/// **Windows sait-il lire une vidéo ?** Media Foundation est là et décode le H.264.
pub(crate) fn lisible_ici() -> bool {
    mf().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROUGE: &[u8] = include_bytes!("../../../tests/donnees/rouge-64x48.mp4");

    /// **Ce que dit le moteur, un fichier joué deux fois** (fiche 175 § 10) : sur le vrai Windows de
    /// la CI, le joueur (qui lit un fichier) ne rendait aucune image, et le retour au début ne
    /// repartait pas, quand des octets en mémoire jouaient jusqu'au bout. Chaque étape se dit dans
    /// la sortie, avec son heure — de quoi trancher : lenteur de l'ouverture, ou moteur figé.
    #[test]
    fn video_windows_1_un_fichier_joue_deux_fois_et_le_dit() {
        if !crate::renderer::video::epreuves::present() {
            return;
        }
        let chemin = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/donnees/rouge-64x48.mp4");
        let debut = Instant::now();
        let lieu = Lieu::de(chemin, None).expect("un lieu");
        let mut lecteur = LecteurVideo::ouvrir(lieu, true).expect("ouverte");
        crate::dire_erreur!("vidéo (Windows) : ouverte en {:?}", debut.elapsed());
        for tour in 1..=2 {
            let (depart, mut images, mut dit) = (Instant::now(), 0, Instant::now());
            lecteur.jouer(true);
            loop {
                match lecteur.suivante(Duration::from_millis(50)) {
                    Ok(Some(_)) => images += 1,
                    Ok(None) => {}
                    Err(fin) => {
                        crate::dire_erreur!(
                            "vidéo (Windows) : tour {tour}, {fin:?} après {:?}, {images} images",
                            depart.elapsed()
                        );
                        assert_eq!(fin, Fin::AuBout);
                        break;
                    }
                }
                if dit.elapsed() >= Duration::from_millis(500) {
                    dit = Instant::now();
                    let m = &lecteur.moteur.0;
                    // SAFETY : le moteur vit autant que le lecteur.
                    let (t, pause, bout, pret) = unsafe {
                        (
                            m.GetCurrentTime(),
                            m.IsPaused(),
                            m.IsEnded(),
                            m.GetReadyState(),
                        )
                    };
                    crate::dire_erreur!(
                        "vidéo (Windows) : tour {tour}, {:?} : {images} images, heure {t}, \
                         pause {}, au bout {}, prête {pret}",
                        depart.elapsed(),
                        pause.as_bool(),
                        bout.as_bool()
                    );
                }
                assert!(
                    depart.elapsed() < Duration::from_secs(20),
                    "tour {tour} figé"
                );
            }
            if tour == 1 {
                let retour = Instant::now();
                assert!(lecteur.revenir_au_debut());
                crate::dire_erreur!("vidéo (Windows) : revenue en {:?}", retour.elapsed());
            }
        }
    }

    /// Revenue au début par un moteur neuf, la vidéo garde le son qu'on lui avait rendu.
    #[test]
    fn video_windows_1_le_son_reste_tel_qu_il_etait_au_retour() {
        if !crate::renderer::video::epreuves::present() {
            return;
        }
        let lecteur = LecteurVideo::ouvrir(Lieu::Octets(ROUGE.into()), true);
        let mut lecteur = lecteur.expect("ouverte");
        lecteur.couper_le_son(false);
        lecteur.jouer(true);
        let debut = Instant::now();
        while !matches!(
            lecteur.suivante(Duration::from_millis(100)),
            Err(Fin::AuBout)
        ) {
            assert!(debut.elapsed() < Duration::from_secs(10), "jamais finie");
        }
        assert!(lecteur.revenir_au_debut());
        // SAFETY : le moteur vit autant que le lecteur.
        assert!(
            !unsafe { lecteur.moteur.0.GetMuted() }.as_bool(),
            "le son recoupé"
        );
    }
}
