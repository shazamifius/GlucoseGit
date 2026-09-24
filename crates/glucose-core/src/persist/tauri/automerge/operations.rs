//! Les lignes d'opérations d'un morceau : une opération par ligne, un champ par colonne.
//!
//! Un morceau « document » et un morceau « delta » rangent leurs opérations dans les mêmes
//! colonnes, à deux différences près que ce module absorbe :
//!
//! * le document écrit l'identifiant de chaque opération ; le delta le **déduit** — la
//!   i-ème opération d'un delta a pour compteur `start_op + i` et pour auteur celui du delta ;
//! * le document écrit les **successeurs** d'une opération (ce qui l'a remplacée), le delta
//!   ses **prédécesseurs** (ce qu'elle remplace). Les deux décrivent la même arête, lue d'un
//!   bout ou de l'autre.

use super::colonnes::{chaines, entiers, Booleens, Chaines, Deltas, Entiers, Primitive, Valeurs};
use crate::error::CoreError;
use crate::persist::tauri::illisible;
use std::borrow::Cow;

/// L'identifiant d'une opération : un compteur, et l'auteur par son rang dans la table
/// globale des auteurs du document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OpId {
    pub compteur: u64,
    pub auteur: u32,
}

/// L'objet qu'une opération modifie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Objet {
    Racine,
    Cree(OpId),
}

/// La place qu'une opération vise dans son objet.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Cle {
    /// Une clé de carte.
    Carte(String),
    /// Le début d'une liste : une insertion « après la tête » se place en premier.
    Tete,
    /// Un élément de liste, désigné par l'opération qui l'a inséré.
    Element(OpId),
}

/// Une opération, telle que la reconstruction de l'état en a besoin.
#[derive(Debug, Clone)]
pub struct Op {
    pub id: OpId,
    pub objet: Objet,
    pub cle: Cle,
    pub insertion: bool,
    pub action: u64,
    pub valeur: Primitive,
    /// Dans un document : ce qui a remplacé cette opération. Dans un delta : ce qu'elle
    /// remplace. [`super::lire`] ramène tout au premier sens avant de reconstruire.
    pub liens: Vec<OpId>,
}

/// Les identifiants des colonnes d'opérations (spécification, « Operation Columns »), sans le
/// bit de compression.
mod spec {
    pub const OBJ_AUTEUR: u64 = 1;
    pub const OBJ_COMPTEUR: u64 = 2;
    pub const CLE_AUTEUR: u64 = 17;
    pub const CLE_COMPTEUR: u64 = 19;
    pub const CLE_CHAINE: u64 = 21;
    pub const ID_AUTEUR: u64 = 33;
    pub const ID_COMPTEUR: u64 = 35;
    pub const INSERTION: u64 = 52;
    pub const ACTION: u64 = 66;
    pub const VALEUR_META: u64 = 86;
    pub const VALEUR: u64 = 87;
    pub const PRED_GROUPE: u64 = 112;
    pub const PRED_AUTEUR: u64 = 113;
    pub const PRED_COMPTEUR: u64 = 115;
    pub const SUCC_GROUPE: u64 = 128;
    pub const SUCC_AUTEUR: u64 = 129;
    pub const SUCC_COMPTEUR: u64 = 131;
}

/// Le bit qui marque une colonne compressée (DEFLATE), permis dans les seuls documents.
pub const BIT_DEFLATE: u64 = 0b1000;

/// Les colonnes d'un morceau, rangées par spécification, décompressées si besoin.
pub struct Colonnes<'a> {
    colonnes: Vec<(u64, Cow<'a, [u8]>)>,
}

impl<'a> Colonnes<'a> {
    /// Découpe `donnees` selon la table `(spécification, longueur)`, et décompresse ce qui
    /// doit l'être.
    pub fn decouper(
        table: &[(u64, u64)],
        donnees: &'a [u8],
        compression_permise: bool,
    ) -> Result<Self, CoreError> {
        let mut c = super::colonnes::Curseur::new(donnees);
        let mut colonnes = Vec::with_capacity(table.len());
        for &(spec, longueur) in table {
            let brut = c.prendre(longueur)?;
            let contenu = if spec & BIT_DEFLATE != 0 {
                if !compression_permise {
                    return Err(illisible(
                        "une colonne compressée dans un delta, ce que le format interdit",
                    ));
                }
                Cow::Owned(
                    crate::persist::tauri::inflate::inflate(brut)
                        .map_err(|_| illisible("une colonne compressée ne se décompresse pas"))?,
                )
            } else {
                Cow::Borrowed(brut)
            };
            colonnes.push((spec & !BIT_DEFLATE, contenu));
        }
        Ok(Self { colonnes })
    }

    /// Les octets d'une colonne, vides si elle est absente — une colonne absente ne porte que
    /// des nuls, et c'est exactement ce que rend une colonne vide.
    pub fn octets(&self, spec: u64) -> &[u8] {
        self.colonnes
            .iter()
            .find(|(s, _)| *s == spec)
            .map(|(_, o)| o.as_ref())
            .unwrap_or(&[])
    }
}

/// D'où viennent les identifiants et dans quel sens vont les liens.
pub enum Forme {
    /// Un document : identifiants écrits, liens vers les successeurs.
    Document,
    /// Un delta : identifiants déduits de `start_op`, liens vers les prédécesseurs.
    Delta { start_op: u64 },
}

/// Lit toutes les opérations d'un morceau.
///
/// `auteurs` traduit un rang d'auteur **local au morceau** en rang dans la table globale.
pub fn lire(cols: &Colonnes<'_>, forme: &Forme, auteurs: &[u32]) -> Result<Vec<Op>, CoreError> {
    let lignes = entiers(cols.octets(spec::ACTION)).compter()?;
    let n = usize::try_from(lignes).map_err(|_| illisible("trop d'opérations"))?;
    let mut ops = Vec::new();
    ops.try_reserve_exact(n).map_err(|_| {
        illisible("le document annonce plus d'opérations que la mémoire n'en peut tenir")
    })?;
    let mut l = Lignes::new(cols, forme, auteurs);
    for i in 0..lignes {
        ops.push(l.suivante(i)?);
    }
    Ok(ops)
}

/// Une tête de lecture par colonne : chaque ligne avance chacune d'un cran.
struct Lignes<'a> {
    forme: &'a Forme,
    auteurs: &'a [u32],
    obj_a: Entiers<'a>,
    obj_c: Entiers<'a>,
    cle_a: Entiers<'a>,
    cle_c: Deltas<'a>,
    cle_s: Chaines<'a>,
    id_a: Entiers<'a>,
    id_c: Deltas<'a>,
    insertion: Booleens<'a>,
    action: Entiers<'a>,
    valeurs: Valeurs<'a>,
    lien_g: Entiers<'a>,
    lien_a: Entiers<'a>,
    lien_c: Deltas<'a>,
}

impl<'a> Lignes<'a> {
    fn new(cols: &'a Colonnes<'_>, forme: &'a Forme, auteurs: &'a [u32]) -> Self {
        let (g, a, c) = match forme {
            Forme::Document => (spec::SUCC_GROUPE, spec::SUCC_AUTEUR, spec::SUCC_COMPTEUR),
            Forme::Delta { .. } => (spec::PRED_GROUPE, spec::PRED_AUTEUR, spec::PRED_COMPTEUR),
        };
        Self {
            forme,
            auteurs,
            obj_a: entiers(cols.octets(spec::OBJ_AUTEUR)),
            obj_c: entiers(cols.octets(spec::OBJ_COMPTEUR)),
            cle_a: entiers(cols.octets(spec::CLE_AUTEUR)),
            cle_c: Deltas::new(cols.octets(spec::CLE_COMPTEUR)),
            cle_s: chaines(cols.octets(spec::CLE_CHAINE)),
            id_a: entiers(cols.octets(spec::ID_AUTEUR)),
            id_c: Deltas::new(cols.octets(spec::ID_COMPTEUR)),
            insertion: Booleens::new(cols.octets(spec::INSERTION)),
            action: entiers(cols.octets(spec::ACTION)),
            valeurs: Valeurs::new(cols.octets(spec::VALEUR_META), cols.octets(spec::VALEUR)),
            lien_g: entiers(cols.octets(g)),
            lien_a: entiers(cols.octets(a)),
            lien_c: Deltas::new(cols.octets(c)),
        }
    }

    fn auteur(&self, rang: Option<u64>) -> Result<u32, CoreError> {
        rang.and_then(|r| self.auteurs.get(usize::try_from(r).ok()?).copied())
            .ok_or_else(|| illisible("un auteur hors de la table des auteurs"))
    }

    /// La `i`-ème opération du morceau.
    fn suivante(&mut self, i: u64) -> Result<Op, CoreError> {
        let id = match self.forme {
            Forme::Document => {
                let rang = self.id_a.suivante()?;
                OpId {
                    compteur: compteur(self.id_c.suivante()?)?,
                    auteur: self.auteur(rang)?,
                }
            }
            Forme::Delta { start_op } => OpId {
                compteur: start_op + i,
                auteur: self.auteur(Some(0))?,
            },
        };
        let objet = match (self.obj_a.suivante()?, self.obj_c.suivante()?) {
            (None, None) | (_, Some(0)) => Objet::Racine,
            (a, Some(c)) => Objet::Cree(OpId {
                compteur: c,
                auteur: self.auteur(a)?,
            }),
            (Some(_), None) => return Err(illisible("un objet sans compteur")),
        };
        let cle = self.cle()?;
        let liens = self.liens()?;
        Ok(Op {
            id,
            objet,
            cle,
            insertion: self.insertion.suivante()?,
            action: self
                .action
                .suivante()?
                .ok_or_else(|| illisible("une opération sans action"))?,
            valeur: self.valeurs.suivante()?,
            liens,
        })
    }

    fn cle(&mut self) -> Result<Cle, CoreError> {
        let (ka, kc, ks) = (
            self.cle_a.suivante()?,
            self.cle_c.suivante()?,
            self.cle_s.suivante()?,
        );
        Ok(match (ks, kc) {
            (Some(s), _) => Cle::Carte(s),
            (None, Some(0)) => Cle::Tete,
            (None, Some(c)) => Cle::Element(OpId {
                compteur: compteur(Some(c))?,
                auteur: self.auteur(ka)?,
            }),
            (None, None) => return Err(illisible("une opération sans clé")),
        })
    }

    fn liens(&mut self) -> Result<Vec<OpId>, CoreError> {
        let n = self.lien_g.suivante()?.unwrap_or(0);
        let mut liens = Vec::new();
        for _ in 0..n {
            let rang = self.lien_a.suivante()?;
            liens.push(OpId {
                compteur: compteur(self.lien_c.suivante()?)?,
                auteur: self.auteur(rang)?,
            });
        }
        Ok(liens)
    }
}

fn compteur(c: Option<i64>) -> Result<u64, CoreError> {
    c.and_then(|c| u64::try_from(c).ok())
        .ok_or_else(|| illisible("un compteur d'opération absent ou négatif"))
}
