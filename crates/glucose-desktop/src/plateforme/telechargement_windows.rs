//! **Télécharger une adresse**, avec ce que Windows sait déjà faire (DEPOT-WEB-4).
//!
//! # Pourquoi WinHTTP, et pourquoi aucune dépendance de plus
//!
//! Télécharger en HTTPS demande une pile TLS, et en écrire une serait absurde ; en tirer une
//! de l'écosystème — `ureq`, `rustls`, leurs certificats — ajouterait une vingtaine de
//! caisses. Windows porte la sienne depuis toujours, celle de tous ses programmes :
//! **WinHTTP**, avec ses certificats, son proxy et ses mises à jour de sécurité. La caisse
//! `windows` est déjà là pour le dépôt ; il ne s'agit que d'en nommer un recoin de plus.
//!
//! La note [`decisions/02`](../../../../docs/carnet/decisions/02-TELECHARGER-CE-QU-ON-DEPOSE.md)
//! dit le reste. Ce module ne décide de rien : il reçoit une adresse déjà découpée et jugée
//! par [`super::sources`], et rend des octets ou la raison de n'en pas rendre.

use super::sources::Adresse;
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Networking::WinHttp::{
    WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryDataAvailable,
    WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest,
    WinHttpSetOption, WinHttpSetTimeouts, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
    WINHTTP_DECOMPRESSION_FLAG_DEFLATE, WINHTTP_DECOMPRESSION_FLAG_GZIP, WINHTTP_FLAG_SECURE,
    WINHTTP_OPEN_REQUEST_FLAGS, WINHTTP_OPTION_DECOMPRESSION, WINHTTP_QUERY_FLAG_NUMBER,
    WINHTTP_QUERY_STATUS_CODE,
};

use super::{DELAI, NAVIGATEUR};

/// Le délai de chaque étape, comme WinHTTP le compte : en millisecondes.
const DELAI_MS: i32 = DELAI.as_millis() as i32;

/// Une poignée WinHTTP, fermée quoi qu'il arrive.
struct Poignee(*mut core::ffi::c_void);

impl Drop for Poignee {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { WinHttpCloseHandle(self.0).ok() };
        }
    }
}

impl Poignee {
    /// La poignée, ou l'étape qui a échoué.
    fn ou(self, etape: &str) -> Result<Self, String> {
        if self.0.is_null() {
            Err(format!("{etape} : {}", windows::core::Error::from_thread()))
        } else {
            Ok(self)
        }
    }
}

/// **Les octets que cette adresse rend**, au plus `limite`, ou la raison de n'en pas rendre.
///
/// Seul un `200` compte : une redirection est suivie par WinHTTP lui-même, et tout le reste —
/// une page introuvable, un refus — n'est pas une image.
pub(super) fn telecharger(adresse: &Adresse, limite: usize) -> Result<Vec<u8>, String> {
    let requete = ouvrir(adresse)?;
    unsafe {
        WinHttpSendRequest(requete.requete.0, None, None, 0, 0, 0)
            .map_err(|e| raison("envoi", &e))?;
        WinHttpReceiveResponse(requete.requete.0, core::ptr::null_mut())
            .map_err(|e| raison("reponse", &e))?;
    }
    let statut = statut(&requete.requete)?;
    if statut != 200 {
        return Err(format!("le serveur repond {statut}"));
    }
    lire_le_corps(&requete.requete, limite)
}

/// **Ce qu'une étape a rencontré, dit en mots** (DEPOT-WEB-6) : la phrase de Windows est longue,
/// traduite, et ne dit pas l'essentiel — le réseau manquait-il, ou le serveur s'est-il tu ? Les
/// quatre causes qu'un dépôt rencontre ont leur nom ; les autres gardent celle de Windows.
///
/// Le 07/10, six épingles sont devenues six liens à seize secondes d'intervalle : le délai de
/// chaque étape, épuisé, et rien ne l'avait dit.
fn raison(etape: &str, e: &windows::core::Error) -> String {
    // Les erreurs de WinHTTP arrivent en `HRESULT_FROM_WIN32` : le code est dans les seize bits
    // bas, sous la facilité 7.
    let code = e.code().0 as u32;
    let win32 = (code >> 16 == 0x8007).then_some(code & 0xFFFF);
    let dit = match win32 {
        Some(12002) => "délai dépassé",
        Some(12007) => "nom introuvable, le réseau manque peut-être",
        Some(12029) => "connexion impossible",
        Some(12030) => "connexion coupée",
        Some(12175) => "connexion sécurisée refusée",
        _ => return format!("{etape} : {e}"),
    };
    format!("{dit} ({etape})")
}

/// La requête, et ce qu'elle ne doit pas survivre : les champs se ferment dans l'ordre où ils
/// sont écrits — la requête, puis sa connexion, puis la session.
struct Requete {
    requete: Poignee,
    _connexion: Poignee,
    _session: Poignee,
}

/// Ouvre la requête `GET` de cette adresse.
fn ouvrir(adresse: &Adresse) -> Result<Requete, String> {
    unsafe {
        let session = Poignee(WinHttpOpen(
            &HSTRING::from(NAVIGATEUR),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        ))
        .ou("session")?;
        WinHttpSetTimeouts(session.0, DELAI_MS, DELAI_MS, DELAI_MS, DELAI_MS)
            .map_err(|e| format!("delais : {e}"))?;
        // **Demander la page compressée, comme tout navigateur** (DEPOT-WEB-6). WinHTTP ne
        // le fait que si on le lui dit, et une page d'épingle pèse 1,2 Mo nue contre 127 Ko
        // en gzip : mesurée le 07/10, 20 s au lieu d'une, et jusqu'à 53 — assez pour qu'une
        // pause du serveur épuise le délai d'une étape, et qu'un dépôt devienne un lien.
        // WinHTTP décompresse lui-même ; un Windows qui ne sait pas (avant 8.1) refuse
        // l'option, et l'on télécharge comme avant.
        let toutes =
            (WINHTTP_DECOMPRESSION_FLAG_GZIP | WINHTTP_DECOMPRESSION_FLAG_DEFLATE).to_ne_bytes();
        WinHttpSetOption(Some(session.0), WINHTTP_OPTION_DECOMPRESSION, Some(&toutes)).ok();
        let connexion = Poignee(WinHttpConnect(
            session.0,
            &HSTRING::from(adresse.hote.as_str()),
            adresse.port,
            0,
        ))
        .ou("connexion")?;
        let drapeaux = if adresse.securise {
            WINHTTP_FLAG_SECURE
        } else {
            WINHTTP_OPEN_REQUEST_FLAGS(0)
        };
        let requete = Poignee(WinHttpOpenRequest(
            connexion.0,
            w!("GET"),
            &HSTRING::from(adresse.chemin.as_str()),
            PCWSTR::null(),
            PCWSTR::null(),
            core::ptr::null(),
            drapeaux,
        ))
        .ou("requete")?;
        Ok(Requete {
            requete,
            _connexion: connexion,
            _session: session,
        })
    }
}

/// Le code de statut de la réponse.
fn statut(requete: &Poignee) -> Result<u32, String> {
    let mut code = 0u32;
    let mut taille = core::mem::size_of::<u32>() as u32;
    unsafe {
        WinHttpQueryHeaders(
            requete.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some((&mut code as *mut u32).cast()),
            &mut taille,
            core::ptr::null_mut(),
        )
        .map_err(|e| format!("statut : {e}"))?;
    }
    Ok(code)
}

/// Le corps de la réponse, lu par morceaux jusqu'au bout — ou jusqu'à la limite.
fn lire_le_corps(requete: &Poignee, limite: usize) -> Result<Vec<u8>, String> {
    let mut corps = Vec::new();
    loop {
        let mut disponible = 0u32;
        unsafe { WinHttpQueryDataAvailable(requete.0, &mut disponible) }
            .map_err(|e| raison("lecture", &e))?;
        if disponible == 0 {
            return Ok(corps);
        }
        let debut = corps.len();
        corps.resize(debut + disponible as usize, 0);
        let mut lus = 0u32;
        unsafe {
            WinHttpReadData(
                requete.0,
                corps[debut..].as_mut_ptr().cast(),
                disponible,
                &mut lus,
            )
        }
        .map_err(|e| raison("lecture", &e))?;
        corps.truncate(debut + lus as usize);
        if corps.len() > limite {
            return Err(format!("plus de {} Mo", limite / (1024 * 1024)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::raison;
    use windows::core::{Error, HRESULT};

    /// L'erreur que WinHTTP rend pour ce code Win32, telle qu'elle remonte d'un appel.
    fn erreur(win32: u32) -> Error {
        Error::from_hresult(HRESULT((0x8007_0000 | win32) as i32))
    }

    /// **Les causes qu'un dépôt rencontre se disent en mots** (DEPOT-WEB-6), et les autres
    /// gardent la phrase de Windows : le 07/10, « délai dépassé » est ce qu'il fallait lire.
    #[test]
    fn test_les_causes_d_un_depot_rate_se_disent_en_mots() {
        assert_eq!(raison("reponse", &erreur(12002)), "délai dépassé (reponse)");
        assert_eq!(
            raison("envoi", &erreur(12029)),
            "connexion impossible (envoi)"
        );
        assert!(raison("envoi", &erreur(12007)).starts_with("nom introuvable"));
        assert!(raison("lecture", &erreur(5)).starts_with("lecture : "));
    }
}
