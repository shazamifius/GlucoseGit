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
//! # Ce qui trompe, et qu'on ne cache pas
//!
//! Une photo dont le **contenu** commence par une zone unie — un ciel sans nuage en haut, un
//! mur blanc à gauche — se fera rogner de cette zone. FFmpeg a le même défaut et le
//! documente. Il n'y a pas de critère local qui distingue un ciel uni d'une bande, parce que
//! ce sont les mêmes pixels ; seule l'intention les sépare. L'action est donc **explicite et
//! annulable** — un geste, un `Ctrl+Z` — et jamais automatique.
//!
//! Et une image entièrement unie proposerait de tout retirer : c'est l'invariant du
//! [`Recadrage`] qui garantit alors qu'il reste quelque chose, pas ce module.

use crate::report::{Pixel, Vue};
use crate::types::Recadrage;

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
/// l'image : le balayage s'arrête à la première ligne qui n'en est plus, donc une photo sans
/// bordure ne coûte que quatre lignes.
pub fn detecter(image: &Vue<'_>) -> Recadrage {
    let (l, h) = (image.largeur(), image.hauteur());
    // Le rectangle qui reste : [x0, x1[ × [y0, y1[. Il ne peut que rétrécir.
    let (mut x0, mut y0, mut x1, mut y1) = (0u32, 0u32, l, h);
    // La couleur de la bande de chaque bord -- haut, bas, gauche, droite --, figée dès qu'il
    // l'a trouvée (BORDURES-3).
    let mut bandes: [Option<Pixel>; 4] = [None; 4];
    loop {
        let avant = (x0, y0, x1, y1);
        y0 += compter((y0..y1).collect(), &mut bandes[0], |y| {
            ligne(image, y, x0..x1)
        });
        y1 -= compter((y0..y1).rev().collect(), &mut bandes[1], |y| {
            ligne(image, y, x0..x1)
        });
        x0 += compter((x0..x1).collect(), &mut bandes[2], |x| {
            colonne(image, x, y0..y1)
        });
        x1 -= compter((x0..x1).rev().collect(), &mut bandes[3], |x| {
            colonne(image, x, y0..y1)
        });
        if (x0, y0, x1, y1) == avant {
            break;
        }
    }
    Recadrage::depuis_les_marges(
        f64::from(x0) / f64::from(l),
        f64::from(y0) / f64::from(h),
        f64::from(l - x1) / f64::from(l),
        f64::from(h - y1) / f64::from(h),
    )
}

/// Combien de lignes, prises dans cet ordre, sont de la bande **ou de sa transition**.
///
/// La couleur de la bande est `bande` si ce bord l'a déjà trouvée, sinon celle de la
/// **première** ligne parcourue — le bord —, prise comme médiane par canal pour qu'un logo
/// dans le coin ne la fausse pas. Elle se fige dès que le bord a trouvé sa bande
/// (BORDURES-3). Une ligne est de la bande si au plus [`PART_ABERRANTE`] de ses pixels
/// s'écartent de cette couleur de plus que [`ECART_DE_BANDE`] ; la première qui n'en est
/// plus, et les suivantes, partent encore tant qu'elles sont des transitions.
fn compter<F>(ordre: Vec<u32>, bande: &mut Option<Pixel>, mut ligne_de: F) -> u32
where
    F: FnMut(u32) -> Vec<Pixel>,
{
    let Some(&premier) = ordre.first() else {
        return 0;
    };
    let mut courante = ligne_de(premier);
    if courante.is_empty() {
        return 0;
    }
    let couleur = bande.unwrap_or_else(|| mediane(&courante));
    let mut rang = 0usize;
    while est_de_la_bande(&courante, couleur) {
        rang += 1;
        let Some(&suivant) = ordre.get(rang) else {
            break;
        };
        courante = ligne_de(suivant);
    }
    if rang > 0 {
        bande.get_or_insert(couleur);
    }
    // Une transition n'existe qu'entre une bande et un contenu : sans bande, rien a fondre.
    if bande.is_some() {
        while let Some(&suivant) = ordre.get(rang + 1) {
            let apres = ligne_de(suivant);
            if !est_une_transition(&courante, &apres, couleur) {
                break;
            }
            rang += 1;
            courante = apres;
        }
    }
    u32::try_from(rang).unwrap_or(u32::MAX)
}

/// **Ce rang est-il un mélange de la bande et du rang suivant** (BORDURES-3, BORDURES-4) ?
///
/// Chaque pixel se lit comme `a · bande + (1 − a) · son voisin du rang suivant`, avec **son
/// propre** `a` : la projection du pixel sur le segment qui va du voisin à la bande. Le rang
/// est une transition si deux choses sont vraies à la fois.
///
/// **Le mélange l'explique** : au plus [`PART_ABERRANTE`] de ses pixels s'écartent du leur de
/// plus que le bruit. Un pixel plus sombre que son voisin, vers une bande blanche, n'est pas
/// un mélange — et un contenu texturé en porte toujours bien davantage.
///
/// **Le pixel typique a bougé vers la bande**, et il ne se cherche que parmi ceux qui
/// **peuvent** montrer un fondu : les pixels dont le voisin n'est pas déjà de la couleur de la
/// bande. Un fond blanc sous une bande blanche est identique fondu ou non ; un contenu presque
/// noir sous une bande noire aussi. Il faut que ces pixels-là soient la **majorité** — sinon le
/// rang est surtout du fond sur du fond, comme un texte ou la pointe d'un objet posé sur la
/// marge, et rien ne permet d'y voir un fondu —, et que leur **médiane** ait bougé au-delà du
/// bruit : un dégradé doux, qui change de quelques niveaux par rang, n'en est pas un.
///
/// # Ce que les images de l'utilisateur ont appris (BORDURES-4)
///
/// BORDURES-3 prenait un `a` **commun** à toute la ligne, au motif qu'un flou fond la ligne
/// entière de la même façon, et jugeait le déplacement sur un centième des pixels. Les
/// quarante-neuf images de ses deux documents l'ont démenti deux fois :
///
/// * **le bord d'une peinture ondule.** Le premier rang gardé de sa forêt mêlait 79 à 90 % de
///   blanc selon l'endroit, et le dernier portait deux coups de pinceau débordant sur la
///   marge : le mélange uniforme n'en expliquait que 94 %, et **le liseré restait** ;
/// * **un centième ne fait pas un fondu.** Sur sept illustrations posées sur un fond de la
///   couleur de la marge, les rangs où seule la pointe d'un objet paraît passaient pour des
///   transitions, et **un à trois rangs d'objet** étaient rognés.
///
/// Et la majorité se compte **parmi les pixels qui peuvent montrer un fondu**, pas parmi tous :
/// la colonne gauche d'une peinture sombre sur bande noire — sa luminosité vaut 58 % de celle
/// de sa voisine, uniformément — porte 28 % de pixels presque noirs où aucun fondu ne se voit.
/// Comptée sur tous les pixels, la médiane la gardait, en liseré sombre.
fn est_une_transition(rang: &[Pixel], suivant: &[Pixel], bande: Pixel) -> bool {
    if rang.len() != suivant.len() || rang.is_empty() {
        return false;
    }
    let canal = |p: &Pixel, c: usize| f64::from(p[c]);
    let ecart_au_melange = rang.iter().zip(suivant).map(|(p, q)| {
        let (mut num, mut den) = (0.0f64, 0.0f64);
        for c in 0..3 {
            let vers_la_bande = canal(&bande, c) - canal(q, c);
            num += (canal(p, c) - canal(q, c)) * vers_la_bande;
            den += vers_la_bande * vers_la_bande;
        }
        // Un voisin qui EST la bande n'offre qu'un point : le mélange vaut alors ce point.
        let a = if den > 0.0 {
            (num / den).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (0..3)
            .map(|c| (canal(p, c) - (a * canal(&bande, c) + (1.0 - a) * canal(q, c))).abs())
            .fold(0.0f64, f64::max)
    });
    // Les pixels qui PEUVENT montrer un fondu, et ceux d'entre eux qui ont bougé au-delà du
    // bruit. Leur médiane a bougé quand ils sont plus de la moitié à l'avoir fait.
    let (mut informatifs, mut bouges) = (0usize, 0usize);
    for (p, q) in rang.iter().zip(suivant) {
        if ecart(q, &bande) > ECART_DE_BANDE {
            informatifs += 1;
            bouges += usize::from(ecart(p, q) > ECART_DE_BANDE);
        }
    }
    informatifs * 2 > rang.len()
        && bouges > informatifs / 2
        && au_plus(PART_ABERRANTE, ecart_au_melange, ECART_DE_BANDE)
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
    au_plus(
        PART_ABERRANTE,
        pixels.iter().map(|p| ecart(p, &couleur)),
        ECART_DE_BANDE,
    )
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
