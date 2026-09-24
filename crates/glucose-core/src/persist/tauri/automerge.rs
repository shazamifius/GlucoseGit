//! Notre lecteur du format binaire d'Automerge — celui des fichiers de Glucose Tauri.
//!
//! # Pourquoi nous l'écrivons plutôt que d'importer la bibliothèque
//!
//! La bibliothèque de référence (`automerge`) lit ces fichiers, et elle a été essayée sur les
//! documents de l'utilisateur : elle y arrive. Elle ajouterait **vingt-quatre caisses** à
//! l'application pour un usage unique — lire un ancien document une fois —, alors que ce qu'il
//! faut pour en tirer l'**état courant** tient dans la spécification publique du format
//! (<https://automerge.org/automerge-binary-format-spec/>) : des morceaux, des colonnes, et
//! deux règles de résolution. La seule brique lourde, DEFLATE, est déjà dans l'application
//! (`miniz_oxide`, tiré par PNG).
//!
//! La bibliothèque reste là où elle sert le mieux : **dans les épreuves**, comme oracle. Notre
//! lecteur doit rendre, sur les mêmes octets, exactement la valeur qu'elle rend
//! (`tests/tauri_suite.rs`, et `examples/oracle_tauri.rs` sur les vrais fichiers).
//!
//! # Ce que ce lecteur ne fait pas
//!
//! Il ne garde pas l'histoire : il lit toutes les opérations, en déduit l'état, et les oublie.
//! Il ne vérifie pas le graphe des changements (les empreintes des têtes) : la somme de
//! contrôle de chaque morceau suffit à refuser des octets abîmés, et la cohérence de
//! l'histoire n'a pas d'effet sur l'état qu'on en tire.

mod colonnes;
mod etat;
mod operations;

use crate::error::CoreError;
use crate::persist::tauri::{illisible, Valeur};
use colonnes::Curseur;
use operations::{Colonnes, Forme, Op};
use std::collections::HashMap;

/// La signature d'un morceau Automerge.
pub const MAGIE: [u8; 4] = [0x85, 0x6f, 0x4a, 0x83];

const MORCEAU_DOCUMENT: u8 = 0;
const MORCEAU_DELTA: u8 = 1;
const MORCEAU_DELTA_COMPRESSE: u8 = 2;

/// Ce qu'une lecture a rendu.
#[derive(Debug, Clone, PartialEq)]
pub struct Lu {
    pub valeur: Valeur,
    /// Octets de fin ignorés parce qu'ils ne formaient pas un morceau entier et sain — la trace
    /// d'un enregistrement interrompu. Zéro pour un fichier intact.
    pub fin_ignoree: usize,
}

/// La table des auteurs du document entier : chaque morceau y inscrit les siens.
#[derive(Default)]
pub struct Auteurs {
    liste: Vec<Vec<u8>>,
    rang: HashMap<Vec<u8>, u32>,
}

impl Auteurs {
    fn inscrire(&mut self, auteur: &[u8]) -> u32 {
        if let Some(r) = self.rang.get(auteur) {
            return *r;
        }
        let r = self.liste.len() as u32;
        self.liste.push(auteur.to_vec());
        self.rang.insert(auteur.to_vec(), r);
        r
    }

    /// Le rang de chaque auteur dans l'ordre des octets de son identifiant : c'est cet ordre
    /// qui départage deux opérations de même compteur (horloge de Lamport).
    pub fn ordre(&self) -> Vec<u32> {
        let mut tries: Vec<u32> = (0..self.liste.len() as u32).collect();
        tries.sort_by(|a, b| self.liste[*a as usize].cmp(&self.liste[*b as usize]));
        let mut ordre = vec![0u32; self.liste.len()];
        for (position, auteur) in tries.into_iter().enumerate() {
            ordre[auteur as usize] = position as u32;
        }
        ordre
    }
}

/// Lit un fichier Automerge entier et rend l'état de son document.
///
/// Les morceaux se lisent jusqu'au bout du fichier. Un morceau abîmé **après** au moins un
/// morceau sain arrête la lecture sans la faire échouer : c'est la marque d'un ajout
/// interrompu, et ce qui précède est intact. Un premier morceau abîmé, lui, fait échouer.
pub fn lire(octets: &[u8]) -> Result<Lu, CoreError> {
    let mut auteurs = Auteurs::default();
    // Les opérations d'un document portent leurs successeurs, celles d'un delta leurs
    // prédécesseurs : on les garde séparées jusqu'à la reconstruction, qui ramène tout au
    // même sens.
    let mut document: Vec<Op> = Vec::new();
    let mut deltas: Vec<Op> = Vec::new();
    let mut pos = 0;
    let mut sains = 0usize;
    while pos < octets.len() {
        match lire_morceau(&octets[pos..], &mut auteurs) {
            Ok((lus, est_un_delta, longueur)) => {
                if est_un_delta {
                    deltas.extend(lus);
                } else {
                    document.extend(lus);
                }
                pos += longueur;
                sains += 1;
            }
            Err(e) if sains == 0 => return Err(e),
            Err(_) => break,
        }
    }
    if sains == 0 {
        return Err(illisible("aucun morceau Automerge dans ce fichier"));
    }
    let valeur = etat::reconstruire(document, deltas, &auteurs);
    Ok(Lu {
        valeur,
        fin_ignoree: octets.len() - pos,
    })
}

/// Lit le morceau en tête de `octets`. Rend ses opérations, si elles portent des
/// prédécesseurs (un delta), et la longueur totale du morceau.
fn lire_morceau(octets: &[u8], auteurs: &mut Auteurs) -> Result<(Vec<Op>, bool, usize), CoreError> {
    if octets.len() < 9 || octets[..4] != MAGIE {
        return Err(illisible("signature Automerge absente"));
    }
    let somme = &octets[4..8];
    let nature = octets[8];
    let mut c = Curseur::new(&octets[9..]);
    let longueur = c.uleb()?;
    let entete = 9 + c.position();
    let contenu = c.prendre(longueur)?;
    let total = entete + contenu.len();
    match nature {
        MORCEAU_DOCUMENT => {
            verifier(somme, &octets[8..total])?;
            Ok((lire_document(contenu, auteurs)?, false, total))
        }
        MORCEAU_DELTA => {
            verifier(somme, &octets[8..total])?;
            Ok((lire_delta(contenu, auteurs)?, true, total))
        }
        MORCEAU_DELTA_COMPRESSE => {
            let clair = super::inflate::inflate(contenu)
                .map_err(|_| illisible("un delta compressé ne se décompresse pas"))?;
            // La somme d'un delta compressé se calcule comme s'il ne l'était pas.
            let mut tel_quel = vec![MORCEAU_DELTA];
            ecrire_uleb(&mut tel_quel, clair.len() as u64);
            tel_quel.extend_from_slice(&clair);
            verifier(somme, &tel_quel)?;
            Ok((lire_delta(&clair, auteurs)?, true, total))
        }
        autre => Err(illisible(&format!("nature de morceau inconnue ({autre})"))),
    }
}

/// Les quatre premiers octets du SHA-256 de « nature, longueur, contenu ».
fn verifier(somme: &[u8], couvert: &[u8]) -> Result<(), CoreError> {
    if crate::hash::sha256(couvert)[..4] == *somme {
        Ok(())
    } else {
        Err(illisible(
            "somme de contrôle fausse : le fichier a été abîmé",
        ))
    }
}

fn ecrire_uleb(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let octet = (v & 0x7f) as u8;
        v >>= 7;
        if v == 0 {
            out.push(octet);
            return;
        }
        out.push(octet | 0x80);
    }
}

/// Une table d'auteurs : un nombre, puis chaque identifiant préfixé par sa longueur.
fn lire_auteurs(c: &mut Curseur<'_>, auteurs: &mut Auteurs) -> Result<Vec<u32>, CoreError> {
    let n = c.uleb()?;
    let mut rangs = Vec::new();
    for _ in 0..n {
        let longueur = c.uleb()?;
        rangs.push(auteurs.inscrire(c.prendre(longueur)?));
    }
    Ok(rangs)
}

/// Une table de colonnes : un nombre, puis des paires (spécification, longueur).
fn lire_table(c: &mut Curseur<'_>) -> Result<Vec<(u64, u64)>, CoreError> {
    let n = c.uleb()?;
    let mut table = Vec::new();
    for _ in 0..n {
        table.push((c.uleb()?, c.uleb()?));
    }
    Ok(table)
}

fn lire_document(contenu: &[u8], auteurs: &mut Auteurs) -> Result<Vec<Op>, CoreError> {
    let mut c = Curseur::new(contenu);
    let rangs = lire_auteurs(&mut c, auteurs)?;
    let tetes = c.uleb()?;
    c.prendre(tetes.saturating_mul(32))?;
    let table_changements = lire_table(&mut c)?;
    let table_ops = lire_table(&mut c)?;
    // Les colonnes des changements ne disent rien de l'état : on les enjambe.
    let longueur_changements: u64 = table_changements.iter().map(|(_, l)| l).sum();
    c.prendre(longueur_changements)?;
    let longueur_ops: u64 = table_ops.iter().map(|(_, l)| l).sum();
    let donnees = c.prendre(longueur_ops)?;
    let colonnes = Colonnes::decouper(&table_ops, donnees, true)?;
    operations::lire(&colonnes, &Forme::Document, &rangs)
}

fn lire_delta(contenu: &[u8], auteurs: &mut Auteurs) -> Result<Vec<Op>, CoreError> {
    let mut c = Curseur::new(contenu);
    let dependances = c.uleb()?;
    c.prendre(dependances.saturating_mul(32))?;
    let longueur = c.uleb()?;
    let auteur = auteurs.inscrire(c.prendre(longueur)?);
    let _sequence = c.uleb()?;
    let start_op = c.uleb()?;
    let _instant = c.leb()?;
    let message = c.uleb()?;
    c.prendre(message)?;
    let mut rangs = vec![auteur];
    rangs.extend(lire_auteurs(&mut c, auteurs)?);
    let table = lire_table(&mut c)?;
    let longueur_ops: u64 = table.iter().map(|(_, l)| l).sum();
    let donnees = c.prendre(longueur_ops)?;
    let colonnes = Colonnes::decouper(&table, donnees, false)?;
    operations::lire(&colonnes, &Forme::Delta { start_op }, &rangs)
}
