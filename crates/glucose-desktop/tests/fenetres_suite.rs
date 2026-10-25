//! **FENETRES-1 — aucune fenêtre de Glucose ne reste sourde** (fiche 82).
//!
//! Une fenêtre de premier niveau dont le fil ne lit jamais ses messages fige ce qui lui parle :
//! le Restart Manager ne pouvait plus fermer Glucose, l'extinction de Windows l'attendait. Le
//! moteur OpenGL de `wgpu-hal` en pose une ; les moteurs que Glucose ouvre d'abord
//! ([`glucose_desktop::present::moteurs`]) n'en posent aucune.
//!
//! Une seule épreuve, dans un processus à elle : toute fenêtre que le processus porte est la
//! sienne, et l'ordre des deux temps — les moteurs de Glucose, puis OpenGL — est tenu.
#![cfg(windows)]

use glucose_desktop::present::moteurs;
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowThreadProcessId, SendMessageTimeoutW, SMTO_NORMAL, WM_NULL,
};

/// Les fenêtres de premier niveau de ce processus.
fn fenetres_du_processus() -> Vec<HWND> {
    unsafe extern "system" fn une(fenetre: HWND, liste: LPARAM) -> BOOL {
        let mut processus = 0u32;
        // SAFETY : `liste` est le vecteur prêté par l'appelant, vivant pendant l'énumération.
        unsafe {
            GetWindowThreadProcessId(fenetre, Some(&mut processus));
            if processus == std::process::id() {
                (*(liste.0 as *mut Vec<HWND>)).push(fenetre);
            }
        }
        BOOL(1)
    }
    let mut liste: Vec<HWND> = Vec::new();
    // SAFETY : le rappel ne garde rien au-delà de l'appel.
    let _ = unsafe { EnumWindows(Some(une), LPARAM(&mut liste as *mut Vec<HWND> as isize)) };
    liste
}

/// **Les fenêtres qui ne répondent pas** à un message vide en une demi-seconde.
fn sourdes() -> Vec<HWND> {
    fenetres_du_processus()
        .into_iter()
        .filter(|&fenetre| {
            let mut rendu = 0usize;
            // SAFETY : un message vide, attendu au plus 500 ms.
            let repondu = unsafe {
                SendMessageTimeoutW(
                    fenetre,
                    WM_NULL,
                    WPARAM(0),
                    LPARAM(0),
                    SMTO_NORMAL,
                    500,
                    Some(&mut rendu),
                )
            };
            repondu.0 == 0
        })
        .collect()
}

#[test]
fn test_fenetres_1_les_moteurs_de_glucose_ne_laissent_aucune_fenetre_sourde() {
    let instance = moteurs::instance(moteurs::MOTEURS[0]);
    let _carte =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()));
    assert!(
        sourdes().is_empty(),
        "les moteurs que Glucose ouvre d'abord laissent une fenêtre sourde"
    );

    // À l'envers : OpenGL, lui, en pose une — là où il offre une carte (un serveur sans pilote
    // OpenGL n'a rien à montrer).
    let opengl = moteurs::instance(wgpu::Backends::GL);
    let offre = pollster::block_on(opengl.request_adapter(&wgpu::RequestAdapterOptions::default()));
    if offre.is_ok() {
        assert!(
            !sourdes().is_empty(),
            "le moteur WGL de wgpu-hal devait laisser sa fenêtre sourde : l'épreuve est-elle aveugle ?"
        );
    }
}
