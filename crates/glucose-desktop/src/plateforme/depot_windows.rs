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
//! 3. **le bitmap** — beaucoup de pages n'offrent pas de fichier promis mais posent l'image
//!    décompressée dans le presse-papiers du glisser, exactement comme un `Ctrl+C` sur une
//!    image. Elle s'écrit en PNG et redevient un fichier comme les deux formats précédents ;
//! 4. **l'adresse seule** — une page qui ne promet aucun contenu, un lien glissé depuis la
//!    barre d'adresse. On pose alors le lien, que [`crate::interactions::links`] rend
//!    cliquable. Un repli visible vaut mieux qu'un geste sans effet.
//!
//! # Ce que Pinterest a appris, et l'instrument qui en est né
//!
//! *« Lorsqu'on importe depuis Pinterest ça ne fonctionne pas ; il faut remonter le lien
//! jusqu'à trouver l'image. »* Ce qu'on glisse depuis une grille de Pinterest n'est pas une
//! image mais un **lien** : le navigateur n'a alors ni fichier promis ni bitmap à offrir, et
//! aucune supposition sur ce qu'il offre ne remplace la liste de ce qu'il a réellement offert.
//!
//! [`dire_les_formats`] l'écrit, sur demande — `GLUCOSE_DEPOT=1`. C'est un instrument, pas une
//! trace : il se déclenche chez celui qui fait le geste, là où le geste a lieu.
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

mod formats;

use formats::{dire_les_formats, format_enregistre, offre, tirer, Bloc};

use super::moisson::{self, Depot, Moisson};
use std::sync::mpsc::Sender;
use windows::core::{implement, Interface, Ref, Result as WinResult};
use windows::Win32::Foundation::{DRAGDROP_E_ALREADYREGISTERED, HWND, POINTL};
use windows::Win32::System::Com::{
    IDataObject, ISequentialStream, IStream, TYMED, TYMED_HGLOBAL, TYMED_ISTREAM,
};
use windows::Win32::System::Ole::{
    IDropTarget, IDropTarget_Impl, RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop, CF_DIB,
    CF_DIBV5, CF_HDROP, CF_UNICODETEXT, DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_NONE,
};
use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;
use windows::Win32::UI::Shell::{DragQueryFileW, FILEGROUPDESCRIPTORW, HDROP};

/// La borne d'un fichier promis : le flux est lu **dans le fil de l'interface**, pendant que le
/// navigateur le remplit, et une page peut en promettre autant qu'elle veut.
use moisson::OCTETS_MAX;

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
pub fn installer(hwnd: isize, vers: Sender<Depot>, reveil: super::Reveil) -> bool {
    let fenetre = HWND(hwnd as *mut core::ffi::c_void);
    let cible: IDropTarget = Cible { vers, reveil }.into();
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
    vers: Sender<Depot>,
    /// De quoi la réveiller quand une image rapatriée arrive après coup.
    reveil: super::Reveil,
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
        let (mut recolte, sorte) = recolter(objet);
        // `POINTL` est en pixels de l'écran ; la fenêtre les convertira en pixels à elle.
        recolte.ou = Some((f64::from(ou.x), f64::from(ou.y)));
        if let Sorte::Repli = sorte {
            let adresses = adresses_a_rapatrier(objet, &recolte);
            if !adresses.is_empty() {
                super::rapatrier::rapatrier(
                    adresses,
                    recolte,
                    (self.this.vers.clone(), self.this.reveil.clone()),
                );
                return Ok(());
            }
        }
        if !recolte.est_vide() {
            // Le récepteur peut avoir disparu si la fenêtre se ferme pendant un dépôt : ce
            // n'est pas une panne, c'est la fin.
            let pose = Depot::Pose {
                numero: None,
                moisson: recolte,
            };
            self.this.vers.send(pose).ok();
        }
        Ok(())
    }
}

/// **Toutes les adresses que ce dépôt porte** : celles des formats de texte et de page, et
/// celles des raccourcis qu'on s'apprêtait à poser en liens.
fn adresses_a_rapatrier(objet: &IDataObject, repli: &Moisson) -> Vec<String> {
    let mut adresses = formats::adresses_portees(objet);
    adresses.extend(moisson::lire_les_raccourcis(&repli.chemins).1);
    adresses.extend(repli.liens.iter().cloned());
    adresses
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
        CF_DIBV5.0,
        CF_DIB.0,
        CF_UNICODETEXT.0,
        format_enregistre("UniformResourceLocatorW"),
        // **Ce que Pinterest porte, et rien d'autre** (DEPOT-WEB-4) : une page, ou les donnees
        // qu'elle pose elle-meme. Le curseur montrait « interdit » a l'entree alors que
        // l'adresse de l'image s'y trouvait peut-etre.
        format_enregistre("HTML Format"),
        format_enregistre("Chromium Web Custom MIME Data Format"),
    ]
    .into_iter()
    .any(|format| format != 0 && offre(objet, format, TYMED(u32::MAX as i32)))
}

/// **Tout ce que ce dépôt apporte**, dans l'ordre du plus sûr au plus pauvre.
///
/// Le premier format qui donne quelque chose gagne : un navigateur offre souvent l'adresse
/// *en plus* des octets, et poser les deux mettrait deux nœuds là où l'utilisateur en a
/// déposé un.
fn recolter(objet: &IDataObject) -> (Moisson, Sorte) {
    dire_les_formats(objet);
    let par_fichiers = fichiers_reels(objet);
    if !par_fichiers.is_empty() {
        let m = Moisson {
            chemins: par_fichiers,
            ..Moisson::default()
        };
        return (m, Sorte::Contenu);
    }
    // **Un raccourci promis passe APRES le bitmap** (DEPOT-WEB-2). Glisser une epingle depuis
    // Pinterest fait promettre a Chrome un `.url` -- une adresse habillee en fichier, le
    // format le plus pauvre qui soit. Le prendre au deuxieme rang faisait passer une adresse
    // devant l'image que la page posait peut-etre a cote.
    let promis = fichiers_promis(objet);
    let que_des_raccourcis = promis.iter().all(|p| moisson::est_un_raccourci(p));
    if !promis.is_empty() && !que_des_raccourcis {
        let m = Moisson {
            chemins: promis,
            ..Moisson::default()
        };
        return (m, Sorte::Contenu);
    }
    // **Tout le reste est un repli** (DEPOT-WEB-4) : la vignette que la page a posee -- a la
    // taille ou elle l'affichait --, des raccourcis, une adresse. S'il y a mieux a rapatrier,
    // c'est le pont qui le cherchera.
    let bitmap = bitmap_pose(objet);
    let repli = if !bitmap.is_empty() {
        Moisson {
            chemins: bitmap,
            ..Moisson::default()
        }
    } else if !promis.is_empty() {
        // Les raccourcis se poseront en liens : `drop` lit leur adresse.
        Moisson {
            chemins: promis,
            ..Moisson::default()
        }
    } else {
        Moisson {
            liens: adresses(objet),
            ..Moisson::default()
        }
    };
    (repli, Sorte::Repli)
}

/// **Ce que la récolte a trouvé** : l'image elle-même, ou seulement de quoi se replier.
enum Sorte {
    /// De vrais fichiers, ou ceux que le navigateur a téléchargés lui-même : on les pose.
    Contenu,
    /// Une vignette, des raccourcis, des adresses : on cherche mieux avant de s'y résoudre.
    Repli,
}

/// **L'image décompressée que la page a posée dans le presse-papiers du glisser**, écrite en
/// fichier.
///
/// Beaucoup de pages n'offrent aucun fichier promis mais posent le bitmap, exactement comme un
/// `Ctrl+C` sur une image — c'est d'ailleurs ce que `paste_from_clipboard` lit déjà depuis
/// toujours, et c'est pour cela que le collage marchait là où le glisser ne marchait pas.
///
/// Le DIB de Windows est **presque** un fichier BMP : il lui manque quatorze octets d'en-tête.
/// Les ajouter laisse le décodeur du projet faire le reste — palettes, masques, orientation —
/// plutôt que d'écrire un second décodeur d'images dans un module de COM.
fn bitmap_pose(objet: &IDataObject) -> Vec<std::path::PathBuf> {
    // `CF_DIBV5` d'abord : il porte l'espace colorimétrique et la transparence, que `CF_DIB`
    // perd. Une page qui offre les deux offre le même contenu, en moins bien pour le second.
    for format in [CF_DIBV5.0, CF_DIB.0] {
        let Some(mut medium) = tirer(objet, format, TYMED_HGLOBAL, -1) else {
            continue;
        };
        let dib = unsafe {
            let lu = Bloc::prendre(medium.u.hGlobal).map(|bloc| bloc.copier());
            ReleaseStgMedium(&mut medium);
            lu
        };
        let Some(dib) = dib else { continue };
        let Some(bmp) = en_fichier_bmp(&dib) else {
            continue;
        };
        let Ok(dossier) = moisson::dossier() else {
            return Vec::new();
        };
        if let Some(chemin) = moisson::poser(&dossier, "image.bmp", 0, &bmp) {
            return vec![chemin];
        }
    }
    Vec::new()
}

/// Les quatorze octets qui font d'un DIB un fichier BMP.
///
/// L'offset des pixels se **calcule** depuis l'en-tête : sa taille, sa palette, et les trois
/// masques qu'un `BI_BITFIELDS` ajoute. Le supposer à quarante octets marcherait sur le cas
/// courant et donnerait une image décalée sur tous les autres — le genre de faute qui ne se
/// voit que chez quelqu'un d'autre.
fn en_fichier_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    const BI_BITFIELDS: u32 = 3;
    let lire =
        |i: usize| -> Option<u32> { Some(u32::from_le_bytes(dib.get(i..i + 4)?.try_into().ok()?)) };
    let taille_entete = lire(0)? as usize;
    if taille_entete < 12 || taille_entete > dib.len() {
        return None;
    }
    // Un en-tête de douze octets est l'ancien `BITMAPCOREHEADER`, dont les champs ne sont pas
    // aux mêmes places. On ne le traite pas : plus aucun navigateur n'en produit, et le
    // traiter à moitié serait pire que de le refuser.
    if taille_entete < 40 {
        return None;
    }
    let bits = u32::from(u16::from_le_bytes(dib.get(14..16)?.try_into().ok()?));
    let compression = lire(16)?;
    let couleurs = lire(32)? as usize;
    let palette = if bits <= 8 {
        let entrees = if couleurs > 0 {
            couleurs
        } else {
            1usize << bits
        };
        entrees * 4
    } else {
        0
    };
    // Les masques ne suivent l'en-tête que pour un `BITMAPINFOHEADER` ; un V4 ou un V5 les
    // porte dans ses propres champs.
    let masques = if compression == BI_BITFIELDS && taille_entete == 40 {
        12
    } else {
        0
    };
    let debut = 14 + taille_entete + palette + masques;
    let total = 14 + dib.len();
    let mut bmp = Vec::with_capacity(total);
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&(total as u32).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&(debut as u32).to_le_bytes());
    bmp.extend_from_slice(dib);
    Some(bmp)
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

#[cfg(test)]
mod tests {
    use super::en_fichier_bmp;

    /// Un DIB tel que Windows le pose : l'en-tête d'un BMP moins ses quatorze premiers octets.
    fn dib_depuis_un_bmp(bmp: &[u8]) -> Vec<u8> {
        bmp[14..].to_vec()
    }

    /// **Un DIB redevient un BMP que le décodeur du projet sait lire**, et l'image en sort
    /// intacte.
    ///
    /// C'est tout ce qu'on demande à ces quatorze octets, et c'est exactement ce qui se casse
    /// sans qu'on le voie : un offset de pixels supposé à quarante marche sur le cas courant et
    /// donne une image décalée dès qu'une palette ou des masques s'intercalent.
    #[test]
    fn test_un_dib_redevient_un_bmp_lisible() {
        for (largeur, hauteur) in [(7u32, 5u32), (64, 1), (1, 64)] {
            let mut source = image::RgbaImage::new(largeur, hauteur);
            for (x, y, p) in source.enumerate_pixels_mut() {
                *p = image::Rgba([(x * 37 % 256) as u8, (y * 91 % 256) as u8, 40, 255]);
            }
            let mut bmp = std::io::Cursor::new(Vec::new());
            source
                .write_to(&mut bmp, image::ImageFormat::Bmp)
                .expect("ecriture BMP");
            let bmp = bmp.into_inner();

            let refait = en_fichier_bmp(&dib_depuis_un_bmp(&bmp)).expect("un DIB se recompose");
            let relu = image::load_from_memory(&refait)
                .expect("le BMP recompose doit se lire")
                .to_rgba8();
            assert_eq!(relu.dimensions(), (largeur, hauteur));
            for (x, y, p) in relu.enumerate_pixels() {
                assert_eq!(
                    p.0[..3],
                    source.get_pixel(x, y).0[..3],
                    "pixel ({x}, {y}) de l'image {largeur} x {hauteur}"
                );
            }
        }
    }

    /// **Ce qui n'est pas un DIB est refusé**, plutôt que de produire un fichier illisible que
    /// le routage du dépôt prendrait pour un lanceur.
    #[test]
    fn test_ce_qui_n_est_pas_un_dib_est_refuse() {
        assert!(en_fichier_bmp(&[]).is_none());
        assert!(
            en_fichier_bmp(&[0, 0, 0, 0]).is_none(),
            "un en-tete de taille nulle"
        );
        // Un `BITMAPCOREHEADER` de douze octets : refusé sciemment, ses champs ne sont pas aux
        // mêmes places et plus aucun navigateur n'en produit.
        let mut core = vec![0u8; 12];
        core[0] = 12;
        assert!(en_fichier_bmp(&core).is_none());
        // Un en-tête qui annonce plus grand que ce qu'il porte.
        let mut menteur = vec![0u8; 40];
        menteur[0] = 200;
        assert!(en_fichier_bmp(&menteur).is_none());
    }
}
