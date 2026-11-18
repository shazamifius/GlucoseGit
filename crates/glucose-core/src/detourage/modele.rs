//! **Le modèle de détourage, tel que `outils/detourage/convertir.py` l'écrit** (DETOURER-1).
//!
//! Une liste d'opérations aux tailles connues ; le format est dit en tête du script. La lecture
//! **propage les formes** du tenseur d'entrée jusqu'au masque et refuse tout ce qui ne s'enchaîne
//! pas — un canal de trop, un tenseur lu avant d'être écrit, un octet en trop : un fichier abîmé se
//! refuse ici, jamais au milieu d'un calcul.

/// Ce qu'une opération fait, aux tailles figées par la conversion.
#[derive(Debug, Clone)]
pub(super) enum Sorte {
    Conv(Conv),
    MaxPool {
        noyau: usize,
        pas: usize,
        plafond: bool,
    },
    Agrandir {
        hauteur: usize,
        largeur: usize,
    },
    Concat,
    Ajouter,
    Sigmoide,
}

/// Une convolution : ses poids rangés `[sortie][entrée][ky][kx]`, son biais.
#[derive(Debug, Clone)]
pub(super) struct Conv {
    pub cin: usize,
    pub cout: usize,
    pub kh: usize,
    pub kl: usize,
    pub pas: usize,
    pub dilatation: usize,
    pub remplissage: usize,
    pub poids: Vec<f32>,
    pub biais: Vec<f32>,
}

impl Conv {
    /// La taille de sortie le long d'un axe d'entrée `n` (la formule d'ONNX et de PyTorch).
    pub fn sortie(&self, n: usize, k: usize) -> Option<usize> {
        let utile = (n + 2 * self.remplissage).checked_sub(self.dilatation * (k - 1) + 1)?;
        Some(utile / self.pas + 1)
    }
}

#[derive(Debug, Clone)]
pub(super) struct Operation {
    pub sorte: Sorte,
    /// Le ReLU qui suivait l'opération dans le graphe d'origine.
    pub relu: bool,
    pub entrees: Vec<usize>,
    pub sortie: usize,
}

/// **Un modèle de détourage lu et vérifié.**
#[derive(Debug, Clone)]
pub struct Modele {
    /// L'image d'entrée est `taille × taille`.
    pub taille: usize,
    pub(super) moyenne: [f32; 3],
    pub(super) ecart: [f32; 3],
    pub(super) tenseurs: usize,
    pub(super) sortie: usize,
    pub(super) operations: Vec<Operation>,
    /// Pour chaque tenseur, la dernière opération qui le lit : passé elle, sa mémoire se rend.
    pub(super) derniere_lecture: Vec<usize>,
}

const SIGNATURE: &[u8] = b"GLUCDET1";

struct Lecteur<'a> {
    octets: &'a [u8],
    pos: usize,
}

impl Lecteur<'_> {
    fn prendre(&mut self, n: usize) -> Result<&[u8], String> {
        let fin = self.pos.checked_add(n).filter(|&f| f <= self.octets.len());
        let fin = fin.ok_or("le modèle s'arrête trop tôt")?;
        let tranche = &self.octets[self.pos..fin];
        self.pos = fin;
        Ok(tranche)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.prendre(1)?[0])
    }
    fn u16(&mut self) -> Result<usize, String> {
        let b = self.prendre(2)?;
        Ok(usize::from(u16::from_le_bytes([b[0], b[1]])))
    }
    fn u32(&mut self) -> Result<usize, String> {
        let b = self.prendre(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
    }
    fn f32s(&mut self, n: usize) -> Result<Vec<f32>, String> {
        let octets = self.prendre(n.checked_mul(4).ok_or("taille impossible")?)?;
        Ok(octets
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect())
    }
}

impl Modele {
    /// Lit un modèle et vérifie qu'il s'enchaîne, de l'image au masque.
    pub fn lire(octets: &[u8]) -> Result<Self, String> {
        let mut lu = Lecteur { octets, pos: 0 };
        if lu.prendre(8)? != SIGNATURE {
            return Err("ce n'est pas un modèle de détourage (GLUCDET1)".into());
        }
        let (taille, tenseurs, n, sortie) = (lu.u32()?, lu.u32()?, lu.u32()?, lu.u32()?);
        let norme = lu.f32s(6)?;
        let mut operations = Vec::with_capacity(n.min(4096));
        for _ in 0..n {
            operations.push(lire_une_operation(&mut lu)?);
        }
        if lu.pos != octets.len() {
            return Err("des octets suivent la dernière opération".into());
        }
        let modele = Self {
            taille,
            moyenne: [norme[0], norme[1], norme[2]],
            ecart: [norme[3], norme[4], norme[5]],
            tenseurs,
            sortie,
            derniere_lecture: Vec::new(),
            operations,
        };
        modele.verifie()
    }

    /// Les formes, de l'entrée au masque : chaque tenseur écrit une fois, lu après l'avoir été.
    fn verifie(mut self) -> Result<Self, String> {
        if self.taille == 0 || self.taille > 4096 || self.ecart.contains(&0.0) {
            return Err("taille ou normalisation impossible".into());
        }
        let mut formes: Vec<Option<[usize; 3]>> = vec![None; self.tenseurs];
        *formes.first_mut().ok_or("aucun tenseur")? = Some([3, self.taille, self.taille]);
        let mut derniere = vec![0; self.tenseurs];
        for (rang, op) in self.operations.iter().enumerate() {
            let lues = op
                .entrees
                .iter()
                .map(|&e| {
                    if let Some(d) = derniere.get_mut(e) {
                        *d = rang;
                    }
                    formes
                        .get(e)
                        .copied()
                        .flatten()
                        .ok_or("tenseur lu avant d'être écrit")
                })
                .collect::<Result<Vec<_>, _>>()?;
            let forme = forme_de_sortie(&op.sorte, &lues)?;
            let place = formes.get_mut(op.sortie).ok_or("tenseur hors du modèle")?;
            if place.is_some() || op.sortie == 0 {
                return Err("tenseur écrit deux fois".into());
            }
            *place = Some(forme);
        }
        match formes.get(self.sortie).copied().flatten() {
            Some([1, h, l]) if h == self.taille && l == self.taille => {}
            _ => return Err("le masque n'a pas la forme de l'image".into()),
        }
        self.derniere_lecture = derniere;
        Ok(self)
    }
}

fn lire_une_operation(lu: &mut Lecteur) -> Result<Operation, String> {
    let (code, relu, n, sortie) = (lu.u8()?, lu.u8()? == 1, lu.u16()?, lu.u32()?);
    let entrees = (0..n).map(|_| lu.u32()).collect::<Result<Vec<_>, _>>()?;
    let sorte = match code {
        1 => {
            let [cin, cout, kh, kl, pas, dilatation, remplissage] = [
                lu.u32()?,
                lu.u32()?,
                lu.u32()?,
                lu.u32()?,
                lu.u32()?,
                lu.u32()?,
                lu.u32()?,
            ];
            let n = cout
                .checked_mul(cin)
                .and_then(|n| n.checked_mul(kh)?.checked_mul(kl))
                .ok_or("convolution démesurée")?;
            let (poids, biais) = (lu.f32s(n)?, lu.f32s(cout)?);
            Sorte::Conv(Conv {
                cin,
                cout,
                kh,
                kl,
                pas,
                dilatation,
                remplissage,
                poids,
                biais,
            })
        }
        2 => {
            let (noyau, pas, plafond) = (lu.u32()?, lu.u32()?, lu.u32()? == 1);
            Sorte::MaxPool {
                noyau,
                pas,
                plafond,
            }
        }
        3 => {
            let (hauteur, largeur) = (lu.u32()?, lu.u32()?);
            // Le repère (« half_pixel » ou celui de PyTorch) ne diffère que pour une sortie d'un
            // seul pixel, que la lecture des formes refuse : les deux se calculent pareil.
            lu.u8()?;
            Sorte::Agrandir { hauteur, largeur }
        }
        4 => Sorte::Concat,
        5 => Sorte::Ajouter,
        6 => Sorte::Sigmoide,
        _ => return Err(format!("opération inconnue ({code})")),
    };
    Ok(Operation {
        sorte,
        relu,
        entrees,
        sortie,
    })
}

/// La forme `[canaux, hauteur, largeur]` de ce qu'une opération écrit, ou pourquoi elle ne
/// s'enchaîne pas.
fn forme_de_sortie(sorte: &Sorte, lues: &[[usize; 3]]) -> Result<[usize; 3], String> {
    let premiere = *lues.first().ok_or("opération sans entrée")?;
    let [c, h, l] = premiere;
    let une_seule = |f: [usize; 3]| {
        if lues.len() == 1 {
            Ok(f)
        } else {
            Err("entrées en trop")
        }
    };
    Ok(match sorte {
        Sorte::Conv(cv) => {
            let bonne = cv.cin == c && cv.pas > 0 && cv.dilatation > 0 && cv.kh > 0 && cv.kl > 0;
            let ho = cv.sortie(h, cv.kh).filter(|_| bonne);
            let lo = cv.sortie(l, cv.kl);
            une_seule([
                cv.cout,
                ho.ok_or("convolution impossible")?,
                lo.ok_or("convolution impossible")?,
            ])?
        }
        Sorte::MaxPool {
            noyau,
            pas,
            plafond,
        } => {
            if *noyau == 0 || *pas == 0 || h < *noyau || l < *noyau {
                return Err("regroupement impossible".into());
            }
            let n = |v: usize| {
                if *plafond {
                    (v - noyau).div_ceil(*pas) + 1
                } else {
                    (v - noyau) / pas + 1
                }
            };
            une_seule([c, n(h), n(l)])?
        }
        Sorte::Agrandir { hauteur, largeur } => {
            if *hauteur < 2 || *largeur < 2 {
                return Err("agrandissement impossible".into());
            }
            une_seule([c, *hauteur, *largeur])?
        }
        Sorte::Concat => {
            if lues.iter().any(|f| f[1] != h || f[2] != l) {
                return Err("concaténation de tailles différentes".into());
            }
            [lues.iter().map(|f| f[0]).sum(), h, l]
        }
        Sorte::Ajouter => {
            if lues.len() != 2 || lues[1] != premiere {
                return Err("addition de formes différentes".into());
            }
            premiere
        }
        Sorte::Sigmoide => une_seule(premiere)?,
    })
}
