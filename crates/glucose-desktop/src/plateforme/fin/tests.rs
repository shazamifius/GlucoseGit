//! FIN-SESSION-1 — ce que le système dit de la fin de Glucose, vu sans écran.

use super::*;

/// **Le premier signal demande la fin, le suivant fait partir** — avec le code des shells.
#[test]
fn test_fin_session_1_le_premier_signal_finit_le_suivant_fait_partir() {
    let demande = Demande::default();
    assert!(!demande.faite());
    assert_eq!(demande.recevoir(15), Issue::Finir, "SIGTERM, le premier");
    assert!(demande.faite(), "la boucle le lira");
    assert_eq!(
        demande.recevoir(2),
        Issue::Partir(130),
        "un Ctrl+C de plus : partir, comme un processus tué par SIGINT"
    );
    assert!(
        demande.clone().faite(),
        "le fil et la boucle partagent la même demande"
    );
}

/// **Un `SIGTERM` réveille la boucle, et ne tue pas** : le gestionnaire écrit dans le tuyau, le
/// fil lit, demande la fin et réveille. Un seul signal : le second ferait partir l'épreuve.
#[cfg(target_os = "linux")]
#[test]
fn test_fin_session_1_un_sigterm_reveille_la_boucle_sans_tuer() {
    let (vers, reveils) = std::sync::mpsc::channel();
    let vers = std::sync::Mutex::new(vers);
    let reveil: crate::plateforme::Reveil = Arc::new(move || {
        let _ = vers.lock().map(|v| v.send(()));
    });
    let demande = ecouter_les_signaux(reveil).expect("le tuyau et le fil");
    assert!(!demande.faite());
    // SAFETY : un signal envoyé à ce processus, que le gestionnaire vient de prendre.
    unsafe { libc::raise(libc::SIGTERM) };
    reveils
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("le fil des signaux a réveillé la boucle");
    assert!(
        demande.faite(),
        "la fin est demandée, et l'épreuve vit encore"
    );
}

#[cfg(windows)]
pub(crate) use temoin::{fenetre_temoin, temoin};

/// **Une fenêtre cachée dont la procédure tient la place de `winit`** : elle note ce qui lui
/// parvient, et rend `0` à `WM_CLOSE` comme lui — sans détruire la fenêtre.
#[cfg(windows)]
mod temoin {
    use crate::plateforme::fin::session_windows::la_session_finit;
    use std::cell::RefCell;
    use windows::core::w;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WM_CLOSE, WM_NCDESTROY,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
    };

    /// Ce que la procédure témoin a vu, sur ce fil.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct Temoin {
        pub fermetures: usize,
        /// La marque était-elle posée quand la fermeture est arrivée ?
        pub marquee_a_la_fermeture: bool,
        /// La marque était-elle encore là au dernier message de la fenêtre ?
        pub marquee_a_la_destruction: Option<bool>,
    }

    thread_local! {
        static VU: RefCell<Temoin> = RefCell::default();
    }

    /// Ce que la procédure témoin a vu depuis la dernière fenêtre témoin de ce fil.
    pub(crate) fn temoin() -> Temoin {
        VU.with(|v| *v.borrow())
    }

    /// Une fenêtre à −32 000 pixels, jamais montrée, sous la procédure témoin.
    pub(crate) fn fenetre_temoin() -> isize {
        VU.with(|v| *v.borrow_mut() = Temoin::default());
        // SAFETY : une fenêtre de ce fil, dont on remplace la procédure avant tout message
        // qu'on lui envoie.
        unsafe {
            let fenetre = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                w!("STATIC"),
                w!("glucose-epreuve-fin"),
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
            .expect("une fenêtre hors de l'écran");
            SetWindowLongPtrW(fenetre, GWLP_WNDPROC, procedure as *const () as isize);
            fenetre.0 as isize
        }
    }

    unsafe extern "system" fn procedure(
        fenetre: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        let marquee = la_session_finit(fenetre.0 as isize);
        match message {
            WM_CLOSE => {
                VU.with(|v| {
                    let mut v = v.borrow_mut();
                    v.fermetures += 1;
                    v.marquee_a_la_fermeture = marquee;
                });
                LRESULT(0)
            }
            _ => {
                if message == WM_NCDESTROY {
                    VU.with(|v| v.borrow_mut().marquee_a_la_destruction = Some(marquee));
                }
                // SAFETY : la procédure par défaut, pour tout ce que le témoin ne note pas.
                unsafe { DefWindowProcW(fenetre, message, wparam, lparam) }
            }
        }
    }
}

#[cfg(windows)]
pub(crate) use windows_seulement::{detruire, envoyer_la_fin};

#[cfg(windows)]
mod windows_seulement {
    use super::*;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyWindow, SendMessageW, WM_ENDSESSION};

    pub(crate) fn envoyer_la_fin(fenetre: isize, finit: bool) {
        // SAFETY : une fenêtre témoin de ce fil ; le message est traité avant le retour.
        unsafe {
            SendMessageW(
                HWND(fenetre as *mut _),
                WM_ENDSESSION,
                Some(WPARAM(usize::from(finit))),
                Some(LPARAM(0)),
            )
        };
    }

    pub(crate) fn detruire(fenetre: isize) {
        // SAFETY : la fenêtre témoin, de ce fil.
        let _ = unsafe { DestroyWindow(HWND(fenetre as *mut _)) };
    }

    /// **FIN-SESSION-1 : la fenêtre est marquée AVANT que `winit` reçoive la fermeture**, et
    /// il la reçoit une fois. Dans l'autre ordre, l'application prendrait la croix ordinaire, et
    /// poserait une question que personne ne lira avant l'extinction.
    #[test]
    fn test_fin_session_1_la_marque_precede_la_fermeture() {
        let fenetre = fenetre_temoin();
        let session = Session::de_la_poignee(fenetre).expect("la procédure se pose");
        assert!(!session.finit(), "rien n'est annoncé");
        envoyer_la_fin(fenetre, true);
        let vu = temoin();
        assert_eq!(vu.fermetures, 1, "winit reçoit une fermeture : {vu:?}");
        assert!(vu.marquee_a_la_fermeture, "la marque la précède : {vu:?}");
        assert!(session.finit());
        detruire(fenetre);
    }

    /// **Une extinction annulée ne change rien** (`wParam` faux) : ni marque, ni fermeture.
    #[test]
    fn test_fin_session_1_une_extinction_annulee_ne_ferme_rien() {
        let fenetre = fenetre_temoin();
        let session = Session::de_la_poignee(fenetre).expect("la procédure se pose");
        envoyer_la_fin(fenetre, false);
        assert_eq!(temoin().fermetures, 0, "rien ne se ferme");
        assert!(!session.finit(), "rien n'est marqué");
        detruire(fenetre);
    }

    /// **La marque part avec la fenêtre** (la règle de `SetPropW`) : au dernier message, elle
    /// n'y est plus quand il atteint la procédure d'avant.
    #[test]
    fn test_fin_session_1_la_marque_part_avec_la_fenetre() {
        let fenetre = fenetre_temoin();
        let _session = Session::de_la_poignee(fenetre).expect("la procédure se pose");
        envoyer_la_fin(fenetre, true);
        assert!(temoin().marquee_a_la_fermeture);
        detruire(fenetre);
        assert_eq!(
            temoin().marquee_a_la_destruction,
            Some(false),
            "nos propriétés sont retirées avant la procédure d'avant"
        );
    }
}
