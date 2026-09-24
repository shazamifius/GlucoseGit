//! **La lisière** : ce qui sépare la bande du contenu quand ils ne se touchent pas d'un trait.
//!
//! Sortie de [`super`] quand celui-ci a dépassé sa taille admise, et le découpage n'est pas
//! arbitraire : le parent dit ce qu'est une bande et comment les quatre bords la balaient ;
//! ce module dit ce qui la **prolonge** au-delà de sa dernière ligne unie. Trois formes, trois
//! fonctions, toutes jugées sur les deux grandeurs du parent — [`super::ECART_DE_BANDE`] et
//! [`super::PART_ABERRANTE`] — et sur la majorité :
//!
//! * le **fondu** d'un rang, mélange de la bande et du rang suivant (BORDURES-3 et 4,
//!   [`est_une_transition`]) ;
//! * une **tache** sur la marge, que la bande enjambe (BORDURES-6, [`enjamber`]) ;
//! * la **frange** d'un bord de plaque, où la marge se retire sur quelques lignes
//!   (BORDURES-7, [`frange`]).
//!
//! # BORDURES-6 — une tache sur la marge ne l'arrête pas
//!
//! Sa gravure du 24/09, 736 × 828 : une plaque sur une marge de papier. `Ctrl+B` retirait 49,
//! 51 et 51 pixels à gauche, en haut et en bas — et **11** à droite, sur 56. À quinze pixels du
//! bord, une petite tache sur le papier, invisible à l'œil, vingt-cinq à quarante niveaux sous
//! le blanc : chaque colonne qui la traversait en portait un peu plus d'un centième, et la
//! tolérance, jugée ligne par ligne, y voyait le début du contenu. Deux autres, plus loin,
//! auraient arrêté le balayage à leur tour.
//!
//! La tolérance du centième se juge désormais **aussi sur la bande entière** : une tache
//! s'enjambe si la marge reprend derrière elle, si tout ce qu'on retire garde au plus un
//! centième de pixels étrangers, et si la marge s'arrête sur **un bord net** — une ligne dont
//! la majorité des pixels la quittent, atteinte par des lignes qui en portent chacune plus que
//! la précédente, comme le fait un bord droit, même penché ([`enjamber`]).
//!
//! **Le bord net a été imposé par ses images.** Sans lui, la même règle changeait 48 de ses
//! 286 images, et la plupart en perdant du contenu : le texte sous une affiche, le titre
//! manuscrit d'un dessin, tout un pan sombre d'une peinture de marais, les étoiles autour d'un
//! arbre fractal. Une zone sombre parsemée de détails ressemble trait pour trait à une marge
//! tachée ; ce qui les sépare est la façon dont elle finit. Avec lui, **deux** images changent
//! — sa gravure, et une tache rose au-dessus d'une plante, retirée juste au-dessus de la pointe
//! de sa tige —, et les deux sont justes.
//!
//! # BORDURES-7 — la frange d'un bord de plaque
//!
//! Ses deux gravures, après BORDURES-6 : *« le problème, c'est qu'elles possèdent encore des
//! contours »*. Un fin liseré clair restait le long de leurs bords. Une plaque tirée à la main
//! n'est jamais tout à fait droite : elle penche, elle ondule, l'encre bave, et la marge s'y
//! retire sur quelques lignes au lieu d'une — la part de papier tombe de 95 à 90, 81, 63, puis
//! 0,5 %. La coupe s'arrêtait à la première de ces lignes. Sur la seconde, recadrée de travers
//! avant de lui parvenir, le haut et la droite n'avaient même plus de bande : leur première
//! ligne était déjà en partie de la plaque.
//!
//! Ces lignes forment une **frange** ([`frange`]) : le papier y recule de plus d'un centième à
//! chaque ligne, sur au plus un centième de la longueur d'une ligne, jusqu'à un contenu en
//! majorité sans papier. Un bord sans bande juge son papier sur la marge d'en face — une marge
//! est une seule feuille. Sur ses 308 images, 31 changent, de 1 à 9 pixels, et chaque bord
//! changé a été regardé : des franges de papier, le flou d'une bande noire sur l'image, la ligne
//! claire d'un bord de scan.
//!

use super::{au_plus, ecart, etrangers, tolere, Bord, Pixel, ECART_DE_BANDE, PART_ABERRANTE};

/// **La frange d'un bord** : ce qui reste de marge quand le contenu ne commence pas d'un coup
/// (BORDURES-7). Rend combien de lignes elle compte, ou zéro.
///
/// Un bord de plaque n'est jamais tout à fait droit : il penche, il ondule, l'encre bave, et
/// la marge s'y retire sur quelques lignes, pas en une. Sur ses deux gravures, la part de
/// papier tombe ainsi de 95 à 90, 81, 63, puis 0,5 % ; de 85 à 73, 24, 4, puis 2,7 %. Ces
/// lignes-là forment une frange, et partent avec la marge, si :
///
/// * chacune porte **plus d'un centième de papier de moins** que la précédente — le bord
///   avance, il ne traîne pas ;
/// * elles tiennent en **au plus un centième** de la longueur d'une ligne — un bord penché
///   d'un centième. Un dégradé, le sommet arrondi d'un sujet s'étendent sur bien davantage ;
/// * elles débouchent sur une ligne **en majorité sans papier** : le contenu. Le haut d'un
///   objet qui ne remplit pas la ligne s'élargit vite puis se stabilise, papier en majorité,
///   et reste.
///
/// La première ligne du contenu est celle d'où le papier ne tombe plus : son reste est ce que
/// le contenu porte lui-même de blanc.
pub(super) fn frange(mut lignes: impl Iterator<Item = Vec<Pixel>>, couleur: Pixel) -> usize {
    let Some(premiere) = lignes.next() else {
        return 0;
    };
    let n = premiere.len();
    let papier = |ligne: &[Pixel]| ligne.len() - etrangers(ligne, couleur);
    let mut restant = papier(&premiere);
    if restant <= tolere(n) {
        return 0;
    }
    let mut profondeur = 0usize;
    loop {
        let Some(ligne) = lignes.next() else {
            return 0;
        };
        let suivant = papier(&ligne);
        if restant.saturating_sub(suivant) <= tolere(n) {
            break;
        }
        profondeur += 1;
        restant = suivant;
        if profondeur > tolere(n) {
            return 0;
        }
    }
    if profondeur > 0 && 2 * restant <= n {
        profondeur
    } else {
        0
    }
}

/// **Une tache sur la marge s'enjambe-t-elle ?** Rend combien de lignes retirer — jusqu'à la
/// dernière où la marge reprend —, ou zéro (BORDURES-6).
///
/// `lignes` part de la première ligne qui n'est pas de la bande. Trois conditions, et aucune
/// ne demande un nombre nouveau :
///
/// * **la marge reprend** derrière la tache, de sa couleur ;
/// * **ce qu'on retire reste une marge** : au plus [`PART_ABERRANTE`] de tous ses pixels —
///   bande et taches ensemble — s'écartent de sa couleur. Un titre, une rangée de texte, un
///   objet pèsent bien davantage, et arrêtent le balayage comme avant ;
/// * **la marge s'arrête sur un bord net** : une ligne dont la **majorité** des pixels la
///   quittent, atteinte depuis la dernière reprise par des lignes qui en portent chacune
///   **plus** que la précédente. C'est ce que fait un bord droit, même penché — celui d'une
///   plaque, d'une photo posée sur une page. Le bord d'une peinture qui s'éclaircit ou d'une
///   aquarelle monte et descend : sur ses images, c'est ce qui séparait les vrais cas des
///   faux.
pub(super) fn enjamber(
    bord: &mut Bord,
    couleur: Pixel,
    lignes: impl Iterator<Item = Vec<Pixel>>,
) -> usize {
    let (mut pixels, mut etrangers_vus) = (bord.pixels, bord.etrangers);
    // La dernière reprise de la marge : combien de lignes jusqu'à elle, et le compte d'alors.
    let mut reprise: Option<(usize, usize, usize)> = None;
    // Depuis la dernière reprise : les écarts de la ligne précédente, et s'ils montent.
    let (mut precedente, mut monte) = (None::<usize>, true);
    for (k, ligne) in lignes.enumerate() {
        let (n, e) = (ligne.len(), etrangers(&ligne, couleur));
        if n == 0 {
            return 0;
        }
        if 2 * e > n {
            let Some((lignes_retirees, p, et)) = reprise.filter(|_| monte) else {
                return 0;
            };
            (bord.pixels, bord.etrangers) = (p, et);
            return lignes_retirees;
        }
        pixels += n;
        etrangers_vus += e;
        if e <= tolere(n) {
            // La marge reprend -- à condition que les taches n'en fassent pas autre chose.
            if etrangers_vus > tolere(pixels) {
                return 0;
            }
            reprise = Some((k + 1, pixels, etrangers_vus));
            (precedente, monte) = (None, true);
        } else {
            monte = monte && precedente.is_none_or(|p| e > p);
            precedente = Some(e);
            // Ni une rampe vers un bord net, ni une tache qu'une ligne propre rattraperait : la
            // reprise suivante la refuserait. S'arrêter ici ne change rien au verdict, et c'est
            // ce qui borne le balayage à ce que coûtent les bandes.
            if !monte && etrangers_vus > tolere(pixels + n) {
                return 0;
            }
        }
    }
    0
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
pub(super) fn est_une_transition(rang: &[Pixel], suivant: &[Pixel], bande: Pixel) -> bool {
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
