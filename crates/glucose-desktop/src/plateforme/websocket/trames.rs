//! **Les trames d'un WebSocket** (RFC 6455 § 5) et sa poignée de main (§ 4) — pur : des octets
//! entrent, des messages sortent. La voie sans Windows ([`super::flux`]) s'en sert ; WinHTTP fait
//! les siennes.
//!
//! Ce que Glucose en emploie : des messages texte entiers, le ping et le pong du protocole, la
//! fermeture. Un message fragmenté se recompose ; un message binaire se lit et s'ignore.

use glucose_core::encodage::base64_ecrire;
use glucose_core::hash::sha1;

/// Ce que la RFC 6455 § 1.3 ajoute à la clé du client avant d'en prendre l'empreinte.
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
/// **Le plus grand message accepté** : celui que Cloudflare accepte d'un Glucose (32 Mio depuis
/// octobre 2025) — au-delà, le relais fermerait la connexion ; en deçà, une ligne sans fin ne
/// remplit pas la mémoire.
pub const MESSAGE_MAX: usize = 32 << 20;
/// La plus longue réponse à la poignée de main qu'on lise avant de renoncer.
const ENTETES_MAX: usize = 16 << 10;

const CONTINUATION: u8 = 0x0;
const TEXTE: u8 = 0x1;
const BINAIRE: u8 = 0x2;
const FERMETURE: u8 = 0x8;
const PING: u8 = 0x9;
const PONG: u8 = 0xA;

/// Un message, une fois ses trames réunies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trame {
    Texte(String),
    Binaire(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    /// La fermeture, et son code s'il en porte un.
    Fermeture(Option<u16>),
}

/// **Écrit une trame entière**, masquée si un masque est donné : un client masque toujours, un
/// serveur jamais (§ 5.1).
pub fn ecrire(trame: &Trame, masque: Option<[u8; 4]>) -> Vec<u8> {
    let (code, charge): (u8, &[u8]) = match trame {
        Trame::Texte(t) => (TEXTE, t.as_bytes()),
        Trame::Binaire(o) => (BINAIRE, o),
        Trame::Ping(o) => (PING, o),
        Trame::Pong(o) => (PONG, o),
        Trame::Fermeture(_) => (FERMETURE, &[]),
    };
    let ferme;
    let charge = match trame {
        Trame::Fermeture(Some(c)) => {
            ferme = c.to_be_bytes();
            &ferme[..]
        }
        _ => charge,
    };
    let mut out = Vec::with_capacity(charge.len() + 14);
    out.push(0x80 | code);
    let bit_du_masque = if masque.is_some() { 0x80 } else { 0 };
    match charge.len() {
        n @ 0..=125 => out.push(bit_du_masque | n as u8),
        n @ 126..=0xffff => {
            out.push(bit_du_masque | 126);
            out.extend_from_slice(&(n as u16).to_be_bytes());
        }
        n => {
            out.push(bit_du_masque | 127);
            out.extend_from_slice(&(n as u64).to_be_bytes());
        }
    }
    match masque {
        Some(m) => {
            out.extend_from_slice(&m);
            out.extend(charge.iter().enumerate().map(|(i, o)| o ^ m[i % 4]));
        }
        None => out.extend_from_slice(charge),
    }
    out
}

/// **Recompose les messages** d'un flux d'octets qui arrive par morceaux.
pub struct Decodeur {
    tampon: Vec<u8>,
    /// Un message fragmenté en cours : son code, et ce qui en est déjà arrivé.
    fragments: Option<(u8, Vec<u8>)>,
    /// Ce que l'autre bout doit faire : un client masque ses trames, un serveur jamais.
    masquees: bool,
}

impl Decodeur {
    /// Le décodeur d'un client : il lit les trames d'un serveur, jamais masquées.
    pub fn d_un_serveur() -> Self {
        Self {
            tampon: Vec::new(),
            fragments: None,
            masquees: false,
        }
    }

    /// Le décodeur d'un serveur : il lit les trames d'un client, toujours masquées.
    #[cfg(test)]
    pub fn d_un_client() -> Self {
        Self {
            masquees: true,
            ..Self::d_un_serveur()
        }
    }

    pub fn pousser(&mut self, octets: &[u8]) {
        self.tampon.extend_from_slice(octets);
    }

    /// **Le prochain message entier**, s'il est arrivé ; une erreur, et la connexion est à
    /// fermer : l'autre bout ne parle pas le protocole.
    pub fn suivante(&mut self) -> Result<Option<Trame>, String> {
        loop {
            let Some((fin, code, charge)) = self.trame()? else {
                return Ok(None);
            };
            if code >= FERMETURE {
                if !fin || charge.len() > 125 {
                    return Err("trame de contrôle fragmentée ou trop longue".into());
                }
                return Ok(Some(match code {
                    PING => Trame::Ping(charge),
                    PONG => Trame::Pong(charge),
                    FERMETURE => {
                        Trame::Fermeture(charge.get(..2).map(|c| u16::from_be_bytes([c[0], c[1]])))
                    }
                    _ => return Err(format!("code de trame inconnu : {code}")),
                }));
            }
            let (code, mut tout) = match (code, self.fragments.take()) {
                (CONTINUATION, Some((c, mut deja))) => {
                    deja.extend_from_slice(&charge);
                    (c, deja)
                }
                (TEXTE | BINAIRE, None) => (code, charge),
                _ => return Err("suite de fragments incohérente".into()),
            };
            if tout.len() > MESSAGE_MAX {
                return Err("message trop long".into());
            }
            if !fin {
                self.fragments = Some((code, tout));
                continue;
            }
            return Ok(Some(if code == TEXTE {
                Trame::Texte(
                    String::from_utf8(std::mem::take(&mut tout))
                        .map_err(|_| "un message texte qui n'est pas de l'UTF-8".to_string())?,
                )
            } else {
                Trame::Binaire(tout)
            }));
        }
    }

    /// Une trame du tampon, si elle y est entière : sa fin, son code, sa charge démasquée.
    fn trame(&mut self) -> Result<Option<(bool, u8, Vec<u8>)>, String> {
        let t = &self.tampon;
        let (Some(&b0), Some(&b1)) = (t.first(), t.get(1)) else {
            return Ok(None);
        };
        if b0 & 0x70 != 0 {
            return Err("bits réservés sans extension négociée".into());
        }
        if (b1 & 0x80 != 0) != self.masquees {
            return Err("masque contraire au protocole".into());
        }
        let (longueur, mut i) = match b1 & 0x7f {
            126 => match t.get(2..4) {
                Some(o) => (u64::from(u16::from_be_bytes([o[0], o[1]])), 4),
                None => return Ok(None),
            },
            127 => match t.get(2..10).and_then(|o| <[u8; 8]>::try_from(o).ok()) {
                Some(o) => (u64::from_be_bytes(o), 10),
                None => return Ok(None),
            },
            n => (u64::from(n), 2),
        };
        if longueur > MESSAGE_MAX as u64 {
            return Err("trame trop longue".into());
        }
        let masque = if self.masquees {
            let Some(m) = t.get(i..i + 4).and_then(|m| <[u8; 4]>::try_from(m).ok()) else {
                return Ok(None);
            };
            i += 4;
            Some(m)
        } else {
            None
        };
        let fin_de_trame = i + longueur as usize;
        let Some(charge) = t.get(i..fin_de_trame) else {
            return Ok(None);
        };
        let charge = match masque {
            Some(m) => charge
                .iter()
                .enumerate()
                .map(|(k, o)| o ^ m[k % 4])
                .collect(),
            None => charge.to_vec(),
        };
        let trame = (b0 & 0x80 != 0, b0 & 0x0f, charge);
        self.tampon.drain(..fin_de_trame);
        Ok(Some(trame))
    }

    /// Ce qui reste dans le tampon, pas encore lu.
    #[cfg(test)]
    pub fn reste(&self) -> usize {
        self.tampon.len()
    }
}

/// **L'acceptation** qu'un serveur doit rendre pour cette clé (§ 4.2.2) : l'empreinte SHA-1 de
/// la clé suivie du GUID, en base64.
pub fn acceptation(cle: &str) -> String {
    base64_ecrire(&sha1(format!("{cle}{GUID}").as_bytes()))
}

/// **La demande d'ouverture** d'un client (§ 4.1), ses en-têtes en plus.
pub fn demande(hote: &str, chemin: &str, cle: &str, entetes: &[(&str, &str)]) -> String {
    let mut d = format!(
        "GET {chemin} HTTP/1.1\r\nHost: {hote}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\
         Sec-WebSocket-Key: {cle}\r\nSec-WebSocket-Version: 13\r\n"
    );
    for (nom, valeur) in entetes {
        d.push_str(&format!("{nom}: {valeur}\r\n"));
    }
    d.push_str("\r\n");
    d
}

/// **Lit la réponse du serveur** à la demande d'ouverture : `Ok(Some(n))` quand elle est entière
/// et dit oui — `n` octets lui appartenaient, le reste est déjà des trames —, `Ok(None)` s'il en
/// manque, une erreur sinon (§ 4.1, « the client MUST _Fail the WebSocket Connection_ »).
pub fn lire_la_reponse(octets: &[u8], cle: &str) -> Result<Option<usize>, String> {
    let Some(fin) = octets.windows(4).position(|w| w == b"\r\n\r\n") else {
        return if octets.len() > ENTETES_MAX {
            Err("réponse sans fin d'en-têtes".into())
        } else {
            Ok(None)
        };
    };
    let texte = String::from_utf8_lossy(&octets[..fin]);
    let mut lignes = texte.split("\r\n");
    let statut = lignes.next().unwrap_or_default();
    if statut.split(' ').nth(1) != Some("101") {
        return Err(format!("le relais refuse l'ouverture : {statut}"));
    }
    let entete = |nom: &str| {
        texte.split("\r\n").skip(1).find_map(|l| {
            let (n, v) = l.split_once(':')?;
            n.trim()
                .eq_ignore_ascii_case(nom)
                .then(|| v.trim().to_string())
        })
    };
    let upgrade = entete("upgrade").unwrap_or_default();
    let connection = entete("connection").unwrap_or_default();
    if !upgrade.eq_ignore_ascii_case("websocket")
        || !connection
            .split(',')
            .any(|c| c.trim().eq_ignore_ascii_case("upgrade"))
    {
        return Err("la réponse ne passe pas en WebSocket".into());
    }
    if entete("sec-websocket-accept").as_deref() != Some(acceptation(cle).as_str()) {
        return Err("l'acceptation ne répond pas à notre clé".into());
    }
    Ok(Some(fin + 4))
}

/// Une clé d'ouverture neuve : seize octets du hasard, en base64 (§ 4.1).
pub fn cle_neuve() -> Result<String, String> {
    crate::plateforme::hasard::octets::<16>().map(|o| base64_ecrire(&o))
}
