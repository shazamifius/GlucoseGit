//! La voie de Windows du pavé (fiche 53), sur une fenêtre qui n'est jamais montrée : ni écran,
//! ni main. Un vrai geste du pavé ne se simule pas — Windows ne sait injecter que des doigts
//! d'écran tactile et des stylets —, c'est son écran qui le jugera.

use super::*;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, SendMessageW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};

/// Une fenêtre à −32 000 pixels, jamais montrée.
fn fenetre_cachee() -> HWND {
    let _ = unsafe {
        windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_APARTMENTTHREADED,
        )
    };
    unsafe {
        CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            w!("STATIC"),
            w!("glucose-epreuve-pave"),
            WS_POPUP,
            -32_000,
            -32_000,
            8,
            8,
            None,
            None,
            None,
            None,
        )
    }
    .expect("une fenetre hors de l'ecran")
}

/// La procédure de la fenêtre, et le viewport que la fenêtre garde pour le pavé.
fn sous_classement(fenetre: HWND) -> (isize, usize) {
    unsafe {
        (
            GetWindowLongPtrW(fenetre, GWLP_WNDPROC),
            GetPropW(fenetre, VIEWPORT).0 as usize,
        )
    }
}

/// **Le pavé s'installe, dort hors geste, et se retire entièrement** : sans geste, rien ne
/// réveille la boucle ; détruit, il ne laisse rien pointer sur un viewport mort — ce qui
/// planterait au premier contact du pavé — et la procédure d'origine reprend sa place.
#[test]
fn test_le_pave_s_installe_dort_et_se_retire() {
    let fenetre = fenetre_cachee();
    let d_origine = sous_classement(fenetre).0;
    let reveil: super::super::Reveil = std::sync::Arc::new(|| {});
    let mut pave = installer(fenetre.0 as isize, reveil).expect("Direct Manipulation s'installe");

    assert!(!pave.en_geste(), "hors geste, le pavé dort");
    assert!(pave.avancer().is_empty());
    let (posee, garde) = sous_classement(fenetre);
    assert_eq!(
        posee, procedure as *const () as isize,
        "notre procédure est devant"
    );
    assert_eq!(
        garde, &*pave.viewport as *const IDirectManipulationViewport as usize,
        "la fenêtre garde l'adresse du viewport"
    );

    // Un contact qui n'est pas celui d'un pavé suit son chemin, jusqu'à la procédure d'origine.
    let _ = unsafe {
        SendMessageW(
            fenetre,
            DM_POINTERHITTEST,
            Some(WPARAM(4242)),
            Some(LPARAM(0)),
        )
    };

    drop(pave);
    assert_eq!(
        sous_classement(fenetre),
        (d_origine, 0),
        "la procédure d'origine reprend sa place, et plus rien ne pointe sur le viewport"
    );
    let _ = unsafe { DestroyWindow(fenetre) };
}

/// **Toute la chaîne, sans pavé** : le viewport, zoomé par programme d'un facteur deux, rend sa
/// transformation à l'écouteur pendant `Update`, et la lecture en tire une octave. Puis le
/// système passe à `READY` : l'écouteur remet le viewport à l'identité — et cette remise **ne se
/// lit pas** comme un dézoom qui défairait le geste. Les rappels du système passent par la file
/// de messages du fil : la boucle de `winit` la vide, l'épreuve aussi.
#[test]
fn test_la_chaine_rend_le_geste_puis_se_remet_sans_le_defaire() {
    use windows::Win32::Graphics::DirectManipulation::IDirectManipulationContent;
    let fenetre = fenetre_cachee();
    let reveil: super::super::Reveil = std::sync::Arc::new(|| {});
    let mut pave = installer(fenetre.0 as isize, reveil).expect("Direct Manipulation s'installe");
    unsafe { pave.viewport.ZoomToRect(0.0, 0.0, 500.0, 500.0, false) }.expect("le zoom");
    let mut rendus = Vec::new();
    for _ in 0..4 {
        pomper();
        rendus.extend(pave.avancer());
    }
    assert_eq!(
        rendus,
        vec![Mouvement::Zoomer(1.0)],
        "une octave, et rien d'autre"
    );
    assert_eq!(
        pave.etat.borrow().lecteur,
        Lecteur::default(),
        "la lecture est remise"
    );
    let contenu: IDirectManipulationContent =
        unsafe { pave.viewport.GetPrimaryContent() }.expect("le contenu");
    let mut m = [0f32; 6];
    unsafe { contenu.GetContentTransform(&mut m) }.expect("la transformation");
    assert_eq!(
        m,
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        "le viewport est revenu à l'identité"
    );
    drop(pave);
    let _ = unsafe { DestroyWindow(fenetre) };
}

/// Vide la file de messages du fil, comme la boucle de `winit` le fait.
fn pomper() {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };
    let mut m = MSG::default();
    while unsafe { PeekMessageW(&mut m, None, 0, 0, PM_REMOVE) }.as_bool() {
        unsafe {
            let _ = TranslateMessage(&m);
            DispatchMessageW(&m);
        }
    }
}

#[test]
fn test_essai_les_bords() {
    use windows::Win32::Graphics::DirectManipulation::IDirectManipulationContent;
    let fenetre = fenetre_cachee();
    let reveil: super::super::Reveil = std::sync::Arc::new(|| {});
    let mut pave = installer(fenetre.0 as isize, reveil).expect("installe");
    let contenu: IDirectManipulationContent = unsafe { pave.viewport.GetPrimaryContent() }.unwrap();
    let r = unsafe { contenu.GetContentRect() };
    println!("contenu : {r:?}");
    for (x, y) in [
        (300.0f32, 0.0f32),
        (-300.0, 0.0),
        (0.0, 250.0),
        (-200.0, -200.0),
    ] {
        let s = unsafe {
            pave.viewport
                .ZoomToRect(x, y, x + 1000.0, y + 1000.0, false)
        };
        let mut rendus = Vec::new();
        for _ in 0..3 {
            pomper();
            rendus.extend(pave.avancer());
        }
        let mut m = [0f32; 6];
        unsafe { contenu.GetContentTransform(&mut m) }.unwrap();
        println!("ScrollTo({x},{y}) -> {s:?} ; transformation {m:?} ; rendus {rendus:?}");
    }
    drop(pave);
    let _ = unsafe { DestroyWindow(fenetre) };
}
