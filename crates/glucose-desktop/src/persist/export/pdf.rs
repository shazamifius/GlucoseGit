//! **Le PDF des planches, écrit à la main** (P2, proposition 7, fiche 68) — sans dépendance.
//!
//! # Une page, une image, aucun pixel recompressé
//!
//! La méthode d'img2pdf (`src/img2pdf.py`, `parse_png`) : un PNG en rouge, vert, bleu sur huit
//! bits, non entrelacé, est un flux zlib découpé en blocs `IDAT`, dont chaque rangée commence
//! par l'octet de son filtre. Mis bout à bout, ces blocs sont exactement ce qu'un PDF décode
//! par `/FlateDecode` sous `/DecodeParms << /Predictor 15 … >>` — les prédicteurs du PNG, que
//! la norme reprend (PDF 1.4, § 3.3.3). La planche s'encode donc une fois, par l'écrivain de
//! l'export en image, et le PDF ne fait que poser ses octets dans une page.
//!
//! # La taille d'une page
//!
//! Celle de la planche : un pixel vaut un pixel CSS, 1/96 de pouce, soit trois quarts de point —
//! ce que « 100 % » dit à l'écran. La planche a la proportion d'une feuille A à l'italienne
//! (`Disposition::planches`) : à l'impression, elle la remplit.
//!
//! # Ce qu'un lecteur doit trouver
//!
//! Un en-tête, des objets numérotés, la table de leurs positions (`xref`), dont chaque ligne
//! fait vingt octets, et la position de la table à la fin. Aucune compression des objets, aucun
//! flux d'objets : la forme que tous les lecteurs lisent depuis 2001.

use std::io::{self, Write};

/// **Une planche prête à poser** : sa taille en pixels, et les blocs `IDAT` de son PNG, mis
/// bout à bout.
#[derive(Debug, Clone, PartialEq)]
pub struct Planche {
    pub largeur: u32,
    pub hauteur: u32,
    pub idat: Vec<u8>,
}

/// La signature d'un PNG.
const SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

/// **Les blocs `IDAT` d'un PNG**, mis bout à bout — `None` si le PNG n'est pas de la seule forme
/// qu'un PDF décode tel quel : huit bits, rouge vert bleu, sans entrelacement.
pub fn planche(png: &[u8]) -> Option<Planche> {
    let mut reste = png.strip_prefix(&SIGNATURE)?;
    let (mut taille, mut idat) = (None, Vec::new());
    while reste.len() >= 12 {
        let n = u32::from_be_bytes(reste[0..4].try_into().ok()?) as usize;
        let donnees = reste.get(8..8 + n)?;
        match &reste[4..8] {
            b"IHDR" => {
                let mot = |i: usize| donnees.get(i..i + 4).map(|m| [m[0], m[1], m[2], m[3]]);
                // Profondeur 8, couleur 2 (RVB), compression 0, filtre 0, entrelacement 0.
                if donnees.get(8..13) != Some(&[8, 2, 0, 0, 0][..]) {
                    return None;
                }
                taille = Some((u32::from_be_bytes(mot(0)?), u32::from_be_bytes(mot(4)?)));
            }
            b"IDAT" => idat.extend_from_slice(donnees),
            b"IEND" => break,
            _ => {}
        }
        reste = reste.get(12 + n..)?;
    }
    let (largeur, hauteur) = taille?;
    Some(Planche {
        largeur,
        hauteur,
        idat,
    })
}

/// Une sortie qui compte ce qu'on y écrit : la table des objets dit leur position.
struct Compteur<'a, W: Write> {
    sortie: &'a mut W,
    ecrits: u64,
}

impl<W: Write> Compteur<'_, W> {
    fn ecrire(&mut self, octets: &[u8]) -> io::Result<()> {
        self.sortie.write_all(octets)?;
        self.ecrits += octets.len() as u64;
        Ok(())
    }

    /// L'objet `numero`, son dictionnaire, et son flux s'il en a un ; sa position est notée.
    fn objet(
        &mut self,
        positions: &mut Vec<u64>,
        (numero, dictionnaire): (usize, &str),
        flux: Option<&[u8]>,
    ) -> io::Result<()> {
        debug_assert_eq!(numero, positions.len() + 1, "les objets dans l'ordre");
        positions.push(self.ecrits);
        match flux {
            None => self.ecrire(format!("{numero} 0 obj\n{dictionnaire}\nendobj\n").as_bytes()),
            Some(octets) => {
                let tete = format!(
                    "{numero} 0 obj\n<< {dictionnaire} /Length {} >>\nstream\n",
                    octets.len()
                );
                self.ecrire(tete.as_bytes())?;
                self.ecrire(octets)?;
                self.ecrire(b"\nendstream\nendobj\n")
            }
        }
    }
}

/// Une longueur en pixels, en points : trois quarts.
fn points(pixels: u32) -> f64 {
    f64::from(pixels) * 0.75
}

/// **Écrit le PDF** : une page par planche, dans l'ordre. Rend le nombre d'octets écrits.
///
/// Les objets : 1 le catalogue, 2 l'arbre des pages, puis pour la planche `k` sa page
/// (`3 + 3k`), son dessin (`4 + 3k`) et son image (`5 + 3k`).
pub fn ecrire(sortie: &mut impl Write, planches: &[Planche]) -> io::Result<u64> {
    let mut w = Compteur { sortie, ecrits: 0 };
    let mut positions = Vec::new();
    // Quatre octets au-delà de 127 : un lecteur sait le fichier binaire (PDF 1.4, § 3.4.1).
    w.ecrire(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n")?;
    w.objet(
        &mut positions,
        (1, "<< /Type /Catalog /Pages 2 0 R >>"),
        None,
    )?;
    let pages: Vec<String> = (0..planches.len())
        .map(|k| format!("{} 0 R", 3 + 3 * k))
        .collect();
    let arbre = format!(
        "<< /Type /Pages /Kids [{}] /Count {} >>",
        pages.join(" "),
        planches.len()
    );
    w.objet(&mut positions, (2, &arbre), None)?;
    for (k, p) in planches.iter().enumerate() {
        let (page, dessin, image) = (3 + 3 * k, 4 + 3 * k, 5 + 3 * k);
        let (pl, ph) = (points(p.largeur), points(p.hauteur));
        let dictionnaire = format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {pl} {ph}] \
             /Resources << /XObject << /Im {image} 0 R >> >> /Contents {dessin} 0 R >>"
        );
        w.objet(&mut positions, (page, &dictionnaire), None)?;
        let trace = format!("q {pl} 0 0 {ph} 0 0 cm /Im Do Q");
        w.objet(&mut positions, (dessin, ""), Some(trace.as_bytes()))?;
        let decrite = format!(
            "/Type /XObject /Subtype /Image /Width {l} /Height {h} /ColorSpace /DeviceRGB \
             /BitsPerComponent 8 /Filter /FlateDecode \
             /DecodeParms << /Predictor 15 /Colors 3 /BitsPerComponent 8 /Columns {l} >>",
            l = p.largeur,
            h = p.hauteur,
        );
        w.objet(&mut positions, (image, &decrite), Some(&p.idat))?;
    }
    let table = w.ecrits;
    let mut xref = format!("xref\n0 {}\n0000000000 65535 f \n", positions.len() + 1);
    for position in &positions {
        xref.push_str(&format!("{position:010} 00000 n \n"));
    }
    w.ecrire(xref.as_bytes())?;
    let fin = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{table}\n%%EOF\n",
        positions.len() + 1
    );
    w.ecrire(fin.as_bytes())?;
    Ok(w.ecrits)
}

#[cfg(test)]
mod tests;
