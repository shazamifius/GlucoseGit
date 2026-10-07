//! **Un lot de nœuds dans le presse-papiers du système** (fiche 51 § 2) : ce qu'`arboard` ne
//! sait pas dire.
//!
//! # Pourquoi un format à soi sous Windows, et du HTML ailleurs
//!
//! Le lot est un petit `.glucose`, images comprises : il pèse ce que pèsent ses photos. Les
//! logiciels sur le web (tldraw, Figma) le glissent dans du HTML, parce qu'un navigateur n'a pas
//! d'autre canal ; mais tout logiciel qui colle du HTML — Word, un client de courrier — recevrait
//! alors des mégaoctets de base64 qu'il ne sait pas lire. Windows offre ce que le web n'a pas :
//! un **format enregistré** (`RegisterClipboardFormat`), que les autres logiciels ne demandent
//! jamais, et qui porte les octets tels quels, sans base64. Le texte des nœuds l'accompagne,
//! dans la même ouverture du presse-papiers : c'est ce que les autres logiciels collent.
//!
//! Ailleurs, `arboard` ne connaît pas de format à soi : le lot voyage dans du HTML, comme chez
//! tldraw, avec le même texte de repli. C'est la seule voie qu'il ouvre sur Linux et macOS.
//!
//! # La longueur voyage avec le lot
//!
//! `GlobalSize` rend la taille du **bloc**, que Windows peut arrondir : huit octets de longueur
//! précèdent donc le lot, et la lecture n'en prend pas un de plus.

/// Le nom du format. Il ne change jamais : deux versions de Glucose doivent se reconnaître, et
/// c'est le `.glucose` qu'il porte qui dit son schéma.
#[cfg(windows)]
const FORMAT: &str = "Glucose.Lot";

/// Écrit le lot, et le texte que les autres logiciels colleront à sa place.
pub fn ecrire_un_lot(texte: Option<&str>, lot: &[u8]) -> Result<(), String> {
    imp::ecrire_un_lot(texte, lot)
}

/// Le lot que porte le presse-papiers, s'il en porte un.
pub fn lire_un_lot() -> Option<Vec<u8>> {
    imp::lire_un_lot()
}

/// **Le lot, précédé de sa longueur** : le presse-papiers et le glisser l'emportent ainsi.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn envelopper_le_lot(lot: &[u8]) -> Vec<u8> {
    let mut enveloppe = Vec::with_capacity(8 + lot.len());
    enveloppe.extend_from_slice(&(lot.len() as u64).to_le_bytes());
    enveloppe.extend_from_slice(lot);
    enveloppe
}

/// **Le lot qu'une enveloppe porte**, sans les octets qu'un bloc arrondi ajoute après lui.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn deballer_le_lot(tout: &[u8]) -> Option<Vec<u8>> {
    let n = usize::try_from(u64::from_le_bytes(tout.get(..8)?.try_into().ok()?)).ok()?;
    tout.get(8..8usize.checked_add(n)?).map(<[u8]>::to_vec)
}

/// Un texte comme `CF_UNICODETEXT` le veut : les fins de ligne de Windows, en UTF-16, et le zéro
/// final que le format exige.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn texte_large(texte: &str) -> Vec<u8> {
    texte
        .replace('\n', "\r\n")
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

/// Le numéro que Windows donne au format du lot, sur cette session.
#[cfg(windows)]
pub(crate) fn format_du_lot() -> Result<u32, String> {
    imp::format()
}

/// **Un bloc de mémoire globale qui porte ces octets** : ce que le presse-papiers et le
/// glisser confient au système. Une fois confié, il est à lui.
#[cfg(windows)]
pub(crate) fn bloc_global(octets: &[u8]) -> Result<windows::Win32::Foundation::HGLOBAL, String> {
    imp::bloc_global(octets)
}

/// Écrit une image comme le fait « Copier l'image » d'un navigateur (fiche 51 § 3).
pub fn ecrire_une_image(image: &crate::interactions::clipboard::ImagePosee) -> Result<(), String> {
    imp::ecrire_une_image(image)
}

#[cfg(windows)]
mod imp {
    use super::FORMAT;
    use windows::core::HSTRING;
    use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, RegisterClipboardFormatW,
        SetClipboardData,
    };
    use windows::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
    };
    use windows::Win32::System::Ole::{CF_DIBV5, CF_UNICODETEXT};

    pub(super) fn format() -> Result<u32, String> {
        match unsafe { RegisterClipboardFormatW(&HSTRING::from(FORMAT)) } {
            0 => Err("le format de Glucose n'a pas pu être enregistré".into()),
            f => Ok(f),
        }
    }

    /// Le presse-papiers ouvert ; il se referme quand on le lâche, quoi qu'il arrive.
    struct Ouvert;

    impl Ouvert {
        /// Un autre logiciel peut le tenir un instant : on réessaie, brièvement, comme arboard.
        fn prendre() -> Result<Self, String> {
            for _ in 0..10 {
                if unsafe { OpenClipboard(None) }.is_ok() {
                    return Ok(Self);
                }
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Err("le presse-papiers est tenu par un autre logiciel".into())
        }
    }

    impl Drop for Ouvert {
        fn drop(&mut self) {
            let _ = unsafe { CloseClipboard() };
        }
    }

    pub fn ecrire_un_lot(texte: Option<&str>, lot: &[u8]) -> Result<(), String> {
        let format = format()?;
        let enveloppe = super::envelopper_le_lot(lot);
        let _ouvert = Ouvert::prendre()?;
        unsafe { EmptyClipboard() }.map_err(|e| e.to_string())?;
        if let Some(t) = texte {
            poser(u32::from(CF_UNICODETEXT.0), &super::texte_large(t))?;
        }
        poser(format, &enveloppe)
    }

    pub(super) fn bloc_global(octets: &[u8]) -> Result<HGLOBAL, String> {
        unsafe {
            let bloc =
                GlobalAlloc(GMEM_MOVEABLE, octets.len().max(1)).map_err(|e| e.to_string())?;
            let ou = GlobalLock(bloc).cast::<u8>();
            if ou.is_null() {
                let _ = GlobalFree(Some(bloc));
                return Err("mémoire du presse-papiers refusée".into());
            }
            std::ptr::copy_nonoverlapping(octets.as_ptr(), ou, octets.len());
            let _ = GlobalUnlock(bloc);
            Ok(bloc)
        }
    }

    /// Confie ces octets au système sous ce format. Une fois confié, le bloc est à lui.
    fn poser(format: u32, octets: &[u8]) -> Result<(), String> {
        let bloc = bloc_global(octets)?;
        unsafe {
            if let Err(e) = SetClipboardData(format, Some(HANDLE(bloc.0))) {
                let _ = GlobalFree(Some(bloc));
                return Err(e.to_string());
            }
        }
        Ok(())
    }

    /// Le PNG sous le format enregistré « PNG », que Chromium lit d'abord ; les pixels en
    /// `CF_DIBV5`, d'où Windows tire le bitmap de tous les autres.
    pub fn ecrire_une_image(
        image: &crate::interactions::clipboard::ImagePosee,
    ) -> Result<(), String> {
        let png = match unsafe { RegisterClipboardFormatW(&HSTRING::from("PNG")) } {
            0 => return Err("le format PNG n'a pas pu être enregistré".into()),
            f => f,
        };
        let dib = super::dib_v5(image.largeur, image.hauteur, &image.rgba);
        let _ouvert = Ouvert::prendre()?;
        unsafe { EmptyClipboard() }.map_err(|e| e.to_string())?;
        poser(png, &image.png)?;
        poser(u32::from(CF_DIBV5.0), &dib)
    }

    pub fn lire_un_lot() -> Option<Vec<u8>> {
        let format = format().ok()?;
        let _ouvert = Ouvert::prendre().ok()?;
        unsafe {
            let bloc = HGLOBAL(GetClipboardData(format).ok()?.0);
            let taille = GlobalSize(bloc);
            let ou = GlobalLock(bloc).cast::<u8>().cast_const();
            if ou.is_null() {
                return None;
            }
            // Sûr : le système garantit `taille` octets lisibles tant que le bloc est verrouillé.
            let lot = super::deballer_le_lot(std::slice::from_raw_parts(ou, taille));
            let _ = GlobalUnlock(bloc);
            lot
        }
    }
}

#[cfg(all(not(windows), not(target_os = "android")))]
mod imp {
    /// L'attribut qui porte le lot, en base64, dans le HTML.
    const ATTRIBUT: &str = "data-glucose-lot";

    pub fn ecrire_un_lot(texte: Option<&str>, lot: &[u8]) -> Result<(), String> {
        let html = super::html::envelopper(ATTRIBUT, texte.unwrap_or(""), lot);
        arboard::Clipboard::new()
            .and_then(|mut c| c.set_html(html, texte.map(str::to_string)))
            .map_err(|e| e.to_string())
    }

    pub fn lire_un_lot() -> Option<Vec<u8>> {
        let html = arboard::Clipboard::new().ok()?.get().html().ok()?;
        super::html::deballer(ATTRIBUT, &html)
    }

    pub fn ecrire_une_image(
        image: &crate::interactions::clipboard::ImagePosee,
    ) -> Result<(), String> {
        arboard::Clipboard::new()
            .and_then(|mut c| {
                c.set_image(arboard::ImageData {
                    width: image.largeur as usize,
                    height: image.hauteur as usize,
                    bytes: std::borrow::Cow::Borrowed(&image.rgba),
                })
            })
            .map_err(|e| e.to_string())
    }
}

/// Sous Android, Glucose n'atteint pas encore le presse-papiers du système (fiche 54) : son
/// propre presse-papiers le remplace ([`crate::interactions::presse_papiers`]), et ces portes ne
/// sont jamais appelées — elles le disent si elles l'étaient.
#[cfg(target_os = "android")]
mod imp {
    const PAS_ENCORE: &str = "le presse-papiers du systeme n'est pas encore pris sous Android";

    pub fn ecrire_un_lot(_texte: Option<&str>, _lot: &[u8]) -> Result<(), String> {
        Err(PAS_ENCORE.into())
    }

    pub fn lire_un_lot() -> Option<Vec<u8>> {
        None
    }

    pub fn ecrire_une_image(
        _image: &crate::interactions::clipboard::ImagePosee,
    ) -> Result<(), String> {
        Err(PAS_ENCORE.into())
    }
}

/// **Les pixels en `CF_DIBV5`** : l'en-tête `BITMAPV5HEADER`, puis les rangées **du bas vers le
/// haut**, en bleu-vert-rouge-alpha. Une hauteur négative dirait « du haut vers le bas » sans
/// rien retourner, mais Word et WordPad refusent alors de coller — arboard l'a appris avant nous.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn dib_v5(largeur: u32, hauteur: u32, rgba: &[u8]) -> Vec<u8> {
    const EN_TETE: u32 = 124;
    const BI_BITFIELDS: u32 = 3;
    const LCS_SRGB: u32 = 0x7352_4742;
    const LCS_GM_IMAGES: u32 = 4;
    let taille = 4 * largeur * hauteur;
    let mut o = Vec::with_capacity((EN_TETE + taille) as usize);
    for mot in [EN_TETE, largeur, hauteur] {
        o.extend_from_slice(&mot.to_le_bytes());
    }
    o.extend_from_slice(&1u16.to_le_bytes()); // plans
    o.extend_from_slice(&32u16.to_le_bytes()); // bits par pixel
    for mot in [BI_BITFIELDS, taille, 0, 0, 0, 0] {
        o.extend_from_slice(&mot.to_le_bytes());
    }
    for masque in [
        0x00ff_0000u32,
        0x0000_ff00,
        0x0000_00ff,
        0xff00_0000,
        LCS_SRGB,
    ] {
        o.extend_from_slice(&masque.to_le_bytes());
    }
    o.extend_from_slice(&[0u8; 36]); // les extrémités, ignorées hors LCS_CALIBRATED_RGB
    for mot in [0u32, 0, 0, LCS_GM_IMAGES, 0, 0, 0] {
        o.extend_from_slice(&mot.to_le_bytes());
    }
    debug_assert_eq!(o.len(), EN_TETE as usize);
    for rangee in rgba.chunks_exact(4 * largeur as usize).rev() {
        for p in rangee.as_chunks::<4>().0 {
            o.extend_from_slice(&[p[2], p[1], p[0], p[3]]);
        }
    }
    o
}

/// Le lot dans du HTML, pour les systèmes où le presse-papiers n'a pas de format à soi. Écrit
/// partout, pour que ses épreuves tournent aussi sous Windows.
// Les bureaux Linux et Mac seuls le portent : Windows a son format, Android pas encore de porte.
#[cfg_attr(any(windows, target_os = "android"), allow(dead_code))]
pub(crate) mod html {
    /// Le texte visible, échappé, et le lot dans un attribut — que le navigateur ignore.
    pub fn envelopper(attribut: &str, texte: &str, lot: &[u8]) -> String {
        let visible: String = texte
            .chars()
            .map(|c| match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                '\n' => "<br>".to_string(),
                c => c.to_string(),
            })
            .collect();
        format!(
            "<meta charset=\"utf-8\"><div {attribut}=\"{}\">{visible}</div>",
            base64(lot)
        )
    }

    /// Le lot qu'un HTML porte, s'il en porte un.
    pub fn deballer(attribut: &str, html: &str) -> Option<Vec<u8>> {
        let debut = html.find(&format!("{attribut}=\""))? + attribut.len() + 2;
        let fin = debut + html[debut..].find('"')?;
        glucose_core::persist::tauri::provenance::base64(&html[debut..fin])
    }

    /// Le base64 standard (RFC 4648), que le noyau sait relire.
    fn base64(octets: &[u8]) -> String {
        const ALPHABET: &[u8; 64] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut s = String::with_capacity(octets.len().div_ceil(3) * 4);
        for bloc in octets.chunks(3) {
            let n = bloc
                .iter()
                .enumerate()
                .fold(0u32, |n, (i, &o)| n | u32::from(o) << (16 - 8 * i));
            for i in 0..4 {
                if i <= bloc.len() {
                    s.push(char::from(ALPHABET[(n >> (18 - 6 * i)) as usize & 63]));
                } else {
                    s.push('=');
                }
            }
        }
        s
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// **Le lot traverse le HTML intact**, toutes longueurs de reste comprises, et le texte
        /// visible ne peut pas fermer la balise qui le porte.
        #[test]
        fn test_le_lot_traverse_le_html() {
            for n in 0..9u8 {
                let lot: Vec<u8> = (0..n).map(|i| i.wrapping_mul(37)).collect();
                let html = envelopper("data-x", "a <b> & \"c\"\nd", &lot);
                assert_eq!(deballer("data-x", &html), Some(lot));
                assert!(html.contains("a &lt;b&gt; &amp;"));
            }
        }
    }
}
