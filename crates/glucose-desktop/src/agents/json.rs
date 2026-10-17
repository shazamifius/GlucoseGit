//! **JSON, lu et écrit ici** — la langue du protocole MCP (RFC 8259).
//!
//! Le protocole n'emploie que des objets, des listes, des chaînes, des nombres et des booléens,
//! une ligne par message. Une caisse pour cela (`serde_json`) apporterait trois dépendances et
//! un mécanisme de dérivation pour ce que deux cents lignes lisibles font : la charte tranche
//! (« on prend une dépendance quand on ne peut pas faire sans »).
//!
//! # JSON-1 — une entrée hostile ne fait pas tomber Glucose
//!
//! Ce qui arrive par la porte vient d'un autre programme. Le lecteur ne panique sur aucune
//! entrée : un texte mal formé rend une erreur, et l'imbrication est bornée
//! ([`PROFONDEUR_MAX`]) — sans borne, `[[[[…` sur quelques mégaoctets ferait déborder la pile
//! du fil qui lit.

use std::fmt::Write as _;

/// L'imbrication la plus profonde qu'on lise. Les messages du protocole en ont six ou sept ;
/// cent vingt-huit est celle que `serde_json` refuse par défaut, pour la même raison.
const PROFONDEUR_MAX: usize = 128;

/// Une valeur JSON. Un objet garde l'ordre de ses clés : ce qu'on écrit se relit comme on l'a
/// écrit.
#[derive(Debug, Clone, PartialEq)]
pub enum Json {
    Nul,
    Bool(bool),
    Nombre(f64),
    Texte(String),
    Liste(Vec<Json>),
    Objet(Vec<(String, Json)>),
}

impl Json {
    /// **Lit** un texte JSON entier — rien ne doit le suivre que des blancs.
    pub fn lire(texte: &str) -> Result<Json, String> {
        let mut l = Lecteur {
            o: texte.as_bytes(),
            i: 0,
        };
        let v = l.valeur(0)?;
        l.blancs();
        if l.i != l.o.len() {
            return Err(format!("texte en trop à l'octet {}", l.i));
        }
        Ok(v)
    }

    /// **Écrit** la valeur sur une seule ligne : un saut de ligne dans une chaîne s'écrit
    /// `\n`, ce que le cadrage du protocole exige (un message par ligne).
    pub fn ecrire(&self) -> String {
        let mut s = String::new();
        self.ecrire_dans(&mut s);
        s
    }

    fn ecrire_dans(&self, s: &mut String) {
        match self {
            Json::Nul => s.push_str("null"),
            Json::Bool(b) => s.push_str(if *b { "true" } else { "false" }),
            Json::Nombre(n) => ecrire_un_nombre(*n, s),
            Json::Texte(t) => ecrire_une_chaine(t, s),
            Json::Liste(l) => {
                s.push('[');
                for (k, v) in l.iter().enumerate() {
                    if k > 0 {
                        s.push(',');
                    }
                    v.ecrire_dans(s);
                }
                s.push(']');
            }
            Json::Objet(o) => {
                s.push('{');
                for (k, (cle, v)) in o.iter().enumerate() {
                    if k > 0 {
                        s.push(',');
                    }
                    ecrire_une_chaine(cle, s);
                    s.push(':');
                    v.ecrire_dans(s);
                }
                s.push('}');
            }
        }
    }

    /// Un objet, de paires.
    pub fn objet<'a>(paires: impl IntoIterator<Item = (&'a str, Json)>) -> Json {
        Json::Objet(
            paires
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    pub fn texte(t: impl Into<String>) -> Json {
        Json::Texte(t.into())
    }

    /// La valeur d'une clé d'objet — `None` hors d'un objet.
    pub fn get(&self, cle: &str) -> Option<&Json> {
        match self {
            Json::Objet(o) => o.iter().find(|(k, _)| k == cle).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Json::Texte(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Json::Nombre(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Json::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_liste(&self) -> Option<&[Json]> {
        match self {
            Json::Liste(l) => Some(l),
            _ => None,
        }
    }
}

impl From<&str> for Json {
    fn from(t: &str) -> Self {
        Json::Texte(t.to_string())
    }
}

impl From<f64> for Json {
    fn from(n: f64) -> Self {
        Json::Nombre(n)
    }
}

impl<T: Into<Json>> From<Option<T>> for Json {
    fn from(o: Option<T>) -> Self {
        o.map_or(Json::Nul, Into::into)
    }
}

impl From<String> for Json {
    fn from(t: String) -> Self {
        Json::Texte(t)
    }
}

/// Un nombre entier s'écrit sans « .0 » — un identifiant de requête revient tel qu'il est
/// venu. Un nombre qui n'en est pas un (`NaN`, l'infini) n'existe pas en JSON : `null`.
fn ecrire_un_nombre(n: f64, s: &mut String) {
    if !n.is_finite() {
        s.push_str("null");
    } else if n.fract() == 0.0 && n.abs() < 1e15 {
        let _ = write!(s, "{}", n as i64);
    } else {
        let _ = write!(s, "{n}");
    }
}

fn ecrire_une_chaine(t: &str, s: &mut String) {
    s.push('"');
    for c in t.chars() {
        match c {
            '"' => s.push_str("\\\""),
            '\\' => s.push_str("\\\\"),
            '\n' => s.push_str("\\n"),
            '\r' => s.push_str("\\r"),
            '\t' => s.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(s, "\\u{:04x}", c as u32);
            }
            c => s.push(c),
        }
    }
    s.push('"');
}

struct Lecteur<'a> {
    o: &'a [u8],
    i: usize,
}

impl Lecteur<'_> {
    fn blancs(&mut self) {
        while self.o.get(self.i).is_some_and(|b| b" \t\r\n".contains(b)) {
            self.i += 1;
        }
    }

    fn erreur<T>(&self, quoi: &str) -> Result<T, String> {
        Err(format!("{quoi} à l'octet {}", self.i))
    }

    fn valeur(&mut self, profondeur: usize) -> Result<Json, String> {
        if profondeur > PROFONDEUR_MAX {
            return self.erreur("imbrication trop profonde");
        }
        self.blancs();
        match self.o.get(self.i) {
            Some(b'{') => self.objet(profondeur),
            Some(b'[') => self.liste(profondeur),
            Some(b'"') => self.chaine().map(Json::Texte),
            Some(b't') => self.mot("true", Json::Bool(true)),
            Some(b'f') => self.mot("false", Json::Bool(false)),
            Some(b'n') => self.mot("null", Json::Nul),
            Some(b'-' | b'0'..=b'9') => self.nombre(),
            _ => self.erreur("valeur attendue"),
        }
    }

    fn mot(&mut self, mot: &str, v: Json) -> Result<Json, String> {
        if self.o[self.i..].starts_with(mot.as_bytes()) {
            self.i += mot.len();
            Ok(v)
        } else {
            self.erreur("mot inconnu")
        }
    }

    fn nombre(&mut self) -> Result<Json, String> {
        let debut = self.i;
        while self
            .o
            .get(self.i)
            .is_some_and(|b| b"+-.eE0123456789".contains(b))
        {
            self.i += 1;
        }
        // Les octets lus ne sont que des chiffres et des signes : de l'ASCII.
        let texte = std::str::from_utf8(&self.o[debut..self.i]).unwrap_or_default();
        texte
            .parse::<f64>()
            .map(Json::Nombre)
            .or_else(|_| self.erreur("nombre mal formé"))
    }

    fn liste(&mut self, profondeur: usize) -> Result<Json, String> {
        self.i += 1;
        let mut l = Vec::new();
        self.blancs();
        if self.o.get(self.i) == Some(&b']') {
            self.i += 1;
            return Ok(Json::Liste(l));
        }
        loop {
            l.push(self.valeur(profondeur + 1)?);
            self.blancs();
            match self.o.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Json::Liste(l));
                }
                _ => return self.erreur("« , » ou « ] » attendu"),
            }
        }
    }

    fn objet(&mut self, profondeur: usize) -> Result<Json, String> {
        self.i += 1;
        let mut o = Vec::new();
        self.blancs();
        if self.o.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Ok(Json::Objet(o));
        }
        loop {
            self.blancs();
            if self.o.get(self.i) != Some(&b'"') {
                return self.erreur("clé attendue");
            }
            let cle = self.chaine()?;
            self.blancs();
            if self.o.get(self.i) != Some(&b':') {
                return self.erreur("« : » attendu");
            }
            self.i += 1;
            o.push((cle, self.valeur(profondeur + 1)?));
            self.blancs();
            match self.o.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Json::Objet(o));
                }
                _ => return self.erreur("« , » ou « } » attendu"),
            }
        }
    }

    /// Une chaîne : les octets UTF-8 passent tels quels, les échappements se décodent — une
    /// paire de substitution UTF-16 (`😀`) en un seul caractère.
    fn chaine(&mut self) -> Result<String, String> {
        self.i += 1;
        let mut octets = Vec::new();
        loop {
            match self.o.get(self.i) {
                None => return self.erreur("chaîne non fermée"),
                Some(b'"') => {
                    self.i += 1;
                    return String::from_utf8(octets).or_else(|_| self.erreur("UTF-8 invalide"));
                }
                Some(b'\\') => {
                    self.i += 1;
                    let c = self.echappement()?;
                    let mut tampon = [0; 4];
                    octets.extend_from_slice(c.encode_utf8(&mut tampon).as_bytes());
                }
                Some(&b) if b < 0x20 => {
                    return self.erreur("caractère de contrôle dans une chaîne")
                }
                Some(&b) => {
                    octets.push(b);
                    self.i += 1;
                }
            }
        }
    }

    fn echappement(&mut self) -> Result<char, String> {
        let Some(&b) = self.o.get(self.i) else {
            return self.erreur("échappement coupé");
        };
        self.i += 1;
        Ok(match b {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => return self.unicode(),
            _ => return self.erreur("échappement inconnu"),
        })
    }

    fn quatre_chiffres(&mut self) -> Result<u32, String> {
        let chiffres = self
            .o
            .get(self.i..self.i + 4)
            .and_then(|c| std::str::from_utf8(c).ok());
        let n = chiffres.and_then(|c| u32::from_str_radix(c, 16).ok());
        let Some(n) = n else {
            return self.erreur("\\u sans ses quatre chiffres");
        };
        self.i += 4;
        Ok(n)
    }

    fn unicode(&mut self) -> Result<char, String> {
        let haut = self.quatre_chiffres()?;
        let point = if (0xD800..0xDC00).contains(&haut) {
            if !self.o[self.i..].starts_with(b"\\u") {
                return self.erreur("moitié de paire UTF-16 seule");
            }
            self.i += 2;
            let bas = self.quatre_chiffres()?;
            if !(0xDC00..0xE000).contains(&bas) {
                return self.erreur("paire UTF-16 invalide");
            }
            0x10000 + ((haut - 0xD800) << 10) + (bas - 0xDC00)
        } else {
            haut
        };
        char::from_u32(point).map_or_else(|| self.erreur("point de code invalide"), Ok)
    }
}

#[cfg(test)]
mod tests;
