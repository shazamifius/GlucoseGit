//! **L'image du glisser au-dessus de Glucose** (Q10, fiche 156). Windows ne la montre que sur une
//! fenêtre dont la cible le lui demande, par `IDropTargetHelper` — l'explorateur, Chromium, et
//! désormais nous. Sans lui, l'ombre des nœuds disparaissait sur toute fenêtre de Glucose, celle
//! d'où ils partent comprise, et la vignette d'un fichier venu de l'explorateur de même.
//!
//! Chaque appel suit celui de la cible, avec l'effet qu'elle vient de répondre. Une image qui ne
//! se montre pas n'empêche rien : ses erreurs se taisent.

use windows::Win32::Foundation::{HWND, POINT, POINTL};
use windows::Win32::System::Com::{CoCreateInstance, IDataObject, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Ole::DROPEFFECT;
use windows::Win32::UI::Shell::{CLSID_DragDropHelper, IDropTargetHelper};

/// Le gestionnaire d'images du glisser, s'il a pu naître.
pub(super) struct Image(Option<IDropTargetHelper>);

impl Image {
    /// Sur le fil de la fenêtre, où OLE est ouvert.
    pub(super) fn nouvelle() -> Self {
        // SAFETY : un objet du système, créé dans ce processus ; rien n'est emprunté.
        Self(unsafe { CoCreateInstance(&CLSID_DragDropHelper, None, CLSCTX_INPROC_SERVER) }.ok())
    }

    pub(super) fn entrer(
        &self,
        fenetre: isize,
        objet: &IDataObject,
        ou: &POINTL,
        effet: DROPEFFECT,
    ) {
        if let Some(aide) = &self.0 {
            // SAFETY : la fenêtre vit autant que sa cible ; le point vit pendant l'appel.
            let _ = unsafe { aide.DragEnter(HWND(fenetre as *mut _), objet, &point(ou), effet) };
        }
    }

    pub(super) fn survoler(&self, ou: &POINTL, effet: DROPEFFECT) {
        if let Some(aide) = &self.0 {
            let _ = unsafe { aide.DragOver(&point(ou), effet) };
        }
    }

    pub(super) fn quitter(&self) {
        if let Some(aide) = &self.0 {
            let _ = unsafe { aide.DragLeave() };
        }
    }

    pub(super) fn deposer(&self, objet: &IDataObject, ou: &POINTL, effet: DROPEFFECT) {
        if let Some(aide) = &self.0 {
            let _ = unsafe { aide.Drop(objet, &point(ou), effet) };
        }
    }
}

/// Le point de l'écran, tel que le gestionnaire le lit.
fn point(ou: &POINTL) -> POINT {
    POINT { x: ou.x, y: ou.y }
}
