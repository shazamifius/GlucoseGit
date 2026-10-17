//! **La porte sous Windows** : un tuyau nommé (*named pipe*), le canal local que Windows réserve
//! aux processus d'une même machine — et, ici, d'un même compte (AGENTS-2).
//!
//! * Il ne s'ouvre qu'au compte qui l'a créé : son descripteur de sécurité ne nomme que lui
//!   (`D:P(A;;GA;;;<son SID>)`), au lieu de la lecture laissée à tous par défaut ;
//! * il refuse les clients du réseau (`PIPE_REJECT_REMOTE_CLIENTS`) ;
//! * sa première instance exige que le nom soit libre (`FILE_FLAG_FIRST_PIPE_INSTANCE`) : un
//!   second Glucose ouvert ne prend pas la porte du premier ;
//! * et le relais, en se connectant, vérifie que le processus qui tient le tuyau est du même
//!   compte que lui ([`connecter`]) : un autre compte qui aurait pris le nom le premier ne
//!   recevrait rien de ce qu'un agent écrit.
//!
//! Une instance par connexion : sitôt un agent branché, la suivante attend le prochain.

use crate::agents::Boite;
use std::fs::File;
use std::os::windows::io::FromRawHandle;
use windows::core::{HSTRING, PWSTR};
use windows::Win32::Foundation::{
    CloseHandle, LocalFree, ERROR_PIPE_CONNECTED, HANDLE, HLOCAL, INVALID_HANDLE_VALUE,
};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{
    GetTokenInformation, TokenUser, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY,
    TOKEN_USER,
};
use windows::Win32::Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, GetNamedPipeServerProcessId, PIPE_READMODE_BYTE,
    PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};

/// Ce que le tampon d'une instance garde dans chaque sens. Une indication au système, qui
/// l'agrandit au besoin : elle ne borne pas un message.
const TAMPON: u32 = 64 * 1024;

/// **La porte ouverte** : le fil qui écoute vit autant que Glucose.
pub struct Porte {
    nom: String,
}

impl Porte {
    /// **Ouvre la porte** sous ce nom — `None` si un autre la tient déjà, ou si Windows refuse.
    pub fn ouvrir(nom: &str, boite: Boite) -> Option<Self> {
        let sid = sid_du_processus(unsafe { GetCurrentProcess() })?;
        let premiere = creer_une_instance(nom, &sid, true)?;
        let (n, s) = (nom.to_string(), sid);
        // Une poignée n'est qu'un nombre pour passer d'un fil à l'autre ; elle n'appartient
        // qu'au fil qui écoute.
        let premiere = premiere.0 as isize;
        std::thread::Builder::new()
            .name("glucose-porte".into())
            .spawn(move || ecouter(&n, &s, HANDLE(premiere as *mut _), &boite))
            .ok()?;
        Some(Self {
            nom: nom.to_string(),
        })
    }

    pub fn nom(&self) -> &str {
        &self.nom
    }
}

/// **Le nom de la porte de ce compte** : un par utilisateur de la machine.
pub fn nom_par_defaut() -> Option<String> {
    let sid = sid_du_processus(unsafe { GetCurrentProcess() })?;
    Some(format!(r"\\.\pipe\glucose-agents-{sid}"))
}

/// Attend un agent sur chaque instance, et le sert sur un fil à lui.
fn ecouter(nom: &str, sid: &str, mut instance: HANDLE, boite: &Boite) {
    loop {
        let branchee = match unsafe { ConnectNamedPipe(instance, None) } {
            Ok(()) => true,
            // Le client est arrivé entre la création et l'attente : il est bien là.
            Err(e) => e.code() == ERROR_PIPE_CONNECTED.to_hresult(),
        };
        let suivante = creer_une_instance(nom, sid, false);
        if branchee {
            // SAFETY : l'instance est à nous seuls, et le fichier la fermera.
            let flux = unsafe { File::from_raw_handle(instance.0) };
            let b = boite.clone();
            let _ = std::thread::Builder::new()
                .name("glucose-agent".into())
                .spawn(move || super::servir(&flux, &flux, &b));
        } else {
            let _ = unsafe { CloseHandle(instance) };
        }
        match suivante {
            Some(h) => instance = h,
            None => return,
        }
    }
}

/// Une instance du tuyau, qui ne s'ouvre qu'à ce compte.
fn creer_une_instance(nom: &str, sid: &str, premiere: bool) -> Option<HANDLE> {
    let sddl = HSTRING::from(format!("D:P(A;;GA;;;{sid})"));
    let mut descripteur = PSECURITY_DESCRIPTOR::default();
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            &sddl,
            SDDL_REVISION_1,
            &mut descripteur,
            None,
        )
    }
    .ok()?;
    let attributs = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descripteur.0,
        bInheritHandle: false.into(),
    };
    let mut ouverture = PIPE_ACCESS_DUPLEX;
    if premiere {
        ouverture |= FILE_FLAG_FIRST_PIPE_INSTANCE;
    }
    let mode = PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS;
    let h = unsafe {
        CreateNamedPipeW(
            &HSTRING::from(nom),
            ouverture,
            mode,
            PIPE_UNLIMITED_INSTANCES,
            TAMPON,
            TAMPON,
            0,
            Some(&attributs),
        )
    };
    unsafe { LocalFree(Some(HLOCAL(descripteur.0))) };
    (h != INVALID_HANDLE_VALUE && !h.is_invalid()).then_some(h)
}

/// **Se connecte à la porte**, et vérifie que celui qui la tient est de ce compte (AGENTS-2).
pub fn connecter(nom: &str) -> std::io::Result<super::tuyau::Tuyau> {
    let flux = super::tuyau::Tuyau::ouvrir(nom)?;
    let mut pid = 0u32;
    unsafe { GetNamedPipeServerProcessId(flux.poignee(), &mut pid) }
        .map_err(std::io::Error::other)?;
    let serveur = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
        .map_err(std::io::Error::other)?;
    let le_sien = sid_du_processus(serveur);
    let _ = unsafe { CloseHandle(serveur) };
    let le_mien = sid_du_processus(unsafe { GetCurrentProcess() });
    if le_sien.is_none() || le_sien != le_mien {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "la porte est tenue par un autre compte de cette machine",
        ));
    }
    Ok(flux)
}

/// **Le compte d'un processus**, en texte (`S-1-5-21-…`).
fn sid_du_processus(processus: HANDLE) -> Option<String> {
    let mut jeton = HANDLE::default();
    unsafe { OpenProcessToken(processus, TOKEN_QUERY, &mut jeton) }.ok()?;
    let mut taille = 0u32;
    let _ = unsafe { GetTokenInformation(jeton, TokenUser, None, 0, &mut taille) };
    // Des mots de huit octets : la structure porte un pointeur, qu'il faut lire aligné.
    let mut tampon = vec![0u64; (taille as usize).div_ceil(8)];
    let lu = unsafe {
        GetTokenInformation(
            jeton,
            TokenUser,
            Some(tampon.as_mut_ptr().cast()),
            taille,
            &mut taille,
        )
    };
    let _ = unsafe { CloseHandle(jeton) };
    lu.ok()?;
    // SAFETY : Windows vient d'écrire une `TOKEN_USER` au début du tampon, aligné.
    let utilisateur = unsafe { &*tampon.as_ptr().cast::<TOKEN_USER>() };
    let mut texte = PWSTR::null();
    unsafe { ConvertSidToStringSidW(utilisateur.User.Sid, &mut texte) }.ok()?;
    let sid = unsafe { texte.to_string() }.ok();
    unsafe { LocalFree(Some(HLOCAL(texte.0.cast()))) };
    sid
}
