//! **Base64** (RFC 4648), dans les deux sens et les deux alphabets — sans dépendance.
//!
//! Lire : les documents de Glucose Tauri portaient leurs plus vieilles images en `data:`, la
//! mise à jour signe en base64, le presse-papiers HTML aussi. Écrire : la porte publique (fiche
//! 116) — la clé d'ouverture d'un WebSocket (RFC 6455), les jetons et le défi PKCE d'OAuth
//! (RFC 7636), qui s'écrivent dans l'alphabet « sûr pour une adresse », sans `=`.
//!
//! **Un seul décodeur** pour les deux alphabets : `+` et `-` valent 62, `/` et `_` valent 63 ;
//! rien ne peut les confondre, et un texte de l'un se lit comme de l'autre.

const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const ADRESSE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Décode le base64, des deux alphabets, blancs ignorés, `=` final facultatif. `None` pour un
/// texte qui n'en est pas.
pub fn base64(texte: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(texte.len() * 3 / 4);
    let mut acc = 0u32;
    let mut bits = 0u32;
    for c in texte.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => break,
            c if c.is_ascii_whitespace() => continue,
            _ => return None,
        };
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

/// Écrit en base64 standard, complété par `=` — la forme des en-têtes de HTTP.
pub fn base64_ecrire(octets: &[u8]) -> String {
    ecrire(octets, STANDARD, true)
}

/// Écrit en base64 « sûr pour une adresse », sans `=` (RFC 4648 § 5) — la forme d'OAuth.
pub fn base64_adresse(octets: &[u8]) -> String {
    ecrire(octets, ADRESSE, false)
}

fn ecrire(octets: &[u8], alphabet: &[u8; 64], completer: bool) -> String {
    let mut out = String::with_capacity(octets.len().div_ceil(3) * 4);
    for groupe in octets.chunks(3) {
        let n = groupe
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, o)| acc | u32::from(*o) << (16 - 8 * i));
        // Trois octets donnent quatre signes ; un groupe incomplet en donne un de plus que
        // d'octets, puis le complément.
        for k in 0..=groupe.len() {
            out.push(alphabet[(n >> (18 - 6 * k) & 63) as usize] as char);
        }
        if completer {
            for _ in groupe.len()..3 {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les vecteurs de la RFC 4648 § 10, dans les deux sens.
    #[test]
    fn les_vecteurs_de_la_norme() {
        let vecteurs = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (clair, code) in vecteurs {
            assert_eq!(base64_ecrire(clair.as_bytes()), code);
            assert_eq!(base64(code).as_deref(), Some(clair.as_bytes()));
            assert_eq!(base64_adresse(clair.as_bytes()), code.trim_end_matches('='));
        }
    }

    /// Les deux signes qui changent d'un alphabet à l'autre : 62 et 63.
    #[test]
    fn l_alphabet_des_adresses_ne_porte_ni_plus_ni_barre() {
        let octets = [0xfb, 0xff, 0xbf];
        assert_eq!(base64_ecrire(&octets), "+/+/");
        assert_eq!(base64_adresse(&octets), "-_-_");
        assert_eq!(base64("-_-_").as_deref(), Some(&octets[..]));
    }

    /// Tout octet, toute longueur : écrire puis lire rend l'original.
    #[test]
    fn ecrire_puis_lire_rend_l_original() {
        for n in 0..70 {
            let octets: Vec<u8> = (0..n).map(|i| (i * 37 + 11) as u8).collect();
            assert_eq!(base64(&base64_ecrire(&octets)), Some(octets.clone()));
            assert_eq!(base64(&base64_adresse(&octets)), Some(octets));
        }
    }

    #[test]
    fn un_texte_qui_n_en_est_pas_ne_se_lit_pas() {
        assert_eq!(base64("Zm9v!"), None);
    }
}
