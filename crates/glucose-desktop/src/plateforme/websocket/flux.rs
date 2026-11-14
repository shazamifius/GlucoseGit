//! **Le WebSocket hors de Windows** : une connexion TCP, chiffrée par `rustls` et les racines de
//! Mozilla pour `wss://` (comme `ureq`, decisions/05 et 16), en clair pour `ws://` (les épreuves,
//! un relais sur la même machine), et les trames de [`super::trames`].
//!
//! Un seul fil la tient : `recevoir` lit avec un délai, et c'est entre deux lectures qu'on
//! écrit. `rustls` ne se partage pas entre deux fils, et n'a pas à l'être.

use super::trames::{self, Decodeur, Trame};
use super::Canal;
use crate::plateforme::sources::Adresse;
use crate::plateforme::DELAI;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

/// Ce qu'on lit et écrit : la connexion nue, ou chiffrée.
trait LireEcrire: Read + Write + Send {}
impl<T: Read + Write + Send> LireEcrire for T {}

struct Flux {
    flux: Box<dyn LireEcrire>,
    /// Une copie de la connexion TCP, pour régler le délai de lecture sous `rustls`.
    tcp: TcpStream,
    decodeur: Decodeur,
}

/// **Ouvre** : la connexion, le chiffrement s'il le faut, puis la poignée de main.
pub(super) fn ouvrir(
    adresse: &Adresse,
    entetes: &[(&str, &str)],
) -> Result<Box<dyn Canal>, String> {
    let tcp = connecter(&adresse.hote, adresse.port)?;
    let horloge = tcp.try_clone().map_err(|e| format!("connexion : {e}"))?;
    let flux: Box<dyn LireEcrire> = if adresse.securise {
        let nom = rustls::pki_types::ServerName::try_from(adresse.hote.clone())
            .map_err(|e| format!("nom du serveur : {e}"))?;
        let client = rustls::ClientConnection::new(configuration()?, nom)
            .map_err(|e| format!("chiffrement : {e}"))?;
        Box::new(rustls::StreamOwned::new(client, tcp))
    } else {
        Box::new(tcp)
    };
    let mut f = Flux {
        flux,
        tcp: horloge,
        decodeur: Decodeur::d_un_serveur(),
    };
    f.poignee_de_main(adresse, entetes)?;
    Ok(Box::new(f))
}

/// La première adresse du nom qui réponde, dans le délai de chaque étape.
fn connecter(hote: &str, port: u16) -> Result<TcpStream, String> {
    let mut derniere = format!("{hote} : aucune adresse");
    for a in (hote, port)
        .to_socket_addrs()
        .map_err(|e| format!("{hote} : {e}"))?
    {
        match TcpStream::connect_timeout(&a, DELAI) {
            Ok(tcp) => {
                // Un message part dès qu'il est écrit : rien ne le retient pour en grouper d'autres.
                tcp.set_nodelay(true).ok();
                tcp.set_write_timeout(Some(DELAI)).ok();
                return Ok(tcp);
            }
            Err(e) => derniere = format!("{a} : {e}"),
        }
    }
    Err(derniere)
}

/// Celle d'`ureq` : le fournisseur `ring`, les racines de Mozilla, aucune authentification du
/// client. Faite une fois.
fn configuration() -> Result<Arc<rustls::ClientConfig>, String> {
    static CONFIG: OnceLock<Result<Arc<rustls::ClientConfig>, String>> = OnceLock::new();
    CONFIG
        .get_or_init(|| {
            let racines = rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
            };
            rustls::ClientConfig::builder_with_provider(Arc::new(
                rustls::crypto::ring::default_provider(),
            ))
            .with_safe_default_protocol_versions()
            .map(|b| Arc::new(b.with_root_certificates(racines).with_no_client_auth()))
            .map_err(|e| format!("chiffrement : {e}"))
        })
        .clone()
}

impl Flux {
    fn poignee_de_main(&mut self, a: &Adresse, entetes: &[(&str, &str)]) -> Result<(), String> {
        let cle = trames::cle_neuve()?;
        let hote = if a.port == if a.securise { 443 } else { 80 } {
            a.hote.clone()
        } else {
            format!("{}:{}", a.hote, a.port)
        };
        let demande = trames::demande(&hote, &a.chemin, &cle, entetes);
        self.ecrire(demande.as_bytes())?;
        let debut = Instant::now();
        let mut recu = Vec::new();
        loop {
            if let Some(n) = trames::lire_la_reponse(&recu, &cle)? {
                // Ce qui suit les en-têtes, s'il en est venu, est déjà des trames.
                self.decodeur.pousser(&recu[n..]);
                return Ok(());
            }
            let reste = DELAI.saturating_sub(debut.elapsed());
            match self.lire(reste.max(Duration::from_millis(1)))? {
                Some(o) => recu.extend_from_slice(&o),
                None if debut.elapsed() >= DELAI => {
                    return Err("le relais ne répond pas à l'ouverture".into())
                }
                None => {}
            }
        }
    }

    fn ecrire(&mut self, octets: &[u8]) -> Result<(), String> {
        self.flux
            .write_all(octets)
            .and_then(|()| self.flux.flush())
            .map_err(|e| format!("envoi : {e}"))
    }

    /// Ce qui arrive avant `delai`, `None` s'il n'arrive rien ; la fin du flux est une erreur.
    fn lire(&mut self, delai: Duration) -> Result<Option<Vec<u8>>, String> {
        self.tcp
            .set_read_timeout(Some(delai.max(Duration::from_millis(1))))
            .map_err(|e| format!("délai : {e}"))?;
        let mut tampon = vec![0u8; 64 << 10];
        match self.flux.read(&mut tampon) {
            Ok(0) => Err("le relais a fermé la connexion".into()),
            Ok(n) => {
                tampon.truncate(n);
                Ok(Some(tampon))
            }
            Err(e) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => Ok(None),
            Err(e) if e.kind() == ErrorKind::Interrupted => Ok(None),
            Err(e) => Err(format!("lecture : {e}")),
        }
    }
}

impl Canal for Flux {
    fn envoyer(&mut self, texte: &str) -> Result<(), String> {
        let masque = crate::plateforme::hasard::octets::<4>()?;
        self.ecrire(&trames::ecrire(
            &Trame::Texte(texte.to_string()),
            Some(masque),
        ))
    }

    fn recevoir(&mut self, delai: Duration) -> Result<Option<String>, String> {
        let debut = Instant::now();
        loop {
            while let Some(trame) = self.decodeur.suivante()? {
                match trame {
                    Trame::Texte(t) => return Ok(Some(t)),
                    Trame::Ping(charge) => {
                        let masque = crate::plateforme::hasard::octets::<4>()?;
                        self.ecrire(&trames::ecrire(&Trame::Pong(charge), Some(masque)))?;
                    }
                    Trame::Fermeture(code) => {
                        return Err(format!("le relais a fermé la connexion ({code:?})"))
                    }
                    Trame::Pong(_) | Trame::Binaire(_) => {}
                }
            }
            let reste = delai.saturating_sub(debut.elapsed());
            match self.lire(reste)? {
                Some(o) => self.decodeur.pousser(&o),
                None if debut.elapsed() >= delai => return Ok(None),
                None => {}
            }
        }
    }
}

impl Drop for Flux {
    /// Dit au revoir (§ 5.5.1) ; s'il est trop tard, la connexion se ferme quand même.
    fn drop(&mut self) {
        if let Ok(masque) = crate::plateforme::hasard::octets::<4>() {
            let _ = self.ecrire(&trames::ecrire(&Trame::Fermeture(Some(1000)), Some(masque)));
        }
    }
}
