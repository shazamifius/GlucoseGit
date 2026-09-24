//! Ce qu'un `src` ou un `href` de Glucose Tauri désigne — sans toucher au disque.
//!
//! Tauri ne gardait presque jamais les images dans le document. Depuis l'été 2026, il les
//! rangeait dans un **magasin global** par empreinte — `<données de l'application>/assets/`,
//! chaque fichier nommé `<16 premiers chiffres hexadécimaux du SHA-256>.<extension>` — et le
//! document ne portait que `asset:<nom>`. Un document **portable** emportait les siens dans
//! un dossier `objects/` voisin. Les plus anciens portaient les images en `data:`, en base64.
//!
//! Ce module dit **où** chercher ; le bureau va chercher, et vérifie ce qu'il trouve.

use super::projet::Provenance;

/// Ce qu'un `src` ou un `href` de Tauri désigne.
pub fn provenance_de(s: &str) -> Provenance {
    if let Some(nom) = s.strip_prefix("asset:").filter(|n| !n.starts_with("//")) {
        return if nom_sur(nom) {
            Provenance::Magasin(nom.to_string())
        } else {
            Provenance::Inconnue
        };
    }
    if let Some(reste) = s.strip_prefix("data:") {
        return match reste.split_once(";base64,") {
            Some((_, charge)) => base64(charge).map_or(Provenance::Inconnue, Provenance::Octets),
            None => Provenance::Inconnue,
        };
    }
    // L'adresse que Tauri fabriquait pour lire un fichier local (`convertFileSrc`) : c'est
    // un chemin, encodé dans une URL.
    for prefixe in [
        "http://asset.localhost/",
        "https://asset.localhost/",
        "asset://localhost/",
    ] {
        if let Some(chemin) = s.strip_prefix(prefixe) {
            return Provenance::Chemin(pourcent(chemin));
        }
    }
    if s.starts_with("http://") || s.starts_with("https://") {
        return Provenance::Web(s.to_string());
    }
    if let Some(chemin) = s
        .strip_prefix("file:///")
        .or_else(|| s.strip_prefix("file://"))
    {
        return Provenance::Chemin(pourcent(chemin));
    }
    if s.is_empty() {
        Provenance::Inconnue
    } else {
        Provenance::Chemin(s.to_string())
    }
}

/// Un nom du magasin ne désigne qu'un fichier **du magasin** : ni séparateur, ni `..`. Un
/// document forgé qui porterait `asset:../../secret` ne ferait lire rien d'autre.
fn nom_sur(nom: &str) -> bool {
    !nom.is_empty() && !nom.starts_with('.') && !nom.contains(['/', '\\', ':'])
}

/// Décode le base64 standard (RFC 4648), blancs ignorés. `None` pour un texte qui n'en est pas.
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

/// Décode les `%XX` d'une URL. Une séquence invalide reste telle quelle.
fn pourcent(s: &str) -> String {
    let o = s.as_bytes();
    let mut out = Vec::with_capacity(o.len());
    let mut i = 0;
    while i < o.len() {
        if o[i] == b'%' {
            if let Some(v) = o
                .get(i + 1..i + 3)
                .and_then(|h| std::str::from_utf8(h).ok())
                .and_then(|h| u8::from_str_radix(h, 16).ok())
            {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(o[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
