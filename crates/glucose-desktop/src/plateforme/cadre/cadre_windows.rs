//! **La voie de Windows de [`super`]** : la procédure glissée devant celle de `winit`, comme
//! `fin/session_windows`. Tout l'appel à Windows est ici ; ce qu'est un point de la bande se
//! décide dans [`super::Zones::sous`], éprouvé partout.

use super::{Zone, Zones};
use std::sync::Mutex;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, GetPropW, GetWindowLongPtrW, IsZoomed, RemovePropW, SetPropW,
    SetWindowLongPtrW, SetWindowPos, GWLP_WNDPROC, HTBOTTOM, HTBOTTOMLEFT, HTBOTTOMRIGHT,
    HTCAPTION, HTCLIENT, HTLEFT, HTRIGHT, HTTOP, HTTOPLEFT, HTTOPRIGHT, NCCALCSIZE_PARAMS,
    SM_CXPADDEDBORDER, SM_CYSIZEFRAME, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_NOZORDER, WM_NCCALCSIZE, WM_NCDESTROY, WM_NCHITTEST, WNDPROC,
};

/// La procédure qui était là avant la nôtre.
const ANCIENNE: PCWSTR = w!("Glucose.Cadre.Procedure");

/// Ce que la bande occupe. Windows pose sa question sur le fil de la fenêtre, qui dessine aussi :
/// le verrou n'attend jamais personne.
static ZONES: Mutex<Zones> = Mutex::new(Zones {
    bande: 0,
    tenus: Vec::new(),
});

pub(super) fn poser(zones: Zones) {
    if let Ok(mut z) = ZONES.lock() {
        *z = zones;
    }
}

/// **Glisse la procédure, puis demande à Windows de recalculer le cadre.** Rend `false` si
/// Windows refuse : la barre du système reste, et rien d'autre ne change.
pub(super) fn habiller(fenetre: isize) -> bool {
    let fenetre = HWND(fenetre as *mut _);
    // SAFETY : `fenetre` est la fenêtre vivante de ce fil ; la procédure ne lit que la propriété
    // qu'elle retire avec elle.
    unsafe {
        let actuelle = GetWindowLongPtrW(fenetre, GWLP_WNDPROC);
        if actuelle == 0 || SetPropW(fenetre, ANCIENNE, Some(HANDLE(actuelle as *mut _))).is_err() {
            return false;
        }
        if SetWindowLongPtrW(fenetre, GWLP_WNDPROC, procedure as *const () as isize) == 0 {
            return false;
        }
        SetWindowPos(
            fenetre,
            None,
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
        .is_ok()
    }
}

/// Ce que Windows ajoute autour d'une fenêtre : la bordure qui dépasse de l'écran, agrandie.
fn bordure(fenetre: HWND) -> i32 {
    // SAFETY : lire une mesure du système n'exige qu'une poignée vivante.
    unsafe {
        let dpi = GetDpiForWindow(fenetre);
        GetSystemMetricsForDpi(SM_CYSIZEFRAME, dpi) + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi)
    }
}

unsafe extern "system" fn procedure(
    fenetre: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let brute = unsafe { GetPropW(fenetre, ANCIENNE) }.0;
    // SAFETY : la propriété porte la procédure d'avant, et n'en part qu'au dernier message.
    let ancienne: WNDPROC =
        unsafe { std::mem::transmute::<*mut core::ffi::c_void, WNDPROC>(brute) };
    let suite = || unsafe { CallWindowProcW(ancienne, fenetre, message, wparam, lparam) };
    match message {
        // Windows retire les bordures et la barre ; on lui rend le haut, qu'il garde tel quel.
        // Agrandie, la fenêtre dépasse de l'écran de sa bordure : sa bande doit rester visible.
        WM_NCCALCSIZE if wparam.0 != 0 => unsafe {
            let parametres = lparam.0 as *mut NCCALCSIZE_PARAMS;
            let haut = (*parametres).rgrc[0].top;
            let rendu = suite();
            if rendu.0 == 0 {
                let retrait = if IsZoomed(fenetre).as_bool() {
                    bordure(fenetre)
                } else {
                    0
                };
                (*parametres).rgrc[0].top = haut + retrait;
            }
            rendu
        },
        WM_NCHITTEST => {
            let defaut = suite();
            // Les bords des côtés et du bas restent ceux de Windows.
            let u = defaut.0 as u32;
            if [
                HTLEFT,
                HTRIGHT,
                HTBOTTOM,
                HTBOTTOMLEFT,
                HTBOTTOMRIGHT,
                HTTOPLEFT,
                HTTOPRIGHT,
            ]
            .contains(&u)
            {
                return defaut;
            }
            let mut point = POINT {
                x: i32::from(lparam.0 as u16 as i16),
                y: i32::from((lparam.0 >> 16) as u16 as i16),
            };
            // SAFETY : un point à nous, et la fenêtre vivante qui reçoit ce message.
            if !unsafe { ScreenToClient(fenetre, &raw mut point) }.as_bool() {
                return defaut;
            }
            let agrandie = unsafe { IsZoomed(fenetre) }.as_bool();
            let zone = ZONES.lock().map_or(Zone::Client, |z| {
                z.sous(point.x, point.y, agrandie, bordure(fenetre))
            });
            LRESULT(match zone {
                Zone::BordDuHaut => HTTOP,
                Zone::Deplace => HTCAPTION,
                Zone::Client => HTCLIENT,
            } as isize)
        }
        WM_NCDESTROY => unsafe {
            if GetWindowLongPtrW(fenetre, GWLP_WNDPROC) == procedure as *const () as isize {
                SetWindowLongPtrW(fenetre, GWLP_WNDPROC, brute as isize);
            }
            let _ = RemovePropW(fenetre, ANCIENNE);
            CallWindowProcW(ancienne, fenetre, message, wparam, lparam)
        },
        _ => suite(),
    }
}
