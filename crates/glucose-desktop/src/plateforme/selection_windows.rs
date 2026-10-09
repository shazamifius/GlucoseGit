//! **COPIER-1 — la sélection sous toutes les formes qu'une autre application sait prendre,
//! fabriquées quand elle les demande** (fiche 59). Le même objet part au presse-papiers
//! (`OleSetClipboard`) et dans un glisser (`DoDragDrop`).
//!
//! # Ce que Chromium lit, à la source
//!
//! Discord, Google Docs, tout navigateur : Chromium (`data_object.cc`, `clipboard_win.cc`) fait
//! de chaque fichier d'un `CF_HDROP` une pièce jointe — et d'un bitmap présent (`CF_DIB`, que
//! Windows tire de `CF_DIBV5`) **une de plus**. D'où, au presse-papiers :
//!
//! * **une image seule** : un PNG sous le format « PNG », que Chromium lit d'abord, et ses
//!   pixels en `CF_DIBV5`, pour tout logiciel qui ne lit que le bitmap (Paint, Word) — ce que
//!   pose « Copier l'image » d'un navigateur ;
//! * **plusieurs images** : la liste de leurs fichiers, comme l'explorateur copie des fichiers —
//!   et pas de bitmap, qui ferait une pièce jointe de trop ;
//! * **toujours** le lot de Glucose, que Glucose lit avant tout le reste : un bloc recolle un
//!   bloc ; et le texte des nœuds de texte.
//!
//! Le glisser emporte les fichiers dès une image : une cible de dépôt prend des fichiers
//! (Discord, Docs, l'explorateur, le bureau). Et « l'effet préféré » dit *copier* : une cible
//! qui déplacerait les fichiers les retirerait du dossier d'où d'autres collages les liront.
//!
//! # Rien ne s'écrit avant qu'on le demande
//!
//! Les fichiers sont les originaux des images, octet pour octet. Les écrire à chaque `Ctrl+C` —
//! trente images, des dizaines de mégaoctets — pour recoller dans Glucose, qui ne les lit
//! jamais, serait du travail perdu. L'objet ne fabrique donc une forme que quand une application
//! la demande (`GetData`) : les fichiers s'écrivent au premier collage qui les veut, une fois. À
//! la fermeture, `OleFlushClipboard` rend tout, pour qu'un collage après coup trouve encore la
//! sélection.
//!
//! # Où, et combien de temps
//!
//! Dans `%TEMP%\glucose-copies\<la date du lot>\`, sous le nom de chaque image. Le dossier de
//! l'objet précédent s'efface quand un nouveau part : le presse-papiers ne portait plus que le
//! nouveau. Ce que la fermeture laisse, le nettoyage du système le reprend.

use crate::interactions::clipboard::Formes;
use std::cell::{OnceCell, RefCell};
use std::collections::HashSet;
use std::mem::ManuallyDrop;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use windows::core::{implement, HSTRING};
use windows::Win32::Foundation::{
    DATA_S_SAMEFORMATETC, DV_E_FORMATETC, E_FAIL, E_NOTIMPL, OLE_E_ADVISENOTSUPPORTED, S_OK,
};
use windows::Win32::System::Com::{
    IAdviseSink, IDataObject, IDataObject_Impl, IEnumFORMATETC, IEnumSTATDATA, DATADIR_GET,
    DVASPECT_CONTENT, FORMATETC, STGMEDIUM, STGMEDIUM_0, TYMED_HGLOBAL,
};
use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
use windows::Win32::System::Ole::{
    OleFlushClipboard, OleSetClipboard, CF_DIBV5, CF_HDROP, CF_UNICODETEXT, DROPEFFECT_COPY,
};
use windows::Win32::UI::Shell::SHCreateStdEnumFmtEtc;

/// Pour quoi l'objet part : ses formes n'y sont pas les mêmes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pour {
    PressePapiers,
    Glisser,
}

/// Une forme que l'objet sait fabriquer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Forme {
    Lot,
    Texte,
    Png,
    Pixels,
    Fichiers,
    EffetPrefere,
}

/// La sélection, et ce qu'elle a déjà écrit.
#[implement(IDataObject)]
pub struct Selection {
    formes: Formes,
    /// Chaque format offert, et la forme qui le remplit.
    offertes: Vec<(u16, Forme)>,
    /// Le dossier où ses fichiers s'écrivent, quand on les demande.
    dossier: PathBuf,
    /// Les fichiers écrits — une fois, au premier collage qui les veut.
    ecrits: OnceCell<Result<Vec<PathBuf>, String>>,
}

thread_local! {
    /// L'objet que ce fil a confié au presse-papiers.
    static AU_PRESSE_PAPIERS: RefCell<Option<IDataObject>> = const { RefCell::new(None) };
    /// Le dossier du dernier objet parti, au presse-papiers et en glisser : celui-là seul peut
    /// encore servir. Les deux ne se remplacent pas — un glisser n'efface pas ce qu'un collage
    /// lira encore.
    static DERNIERS_DOSSIERS: RefCell<[Option<PathBuf>; 2]> = const { RefCell::new([None, None]) };
}

/// Le numéro d'un format enregistré.
fn format_nomme(nom: &str) -> Result<u16, String> {
    match unsafe { RegisterClipboardFormatW(&HSTRING::from(nom)) } {
        0 => Err(format!("le format « {nom} » n'a pas pu être enregistré")),
        f => u16::try_from(f).map_err(|_| "format hors d'atteinte".into()),
    }
}

/// Le dossier où les copies de Glucose posent leurs fichiers.
pub fn dossier_des_copies() -> PathBuf {
    std::env::temp_dir().join("glucose-copies")
}

impl Selection {
    /// L'objet d'une sélection, pour le presse-papiers ou pour un glisser.
    pub fn nouvelle(formes: Formes, pour: Pour) -> Result<Self, String> {
        let offertes = offertes(&formes, pour)?;
        let dossier = dossier_des_copies().join(formes.date.to_string());
        DERNIERS_DOSSIERS.with(|d| {
            let precedent = d.borrow_mut()[pour as usize].replace(dossier.clone());
            if let Some(ancien) = precedent.filter(|a| *a != dossier) {
                let _ = std::fs::remove_dir_all(ancien);
            }
        });
        Ok(Self {
            formes,
            offertes,
            dossier,
            ecrits: OnceCell::new(),
        })
    }

    fn forme(&self, demande: &FORMATETC) -> Option<Forme> {
        let accepte =
            demande.tymed & TYMED_HGLOBAL.0 as u32 != 0 && demande.dwAspect == DVASPECT_CONTENT.0;
        accepte
            .then(|| self.offertes.iter().find(|(f, _)| *f == demande.cfFormat))
            .flatten()
            .map(|(_, forme)| *forme)
    }

    /// Les octets d'une forme, fabriqués maintenant.
    fn octets(&self, forme: Forme) -> Result<Vec<u8>, String> {
        use super::presse_papiers::{dib_v5, envelopper_le_lot, texte_large};
        let image = || self.formes.image.as_ref().ok_or("aucune image seule");
        Ok(match forme {
            Forme::Lot => envelopper_le_lot(&self.formes.lot),
            Forme::Texte => texte_large(self.formes.texte.as_deref().unwrap_or("")),
            Forme::Png => image()?.png.clone(),
            Forme::Pixels => {
                let i = image()?;
                dib_v5(i.largeur, i.hauteur, &i.rgba)
            }
            Forme::Fichiers => liste_de_fichiers(&self.fichiers_ecrits()?),
            Forme::EffetPrefere => DROPEFFECT_COPY.0.to_le_bytes().to_vec(),
        })
    }

    /// **Les fichiers, écrits au premier collage qui les veut**, et jamais deux fois.
    pub fn fichiers_ecrits(&self) -> Result<Vec<PathBuf>, String> {
        self.ecrits
            .get_or_init(|| ecrire_les_fichiers(&self.formes, &self.dossier))
            .clone()
    }
}

/// Les formats qu'une sélection offre, selon qu'elle part au presse-papiers ou en glisser.
fn offertes(formes: &Formes, pour: Pour) -> Result<Vec<(u16, Forme)>, String> {
    let mut v = vec![(format_nomme("Glucose.Lot")?, Forme::Lot)];
    if formes.texte.is_some() {
        v.push((CF_UNICODETEXT.0, Forme::Texte));
    }
    let image_seule = pour == Pour::PressePapiers && formes.image.is_some();
    if image_seule {
        v.push((format_nomme("PNG")?, Forme::Png));
        v.push((CF_DIBV5.0, Forme::Pixels));
    } else if !formes.fichiers.is_empty() {
        v.push((CF_HDROP.0, Forme::Fichiers));
        v.push((format_nomme("Preferred DropEffect")?, Forme::EffetPrefere));
    }
    Ok(v)
}

/// Écrit chaque image sous un nom libre du dossier, ses octets d'origine tels quels.
fn ecrire_les_fichiers(formes: &Formes, dossier: &std::path::Path) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(dossier).map_err(|e| e.to_string())?;
    let mut pris = HashSet::new();
    let mut chemins = Vec::with_capacity(formes.fichiers.len());
    for f in &formes.fichiers {
        let chemin = dossier.join(nom_libre(&f.nom, &mut pris));
        std::fs::write(&chemin, f.octets.as_slice()).map_err(|e| e.to_string())?;
        chemins.push(chemin);
    }
    Ok(chemins)
}

/// **Un nom que le dossier n'a pas encore** : « photo.png », puis « photo 2.png »… Windows ne
/// distingue pas les majuscules : la comparaison non plus.
fn nom_libre(nom: &str, pris: &mut HashSet<String>) -> String {
    let (base, extension) = nom.rsplit_once('.').unwrap_or((nom, ""));
    let mut candidat = nom.to_string();
    let mut n = 1;
    while !pris.insert(candidat.to_lowercase()) {
        n += 1;
        candidat = if extension.is_empty() {
            format!("{base} {n}")
        } else {
            format!("{base} {n}.{extension}")
        };
    }
    candidat
}

/// **Un `CF_HDROP`** : l'en-tête `DROPFILES` — vingt octets, les chemins en UTF-16 juste après
/// —, puis chaque chemin terminé par un zéro, et un zéro de plus pour finir la liste.
pub fn liste_de_fichiers(chemins: &[PathBuf]) -> Vec<u8> {
    const EN_TETE: u32 = 20;
    let mut o = Vec::new();
    o.extend_from_slice(&EN_TETE.to_le_bytes()); // pFiles
    o.extend_from_slice(&[0u8; 8]); // pt
    o.extend_from_slice(&0u32.to_le_bytes()); // fNC
    o.extend_from_slice(&1u32.to_le_bytes()); // fWide
    for chemin in chemins {
        for unite in chemin.as_os_str().encode_wide().chain(std::iter::once(0)) {
            o.extend_from_slice(&unite.to_le_bytes());
        }
    }
    o.extend_from_slice(&0u16.to_le_bytes());
    o
}

/// **Confie la sélection au presse-papiers** : il ne reçoit qu'un objet, qui fabriquera chaque
/// forme quand une application la lui demandera. Sur le fil de la fenêtre, qui tient OLE.
pub fn confier_au_presse_papiers(formes: Formes) -> Result<(), String> {
    let objet: IDataObject = Selection::nouvelle(formes, Pour::PressePapiers)?.into();
    unsafe { OleSetClipboard(&objet) }.map_err(|e| e.to_string())?;
    AU_PRESSE_PAPIERS.with(|p| *p.borrow_mut() = Some(objet));
    Ok(())
}

/// **À la fermeture** : ce que Glucose a confié au presse-papiers y est rendu une fois pour
/// toutes — un collage après coup le trouve encore. Sans effet si rien n'est parti d'ici.
pub fn rendre_avant_de_quitter() {
    if AU_PRESSE_PAPIERS.with(|p| p.borrow_mut().take()).is_some() {
        let _ = unsafe { OleFlushClipboard() };
    }
}

// Les noms sont ceux de l'interface COM ; et les pointeurs viennent de l'appelant, qui en
// garantit la validité le temps de l'appel — c'est le contrat d'`IDataObject`, que la signature
// imposée ne permet pas de marquer `unsafe`. Chacun est lu par `as_ref`, qui refuse le nul.
#[allow(non_snake_case, clippy::not_unsafe_ptr_arg_deref)]
impl IDataObject_Impl for Selection_Impl {
    fn GetData(&self, demande: *const FORMATETC) -> windows::core::Result<STGMEDIUM> {
        let demande = unsafe { demande.as_ref() }.ok_or(windows::core::Error::from(E_FAIL))?;
        let forme = self
            .forme(demande)
            .ok_or(windows::core::Error::from(DV_E_FORMATETC))?;
        let octets = self
            .octets(forme)
            .map_err(|_| windows::core::Error::from(E_FAIL))?;
        let bloc = super::presse_papiers::bloc_global(&octets)
            .map_err(|_| windows::core::Error::from(E_FAIL))?;
        Ok(STGMEDIUM {
            tymed: TYMED_HGLOBAL.0 as u32,
            u: STGMEDIUM_0 { hGlobal: bloc },
            pUnkForRelease: ManuallyDrop::new(None),
        })
    }

    fn GetDataHere(&self, _: *const FORMATETC, _: *mut STGMEDIUM) -> windows::core::Result<()> {
        Err(E_NOTIMPL.into())
    }

    fn QueryGetData(&self, demande: *const FORMATETC) -> windows::core::HRESULT {
        match unsafe { demande.as_ref() }.and_then(|d| self.forme(d)) {
            Some(_) => S_OK,
            None => DV_E_FORMATETC,
        }
    }

    fn GetCanonicalFormatEtc(
        &self,
        entree: *const FORMATETC,
        sortie: *mut FORMATETC,
    ) -> windows::core::HRESULT {
        if let (Some(e), Some(s)) = unsafe { (entree.as_ref(), sortie.as_mut()) } {
            *s = *e;
            s.ptd = std::ptr::null_mut();
        }
        DATA_S_SAMEFORMATETC
    }

    fn SetData(
        &self,
        _: *const FORMATETC,
        _: *const STGMEDIUM,
        _: windows::core::BOOL,
    ) -> windows::core::Result<()> {
        Err(E_NOTIMPL.into())
    }

    fn EnumFormatEtc(&self, direction: u32) -> windows::core::Result<IEnumFORMATETC> {
        if direction != DATADIR_GET.0 as u32 {
            return Err(E_NOTIMPL.into());
        }
        let formats: Vec<FORMATETC> = self
            .offertes
            .iter()
            .map(|(f, _)| FORMATETC {
                cfFormat: *f,
                ptd: std::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT.0,
                lindex: -1,
                tymed: TYMED_HGLOBAL.0 as u32,
            })
            .collect();
        unsafe { SHCreateStdEnumFmtEtc(&formats) }
    }

    fn DAdvise(
        &self,
        _: *const FORMATETC,
        _: u32,
        _: windows::core::Ref<IAdviseSink>,
    ) -> windows::core::Result<u32> {
        Err(OLE_E_ADVISENOTSUPPORTED.into())
    }

    fn DUnadvise(&self, _: u32) -> windows::core::Result<()> {
        Err(OLE_E_ADVISENOTSUPPORTED.into())
    }

    fn EnumDAdvise(&self) -> windows::core::Result<IEnumSTATDATA> {
        Err(OLE_E_ADVISENOTSUPPORTED.into())
    }
}

#[cfg(test)]
mod tests;
