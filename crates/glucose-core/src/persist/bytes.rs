//! Primitives d'octets du format `.glucose` v2 — 100% `std`, 0 dépendance.
//!
//! # Conventions, valables pour TOUT le format
//!
//! - Les scalaires de taille fixe (`u16`, `u32`, `u64`, `i32`, `i64`) sont **petit-boutistes**.
//! - Un `f64` est écrit par ses **bits** (`to_bits`) et non par son écriture décimale : c'est la
//!   seule façon de garantir qu'un aller-retour rende *exactement* le même flottant, y compris
//!   `-0.0`. Un `f64` occupe toujours 8 octets.
//! - Les **longueurs** et les **compteurs** sont des varints LEB128 non signés. Une chaîne courte
//!   coûte alors 1 octet d'en-tête au lieu de 4, et aucune longueur n'est jamais tronquée par un
//!   `as u32` silencieux.
//! - Un `Option` s'écrit `0x00` (absent) ou `0x01` suivi de la valeur. Tout autre octet est une
//!   corruption, pas un « absent » par défaut.
//!
//! # Pourquoi le lecteur ne panique jamais
//!
//! [`Reader`] ne fait **aucune** indexation directe : toute lecture passe par [`Reader::take`],
//! qui vérifie les bornes et rend une `CoreError::DeserializationError`. Un fichier tronqué
//! produit donc une erreur remontée à l'utilisateur, jamais un `panic` (standard § 6.2).

use crate::error::{CoreError, CoreResult};

/// Nombre maximal d'éléments pré-alloués d'après un compteur lu sur le disque.
///
/// INVARIANT PERSIST-2 — un compteur corrompu peut annoncer 2^64 éléments. Pré-allouer sans
/// borne ferait tomber le processus sur un échec d'allocation *avant* que le contrôle de bornes
/// n'ait pu rendre une erreur propre. On alloue donc par paliers et on laisse `take` détecter
/// la troncature.
const MAX_PREALLOC: usize = 1024;

fn overflow(what: &str) -> CoreError {
    CoreError::DeserializationError(format!(
        "{what} dépasse la capacité adressable de cette machine — le fichier est corrompu"
    ))
}

// ── Écriture ────────────────────────────────────────────────────────────────

/// Tampon d'écriture séquentiel. Infaillible : écrire dans un `Vec` ne peut pas échouer.
#[derive(Debug, Default)]
pub struct Writer {
    buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i32(&mut self, v: i32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i64(&mut self, v: i64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn f64(&mut self, v: f64) {
        self.buf.extend_from_slice(&v.to_bits().to_le_bytes());
    }

    pub fn flag(&mut self, v: bool) {
        self.buf.push(u8::from(v));
    }

    /// Varint LEB128 non signé : 1 octet jusqu'à 127, 10 au maximum.
    pub fn uvarint(&mut self, mut v: u64) {
        while v >= 0x80 {
            self.buf.push((v as u8) | 0x80);
            v >>= 7;
        }
        self.buf.push(v as u8);
    }

    pub fn count(&mut self, n: usize) {
        self.uvarint(n as u64);
    }

    pub fn blob(&mut self, v: &[u8]) {
        self.count(v.len());
        self.buf.extend_from_slice(v);
    }

    pub fn text(&mut self, v: &str) {
        self.blob(v.as_bytes());
    }

    pub fn raw(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
    }

    pub fn opt<T>(&mut self, v: Option<T>, encode: impl FnOnce(&mut Self, T)) {
        match v {
            Some(inner) => {
                self.u8(1);
                encode(self, inner);
            }
            None => self.u8(0),
        }
    }

    pub fn opt_text(&mut self, v: Option<&String>) {
        self.opt(v, |w, s| w.text(s));
    }

    pub fn opt_f64(&mut self, v: Option<f64>) {
        self.opt(v, |w, x| w.f64(x));
    }

    pub fn opt_u64(&mut self, v: Option<u64>) {
        self.opt(v, |w, x| w.u64(x));
    }

    pub fn seq<T>(&mut self, items: &[T], encode: impl Fn(&mut Self, &T)) {
        self.count(items.len());
        for item in items {
            encode(self, item);
        }
    }
}

// ── Lecture ─────────────────────────────────────────────────────────────────

/// Curseur de lecture à bornes vérifiées sur un tampon d'octets.
#[derive(Debug)]
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    /// Consomme `n` octets. C'est l'UNIQUE point d'accès aux données : tout dépassement est
    /// converti ici en erreur, ce qui rend le décodeur entier insensible à la troncature.
    pub fn take(&mut self, n: usize) -> CoreResult<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| overflow("une longueur"))?;
        if end > self.data.len() {
            return Err(CoreError::DeserializationError(format!(
                "fichier tronqué : {} octets attendus à l'offset {}, il n'en reste que {} — \
                 la sauvegarde a été interrompue, rouvre la copie précédente",
                n,
                self.pos,
                self.remaining()
            )));
        }
        let out = &self.data[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    fn fixed<const N: usize>(&mut self) -> CoreResult<[u8; N]> {
        let slice = self.take(N)?;
        let mut out = [0u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    pub fn u8(&mut self) -> CoreResult<u8> {
        Ok(self.fixed::<1>()?[0])
    }

    pub fn u16(&mut self) -> CoreResult<u16> {
        Ok(u16::from_le_bytes(self.fixed::<2>()?))
    }

    pub fn u64(&mut self) -> CoreResult<u64> {
        Ok(u64::from_le_bytes(self.fixed::<8>()?))
    }

    pub fn i32(&mut self) -> CoreResult<i32> {
        Ok(i32::from_le_bytes(self.fixed::<4>()?))
    }

    pub fn i64(&mut self) -> CoreResult<i64> {
        Ok(i64::from_le_bytes(self.fixed::<8>()?))
    }

    pub fn f64(&mut self) -> CoreResult<f64> {
        Ok(f64::from_bits(u64::from_le_bytes(self.fixed::<8>()?)))
    }

    pub fn digest(&mut self) -> CoreResult<[u8; 32]> {
        self.fixed::<32>()
    }

    pub fn flag(&mut self) -> CoreResult<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(CoreError::DeserializationError(format!(
                "booléen invalide ({other}) : le document est corrompu, rouvre la sauvegarde précédente"
            ))),
        }
    }

    pub fn uvarint(&mut self) -> CoreResult<u64> {
        let mut value: u64 = 0;
        for shift in (0..64u32).step_by(7) {
            let byte = self.u8()?;
            let payload = u64::from(byte & 0x7f);
            value |= payload
                .checked_shl(shift)
                .ok_or_else(|| overflow("un varint"))?;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(overflow("un varint"))
    }

    pub fn count(&mut self) -> CoreResult<usize> {
        usize::try_from(self.uvarint()?).map_err(|_| overflow("un compteur"))
    }

    pub fn blob(&mut self) -> CoreResult<Vec<u8>> {
        let len = self.count()?;
        Ok(self.take(len)?.to_vec())
    }

    pub fn text(&mut self) -> CoreResult<String> {
        let len = self.count()?;
        let start = self.pos;
        let raw = self.take(len)?;
        String::from_utf8(raw.to_vec()).map_err(|e| {
            CoreError::DeserializationError(format!(
                "chaîne UTF-8 invalide à l'offset {start} : {e} — le document est corrompu"
            ))
        })
    }

    pub fn opt<T>(
        &mut self,
        decode: impl FnOnce(&mut Self) -> CoreResult<T>,
    ) -> CoreResult<Option<T>> {
        match self.u8()? {
            0 => Ok(None),
            1 => decode(self).map(Some),
            other => Err(CoreError::DeserializationError(format!(
                "marqueur d'option invalide ({other}) : le document est corrompu"
            ))),
        }
    }

    pub fn opt_text(&mut self) -> CoreResult<Option<String>> {
        self.opt(|r| r.text())
    }

    pub fn opt_f64(&mut self) -> CoreResult<Option<f64>> {
        self.opt(|r| r.f64())
    }

    pub fn opt_u64(&mut self) -> CoreResult<Option<u64>> {
        self.opt(|r| r.u64())
    }

    pub fn seq<T>(&mut self, decode: impl Fn(&mut Self) -> CoreResult<T>) -> CoreResult<Vec<T>> {
        let len = self.count()?;
        let mut out = Vec::with_capacity(len.min(MAX_PREALLOC));
        for _ in 0..len {
            out.push(decode(self)?);
        }
        Ok(out)
    }

    /// Vérifie que la section a été consommée intégralement.
    ///
    /// Des octets en trop signifient que l'écrivain et le lecteur ne s'accordent pas sur le
    /// schéma : mieux vaut le dire que charger un document à moitié juste.
    pub fn finish(&self) -> CoreResult<()> {
        if self.remaining() != 0 {
            return Err(CoreError::DeserializationError(format!(
                "{} octets inattendus en fin de section — document écrit par un autre schéma",
                self.remaining()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalars_round_trip_bit_exactly() {
        let mut w = Writer::new();
        w.u8(0xAB);
        w.u16(0xBEEF);
        w.u64(u64::MAX);
        w.i32(i32::MIN);
        w.i64(-1_234_567_890_123);
        w.f64(-0.0);
        w.f64(f64::MAX);
        w.flag(true);
        w.flag(false);
        let bytes = w.into_bytes();

        let mut r = Reader::new(&bytes);
        assert_eq!(r.u8().expect("u8"), 0xAB);
        assert_eq!(r.u16().expect("u16"), 0xBEEF);
        assert_eq!(r.u64().expect("u64"), u64::MAX);
        assert_eq!(r.i32().expect("i32"), i32::MIN);
        assert_eq!(r.i64().expect("i64"), -1_234_567_890_123);
        // `-0.0 == 0.0` en Rust : seuls les bits distinguent les deux.
        assert_eq!(r.f64().expect("f64").to_bits(), (-0.0f64).to_bits());
        assert_eq!(r.f64().expect("f64"), f64::MAX);
        assert!(r.flag().expect("flag"));
        assert!(!r.flag().expect("flag"));
        assert!(r.finish().is_ok());
    }

    #[test]
    fn test_uvarint_boundaries() {
        for value in [0u64, 1, 127, 128, 16_383, 16_384, u64::MAX] {
            let mut w = Writer::new();
            w.uvarint(value);
            let bytes = w.into_bytes();
            let mut r = Reader::new(&bytes);
            assert_eq!(r.uvarint().expect("uvarint"), value, "valeur {value}");
            assert!(r.finish().is_ok());
        }
    }

    #[test]
    fn test_truncated_buffer_is_an_error_not_a_panic() {
        let mut w = Writer::new();
        w.text("bonjour");
        let mut bytes = w.into_bytes();
        bytes.truncate(3);

        let mut r = Reader::new(&bytes);
        let err = r.text().expect_err("une chaîne tronquée doit échouer");
        assert!(err.to_string().contains("tronqué"), "message: {err}");
    }

    #[test]
    fn test_invalid_utf8_is_reported() {
        let mut w = Writer::new();
        w.blob(&[0xff, 0xfe, 0xfd]);
        let bytes = w.into_bytes();
        let mut r = Reader::new(&bytes);
        let err = r
            .text()
            .expect_err("les octets 0xff ne sont pas de l'UTF-8");
        assert!(err.to_string().contains("UTF-8"), "message: {err}");
    }

    #[test]
    fn test_invalid_option_marker_is_reported() {
        let bytes = [7u8];
        let mut r = Reader::new(&bytes);
        let err = r.opt_text().expect_err("0x07 n'est ni 0 ni 1");
        assert!(err.to_string().contains("option"), "message: {err}");
    }

    #[test]
    fn test_invalid_boolean_is_reported() {
        let bytes = [2u8];
        let mut r = Reader::new(&bytes);
        let err = r.flag().expect_err("0x02 n'est ni faux ni vrai");
        assert!(err.to_string().contains("booléen"), "message: {err}");
    }

    #[test]
    fn test_absurd_sequence_count_fails_without_allocating_the_world() {
        // Un compteur de 2^60 éléments suivi d'aucune donnée : le décodeur doit rendre une
        // erreur de troncature immédiatement (INVARIANT PERSIST-2), pas tenter l'allocation.
        let mut w = Writer::new();
        w.uvarint(1 << 60);
        let bytes = w.into_bytes();
        let mut r = Reader::new(&bytes);
        let err = r
            .seq(|rr| rr.u8())
            .expect_err("un compteur absurde doit échouer proprement");
        assert!(err.to_string().contains("tronqué"), "message: {err}");
    }

    #[test]
    fn test_finish_rejects_trailing_bytes() {
        let bytes = [1u8, 2, 3];
        let mut r = Reader::new(&bytes);
        assert_eq!(r.u8().expect("u8"), 1);
        let err = r.finish().expect_err("2 octets restent");
        assert!(err.to_string().contains("inattendus"), "message: {err}");
    }

    #[test]
    fn test_seq_and_opt_round_trip() {
        let mut w = Writer::new();
        w.seq(&[1.5f64, -2.5, 0.0], |ww, v| ww.f64(*v));
        w.opt_text(Some(&"présent".to_string()));
        w.opt_text(None);
        w.opt_f64(Some(3.25));
        w.opt_u64(Some(42));
        let bytes = w.into_bytes();

        let mut r = Reader::new(&bytes);
        assert_eq!(r.seq(|rr| rr.f64()).expect("seq"), vec![1.5, -2.5, 0.0]);
        assert_eq!(r.opt_text().expect("opt"), Some("présent".to_string()));
        assert_eq!(r.opt_text().expect("opt"), None);
        assert_eq!(r.opt_f64().expect("opt"), Some(3.25));
        assert_eq!(r.opt_u64().expect("opt"), Some(42));
        assert!(r.finish().is_ok());
    }
}
