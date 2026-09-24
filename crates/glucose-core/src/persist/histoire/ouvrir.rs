//! Ouvrir un document : sa base, puis son histoire, **sans lire ses images**.
//!
//! Un document de 180 Mo s'ouvre en lisant son en-tête, sa table, son manifeste, son
//! document, puis les en-têtes de son histoire et ses entrées légères. Les octets des images
//! sont enjambés : on retient seulement **où** ils sont, et l'empreinte qu'ils annoncent. Ils
//! se lisent — et se vérifient — le jour où l'on en a besoin.
//!
//! # L'état rejoué
//!
//! L'état s'obtient depuis le **dernier instantané** (ou la base), en appliquant dans l'ordre
//! les gestes et les vues qui le suivent. Un geste qui ne s'applique pas — ce qui voudrait
//! dire qu'une édition a contourné le journal — arrête le rejeu : le document s'ouvre dans le
//! dernier état cohérent, et [`Ouvert::geste_en_echec`] le dit.

use super::super::container::{
    CONTAINER_VERSION, ENTRY_LEN, HEADER_LEN, KIND_ASSET, KIND_DOCUMENT, KIND_MANIFEST, MAGIC,
    MIN_READABLE_VERSION,
};
use super::super::manifest::{self, Manifest};
use super::{lire_geste, lire_instantane, lire_jalon, lire_lien, nature, suivante, vue};
use super::{Chaine, Jalon, Vue, ENTETE};
use crate::error::{CoreError, CoreResult};
use crate::hash::sha256;
use crate::persist::bytes::Reader;
use crate::types::Project;
use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};

/// Où sont les octets d'une image dans le fichier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tranche {
    pub offset: u64,
    pub longueur: u64,
}

/// Un geste de l'histoire, repéré sans être relu : de quoi le retrouver et le dater.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Repere {
    /// Position de son contenu dans le fichier, et longueur.
    pub tranche: Tranche,
    pub instant: i64,
}

/// Un instantané de l'histoire : après combien de gestes, et où.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instantane {
    pub apres: usize,
    pub tranche: Tranche,
}

/// Ce qu'un fichier ouvert a rendu.
#[derive(Debug, Clone)]
pub struct Ouvert {
    /// L'état du document : la base ou le dernier instantané, et ce qui les suit.
    pub projet: Project,
    pub manifeste: Manifest,
    /// Les octets de chaque image, par empreinte.
    pub objets: HashMap<[u8; 32], Tranche>,
    /// Chaque clé d'image du document, liée à l'empreinte de ses octets.
    pub liens: HashMap<String, [u8; 32]>,
    /// Tous les gestes, dans l'ordre : c'est la réglette de la Time Machine.
    pub gestes: Vec<Repere>,
    /// Les jalons, chacun avec le nombre de gestes qui le précèdent.
    pub jalons: Vec<(usize, Jalon)>,
    pub instantanes: Vec<Instantane>,
    /// Là où la prochaine entrée s'écrira, et la chaîne à y reprendre.
    pub fin: u64,
    pub chaine: Chaine,
    /// Octets de fin qui ne suivaient pas la chaîne — une écriture interrompue. Zéro pour un
    /// fichier intact.
    pub fin_ignoree: u64,
    /// La version de conteneur lue dans l'en-tête : 2 tant que rien n'a été ajouté.
    pub version_du_conteneur: u16,
    /// Octets de gestes écrits depuis le dernier instantané, et taille de celui-ci : de quoi
    /// décider quand en écrire un autre.
    pub octets_depuis_l_instantane: u64,
    pub taille_de_l_instantane: u64,
    /// Le rang du geste qui n'a pas pu se rejouer, s'il y en a un.
    pub geste_en_echec: Option<usize>,
}

/// Lit un document entier depuis un lecteur à accès direct.
pub fn ouvrir<R: Read + Seek>(r: &mut R) -> CoreResult<Ouvert> {
    let taille = r.seek(SeekFrom::End(0)).map_err(io)?;
    let base = lire_la_base(r, taille)?;
    let manifeste = manifest::decode(&lire_section(r, &base.manifeste)?)?;
    super::super::check_document_version(manifeste.document_version)?;
    let document = super::super::decode_document_v(
        &lire_section(r, &base.document)?,
        manifeste.document_version,
    )?;
    let mut ouvert = Ouvert {
        taille_de_l_instantane: base.document.payload.len() as u64,
        projet: document,
        objets: base.objets,
        liens: manifeste
            .assets
            .iter()
            .map(|a| (a.key.clone(), a.digest))
            .collect(),
        manifeste,
        gestes: Vec::new(),
        jalons: Vec::new(),
        instantanes: Vec::new(),
        fin: base.fin,
        chaine: base.graine,
        fin_ignoree: 0,
        version_du_conteneur: base.version,
        octets_depuis_l_instantane: 0,
        geste_en_echec: None,
    };
    let a_rejouer = lire_la_queue(r, taille, &mut ouvert)?;
    rejouer(&mut ouvert, a_rejouer);
    Ok(ouvert)
}

fn io(e: std::io::Error) -> CoreError {
    CoreError::IoError(e.to_string())
}

/// Une section de la base, repérée : sa place et son empreinte, sans son contenu.
struct Section {
    offset: u64,
    longueur: u64,
    empreinte: [u8; 32],
    /// Le contenu, pour les deux sections qu'on lit toujours ; vide pour une image.
    payload: Vec<u8>,
}

struct Base {
    version: u16,
    manifeste: Section,
    document: Section,
    objets: HashMap<[u8; 32], Tranche>,
    fin: u64,
    graine: Chaine,
}

fn lire_la_base<R: Read + Seek>(r: &mut R, taille: u64) -> CoreResult<Base> {
    let mut entete = [0u8; HEADER_LEN];
    r.seek(SeekFrom::Start(0)).map_err(io)?;
    r.read_exact(&mut entete)
        .map_err(|_| super::tronque("l'en-tête"))?;
    let version = verifier_l_entete(&entete)?;
    let n = u32::from_le_bytes([entete[12], entete[13], entete[14], entete[15]]) as u64;
    let longueur_table = n
        .checked_mul(ENTRY_LEN as u64)
        .filter(|l| HEADER_LEN as u64 + l <= taille)
        .ok_or_else(|| super::tronque("la table des sections"))?;
    let mut prefixe = entete.to_vec();
    prefixe.resize(HEADER_LEN + longueur_table as usize, 0);
    r.read_exact(&mut prefixe[HEADER_LEN..])
        .map_err(|_| super::tronque("la table des sections"))?;
    let graine = Chaine::de_la_base(&prefixe)?;
    let mut sections = Vec::with_capacity(n as usize);
    let mut fin = prefixe.len() as u64;
    for i in 0..n as usize {
        let e = &prefixe[HEADER_LEN + i * ENTRY_LEN..HEADER_LEN + (i + 1) * ENTRY_LEN];
        let offset = u64::from_le_bytes(e[8..16].try_into().unwrap_or([0; 8]));
        let longueur = u64::from_le_bytes(e[16..24].try_into().unwrap_or([0; 8]));
        let bout = offset
            .checked_add(longueur)
            .filter(|b| *b <= taille)
            .ok_or_else(|| super::tronque("un contenu de section"))?;
        fin = fin.max(bout);
        let mut empreinte = [0u8; 32];
        empreinte.copy_from_slice(&e[24..56]);
        sections.push((e[0], offset, longueur, empreinte));
    }
    let prendre = |nature: u8, nom: &str| -> CoreResult<Section> {
        let (_, offset, longueur, empreinte) =
            *sections.iter().find(|s| s.0 == nature).ok_or_else(|| {
                CoreError::DeserializationError(format!(
                    "{nom} est absent du fichier .glucose — le projet est incomplet, rouvre la \
                     copie précédente"
                ))
            })?;
        Ok(Section {
            offset,
            longueur,
            empreinte,
            payload: Vec::new(),
        })
    };
    let objets = sections
        .iter()
        .filter(|s| s.0 == KIND_ASSET)
        .map(|&(_, offset, longueur, empreinte)| (empreinte, Tranche { offset, longueur }))
        .collect();
    let mut base = Base {
        version,
        manifeste: prendre(KIND_MANIFEST, "le manifeste")?,
        document: prendre(KIND_DOCUMENT, "le document")?,
        objets,
        fin,
        graine,
    };
    base.document.payload = lire_section(r, &base.document)?;
    Ok(base)
}

/// Les seize premiers octets : la signature, les fanions, et une version que cette build sait
/// lire. Rend la version.
fn verifier_l_entete(entete: &[u8; HEADER_LEN]) -> CoreResult<u16> {
    if entete[0..8] != MAGIC {
        return Err(CoreError::DeserializationError(
            "ce fichier n'est pas un projet Glucose : sa signature ne correspond pas — \
             vérifie que tu ouvres bien un fichier .glucose"
                .to_string(),
        ));
    }
    if entete[10..12] != [0, 0] {
        return Err(CoreError::DeserializationError(
            "fanions d'en-tête inconnus : ce projet utilise une extension du format que cette \
             version ignore — mets Glucose à jour"
                .to_string(),
        ));
    }
    let version = u16::from_le_bytes([entete[8], entete[9]]);
    if !(MIN_READABLE_VERSION..=CONTAINER_VERSION).contains(&version) {
        return Err(CoreError::DeserializationError(format!(
            "projet écrit au format .glucose v{version}, cette version de Glucose lit de la \
             v{MIN_READABLE_VERSION} à la v{CONTAINER_VERSION} — mets Glucose à jour pour l'ouvrir"
        )));
    }
    Ok(version)
}

/// Le contenu d'une section, vérifié contre son empreinte.
fn lire_section<R: Read + Seek>(r: &mut R, s: &Section) -> CoreResult<Vec<u8>> {
    if !s.payload.is_empty() {
        return Ok(s.payload.clone());
    }
    let mut contenu = vec![0u8; s.longueur as usize];
    r.seek(SeekFrom::Start(s.offset)).map_err(io)?;
    r.read_exact(&mut contenu)
        .map_err(|_| super::tronque("un contenu de section"))?;
    if sha256(&contenu) != s.empreinte {
        return Err(CoreError::DeserializationError(
            "somme de contrôle fausse dans la base : le fichier a été altéré — rouvre la \
             copie précédente du projet"
                .to_string(),
        ));
    }
    Ok(contenu)
}

/// Ce qui reste à rejouer après le dernier instantané : les gestes et les vues, dans l'ordre.
enum ARejouer {
    Geste(usize, Vec<u8>),
    Vue(Vue),
}

/// Suit la chaîne jusqu'à sa fin. Rend ce qu'il faudra rejouer depuis le dernier instantané.
fn lire_la_queue<R: Read + Seek>(
    r: &mut R,
    taille: u64,
    o: &mut Ouvert,
) -> CoreResult<Vec<ARejouer>> {
    let mut a_rejouer = Vec::new();
    let mut pos = o.fin;
    r.seek(SeekFrom::Start(pos)).map_err(io)?;
    while let Some((nature, contenu, longueur)) = entree_suivante(r, taille, pos, &mut o.chaine)? {
        let debut = pos + ENTETE as u64;
        let tranche = Tranche {
            offset: debut,
            longueur: longueur as u64,
        };
        ranger(o, &mut a_rejouer, nature, contenu, tranche)?;
        pos = debut + longueur as u64;
    }
    o.fin_ignoree = taille - pos;
    o.fin = pos;
    Ok(a_rejouer)
}

/// L'entrée qui commence à `pos`, si elle suit la chaîne. Rend sa nature, son contenu (sauf
/// pour un objet, dont seule l'empreinte est lue) et sa longueur.
fn entree_suivante<R: Read + Seek>(
    r: &mut R,
    taille: u64,
    pos: u64,
    chaine: &mut Chaine,
) -> CoreResult<Option<(u8, Vec<u8>, u32)>> {
    if pos + ENTETE as u64 > taille {
        return Ok(None);
    }
    let mut e = [0u8; ENTETE];
    r.seek(SeekFrom::Start(pos)).map_err(io)?;
    r.read_exact(&mut e).map_err(io)?;
    let longueur = u32::from_le_bytes([e[4], e[5], e[6], e[7]]);
    if e[1..4] != [0, 0, 0] || pos + ENTETE as u64 + u64::from(longueur) > taille {
        return Ok(None);
    }
    let (contenu, h) = if e[0] == nature::OBJET {
        if longueur < 32 {
            return Ok(None);
        }
        let mut empreinte = [0u8; 32];
        r.read_exact(&mut empreinte).map_err(io)?;
        (empreinte.to_vec(), empreinte)
    } else {
        let mut contenu = vec![0u8; longueur as usize];
        r.read_exact(&mut contenu).map_err(io)?;
        let h = sha256(&contenu);
        (contenu, h)
    };
    let attendue = suivante(chaine.0, e[0], longueur, &h);
    if e[8..16] != attendue {
        return Ok(None);
    }
    chaine.0 = attendue;
    Ok(Some((e[0], contenu, longueur)))
}

/// Range une entrée lue et vérifiée.
fn ranger(
    o: &mut Ouvert,
    a_rejouer: &mut Vec<ARejouer>,
    n: u8,
    contenu: Vec<u8>,
    tranche: Tranche,
) -> CoreResult<()> {
    match n {
        nature::OBJET => {
            let mut empreinte = [0u8; 32];
            empreinte.copy_from_slice(&contenu);
            o.objets.insert(
                empreinte,
                Tranche {
                    offset: tranche.offset + 32,
                    longueur: tranche.longueur - 32,
                },
            );
        }
        nature::GESTE => {
            let instant = contenu
                .get(2..10)
                .map(|c| i64::from_le_bytes(c.try_into().unwrap_or([0; 8])))
                .unwrap_or(0);
            o.gestes.push(Repere { tranche, instant });
            o.octets_depuis_l_instantane += tranche.longueur;
            a_rejouer.push(ARejouer::Geste(o.gestes.len() - 1, contenu));
        }
        nature::INSTANTANE => {
            o.instantanes.push(Instantane {
                apres: o.gestes.len(),
                tranche,
            });
            o.projet = lire_instantane(&contenu)?;
            o.taille_de_l_instantane = tranche.longueur;
            o.octets_depuis_l_instantane = 0;
            a_rejouer.clear();
        }
        nature::JALON => {
            let jalon = lire_jalon(&mut Reader::new(&contenu))?;
            o.jalons.push((o.gestes.len(), jalon));
        }
        nature::LIEN => {
            let (cle, empreinte) = lire_lien(&mut Reader::new(&contenu))?;
            o.liens.insert(cle, empreinte);
        }
        nature::VUE => a_rejouer.push(ARejouer::Vue(vue::lire(&mut Reader::new(&contenu))?)),
        autre => {
            return Err(CoreError::DeserializationError(format!(
                "entrée d'histoire de nature {autre} : ce document vient d'une version plus \
                 récente de Glucose, mets-la à jour pour l'ouvrir"
            )))
        }
    }
    Ok(())
}

/// Applique, dans l'ordre, ce qui suit le dernier instantané.
fn rejouer(o: &mut Ouvert, a_rejouer: Vec<ARejouer>) {
    for entree in a_rejouer {
        match entree {
            ARejouer::Vue(v) => v.poser(&mut o.projet),
            ARejouer::Geste(rang, contenu) => {
                let applique = lire_geste(&contenu)
                    .map(|g| g.transaction.apply(&mut o.projet))
                    .unwrap_or(false);
                if !applique {
                    o.geste_en_echec = Some(rang);
                    return;
                }
            }
        }
    }
}
