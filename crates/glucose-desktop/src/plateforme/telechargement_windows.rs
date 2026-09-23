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
//! La note [`decisions/02`](../../../../docs/architecture/decisions/02-TELECHARGER-CE-QU-ON-DEPOSE.md)
//! dit le reste. Ce module ne décide de rien : il reçoit une adresse déjà découpée et jugée
//! par [`super::sources`], et rend des octets ou la raison de n'en pas rendre.

use super::sources::Adresse;
use windows::core::{w, HSTRING, PCWSTR};
use windows::Win32::Networking::WinHttp::{
    WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryDataAvailable,
    WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest,
    WinHttpSetTimeouts, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE,
    WINHTTP_OPEN_REQUEST_FLAGS, WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE,
};

/// **Le nom sous lequel Glucose se présente.**
///
/// Celui d'un navigateur courant : plusieurs serveurs d'images servent une page d'erreur, ou
/// rien, à un client qu'ils ne reconnaissent pas. On ne cache rien de ce qu'on demande — on
/// demande ce que l'utilisateur a glissé depuis son propre navigateur, et rien d'autre.
const NAVIGATEUR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                          (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36 Glucose";

/// **Combien de temps on attend chaque étape**, en millisecondes : résoudre le nom, se
/// connecter, envoyer, recevoir.
///
/// Le téléchargement se fait hors de la boucle d'images, donc aucune image n'attend ; mais
/// un dépôt qui ne se pose jamais doit finir par retomber sur son repli. Quinze secondes, c'est
/// ce qu'un navigateur accorde avant d'afficher qu'une page ne répond pas.
const DELAI_MS: i32 = 15_000;

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
            .map_err(|e| format!("envoi : {e}"))?;
        WinHttpReceiveResponse(requete.requete.0, core::ptr::null_mut())
            .map_err(|e| format!("reponse : {e}"))?;
    }
    let statut = statut(&requete.requete)?;
    if statut != 200 {
        return Err(format!("le serveur repond {statut}"));
    }
    lire_le_corps(&requete.requete, limite)
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
            .map_err(|e| format!("lecture : {e}"))?;
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
        .map_err(|e| format!("lecture : {e}"))?;
        corps.truncate(debut + lus as usize);
        if corps.len() > limite {
            return Err(format!("plus de {} Mo", limite / (1024 * 1024)));
        }
    }
}
