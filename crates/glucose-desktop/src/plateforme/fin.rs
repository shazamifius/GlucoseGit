//! **Quand le système met fin à Glucose** (FIN-SESSION-1, fiche 82) : une extinction, un
//! redémarrage, une déconnexion, un installeur qui doit remplacer un fichier, `kill`, `Ctrl+C`.
//!
//! # Ce qui était vrai, et ce qui ne l'était pas
//!
//! Le travail ne s'y perdait pas : le scribe synchronise chaque geste, et le texte en cours de
//! frappe a son propre fichier. Mais Glucose ne **finissait** pas : sous Windows, `winit` 0.30 ne
//! traite pas `WM_ENDSESSION`, sous Linux personne n'écoutait `SIGTERM`, et le système tuait le
//! processus. La boîte noire lisait alors la session « interrompue » : une extinction se
//! confondait avec un plantage — chez un utilisateur, deux sessions de 3 h 37 et de 7 h, les 21
//! et 22/10, chacune suivie du redémarrage de son PC. Un journal qui crie au plantage pour une
//! extinction cache les vrais.
//!
//! # Deux systèmes, deux formes — parce qu'ils ne tuent pas pareil
//!
//! * **Windows** peut tuer le processus dès que toutes les applications ont rendu la main sur
//!   `WM_ENDSESSION` : Glucose doit finir **pendant** le message ([`session_windows`]).
//! * **Linux** envoie `SIGTERM`, puis attend avant `SIGKILL` (quatre-vingt-dix secondes chez
//!   systemd) : un fil reçoit le signal et réveille la boucle, qui finit à son tour suivant
//!   ([`signaux`]). Un second signal fait partir sur-le-champ ([`Demande::recevoir`]) : un
//!   Glucose figé reste tuable, sans qu'aucun délai ait à être choisi.
//! * **macOS** annonce la fin par AppKit (`applicationWillTerminate:`), que `winit` transmet
//!   déjà à `exiting`. **Android** a sa propre vie (VIE-1) : le système y tue sans prévenir une
//!   application passée derrière, et la boîte noire le sait.
//!
//! Partout, ce qui finit est la même chose : [`crate::app::GlucoseApp::finir`].

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

#[cfg(windows)]
mod session_windows;
#[cfg(target_os = "linux")]
mod signaux;

/// **La fenêtre dont Windows annonce la fin de session.** Ailleurs, rien ne la construit.
#[derive(Debug)]
pub struct Session {
    /// La poignée Win32 de la fenêtre veillée.
    #[cfg_attr(not(windows), allow(dead_code))]
    fenetre: isize,
}

impl Session {
    /// **Windows a-t-il annoncé la fin de la session ?** La marque est posée sur la fenêtre
    /// pendant `WM_ENDSESSION`, avant que `winit` reçoive la demande de fermeture qui la suit.
    pub fn finit(&self) -> bool {
        #[cfg(windows)]
        return session_windows::la_session_finit(self.fenetre);
        #[cfg(not(windows))]
        false
    }

    /// Veille sur cette poignée — celle d'une fenêtre d'épreuve.
    #[cfg(all(windows, test))]
    pub(crate) fn de_la_poignee(fenetre: isize) -> Option<Self> {
        session_windows::veiller(fenetre).then_some(Self { fenetre })
    }
}

/// **Veille sur la fin de session de cette fenêtre** : sous Windows, une procédure glissée
/// devant celle de `winit`. Rend `None` ailleurs, ou si Windows refuse — Glucose est alors tué
/// à l'extinction, comme avant, sans rien perdre de ce qui est écrit.
pub fn veiller_sur_la_fenetre(fenetre: &winit::window::Window) -> Option<Session> {
    #[cfg(windows)]
    {
        use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let RawWindowHandle::Win32(w) = fenetre.window_handle().ok()?.as_raw() else {
            return None;
        };
        let fenetre = w.hwnd.get();
        session_windows::veiller(fenetre).then_some(Session { fenetre })
    }
    #[cfg(not(windows))]
    {
        let _ = fenetre;
        None
    }
}

/// **Ce que les signaux du système ont demandé**, partagé entre le fil qui les reçoit et la
/// boucle qui les lit.
#[derive(Clone, Default)]
pub struct Demande {
    faite: Arc<AtomicBool>,
    reveil: Arc<OnceLock<super::Reveil>>,
}

/// Ce que fait un signal reçu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    /// Le premier : Glucose finit à son prochain tour de boucle.
    Finir,
    /// Un autre, alors que la fin est déjà demandée : partir sur-le-champ, avec le code que les
    /// shells donnent à un processus tué par ce signal (`128 + n`).
    Partir(i32),
}

impl Demande {
    /// **La fin est-elle demandée ?**
    pub fn faite(&self) -> bool {
        self.faite.load(Ordering::Acquire)
    }

    /// **Un signal arrive** : le premier demande la fin, les suivants font partir. Une seule
    /// opération atomique tranche entre les deux — deux signaux presque simultanés ne peuvent
    /// pas être tous deux « le premier ».
    pub fn recevoir(&self, signal: i32) -> Issue {
        if self.faite.swap(true, Ordering::AcqRel) {
            Issue::Partir(128 + signal)
        } else {
            Issue::Finir
        }
    }

    /// De quoi réveiller la boucle : sans lui, un Glucose au repos ne lirait la demande qu'au
    /// prochain mouvement de souris.
    pub fn brancher(&self, reveil: super::Reveil) {
        let _ = self.reveil.set(reveil);
    }

    #[cfg(target_os = "linux")]
    fn reveiller(&self) {
        if let Some(reveil) = self.reveil.get() {
            reveil();
        }
    }
}

/// **Écoute `SIGTERM`, `SIGINT` et `SIGHUP`** — sous Linux. Rend `None` ailleurs, ou si le
/// système refuse.
pub fn ecouter_les_signaux(reveil: super::Reveil) -> Option<Demande> {
    let demande = Demande::default();
    demande.brancher(reveil);
    #[cfg(target_os = "linux")]
    return signaux::ecouter(demande);
    #[cfg(not(target_os = "linux"))]
    {
        let _ = demande;
        None
    }
}

#[cfg(test)]
pub(crate) mod tests;
