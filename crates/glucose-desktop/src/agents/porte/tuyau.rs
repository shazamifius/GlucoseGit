//! **Le bout du relais, sous Windows : un tuyau en E/S superposée** (*overlapped*).
//!
//! # TUYAU-1 — lire et écrire en même temps ne se bloquent pas
//!
//! Le relais lit les réponses de Glucose sur un fil pendant qu'il écrit les requêtes de l'agent
//! sur un autre. Sur un tuyau ouvert en mode **synchrone**, Windows sérialise les opérations d'un
//! même objet fichier : l'écriture attend que la lecture en cours finisse — qui attend une
//! réponse à la requête qu'on n'a pas pu écrire. Mesuré le 17/10 : la découverte passait, et
//! `tools/list` n'arrivait jamais. Ouvert avec `FILE_FLAG_OVERLAPPED`, chaque opération porte sa
//! propre `OVERLAPPED` et son événement : une lecture en attente n'empêche plus une écriture. Le
//! fil qui appelle attend sa propre opération (`GetOverlappedResult`), et rien d'autre.

use std::io::{Read, Write};
use std::sync::Arc;
use windows::core::HSTRING;
use windows::Win32::Foundation::{
    CloseHandle, ERROR_BROKEN_PIPE, ERROR_IO_PENDING, ERROR_PIPE_BUSY, GENERIC_READ, GENERIC_WRITE,
    HANDLE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_FLAG_OVERLAPPED, FILE_SHARE_NONE, OPEN_EXISTING,
};
use windows::Win32::System::Pipes::WaitNamedPipeW;
use windows::Win32::System::Threading::CreateEventW;
use windows::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};

/// Une poignée, fermée quand le dernier bout du tuyau part.
struct Poignee(HANDLE);

// SAFETY : une poignée Windows n'est qu'un nombre que le noyau interprète ; ouverte en E/S
// superposée, plusieurs fils peuvent y opérer en même temps (TUYAU-1).
unsafe impl Send for Poignee {}
unsafe impl Sync for Poignee {}

impl Drop for Poignee {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

/// **Un bout du tuyau** : on le clone pour lire sur un fil et écrire sur l'autre.
#[derive(Clone)]
pub struct Tuyau(Arc<Poignee>);

impl Tuyau {
    /// Ouvre la porte `nom` ; attend une instance libre si toutes sont prises.
    pub(super) fn ouvrir(nom: &str) -> std::io::Result<Self> {
        let nom = HSTRING::from(nom);
        loop {
            let h = unsafe {
                CreateFileW(
                    &nom,
                    (GENERIC_READ | GENERIC_WRITE).0,
                    FILE_SHARE_NONE,
                    None,
                    OPEN_EXISTING,
                    FILE_FLAG_OVERLAPPED,
                    None,
                )
            };
            match h {
                Ok(h) => return Ok(Self(Arc::new(Poignee(h)))),
                // Toutes les instances sont prises : la suivante naît dès qu'un agent se branche.
                Err(e) if e.code() == ERROR_PIPE_BUSY.to_hresult() => {
                    let _ = unsafe { WaitNamedPipeW(&nom, 2_000) };
                }
                Err(e) => return Err(std::io::Error::from_raw_os_error(e.code().0 & 0xFFFF)),
            }
        }
    }

    pub fn try_clone(&self) -> std::io::Result<Self> {
        Ok(self.clone())
    }

    pub(super) fn poignee(&self) -> HANDLE {
        self.0 .0
    }

    /// Une opération, et l'attente de **sa** fin seulement.
    fn operer(
        &self,
        f: impl FnOnce(HANDLE, *mut OVERLAPPED) -> windows::core::Result<()>,
    ) -> std::io::Result<usize> {
        let evenement =
            unsafe { CreateEventW(None, true, false, None) }.map_err(std::io::Error::other)?;
        let mut ov = OVERLAPPED {
            hEvent: evenement,
            ..Default::default()
        };
        let h = self.poignee();
        let issue = match f(h, &mut ov) {
            Err(e) if e.code() != ERROR_IO_PENDING.to_hresult() => Err(e),
            _ => {
                let mut n = 0u32;
                unsafe { GetOverlappedResult(h, &ov, &mut n, true) }.map(|()| n as usize)
            }
        };
        let _ = unsafe { CloseHandle(evenement) };
        match issue {
            Ok(n) => Ok(n),
            // L'autre bout s'est fermé : c'est la fin du flux, pas une faute.
            Err(e) if e.code() == ERROR_BROKEN_PIPE.to_hresult() => Ok(0),
            Err(e) => Err(std::io::Error::other(e)),
        }
    }
}

impl Read for Tuyau {
    fn read(&mut self, tampon: &mut [u8]) -> std::io::Result<usize> {
        self.operer(|h, ov| unsafe { ReadFile(h, Some(tampon), None, Some(ov)) })
    }
}

impl Write for Tuyau {
    fn write(&mut self, octets: &[u8]) -> std::io::Result<usize> {
        self.operer(|h, ov| unsafe { WriteFile(h, Some(octets), None, Some(ov)) })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
