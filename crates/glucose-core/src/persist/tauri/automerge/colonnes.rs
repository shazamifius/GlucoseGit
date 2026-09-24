//! Les colonnes du format Automerge, lues **paresseusement**.
//!
//! Une colonne compresse ses valeurs par plages : « 5 fois 0 », « 3 nuls », « ces 4 valeurs ».
//! Déplier une colonne entière en mémoire serait le plus simple, et c'est ce qui laisserait un
//! fichier abîmé — une plage qui annonce 2⁶⁰ nuls en trois octets — épuiser la mémoire. Les
//! lecteurs ci-dessous rendent une valeur à la fois : une colonne ne coûte jamais que ses
//! octets, et aucune borne arbitraire n'a besoin d'exister.
//!
//! Les codages sont ceux de la spécification (« Column Types ») : plages signées pour les
//! entiers, deltas cumulés, booléens en longueurs alternées, chaînes préfixées.

use crate::error::CoreError;
use crate::persist::tauri::illisible;

/// Un curseur sur des octets, avec les deux codages d'entiers du format.
#[derive(Debug, Clone)]
pub struct Curseur<'a> {
    octets: &'a [u8],
    pos: usize,
}

impl<'a> Curseur<'a> {
    pub fn new(octets: &'a [u8]) -> Self {
        Self { octets, pos: 0 }
    }

    pub fn fini(&self) -> bool {
        self.pos >= self.octets.len()
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    /// Un entier non signé, 7 bits par octet (uLEB128).
    pub fn uleb(&mut self) -> Result<u64, CoreError> {
        let mut valeur = 0u64;
        let mut decalage = 0u32;
        loop {
            let octet = self.octet()?;
            let bits = u64::from(octet & 0x7f);
            if decalage >= 64 || (decalage == 63 && bits > 1) {
                return Err(illisible("un entier dépasse 64 bits"));
            }
            valeur |= bits << decalage;
            if octet & 0x80 == 0 {
                return Ok(valeur);
            }
            decalage += 7;
        }
    }

    /// Un entier signé en complément à deux, étendu par son signe (LEB128).
    pub fn leb(&mut self) -> Result<i64, CoreError> {
        let mut valeur = 0i64;
        let mut decalage = 0u32;
        loop {
            let octet = self.octet()?;
            if decalage >= 64 {
                return Err(illisible("un entier signé dépasse 64 bits"));
            }
            valeur |= i64::from(octet & 0x7f) << decalage;
            decalage += 7;
            if octet & 0x80 == 0 {
                if decalage < 64 && octet & 0x40 != 0 {
                    valeur |= -1i64 << decalage;
                }
                return Ok(valeur);
            }
        }
    }

    pub fn octet(&mut self) -> Result<u8, CoreError> {
        let o = *self
            .octets
            .get(self.pos)
            .ok_or_else(|| illisible("données tronquées"))?;
        self.pos += 1;
        Ok(o)
    }

    pub fn prendre(&mut self, n: u64) -> Result<&'a [u8], CoreError> {
        let n = usize::try_from(n).map_err(|_| illisible("longueur démesurée"))?;
        let fin = self
            .pos
            .checked_add(n)
            .filter(|f| *f <= self.octets.len())
            .ok_or_else(|| illisible("données tronquées"))?;
        let tranche = &self.octets[self.pos..fin];
        self.pos = fin;
        Ok(tranche)
    }

    /// Une chaîne préfixée par sa longueur. Les octets qui ne sont pas de l'UTF-8 valide
    /// deviennent le caractère de remplacement, comme la spécification le demande.
    pub fn chaine(&mut self) -> Result<String, CoreError> {
        let n = self.uleb()?;
        Ok(String::from_utf8_lossy(self.prendre(n)?).into_owned())
    }
}

/// Ce qu'une plage répète.
#[derive(Debug, Clone)]
enum Plage<T> {
    /// Rien à rendre : il faut lire l'en-tête de la plage suivante.
    Vide,
    Repete(Option<T>, u64),
    Litterale(u64),
}

/// Une colonne codée par plages, dont chaque valeur se lit avec `lire`.
///
/// Une colonne absente du fichier — la spécification les omet quand elles ne portent que des
/// nuls — se représente par une colonne sur zéro octet : elle rend des nuls sans fin.
pub struct Plages<'a, T, F> {
    c: Curseur<'a>,
    plage: Plage<T>,
    lire: F,
}

impl<'a, T: Clone, F: FnMut(&mut Curseur<'a>) -> Result<T, CoreError>> Plages<'a, T, F> {
    pub fn new(octets: &'a [u8], lire: F) -> Self {
        Self {
            c: Curseur::new(octets),
            plage: Plage::Vide,
            lire,
        }
    }

    /// La valeur suivante, `None` pour un nul.
    pub fn suivante(&mut self) -> Result<Option<T>, CoreError> {
        loop {
            match &mut self.plage {
                Plage::Repete(v, n) if *n > 0 => {
                    *n -= 1;
                    return Ok(v.clone());
                }
                Plage::Litterale(n) if *n > 0 => {
                    *n -= 1;
                    return (self.lire)(&mut self.c).map(Some);
                }
                _ => {}
            }
            if self.c.fini() {
                return Ok(None);
            }
            let n = self.c.leb()?;
            self.plage = match n {
                0 => Plage::Repete(None, self.c.uleb()?),
                n if n > 0 => Plage::Repete(Some((self.lire)(&mut self.c)?), n as u64),
                n => Plage::Litterale(n.unsigned_abs()),
            };
        }
    }

    /// Combien de valeurs la colonne porte en tout — sans les déplier. Consomme la colonne.
    pub fn compter(mut self) -> Result<u64, CoreError> {
        let mut total = 0u64;
        while !self.c.fini() {
            let n = self.c.leb()?;
            let k = match n {
                0 => self.c.uleb()?,
                n if n > 0 => {
                    (self.lire)(&mut self.c)?;
                    n as u64
                }
                n => {
                    let k = n.unsigned_abs();
                    for _ in 0..k {
                        (self.lire)(&mut self.c)?;
                    }
                    k
                }
            };
            total = total
                .checked_add(k)
                .ok_or_else(|| illisible("colonne démesurée"))?;
        }
        Ok(total)
    }
}

/// Une colonne d'entiers non signés, telle que [`entiers`] l'ouvre.
pub type Entiers<'a> = Plages<'a, u64, fn(&mut Curseur<'a>) -> Result<u64, CoreError>>;
/// Une colonne d'entiers signés, que [`Deltas`] cumule.
pub type Signes<'a> = Plages<'a, i64, fn(&mut Curseur<'a>) -> Result<i64, CoreError>>;
/// Une colonne de chaînes, telle que [`chaines`] l'ouvre.
pub type Chaines<'a> = Plages<'a, String, fn(&mut Curseur<'a>) -> Result<String, CoreError>>;

/// Une colonne d'entiers non signés (acteurs, actions, groupes, métadonnées de valeur).
pub fn entiers(octets: &[u8]) -> Entiers<'_> {
    Plages::new(octets, Curseur::uleb)
}

/// Une colonne de chaînes.
pub fn chaines(octets: &[u8]) -> Chaines<'_> {
    Plages::new(octets, Curseur::chaine)
}

/// Une colonne de deltas : chaque valeur est la précédente plus ce que la plage porte, à
/// partir de zéro. Un nul ne fait pas avancer la somme.
pub struct Deltas<'a> {
    plages: Signes<'a>,
    somme: i64,
}

impl<'a> Deltas<'a> {
    pub fn new(octets: &'a [u8]) -> Self {
        Self {
            plages: Plages::new(octets, Curseur::leb),
            somme: 0,
        }
    }

    pub fn suivante(&mut self) -> Result<Option<i64>, CoreError> {
        Ok(match self.plages.suivante()? {
            Some(d) => {
                self.somme = self.somme.wrapping_add(d);
                Some(self.somme)
            }
            None => None,
        })
    }
}

/// Une colonne de booléens : des longueurs alternées de faux puis de vrai, en commençant
/// toujours par faux. Une colonne épuisée rend faux.
pub struct Booleens<'a> {
    c: Curseur<'a>,
    valeur: bool,
    reste: u64,
}

impl<'a> Booleens<'a> {
    pub fn new(octets: &'a [u8]) -> Self {
        Self {
            c: Curseur::new(octets),
            // La première longueur lue compte des faux : on part de vrai pour basculer.
            valeur: true,
            reste: 0,
        }
    }

    pub fn suivante(&mut self) -> Result<bool, CoreError> {
        while self.reste == 0 {
            if self.c.fini() {
                return Ok(false);
            }
            self.reste = self.c.uleb()?;
            self.valeur = !self.valeur;
        }
        self.reste -= 1;
        Ok(self.valeur)
    }
}

/// Une valeur primitive, telle que la colonne de valeurs la porte.
#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Nul,
    Booleen(bool),
    Entier(i64),
    Flottant(f64),
    Texte(String),
    Octets(Vec<u8>),
    /// Un compteur : sa valeur de départ, à laquelle s'ajoutent ses incréments.
    Compteur(i64),
    /// Un type qu'une version future d'Automerge aurait ajouté. Il se garde sans se lire.
    Inconnu,
}

/// La paire « métadonnées + octets bruts » d'une colonne de valeurs.
pub struct Valeurs<'a> {
    meta: Entiers<'a>,
    brut: Curseur<'a>,
}

impl<'a> Valeurs<'a> {
    pub fn new(meta: &'a [u8], brut: &'a [u8]) -> Self {
        Self {
            meta: entiers(meta),
            brut: Curseur::new(brut),
        }
    }

    pub fn suivante(&mut self) -> Result<Primitive, CoreError> {
        let Some(m) = self.meta.suivante()? else {
            return Ok(Primitive::Nul);
        };
        let longueur = m >> 4;
        let octets = self.brut.prendre(longueur)?;
        let mut c = Curseur::new(octets);
        Ok(match m & 0x0f {
            0 => Primitive::Nul,
            1 => Primitive::Booleen(false),
            2 => Primitive::Booleen(true),
            3 => Primitive::Entier(i64::try_from(c.uleb()?).unwrap_or(i64::MAX)),
            4 | 9 => Primitive::Entier(c.leb()?),
            5 => {
                let tableau: [u8; 8] = octets
                    .try_into()
                    .map_err(|_| illisible("un flottant ne fait pas 8 octets"))?;
                Primitive::Flottant(f64::from_le_bytes(tableau))
            }
            6 => Primitive::Texte(String::from_utf8_lossy(octets).into_owned()),
            7 => Primitive::Octets(octets.to_vec()),
            8 => Primitive::Compteur(c.leb()?),
            _ => Primitive::Inconnu,
        })
    }
}
