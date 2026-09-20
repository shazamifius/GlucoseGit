//! Les plages d'une image : où elle est opaque, transparente, ou entre les deux — lues **une
//! fois**, pour que chaque composition n'ait plus à les chercher.
//!
//! # Le défaut que ce module corrige, mesuré
//!
//! Le mélange par ligne reconnaissait les plages **à chaque composition** : une comparaison
//! par pixel, sur un octet lu tous les quatre — un parcours que le compilateur ne sait pas
//! vectoriser. Sur un écran de 2560 × 1600 couvert de tuiles, `bench_salissure` mesurait ce
//! balayage à 4,8 ms : quatre fois la copie qu'il préparait, et la moitié de l'image.
//!
//! Or une tuile ne change pas entre deux images ; ses plages non plus. Elles se lisent au
//! rangement, une fois, et la composition n'a plus qu'à copier les opaques, sauter les
//! transparentes et mélanger les rares mixtes — sans lire un seul alpha.
//!
//! # La forme
//!
//! Par ligne, une suite de plages contiguës `[x0, x1[` d'une même nature, qui couvrent la
//! ligne entière. Les débuts de ligne indexent une liste plate : aucune allocation par ligne,
//! et une ligne se lit d'une tranche. Une tuile de photos a deux ou trois plages par ligne ;
//! le pire cas — un damier d'alphas — en a autant que de pixels, et coûte alors à composer ce
//! que coûtait le balayage, jamais plus.

use super::Pixel;

/// Ce qu'une plage de pixels fait au source-over, en prémultiplié.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nature {
    /// Alpha nul partout : `d = 0 + d × 1 = d`, rien à faire.
    Transparente,
    /// Un alpha entre les deux quelque part : le calcul général.
    Mixte,
    /// Alpha plein partout : `d = s + d × 0 = s`, une copie.
    Opaque,
}

impl Nature {
    pub(super) fn de(alpha: u8) -> Self {
        match alpha {
            0 => Self::Transparente,
            255 => Self::Opaque,
            _ => Self::Mixte,
        }
    }
}

/// Une plage contiguë d'une ligne, bord droit exclu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plage {
    pub x0: u32,
    pub x1: u32,
    pub nature: Nature,
}

/// Les plages de toutes les lignes d'une image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plages {
    largeur: u32,
    /// L'indice de la première plage de chaque ligne, plus un de fin : `hauteur + 1` entrées.
    debuts: Vec<u32>,
    plages: Vec<Plage>,
}

impl Plages {
    /// Lit les plages de `pixels`, une image de `largeur × hauteur`.
    ///
    /// Un seul passage sur les alphas. Rend des plages vides si les pixels ne font pas la
    /// taille annoncée : une image dont la taille ment n'a pas de plages qu'on puisse croire.
    pub fn de(pixels: &[Pixel], largeur: u32, hauteur: u32) -> Self {
        let attendu = (largeur as usize).checked_mul(hauteur as usize);
        if attendu != Some(pixels.len()) || largeur == 0 {
            return Self {
                largeur,
                debuts: vec![0; hauteur as usize + 1],
                plages: Vec::new(),
            };
        }
        let mut debuts = Vec::with_capacity(hauteur as usize + 1);
        let mut plages = Vec::new();
        for ligne in pixels.chunks_exact(largeur as usize) {
            debuts.push(plages.len() as u32);
            let mut x0 = 0u32;
            let mut nature = Nature::de(ligne[0][3]);
            for (x, pixel) in ligne.iter().enumerate().skip(1) {
                let n = Nature::de(pixel[3]);
                if n != nature {
                    plages.push(Plage {
                        x0,
                        x1: x as u32,
                        nature,
                    });
                    x0 = x as u32;
                    nature = n;
                }
            }
            plages.push(Plage {
                x0,
                x1: largeur,
                nature,
            });
        }
        debuts.push(plages.len() as u32);
        Self {
            largeur,
            debuts,
            plages,
        }
    }

    /// Les plages de la ligne `y`, de gauche à droite. Vide hors de l'image.
    pub fn ligne(&self, y: u32) -> &[Plage] {
        let y = y as usize;
        if y + 1 >= self.debuts.len() {
            return &[];
        }
        &self.plages[self.debuts[y] as usize..self.debuts[y + 1] as usize]
    }

    /// Combien de lignes ont été lues.
    pub fn hauteur(&self) -> u32 {
        self.debuts.len().saturating_sub(1) as u32
    }

    /// La boîte des pixels non transparents, `(x0, y0, x1, y1)` bords droit et bas exclus,
    /// ou `None` si tout est transparent.
    pub fn boite(&self) -> Option<(u32, u32, u32, u32)> {
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        for y in 0..self.hauteur() {
            for p in self
                .ligne(y)
                .iter()
                .filter(|p| p.nature != Nature::Transparente)
            {
                x0 = x0.min(p.x0);
                x1 = x1.max(p.x1);
                y0 = y0.min(y);
                y1 = y1.max(y + 1);
            }
        }
        (x0 != u32::MAX).then_some((x0, y0, x1, y1))
    }

    /// **Toute la boîte est-elle opaque** — pas un seul pixel qui laisse voir dessous ?
    /// Vrai pour une image entièrement transparente, qui n'a pas de boîte.
    ///
    /// # Le défaut que cette formulation corrige, et il se voyait à l'écran
    ///
    /// La première version répondait « aucune plage mixte », ce qui acceptait les plages
    /// **transparentes**. Une tuile trouée — deux photos séparées par du vide — était donc
    /// déclarée opaque, composée par `Remplacer`, et son trou ÉCRASAIT le fond en
    /// transparent : de gros carrés noirs derrière les photos, que l'utilisateur a vus dès
    /// la session suivante.
    ///
    /// L'opacité utile est celle de la **boîte** : c'est elle qui borne la composition, et
    /// c'est donc sur elle seule que la question se pose. Un trou à l'intérieur suffit à
    /// répondre non ; ce qui est autour ne décide de rien, puisque rien ne l'y compose.
    pub fn opaque(&self) -> bool {
        let Some((x0, y0, x1, y1)) = self.boite() else {
            return true;
        };
        (y0..y1).all(|y| {
            self.ligne(y)
                .iter()
                .filter(|p| p.x1 > x0 && p.x0 < x1)
                .all(|p| p.nature == Nature::Opaque)
        })
    }

    /// Ce que ces plages pèsent en mémoire, hors leur propre en-tête.
    pub fn octets(&self) -> usize {
        self.debuts.len() * std::mem::size_of::<u32>()
            + self.plages.len() * std::mem::size_of::<Plage>()
    }

    /// La largeur des lignes lues.
    pub fn largeur(&self) -> u32 {
        self.largeur
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(lignes: &[&[u8]]) -> (Vec<Pixel>, u32, u32) {
        let largeur = lignes[0].len() as u32;
        let pixels = lignes
            .iter()
            .flat_map(|l| l.iter().map(|a| [*a, *a, *a, *a]))
            .collect();
        (pixels, largeur, lignes.len() as u32)
    }

    #[test]
    fn test_les_plages_couvrent_chaque_ligne_sans_trou_ni_recouvrement() {
        let (px, l, h) = image(&[&[0, 0, 255, 255, 7, 0], &[255; 6], &[0; 6]]);
        let plages = Plages::de(&px, l, h);
        for y in 0..h {
            let ligne = plages.ligne(y);
            assert_eq!(ligne[0].x0, 0);
            assert_eq!(ligne.last().unwrap().x1, l);
            for w in ligne.windows(2) {
                assert_eq!(w[0].x1, w[1].x0, "un trou ou un recouvrement");
                assert_ne!(
                    w[0].nature, w[1].nature,
                    "deux plages voisines de même nature"
                );
            }
        }
        assert_eq!(
            plages.ligne(0),
            &[
                Plage {
                    x0: 0,
                    x1: 2,
                    nature: Nature::Transparente
                },
                Plage {
                    x0: 2,
                    x1: 4,
                    nature: Nature::Opaque
                },
                Plage {
                    x0: 4,
                    x1: 5,
                    nature: Nature::Mixte
                },
                Plage {
                    x0: 5,
                    x1: 6,
                    nature: Nature::Transparente
                },
            ]
        );
        assert_eq!(plages.ligne(1).len(), 1);
        assert_eq!(plages.ligne(3), &[], "hors de l'image, rien");
    }

    #[test]
    fn test_la_boite_et_l_opacite_se_deduisent_des_plages() {
        let (px, l, h) = image(&[&[0, 0, 0, 0], &[0, 255, 255, 0], &[0, 255, 255, 0], &[0; 4]]);
        let plages = Plages::de(&px, l, h);
        assert_eq!(plages.boite(), Some((1, 1, 3, 3)));
        assert!(plages.opaque());

        let (px, l, h) = image(&[&[0, 128, 255, 0]]);
        let plages = Plages::de(&px, l, h);
        assert_eq!(plages.boite(), Some((1, 0, 3, 1)));
        assert!(!plages.opaque(), "un alpha partiel suffit");

        let (px, l, h) = image(&[&[0; 3], &[0; 3]]);
        let plages = Plages::de(&px, l, h);
        assert_eq!(plages.boite(), None);
        assert!(plages.opaque(), "vide vaut opaque : rien à mélanger");
    }

    /// **Un trou dans la boîte suffit à refuser l'opacité.**
    ///
    /// C'est le défaut qui a mis de gros carrés noirs derrière les photos : une tuile
    /// portant deux photos séparées par du vide était déclarée opaque, donc composée par
    /// `Remplacer`, et son trou écrasait le fond en transparent. La première version ne
    /// regardait que l'absence de plages MIXTES, ce qui laissait passer les transparentes ;
    /// ce test échoue sur elle.
    #[test]
    fn test_un_trou_dans_la_boite_refuse_l_opacite() {
        // Deux photos opaques séparées par du vide, sur la même ligne.
        let (px, l, h) = image(&[&[0, 255, 255, 0, 0, 255, 255, 0]]);
        let plages = Plages::de(&px, l, h);
        assert_eq!(plages.boite(), Some((1, 0, 7, 1)));
        assert!(
            !plages.opaque(),
            "le vide entre les deux photos est DANS la boîte : elle n'est pas opaque"
        );

        // Le même trou, mais en diagonale : une photo en haut à gauche, une en bas à droite.
        let (px, l, h) = image(&[&[255, 255, 0, 0], &[0, 0, 255, 255]]);
        let plages = Plages::de(&px, l, h);
        assert_eq!(plages.boite(), Some((0, 0, 4, 2)));
        assert!(!plages.opaque(), "chaque ligne a son trou dans la boîte");

        // Et le cas qui doit rester opaque : la boîte est pleine, le vide est autour.
        let (px, l, h) = image(&[&[0; 4], &[0, 255, 255, 0], &[0; 4]]);
        let plages = Plages::de(&px, l, h);
        assert_eq!(plages.boite(), Some((1, 1, 3, 2)));
        assert!(
            plages.opaque(),
            "ce qui est hors de la boîte n'est jamais composé, donc ne décide de rien"
        );
    }

    #[test]
    fn test_une_taille_qui_ment_ne_donne_aucune_plage() {
        let plages = Plages::de(&[[0; 4]; 5], 3, 2);
        assert_eq!(plages.ligne(0), &[]);
        assert_eq!(plages.boite(), None);
    }
}
