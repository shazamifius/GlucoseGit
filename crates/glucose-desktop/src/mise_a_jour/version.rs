//! **Une version, et son ordre** : `MAJEURE.MINEURE.CORRECTIVE[-PRÉVERSION][+CONSTRUCTION]`, au
//! sens de semver 2.0.0 — celui que Tauri applique aussi, pour que la bascule compare comme lui.
//!
//! Une préversion précède sa version (`2.0.1-beta.1 < 2.0.1`) ; ses identifiants se comparent un
//! à un, les nombres par leur valeur et avant les mots ; la construction ne compte pas.

use std::cmp::Ordering;
use std::fmt;

/// Une version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    noyau: [u64; 3],
    preversion: Vec<Identifiant>,
}

/// Un identifiant de préversion : un nombre précède toujours un mot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Identifiant {
    Nombre(u64),
    Mot(String),
}

impl Version {
    /// Lit une version ; un `v` devant est admis, comme Tauri l'admet.
    pub fn lire(texte: &str) -> Option<Self> {
        let texte = texte.trim();
        let texte = texte.strip_prefix('v').unwrap_or(texte);
        let sans_construction = texte.split('+').next()?;
        let (noyau, preversion) = match sans_construction.split_once('-') {
            Some((n, p)) => (n, Some(p)),
            None => (sans_construction, None),
        };
        let mut nombres = noyau.split('.').map(|n| n.parse::<u64>().ok());
        let noyau = [nombres.next()??, nombres.next()??, nombres.next()??];
        if nombres.next().is_some() {
            return None;
        }
        let preversion = match preversion {
            None => Vec::new(),
            Some(p) => p
                .split('.')
                .map(|i| match i.parse::<u64>() {
                    _ if i.is_empty() => None,
                    Ok(n) => Some(Identifiant::Nombre(n)),
                    Err(_) => Some(Identifiant::Mot(i.to_string())),
                })
                .collect::<Option<Vec<_>>>()?,
        };
        Some(Self { noyau, preversion })
    }

    /// La version de ce programme.
    pub fn courante() -> Self {
        Self::lire(env!("CARGO_PKG_VERSION")).expect("la version du paquet est une version")
    }
}

impl Ord for Version {
    fn cmp(&self, autre: &Self) -> Ordering {
        self.noyau.cmp(&autre.noyau).then_with(|| {
            match (self.preversion.is_empty(), autre.preversion.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => self.preversion.cmp(&autre.preversion),
            }
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, autre: &Self) -> Option<Ordering> {
        Some(self.cmp(autre))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c] = self.noyau;
        write!(f, "{a}.{b}.{c}")?;
        for (k, i) in self.preversion.iter().enumerate() {
            f.write_str(if k == 0 { "-" } else { "." })?;
            match i {
                Identifiant::Nombre(n) => write!(f, "{n}")?,
                Identifiant::Mot(m) => f.write_str(m)?,
            }
        }
        Ok(())
    }
}
