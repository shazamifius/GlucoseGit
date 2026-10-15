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
    /// **Un signe de vie de la voie du pavé** (fiche 54) : un contact confié au système, un
    /// changement d'état, un geste qui commence ou finit. Sa coupure de 7 à 10 secondes ne
    /// laissait aucune trace ; ces lignes disent à quel étage elle s'arrête.
    Pave {
        instant_ms: u64,
        quoi: &'static str,
        valeur: u64,
    },
    /// **La carte graphique qui présente** (fiche 60), au lancement et à chaque réouverture.
    Carte { instant_ms: u64, carte: Carte },
    /// **Une étape du dessin qui a ralenti un épisode** (POSTES-1, fiche 63).
    Poste(Poste),
    /// **Glucose passe derrière une autre application** (BOITE-ANDROID, fiche 65) : sous
    /// Android, le système peut alors le tuer sans prévenir — une session qui finit là a fini
    /// comme finit une application de téléphone, et non par une faute.
    ArrierePlan { instant_ms: u64 },
    /// Glucose revient au premier plan.
    PremierPlan { instant_ms: u64 },
    /// **Ce que Glucose occupe, et ce que la machine laisse** (MEMOIRE-1, fiche 66) : un record
    /// — l'empreinte la plus haute de la session, ou ce qui reste le plus bas —, comme
    /// [`Self::Pire`] pour les images. Son téléphone s'est figé sans qu'aucune ligne ne dise la
    /// mémoire.
    Memoire {
        instant_ms: u64,
        occupation: Occupation,
    },
    /// **Le système demande de rendre de la mémoire** (MEMOIRE-2) : sous Android, `onTrimMemory`
    /// — dont le niveau se perd en route — et `onLowMemory`.
    AlerteMemoire { instant_ms: u64, arriere_plan: bool },
    /// **Comment Android a fermé la session d'avant** (SORTIE-1, fiche 72) : la raison qu'il a
    /// gardée, et la mémoire qu'elle occupait à sa mort — relu au lancement, comme `precedente`.
    Sortie {
        instant_ms: u64,
        fermeture: super::sortie::Fermeture,
    },
    /// **Ce que le lancement a rouvert** (REPRISE-1, fiche 72) : rien, le brouillon qu'un arrêt
    /// a laissé, le dernier document — ou un échec —, et ce qu'il porte.
    Reprise { instant_ms: u64, reprise: Reprise },
    /// La fin propre de la session.
    Fin { instant_ms: u64 },
}

/// **Ce que le lancement a rouvert** (REPRISE-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reprise {
    /// `rien`, `brouillon`, `document` ou `echec`.
    pub rouvert: &'static str,
    /// Les brouillons qui attendaient, celui qu'on rouvre compris.
    pub brouillons: u64,
    /// Les images du document ouvert après la reprise.
    pub images: u64,
    /// Le poids du fichier rouvert, en mébioctets.
    pub mo: u64,
}

impl Reprise {
    /// Tout ce que `rouvert` peut valoir : une épreuve vérifie que le serveur le connaît.
    pub const ROUVERTS: [&'static str; 4] = ["rien", "brouillon", "document", "echec"];

    fn champs(&self, instant_ms: u64) -> Vec<(&'static str, Champ)> {
        use Champ::{Entier, Nom};
        vec![
            ("instant_ms", Entier(instant_ms)),
            ("rouvert", Nom(self.rouvert)),
            ("brouillons", Entier(self.brouillons)),
            ("images", Entier(self.images)),
            ("mo", Entier(self.mo)),
        ]
    }
}

/// **Ce que Glucose occupe, et ce que la machine laisse**, en mébioctets (MEMOIRE-1).
///
/// Chaque grandeur a sa source, et aucune ne se déduit d'une autre : ce que le système compte
/// pour le processus, ce qu'il dit pouvoir donner encore, et ce que Glucose sait tenir lui-même
/// — les niveaux d'images tenus et offerts (ETAGES-1), les textures confiées à la carte. Ce qui
/// ne se lit pas sur cette plateforme est absent, jamais zéro (cliquet 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Occupation {
    /// La mémoire résidente du processus, telle que le système la compte.
    pub processus_mo: Option<u64>,
    /// Ce que le système peut encore donner sans faire pagner (`MemAvailable` sous Linux et
    /// Android, ce que lit aussi `ActivityManager.getMemoryInfo`).
    pub disponible_mo: Option<u64>,
    /// Les niveaux d'images que Glucose tient en mémoire vive.
    pub images_mo: u64,
    /// Ceux qu'il a offerts au système, qui peut les reprendre sans rien demander.
    pub offerts_mo: u64,
    /// Ce que les décodages en cours tiendront au plus (MEMOIRE-3) : la mémoire qu'une image
    /// prend avant d'entrer au magasin.
    pub decodages_mo: u64,
    /// Les textures que Glucose a confiées à la carte graphique — sa propre comptabilité, la
    /// même partout ; absente sur la voie processeur.
    pub carte_mo: Option<u64>,
}

impl Occupation {
    /// Des octets en mébioctets entiers.
    pub fn mo(octets: u64) -> u64 {
        octets >> 20
    }

    fn champs(&self, instant_ms: u64) -> Vec<(&'static str, Champ)> {
        use Champ::Entier;
        vec![
            ("instant_ms", Entier(instant_ms)),
            ("processus_mo", self.processus_mo.into()),
            ("disponible_mo", self.disponible_mo.into()),
            ("images_mo", Entier(self.images_mo)),
            ("offerts_mo", Entier(self.offerts_mo)),
            ("decodages_mo", Entier(self.decodages_mo)),
            ("carte_mo", self.carte_mo.into()),
        ]
    }
}

/// **Une carte graphique, par ses numéros et des noms fermés** (fiche 60) — jamais le nom que
/// donne son pilote : c'est un texte venu de l'appareil, et le type ne doit pas pouvoir en
/// porter. Le fabricant et le modèle se relisent dans les tables publiques (PCI, Khronos).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Carte {
    /// Le moteur graphique : `vulkan`, `gl`, `dx12`, `metal`, `webgpu`, `aucun`.
    pub moteur: &'static str,
    /// `integree`, `dediee`, `virtuelle`, `processeur` ou `autre`.
    pub genre: &'static str,
    /// L'identifiant du fabricant : PCI sur un ordinateur, Khronos sur un téléphone.
    pub fabricant: u32,
    /// L'identifiant du modèle chez ce fabricant — zéro quand le pilote ne le dit pas (GL).
    pub modele: u32,
    /// Le côté de la plus grande texture qu'elle accepte (PLAFOND-1, fiche 59).
    pub plafond: u32,
}

impl Carte {
    /// La carte que `wgpu` a ouverte.
    pub fn de(info: &wgpu::AdapterInfo, plafond: u32) -> Self {
        Self {
            moteur: moteur(info.backend),
            genre: genre(info.device_type),
            fabricant: info.vendor,
            modele: info.device,
            plafond,
        }
    }

    /// Les champs de sa ligne, à cet instant.
    fn champs(&self, instant_ms: u64) -> Vec<(&'static str, Champ)> {
        use Champ::{Entier, Nom};
        vec![
            ("instant_ms", Entier(instant_ms)),
            ("moteur", Nom(self.moteur)),
            ("genre", Nom(self.genre)),
            ("fabricant", Entier(u64::from(self.fabricant))),
            ("modele", Entier(u64::from(self.modele))),
            ("plafond", Entier(u64::from(self.plafond))),
        ]
    }
}

/// **Une étape du dessin d'un épisode trop lent** (POSTES-1, fiche 63) : son nom — une marque
/// de mesure du code (`perf::stage`), fixée à la compilation — et ce qu'elle a coûté pendant
/// l'épisode. C'est ce qui dit, d'une machine qu'on ne possède pas, **où** part le temps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Poste {
    /// Quand l'épisode a commencé.
    pub instant_ms: u64,
    pub geste: &'static str,
    pub nom: &'static str,
    /// Ce qu'elle a coûté à une image ordinaire de l'épisode, et aux plus lentes.
    pub median_us: u32,
    pub p99_us: u32,
}

impl Poste {
    fn champs(&self) -> Vec<(&'static str, Champ)> {
        use Champ::{Entier, Nom};
        vec![
            ("instant_ms", Entier(self.instant_ms)),
            ("geste", Nom(self.geste)),
            ("nom", Nom(self.nom)),
            ("median_us", Entier(u64::from(self.median_us))),
            ("p99_us", Entier(u64::from(self.p99_us))),
        ]
    }
}

/// Le nom fermé d'un moteur graphique.
pub fn moteur(b: wgpu::Backend) -> &'static str {
    match b {
        wgpu::Backend::Vulkan => "vulkan",
        wgpu::Backend::Gl => "gl",
        wgpu::Backend::Dx12 => "dx12",
        wgpu::Backend::Metal => "metal",
        wgpu::Backend::BrowserWebGpu => "webgpu",
        wgpu::Backend::Noop => "aucun",
    }
}

/// Le nom fermé d'un genre de carte.
pub fn genre(d: wgpu::DeviceType) -> &'static str {
    match d {
        wgpu::DeviceType::IntegratedGpu => "integree",
        wgpu::DeviceType::DiscreteGpu => "dediee",
        wgpu::DeviceType::VirtualGpu => "virtuelle",
        wgpu::DeviceType::Cpu => "processeur",
        wgpu::DeviceType::Other => "autre",
    }
}

/// Les champs de la ligne `sortie` (SORTIE-1, fiche 72).
fn champs_de_la_sortie(
    instant_ms: u64,
    f: &super::sortie::Fermeture,
) -> Vec<(&'static str, Champ)> {
    use Champ::{Entier, Nom};
    vec![
        ("instant_ms", Entier(instant_ms)),
        ("raison", Nom(f.raison)),
        ("statut", f.statut.into()),
        ("importance", Entier(f.importance)),
        ("pss_mo", Entier(f.pss_mo)),
        ("rss_mo", Entier(f.rss_mo)),
    ]
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
            Self::Pave { .. } => "pave",
            Self::Carte { .. } => "carte",
            Self::Poste(_) => "poste",
            Self::ArrierePlan { .. } => "arriere_plan",
            Self::PremierPlan { .. } => "premier_plan",
            Self::Memoire { .. } => "memoire",
            Self::AlerteMemoire { .. } => "alerte_memoire",
            Self::Sortie { .. } => "sortie",
            Self::Reprise { .. } => "reprise",
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
            Self::Debut { .. } => self.champs_du_debut(),
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
            Self::Pave {
                instant_ms,
                quoi,
                valeur,
            } => vec![
                ("instant_ms", Entier(instant_ms)),
                ("quoi", Nom(quoi)),
                ("valeur", Entier(valeur)),
            ],
            Self::Carte { instant_ms, carte } => carte.champs(instant_ms),
            Self::Reprise {
                instant_ms,
                reprise,
            } => reprise.champs(instant_ms),
            Self::Poste(poste) => poste.champs(),
            Self::Memoire { .. } | Self::AlerteMemoire { .. } => self.champs_de_la_memoire(),
            Self::Sortie {
                instant_ms,
                fermeture,
            } => champs_de_la_sortie(instant_ms, &fermeture),
            Self::ArrierePlan { instant_ms }
            | Self::PremierPlan { instant_ms }
            | Self::Fin { instant_ms } => vec![("instant_ms", Entier(instant_ms))],
        }
    }
}

impl Enregistrement {
    /// Les champs des deux lignes de la mémoire (MEMOIRE-1, 2).
    fn champs_de_la_memoire(&self) -> Vec<(&'static str, Champ)> {
        use Champ::{Booleen, Entier};
        match *self {
            Self::Memoire {
                instant_ms,
                occupation,
            } => occupation.champs(instant_ms),
            Self::AlerteMemoire {
                instant_ms,
                arriere_plan,
            } => vec![
                ("instant_ms", Entier(instant_ms)),
                ("arriere_plan", Booleen(arriere_plan)),
            ],
            _ => Vec::new(),
        }
    }

    /// Les champs du début d'une session : ce qui tourne, quand, et depuis quand l'appareil est
    /// allumé.
    fn champs_du_debut(&self) -> Vec<(&'static str, Champ)> {
        use Champ::{Entier, Nom};
        let Self::Debut {
            epoque_ms,
            version,
            systeme,
            architecture,
            demarrage_appareil_ms,
        } = *self
        else {
            return Vec::new();
        };
        vec![
            ("epoque_ms", Entier(epoque_ms)),
            ("version", Nom(version)),
            ("systeme", Nom(systeme)),
            ("architecture", Nom(architecture)),
            ("demarrage_appareil_ms", demarrage_appareil_ms.into()),
        ]
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

/// **La ligne d'un plantage que le système a vu**, pour la session d'avant.
///
/// Avec celle d'une panique, la seule à porter des textes venus de l'exécution : le nom d'un
/// module et sa version, que [`super::plantage`] ne laisse passer que faits de lettres, de
/// chiffres et de `.-_+` — un nom de fichier, jamais un chemin.
pub fn ligne_de_plantage(instant_ms: u64, p: &super::plantage::Plantage) -> String {
    let texte = |t: &Option<String>| {
        t.as_deref()
            .map_or("null".to_string(), |t| format!("\"{}\"", echapper(t)))
    };
    let nombre = |n: Option<u64>| n.map_or("null".to_string(), |n| n.to_string());
    format!(
        "{{\"type\":\"plantage\",\"instant_ms\":{instant_ms},\"gel\":{},\"module\":{},\
         \"version_du_module\":{},\"code\":{},\"decalage\":{}}}\n",
        p.gel,
        texte(&p.module),
        texte(&p.version_du_module),
        nombre(p.code.map(u64::from)),
        nombre(p.decalage),
    )
}

/// Échappe ce que JSON demande. Les noms sont fixés à la compilation, mais la ligne doit rester
/// du JSON quel que soit celui qu'on y ajoutera demain.
pub(crate) fn echapper(s: &str) -> String {
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
