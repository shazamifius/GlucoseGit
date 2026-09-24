//! DEFLATE, en lecture seule (RFC 1951) — pour les colonnes compressées d'Automerge.
//!
//! # Pourquoi l'écrire
//!
//! Le noyau n'a aucune dépendance, et c'est une règle qu'il tient depuis le début. Lire un
//! flux DEFLATE demande trois choses que la RFC spécifie entièrement : lire des bits du
//! poids faible au poids fort, décoder des codes de Huffman canoniques, et recopier ce qui a
//! déjà été produit. La bibliothèque `miniz_oxide`, que l'application tire déjà pour PNG,
//! sert d'**oracle** dans les épreuves : ce qu'elle compresse, ce module doit le rendre à
//! l'octet près, à chaque niveau de compression.
//!
//! La méthode de décodage est celle de `puff` (Mark Adler, l'un des auteurs de zlib) : un code
//! canonique se reconnaît bit à bit, longueur après longueur, sans table de 2¹⁵ entrées à
//! construire — la bonne taille pour des colonnes de quelques kilo-octets.
//!
//! # Ce qu'un flux forgé ne peut pas faire
//!
//! Lire hors de l'entrée, recopier avant le début de la sortie, ou boucler : chaque lecture
//! est bornée, chaque distance vérifiée. Il peut seulement se dilater — mille fois au plus, la
//! limite du format lui-même.

use super::illisible;
use crate::error::CoreError;

/// La plus longue longueur de code qu'autorise le format.
const BITS_MAX: usize = 15;

/// Les longueurs 257 à 285 : base, et nombre de bits en plus.
const LONGUEUR_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LONGUEUR_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
/// Les distances 0 à 29 : base, et nombre de bits en plus.
const DISTANCE_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
/// L'ordre dans lequel un bloc dynamique donne les longueurs du code des longueurs.
const ORDRE_DES_LONGUEURS: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// Décompresse un flux DEFLATE brut (sans en-tête zlib).
pub fn inflate(entree: &[u8]) -> Result<Vec<u8>, CoreError> {
    let mut b = Bits {
        o: entree,
        pos: 0,
        acc: 0,
        n: 0,
    };
    let mut sortie = Vec::with_capacity(entree.len() * 4);
    loop {
        let dernier = b.lire(1)? == 1;
        match b.lire(2)? {
            0 => brut(&mut b, &mut sortie)?,
            1 => {
                let (lettres, distances) = codes_fixes()?;
                blocs(&mut b, &mut sortie, &lettres, &distances)?
            }
            2 => {
                let (lettres, distances) = codes_dynamiques(&mut b)?;
                blocs(&mut b, &mut sortie, &lettres, &distances)?
            }
            _ => return Err(illisible("bloc DEFLATE de type réservé")),
        }
        if dernier {
            return Ok(sortie);
        }
    }
}

/// Les bits d'un flux, du poids faible au poids fort.
struct Bits<'a> {
    o: &'a [u8],
    pos: usize,
    acc: u32,
    /// Bits en attente dans `acc` — toujours moins de 8 entre deux lectures.
    n: u32,
}

impl Bits<'_> {
    fn lire(&mut self, k: u32) -> Result<u32, CoreError> {
        while self.n < k {
            let octet = *self
                .o
                .get(self.pos)
                .ok_or_else(|| illisible("flux DEFLATE tronqué"))?;
            self.pos += 1;
            self.acc |= u32::from(octet) << self.n;
            self.n += 8;
        }
        let v = self.acc & ((1u32 << k) - 1);
        self.acc >>= k;
        self.n -= k;
        Ok(v)
    }

    /// Abandonne les bits qui restent de l'octet en cours.
    fn aligner(&mut self) {
        self.acc = 0;
        self.n = 0;
    }
}

/// Un code de Huffman canonique : combien de codes de chaque longueur, et les symboles rangés
/// par longueur puis par valeur.
struct Code {
    compte: [u16; BITS_MAX + 1],
    symboles: Vec<u16>,
}

impl Code {
    fn depuis(longueurs: &[u8]) -> Result<Self, CoreError> {
        let mut compte = [0u16; BITS_MAX + 1];
        for &l in longueurs {
            compte[usize::from(l)] += 1;
        }
        compte[0] = 0;
        // Un code « sur-souscrit » — plus de codes qu'il n'y a de place — n'est pas un code.
        let mut libres: i32 = 1;
        for &c in &compte[1..] {
            libres = (libres << 1) - i32::from(c);
            if libres < 0 {
                return Err(illisible("code de Huffman impossible"));
            }
        }
        let mut debut = [0usize; BITS_MAX + 2];
        for l in 1..=BITS_MAX {
            debut[l + 1] = debut[l] + usize::from(compte[l]);
        }
        let mut symboles = vec![0u16; debut[BITS_MAX + 1]];
        for (symbole, &l) in longueurs.iter().enumerate() {
            if l != 0 {
                symboles[debut[usize::from(l)]] = symbole as u16;
                debut[usize::from(l)] += 1;
            }
        }
        Ok(Self { compte, symboles })
    }

    /// Le symbole suivant, reconnu bit à bit.
    fn decoder(&self, b: &mut Bits<'_>) -> Result<u16, CoreError> {
        let (mut code, mut premier, mut index) = (0i32, 0i32, 0i32);
        for l in 1..=BITS_MAX {
            code |= b.lire(1)? as i32;
            let compte = i32::from(self.compte[l]);
            if code - compte < premier {
                return Ok(self.symboles[(index + code - premier) as usize]);
            }
            index += compte;
            premier = (premier + compte) << 1;
            code <<= 1;
        }
        Err(illisible("code de Huffman inconnu"))
    }
}

/// Un bloc stocké tel quel : longueur, son complément, puis les octets.
fn brut(b: &mut Bits<'_>, sortie: &mut Vec<u8>) -> Result<(), CoreError> {
    b.aligner();
    let tete =
        b.o.get(b.pos..b.pos + 4)
            .ok_or_else(|| illisible("bloc DEFLATE brut tronqué"))?;
    let longueur = u16::from_le_bytes([tete[0], tete[1]]);
    if u16::from_le_bytes([tete[2], tete[3]]) != !longueur {
        return Err(illisible("bloc DEFLATE brut incohérent"));
    }
    b.pos += 4;
    let octets =
        b.o.get(b.pos..b.pos + usize::from(longueur))
            .ok_or_else(|| illisible("bloc DEFLATE brut tronqué"))?;
    sortie.extend_from_slice(octets);
    b.pos += usize::from(longueur);
    Ok(())
}

/// Les codes fixes de la RFC (§ 3.2.6).
fn codes_fixes() -> Result<(Code, Code), CoreError> {
    let mut lettres = [8u8; 288];
    lettres[144..256].fill(9);
    lettres[256..280].fill(7);
    Ok((Code::depuis(&lettres)?, Code::depuis(&[5u8; 30])?))
}

/// Les codes qu'un bloc dynamique décrit lui-même (§ 3.2.7).
fn codes_dynamiques(b: &mut Bits<'_>) -> Result<(Code, Code), CoreError> {
    let nb_lettres = b.lire(5)? as usize + 257;
    let nb_distances = b.lire(5)? as usize + 1;
    let nb_longueurs = b.lire(4)? as usize + 4;
    if nb_lettres > 286 || nb_distances > 30 {
        return Err(illisible("bloc DEFLATE dynamique démesuré"));
    }
    let mut l = [0u8; 19];
    for &position in &ORDRE_DES_LONGUEURS[..nb_longueurs] {
        l[position] = b.lire(3)? as u8;
    }
    let code_des_longueurs = Code::depuis(&l)?;
    let total = nb_lettres + nb_distances;
    let mut longueurs: Vec<u8> = Vec::with_capacity(total);
    while longueurs.len() < total {
        let (valeur, fois) = match code_des_longueurs.decoder(b)? {
            s @ 0..=15 => (s as u8, 1),
            16 => {
                let precedente = *longueurs
                    .last()
                    .ok_or_else(|| illisible("répétition sans longueur précédente"))?;
                (precedente, 3 + b.lire(2)? as usize)
            }
            17 => (0, 3 + b.lire(3)? as usize),
            _ => (0, 11 + b.lire(7)? as usize),
        };
        if longueurs.len() + fois > total {
            return Err(illisible("longueurs de code en trop"));
        }
        longueurs.extend(std::iter::repeat_n(valeur, fois));
    }
    if longueurs[256] == 0 {
        return Err(illisible("bloc DEFLATE sans code de fin"));
    }
    Ok((
        Code::depuis(&longueurs[..nb_lettres])?,
        Code::depuis(&longueurs[nb_lettres..])?,
    ))
}

/// Les symboles d'un bloc compressé : des lettres, des recopies, puis la fin.
fn blocs(
    b: &mut Bits<'_>,
    sortie: &mut Vec<u8>,
    lettres: &Code,
    distances: &Code,
) -> Result<(), CoreError> {
    loop {
        let symbole = lettres.decoder(b)?;
        match symbole {
            0..=255 => sortie.push(symbole as u8),
            256 => return Ok(()),
            _ => {
                let s = usize::from(symbole - 257);
                if s >= LONGUEUR_BASE.len() {
                    return Err(illisible("longueur de recopie inconnue"));
                }
                let longueur =
                    usize::from(LONGUEUR_BASE[s]) + b.lire(u32::from(LONGUEUR_EXTRA[s]))? as usize;
                let d = usize::from(distances.decoder(b)?);
                if d >= DISTANCE_BASE.len() {
                    return Err(illisible("distance de recopie inconnue"));
                }
                let distance =
                    usize::from(DISTANCE_BASE[d]) + b.lire(u32::from(DISTANCE_EXTRA[d]))? as usize;
                if distance > sortie.len() {
                    return Err(illisible("recopie avant le début du flux"));
                }
                // Octet par octet : une recopie peut chevaucher ce qu'elle produit (« aaaa »
                // s'écrit « a », puis « recopie 3 depuis 1 en arrière »).
                let depart = sortie.len() - distance;
                for k in 0..longueur {
                    let octet = sortie[depart + k];
                    sortie.push(octet);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_un_bloc_brut_se_rend_tel_quel() {
        // BFINAL=1, BTYPE=00, puis LEN=5, NLEN=!5, puis « salut ».
        let flux = [0x01, 0x05, 0x00, 0xfa, 0xff, b's', b'a', b'l', b'u', b't'];
        assert_eq!(inflate(&flux).unwrap(), b"salut");
    }

    #[test]
    fn test_un_code_fixe_et_une_recopie_chevauchante() {
        // zlib, niveau 9, de « aaaaaaaaaa », sans en-tête : une lettre puis une recopie de 9
        // à distance 1 — la recopie chevauche ce qu'elle produit.
        let flux = [0x4b, 0x4c, 0x84, 0x01, 0x00];
        assert_eq!(inflate(&flux).unwrap(), b"aaaaaaaaaa");
    }

    #[test]
    fn test_un_flux_tronque_ou_forge_est_refuse_sans_paniquer() {
        assert!(inflate(&[]).is_err());
        assert!(inflate(&[0x07]).is_err(), "type de bloc réservé");
        assert!(
            inflate(&[0x01, 0x05, 0x00, 0x00, 0x00]).is_err(),
            "NLEN faux"
        );
        assert!(inflate(&[0x4b, 0x4c, 0x84]).is_err(), "coupé au milieu");
    }
}
