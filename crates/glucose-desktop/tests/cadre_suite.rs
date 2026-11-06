//! **BARRE-1 — le vrai Glucose, sous un vrai Windows, sans sa barre** (fiche 85).
//!
//! Cette machine de développement n'a plus Windows : seule la CI joue ce chemin. Une fenêtre
//! ordinaire pour le système, une bande qui la déplace, et trois boutons qui font ce qu'ils disent.
#![cfg(windows)]

use std::time::{Duration, Instant};
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClientRect, GetSystemMetrics, GetWindowLongW, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, IsZoomed, PostMessageW, SendMessageW,
    ShowWindow, GWL_STYLE, HTCAPTION, HTCLIENT, SM_CYCAPTION, SW_RESTORE, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MOUSEMOVE, WM_NCHITTEST, WS_CAPTION,
};

/// Le programme, tué si l'épreuve tombe avant lui.
struct Programme(std::process::Child);

impl Drop for Programme {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

/// Les fenêtres visibles de ce processus : leur poignée, leur titre, leur style.
fn fenetres_de(processus: u32) -> Vec<(HWND, String, u32)> {
    struct Cherche(u32, Vec<(HWND, String, u32)>);
    unsafe extern "system" fn une(fenetre: HWND, cherche: LPARAM) -> BOOL {
        let c = unsafe { &mut *(cherche.0 as *mut Cherche) };
        let mut proprietaire = 0;
        unsafe { GetWindowThreadProcessId(fenetre, Some(&raw mut proprietaire)) };
        if proprietaire == c.0 && unsafe { IsWindowVisible(fenetre) }.as_bool() {
            let mut titre = [0u16; 128];
            let lu = unsafe { GetWindowTextW(fenetre, &mut titre) }.max(0) as usize;
            let style = unsafe { GetWindowLongW(fenetre, GWL_STYLE) } as u32;
            c.1.push((fenetre, String::from_utf16_lossy(&titre[..lu]), style));
        }
        BOOL(1)
    }
    let mut cherche = Cherche(processus, Vec::new());
    let _ = unsafe { EnumWindows(Some(une), LPARAM(&raw mut cherche as isize)) };
    cherche.1
}

/// **La fenêtre de Glucose** : celle qui porte son nom, et non la première venue.
fn fenetre_de(processus: u32) -> Option<HWND> {
    fenetres_de(processus)
        .into_iter()
        .find(|(_, titre, _)| titre.contains("Glucose") || titre.contains("GLUCOSE"))
        .map(|(fenetre, _, _)| fenetre)
}

fn attendre(quoi: &str, condition: impl Fn() -> bool) {
    let limite = Instant::now() + Duration::from_secs(30);
    while !condition() {
        assert!(
            Instant::now() < limite,
            "{quoi} : rien après trente secondes"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn point_ecran(fenetre: HWND, x: i32, y: i32) -> LPARAM {
    let mut p = POINT { x, y };
    assert!(unsafe { ClientToScreen(fenetre, &raw mut p) }.as_bool());
    LPARAM(((p.y as isize & 0xffff) << 16) | (p.x as isize & 0xffff))
}

fn zone(fenetre: HWND, x: i32, y: i32) -> u32 {
    unsafe {
        SendMessageW(
            fenetre,
            WM_NCHITTEST,
            None,
            Some(point_ecran(fenetre, x, y)),
        )
    }
    .0 as u32
}

/// Un clic par les messages que Windows enverrait : l'épreuve ne dépend pas de ce qui a le focus.
fn cliquer(fenetre: HWND, x: i32, y: i32) {
    let dedans = LPARAM(((y as isize & 0xffff) << 16) | (x as isize & 0xffff));
    let poster = |message, bouton| unsafe {
        PostMessageW(Some(fenetre), message, WPARAM(bouton), dedans).expect("un message");
        std::thread::sleep(Duration::from_millis(250));
    };
    poster(WM_MOUSEMOVE, 0);
    poster(WM_LBUTTONDOWN, 1);
    poster(WM_LBUTTONUP, 0);
}

#[test]
fn test_barre_1_la_fenetre_sans_barre_se_deplace_et_ses_boutons_font_ce_qu_ils_disent() {
    let dossier = std::env::temp_dir().join(format!("glucose-cadre-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dossier);
    std::fs::create_dir_all(dossier.join("Glucose")).expect("le dossier");
    // Jamais de télémétrie depuis une épreuve.
    std::fs::write(
        dossier.join("Glucose").join("telemetrie.txt"),
        "envoyer=non\nvue=oui\ninstallation=00000000000000000000000000000abc\n",
    )
    .expect("l'accord");
    let mut programme = Programme(
        std::process::Command::new(env!("CARGO_BIN_EXE_glucose-desktop"))
            .env("LOCALAPPDATA", &dossier)
            .spawn()
            .expect("Glucose"),
    );
    let processus = programme.0.id();
    attendre("sa fenêtre", || fenetre_de(processus).is_some());
    let fenetre = fenetre_de(processus).expect("sa fenêtre");
    std::thread::sleep(Duration::from_secs(5)); // sa première image a posé les zones

    // **Une fenêtre ordinaire pour le système** : l'application NVIDIA ne la prend pas pour un jeu.
    let style = unsafe { GetWindowLongW(fenetre, GWL_STYLE) } as u32;
    let vues: Vec<_> = fenetres_de(processus)
        .into_iter()
        .map(|(_, titre, style)| format!("« {titre} » {style:#x}"))
        .collect();
    assert_ne!(
        style & WS_CAPTION.0,
        0,
        "elle garde WS_CAPTION (fiche 52) ; fenêtres vues : {vues:?}"
    );

    // **Sans la barre** : ce qui sépare la fenêtre de sa zone cliente n'a plus sa hauteur.
    let (mut dehors, mut dedans) = (RECT::default(), RECT::default());
    unsafe {
        GetWindowRect(fenetre, &raw mut dehors).expect("sa taille");
        GetClientRect(fenetre, &raw mut dedans).expect("sa zone");
    }
    let ecart = (dehors.bottom - dehors.top) - dedans.bottom;
    assert!(
        ecart < unsafe { GetSystemMetrics(SM_CYCAPTION) },
        "la barre du système prend encore {ecart} pixels"
    );

    let largeur = dedans.right;
    // Le titre « GLUCOSE » déplace ; le bouton de fermeture et le canevas reçoivent le clic.
    assert_eq!(zone(fenetre, 30, 20), HTCAPTION, "le titre de la bande");
    assert_eq!(zone(fenetre, largeur - 10, 20), HTCLIENT, "fermer");
    assert_eq!(
        zone(fenetre, largeur / 2, dedans.bottom / 2),
        HTCLIENT,
        "le canevas"
    );

    let (agrandir, reduire, fermer) = (largeur - 69, largeur - 115, largeur - 23);
    cliquer(fenetre, agrandir, 20);
    attendre("agrandir", || unsafe { IsZoomed(fenetre) }.as_bool());
    std::thread::sleep(Duration::from_secs(1));
    let mut agrandie = RECT::default();
    unsafe { GetClientRect(fenetre, &raw mut agrandie) }.expect("sa zone");
    cliquer(fenetre, agrandie.right - 69, 20);
    attendre("restaurer", || !unsafe { IsZoomed(fenetre) }.as_bool());
    std::thread::sleep(Duration::from_secs(1));

    cliquer(fenetre, reduire, 20);
    attendre("réduire", || unsafe { IsIconic(fenetre) }.as_bool());
    let _ = unsafe { ShowWindow(fenetre, SW_RESTORE) };
    attendre("revenir", || !unsafe { IsIconic(fenetre) }.as_bool());
    std::thread::sleep(Duration::from_secs(1));

    cliquer(fenetre, fermer, 20);
    let limite = Instant::now() + Duration::from_secs(30);
    let statut = loop {
        if let Some(s) = programme.0.try_wait().expect("son état") {
            break s;
        }
        assert!(Instant::now() < limite, "fermer n'a rien fermé");
        std::thread::sleep(Duration::from_millis(100));
    };
    assert!(statut.success(), "il finit proprement : {statut}");
    let _ = std::fs::remove_dir_all(&dossier);
}
