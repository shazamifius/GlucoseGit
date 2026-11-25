//! **Le mouvement réduit** (GIF-3, fiche 159 § 8) : le réglage du système qui demande moins
//! d'animations — les GIF restent alors sur la toile qu'ils montrent. Relu au lancement, puis
//! chaque fois que la fenêtre revient au premier plan : c'est là qu'on revient d'avoir changé le
//! réglage. Linux : le portail du bureau (`org.freedesktop.appearance`, `reduced-motion`), sinon
//! l'interrupteur de GNOME ; Windows : `SPI_GETCLIENTAREAANIMATION`. Le Mac et Android ne le
//! lisent pas encore (fiche 159 § 8) : pour eux, le système n'a rien dit.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Mutex, PoisonError};

static REDUIT: AtomicBool = AtomicBool::new(false);
static REVEIL: Mutex<Option<super::Reveil>> = Mutex::new(None);

/// **Le système demande-t-il moins de mouvement ?** Faux tant qu'il n'a rien dit.
pub fn reduit() -> bool {
    REDUIT.load(Relaxed)
}

/// De quoi réveiller la boucle quand le réglage change, donné au lancement ; puis sa lecture.
pub fn brancher(reveil: super::Reveil) {
    *REVEIL.lock().unwrap_or_else(PoisonError::into_inner) = Some(reveil);
    relire();
}

/// **Relit le réglage sur un fil** : le portail répond en quelques millisecondes, jamais sur le
/// fil qui dessine.
pub fn relire() {
    let _ = std::thread::Builder::new()
        .name("mouvement".into())
        .spawn(|| {
            if let Some(reduit) = natif::lire() {
                poser(reduit);
            }
        });
}

/// Ce que le système vient de dire (et ce qu'une épreuve lui fait dire).
pub(crate) fn poser(reduit: bool) {
    if REDUIT.swap(reduit, Relaxed) == reduit {
        return;
    }
    crate::dire!(
        "[Glucose] mouvement réduit : {}",
        if reduit { "oui" } else { "non" }
    );
    if let Some(reveil) = REVEIL
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
    {
        reveil();
    }
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "android"))))]
mod natif {
    /// Le portail d'abord (xdg-desktop-portal 1.21, janvier 2026 : `1` demande moins de
    /// mouvement) ; un portail plus ancien n'a pas la clé, l'interrupteur de GNOME la remplace.
    pub fn lire() -> Option<bool> {
        pollster::block_on(async {
            let reglages = ashpd::desktop::settings::Settings::new().await.ok()?;
            let portail =
                (reglages.read::<u32>("org.freedesktop.appearance", "reduced-motion")).await;
            match portail {
                Ok(valeur) => Some(valeur == 1),
                Err(_) => (reglages
                    .read::<bool>("org.gnome.desktop.interface", "enable-animations"))
                .await
                .ok()
                .map(|anime| !anime),
            }
        })
    }
}

#[cfg(windows)]
mod natif {
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };

    /// « Afficher les animations dans Windows » (Accessibilité, Effets visuels).
    pub fn lire() -> Option<bool> {
        let mut anime = windows::core::BOOL::default();
        let ou = std::ptr::from_mut(&mut anime).cast();
        let sans_rien_ecrire = SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0);
        unsafe { SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, Some(ou), sans_rien_ecrire) }
            .ok()?;
        Some(!anime.as_bool())
    }
}

#[cfg(any(target_os = "macos", target_os = "android"))]
mod natif {
    pub fn lire() -> Option<bool> {
        None
    }
}

#[cfg(test)]
mod tests {
    /// **Sur un bureau qui a son portail** (pas la CI) : la lecture rend une réponse — vérifié le
    /// 25/11 sous niri, portail en version 2 (sans la clé : l'interrupteur de GNOME répond).
    #[test]
    #[ignore = "demande le portail d'un bureau"]
    fn gif_3_le_portail_repond() {
        let lu = super::natif::lire();
        crate::dire!("[Glucose] mouvement réduit, lu : {lu:?}");
        assert!(lu.is_some(), "le portail doit répondre");
    }
}
