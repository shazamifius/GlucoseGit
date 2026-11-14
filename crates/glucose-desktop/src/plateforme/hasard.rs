//! **Le hasard du système** — celui dont on fait les clés (fiche 116) : le secret qui dit qu'un
//! Glucose est bien lui devant son relais, les jetons qu'il donne aux IA publiques, le masque
//! des trames d'un WebSocket (RFC 6455 § 10.3 : imprévisible).
//!
//! Aucune caisse : chaque système a le sien, celui que tous ses programmes emploient —
//! `/dev/urandom` sous Linux, macOS et Android, `BCryptGenRandom` sous Windows (le générateur
//! préféré du système, CNG). Un générateur écrit ici serait une faute : le hasard d'une clé ne
//! se fabrique pas, il se demande.

/// **`N` octets imprévisibles**, ou la raison de n'en pas avoir. Ne retombe jamais sur un
/// hasard faible : une clé qu'on ne peut pas tirer ne se tire pas.
pub fn octets<const N: usize>() -> Result<[u8; N], String> {
    let mut out = [0u8; N];
    remplir(&mut out)?;
    Ok(out)
}

#[cfg(not(windows))]
fn remplir(tampon: &mut [u8]) -> Result<(), String> {
    use std::io::Read;
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(tampon))
        .map_err(|e| format!("hasard du système : {e}"))
}

#[cfg(windows)]
fn remplir(tampon: &mut [u8]) -> Result<(), String> {
    use windows::Win32::Security::Cryptography::{
        BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG,
    };
    unsafe { BCryptGenRandom(None, tampon, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .map_err(|e| format!("hasard du système : {e}"))
}

#[cfg(test)]
mod tests {
    /// Deux tirages ne se ressemblent pas, et un tirage n'est pas fait de zéros : un hasard
    /// « rempli » de zéros (une lecture qui n'aurait rien lu) donnerait la même clé à tous.
    #[test]
    fn deux_tirages_different_et_ne_sont_pas_vides() {
        let a = super::octets::<32>().expect("le système a du hasard");
        let b = super::octets::<32>().expect("le système a du hasard");
        assert_ne!(a, b);
        assert_ne!(a, [0u8; 32]);
    }
}
