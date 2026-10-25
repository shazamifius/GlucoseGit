//! **Windows finit la session** (FIN-SESSION-1) — la voie de Windows de [`super`].
//!
//! # Ce que Windows fait
//!
//! Il demande d'abord (`WM_QUERYENDSESSION`) : `DefWindowProc` répond oui, et Glucose ne retient
//! jamais l'extinction — il n'a rien à sauver qui ne soit déjà sur le disque, et Microsoft le
//! demande (« Applications should not block shutdown »). Puis il annonce (`WM_ENDSESSION`,
//! `wParam` vrai) : la session peut finir **dès que toutes les applications ont rendu la main**
//! sur ce message. `ENDSESSION_CLOSEAPP` y dit le Restart Manager — un installeur qui doit
//! remplacer un fichier, Windows Update : l'application doit alors se fermer, en sauvant sans
//! rien demander.
//!
//! # Comment Glucose finit pendant le message
//!
//! La procédure se glisse devant celle de `winit` (la forme du pavé, `pave_windows`), **marque
//! la fenêtre**, puis passe un `WM_CLOSE` à la procédure qui était là avant elle. `winit` le
//! traduit en `CloseRequested` et appelle l'application **avant de rendre la main** quand elle
//! n'est pas déjà occupée (`EventLoopRunner::send_event`) ; l'application lit la marque, et
//! finit sans poser la question du travail non enregistré — un brouillon reste sur le disque, et
//! le lancement suivant le rouvre. Si l'application était occupée (une boucle modale qui pompe
//! les messages), `winit` garde la demande pour plus tard : le travail, lui, est déjà écrit.
//!
//! La marque est une **propriété de la fenêtre** : aucun état à côté, rien qui survive à la
//! fenêtre, et deux fenêtres d'épreuve ne se voient pas.
//!
//! # `unsafe`, et où il s'arrête
//!
//! Tout l'appel à Windows est ici ; ce que l'application fait de la marque ne l'est pas.

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, GetPropW, GetWindowLongPtrW, RemovePropW, SetPropW, SetWindowLongPtrW,
    GWLP_WNDPROC, WM_CLOSE, WM_ENDSESSION, WM_NCDESTROY, WNDPROC,
};

/// La procédure qui était là avant la nôtre — celle de `winit`, ou celle du pavé.
const ANCIENNE: PCWSTR = w!("Glucose.Fin.Procedure");
/// La marque : Windows a annoncé la fin de la session.
const FINIT: PCWSTR = w!("Glucose.Fin.Session");

fn poignee(fenetre: isize) -> HWND {
    HWND(fenetre as *mut _)
}

/// **Glisse notre procédure devant celle qui est en place.** La propriété d'abord : la
/// procédure doit toujours trouver à qui rendre la main. Rend `false` si Windows refuse.
pub(super) fn veiller(fenetre: isize) -> bool {
    let fenetre = poignee(fenetre);
    // SAFETY : `fenetre` est une fenêtre vivante de ce fil (celle de `winit`, ou une fenêtre
    // d'épreuve) ; la procédure posée ne lit que des propriétés qu'elle retire avec elle.
    unsafe {
        let actuelle = GetWindowLongPtrW(fenetre, GWLP_WNDPROC);
        if actuelle == 0 || SetPropW(fenetre, ANCIENNE, Some(HANDLE(actuelle as *mut _))).is_err() {
            return false;
        }
        SetWindowLongPtrW(fenetre, GWLP_WNDPROC, procedure as *const () as isize) != 0
    }
}

/// **La fenêtre porte-t-elle la marque ?**
pub(super) fn la_session_finit(fenetre: isize) -> bool {
    // SAFETY : lire une propriété n'exige qu'une poignée ; morte, elle ne porte rien.
    !unsafe { GetPropW(poignee(fenetre), FINIT) }.0.is_null()
}

/// **La fin de la session marque la fenêtre, puis demande à `winit` de la fermer** ; tout le
/// reste suit son chemin. Au dernier message de la fenêtre, nos propriétés partent avec elle
/// (la règle de `SetPropW`), et la procédure d'avant reprend sa place si nous sommes devant.
unsafe extern "system" fn procedure(
    fenetre: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let brute = unsafe { GetPropW(fenetre, ANCIENNE) }.0;
    // SAFETY : la propriété porte la procédure qui était là avant nous ; elle n'en part qu'au
    // dernier message, après lequel plus rien n'arrive.
    let ancienne: WNDPROC =
        unsafe { std::mem::transmute::<*mut core::ffi::c_void, WNDPROC>(brute) };
    match message {
        // `wParam` faux : l'extinction est annulée, rien ne finit.
        WM_ENDSESSION if wparam.0 != 0 => unsafe {
            let _ = SetPropW(fenetre, FINIT, Some(HANDLE(std::ptr::dangling_mut())));
            CallWindowProcW(ancienne, fenetre, WM_CLOSE, WPARAM(0), LPARAM(0));
        },
        WM_NCDESTROY => unsafe {
            if GetWindowLongPtrW(fenetre, GWLP_WNDPROC) == procedure as *const () as isize {
                SetWindowLongPtrW(fenetre, GWLP_WNDPROC, brute as isize);
            }
            let _ = RemovePropW(fenetre, FINIT);
            let _ = RemovePropW(fenetre, ANCIENNE);
        },
        _ => {}
    }
    unsafe { CallWindowProcW(ancienne, fenetre, message, wparam, lparam) }
}
