//! **BORDURES-1** : trouver les bandes unies qui entourent une image, pour les retirer.
//!
//! # Ce que l'utilisateur demande
//!
//! *« Un bouton qui retire les bordures d'un lot d'images sélectionnées — détecter les bandes
//! noires ou blanches, puis rogner. »* Une capture d'écran de vidéo en boîte aux lettres, une
//! photo scannée avec sa marge, une image de site web sur son fond : la bande est du bruit
//! autour de ce qu'on voulait garder, et la retirer à la main sur cinquante images est
//! exactement le genre de corvée qu'un logiciel doit faire seul.
//!
//! # Ce que la référence fait, et ce qu'on en garde
//!
//! `cropdetect`, le filtre de FFmpeg qui fait ce travail sur des millions de vidéos depuis
//! vingt ans, procède ainsi : depuis chaque bord, ligne après ligne, il fait la **moyenne** de
//! la ligne et s'arrête à la première dont la moyenne dépasse un seuil de noir — **24 sur
//! 255** par défaut, soit le noir du signal vidéo (16) plus une marge pour le bruit.
//!
//! On garde la structure — quatre balayages depuis les bords, un arrêt à la première ligne
//! qui n'est plus bande — et on change deux choses, que deux tests ont imposées.
//!
//! **La bande n'est pas noire, elle est unie.** Ce qu'on mesure n'est donc pas la clarté de la
//! ligne, mais sa **distance à la couleur du bord**. Une bande blanche, grise ou d'une couleur
//! de fond se détecte alors exactement comme une noire, et le seuil de FFmpeg garde son sens :
//! c'est l'écart qu'un noir vidéo bruité atteint sans cesser d'être du noir.
//!
//! **Un centile, et non la moyenne ni le pire pixel.** Quatre critères ont été essayés, et
//! chacun a été refusé par une mesure :
//!
//! | critère | ce qu'il fait de faux | ce qui l'a dit |
//! |---|---|---|
//! | la **moyenne** (FFmpeg) | mange le haut des lettres : la première ligne de pixels d'un texte porte trois pour cent d'encre, et sa moyenne reste sous le seuil | un test de texte fin sur fond uni |
//! | la **médiane** | prend un damier entier pour une bande — la moitié des pixels d'une ligne de texte sont du fond | le damier des preuves |
//! | le **pire pixel** | s'arrête trop tôt et **laisse un liseré visible** : sur une illustration réelle, la première ligne refusée portait quatre pixels de plume sur cinq cent six | l'image de l'utilisateur, mesurée |
//! | le **centile 99** | retenu | — |
//!
//! Une ligne est de la bande si **au plus un centième** de ses pixels s'écartent du bord de
//! plus que le bruit. Ce que cela laisse passer est ce qu'on ne perd pas à couper : sur une
//! ligne de cinq cents pixels, un pour cent en fait cinq — le bruit d'un bord compressé, ou le
//! premier rang anti-crénelé d'une forme. Ce que cela arrête est un contenu, qui en occupe
//! davantage dès sa première ligne.
//!
//! **Ce nombre se juge à la main**, comme `TAU_CONDUITE` et `PAN_LIGNE_PX` : aucune loi ne le
//! donne, et son journal est au-dessus de [`PART_ABERRANTE`]. Un filigrane posé dans la bande
//! l'arrête à sa ligne, comme chez FFmpeg — c'est assumé, et un test le dit.
//!
//! # Les quatre bords se rognent ensemble, jusqu'à ce que plus rien ne bouge
//!
//! Le bord droit d'une image encadrée traverse les bandes du haut et du bas, qui n'ont aucune
//! raison d'être de sa couleur : mesuré sur toute la hauteur, il n'est pas uni, et on ne le
//! trouve pas. Le second test l'a dit. Chaque bord se mesure donc **à l'intérieur de ce que les
//! autres ont déjà retiré**, et on recommence tant qu'un bord recule encore. Deux passes
//! suffisent en pratique ; la boucle s'arrête d'elle-même parce qu'un rectangle ne peut que
//! rétrécir.
//!
//! # BORDURES-3 — la couleur d'une bande se fige, et elle emporte ses rangs de transition
//!
//! L'utilisateur l'a signalé deux fois, et sa capture du 23/09 l'a chiffré : après `Ctrl+B`,
//! **un liseré clair reste** sur trois côtés. Au bord du haut, de l'extérieur vers l'intérieur :
//! `243,239,218`, puis `147,140,103`, puis l'image. La bande blanche est partie ; les deux rangs
//! qui la **mêlaient** à l'image sont restés. L'anticrénelage et la compression fondent toujours
//! le rang qui touche la bande : il n'est ni la bande, ni le contenu.
//!
//! Un rang est une **transition** quand il s'explique comme un mélange de la couleur de la
//! bande et du rang suivant — `a · bande + (1 − a) · suivant`, avec un `a` commun à toute la
//! ligne, trouvé par moindres carrés — **et** ne s'explique pas comme du simple contenu, c'est
//! à dire qu'il diffère du rang suivant de plus que le bruit. Les deux tolérances sont
//! [`ECART_DE_BANDE`] au centile de [`PART_ABERRANTE`] : aucun nombre n'est nouveau. Un dégradé
//! du contenu, qui change de quelques niveaux par rang, n'est jamais une transition ; le rang
//! qui saute de quarante niveaux vers la bande en est une.
//!
//! **Et un défaut plus ancien, que le test du dégradé a montré** : la boucle des quatre bords
//! reprenait à chaque passage, comme couleur de bande, **le rang où elle venait de s'arrêter**.
//! Un dégradé qui touche une bande se faisait donc grignoter de deux ou trois rangs à chaque
//! tour — vingt et un rangs retirés là où la bande en comptait dix. La couleur d'un bord se
//! **fige** désormais dès qu'il a trouvé sa bande ; un bord qui n'en a pas encore trouvé la
//! cherche toujours à l'intérieur de ce que les autres ont retiré, ce pour quoi la boucle
//! existe.
//!
//! # BORDURES-5 — un filet au bord ne cache pas la bande
//!
//! Son image du 24/09, 849 × 1200 : une photo sombre dans une marge blanche. Après `Ctrl+B`, le
//! haut, la gauche et le bas étaient parfaits, et **toute la marge de droite restait**. Sa
//! dernière colonne est un **filet gris** — `88,88,88` sur toute la hauteur, un pixel de large
//! —, et les cent huit colonnes blanches commencent juste derrière. Le bord droit prenait ce
//! gris pour la couleur de sa bande, en retirait la colonne, puis s'arrêtait sur le blanc,
//! qui n'est pas du gris. Les trois autres bords, eux, ne croisaient ce filet que sur un pixel
//! de leur ligne : dans la tolérance.
//!
//! Ce n'était pas un accident : sur les 275 images de ses documents, **quatre** portent un tel
//! filet devant une bande — la découpe ou le redimensionnement en laisse un sur la dernière
//! ligne —, et chaque fois tout un côté restait. Un filet est **une seule ligne unie, derrière
//! laquelle commence une bande épaisse d'une autre couleur** ; il part, et la bande se cherche
//! derrière lui comme au bord. D'où la propriété que les épreuves vérifient sur chacune de
//! leurs images : **un filet au bord ne change rien à ce que `Ctrl+B` trouve derrière lui** —
//! une ligne de plus, exactement.
//!
//! **Un seul filet.** En admettre plusieurs à la suite fait d'un dégradé une pile de filets :
//! le haut d'une photo de dune part d'un voile clair qui s'assombrit ligne après ligne, et le
//! ciel s'ouvrait comme une bande derrière — soixante-quatorze lignes de ciel partaient.
//!
//! **Ce qui a été essayé et refusé** : ouvrir une bande derrière n'importe quelle bande, dès
//! qu'une ligne unie succède franchement à la précédente. Cela retirait aussi un passe-partout
//! entier — mais, sur ses images, le fond uni d'une photo posée entre deux bandes noires
//! partait avec : cent vingt-trois lignes de photo. Une bande noire de film et le fond gris
//! d'un studio sont les mêmes pixels ; seul le filet, qui n'a qu'une ligne, ne se confond
//! avec rien.
//!
//! # Ce qui sépare la bande du contenu
//!
//! Quand ils ne se touchent pas d'un trait, la lisière est dans le module `lisiere` : le fondu d'un
//! rang (BORDURES-3), une tache sur la marge (BORDURES-6), la frange d'un bord de plaque
//! (BORDURES-7).
//!
//! # Ce qui trompe, et qu'on ne cache pas
//!
//! Une photo dont le **contenu** commence par une zone unie — un ciel sans nuage en haut, un
//! mur blanc à gauche — se fera rogner de cette zone. FFmpeg a le même défaut et le
//! documente. Il n'y a pas de critère local qui distingue un ciel uni d'une bande, parce que
//! ce sont les mêmes pixels ; seule l'intention les sépare. L'action est donc **explicite et
//! annulable** — un geste, un `Ctrl+Z` — et jamais automatique.
//!
//! Et une image où **tout** est bande n'a rien autour de quoi une bordure existerait : rien
//! n'en est retiré. Avant, le recadrage la réduisait à son centième, la plus fine bande que
//! son invariant tolère — une nuance unie, `Ctrl+B`, et il en restait un trait.

use crate::report::{Pixel, Vue};
use crate::types::Recadrage;
use lisiere::{enjamber, est_une_transition, frange};

mod lisiere;

/// **L'écart au-delà duquel un pixel n'est plus de la bande**, en niveaux sur 255.
///
/// C'est le seuil de noir de `cropdetect` — `24.0 / 255` dans `vf_cropdetect.c` — et il n'est
/// pas à nous : c'est le noir du signal vidéo, seize, plus ce que le bruit de compression y
/// ajoute sans qu'on cesse de le voir noir. Appliqué à une distance plutôt qu'à une clarté, il
/// dit la même chose d'une bande blanche que d'une bande noire.
pub const ECART_DE_BANDE: f64 = 24.0;

/// **La part des pixels d'une ligne qui peuvent s'écarter sans qu'elle cesse d'être de la
/// bande.**
///
/// # Ce nombre se juge à la main, et voici son journal
///
/// * **zéro** — le pire pixel. Refusé par l'image de l'utilisateur : la première ligne refusée
///   du haut de son illustration portait **quatre pixels de plume sur cinq cent six**, et les
///   trois lignes suivantes six, huit et treize. Le recadrage s'arrêtait donc trois lignes
///   avant la vraie frontière, et **le liseré blanc restant se voyait à l'écran** ;
/// * **un pour cent** — retenu. Sur une ligne de cinq cents pixels, cinq pixels : le bruit d'un
///   bord compressé, ou le premier rang anti-crénelé d'une forme. Ce n'est pas ce qu'on perd à
///   couper ;
/// * **deux pour cent** — pas essayé sur le terrain. Il couperait trois lignes de plus sur la
///   même image, et rognerait la première ligne d'un texte fin à moins de deux pour cent
///   d'encre. À reprendre si un cas réel le réclame.
///
/// Ce n'est **pas** [`crate::cadence::PART_TOLEREE`], qui vaut le même nombre : celle-là
/// compte des images ratées par une cadence, celle-ci des pixels dans une ligne. Deux
/// grandeurs sans rapport qui partagent une valeur ne doivent pas partager une constante —
/// c'est exactement la faute d'ARBITRE-1, qui avait repris le seuil du tempo sans reprendre sa
/// grandeur.
pub const PART_ABERRANTE: f64 = 0.01;

/// **Les bandes unies qui entourent cette image**, comme recadrage à lui appliquer.
///
/// Rend [`Recadrage::ENTIER`] quand il n'y en a aucune. Le coût est celui des bandes, pas de
/// l'image : le balayage s'arrête à la première ligne qui n'en est plus — ou, derrière une
/// tache, dès que ce qui suit ne peut plus être une marge —, donc une photo sans bordure ne
/// coûte que quelques lignes.
pub fn detecter(image: &Vue<'_>) -> Recadrage {
    let (l, h) = (image.largeur(), image.hauteur());
    // Le rectangle qui reste : [x0, x1[ × [y0, y1[. Il ne peut que rétrécir.
    let (mut x0, mut y0, mut x1, mut y1) = (0u32, 0u32, l, h);
    // Ce que chaque bord -- haut, bas, gauche, droite -- a retiré : la couleur de sa bande,
    // figée dès qu'il l'a trouvée (BORDURES-3) sauf derrière un filet (BORDURES-5), et le
    // compte de ses pixels (BORDURES-6). Deux bords opposés se suivent dans le tableau : le
    // vis-à-vis de `k` est `k ^ 1`.
    let mut bords = [Bord::default(); 4];
    loop {
        let avant = (x0, y0, x1, y1);
        // Chaque bord sait ce qu'il a déjà retiré : un filet ne se reconnaît qu'au bord.
        let vers = |a: u32, b: u32| (a..b).collect::<Vec<u32>>();
        let a_rebours = |a: u32, b: u32| (a..b).rev().collect::<Vec<u32>>();
        y0 += compter(y0, &vers(y0, y1), &mut bords, 0, |y| {
            ligne(image, y, x0..x1)
        });
        y1 -= compter(h - y1, &a_rebours(y0, y1), &mut bords, 1, |y| {
            ligne(image, y, x0..x1)
        });
        x0 += compter(x0, &vers(x0, x1), &mut bords, 2, |x| {
            colonne(image, x, y0..y1)
        });
        x1 -= compter(l - x1, &a_rebours(x0, x1), &mut bords, 3, |x| {
            colonne(image, x, y0..y1)
        });
        if (x0, y0, x1, y1) == avant {
            break;
        }
    }
    // Tout était bande : il n'y a rien autour de quoi une bordure existerait.
    if x0 >= x1 || y0 >= y1 {
        return Recadrage::ENTIER;
    }
    Recadrage::depuis_les_marges(
        f64::from(x0) / f64::from(l),
        f64::from(y0) / f64::from(h),
        f64::from(l - x1) / f64::from(l),
        f64::from(h - y1) / f64::from(h),
    )
}

/// Ce qu'un bord a retiré jusqu'ici : la couleur de sa bande, et le compte de ses pixels —
/// tous, et ceux qui s'écartaient de cette couleur (BORDURES-6).
///
/// C'est sur ce compte que la tolérance du centième se juge **pour la bande entière**, et non
/// plus seulement ligne par ligne : c'est ce qui permet d'enjamber une tache ([`lisiere::enjamber`]).
#[derive(Debug, Clone, Copy, Default)]
struct Bord {
    couleur: Option<Pixel>,
    pixels: usize,
    etrangers: usize,
}

impl Bord {
    /// Retire cette ligne de la bande, et la compte.
    fn retirer(&mut self, ligne: &[Pixel], couleur: Pixel) {
        self.pixels += ligne.len();
        self.etrangers += etrangers(ligne, couleur);
    }
}

/// Combien de lignes, prises dans cet ordre, sont de la bande **ou de sa transition**.
///
/// `deja` est ce que ce bord a retiré aux passages précédents, `bord` la couleur de sa bande
/// s'il l'a trouvée et le compte de ce qu'il a retiré. Chaque ligne est, dans cet ordre :
///
/// * **de la bande** — au plus [`PART_ABERRANTE`] de ses pixels s'écartent de sa couleur de
///   plus que [`ECART_DE_BANDE`] ;
/// * **une transition** entre la bande et la ligne suivante ([`lisiere::est_une_transition`]) ;
/// * **l'ouverture de la bande**, au bord — la **première** ligne, prise comme médiane par
///   canal pour qu'un logo dans le coin ne la fausse pas — **ou derrière un filet**
///   ([`ouvre_une_bande`]) ;
/// * **une tache sur la marge**, que la bande enjambe ([`lisiere::enjamber`]) ;
///
/// et la première qui n'est rien de cela est le contenu — à moins qu'une **frange** n'y mène
/// encore ([`lisiere::frange`]). La couleur d'une bande se fige dès qu'elle est trouvée (BORDURES-3) :
/// seul un filet la cède à la bande qui le suit.
///
/// `bords` porte les quatre bords, `cote` celui qui balaie : la frange d'un bord sans marge se
/// juge sur la couleur de la marge d'en face, ou de n'importe quelle autre — une marge est une
/// seule feuille.
fn compter<F>(deja: u32, ordre: &[u32], bords: &mut [Bord; 4], cote: usize, mut ligne_de: F) -> u32
where
    F: FnMut(u32) -> Vec<Pixel>,
{
    let marge_ailleurs = bords[cote ^ 1]
        .couleur
        .or_else(|| (0..4).filter(|&i| i != cote).find_map(|i| bords[i].couleur));
    let bord = &mut bords[cote];
    let mut rang = 0usize;
    while let Some(&indice) = ordre.get(rang) {
        let courante = ligne_de(indice);
        // Une ligne vide vient d'un rectangle déjà réduit à rien : elle n'est pas une bande.
        if courante.is_empty() {
            break;
        }
        let suivante = ordre.get(rang + 1).map(|&i| ligne_de(i));
        let suivante = suivante.as_deref();
        let avance = match bord.couleur {
            Some(c) if est_de_la_bande(&courante, c) => {
                bord.retirer(&courante, c);
                1
            }
            Some(c) if suivante.is_some_and(|s| est_une_transition(&courante, s, c)) => 1,
            // Ce bord n'a retiré qu'une ligne, et celle-ci n'en est pas : peut-être un filet.
            Some(_) if deja as usize + rang == 1 => ouvrir(bord, &courante, suivante, true),
            None => ouvrir(bord, &courante, suivante, false),
            Some(c) => enjamber(bord, c, ordre[rang..].iter().map(|&i| ligne_de(i))),
        };
        if avance == 0 {
            break;
        }
        rang += avance;
    }
    if let (Some(couleur), Some(reste)) = (bord.couleur.or(marge_ailleurs), ordre.get(rang..)) {
        rang += frange(reste.iter().map(|&i| ligne_de(i)), couleur);
    }
    u32::try_from(rang).unwrap_or(u32::MAX)
}

/// Ouvre la bande sur cette ligne si elle en ouvre une : une ligne retirée, ou aucune.
fn ouvrir(
    bord: &mut Bord,
    ligne: &[Pixel],
    suivante: Option<&[Pixel]>,
    derriere_un_filet: bool,
) -> usize {
    let Some(couleur) = ouvre_une_bande(ligne, suivante, derriere_un_filet) else {
        return 0;
    };
    *bord = Bord {
        couleur: Some(couleur),
        ..Bord::default()
    };
    bord.retirer(ligne, couleur);
    1
}

/// **Cette ligne ouvre-t-elle une bande ?** Rend sa couleur si oui (BORDURES-5).
///
/// **Au bord**, il suffit qu'elle soit unie : au plus [`PART_ABERRANTE`] de ses pixels
/// s'écartent de sa médiane de plus que [`ECART_DE_BANDE`]. C'est le pari de FFmpeg — une
/// ligne unie qui touche le bord est une bande.
///
/// **Derrière un filet**, il faut de plus que la bande soit **épaisse** : la ligne d'après est
/// de sa couleur. Un filet se reconnaît à ce qu'il cache ; une ligne unie seule derrière lui
/// n'est pas une bande, et la retirer serait retirer une ligne de contenu. Le voile de la dune
/// — `241`, puis `218`, puis `192`, puis le ciel — perdait ainsi sa deuxième ligne.
fn ouvre_une_bande(
    ligne: &[Pixel],
    suivante: Option<&[Pixel]>,
    derriere_un_filet: bool,
) -> Option<Pixel> {
    let couleur = mediane(ligne);
    let unie = est_de_la_bande(ligne, couleur);
    let epaisse = !derriere_un_filet || suivante.is_some_and(|s| est_de_la_bande(s, couleur));
    (unie && epaisse).then_some(couleur)
}

/// L'écart entre deux pixels, sur leur canal le plus éloigné.
fn ecart(p: &Pixel, q: &Pixel) -> f64 {
    (0..3)
        .map(|c| f64::from(p[c].abs_diff(q[c])))
        .fold(0.0f64, f64::max)
}

/// La ligne `y`, sur les colonnes `xs`, copiée : les colonnes ne sont pas contiguës, et une
/// seule forme pour les deux vaut mieux que deux boucles.
fn ligne(image: &Vue<'_>, y: u32, xs: std::ops::Range<u32>) -> Vec<Pixel> {
    xs.map(|x| image.pixel(x, y)).collect()
}

/// La colonne `x`, sur les lignes `ys`, copiée.
fn colonne(image: &Vue<'_>, x: u32, ys: std::ops::Range<u32>) -> Vec<Pixel> {
    ys.map(|y| image.pixel(x, y)).collect()
}

/// La médiane de chaque canal, canal par canal.
///
/// Pas la moyenne : un filigrane blanc sur vingt pixels d'une bande noire de mille la
/// déplacerait de cinq niveaux, et la médiane, pas d'un.
fn mediane(pixels: &[Pixel]) -> Pixel {
    let mut sortie = [0u8; 4];
    let mut canal = Vec::with_capacity(pixels.len());
    for (c, valeur) in sortie.iter_mut().enumerate() {
        canal.clear();
        canal.extend(pixels.iter().map(|p| p[c]));
        canal.sort_unstable();
        *valeur = canal.get(canal.len() / 2).copied().unwrap_or(0);
    }
    sortie
}

/// **Cette ligne est-elle de la bande** : au plus [`PART_ABERRANTE`] de ses pixels s'écartent-ils
/// de sa couleur de plus que le bruit ?
///
/// Chaque pixel donne son écart sur son canal le plus éloigné. Quelques pixels d'encre ne
/// suffisent donc pas à arrêter une bande — ce qui laissait un liseré —, mais une ligne de
/// contenu, qui en porte bien davantage dès son premier rang, l'arrête toujours.
fn est_de_la_bande(pixels: &[Pixel], couleur: Pixel) -> bool {
    etrangers(pixels, couleur) <= tolere(pixels.len())
}

/// Combien de ces pixels s'écartent de cette couleur de plus que le bruit.
fn etrangers(pixels: &[Pixel], couleur: Pixel) -> usize {
    pixels
        .iter()
        .filter(|p| ecart(p, &couleur) > ECART_DE_BANDE)
        .count()
}

/// Combien de pixels étrangers la tolérance admet parmi `n` : le centième, arrondi en dessous.
fn tolere(n: usize) -> usize {
    ((n as f64) * PART_ABERRANTE).floor() as usize
}

/// **Au plus la part `part` de ces écarts dépasse-t-elle `seuil` ?** — la question qu'un centile
/// pose, sans le calculer.
///
/// « Le centile qui laisse un centième au-dessus de lui est sous le bruit » et « au plus un
/// centième des écarts dépassent le bruit » sont **la même proposition** : la valeur au rang
/// `n − 1 − ⌊n · part⌋` des écarts triés ne dépasse le seuil que si plus de `⌊n · part⌋`
/// valeurs le dépassent. La seconde forme se vérifie en comptant — un passage, aucun tri,
/// aucune allocation —, là où la première triait la ligne entière pour n'en lire qu'une valeur.
///
/// Une ligne courte ramène la tolérance à zéro : sur dix pixels, un centième n'existe pas, et
/// tolérer un pixel sur dix serait tolérer dix pour cent.
fn au_plus(part: f64, ecarts: impl Iterator<Item = f64>, seuil: f64) -> bool {
    let (mut n, mut au_dela) = (0usize, 0usize);
    for e in ecarts {
        n += 1;
        au_dela += usize::from(e > seuil);
    }
    au_dela <= ((n as f64) * part).floor() as usize
}

#[cfg(test)]
mod tests;
