//! **Le pavé de précision par *Direct Manipulation*** (fiches 53 et 54) — la voie de Windows de
//! [`crate::interactions::pave`].
//!
//! # La forme, celle de Chromium et de Flutter
//!
//! Un gestionnaire, un *viewport* **fictif** qui ne montre rien — c'est sa transformation qu'on
//! lit —, configuré pour le déplacement, son inertie et l'échelle, en **mise à jour manuelle** :
//! le système n'avance que quand on le lui demande, une fois par image. Chaque contact du pavé
//! arrive à la fenêtre en `DM_POINTERHITTEST`, et c'est `SetContact` qui le confie au
//! *viewport* ; les autres contacts (un écran tactile) suivent leur chemin habituel.
//!
//! Le *viewport* a **la taille de la fenêtre**, comme chez Flutter, et la suit : ses
//! coordonnées sont alors les pixels de la fenêtre, et la similitude qu'il rend s'applique telle
//! quelle au canevas. Chromium en prenait un de 1000 × 1000, une constante que rien ne fixait.
//!
//! Le système dit quand un geste commence et finit (`OnInteraction`) : entre les deux, et
//! seulement là, Glucose le fait avancer à chaque image. Hors geste, rien ne réveille la boucle.
//! À la fin (`READY`), ce qui n'a pas encore été lu est mis de côté, le *viewport* revient à
//! l'identité (`ZoomToRect`), et la lecture repart de zéro.
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

use crate::interactions::pave::{Lecteur, Mouvement, Pave, Signe, IDENTITE};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
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

/// **Les deux choses que la fenêtre garde pour nous**, en propriétés : ce que la procédure doit
/// joindre (le viewport et l'état partagé), et la procédure de `winit` à qui rendre tout le
/// reste.
///
/// # Pourquoi pas `SetWindowSubclass`
///
/// C'était la première version, celle que la documentation conseille. Elle vit dans la version 6
/// des contrôles communs, qu'un programme sans manifeste qui la demande ne charge pas : l'épreuve
/// a refusé de **démarrer** (`STATUS_ENTRYPOINT_NOT_FOUND`), et Glucose aurait fait de même chez
/// lui. Le sous-classement de `user32` existe partout, depuis toujours.
const FIL: PCWSTR = w!("Glucose.Pave.Fil");
const ANCIENNE: PCWSTR = w!("Glucose.Pave.Procedure");

/// Ce que le système et Glucose partagent, sur le fil de la fenêtre.
struct Etat {
    lecteur: Lecteur,
    /// La dernière transformation que le système a donnée, lue une fois par image.
    transformation: (f32, f32, f32),
    /// Ce que le geste a fait après la dernière image et avant de finir : rendu à la suivante.
    attente: Vec<Mouvement>,
    /// Entre `INTERACTION_BEGIN` et `INTERACTION_END` : les doigts posés.
    interaction: bool,
    /// Le viewport bouge encore — `RUNNING`, ou l'`INERTIA` que les doigts ont laissée, qui peut
    /// survivre à la fin de l'interaction (Blender suit cet état-là).
    vivant: bool,
    /// La taille du viewport : celle de la fenêtre.
    taille: (f32, f32),
    signes: Vec<Signe>,
}

impl Etat {
    /// **Le geste est fini** (`READY`) : ce qu'il a fait depuis la dernière image est mis de
    /// côté, puis la lecture repart de l'identité, et ce que l'on sait de la transformation
    /// aussi. Rend `true` si le viewport doit y être remis — seulement s'il n'y est pas déjà :
    /// la remise fait elle-même passer le viewport par `RUNNING` puis `READY`, et ce second
    /// `READY` peut arriver avant que le système ait livré l'identité. Le refaire ne finirait
    /// pas, et le relire inventerait un geste (Chromium garde la même condition).
    fn finir_le_geste(&mut self) -> bool {
        let transformation = self.transformation;
        let reste = self.lecteur.lire(transformation);
        self.attente.extend(reste);
        self.lecteur.remettre();
        self.transformation = IDENTITE;
        transformation != IDENTITE
    }

    fn signe(&mut self, quoi: &'static str, valeur: u64) {
        self.signes.push(Signe {
            quand: Instant::now(),
            quoi,
            valeur,
        });
    }
}

/// Ce que la procédure de la fenêtre joint, par la propriété [`FIL`].
struct Fil {
    viewport: IDirectManipulationViewport,
    etat: Rc<RefCell<Etat>>,
}

/// **Le pavé de cette fenêtre**, pris par *Direct Manipulation*.
pub struct PaveWindows {
    fenetre: HWND,
    gestionnaire: IDirectManipulationManager,
    mises_a_jour: IDirectManipulationUpdateManager,
    /// En boîte : le sous-classement en garde l'adresse pour donner chaque contact.
    fil: Box<Fil>,
}

/// Le rectangle du viewport : la fenêtre entière, en ses pixels.
fn rectangle((largeur, hauteur): (f32, f32)) -> RECT {
    RECT {
        left: 0,
        top: 0,
        right: largeur.max(1.0) as i32,
        bottom: hauteur.max(1.0) as i32,
    }
}

/// **Installe la voie du pavé sur cette fenêtre**, de cette taille en pixels, ou rend `None` si
/// Windows refuse.
///
/// # Sûreté
///
/// `hwnd` doit être une fenêtre vivante de ce fil, et COM y être initialisé — ce que l'`OLE` de
/// `winit` fait pour son glisser-déposer.
pub fn installer(hwnd: isize, taille: (u32, u32), reveil: super::Reveil) -> Option<PaveWindows> {
    let fenetre = HWND(hwnd as *mut core::ffi::c_void);
    let taille = (taille.0 as f32, taille.1 as f32);
    let etat = Rc::new(RefCell::new(Etat {
        lecteur: Lecteur::default(),
        transformation: IDENTITE,
        attente: Vec::new(),
        interaction: false,
        vivant: false,
        taille,
        signes: Vec::new(),
    }));
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
        // Sans les « rails » de Chromium (`RAILS_X`, `RAILS_Y`), qui collent un défilement
        // presque vertical à la verticale : juste pour une page, faux pour un canevas, où un
        // déplacement en biais doit rester en biais. Sans inertie d'échelle non plus : le
        // pincement s'arrête avec les doigts.
        let configuration = DIRECTMANIPULATION_CONFIGURATION_INTERACTION
            | DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_X
            | DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_Y
            | DIRECTMANIPULATION_CONFIGURATION_TRANSLATION_INERTIA
            | DIRECTMANIPULATION_CONFIGURATION_SCALING;
        let ecouteur: IDirectManipulationViewportEventHandler = Ecouteur {
            etat: Rc::clone(&etat),
            reveil,
        }
        .into();
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
            .and_then(|()| viewport.SetViewportRect(&rectangle(taille)))
            .and_then(|()| gestionnaire.Activate(fenetre))
            .and_then(|()| viewport.Enable())
            .map_err(|e| dire("la configuration", &e))
            .ok()?;
        let fil = Box::new(Fil { viewport, etat });
        if !sous_classer(fenetre, &*fil) {
            eprintln!("[Glucose] pave : la fenetre refuse d'etre sous-classee");
            return None;
        }
        Some(PaveWindows {
            fenetre,
            gestionnaire,
            mises_a_jour,
            fil,
        })
    }
}

fn dire(quoi: &str, e: &windows::core::Error) {
    eprintln!("[Glucose] pave : Direct Manipulation refuse {quoi} ({e})");
}

impl Pave for PaveWindows {
    fn en_geste(&self) -> bool {
        let etat = self.fil.etat.borrow();
        etat.interaction || etat.vivant || !etat.attente.is_empty()
    }

    fn avancer(&mut self) -> Vec<Mouvement> {
        // `Update` appelle `OnContentUpdated` sur ce fil, dans l'appel : l'emprunt de l'état
        // doit donc être rendu avant.
        unsafe { self.mises_a_jour.Update(None).ok() };
        let mut etat = self.fil.etat.borrow_mut();
        let mut rendus = std::mem::take(&mut etat.attente);
        let transformation = etat.transformation;
        rendus.extend(etat.lecteur.lire(transformation));
        rendus
    }

    fn cadrer(&mut self, largeur: u32, hauteur: u32) {
        let taille = (largeur as f32, hauteur as f32);
        self.fil.etat.borrow_mut().taille = taille;
        if unsafe { self.fil.viewport.SetViewportRect(&rectangle(taille)) }.is_err() {
            self.fil.etat.borrow_mut().signe("cadre_refuse", 0);
        }
    }

    fn signes(&mut self) -> Vec<Signe> {
        std::mem::take(&mut self.fil.etat.borrow_mut().signes)
    }
}

impl Drop for PaveWindows {
    fn drop(&mut self) {
        unsafe {
            // Le sous-classement d'abord : il pointe sur le fil.
            retirer_le_sous_classement(self.fenetre);
            let _ = self.fil.viewport.Stop();
            let _ = self.fil.viewport.Abandon();
            let _ = self.gestionnaire.Deactivate(self.fenetre);
        }
    }
}

/// **Glisse notre procédure devant celle de `winit`.** Les propriétés d'abord : la procédure doit
/// toujours trouver à qui rendre la main.
///
/// # Sûreté
///
/// `fenetre` est vivante et de ce fil ; `fil` vit tant que le sous-classement est posé.
unsafe fn sous_classer(fenetre: HWND, fil: *const Fil) -> bool {
    let actuelle = unsafe { GetWindowLongPtrW(fenetre, GWLP_WNDPROC) };
    let posees = unsafe {
        SetPropW(fenetre, ANCIENNE, Some(HANDLE(actuelle as *mut _)))
            .and_then(|()| SetPropW(fenetre, FIL, Some(HANDLE(fil as *mut _))))
    };
    if posees.is_err() || actuelle == 0 {
        return false;
    }
    unsafe { SetWindowLongPtrW(fenetre, GWLP_WNDPROC, procedure as *const () as isize) != 0 }
}

/// **Retire le pavé de la fenêtre.** Le fil n'y est plus jamais lu ; la procédure de `winit`
/// reprend sa place si personne ne s'est glissé devant nous depuis — sinon la nôtre reste, et ne
/// fait plus que passer la main.
unsafe fn retirer_le_sous_classement(fenetre: HWND) {
    let _ = unsafe { RemovePropW(fenetre, FIL) };
    if unsafe { GetWindowLongPtrW(fenetre, GWLP_WNDPROC) } == procedure as *const () as isize {
        let ancienne = unsafe { GetPropW(fenetre, ANCIENNE) }.0 as isize;
        unsafe { SetWindowLongPtrW(fenetre, GWLP_WNDPROC, ancienne) };
        let _ = unsafe { RemovePropW(fenetre, ANCIENNE) };
    }
}

/// **Chaque contact du pavé est confié au viewport** ; le reste suit son chemin.
///
/// Seul un contact de **pavé** : un doigt sur un écran tactile garde le chemin des pointeurs, et
/// le prendre ici le couperait du reste de Glucose (Chromium fait le même tri). Chaque contact
/// confié laisse un signe — accepté (`0`) ou le code du refus.
unsafe extern "system" fn procedure(
    fenetre: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == DM_POINTERHITTEST {
        // `GET_POINTERID_WPARAM` : les seize bits bas.
        let pointeur = (wparam.0 & 0xFFFF) as u32;
        let fil = unsafe { GetPropW(fenetre, FIL) }.0 as *const Fil;
        let mut sorte = POINTER_INPUT_TYPE::default();
        if !fil.is_null()
            && unsafe { GetPointerType(pointeur, &mut sorte) }.is_ok()
            && sorte == PT_TOUCHPAD
        {
            // SAFETY : la propriété pointe sur le fil tant que le sous-classement est posé.
            let fil = unsafe { &*fil };
            let refus = unsafe { fil.viewport.SetContact(pointeur) }
                .err()
                .map_or(0, |e| u64::from(e.code().0 as u32));
            if let Ok(mut etat) = fil.etat.try_borrow_mut() {
                etat.signe("contact", refus);
            }
            return LRESULT(0);
        }
    }
    // SAFETY : la propriété porte la procédure qui était là avant nous, et elle y reste tant que
    // la nôtre peut être appelée.
    let ancienne: WNDPROC = unsafe { std::mem::transmute(GetPropW(fenetre, ANCIENNE).0) };
    unsafe { CallWindowProcW(ancienne, fenetre, message, wparam, lparam) }
}

/// Le nom d'un état du viewport, pour la boîte noire.
fn nom_du_statut(statut: DIRECTMANIPULATION_STATUS) -> &'static str {
    match statut.0 {
        0 => "statut_en_construction",
        1 => "statut_actif",
        2 => "statut_desactive",
        3 => "statut_en_cours",
        4 => "statut_inertie",
        5 => "statut_pret",
        6 => "statut_suspendu",
        _ => "statut_inconnu",
    }
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
        avant: DIRECTMANIPULATION_STATUS,
    ) -> WinResult<()> {
        let vivant = actuel == DIRECTMANIPULATION_RUNNING || actuel == DIRECTMANIPULATION_INERTIA;
        {
            let mut etat = self.etat.borrow_mut();
            etat.vivant = vivant;
            etat.signe(nom_du_statut(actuel), avant.0 as u64);
        }
        if vivant {
            (self.reveil)();
        }
        // **Le geste est fini** : la lecture d'abord, puis la remise du viewport à l'identité —
        // sinon le geste suivant partirait de l'échelle où celui-ci s'est arrêté, et la remise
        // se lirait comme un mouvement qui défairait tout le geste.
        if actuel == DIRECTMANIPULATION_READY {
            let (remettre, (largeur, hauteur)) = {
                let mut etat = self.etat.borrow_mut();
                (etat.finir_le_geste(), etat.taille)
            };
            if remettre {
                if let Some(viewport) = viewport.as_ref() {
                    unsafe { viewport.ZoomToRect(0.0, 0.0, largeur, hauteur, false).ok() };
                }
                (self.reveil)();
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
        self.etat.borrow_mut().transformation = (m[0], m[4], m[5]);
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
            {
                let mut etat = self.etat.borrow_mut();
                etat.interaction = true;
                etat.signe("interaction_debut", 0);
            }
            // La boucle dort peut-être : c'est ce réveil qui la fait avancer le système.
            (self.reveil)();
        } else if interaction == DIRECTMANIPULATION_INTERACTION_END {
            let mut etat = self.etat.borrow_mut();
            etat.interaction = false;
            etat.signe("interaction_fin", 0);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
