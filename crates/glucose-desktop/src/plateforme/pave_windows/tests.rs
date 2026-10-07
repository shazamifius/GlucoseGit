//! La voie de Windows du pavé (fiches 53 et 54), sur une fenêtre qui n'est jamais montrée : ni
//! écran, ni main. Un vrai geste du pavé ne se simule pas — Windows ne sait injecter que des
//! doigts d'écran tactile et des stylets —, c'est son écran qui le jugera.

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

/// Installe le pavé sur une fenêtre cachée, avec un viewport de 1000 × 1000.
fn pave_cache() -> (HWND, PaveWindows) {
    let fenetre = fenetre_cachee();
    let reveil: super::super::Reveil = std::sync::Arc::new(|| {});
    let pave = installer(fenetre.0 as isize, (1000, 1000), reveil)
        .expect("Direct Manipulation s'installe");
    (fenetre, pave)
}

/// La procédure de la fenêtre, et le fil que la fenêtre garde pour le pavé.
fn sous_classement(fenetre: HWND) -> (isize, usize) {
    unsafe {
        (
            GetWindowLongPtrW(fenetre, GWLP_WNDPROC),
            GetPropW(fenetre, FIL).0 as usize,
        )
    }
}

/// **Le pavé s'installe, dort hors geste, et se retire entièrement** : sans geste, rien ne
/// réveille la boucle ; détruit, il ne laisse rien pointer sur un fil mort — ce qui planterait
/// au premier contact du pavé — et la procédure d'origine reprend sa place.
#[test]
fn test_le_pave_s_installe_dort_et_se_retire() {
    let fenetre = fenetre_cachee();
    let d_origine = sous_classement(fenetre).0;
    let reveil: super::super::Reveil = std::sync::Arc::new(|| {});
    let mut pave =
        installer(fenetre.0 as isize, (1000, 1000), reveil).expect("Direct Manipulation");

    assert!(!pave.en_geste(), "hors geste, le pavé dort");
    assert!(pave.avancer().is_empty());
    let (posee, garde) = sous_classement(fenetre);
    assert_eq!(
        posee, procedure as *const () as isize,
        "notre procédure est devant"
    );
    assert_eq!(
        garde, &*pave.fil as *const Fil as usize,
        "la fenêtre garde l'adresse du fil"
    );

    // Un contact qui n'est pas celui d'un pavé suit son chemin, jusqu'à la procédure d'origine,
    // et ne laisse aucun signe : il n'a pas été confié au système.
    let _ = pave.signes();
    let _ = unsafe {
        SendMessageW(
            fenetre,
            DM_POINTERHITTEST,
            Some(WPARAM(4242)),
            Some(LPARAM(0)),
        )
    };
    assert!(
        pave.signes().iter().all(|s| s.quoi != "contact"),
        "un contact d'autre chose qu'un pavé ne se confie pas"
    );

    drop(pave);
    assert_eq!(
        sous_classement(fenetre),
        (d_origine, 0),
        "la procédure d'origine reprend sa place, et plus rien ne pointe sur le fil"
    );
    let _ = unsafe { DestroyWindow(fenetre) };
}

/// **Toute la chaîne, sans pavé** : le viewport, zoomé par programme d'un facteur deux vers son
/// coin (0, 0), rend sa transformation pendant `Update`, et la lecture en tire **une**
/// similitude : `r = 2`, `b = 0` — le coin reste fixe. Puis le système passe à `READY` :
/// l'écouteur remet le viewport à l'identité, et cette remise **ne se lit pas** comme un
/// mouvement qui défairait le geste. Chaque changement d'état laisse un signe. Les rappels du
/// système passent par la file de messages du fil : la boucle de `winit` la vide, l'épreuve
/// aussi.
#[test]
fn test_la_chaine_rend_le_geste_puis_se_remet_sans_le_defaire() {
    let (fenetre, mut pave) = pave_cache();
    unsafe { pave.fil.viewport.ZoomToRect(0.0, 0.0, 500.0, 500.0, false) }.expect("le zoom");
    let mut rendus = Vec::new();
    for _ in 0..4 {
        pomper();
        rendus.extend(pave.avancer());
    }
    assert_eq!(
        rendus,
        vec![Mouvement {
            echelle: 2.0,
            decalage: (0.0, 0.0)
        }],
        "une similitude, et rien d'autre"
    );
    assert_eq!(
        pave.fil.etat.borrow().lecteur,
        Lecteur::default(),
        "la lecture est remise"
    );
    assert_eq!(
        transformation(&pave),
        [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        "le viewport est revenu à l'identité"
    );
    let signes = pave.signes();
    assert!(
        signes.iter().any(|s| s.quoi == "statut_pret"),
        "chaque changement d'état laisse un signe : {signes:?}"
    );
    drop(pave);
    let _ = unsafe { DestroyWindow(fenetre) };
}

/// **Le viewport suit la taille de la fenêtre** (fiche 54) : ses coordonnées sont les pixels de
/// la fenêtre, et la remise à l'identité vise tout le rectangle.
#[test]
fn test_le_viewport_suit_la_fenetre() {
    let (fenetre, mut pave) = pave_cache();
    pave.cadrer(1600, 900);
    let rect = unsafe { pave.fil.viewport.GetViewportRect() }.expect("le rectangle");
    assert_eq!(
        (rect.left, rect.top, rect.right, rect.bottom),
        (0, 0, 1600, 900)
    );
    // Le même zoom de deux, vers le coin : rendu, puis remis à l'identité sur 1600 × 900.
    unsafe { pave.fil.viewport.ZoomToRect(0.0, 0.0, 800.0, 450.0, false) }.expect("le zoom");
    let mut rendus = Vec::new();
    for _ in 0..4 {
        pomper();
        rendus.extend(pave.avancer());
    }
    assert_eq!(
        rendus.iter().map(|m| m.echelle).product::<f64>(),
        2.0,
        "{rendus:?}"
    );
    assert_eq!(transformation(&pave), [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    drop(pave);
    let _ = unsafe { DestroyWindow(fenetre) };
}

/// **Deux fins de geste de suite ne font qu'un geste** : la remise du viewport annonce elle-même
/// un second `READY`, qui peut arriver avant que le système ait livré l'identité. Le premier
/// met de côté le zoom de deux et demande la remise ; le second ne relit rien et ne la redemande
/// pas — sinon il inventerait un second zoom de deux, et remettrait sans fin.
#[test]
fn test_deux_fins_de_geste_de_suite_ne_font_qu_un_geste() {
    let mut etat = Etat {
        lecteur: Lecteur::default(),
        transformation: (2.0, 0.0, 0.0),
        attente: Vec::new(),
        interaction: false,
        vivant: false,
        taille: (1000.0, 1000.0),
        signes: Vec::new(),
    };
    assert!(etat.finir_le_geste(), "le premier demande la remise");
    assert!(!etat.finir_le_geste(), "le second ne la redemande pas");
    assert_eq!(
        etat.attente,
        vec![Mouvement {
            echelle: 2.0,
            decalage: (0.0, 0.0)
        }],
        "un seul geste"
    );
}

/// **Chaque signe que la voie de Windows émet a un nom connu** : ceux des états, et chaque
/// `signe("…")` écrit dans ce fichier — le serveur de la boîte noire refuserait une session qui
/// porterait un autre nom.
#[test]
fn test_chaque_signe_a_un_nom_connu() {
    use crate::interactions::pave::NOMS_DES_SIGNES;
    for code in -1..=7 {
        let nom = nom_du_statut(DIRECTMANIPULATION_STATUS(code));
        assert!(NOMS_DES_SIGNES.contains(&nom), "{nom}");
    }
    let source = include_str!("../pave_windows.rs");
    let ecrits: Vec<&str> = source
        .split("signe(\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .collect();
    assert!(ecrits.len() >= 4, "{ecrits:?}");
    for nom in ecrits {
        assert!(NOMS_DES_SIGNES.contains(&nom), "{nom}");
    }
}

/// La transformation du contenu, telle que le système la tient.
fn transformation(pave: &PaveWindows) -> [f32; 6] {
    let contenu: IDirectManipulationContent =
        unsafe { pave.fil.viewport.GetPrimaryContent() }.expect("le contenu");
    let mut m = [0f32; 6];
    unsafe { contenu.GetContentTransform(&mut m) }.expect("la transformation");
    m
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
