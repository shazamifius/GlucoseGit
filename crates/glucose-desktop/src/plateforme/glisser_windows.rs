//! **Glisser des nœuds hors de la fenêtre** (fiche 51 § 2) : la source du glisser-déposer, dont
//! [`super::depot_windows`] est la cible.
//!
//! # Ce que Windows fournit, et ce qu'on écrit
//!
//! L'objet qui porte les données est celui du presse-papiers ([`super::selection_windows`],
//! COPIER-1, fiche 59) : le lot sous « Glucose.Lot », pour une autre fenêtre de Glucose ; le
//! texte des nœuds, pour un traitement de texte ; et les images **en fichiers**, pour tout le
//! reste — Discord, Google Docs, l'explorateur, le bureau. Il les fabrique quand la cible les
//! demande : rien ne s'écrit pour un dépôt dans Glucose.
//!
//! On écrit la **source** : deux questions auxquelles `DoDragDrop` revient à chaque mouvement.
//! Continuer ? Tant que le bouton gauche est tenu ; lâché, on dépose ; `Échap`, on renonce. Et
//! quel curseur ? Celui de Windows.
//!
//! # Glisser copie
//!
//! Les nœuds reviennent à leur place dans la fenêtre d'où ils partent, et une copie se pose là
//! où l'on lâche : rien ne peut se perdre en route, même si l'autre fenêtre se ferme pendant
//! le geste. Un déplacement qui retire l'original demanderait de croire l'autre fenêtre sur
//! parole.
//!
//! # Une fenêtre ne se dépose pas sur elle-même
//!
//! Tant qu'un glisser part d'ici, la cible de ce processus refuse les lots : ressortir puis
//! revenir doit rendre les nœuds à leur place, pas en poser une copie par-dessus.
//!
//! # Pendant le geste, la fenêtre d'origine se tait
//!
//! `DoDragDrop` tient sa boucle de messages **à l'intérieur** du gestionnaire d'événements de
//! `winit`. Un `WM_PAINT` qui y arrive ne peut pas nous être livré — le gestionnaire est pris —,
//! et `winit` 0.30 le redemande aussitôt (`RedrawWindow(RDW_INTERNALPAINT)` après
//! `DefWindowProcW`) : la boucle de `DoDragDrop`, qui tire ses messages par `GetMessage`,
//! le reçoit dès que sa file est vide, et un cœur tournait à vide pendant tout le geste. La
//! fenêtre restait aussi figée sur les nœuds tenus au bord, là où ils étaient sortis.
//!
//! L'appelant peint donc l'image où les nœuds sont revenus à leur place **avant** de partir,
//! et [`glisser`] tait la peinture que cette image laissait en attente : il n'y a rien à
//! redessiner pendant le geste, et la boucle de `DoDragDrop` dort entre deux mouvements.
//! Au retour, l'appelant redemande une image, et `winit` la livre comme toute autre.

use std::sync::atomic::{AtomicBool, Ordering};
use windows::core::{implement, BOOL, HRESULT};
use windows::Win32::Foundation::{
    DRAGDROP_S_CANCEL, DRAGDROP_S_DROP, DRAGDROP_S_USEDEFAULTCURSORS, HWND, S_OK,
};
use windows::Win32::Graphics::Gdi::{RedrawWindow, RDW_NOINTERNALPAINT};
use windows::Win32::System::Com::IDataObject;
use windows::Win32::System::Ole::{
    DoDragDrop, IDropSource, IDropSource_Impl, DROPEFFECT, DROPEFFECT_COPY,
};
use windows::Win32::System::SystemServices::{MK_LBUTTON, MODIFIERKEYS_FLAGS};

/// Un glisser part-il de ce processus en ce moment ?
static EN_COURS: AtomicBool = AtomicBool::new(false);

/// Un glisser part-il d'ici ? La cible de ce processus refuse alors les lots.
pub fn part_d_ici() -> bool {
    EN_COURS.load(Ordering::Relaxed)
}

/// **Glisse cette sélection** hors de cette fenêtre, jusqu'au lâcher ou à `Échap`. Rend vrai
/// s'il a été déposé quelque part.
///
/// Bloquant : `DoDragDrop` tient sa propre boucle de messages jusqu'à la fin du geste. La
/// fenêtre doit avoir été peinte juste avant (voir l'en-tête).
pub fn glisser(
    fenetre: *mut core::ffi::c_void,
    formes: crate::interactions::clipboard::Formes,
) -> Result<bool, String> {
    use super::selection_windows::{Pour, Selection};
    let objet: IDataObject = Selection::nouvelle(formes, Pour::Glisser)?.into();
    let source: IDropSource = Source.into();
    let mut effet = DROPEFFECT::default();
    taire_la_peinture_en_attente(HWND(fenetre));
    EN_COURS.store(true, Ordering::Relaxed);
    let rendu = unsafe { DoDragDrop(&objet, &source, DROPEFFECT_COPY, &mut effet) };
    EN_COURS.store(false, Ordering::Relaxed);
    Ok(rendu == DRAGDROP_S_DROP)
}

/// **Retire la peinture que cette fenêtre attend**, celle qu'un `request_redraw` de `winit` a
/// demandée (`RDW_INTERNALPAINT`) : l'image vient d'être peinte, et la redemander pendant
/// `DoDragDrop` ferait tourner sa boucle à vide (voir l'en-tête).
fn taire_la_peinture_en_attente(fenetre: HWND) {
    // Sans effet sur une région réellement invalide : seul le drapeau interne tombe.
    let _ = unsafe { RedrawWindow(Some(fenetre), None, None, RDW_NOINTERNALPAINT) };
}

/// Les deux questions de `DoDragDrop`.
#[implement(IDropSource)]
struct Source;

#[allow(non_snake_case)]
impl IDropSource_Impl for Source_Impl {
    fn QueryContinueDrag(&self, echap: BOOL, touches: MODIFIERKEYS_FLAGS) -> HRESULT {
        if echap.as_bool() {
            DRAGDROP_S_CANCEL
        } else if touches.0 & MK_LBUTTON.0 == 0 {
            DRAGDROP_S_DROP
        } else {
            S_OK
        }
    }

    fn GiveFeedback(&self, _effet: DROPEFFECT) -> HRESULT {
        DRAGDROP_S_USEDEFAULTCURSORS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Le glisser part sans peinture en attente** : celle que `request_redraw` de `winit`
    /// pose (`RDW_INTERNALPAINT`) ne sort plus de la file de messages — c'est elle que la
    /// boucle de `DoDragDrop` tirait sans fin.
    ///
    /// Windows ne peint jamais une fenêtre invisible : celle-ci est montrée, mais à −32 000
    /// pixels, hors de tout écran, sans bouton dans la barre des tâches et sans jamais prendre
    /// la main. Sa vraie zone à peindre est validée d'abord : seule la demande de `winit` compte.
    #[test]
    fn test_la_peinture_demandee_par_winit_ne_sort_plus_de_la_file() {
        use windows::core::w;
        use windows::Win32::Graphics::Gdi::{ValidateRect, RDW_INTERNALPAINT};
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, GetQueueStatus, SetWindowPos, QS_PAINT, SWP_NOACTIVATE,
            SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WS_EX_NOACTIVATE,
            WS_EX_TOOLWINDOW, WS_POPUP,
        };
        let fenetre = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                w!("STATIC"),
                w!("glucose-epreuve"),
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
        .expect("une fenetre hors de l'ecran");
        let drapeaux = SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER;
        let _ = unsafe { SetWindowPos(fenetre, None, 0, 0, 0, 0, drapeaux) };
        let _ = unsafe { ValidateRect(Some(fenetre), None) };
        // La file se lit sans y toucher : `PeekMessageW`, même sans retirer, consomme la
        // peinture interne qu'il trouve, et l'épreuve ne voyait plus rien (sabotage du 07/10).
        let attend_une_peinture = || (unsafe { GetQueueStatus(QS_PAINT) } >> 16) & QS_PAINT.0 != 0;
        let au_repos = attend_une_peinture();
        // Ce que fait `Window::request_redraw` de `winit` 0.30 sous Windows.
        let _ = unsafe { RedrawWindow(Some(fenetre), None, None, RDW_INTERNALPAINT) };
        let demandee = attend_une_peinture();
        taire_la_peinture_en_attente(fenetre);
        let apres = attend_une_peinture();
        let _ = unsafe { DestroyWindow(fenetre) };
        assert!(!au_repos, "la fenetre attendait deja une peinture");
        assert!(
            demandee,
            "la demande de winit ne posait pas de peinture : l'epreuve ne prouve rien"
        );
        assert!(!apres, "la peinture demandee attend toujours dans la file");
    }
}
