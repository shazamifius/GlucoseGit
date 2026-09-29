//! Ce que la boîte noire écrit, une ligne JSON par enregistrement.
//!
//! **Que des nombres, des booléens et des noms fixés à la compilation.** Aucun champ n'est une
//! chaîne construite pendant l'exécution : le type ne peut pas transporter un mot de
//! l'utilisateur, un nom de fichier ou un chemin — la promesse se vérifie en lisant le type, et
//! non en relisant chaque appel.

/// Un enregistrement de la boîte noire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enregistrement {
    /// Le début d'une session : ce qui tourne, quand, et depuis quand l'appareil est allumé.
    Debut {
        /// L'heure du début, en millisecondes depuis 1970.
        epoque_ms: u64,
        version: &'static str,
        systeme: &'static str,
        architecture: &'static str,
        /// Le démarrage de l'appareil, en millisecondes depuis 1970, si le système le dit.
        demarrage_appareil_ms: Option<u64>,
    },
    /// Comment la session d'avant a fini — relu au lancement, et gardé ici pour voyager avec
    /// celle-ci.
    Precedente {
        instant_ms: u64,
        fin: &'static str,
        /// L'appareil a-t-il redémarré depuis sa dernière trace ?
        appareil_redemarre: bool,
        duree_ms: u64,
        batterie_pct: Option<u8>,
        en_charge: Option<bool>,
    },
    /// Un épisode clos : une suite d'images d'un même geste, et ce qu'elles ont coûté.
    Episode {
        instant_ms: u64,
        duree_ms: u64,
        geste: &'static str,
        images: u64,
        median_us: u32,
        p99_us: u32,
        pire_us: u32,
    },
    /// L'état de la machine, quand il a changé.
    Machine {
        instant_ms: u64,
        batterie_pct: Option<u8>,
        en_charge: Option<bool>,
    },
    /// Une image plus lente que toutes les précédentes de la session : un record, pas
    /// forcément un gel — sa durée le dit. Leur nombre croît comme le logarithme de celui des
    /// images, et c'est l'instant où la session a quelque chose de plus à dire (la règle de
    /// [`crate::chronique::Chronique::du_neuf`]).
    Pire {
        instant_ms: u64,
        duree_us: u32,
        geste: &'static str,
    },
    /// La fin propre de la session.
    Fin { instant_ms: u64 },
}

/// Une valeur de champ.
enum Champ {
    Entier(u64),
    Nom(&'static str),
    Booleen(bool),
    Rien,
}

impl From<Option<u64>> for Champ {
    fn from(v: Option<u64>) -> Self {
        v.map_or(Champ::Rien, Champ::Entier)
    }
}

impl From<Option<u8>> for Champ {
    fn from(v: Option<u8>) -> Self {
        v.map_or(Champ::Rien, |v| Champ::Entier(u64::from(v)))
    }
}

impl From<Option<bool>> for Champ {
    fn from(v: Option<bool>) -> Self {
        v.map_or(Champ::Rien, Champ::Booleen)
    }
}

impl Enregistrement {
    /// Son nom, le champ `type` de sa ligne.
    pub fn nom(&self) -> &'static str {
        match self {
            Self::Debut { .. } => "debut",
            Self::Precedente { .. } => "precedente",
            Self::Episode { .. } => "episode",
            Self::Machine { .. } => "machine",
            Self::Pire { .. } => "pire",
            Self::Fin { .. } => "fin",
        }
    }

    /// La ligne JSON, terminée par un saut de ligne.
    pub fn ligne(&self) -> String {
        let mut t = format!("{{\"type\":\"{}\"", self.nom());
        for (cle, valeur) in self.champs() {
            t.push_str(&format!(",\"{cle}\":"));
            match valeur {
                Champ::Entier(n) => t.push_str(&n.to_string()),
                Champ::Nom(s) => t.push_str(&format!("\"{}\"", echapper(s))),
                Champ::Booleen(b) => t.push_str(if b { "true" } else { "false" }),
                Champ::Rien => t.push_str("null"),
            }
        }
        t.push_str("}\n");
        t
    }

    fn champs(&self) -> Vec<(&'static str, Champ)> {
        use Champ::{Booleen, Entier, Nom};
        match *self {
            Self::Debut {
                epoque_ms,
                version,
                systeme,
                architecture,
                demarrage_appareil_ms,
            } => vec![
                ("epoque_ms", Entier(epoque_ms)),
                ("version", Nom(version)),
                ("systeme", Nom(systeme)),
                ("architecture", Nom(architecture)),
                ("demarrage_appareil_ms", demarrage_appareil_ms.into()),
            ],
            Self::Precedente {
                instant_ms,
                fin,
                appareil_redemarre,
                duree_ms,
                batterie_pct,
                en_charge,
            } => vec![
                ("instant_ms", Entier(instant_ms)),
                ("fin", Nom(fin)),
                ("appareil_redemarre", Booleen(appareil_redemarre)),
                ("duree_ms", Entier(duree_ms)),
                ("batterie_pct", batterie_pct.into()),
                ("en_charge", en_charge.into()),
            ],
            Self::Episode {
                instant_ms,
                duree_ms,
                geste,
                images,
                median_us,
                p99_us,
                pire_us,
            } => vec![
                ("instant_ms", Entier(instant_ms)),
                ("duree_ms", Entier(duree_ms)),
                ("geste", Nom(geste)),
                ("images", Entier(images)),
                ("median_us", Entier(u64::from(median_us))),
                ("p99_us", Entier(u64::from(p99_us))),
                ("pire_us", Entier(u64::from(pire_us))),
            ],
            Self::Machine {
                instant_ms,
                batterie_pct,
                en_charge,
            } => vec![
                ("instant_ms", Entier(instant_ms)),
                ("batterie_pct", batterie_pct.into()),
                ("en_charge", en_charge.into()),
            ],
            Self::Pire {
                instant_ms,
                duree_us,
                geste,
            } => vec![
                ("instant_ms", Entier(instant_ms)),
                ("duree_us", Entier(u64::from(duree_us))),
                ("geste", Nom(geste)),
            ],
            Self::Fin { instant_ms } => vec![("instant_ms", Entier(instant_ms))],
        }
    }
}

/// **La ligne d'une panique** : où, dans le code de Glucose — jamais son message, qui peut
/// citer une donnée.
///
/// La seule ligne qui porte un texte venu de l'exécution, et c'est pourquoi elle n'est pas un
/// [`Enregistrement`] : le chemin du fichier source où la panique a eu lieu, que le compilateur
/// a écrit dans le programme. Le système ne promet pas qu'il soit une constante ; il n'est
/// jamais une donnée de l'utilisateur.
pub fn ligne_de_panique(instant_ms: u64, fichier: &str, ligne: u32, principal: bool) -> String {
    format!(
        "{{\"type\":\"panique\",\"instant_ms\":{instant_ms},\"fichier\":\"{}\",\
         \"ligne\":{ligne},\"principal\":{principal}}}\n",
        echapper(fichier)
    )
}

/// Échappe ce que JSON demande. Les noms sont fixés à la compilation, mais la ligne doit rester
/// du JSON quel que soit celui qu'on y ajoutera demain.
fn echapper(s: &str) -> String {
    let mut t = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => t.push_str("\\\""),
            '\\' => t.push_str("\\\\"),
            c if (c as u32) < 0x20 => t.push_str(&format!("\\u{:04x}", c as u32)),
            c => t.push(c),
        }
    }
    t
}
