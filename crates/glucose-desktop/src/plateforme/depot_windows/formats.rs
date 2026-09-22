//! **Ce qu'un objet de dépôt porte, et comment on le lui demande** (DEPOT-WEB-1).
//!
//! Extrait de [`super::depot_windows`], qui portait la cible de dépôt **et** la lecture des
//! formats : deux raisons de changer, et sept cents lignes là où la fiche 05 en admet six
//! cents. La coupure tombe là où la nature du travail change — là-bas on décide ce qu'un dépôt
//! devient, ici on lit des octets que Windows a posés.
//!
//! Rien ici ne juge de quoi que ce soit. C'est de la mécanique COM : demander un format, le
//! nommer, verrouiller un bloc, le rendre.

use windows::core::PCWSTR;
use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::System::Com::{IDataObject, DATADIR_GET, FORMATETC, STGMEDIUM, TYMED};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

/// Combien d'octets au plus on accepte d'un seul bloc — la borne de [`super`], reprise ici
/// parce que c'est ici qu'on lit.
pub(super) const OCTETS_MAX: usize = 256 * 1024 * 1024;

/// **Écrit dans la console tout ce que ce dépôt portait**, quand `GLUCOSE_DEPOT` est posé.
///
/// # Pourquoi un instrument plutôt qu'une hypothèse
///
/// Un dépôt qui ne donne rien ne dit pas **pourquoi** : le format manquait-il, ou l'a-t-on mal
/// demandé ? Les deux se corrigent à l'opposé l'un de l'autre. Pinterest a montré le cas : ce
/// qu'on glisse depuis sa grille n'est pas une image mais un **lien**, et aucune supposition
/// sur ce que Chrome offre ne remplace la liste de ce qu'il a réellement offert.
///
/// C'est la règle du dépôt appliquée à autre chose qu'au rendu : on n'interroge pas, on
/// regarde. Et c'est à l'utilisateur de le déclencher, parce que c'est chez lui que le geste
/// a lieu.
pub(super) fn dire_les_formats(objet: &IDataObject) {
    if std::env::var_os("GLUCOSE_DEPOT").is_none() {
        return;
    }
    let Ok(enumerateur) = (unsafe { objet.EnumFormatEtc(DATADIR_GET.0 as u32) }) else {
        eprintln!("[Glucose] depot : l'objet refuse d'enumerer ses formats");
        return;
    };
    eprintln!("[Glucose] depot : ce que l'objet porte --");
    let mut tampon = [FORMATETC::default(); 1];
    loop {
        let mut lus = 0u32;
        if unsafe { enumerateur.Next(&mut tampon, Some(&mut lus)) }.is_err() || lus == 0 {
            break;
        }
        let f = tampon[0];
        let nom = nom_du_format(f.cfFormat);
        // Ce qui est **disponible** compte autant que ce qui est offert : un format annonce
        // qui refuse `QueryGetData` est un format qu'on ne peut pas lire.
        let lisible = unsafe { objet.QueryGetData(&f) }.is_ok();
        eprintln!(
            "    {:5}  {:<34} tymed {:#06x}  index {:3}  {}",
            f.cfFormat,
            nom,
            f.tymed,
            f.lindex,
            if lisible {
                "lisible"
            } else {
                "ANNONCE SEULEMENT"
            }
        );
    }
}

/// Le nom qu'un format porte, pour les formats nommés ; son numéro sinon.
pub(super) fn nom_du_format(format: u16) -> String {
    let mut tampon = [0u16; 128];
    let ecrits = unsafe {
        windows::Win32::System::DataExchange::GetClipboardFormatNameW(
            u32::from(format),
            &mut tampon,
        )
    };
    if ecrits > 0 {
        return String::from_utf16_lossy(&tampon[..ecrits as usize]);
    }
    match format {
        1 => "CF_TEXT".into(),
        2 => "CF_BITMAP".into(),
        8 => "CF_DIB".into(),
        13 => "CF_UNICODETEXT".into(),
        15 => "CF_HDROP".into(),
        17 => "CF_DIBV5".into(),
        autre => format!("(format {autre})"),
    }
}

/// Le numéro qu'un format porte sur cette session de Windows.
///
/// Les formats du shell n'ont pas de numéro fixe : chaque session en attribue un, et deux
/// programmes qui demandent le même nom reçoivent le même numéro. Zéro veut dire que Windows
/// a refusé, et un format à zéro ne se demande jamais.
pub(super) fn format_enregistre(nom: &str) -> u16 {
    let large: Vec<u16> = nom.encode_utf16().chain(std::iter::once(0)).collect();
    // `RegisterClipboardFormatW` rend un `u32` dont seuls les seize bits bas servent de
    // format ; zéro signale l'échec, et c'est le seul cas que nous ayons à distinguer.
    let numero = unsafe {
        windows::Win32::System::DataExchange::RegisterClipboardFormatW(PCWSTR(large.as_ptr()))
    };
    u16::try_from(numero).unwrap_or(0)
}

/// Une demande de format, telle que Windows l'attend.
pub(super) fn demande(format: u16, tymed: TYMED, index: i32) -> FORMATETC {
    FORMATETC {
        cfFormat: format,
        ptd: std::ptr::null_mut(),
        dwAspect: windows::Win32::System::Com::DVASPECT_CONTENT.0,
        lindex: index,
        tymed: tymed.0 as u32,
    }
}

/// Cet objet offre-t-il ce format ?
pub(super) fn offre(objet: &IDataObject, format: u16, tymed: TYMED) -> bool {
    unsafe { objet.QueryGetData(&demande(format, tymed, -1)).is_ok() }
}

/// Demande un format à l'objet, et rend ce que Windows a posé.
///
/// Le `STGMEDIUM` rendu appartient à l'appelant : il doit le rendre par `ReleaseStgMedium`,
/// et c'est pour cela que chaque appelant le fait explicitement plutôt que de le confier à un
/// garde — un garde exigerait de posséder le medium, et `ReleaseStgMedium` le veut mutable.
pub(super) fn tirer(
    objet: &IDataObject,
    format: u16,
    tymed: TYMED,
    index: i32,
) -> Option<STGMEDIUM> {
    if format == 0 {
        return None;
    }
    let demande = demande(format, tymed, index);
    unsafe {
        // `QueryGetData` d'abord : un objet refuse `GetData` par une exception chez certains
        // fournisseurs, et une exception traversant une frontiere COM ne se rattrape pas.
        if objet.QueryGetData(&demande).is_err() {
            return None;
        }
        objet.GetData(&demande).ok()
    }
}

/// Un bloc de mémoire globale, verrouillé le temps qu'on le lise.
///
/// `GlobalLock` doit être défait par `GlobalUnlock`, y compris quand la lecture échoue : ce
/// garde est ce qui rend impossible de l'oublier sur un chemin d'erreur.
pub(super) struct Bloc {
    poignee: HGLOBAL,
    pub(super) pointeur: *mut core::ffi::c_void,
    taille: usize,
}

impl Bloc {
    /// Verrouille ce bloc, ou rend rien s'il est vide.
    ///
    /// # Sûreté
    ///
    /// `poignee` doit être une poignée de mémoire globale valide, telle que Windows vient de
    /// la poser dans un `STGMEDIUM`.
    pub(super) unsafe fn prendre(poignee: HGLOBAL) -> Option<Self> {
        let pointeur = unsafe { GlobalLock(poignee) };
        if pointeur.is_null() {
            return None;
        }
        let taille = unsafe { GlobalSize(poignee) };
        if taille == 0 {
            unsafe { GlobalUnlock(poignee).ok() };
            return None;
        }
        Some(Self {
            poignee,
            pointeur,
            taille: taille.min(OCTETS_MAX),
        })
    }

    /// Les octets du bloc, copiés.
    pub(super) fn copier(&self) -> Vec<u8> {
        unsafe { std::slice::from_raw_parts(self.pointeur.cast::<u8>(), self.taille).to_vec() }
    }

    /// Le bloc lu comme du texte large, jusqu'à son premier zéro.
    pub(super) fn texte_large(&self) -> String {
        let combien = self.taille / 2;
        let large = unsafe { std::slice::from_raw_parts(self.pointeur.cast::<u16>(), combien) };
        let fin = large.iter().position(|c| *c == 0).unwrap_or(combien);
        String::from_utf16_lossy(&large[..fin])
    }
}

impl Drop for Bloc {
    fn drop(&mut self) {
        unsafe {
            GlobalUnlock(self.poignee).ok();
        }
    }
}
