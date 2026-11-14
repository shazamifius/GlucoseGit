//! **Le WebSocket sous Windows** : WinHTTP, qui le sait depuis Windows 8 — ses trames, son
//! proxy, ses certificats (decisions/02). Rien de plus à compiler : la caisse `windows` nomme un
//! recoin de plus.
//!
//! # Deux fils, et qui ferme quoi
//!
//! WinHTTP admet une réception et un envoi en cours en même temps (le duplex du protocole). Un fil
//! à lui reçoit donc sans fin, et passe chaque message par un canal ; `envoyer` écrit depuis le
//! fil de la porte. La liaison (le WebSocket, sa connexion, sa session) est **partagée** : elle se
//! ferme quand le dernier des deux la lâche — jamais pendant que l'autre s'en sert.
//!
//! Quand la porte lâche la sienne, elle dit au revoir (`WinHttpWebSocketShutdown`) ; le relais
//! répond par sa fermeture, et le fil de réception s'arrête. Si le relais ne répond plus — c'est
//! souvent pour cela qu'on ferme —, la réception attend au plus [`ATTENTE`] : le ping de la porte
//! passe bien plus souvent, une connexion vivante ne l'atteint jamais.

use super::Canal;
use crate::plateforme::sources::Adresse;
use crate::plateforme::telechargement_windows::{raison, statut, Poignee};
use crate::plateforme::{DELAI, NAVIGATEUR};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Networking::WinHttp::{
    WinHttpAddRequestHeaders, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest,
    WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetOption, WinHttpSetTimeouts,
    WinHttpWebSocketCompleteUpgrade, WinHttpWebSocketReceive, WinHttpWebSocketSend,
    WinHttpWebSocketShutdown, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_ADDREQ_FLAG_ADD,
    WINHTTP_FLAG_SECURE, WINHTTP_OPEN_REQUEST_FLAGS, WINHTTP_OPTION_UPGRADE_TO_WEB_SOCKET,
    WINHTTP_WEB_SOCKET_BUFFER_TYPE, WINHTTP_WEB_SOCKET_CLOSE_BUFFER_TYPE,
    WINHTTP_WEB_SOCKET_UTF8_FRAGMENT_BUFFER_TYPE, WINHTTP_WEB_SOCKET_UTF8_MESSAGE_BUFFER_TYPE,
};

/// Le plus long silence qu'une réception attend : bien plus que l'intervalle du ping de la porte.
const ATTENTE: Duration = Duration::from_secs(60);

/// Le WebSocket et ce qu'il ne doit pas survivre — fermés dans l'ordre où ils sont écrits.
struct Liaison {
    ws: Poignee,
    _connexion: Poignee,
    _session: Poignee,
}

// WinHTTP admet ses poignées d'un fil à l'autre : c'est l'usage qu'il prévoit pour le duplex.
unsafe impl Send for Liaison {}
unsafe impl Sync for Liaison {}

struct Prise {
    liaison: Arc<Liaison>,
    recus: mpsc::Receiver<Result<String, String>>,
}

/// **Ouvre** : la session, la demande de passage en WebSocket, la réponse `101`, puis le fil de
/// réception.
pub(super) fn ouvrir(
    adresse: &Adresse,
    entetes: &[(&str, &str)],
) -> Result<Box<dyn Canal>, String> {
    let ms = |d: Duration| d.as_millis() as i32;
    unsafe {
        let session = Poignee(WinHttpOpen(
            &HSTRING::from(NAVIGATEUR),
            WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            0,
        ))
        .ou("session")?;
        WinHttpSetTimeouts(session.0, ms(DELAI), ms(DELAI), ms(DELAI), ms(ATTENTE))
            .map_err(|e| raison("delais", &e))?;
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
            &HSTRING::from("GET"),
            &HSTRING::from(adresse.chemin.as_str()),
            PCWSTR::null(),
            PCWSTR::null(),
            core::ptr::null(),
            drapeaux,
        ))
        .ou("requete")?;
        WinHttpSetOption(Some(requete.0), WINHTTP_OPTION_UPGRADE_TO_WEB_SOCKET, None)
            .map_err(|e| raison("websocket", &e))?;
        for (nom, valeur) in entetes {
            let ligne: Vec<u16> = format!("{nom}: {valeur}\r\n").encode_utf16().collect();
            WinHttpAddRequestHeaders(requete.0, &ligne, WINHTTP_ADDREQ_FLAG_ADD)
                .map_err(|e| raison("en-tetes", &e))?;
        }
        WinHttpSendRequest(requete.0, None, None, 0, 0, 0).map_err(|e| raison("envoi", &e))?;
        WinHttpReceiveResponse(requete.0, core::ptr::null_mut())
            .map_err(|e| raison("reponse", &e))?;
        let code = statut(&requete)?;
        if code != 101 {
            return Err(format!("le relais refuse l'ouverture : {code}"));
        }
        let ws = Poignee(WinHttpWebSocketCompleteUpgrade(requete.0, None)).ou("passage")?;
        let liaison = Arc::new(Liaison {
            ws,
            _connexion: connexion,
            _session: session,
        });
        let (envoi, recus) = mpsc::channel();
        let lecteur = Arc::clone(&liaison);
        std::thread::Builder::new()
            .name("glucose-websocket".into())
            .spawn(move || recevoir_sans_fin(&lecteur, &envoi))
            .map_err(|e| format!("fil de réception : {e}"))?;
        Ok(Box::new(Prise { liaison, recus }))
    }
}

/// Le fil de réception : chaque message entier part dans le canal ; la première erreur aussi,
/// et le fil s'arrête.
fn recevoir_sans_fin(liaison: &Liaison, envoi: &mpsc::Sender<Result<String, String>>) {
    let mut message = Vec::new();
    let mut tampon = vec![0u8; 64 << 10];
    loop {
        let mut lus = 0u32;
        let mut sorte = WINHTTP_WEB_SOCKET_BUFFER_TYPE(0);
        let code = unsafe {
            WinHttpWebSocketReceive(
                liaison.ws.0,
                tampon.as_mut_ptr().cast(),
                tampon.len() as u32,
                &mut lus,
                &mut sorte,
            )
        };
        let issue = if code != 0 {
            Some(Err(format!("réception : erreur {code}")))
        } else if sorte == WINHTTP_WEB_SOCKET_CLOSE_BUFFER_TYPE {
            Some(Err("le relais a fermé la connexion".to_string()))
        } else {
            let est_texte = sorte == WINHTTP_WEB_SOCKET_UTF8_MESSAGE_BUFFER_TYPE
                || sorte == WINHTTP_WEB_SOCKET_UTF8_FRAGMENT_BUFFER_TYPE;
            if est_texte {
                message.extend_from_slice(&tampon[..lus as usize]);
            }
            if message.len() > super::trames::MESSAGE_MAX {
                Some(Err("message trop long".to_string()))
            } else if sorte == WINHTTP_WEB_SOCKET_UTF8_MESSAGE_BUFFER_TYPE {
                let m = std::mem::take(&mut message);
                Some(String::from_utf8(m).map_err(|_| "message non UTF-8".to_string()))
            } else {
                None
            }
        };
        match issue {
            Some(Ok(texte)) => {
                if envoi.send(Ok(texte)).is_err() {
                    return;
                }
            }
            Some(Err(e)) => {
                let _ = envoi.send(Err(e));
                return;
            }
            None => {}
        }
    }
}

impl Canal for Prise {
    fn envoyer(&mut self, texte: &str) -> Result<(), String> {
        let code = unsafe {
            WinHttpWebSocketSend(
                self.liaison.ws.0,
                WINHTTP_WEB_SOCKET_UTF8_MESSAGE_BUFFER_TYPE,
                Some(texte.as_bytes()),
            )
        };
        if code == 0 {
            Ok(())
        } else {
            Err(format!("envoi : erreur {code}"))
        }
    }

    fn recevoir(&mut self, delai: Duration) -> Result<Option<String>, String> {
        match self.recus.recv_timeout(delai) {
            Ok(Ok(texte)) => Ok(Some(texte)),
            Ok(Err(e)) => Err(e),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err("la réception s'est arrêtée".into()),
        }
    }
}

impl Drop for Prise {
    /// Au revoir (§ 5.5.1) : le relais répond, et le fil de réception s'arrête de lui-même.
    fn drop(&mut self) {
        unsafe { WinHttpWebSocketShutdown(self.liaison.ws.0, 1000, None, 0) };
    }
}
