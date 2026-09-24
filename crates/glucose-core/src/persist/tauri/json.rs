//! Le JSON des premiers documents de Glucose Tauri (« v1 », avant Automerge).
//!
//! Un lecteur sans récursion, strict sur la grammaire (RFC 8259) et tolérant sur un
//! seul point : les nombres. JavaScript n'en a qu'un type, donc `14` et `14.0` sont la même
//! valeur ; on garde un entier quand le texte en écrit un et qu'il tient dans 64 bits, un
//! flottant sinon — c'est [`Valeur::nombre`] qui les réunit à la lecture.

use crate::error::CoreError;
use crate::persist::tauri::{illisible, Valeur};
use std::collections::BTreeMap;

/// Lit un texte JSON entier.
///
/// Sans récursion : les conteneurs ouverts vivent dans une pile explicite. Un fichier forgé de
/// cent mille `[` ne fait donc déborder aucune pile d'appels, et aucune profondeur maximale
/// n'a besoin d'être choisie — la seule borne est la mémoire, que le texte lui-même occupe
/// déjà.
pub fn lire(texte: &str) -> Result<Valeur, CoreError> {
    let mut l = Lecteur {
        o: texte.as_bytes(),
        p: 0,
    };
    let mut pile: Vec<Ouvert> = Vec::new();
    loop {
        let mut v = match l.ouvrir()? {
            Ouverture::Conteneur(o) => {
                pile.push(o);
                continue;
            }
            Ouverture::Valeur(v) => v,
        };
        // Une valeur complète remonte dans son conteneur, et le ferme s'il est fini.
        loop {
            let Some(parent) = pile.last_mut() else {
                l.blancs();
                if l.p != l.o.len() {
                    return Err(l.faute("du texte après la fin du document"));
                }
                return Ok(v);
            };
            if l.ranger(parent, v)? {
                break;
            }
            v = match pile.pop() {
                Some(Ouvert::Liste(liste)) => Valeur::Liste(liste),
                Some(Ouvert::Carte(carte, _)) => Valeur::Carte(carte),
                None => unreachable!("le parent vient d'être lu sur la pile"),
            };
        }
    }
}

/// Un conteneur ouvert, qui attend ses valeurs.
enum Ouvert {
    Liste(Vec<Valeur>),
    /// Une carte, et la clé de la valeur qu'elle attend.
    Carte(BTreeMap<String, Valeur>, String),
}

enum Ouverture {
    /// Un conteneur non vide vient de s'ouvrir : sa première valeur suit.
    Conteneur(Ouvert),
    Valeur(Valeur),
}

struct Lecteur<'a> {
    o: &'a [u8],
    p: usize,
}

impl Lecteur<'_> {
    fn faute(&self, quoi: &str) -> CoreError {
        illisible(&format!("JSON invalide à l'octet {} : {quoi}", self.p))
    }

    /// Les quatre blancs de JSON : espace, tabulation, saut de ligne, retour chariot.
    fn blancs(&mut self) {
        while self.p < self.o.len() && matches!(self.o[self.p], 0x20 | 0x09 | 0x0a | 0x0d) {
            self.p += 1;
        }
    }

    fn voir(&mut self) -> Option<u8> {
        self.blancs();
        self.o.get(self.p).copied()
    }

    fn mot(&mut self, mot: &str, v: Valeur) -> Result<Valeur, CoreError> {
        if self.o[self.p..].starts_with(mot.as_bytes()) {
            self.p += mot.len();
            Ok(v)
        } else {
            Err(self.faute("valeur inconnue"))
        }
    }

    /// Lit une valeur simple, ou ouvre un conteneur. Un conteneur vide est déjà une valeur.
    fn ouvrir(&mut self) -> Result<Ouverture, CoreError> {
        let v = match self.voir() {
            Some(b'{') => {
                self.p += 1;
                if self.voir() == Some(b'}') {
                    self.p += 1;
                    Valeur::Carte(BTreeMap::new())
                } else {
                    return Ok(Ouverture::Conteneur(Ouvert::Carte(
                        BTreeMap::new(),
                        self.cle()?,
                    )));
                }
            }
            Some(b'[') => {
                self.p += 1;
                if self.voir() == Some(b']') {
                    self.p += 1;
                    Valeur::Liste(Vec::new())
                } else {
                    return Ok(Ouverture::Conteneur(Ouvert::Liste(Vec::new())));
                }
            }
            Some(b'"') => Valeur::Texte(self.chaine()?),
            Some(b't') => self.mot("true", Valeur::Booleen(true))?,
            Some(b'f') => self.mot("false", Valeur::Booleen(false))?,
            Some(b'n') => self.mot("null", Valeur::Nul)?,
            Some(c) if c == b'-' || c.is_ascii_digit() => self.nombre()?,
            _ => return Err(self.faute("valeur attendue")),
        };
        Ok(Ouverture::Valeur(v))
    }

    /// Une clé, puis ses deux-points.
    fn cle(&mut self) -> Result<String, CoreError> {
        if self.voir() != Some(b'"') {
            return Err(self.faute("clé attendue"));
        }
        let cle = self.chaine()?;
        if self.voir() != Some(b':') {
            return Err(self.faute("« : » attendu"));
        }
        self.p += 1;
        Ok(cle)
    }

    /// Range `v` dans son conteneur. Rend `true` si une valeur de plus est attendue, `false`
    /// si le conteneur vient de se fermer.
    fn ranger(&mut self, parent: &mut Ouvert, v: Valeur) -> Result<bool, CoreError> {
        let fermeture = match parent {
            Ouvert::Liste(liste) => {
                liste.push(v);
                b']'
            }
            Ouvert::Carte(carte, cle) => {
                carte.insert(std::mem::take(cle), v);
                b'}'
            }
        };
        match self.voir() {
            Some(b',') => {
                self.p += 1;
                if let Ouvert::Carte(_, cle) = parent {
                    *cle = self.cle()?;
                }
                Ok(true)
            }
            Some(c) if c == fermeture => {
                self.p += 1;
                Ok(false)
            }
            _ => Err(self.faute(&format!("« , » ou « {} » attendu", fermeture as char))),
        }
    }

    fn chaine(&mut self) -> Result<String, CoreError> {
        self.p += 1;
        let mut s: Vec<u8> = Vec::new();
        loop {
            let Some(&c) = self.o.get(self.p) else {
                return Err(self.faute("chaîne non fermée"));
            };
            self.p += 1;
            match c {
                b'"' => return String::from_utf8(s).map_err(|_| self.faute("chaîne non UTF-8")),
                b'\\' => self.echappement(&mut s)?,
                c if c < 0x20 => return Err(self.faute("caractère de contrôle dans une chaîne")),
                c => s.push(c),
            }
        }
    }

    fn echappement(&mut self, s: &mut Vec<u8>) -> Result<(), CoreError> {
        let Some(&e) = self.o.get(self.p) else {
            return Err(self.faute("échappement tronqué"));
        };
        self.p += 1;
        let simple = match e {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => return self.unicode(s),
            _ => return Err(self.faute("échappement inconnu")),
        };
        let mut tampon = [0u8; 4];
        s.extend_from_slice(simple.encode_utf8(&mut tampon).as_bytes());
        Ok(())
    }

    /// `\uXXXX`, et la paire de substitution qui code un caractère hors du plan de base.
    fn unicode(&mut self, s: &mut Vec<u8>) -> Result<(), CoreError> {
        let haut = self.hex4()?;
        let point = if (0xD800..0xDC00).contains(&haut) {
            if !self.o[self.p..].starts_with(b"\\u") {
                return Err(self.faute("demi-paire de substitution seule"));
            }
            self.p += 2;
            let bas = self.hex4()?;
            if !(0xDC00..0xE000).contains(&bas) {
                return Err(self.faute("paire de substitution invalide"));
            }
            0x10000 + ((haut - 0xD800) << 10) + (bas - 0xDC00)
        } else {
            haut
        };
        let c = char::from_u32(point).ok_or_else(|| self.faute("point de code invalide"))?;
        let mut tampon = [0u8; 4];
        s.extend_from_slice(c.encode_utf8(&mut tampon).as_bytes());
        Ok(())
    }

    fn hex4(&mut self) -> Result<u32, CoreError> {
        let chiffres = self
            .o
            .get(self.p..self.p + 4)
            .ok_or_else(|| self.faute("\\u tronqué"))?;
        let texte = std::str::from_utf8(chiffres).map_err(|_| self.faute("\\u invalide"))?;
        let v = u32::from_str_radix(texte, 16).map_err(|_| self.faute("\\u invalide"))?;
        self.p += 4;
        Ok(v)
    }

    fn nombre(&mut self) -> Result<Valeur, CoreError> {
        let debut = self.p;
        let mut entier = true;
        while let Some(&c) = self.o.get(self.p) {
            match c {
                b'0'..=b'9' | b'-' | b'+' => {}
                b'.' | b'e' | b'E' => entier = false,
                _ => break,
            }
            self.p += 1;
        }
        let texte =
            std::str::from_utf8(&self.o[debut..self.p]).map_err(|_| self.faute("nombre"))?;
        if entier {
            if let Ok(n) = texte.parse::<i64>() {
                return Ok(Valeur::Entier(n));
            }
        }
        texte
            .parse::<f64>()
            .map(Valeur::Flottant)
            .map_err(|_| self.faute("nombre invalide"))
    }
}
