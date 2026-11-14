//! **Le chiffre des gestes partagés** (fiche 116 § 8) : ChaCha20-Poly1305 (RFC 8439), sans
//! dépendance.
//!
//! # Pourquoi
//!
//! Le co-working que la fiche 113 propose — le **greffier aveugle** — confie les gestes d'un canevas
//! partagé à une salle sur le relais, qui les numérote et les garde **sans pouvoir les lire** : chaque
//! geste part chiffré par la clé que porte le code d'invitation, et que la salle ne voit jamais.
//!
//! # Pourquoi celui-ci, et écrit ici
//!
//! ChaCha20-Poly1305 est le chiffre authentifié de TLS 1.3 et de WireGuard ; il n'a besoin d'aucune
//! instruction spéciale du processeur pour être rapide et sans fuite par le temps (pas de table
//! indexée par le secret, comme AES en logiciel) — **la même voie sur toutes les machines**, le
//! téléphone et l'iPad compris. Il tient en deux cents lignes, éprouvées sur les vecteurs de la
//! norme ; le noyau reste sans dépendance.
//!
//! # Ce qu'il garantit, et ce qu'il demande
//!
//! Un message scellé ne se lit qu'avec la clé, et **ne se modifie pas sans que l'ouverture échoue**
//! (le sceau couvre le chiffré et les données associées — le numéro du geste, par exemple). Il
//! demande qu'un même nonce ne serve **jamais** deux fois sous la même clé : l'appelant le tire du
//! hasard du système (96 bits) ou d'un compteur qui ne revient pas en arrière.

/// La taille d'une clé, d'un nonce, d'un sceau.
pub const CLE: usize = 32;
pub const NONCE: usize = 12;
pub const SCEAU: usize = 16;

fn mot(o: &[u8]) -> u32 {
    u32::from_le_bytes([o[0], o[1], o[2], o[3]])
}

/// Un quart de tour de ChaCha (RFC 8439 § 2.1).
fn quart(s: &mut [u32; 16], [a, b, c, d]: [usize; 4]) {
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(16);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(12);
    s[a] = s[a].wrapping_add(s[b]);
    s[d] = (s[d] ^ s[a]).rotate_left(8);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] = (s[b] ^ s[c]).rotate_left(7);
}

/// **Un bloc de ChaCha20** (§ 2.3) : soixante-quatre octets de flux pour ce compteur.
fn bloc(cle: &[u8; CLE], compteur: u32, nonce: &[u8; NONCE]) -> [u8; 64] {
    let mut etat = [0u32; 16];
    etat[..4].copy_from_slice(&[0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
    for (i, o) in cle.as_chunks::<4>().0.iter().enumerate() {
        etat[4 + i] = u32::from_le_bytes(*o);
    }
    etat[12] = compteur;
    for (i, o) in nonce.as_chunks::<4>().0.iter().enumerate() {
        etat[13 + i] = u32::from_le_bytes(*o);
    }
    let mut w = etat;
    for _ in 0..10 {
        for q in [[0, 4, 8, 12], [1, 5, 9, 13], [2, 6, 10, 14], [3, 7, 11, 15]] {
            quart(&mut w, q);
        }
        for q in [[0, 5, 10, 15], [1, 6, 11, 12], [2, 7, 8, 13], [3, 4, 9, 14]] {
            quart(&mut w, q);
        }
    }
    let mut sortie = [0u8; 64];
    for (i, o) in sortie.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        *o = w[i].wrapping_add(etat[i]).to_le_bytes();
    }
    sortie
}

/// Chiffre ou déchiffre sur place (§ 2.4) : le même geste dans les deux sens.
fn flux(cle: &[u8; CLE], compteur: u32, nonce: &[u8; NONCE], donnees: &mut [u8]) {
    for (k, morceau) in donnees.chunks_mut(64).enumerate() {
        let b = bloc(cle, compteur.wrapping_add(k as u32), nonce);
        for (o, f) in morceau.iter_mut().zip(b) {
            *o ^= f;
        }
    }
}

/// **Poly1305** (§ 2.5) : le sceau d'un message, par une clé à usage unique — en cinq membres de
/// 26 bits, sans branche sur le secret (la forme « donna » de Floodyberry).
fn poly1305(cle: &[u8; 32], message: &[u8]) -> [u8; SCEAU] {
    let r = [
        mot(&cle[0..]) & 0x03ff_ffff,
        (mot(&cle[3..]) >> 2) & 0x03ff_ff03,
        (mot(&cle[6..]) >> 4) & 0x03ff_c0ff,
        (mot(&cle[9..]) >> 6) & 0x03f0_3fff,
        (mot(&cle[12..]) >> 8) & 0x000f_ffff,
    ];
    let s = [r[1] * 5, r[2] * 5, r[3] * 5, r[4] * 5];
    let mut h = [0u32; 5];
    for morceau in message.chunks(16) {
        let mut b = [0u8; 17];
        b[..morceau.len()].copy_from_slice(morceau);
        b[morceau.len()] = 1;
        h[0] += mot(&b[0..]) & 0x03ff_ffff;
        h[1] += (mot(&b[3..]) >> 2) & 0x03ff_ffff;
        h[2] += (mot(&b[6..]) >> 4) & 0x03ff_ffff;
        h[3] += (mot(&b[9..]) >> 6) & 0x03ff_ffff;
        h[4] += (mot(&b[12..]) >> 8) | (u32::from(b[16]) << 24);
        multiplier(&mut h, &r, &s);
    }
    finir(h, &cle[16..])
}

/// `h = h · r mod 2¹³⁰ − 5`, les retenues propagées.
fn multiplier(h: &mut [u32; 5], r: &[u32; 5], s: &[u32; 4]) {
    let m = |a: u32, b: u32| u64::from(a) * u64::from(b);
    let d = [
        m(h[0], r[0]) + m(h[1], s[3]) + m(h[2], s[2]) + m(h[3], s[1]) + m(h[4], s[0]),
        m(h[0], r[1]) + m(h[1], r[0]) + m(h[2], s[3]) + m(h[3], s[2]) + m(h[4], s[1]),
        m(h[0], r[2]) + m(h[1], r[1]) + m(h[2], r[0]) + m(h[3], s[3]) + m(h[4], s[2]),
        m(h[0], r[3]) + m(h[1], r[2]) + m(h[2], r[1]) + m(h[3], r[0]) + m(h[4], s[3]),
        m(h[0], r[4]) + m(h[1], r[3]) + m(h[2], r[2]) + m(h[3], r[1]) + m(h[4], r[0]),
    ];
    let mut retenue = 0u64;
    for (hi, di) in h.iter_mut().zip(d) {
        let v = di + retenue;
        *hi = (v & 0x03ff_ffff) as u32;
        retenue = v >> 26;
    }
    h[0] += (retenue as u32) * 5;
    h[1] += h[0] >> 26;
    h[0] &= 0x03ff_ffff;
}

/// La réduction complète, `h mod p` sans branche, puis `+ s mod 2¹²⁸`.
fn finir(mut h: [u32; 5], s: &[u8]) -> [u8; SCEAU] {
    let mut c;
    for i in 1..5 {
        c = h[i] >> 26;
        h[i] &= 0x03ff_ffff;
        if i < 4 {
            h[i + 1] += c;
        } else {
            h[0] += c * 5;
        }
    }
    c = h[0] >> 26;
    h[0] &= 0x03ff_ffff;
    h[1] += c;
    // g = h + 5 − 2¹³⁰ : si g est positif, h ≥ p, et c'est g qu'on garde.
    let mut g = [0u32; 5];
    c = 5;
    for i in 0..4 {
        let v = h[i] + c;
        g[i] = v & 0x03ff_ffff;
        c = v >> 26;
    }
    g[4] = h[4].wrapping_add(c).wrapping_sub(1 << 26);
    let garder_g = (g[4] >> 31).wrapping_sub(1);
    for i in 0..5 {
        h[i] = (h[i] & !garder_g) | (g[i] & garder_g);
    }
    let mots = [
        h[0] | (h[1] << 26),
        (h[1] >> 6) | (h[2] << 20),
        (h[2] >> 12) | (h[3] << 14),
        (h[3] >> 18) | (h[4] << 8),
    ];
    let mut sceau = [0u8; SCEAU];
    let mut f = 0u64;
    for (i, o) in sceau.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        f = u64::from(mots[i]) + u64::from(mot(&s[4 * i..])) + (f >> 32);
        *o = (f as u32).to_le_bytes();
    }
    sceau
}

/// Le sceau de l'AEAD (§ 2.8) : les données associées, le chiffré, chacun complété à seize
/// octets, puis leurs deux longueurs.
fn sceau(cle: &[u8; CLE], nonce: &[u8; NONCE], associees: &[u8], chiffre: &[u8]) -> [u8; SCEAU] {
    let mut cle_unique = [0u8; 32];
    cle_unique.copy_from_slice(&bloc(cle, 0, nonce)[..32]);
    let completer = |n: usize| (16 - n % 16) % 16;
    let mut m = Vec::with_capacity(associees.len() + chiffre.len() + 48);
    m.extend_from_slice(associees);
    m.resize(m.len() + completer(associees.len()), 0);
    m.extend_from_slice(chiffre);
    m.resize(m.len() + completer(chiffre.len()), 0);
    m.extend_from_slice(&(associees.len() as u64).to_le_bytes());
    m.extend_from_slice(&(chiffre.len() as u64).to_le_bytes());
    poly1305(&cle_unique, &m)
}

/// **Scelle un message** : le chiffré, suivi de son sceau. `associees` est lu par tous et
/// protégé comme le reste (le numéro du geste, l'identité de la salle).
pub fn sceller(cle: &[u8; CLE], nonce: &[u8; NONCE], associees: &[u8], clair: &[u8]) -> Vec<u8> {
    let mut sortie = clair.to_vec();
    flux(cle, 1, nonce, &mut sortie);
    let s = sceau(cle, nonce, associees, &sortie);
    sortie.extend_from_slice(&s);
    sortie
}

/// **Ouvre un message scellé**, ou rien : une clé fausse, un octet changé, des données associées
/// qui ne sont pas les mêmes — l'ouverture échoue, et rien du clair ne sort.
pub fn ouvrir(
    cle: &[u8; CLE],
    nonce: &[u8; NONCE],
    associees: &[u8],
    scelle: &[u8],
) -> Option<Vec<u8>> {
    let (chiffre, recu) = scelle.split_at_checked(scelle.len().checked_sub(SCEAU)?)?;
    let attendu = sceau(cle, nonce, associees, chiffre);
    // Comparé en entier, sans s'arrêter au premier écart : le temps ne dit rien du sceau.
    let ecart = attendu.iter().zip(recu).fold(0u8, |e, (a, b)| e | (a ^ b));
    if ecart != 0 {
        return None;
    }
    let mut clair = chiffre.to_vec();
    flux(cle, 1, nonce, &mut clair);
    Some(clair)
}

#[cfg(test)]
mod tests;
