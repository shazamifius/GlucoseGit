//! Reporter une image dans une autre, restreinte à un rectangle (REPORT-1).
//!
//! # Ce que cette primitive répare
//!
//! L'occlusion sait dire, pour chaque photo, quels morceaux d'elle atteignent l'œil. Le rendu
//! n'en lisait qu'une chose : cette liste est-elle **vide**, c'est-à-dire la photo est-elle
//! *entièrement* cachée. Une photo recouverte à quatre-vingt-dix-neuf pour cent était donc
//! peinte à cent. Les morceaux visibles étaient calculés, puis jetés.
//!
//! Ce n'était pas un oubli. Peindre une image restreinte à un rectangle demande au rastériseur
//! un masque de la taille de l'écran, qui coûte plus cher que ce qu'il fait gagner. Mesuré
//! chez l'utilisateur, cent photos empilées repeignaient **cent quarante-neuf fois** la
//! surface de la fenêtre.
//!
//! # La forme, et pourquoi elle rend le clip gratuit
//!
//! Le clip n'est pas une option de cette fonction : c'est **son domaine d'itération**. On
//! parcourt les pixels du rectangle visible, jamais ceux de l'image. Une photo dont il ne
//! reste qu'une bande de trois pixels coûte trois colonnes, et rien d'autre — il n'y a aucun
//! travail à jeter après coup, puisqu'il n'a pas été fait.
//!
//! # Pourquoi elle vit dans le noyau
//!
//! Elle ne dépend de rien : ni du rastériseur, ni du système, ni de la carte graphique. Des
//! octets en entrée, des octets en sortie, et une géométrie. C'est ce qui la rend utilisable
//! par le chemin processeur d'aujourd'hui comme par celui d'un téléphone demain — et c'est ce
//! que la charte demande du chemin processeur : un vrai chemin, pas un secours.
//!
//! # Le cas rapide se constate, il ne se déclare pas
//!
//! Quand la source est déjà à la taille voulue et posée sur un pixel entier, échantillonner
//! n'a plus de sens : chaque pixel de destination a exactement un pixel source. La fonction le
//! **constate sur les valeurs** — aucun drapeau, aucun appelant à qui faire confiance — et la
//! ligne devient une recopie contiguë, que le compilateur ramène à un déplacement de mémoire.
//! C'est le chemin de la vignette, et c'est le cas dominant.

use crate::occlusion::Boite;

/// Un pixel, tel qu'il vit en mémoire : rouge, vert, bleu, alpha, **prémultipliés**.
///
/// Quatre octets plutôt qu'un entier de trente-deux bits : l'ordre en mémoire est alors le
/// même sur toutes les machines, quel que soit leur boutisme. Une image écrite sur un
/// processeur et lue sur un autre donne les mêmes couleurs.
pub type Pixel = [u8; 4];

/// Une image qu'on lit.
#[derive(Debug, Clone, Copy)]
pub struct Vue<'a> {
    pixels: &'a [Pixel],
    largeur: u32,
    hauteur: u32,
}

/// Une image qu'on écrit.
#[derive(Debug)]
pub struct VueMut<'a> {
    pixels: &'a mut [Pixel],
    largeur: u32,
    hauteur: u32,
}

impl<'a> Vue<'a> {
    /// Rend `None` si les pixels ne sont pas exactement `largeur × hauteur`.
    ///
    /// La vérification a lieu **une fois**, ici, ce qui permet à la boucle de pixels de n'en
    /// faire aucune : c'est le seul endroit où une taille peut mentir.
    pub fn nouvelle(pixels: &'a [Pixel], largeur: u32, hauteur: u32) -> Option<Self> {
        let attendu = (largeur as usize).checked_mul(hauteur as usize)?;
        (pixels.len() == attendu && largeur > 0 && hauteur > 0).then_some(Self {
            pixels,
            largeur,
            hauteur,
        })
    }

    /// Sa largeur en pixels.
    pub fn largeur(&self) -> u32 {
        self.largeur
    }

    /// Sa hauteur en pixels.
    pub fn hauteur(&self) -> u32 {
        self.hauteur
    }

    fn ligne(&self, y: u32) -> &[Pixel] {
        let d = y as usize * self.largeur as usize;
        &self.pixels[d..d + self.largeur as usize]
    }
}

impl<'a> VueMut<'a> {
    /// Rend `None` si les pixels ne sont pas exactement `largeur × hauteur`.
    pub fn nouvelle(pixels: &'a mut [Pixel], largeur: u32, hauteur: u32) -> Option<Self> {
        let attendu = (largeur as usize).checked_mul(hauteur as usize)?;
        (pixels.len() == attendu && largeur > 0 && hauteur > 0).then_some(Self {
            pixels,
            largeur,
            hauteur,
        })
    }

    fn ligne_mut(&mut self, y: u32) -> &mut [Pixel] {
        let d = y as usize * self.largeur as usize;
        &mut self.pixels[d..d + self.largeur as usize]
    }
}

/// Où la source se pose dans la destination, en pixels de destination.
///
/// La rotation n'y figure pas, et c'est voulu : une image tournée ne couvre plus sa boîte,
/// donc elle ne cache rien et ne se clippe pas en rectangles. Elle garde l'autre chemin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub x: f32,
    pub y: f32,
    pub largeur: f32,
    pub hauteur: f32,
}

/// Avec quelle finesse un pixel se tire de la source.
///
/// # Ce que la pixelisation achète, et ce qu'elle coûte
///
/// Interpoler demande **quatre** texels et trois mélanges par pixel ; prendre le plus proche
/// en demande un et aucun. Le gain n'est donc pas un réglage de qualité, c'est un rapport
/// mesurable — et c'est ce qui permet de tenir le budget quand la scène n'y tient pas
/// autrement. La charte le dit sans détour : sous cent images par seconde, on pixelise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filtre {
    /// Les quatre texels voisins, interpolés. Ce que l'œil attend à l'arrêt.
    Lisse,
    /// Le texel le plus proche, seul. Rapide, et franchement pixelisé.
    PlusProche,
}

/// Comment les pixels de la source rejoignent ceux de la destination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Melange {
    /// La source **remplace** : licite seulement si elle est opaque partout.
    Remplacer,
    /// La source se compose par-dessus, en prémultiplié.
    Composer,
}

/// Reporte `src`, posée selon `pose`, dans `dest`, **restreinte à `clip`**.
///
/// Rend le nombre de pixels écrits — la seule grandeur qui décide du gain, et celle que la
/// chronique enregistre. Zéro veut dire que le clip et la pose ne se rencontrent pas, ce qui
/// est le cas normal d'une photo entièrement recouverte.
pub fn reporter(
    dest: &mut VueMut<'_>,
    src: &Vue<'_>,
    pose: Pose,
    clip: Boite,
    melange: Melange,
    filtre: Filtre,
) -> u64 {
    let Some(zone) = domaine(dest, pose, clip) else {
        return 0;
    };

    // Un pixel pour un pixel : la source est déjà à la taille voulue, et posée sur un entier.
    // Se constate sur les valeurs, donc aucun appelant ne peut se tromper en le déclarant.
    if pose.largeur == src.largeur as f32
        && pose.hauteur == src.hauteur as f32
        && pose.x == pose.x.floor()
        && pose.y == pose.y.floor()
    {
        return reporter_tel_quel(dest, src, pose, zone, melange);
    }

    reporter_en_echantillonnant(dest, src, pose, zone, melange, filtre)
}

/// Les pixels de destination à écrire : ceux dont le **centre** tombe à la fois dans la pose,
/// dans le clip et dans l'image.
///
/// La règle du centre est celle d'un rastériseur : elle donne exactement la même couverture
/// qu'un report sans clip lorsque le clip contient toute la pose, ce qu'un test vérifie.
fn domaine(dest: &VueMut<'_>, pose: Pose, clip: Boite) -> Option<(u32, u32, u32, u32)> {
    if pose.largeur <= 0.0 || pose.hauteur <= 0.0 || clip.est_vide() {
        return None;
    }
    let gauche = pose.x.max(clip.x);
    let haut = pose.y.max(clip.y);
    let droite = (pose.x + pose.largeur).min(clip.droite());
    let bas = (pose.y + pose.hauteur).min(clip.bas());

    let x0 = premier_centre(gauche).max(0);
    let y0 = premier_centre(haut).max(0);
    let x1 = dernier_centre(droite).min(i64::from(dest.largeur) - 1);
    let y1 = dernier_centre(bas).min(i64::from(dest.hauteur) - 1);
    (x0 <= x1 && y0 <= y1).then_some((x0 as u32, y0 as u32, x1 as u32, y1 as u32))
}

/// Le premier pixel dont le centre est au moins à `bord`.
fn premier_centre(bord: f32) -> i64 {
    (f64::from(bord) - 0.5).ceil() as i64
}

/// Le dernier pixel dont le centre est strictement avant `bord`.
fn dernier_centre(bord: f32) -> i64 {
    (f64::from(bord) - 0.5).ceil() as i64 - 1
}

/// Un pixel de source pour un pixel de destination : la ligne devient une recopie contiguë.
fn reporter_tel_quel(
    dest: &mut VueMut<'_>,
    src: &Vue<'_>,
    pose: Pose,
    (x0, y0, x1, y1): (u32, u32, u32, u32),
    melange: Melange,
) -> u64 {
    let (dx, dy) = (pose.x as i64, pose.y as i64);
    let mut ecrits = 0u64;
    for y in y0..=y1 {
        let sy = i64::from(y) - dy;
        if sy < 0 || sy >= i64::from(src.hauteur) {
            continue;
        }
        // Le décalage est le même sur les deux bords : la partie commune se lit d'un bloc,
        // et ce qui déborde de la source se rogne une fois pour toute la ligne.
        let sx0 = i64::from(x0) - dx;
        let sx1 = i64::from(x1) - dx;
        let rogne_gauche = (-sx0).max(0);
        let rogne_droite = (sx1 - (i64::from(src.largeur) - 1)).max(0);
        let largeur = (sx1 - sx0 + 1) - rogne_gauche - rogne_droite;
        if largeur <= 0 {
            continue;
        }
        let depart_src = (sx0 + rogne_gauche) as usize;
        let depart_dest = (i64::from(x0) + rogne_gauche) as usize;
        let n = largeur as usize;
        let source = &src.ligne(sy as u32)[depart_src..depart_src + n];
        let cible = &mut dest.ligne_mut(y)[depart_dest..depart_dest + n];
        match melange {
            Melange::Remplacer => cible.copy_from_slice(source),
            Melange::Composer => {
                for (d, s) in cible.iter_mut().zip(source) {
                    *d = compose(*s, *d);
                }
            }
        }
        ecrits += n as u64;
    }
    ecrits
}

/// Combien de bits de partie fractionnaire portent les coordonnées de texel.
///
/// Seize : l'erreur de position est alors de `2⁻¹⁶` texel, indiscernable sur une image de
/// moins de soixante-cinq mille pixels de côté — au-delà, c'est la source qui manque de
/// précision, pas l'adressage.
const FIXE: u32 = 16;

/// L'unité, au format fixe.
const UN: i64 = 1 << FIXE;

/// Une source rééchantillonnée : le cas du zoom, où un pixel d'écran tombe entre deux texels.
///
/// L'incrément est **constant**, en `x` comme en `y`, donc aucune division ne subsiste dans
/// les boucles : la position de texel s'additionne, comme dans tout rastériseur.
fn reporter_en_echantillonnant(
    dest: &mut VueMut<'_>,
    src: &Vue<'_>,
    pose: Pose,
    zone: (u32, u32, u32, u32),
    melange: Melange,
    filtre: Filtre,
) -> u64 {
    // La hauteur ne sert qu'a la boucle : ici on n'a besoin que du coin haut-gauche, d'ou
    // partent les deux positions de texel, et de `x1` pour borner les colonnes interieures.
    let (x0, y0, x1, _) = zone;
    let pas_x = (f64::from(src.largeur) / f64::from(pose.largeur) * UN as f64) as i64;
    let pas_y = (f64::from(src.hauteur) / f64::from(pose.hauteur) * UN as f64) as i64;

    // Le centre du pixel de destination, ramené en texels, puis reculé d'un demi-texel : c'est
    // ce demi qui aligne les centres de texel sur les centres de pixel. Sans lui l'image
    // glisse d'une moitié de texel, et le glissement se voit dès qu'on change d'échelle.
    let depart = |centre: f64, taille: f64, source: u32| -> i64 {
        (centre / taille * f64::from(source) * UN as f64) as i64 - UN / 2
    };
    let u0 = depart(
        f64::from(x0) + 0.5 - f64::from(pose.x),
        f64::from(pose.largeur),
        src.largeur,
    );
    let v = depart(
        f64::from(y0) + 0.5 - f64::from(pose.y),
        f64::from(pose.hauteur),
        src.hauteur,
    );

    // Les colonnes où les deux texels voisins existent vraiment.
    //
    // Les borner à chaque pixel coûtait quatre comparaisons par pixel, sur un chemin qui en
    // traite des millions — alors que la réponse est la même pour toute une colonne, et que
    // l'immense majorité d'entre elles sont à l'intérieur. On les calcule donc **une fois**,
    // et la boucle centrale n'a plus rien à vérifier.
    let dedans = colonnes_interieures(u0, pas_x, src.largeur, (x0, x1));

    let pas = (u0, v, pas_x, pas_y);
    match (melange, filtre) {
        (Melange::Remplacer, Filtre::Lisse) => remplir::<true, true>(dest, src, pas, zone, dedans),
        (Melange::Remplacer, Filtre::PlusProche) => {
            remplir::<true, false>(dest, src, pas, zone, dedans)
        }
        (Melange::Composer, Filtre::Lisse) => remplir::<false, true>(dest, src, pas, zone, dedans),
        (Melange::Composer, Filtre::PlusProche) => {
            remplir::<false, false>(dest, src, pas, zone, dedans)
        }
    }
}

/// La boucle de pixels, une fois le mode de mélange connu.
///
/// # Pourquoi le mélange est un paramètre de type
///
/// Il ne change pas d'un pixel à l'autre — ni même d'une image à l'autre pour une photo
/// donnée. Le tester dans la boucle coûtait une branche par pixel, sur un chemin qui en traite
/// des millions. En paramètre de type, le compilateur produit les deux boucles et chacune ne
/// contient plus que son cas.
///
/// La destination est parcourue par **tranche** plutôt que par indice : la longueur est alors
/// connue de la boucle, et la vérification de bornes par pixel disparaît.
fn remplir<const REMPLACE: bool, const LISSE: bool>(
    dest: &mut VueMut<'_>,
    src: &Vue<'_>,
    (u0, mut v, pas_x, pas_y): (i64, i64, i64, i64),
    (x0, y0, x1, y1): (u32, u32, u32, u32),
    (dedans_debut, dedans_fin): (u32, u32),
) -> u64 {
    let mut ecrits = 0u64;
    for y in y0..=y1 {
        // `v` ne change pas le long d'une ligne : les deux lignes source et leur poids se
        // lisent une fois, au lieu d'une multiplication et de deux bornages par pixel.
        let (ya, yb, fv) = voisins(v, src.hauteur);
        let (haute, basse) = (src.ligne(ya), src.ligne(yb));
        let ligne = &mut dest.ligne_mut(y)[x0 as usize..=x1 as usize];

        let mut u = u0;
        for (i, d) in ligne.iter_mut().enumerate() {
            let x = x0 + i as u32;
            let (xa, xb, fu) = if x >= dedans_debut && x <= dedans_fin {
                let a = (u >> FIXE) as u32;
                (a, a + 1, ((u & (UN - 1)) >> (FIXE - 8)) as u32)
            } else {
                voisins(u, src.largeur)
            };
            // Pixeliser, c'est prendre le texel dont le centre est le plus proche : une
            // lecture au lieu de quatre, aucun melange, et le pas de la source se voit.
            let s = if LISSE {
                melanger(
                    melanger(haute[xa as usize], haute[xb as usize], fu),
                    melanger(basse[xa as usize], basse[xb as usize], fu),
                    fv,
                )
            } else {
                let colonne = if fu >= 128 { xb } else { xa };
                let ligne = if fv >= 128 { basse } else { haute };
                ligne[colonne as usize]
            };
            *d = if REMPLACE { s } else { compose(s, *d) };
            u += pas_x;
        }
        v += pas_y;
        ecrits += u64::from(x1 - x0 + 1);
    }
    ecrits
}

/// Les deux texels qui encadrent la position `t`, et le poids du second, sur huit bits.
///
/// Les bords se **prolongent** au lieu de se replier : un texel demandé hors de l'image rend le
/// plus proche. C'est ce qui évite qu'une image ne bave sur son bord opposé en zoom proche.
///
/// Le poids est ramené à huit bits : l'erreur qu'il introduit vaut un deux-cent-cinquante-
/// sixième de l'écart entre deux texels, soit moins d'un demi-niveau sur une sortie de huit
/// bits. Elle est donc **invisible par construction**, et c'est ce qui permet d'interpoler les
/// quatre canaux d'un seul geste.
fn voisins(t: i64, taille: u32) -> (u32, u32, u32) {
    let entier = t >> FIXE;
    let borne = |k: i64| k.clamp(0, i64::from(taille) - 1) as u32;
    (
        borne(entier),
        borne(entier + 1),
        ((t & (UN - 1)) >> (FIXE - 8)) as u32,
    )
}

/// La plage de colonnes où `u` et `u + 1` tombent tous deux dans la source.
///
/// Rend une plage vide — début après fin — quand il n'y en a aucune, ce qui arrive pour une
/// image vue de très loin, où chaque pixel saute par-dessus plusieurs texels.
///
/// **Le début ne se ramène jamais dans le clip.** Le rogner reviendrait à déclarer intérieure
/// une colonne qui ne l'est pas, et la boucle centrale lirait alors un texel hors de la source.
/// Un clip d'une seule colonne suffisait à le déclencher — c'est ce que le test du découpage
/// en morceaux a attrapé.
fn colonnes_interieures(u0: i64, pas: i64, largeur: u32, (x0, x1): (u32, u32)) -> (u32, u32) {
    const AUCUNE: (u32, u32) = (u32::MAX, 0);
    if pas <= 0 || largeur < 2 {
        return AUCUNE;
    }
    let haut = (i64::from(largeur) - 1) * UN;
    // `u(x) = u0 + (x − x0) · pas`, donc `0 ⩽ u(x) < haut` se renverse exactement.
    let premier = (-u0).div_euclid(pas) + i64::from((-u0).rem_euclid(pas) != 0);
    let dernier = (haut - 1 - u0).div_euclid(pas);
    let debut = i64::from(x0) + premier.max(0);
    let fin = i64::from(x0) + dernier;
    if debut > fin || debut > i64::from(x1) || fin < i64::from(x0) {
        return AUCUNE;
    }
    (debut as u32, fin.min(i64::from(x1)) as u32)
}

/// Un canal sur deux, isolé dans les champs pairs d'un entier de trente-deux bits.
const UN_CANAL_SUR_DEUX: u32 = 0x00FF_00FF;

/// Interpole deux pixels, `t` allant de zéro (tout `a`) à deux cent cinquante-cinq.
///
/// # Les quatre canaux d'un seul geste
///
/// Un pixel tient dans un entier de trente-deux bits. Séparé en deux moitiés — un canal sur
/// deux — chaque moitié occupe deux champs de huit bits **espacés de huit bits vides**. Le
/// produit d'un champ par un poids de huit bits tient alors exactement dans l'espace
/// disponible, et les deux canaux ne peuvent pas déborder l'un sur l'autre : `255 × 256`
/// vaut `65 280`, et le champ en porte `65 535`.
///
/// Deux multiplications remplacent donc huit, sans une seule instruction qui dépende de la
/// machine — ce qui la rend aussi rapide sur un téléphone de 2013 que sur un processeur
/// récent, là où un jeu d'instructions vectoriel exclurait l'un des deux.
///
/// L'ordre des octets n'a pas à être connu : les quatre canaux subissent le même traitement,
/// donc les permuter à l'entrée et à la sortie ne change rien au résultat.
fn melanger(a: Pixel, b: Pixel, t: u32) -> Pixel {
    let (a, b) = (u32::from_ne_bytes(a), u32::from_ne_bytes(b));
    let inverse = 256 - t;
    // Le demi ajouté à chaque champ arrondit au plus proche au lieu de tronquer : sans lui,
    // chaque interpolation perdrait en moyenne un demi-niveau, et une image rééchantillonnée
    // plusieurs fois s'assombrirait.
    let demi = 0x0080_0080;
    let pairs = ((a & UN_CANAL_SUR_DEUX) * inverse + (b & UN_CANAL_SUR_DEUX) * t + demi) >> 8;
    let impairs =
        (((a >> 8) & UN_CANAL_SUR_DEUX) * inverse + ((b >> 8) & UN_CANAL_SUR_DEUX) * t + demi)
            & !UN_CANAL_SUR_DEUX;
    ((pairs & UN_CANAL_SUR_DEUX) | impairs).to_ne_bytes()
}

/// « Source par-dessus », en prémultiplié : `d = s + d × (1 − a)`.
fn compose(s: Pixel, d: Pixel) -> Pixel {
    let inv = 255 - s[3];
    let mut sortie = [0u8; 4];
    for c in 0..4 {
        sortie[c] = s[c].saturating_add(mul255(d[c], inv));
    }
    sortie
}

/// `a × b / 255`, exact pour tous les octets, et sans division.
fn mul255(a: u8, b: u8) -> u8 {
    let t = u32::from(a) * u32::from(b) + 128;
    ((t + (t >> 8)) >> 8) as u8
}

#[cfg(test)]
mod tests;
