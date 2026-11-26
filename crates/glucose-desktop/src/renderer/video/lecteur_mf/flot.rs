//! **Les octets d'une vidéo, tels que Media Foundation les lit** : un `IStream` en lecture seule
//! sur un [`Lieu`] — un fichier entier, la tranche d'un document, ou la mémoire —, lu là où il
//! est : aucune copie sur le disque, un seul chemin pour les trois, comme l'`appsrc` de GStreamer.

use crate::renderer::video::{signature, Lieu};
use std::ffi::c_void;
use std::fs::File;
use std::os::windows::fs::FileExt;
use std::sync::{Arc, Mutex, PoisonError};
use windows::core::{implement, Error, Ref, Result, HRESULT};
use windows::Win32::Foundation::{
    E_NOTIMPL, STG_E_ACCESSDENIED, STG_E_INVALIDFUNCTION, STG_E_INVALIDPOINTER, STG_E_READFAULT,
    S_FALSE, S_OK,
};
use windows::Win32::System::Com::{
    ISequentialStream_Impl, IStream, IStream_Impl, LOCKTYPE, STATFLAG, STATSTG, STGC, STGM_READ,
    STGTY_STREAM, STREAM_SEEK, STREAM_SEEK_CUR, STREAM_SEEK_END, STREAM_SEEK_SET,
};

/// **Un `IStream` sur les octets d'une vidéo**, de `0` à `longueur`.
#[implement(IStream)]
pub(super) struct Flot {
    octets: Octets,
    longueur: u64,
    position: Mutex<u64>,
}

enum Octets {
    Fichier { fichier: File, debut: u64 },
    Memoire(Arc<[u8]>),
}

impl Octets {
    /// Lit à partir de `a` tant qu'il y a de la place et des octets ; rend le nombre lu.
    fn lire(&self, a: u64, tampon: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Memoire(octets) => {
                let reste = octets.get(a as usize..).unwrap_or_default();
                let n = tampon.len().min(reste.len());
                tampon[..n].copy_from_slice(&reste[..n]);
                Ok(n)
            }
            Self::Fichier { fichier, debut } => {
                let mut lus = 0;
                while lus < tampon.len() {
                    match fichier.seek_read(&mut tampon[lus..], debut + a + lus as u64)? {
                        0 => break,
                        n => lus += n,
                    }
                }
                Ok(lus)
            }
        }
    }
}

impl Flot {
    /// **Le flot de ce lieu**, et le nom qui dit au moteur ce qu'il contient : `video.mp4`, son
    /// extension tirée de sa signature — Media Foundation choisit son démultiplexeur par elle.
    pub(super) fn ouvrir(lieu: &Lieu) -> Option<(IStream, String)> {
        let (octets, longueur) = match lieu {
            Lieu::Fichier {
                chemin,
                debut,
                longueur,
            } => {
                let fichier = File::open(chemin).ok()?;
                let longueur = match longueur {
                    Some(l) => *l,
                    None => fichier.metadata().ok()?.len().saturating_sub(*debut),
                };
                let debut = *debut;
                (Octets::Fichier { fichier, debut }, longueur)
            }
            Lieu::Octets(octets) => (Octets::Memoire(octets.clone()), octets.len() as u64),
        };
        let mut tete = [0; signature::LONGUEUR];
        let lus = octets.lire(0, &mut tete).ok()?.min(longueur as usize);
        let nom = format!(
            "video.{}",
            signature::extension(&tete[..lus]).unwrap_or("mp4")
        );
        let flot = Flot {
            octets,
            longueur,
            position: Mutex::new(0),
        };
        Some((flot.into(), nom))
    }
}

impl ISequentialStream_Impl for Flot_Impl {
    fn Read(&self, pv: *mut c_void, cb: u32, pcbread: *mut u32) -> HRESULT {
        let mut position = self.position.lock().unwrap_or_else(PoisonError::into_inner);
        let n = u64::from(cb).min(self.longueur.saturating_sub(*position)) as usize;
        if pv.is_null() && n > 0 {
            return STG_E_INVALIDPOINTER;
        }
        // SAFETY : l'appelant prête `cb` octets à `pv`, et `n` ≤ `cb`.
        let tampon = match n {
            0 => &mut [][..],
            _ => unsafe { std::slice::from_raw_parts_mut(pv.cast::<u8>(), n) },
        };
        let Ok(lus) = self.octets.lire(*position, tampon) else {
            return STG_E_READFAULT;
        };
        *position += lus as u64;
        if !pcbread.is_null() {
            // SAFETY : non nul, il attend le nombre lu.
            unsafe { *pcbread = lus as u32 };
        }
        // Moins que demandé : la fin du flot, que `S_FALSE` dit.
        if lus == cb as usize {
            S_OK
        } else {
            S_FALSE
        }
    }

    fn Write(&self, _: *const c_void, _: u32, _: *mut u32) -> HRESULT {
        STG_E_ACCESSDENIED
    }
}

impl IStream_Impl for Flot_Impl {
    fn Seek(&self, deplacement: i64, origine: STREAM_SEEK, nouvelle: *mut u64) -> Result<()> {
        let mut position = self.position.lock().unwrap_or_else(PoisonError::into_inner);
        let depart = match origine {
            STREAM_SEEK_SET => 0,
            STREAM_SEEK_CUR => *position,
            STREAM_SEEK_END => self.longueur,
            _ => return Err(STG_E_INVALIDFUNCTION.into()),
        };
        let cible = depart
            .checked_add_signed(deplacement)
            .ok_or(Error::from(STG_E_INVALIDFUNCTION))?;
        *position = cible;
        if !nouvelle.is_null() {
            // SAFETY : non nul, il attend la nouvelle position.
            unsafe { *nouvelle = cible };
        }
        Ok(())
    }

    /// Sa longueur, que Media Foundation lit pour connaître le flot.
    fn Stat(&self, stat: *mut STATSTG, _: &STATFLAG) -> Result<()> {
        if stat.is_null() {
            return Err(STG_E_INVALIDPOINTER.into());
        }
        // SAFETY : non nul, il attend la description du flot ; sans nom, rien à libérer.
        unsafe {
            *stat = STATSTG {
                r#type: STGTY_STREAM.0 as u32,
                cbSize: self.longueur,
                grfMode: STGM_READ,
                ..Default::default()
            };
        }
        Ok(())
    }

    fn SetSize(&self, _: u64) -> Result<()> {
        Err(STG_E_ACCESSDENIED.into())
    }

    fn CopyTo(&self, _: Ref<IStream>, _: u64, _: *mut u64, _: *mut u64) -> Result<()> {
        Err(E_NOTIMPL.into())
    }

    fn Commit(&self, _: &STGC) -> Result<()> {
        Ok(())
    }

    fn Revert(&self) -> Result<()> {
        Ok(())
    }

    fn LockRegion(&self, _: u64, _: u64, _: &LOCKTYPE) -> Result<()> {
        Err(E_NOTIMPL.into())
    }

    fn UnlockRegion(&self, _: u64, _: u64, _: u32) -> Result<()> {
        Err(E_NOTIMPL.into())
    }

    fn Clone(&self) -> Result<IStream> {
        Err(E_NOTIMPL.into())
    }
}
