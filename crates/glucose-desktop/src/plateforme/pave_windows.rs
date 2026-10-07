//! **Le pavé de précision par *Direct Manipulation*** (fiche 53) — la voie de Windows de
//! [`crate::interactions::pave`].
//!
//! # La forme, celle de Chromium
//!
//! `direct_manipulation_helper_win.cc` : un gestionnaire, un *viewport* **fictif** de 1000 ×
//! 1000 qui ne montre rien — c'est sa transformation qu'on lit —, configuré pour le
//! déplacement, son inertie et l'échelle, en **mise à jour manuelle** : le système n'avance que
//! quand on le lui demande, une fois par image. Chaque contact du pavé arrive à la fenêtre en
//! `DM_POINTERHITTEST`, et c'est `SetContact` qui le confie au *viewport* ; les autres contacts
//! (un écran tactile) suivent leur chemin habituel.
//!
//! Le système dit quand un geste commence et finit (`OnInteraction`) : entre les deux, et
//! seulement là, Glucose le fait avancer à chaque image. Hors geste, rien ne réveille la boucle.
//! À la fin (`READY`), le *viewport* revient à l'identité (`ZoomToRect`), et la lecture repart
//! de zéro.
//!
//! # Ce qui ne marche pas et ne casse rien
//!
//! Si Windows refuse quoi que ce soit — pas de *Direct Manipulation*, pas de sous-classement —,
//! l'installation rend `None` et le pavé reste ce qu'il était : des défilements de molette,
//! que [`crate::interactions::pan_zoom`] sait lire.
//!
//! # `unsafe`, et où il s'arrête
//!
//! Tout le COM est ici. Rien de ce qui décide n'y est : la lecture de la transformation est
//! [`Lecteur`], pure.

use crate::interactions::pave::{Lecteur, Mouvement, Pave};
use std::cell::RefCell;
use std::rc::Rc;
use windows::core::{implement, Ref, Result as WinResult};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::DirectManipulation::{
    DirectManipulationManager, IDirectManipulationContent, IDirectManipulationFrameInfoProvider,
    IDirectManipulationInteractionEventHandler, IDirectManipulationInteractionEventHandler_Impl,
    IDirectManipulationManager, IDirectManipulationUpdateManager, IDirectManipulationViewport,
    IDirectManipulationViewport2, IDirectManipulationViewportEventHandler,
    IDirectManipulationViewportEventHandler_Impl, DIRECTMANIPULATION_CONFIGURATION_INTERACTION,
    DIRECTMANIPULATION_CONFIGURATION_RAILS_X, DIRECTMANIPULATION_CONFIGURATION_RAILS_Y,
    DIRECTMANIPULATION_CONFIGURATION_SCALING, DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_INERTIA,
    DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_X, DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_Y,
    DIRECTMANIPULATION_INERTIA, DIRECTMANIPULATION_INTERACTION_BEGIN,
    DIRECTMANIPULATION_INTERACTION_END, DIRECTMANIPULATION_INTERACTION_TYPE,
    DIRECTMANIPULATION_READY, DIRECTMANIPULATION_RUNNING, DIRECTMANIPULATION_STATUS,
    DIRECTMANIPULATION_VIEWPORT_OPTIONS_MANUALUPDATE,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::Input::Pointer::GetPointerType;
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, GetPropW, GetWindowLongPtrW, RemovePropW, SetPropW, SetWindowLongPtrW,
    DM_POINTERHITTEST, GWLP_WNDPROC, POINTER_INPUT_TYPE, PT_TOUCHPAD, WNDPROC,
};

/// Le côté du *viewport* fictif, celui de Chromium : il ne se voit pas, seule sa transformation
/// compte, et le système en a besoin d'un pour exister.
const COTE: i32 = 1000;

/// **Les deux choses que la fenêtre garde pour nous**, en propriétés : le viewport à qui donner
/// les contacts, et la procédure de `winit` à qui rendre tout le reste.
///
/// # Pourquoi pas `SetWindowSubclass`
///
/// C'était la première version, celle que la documentation conseille. Elle vit dans la version 6
/// des contrôles communs, qu'un programme sans manifeste qui la demande ne charge pas : l'épreuve
/// a refusé de **démarrer** (`STATUS_ENTRYPOINT_NOT_FOUND`), et Glucose aurait fait de même chez
/// lui. Le sous-classement de `user32` existe partout, depuis toujours.
const VIEWPORT: PCWSTR = w!("Glucose.Pave.Viewport");
const ANCIENNE: PCWSTR = w!("Glucose.Pave.Procedure");

/// Ce que le système et Glucose partagent, sur le fil de la fenêtre.
#[derive(Default)]
struct Etat {
    lecteur: Lecteur,
    /// Entre `INTERACTION_BEGIN` et `INTERACTION_END` : les doigts posés.
    interaction: bool,
    /// Le viewport bouge encore — `RUNNING`, ou l'`INERTIA` que les doigts ont laissée, qui peut
    /// survivre à la fin de l'interaction (Blender suit cet état-là).
    vivant: bool,
    mouvements: Vec<Mouvement>,
}

/// **Le pavé de cette fenêtre**, pris par *Direct Manipulation*.
pub struct PaveWindows {
    fenetre: HWND,
    gestionnaire: IDirectManipulationManager,
    mises_a_jour: IDirectManipulationUpdateManager,
    /// En boîte : le sous-classement en garde l'adresse pour donner chaque contact.
    viewport: Box<IDirectManipulationViewport>,
    etat: Rc<RefCell<Etat>>,
}

/// **Installe la voie du pavé sur cette fenêtre**, ou rend `None` si Windows refuse.
///
/// # Sûreté
///
/// `hwnd` doit être une fenêtre vivante de ce fil, et COM y être initialisé — ce que l'`OLE` de
/// `winit` fait pour son glisser-déposer.
pub fn installer(hwnd: isize, reveil: super::Reveil) -> Option<PaveWindows> {
    let fenetre = HWND(hwnd as *mut core::ffi::c_void);
    let etat = Rc::new(RefCell::new(Etat::default()));
    unsafe {
        let gestionnaire: IDirectManipulationManager =
            CoCreateInstance(&DirectManipulationManager, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| dire("le gestionnaire", &e))
                .ok()?;
        let mises_a_jour: IDirectManipulationUpdateManager = gestionnaire
            .GetUpdateManager()
            .map_err(|e| dire("les mises a jour", &e))
            .ok()?;
        let viewport: IDirectManipulationViewport = gestionnaire
            .CreateViewport(None::<&IDirectManipulationFrameInfoProvider>, fenetre)
            .map_err(|e| dire("le viewport", &e))
            .ok()?;
        let configuration = DIRECTMANIPULATION_CONFIGURATION_INTERACTION
            | DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_X
            | DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_Y
            | DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_INERTIA
            | DIRECTMANIPULATION_CONFIGURATION_RAILS_X
            | DIRECTMANIPULATION_CONFIGURATION_RAILS_Y
            | DIRECTMANIPULATION_CONFIGURATION_SCALING;
        let ecouteur: IDirectManipulationViewportEventHandler = Ecouteur {
            etat: Rc::clone(&etat),
            reveil,
        }
        .into();
        let rect = RECT {
            left: 0,
            top: 0,
            right: COTE,
            bottom: COTE,
        };
        viewport
            .ActivateConfiguration(configuration)
            .and_then(|()| {
                viewport.SetViewportOptions(DIRECTMANIPULATION_VIEWPORT_OPTIONS_MANUALUPDATE)
            })
            .and_then(|()| {
                viewport
                    .AddEventHandler(Some(fenetre), &ecouteur)
                    .map(|_| ())
            })
            .and_then(|()| viewport.SetViewportRect(&rect))
            .and_then(|()| gestionnaire.Activate(fenetre))
            .and_then(|()| viewport.Enable())
            .map_err(|e| dire("la configuration", &e))
            .ok()?;
        let viewport = Box::new(viewport);
        let donnee = &*viewport as *const IDirectManipulationViewport;
        if !sous_classer(fenetre, donnee) {
            eprintln!("[Glucose] pave : la fenetre refuse d'etre sous-classee");
            return None;
        }
        Some(PaveWindows {
            fenetre,
            gestionnaire,
            mises_a_jour,
            viewport,
            etat,
        })
    }
}

fn dire(quoi: &str, e: &windows::core::Error) {
    eprintln!("[Glucose] pave : Direct Manipulation refuse {quoi} ({e})");
}

impl Pave for PaveWindows {
    fn en_geste(&self) -> bool {
        let etat = self.etat.borrow();
        etat.interaction || etat.vivant
    }

    fn avancer(&mut self) -> Vec<Mouvement> {
        // `Update` appelle `OnContentUpdated` sur ce fil, dans l'appel : l'emprunt de l'état
        // doit donc être rendu avant.
        unsafe { self.mises_a_jour.Update(None).ok() };
        std::mem::take(&mut self.etat.borrow_mut().mouvements)
    }
}

impl Drop for PaveWindows {
    fn drop(&mut self) {
        unsafe {
            // Le sous-classement d'abord : il pointe sur le viewport.
            retirer_le_sous_classement(self.fenetre);
            let _ = self.viewport.Stop();
            let _ = self.viewport.Abandon();
            let _ = self.gestionnaire.Deactivate(self.fenetre);
        }
    }
}

/// **Glisse notre procédure devant celle de `winit`.** Les propriétés d'abord : la procédure doit
/// toujours trouver à qui rendre la main.
///
/// # Sûreté
///
/// `fenetre` est vivante et de ce fil ; `viewport` vit tant que le sous-classement est posé.
unsafe fn sous_classer(fenetre: HWND, viewport: *const IDirectManipulationViewport) -> bool {
    let actuelle = unsafe { GetWindowLongPtrW(fenetre, GWLP_WNDPROC) };
    let posees = unsafe {
        SetPropW(fenetre, ANCIENNE, Some(HANDLE(actuelle as *mut _)))
            .and_then(|()| SetPropW(fenetre, VIEWPORT, Some(HANDLE(viewport as *mut _))))
    };
    if posees.is_err() || actuelle == 0 {
        return false;
    }
    unsafe { SetWindowLongPtrW(fenetre, GWLP_WNDPROC, procedure as *const () as isize) != 0 }
}

/// **Retire le pavé de la fenêtre.** Le viewport n'y est plus jamais lu ; la procédure de `winit`
/// reprend sa place si personne ne s'est glissé devant nous depuis — sinon la nôtre reste, et ne
/// fait plus que passer la main.
unsafe fn retirer_le_sous_classement(fenetre: HWND) {
    let _ = unsafe { RemovePropW(fenetre, VIEWPORT) };
    if unsafe { GetWindowLongPtrW(fenetre, GWLP_WNDPROC) } == procedure as *const () as isize {
        let ancienne = unsafe { GetPropW(fenetre, ANCIENNE) }.0 as isize;
        unsafe { SetWindowLongPtrW(fenetre, GWLP_WNDPROC, ancienne) };
        let _ = unsafe { RemovePropW(fenetre, ANCIENNE) };
    }
}

/// **Chaque contact du pavé est confié au viewport** ; le reste suit son chemin.
///
/// Seul un contact de **pavé** : un doigt sur un écran tactile garde le chemin des pointeurs, et
/// le prendre ici le couperait du reste de Glucose (Chromium fait le même tri).
unsafe extern "system" fn procedure(
    fenetre: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == DM_POINTERHITTEST {
        // `GET_POINTERID_WPARAM` : les seize bits bas.
        let pointeur = (wparam.0 & 0xFFFF) as u32;
        let viewport =
            unsafe { GetPropW(fenetre, VIEWPORT) }.0 as *const IDirectManipulationViewport;
        let mut sorte = POINTER_INPUT_TYPE::default();
        if !viewport.is_null()
            && unsafe { GetPointerType(pointeur, &mut sorte) }.is_ok()
            && sorte == PT_TOUCHPAD
        {
            let _ = unsafe { (*viewport).SetContact(pointeur) };
            return LRESULT(0);
        }
    }
    // SAFETY : la propriété porte la procédure qui était là avant nous, et elle y reste tant que
    // la nôtre peut être appelée.
    let ancienne: WNDPROC = unsafe { std::mem::transmute(GetPropW(fenetre, ANCIENNE).0) };
    unsafe { CallWindowProcW(ancienne, fenetre, message, wparam, lparam) }
}

/// Ce que le système dit du viewport : son état, et sa transformation.
#[implement(
    IDirectManipulationViewportEventHandler,
    IDirectManipulationInteractionEventHandler
)]
struct Ecouteur {
    etat: Rc<RefCell<Etat>>,
    reveil: super::Reveil,
}

#[allow(non_snake_case)]
impl IDirectManipulationViewportEventHandler_Impl for Ecouteur_Impl {
    fn OnViewportStatusChanged(
        &self,
        viewport: Ref<'_, IDirectManipulationViewport>,
        actuel: DIRECTMANIPULATION_STATUS,
        _avant: DIRECTMANIPULATION_STATUS,
    ) -> WinResult<()> {
        let vivant = actuel == DIRECTMANIPULATION_RUNNING || actuel == DIRECTMANIPULATION_INERTIA;
        self.etat.borrow_mut().vivant = vivant;
        if vivant {
            (self.reveil)();
        }
        // **Le geste est fini** : le viewport revient à l'identité, et la lecture avec lui —
        // sinon le geste suivant partirait de l'échelle où celui-ci s'est arrêté. La lecture
        // d'abord : si le système annonce le retour à l'identité pendant l'appel, il ne se lit
        // pas comme un dézoom qui défairait tout le geste.
        //
        // Et seulement s'il n'y est pas déjà : la remise fait elle-même passer le viewport par
        // `RUNNING` puis `READY`, et la refaire à chaque fois ne finirait pas (Chromium garde
        // la même condition).
        if actuel == DIRECTMANIPULATION_READY {
            let deja = {
                let mut etat = self.etat.borrow_mut();
                let deja = etat.lecteur == Lecteur::default();
                etat.lecteur.remettre();
                deja
            };
            if let Some(viewport) = viewport.as_ref().filter(|_| !deja) {
                let cote = COTE as f32;
                unsafe { viewport.ZoomToRect(0.0, 0.0, cote, cote, false).ok() };
            }
        }
        Ok(())
    }

    fn OnViewportUpdated(&self, _viewport: Ref<'_, IDirectManipulationViewport>) -> WinResult<()> {
        Ok(())
    }

    fn OnContentUpdated(
        &self,
        _viewport: Ref<'_, IDirectManipulationViewport>,
        contenu: Ref<'_, IDirectManipulationContent>,
    ) -> WinResult<()> {
        let Some(contenu) = contenu.as_ref() else {
            return Ok(());
        };
        // La matrice 3 × 2 : [échelle x, 0, 0, échelle y, décalage x, décalage y].
        let mut m = [0f32; 6];
        unsafe { contenu.GetContentTransform(&mut m)? };
        let mut etat = self.etat.borrow_mut();
        if let Some(mouvement) = etat.lecteur.lire((m[0], m[4], m[5])) {
            etat.mouvements.push(mouvement);
        }
        Ok(())
    }
}

#[allow(non_snake_case)]
impl IDirectManipulationInteractionEventHandler_Impl for Ecouteur_Impl {
    fn OnInteraction(
        &self,
        _viewport: Ref<'_, IDirectManipulationViewport2>,
        interaction: DIRECTMANIPULATION_INTERACTION_TYPE,
    ) -> WinResult<()> {
        if interaction == DIRECTMANIPULATION_INTERACTION_BEGIN {
            self.etat.borrow_mut().interaction = true;
            // La boucle dort peut-être : c'est ce réveil qui la fait avancer le système.
            (self.reveil)();
        } else if interaction == DIRECTMANIPULATION_INTERACTION_END {
            self.etat.borrow_mut().interaction = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
