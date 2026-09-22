//! **DEPOT-WEB-1** : ce qu'on glisse depuis un navigateur arrive enfin jusqu'au canevas.
//!
//! # Ce qui ne marchait pas, et pourquoi personne ne pouvait le voir
//!
//! *« Le glisser-déposer depuis n'importe quel site web »* est une fonction phare de Glucose
//! Tauri, et elle était **cassée** ici. `winit` ne transmet qu'un seul format, `CF_HDROP` —
//! la liste de chemins que l'explorateur de fichiers produit. Un navigateur n'en donne
//! jamais : il n'a pas de fichier à offrir, il a une page. Glucose ne voyait donc
//! strictement **rien** passer, pas même un événement à ignorer.
//!
//! # Ce qu'un navigateur offre vraiment, et c'est plus que ce que je croyais
//!
//! La recherche a démenti la supposition qui menaçait d'engager le projet. On lit partout
//! qu'un navigateur « ne donne qu'une adresse », et l'on en conclut qu'il faudrait
//! **télécharger** l'image soi-même — donc HTTPS, donc une poignée de dépendances, donc une
//! décision qui engage la charte.
//!
//! C'est faux depuis 2009. Windows a un format pour les fichiers qui n'existent pas encore —
//! `CFSTR_FILEDESCRIPTORW` pour les décrire, `CFSTR_FILECONTENTS` pour les lire — et Chrome
//! comme Firefox l'offrent pour toute image glissée hors de la page. **C'est le navigateur
//! qui télécharge**, avec ses propres connexions, son propre cache et ses propres cookies ;
//! nous lisons un flux. Zéro dépendance réseau, et l'image arrive même sur une page qui
//! demande une authentification — ce qu'un téléchargement de notre côté n'aurait jamais su
//! faire.
//!
//! # Trois formats, dans l'ordre du plus sûr au plus pauvre
//!
//! 1. **`CF_HDROP`** — de vrais fichiers, venus de l'explorateur. C'est ce que `winit`
//!    faisait, et le reprendre est le prix de lui avoir pris sa place ;
//! 2. **`FileGroupDescriptorW` + `FileContents`** — le navigateur. Les octets s'écrivent dans
//!    le répertoire temporaire, et le dépôt redevient **exactement** un fichier glissé :
//!    [`crate::interactions::drop`] le route sans savoir d'où il vient ;
//! 3. **l'adresse seule** — une page qui ne promet aucun contenu, un lien glissé depuis la
//!    barre d'adresse. On pose alors le lien, que [`crate::interactions::links`] rend
//!    cliquable. Un repli visible vaut mieux qu'un geste sans effet.
//!
//! # Prendre la place de `winit` plutôt que de coexister
//!
//! Une fenêtre n'a **qu'une** cible de dépôt : `RegisterDragDrop` refuse la seconde avec
//! `DRAGDROP_E_ALREADYREGISTERED`. On révoque donc celle de `winit` avant de poser la nôtre.
//!
//! On ne lui demande pas de ne pas s'enregistrer, et c'est délibéré : c'est son
//! enregistrement qui fait l'`OleInitialize` du fil, dont `RegisterDragDrop` a besoin. Le lui
//! retirer nous obligerait à initialiser OLE nous-mêmes, sur un fil dont nous ne possédons
//! pas le cycle de vie.
//!
//! **Un échec ne casse rien** : si la révocation ou l'enregistrement refuse, on le dit et
//! `winit` garde la main. Le glisser depuis l'explorateur continue de marcher, et seul le
//! dépôt depuis le web manque — c'est-à-dire l'état d'avant.
//!
//! # `unsafe`, et où il s'arrête
//!
//! Tout le COM est ici. **Rien de ce qui décide n'y est** : le nom qu'on accepte d'une page,
//! l'endroit où les octets s'écrivent, ce qu'est une moisson vide sont dans
//! [`super::moisson`], sans un `unsafe`, et testés sur n'importe quelle machine. Ce fichier
//! lit des formats et copie des octets ; il ne juge de rien.

use super::moisson::{self, Moisson};
use std::sync::mpsc::Sender;
use windows::core::{implement, Interface, Ref, Result as WinResult, PCWSTR};
use windows::Win32::Foundation::{DRAGDROP_E_ALREADYREGISTERED, HGLOBAL, HWND, POINTL};
use windows::Win32::System::Com::{
    IDataObject, ISequentialStream, IStream, FORMATETC, STGMEDIUM, TYMED, TYMED_HGLOBAL,
    TYMED_ISTREAM,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop, CF_HDROP,
    CF_UNICODETEXT, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, FILEGROUPDESCRIPTORW, HDROP};

/// Combien d'octets au plus on accepte d'un seul fichier promis par une page.
///
/// Ce n'est pas une borne de confort : le flux est lu **dans le fil de l'interface**, pendant
/// que le navigateur le remplit, et une page peut en promettre autant qu'elle veut. Deux
/// cent cinquante-six mébioctets est ce qu'une image de très haute définition atteint au pire
/// — au-delà, ce n'est plus une image qu'on dépose sur un canevas.
const OCTETS_MAX: usize = 256 * 1024 * 1024;

/// Ce qu'on lit d'un coup dans le flux d'un fichier promis.
const TRANCHE: usize = 64 * 1024;

/// **Installe la cible de dépôt de Glucose sur cette fenêtre**, à la place de celle de `winit`.
///
/// Rend `false` quand Windows a refusé : l'application reste alors exactement ce qu'elle
/// était, `winit` compris.
///
/// # Sûreté
///
/// `hwnd` doit être la fenêtre vivante du fil courant, et ce fil doit être celui de la boucle
/// d'événements — c'est lui qui a initialisé OLE, et c'est lui qui appellera la cible.
pub fn installer(hwnd: isize, vers: Sender<Moisson>) -> bool {
    let fenetre = HWND(hwnd as *mut core::ffi::c_void);
    let cible: IDropTarget = Cible { vers }.into();
    unsafe {
        // **On tente d'abord, on révoque ensuite**, et cet ordre est ce qui rend l'échec sûr.
        //
        // Révoquer d'emblée laisserait la fenêtre **sans aucune** cible le jour où notre
        // enregistrement échoue pour une autre raison — OLE non initialisé, mémoire — et le
        // glisser-déposer de fichiers, qui marchait, cesserait de marcher. En commençant par
        // l'enregistrement, le seul cas où l'on révoque est celui où Windows a dit
        // explicitement que la place est prise : la reprendre est alors la seule chose qui
        // puisse réussir.
        match RegisterDragDrop(fenetre, &cible) {
            Ok(()) => return true,
            Err(e) if e.code() == DRAGDROP_E_ALREADYREGISTERED => {}
            Err(e) => {
                eprintln!("[Glucose] depot web : Windows a refuse la cible de depot ({e})");
                return false;
            }
        }
        if let Err(e) = RevokeDragDrop(fenetre) {
            eprintln!("[Glucose] depot web : la cible de winit ne se retire pas ({e})");
            return false;
        }
        match RegisterDragDrop(fenetre, &cible) {
            Ok(()) => true,
            Err(e) => {
                eprintln!("[Glucose] depot web : la place reprise reste refusee ({e})");
                false
            }
        }
    }
}

/// La cible de dépôt : elle récolte, et confie à la boucle d'images le soin de poser.
///
/// Elle ne touche **pas** au document. `Drop` est appelé depuis les entrailles de Windows,
/// au milieu d'une boucle de messages que nous n'avons pas écrite ; y modifier le document
/// reviendrait à le muter pendant que la boucle d'images le lit. Le canal est ce qui rend
/// l'instant du dépôt indépendant de l'instant où on le pose.
#[implement(IDropTarget)]
struct Cible {
    /// Par où la moisson rejoint la boucle d'images.
    vers: Sender<Moisson>,
}

#[allow(non_snake_case)]
impl IDropTarget_Impl for Cible_Impl {
    fn DragEnter(
        &self,
        donnees: Ref<'_, IDataObject>,
        _touches: MODIFIERKEYS_FLAGS,
        _ou: &POINTL,
        effet: *mut DROPEFFECT,
    ) -> WinResult<()> {
        ecrire_effet(effet, donnees.as_ref().is_some_and(porte_quelque_chose));
        Ok(())
    }

    fn DragOver(
        &self,
        _touches: MODIFIERKEYS_FLAGS,
        _ou: &POINTL,
        effet: *mut DROPEFFECT,
    ) -> WinResult<()> {
        // `DragOver` ne reçoit pas l'objet : ce que `DragEnter` a répondu vaut pour tout le
        // survol, et Windows ne nous redemande pas notre avis tant que rien ne change.
        ecrire_effet(effet, true);
        Ok(())
    }

    fn DragLeave(&self) -> WinResult<()> {
        Ok(())
    }

    fn Drop(
        &self,
        donnees: Ref<'_, IDataObject>,
        _touches: MODIFIERKEYS_FLAGS,
        ou: &POINTL,
        effet: *mut DROPEFFECT,
    ) -> WinResult<()> {
        ecrire_effet(effet, true);
        let Some(objet) = donnees.as_ref() else {
            return Ok(());
        };
        let mut recolte = recolter(objet);
        // `POINTL` est en pixels de l'écran ; la fenêtre les convertira en pixels à elle.
        recolte.ou = Some((f64::from(ou.x), f64::from(ou.y)));
        if !recolte.est_vide() {
            // Le récepteur peut avoir disparu si la fenêtre se ferme pendant un dépôt : ce
            // n'est pas une panne, c'est la fin.
            let _ = self.this.vers.send(recolte);
        }
        Ok(())
    }
}

/// Écrit dans `effet` ce que Windows doit montrer au curseur.
///
/// # Sûreté
///
/// `effet` vient de Windows et pointe sur une valeur valide pour la durée de l'appel.
fn ecrire_effet(effet: *mut DROPEFFECT, accepte: bool) {
    if effet.is_null() {
        return;
    }
    unsafe {
        *effet = if accepte {
            DROPEFFECT_COPY
        } else {
            DROPEFFECT_NONE
        };
    }
}

/// Cet objet porte-t-il quelque chose que nous sachions poser ?
fn porte_quelque_chose(objet: &IDataObject) -> bool {
    [
        CF_HDROP.0,
        format_enregistre("FileGroupDescriptorW"),
        format_enregistre("FileContents"),
        CF_UNICODETEXT.0,
        format_enregistre("UniformResourceLocatorW"),
    ]
    .into_iter()
    .any(|format| format != 0 && offre(objet, format, TYMED(u32::MAX as i32)))
}

/// Le numéro qu'un format porte sur cette session de Windows.
///
/// Les formats du shell n'ont pas de numéro fixe : chaque session en attribue un, et deux
/// programmes qui demandent le même nom reçoivent le même numéro. Zéro veut dire que Windows
/// a refusé, et un format à zéro ne se demande jamais.
fn format_enregistre(nom: &str) -> u16 {
    let large: Vec<u16> = nom.encode_utf16().chain(std::iter::once(0)).collect();
    // `RegisterClipboardFormatW` rend un `u32` dont seuls les seize bits bas servent de
    // format ; zéro signale l'échec, et c'est le seul cas que nous ayons à distinguer.
    let numero = unsafe {
        windows::Win32::System::DataExchange::RegisterClipboardFormatW(PCWSTR(large.as_ptr()))
    };
    u16::try_from(numero).unwrap_or(0)
}

/// Une demande de format, telle que Windows l'attend.
fn demande(format: u16, tymed: TYMED, index: i32) -> FORMATETC {
    FORMATETC {
        cfFormat: format,
        ptd: std::ptr::null_mut(),
        dwAspect: windows::Win32::System::Com::DVASPECT_CONTENT.0,
        lindex: index,
        tymed: tymed.0 as u32,
    }
}

/// Cet objet offre-t-il ce format ?
fn offre(objet: &IDataObject, format: u16, tymed: TYMED) -> bool {
    unsafe { objet.QueryGetData(&demande(format, tymed, -1)).is_ok() }
}

/// **Tout ce que ce dépôt apporte**, dans l'ordre du plus sûr au plus pauvre.
///
/// Le premier format qui donne quelque chose gagne : un navigateur offre souvent l'adresse
/// *en plus* des octets, et poser les deux mettrait deux nœuds là où l'utilisateur en a
/// déposé un.
fn recolter(objet: &IDataObject) -> Moisson {
    let par_fichiers = fichiers_reels(objet);
    if !par_fichiers.is_empty() {
        return Moisson {
            chemins: par_fichiers,
            ..Moisson::default()
        };
    }
    let promis = fichiers_promis(objet);
    if !promis.is_empty() {
        return Moisson {
            chemins: promis,
            ..Moisson::default()
        };
    }
    Moisson {
        liens: adresses(objet),
        ..Moisson::default()
    }
}

/// Les chemins d'un `CF_HDROP` — ce que `winit` transmettait, et rien de plus.
fn fichiers_reels(objet: &IDataObject) -> Vec<std::path::PathBuf> {
    let Some(medium) = tirer(objet, CF_HDROP.0, TYMED_HGLOBAL, -1) else {
        return Vec::new();
    };
    let mut chemins = Vec::new();
    unsafe {
        let poignee = HDROP(medium.u.hGlobal.0);
        let combien = DragQueryFileW(poignee, u32::MAX, None);
        for i in 0..combien {
            // Windows rend la longueur SANS le zéro terminal, et l'attend AVEC : un tampon
            // de la longueur exacte se fait tronquer d'un caractère, silencieusement.
            let taille = DragQueryFileW(poignee, i, None);
            if taille == 0 {
                continue;
            }
            let mut tampon = vec![0u16; taille as usize + 1];
            let ecrit = DragQueryFileW(poignee, i, Some(&mut tampon));
            if ecrit == 0 {
                continue;
            }
            chemins.push(std::path::PathBuf::from(String::from_utf16_lossy(
                &tampon[..ecrit as usize],
            )));
        }
        ReleaseStgMedium(&mut medium.clone());
    }
    chemins
}

/// **Les fichiers qu'une page promet**, écrits sur le disque pour redevenir des fichiers.
///
/// Le descripteur dit combien il y en a et comment ils s'appellent ; le contenu se demande
/// **un par un**, par son rang — c'est le seul format de Windows dont `lindex` désigne autre
/// chose que la totalité.
fn fichiers_promis(objet: &IDataObject) -> Vec<std::path::PathBuf> {
    let noms = noms_promis(objet);
    if noms.is_empty() {
        return Vec::new();
    }
    let Ok(dossier) = moisson::dossier() else {
        return Vec::new();
    };
    let contenus = format_enregistre("FileContents");
    noms.into_iter()
        .enumerate()
        .filter_map(|(rang, nom)| {
            let octets = octets_du_promis(objet, contenus, rang as i32)?;
            moisson::poser(&dossier, &nom, rang, &octets)
        })
        .collect()
}

/// Les noms que le descripteur de groupe annonce, dans son ordre.
fn noms_promis(objet: &IDataObject) -> Vec<String> {
    let format = format_enregistre("FileGroupDescriptorW");
    if format == 0 {
        return Vec::new();
    }
    let Some(mut medium) = tirer(objet, format, TYMED_HGLOBAL, -1) else {
        return Vec::new();
    };
    let mut noms = Vec::new();
    unsafe {
        if let Some(bloc) = Bloc::prendre(medium.u.hGlobal) {
            let groupe = bloc.pointeur.cast::<FILEGROUPDESCRIPTORW>();
            // La structure déclare un tableau d'un seul élément et en porte `cItems` : c'est
            // la forme que Windows emploie partout, et la lire autrement tronquerait le lot.
            let combien = (*groupe).cItems as usize;
            let premier = std::ptr::addr_of!((*groupe).fgd)
                .cast::<windows::Win32::UI::Shell::FILEDESCRIPTORW>();
            for i in 0..combien {
                // `FILEDESCRIPTORW` est compactee : y prendre une reference serait un
                // desalignement, donc un comportement indefini meme sans la dereferencer.
                // On LIT le champ, ce qui le copie a un endroit aligne.
                let brut = std::ptr::addr_of!((*premier.add(i)).cFileName).read_unaligned();
                let fin = brut.iter().position(|c| *c == 0).unwrap_or(brut.len());
                noms.push(String::from_utf16_lossy(&brut[..fin]));
            }
        }
        ReleaseStgMedium(&mut medium);
    }
    noms
}

/// Les octets du fichier promis de ce rang, par flux ou par bloc.
///
/// Le flux est ce que les navigateurs emploient — c'est lui qui leur permet de télécharger
/// pendant qu'on lit — et le bloc est le repli de ce qui tient déjà en mémoire.
fn octets_du_promis(objet: &IDataObject, format: u16, rang: i32) -> Option<Vec<u8>> {
    if let Some(mut medium) = tirer(objet, format, TYMED_ISTREAM, rang) {
        let octets = unsafe {
            let flux: Option<IStream> = (*medium.u.pstm).clone();
            let lu = flux.as_ref().and_then(lire_le_flux);
            ReleaseStgMedium(&mut medium);
            lu
        };
        if octets.is_some() {
            return octets;
        }
    }
    let mut medium = tirer(objet, format, TYMED_HGLOBAL, rang)?;
    unsafe {
        let octets = Bloc::prendre(medium.u.hGlobal).map(|bloc| bloc.copier());
        ReleaseStgMedium(&mut medium);
        octets
    }
}

/// Lit un flux jusqu'à sa fin, ou jusqu'à ce qu'il dépasse ce qu'on accepte.
///
/// Un flux qui ne finit pas est une page qui nous tient : la borne est ce qui garantit que
/// l'application reprend la main.
fn lire_le_flux(flux: &IStream) -> Option<Vec<u8>> {
    let sequentiel: ISequentialStream = flux.cast().ok()?;
    let mut tout = Vec::new();
    let mut tranche = vec![0u8; TRANCHE];
    loop {
        let mut lu = 0u32;
        let issue = unsafe {
            sequentiel.Read(
                tranche.as_mut_ptr().cast(),
                TRANCHE as u32,
                Some(&mut lu as *mut u32),
            )
        };
        if issue.is_err() || lu == 0 {
            break;
        }
        tout.extend_from_slice(&tranche[..lu as usize]);
        if tout.len() > OCTETS_MAX {
            eprintln!("[Glucose] depot web : le fichier promis depasse ce qu'on accepte");
            return None;
        }
    }
    (!tout.is_empty()).then_some(tout)
}

/// L'adresse seule, quand la page ne promet aucun contenu.
fn adresses(objet: &IDataObject) -> Vec<String> {
    for format in [
        format_enregistre("UniformResourceLocatorW"),
        CF_UNICODETEXT.0,
    ] {
        if format == 0 {
            continue;
        }
        let Some(mut medium) = tirer(objet, format, TYMED_HGLOBAL, -1) else {
            continue;
        };
        let texte = unsafe {
            let lu = Bloc::prendre(medium.u.hGlobal).map(|bloc| bloc.texte_large());
            ReleaseStgMedium(&mut medium);
            lu
        };
        if let Some(texte) = texte {
            let net = texte.trim().to_string();
            if !net.is_empty() {
                return vec![net];
            }
        }
    }
    Vec::new()
}

/// Demande un format à l'objet, et rend ce que Windows a posé.
///
/// Le `STGMEDIUM` rendu appartient à l'appelant : il doit le rendre par `ReleaseStgMedium`,
/// et c'est pour cela que chaque appelant le fait explicitement plutôt que de le confier à un
/// garde — un garde exigerait de posséder le medium, et `ReleaseStgMedium` le veut mutable.
fn tirer(objet: &IDataObject, format: u16, tymed: TYMED, index: i32) -> Option<STGMEDIUM> {
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
struct Bloc {
    poignee: HGLOBAL,
    pointeur: *mut core::ffi::c_void,
    taille: usize,
}

impl Bloc {
    /// Verrouille ce bloc, ou rend rien s'il est vide.
    ///
    /// # Sûreté
    ///
    /// `poignee` doit être une poignée de mémoire globale valide, telle que Windows vient de
    /// la poser dans un `STGMEDIUM`.
    unsafe fn prendre(poignee: HGLOBAL) -> Option<Self> {
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
    fn copier(&self) -> Vec<u8> {
        unsafe { std::slice::from_raw_parts(self.pointeur.cast::<u8>(), self.taille).to_vec() }
    }

    /// Le bloc lu comme du texte large, jusqu'à son premier zéro.
    fn texte_large(&self) -> String {
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
