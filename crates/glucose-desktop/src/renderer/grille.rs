//! La passe des images **par la grille de tuiles** — TUILE-1, branché au rendu.
//!
//! # Ce que cette passe remplace, et pourquoi
//!
//! La passe des images posait chaque photo à chaque image, depuis sa pyramide ou depuis une
//! vignette à la forme exacte. Sur le terrain : quatre cent vingt-neuf photos, `redessine
//! 100 %`, `report` à trente millisecondes — pour une scène qui, entre deux images, a bougé
//! de quatre pixels. Et le cache de vignettes, indexé par une phase sous-pixel, servait à
//! zéro pour cent.
//!
//! Une tuile est ancrée au **monde**, pas à l'écran : déplacer la vue ne change rien à ses
//! pixels, seulement à sa place. Ce qui coûtait trente millisecondes coûte alors une
//! composition — `bench_grille` mesure 0,18 ms à chaud, cinquante-deux fois moins.
//!
//! **C'est la variance du coût qui tombe, et c'est elle qui faisait le judder.** Une image à
//! six millisecondes suivie d'une à soixante-sept posait le contenu soixante pixels à côté de
//! sa trajectoire ; des images qui coûtent toutes la même chose le posent où il doit être.
//!
//! # Les trois régimes, et ce qui les décide
//!
//! L'échelle de la vue tombe entre deux niveaux dyadiques, et c'est elle qui décide :
//!
//! * **exacte** — l'échelle est une puissance de deux. Les tuiles se composent pixel pour
//!   pixel, au bit près identiques à un rendu direct (`bench_grille`) ;
//! * **au plus proche** — l'échelle est entre deux niveaux, et l'œil tolère qu'on abîme
//!   l'image ([`crate::perception`]). La tuile du niveau inférieur s'agrandit d'un facteur
//!   entre un et deux, au texel le plus proche : 1,7 ms pour un écran, contre dix fois plus
//!   en interpolant. Jamais le filtre lisse ici — il ne tient pas dans le budget ;
//! * **au plus proche, encore** — l'échelle est entre deux niveaux, l'œil ne tolère plus,
//!   mais **la vue bouge encore** : la fin d'un freinage. Repasser ici au rendu direct ferait
//!   sauter le contenu de soixante pixels, ce qui se voit infiniment plus qu'un agrandissement
//!   d'un facteur un virgule trois. C'est un choix de ressenti, écrit dans [`Regard`], et il
//!   se juge à l'écran ;
//! * **direct** — l'échelle est entre deux niveaux et plus rien ne bouge : l'arrêt. L'ancienne
//!   passe reprend la main, avec ses vignettes, et la finesse est intégrale. C'est le seul
//!   régime où le coût reste celui d'hier, et il ne se produit qu'une fois par arrêt si la
//!   salissure tient sa promesse.
//!
//! # Ce que la grille ne contient pas
//!
//! Rien de ce qui n'appartient pas au document : ni le cadre de sélection, ni les poignées,
//! ni la jauge de domaines, ni le carré d'une image encore en chemin. Les trois premiers se
//! dessinent par-dessus, en direct ; le dernier fait que la tuile n'est **pas gardée**, pour
//! qu'elle se repeigne dès que les octets arrivent.

use super::scale::WorldScale;
use super::scene::image::{draw_image_ornaments, draw_images, PasseImages};
use super::tuiles::{a_l_ecran, ce_que_porte, Portee, Tuiles, COTE_TUILE};
#[allow(unused_imports)]
use super::Regard;
use super::{Cadrage, PaintKit};
use crate::canvas::{screen_to_world, world_to_screen};
use crate::params::ViewPass;
use glucose_core::cout::Cout;
use glucose_core::occlusion::Boite;
use glucose_core::quadtree::{SpatialHash, Visibles};
use glucose_core::report::Filtre;
use glucose_core::store::Store;
use glucose_core::tuile::{Adresse, Empreinte};
use glucose_core::types::Viewport;
use tiny_skia::{Pixmap, PixmapMut};

/// Par quel chemin les images se posent sous cette vue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Regime {
    /// L'échelle est dyadique : les tuiles se composent pixel pour pixel.
    Exact,
    /// Entre deux niveaux, et l'œil tolère : la tuile s'agrandit, interpolée.
    Entre,
    /// Entre deux niveaux, et l'œil ne tolère rien : la passe directe.
    Direct,
}

impl Regime {
    /// Le régime que cette vue commande, sous ce cadrage.
    ///
    /// Une tuile ne se rend jamais par la grille — ce serait se rendre soi-même.
    ///
    /// # Une scène réduite passe par la grille, et c'est un changement
    ///
    /// Elle en était exclue, au motif qu'elle est « déjà une pixelisation pilotée par la
    /// perception ». Le raisonnement confondait deux choses : la grille n'est pas un moyen de
    /// dégrader, c'est un **cache**. En priver la scène réduite, c'est la faire repeindre
    /// entièrement à chaque image, précisément quand on cherchait à la rendre moins chère.
    ///
    /// Le coût de cette confusion se lit dans trois chroniques de terrain d'affilée :
    /// `agrandir` premier poste réel du zoom à 2,90 ms, `grille` absente de son profil, et
    /// **80 % des images rendues plus petites** — pendant que le cache existait, chaud, et ne
    /// servait pas. Pire, elle bouclait : réduire perdait les tuiles, la scène réduite coûtait
    /// donc presque autant qu'entière, le modèle en concluait qu'il fallait réduire encore.
    ///
    /// Rien ne s'y opposait techniquement. La réduction est déjà une **vue** : `cadrer`
    /// divise l'échelle et la translation par `f`, donc le niveau dyadique suit de lui-même
    /// et les tuiles se posent dans ce repère comme dans tout autre.
    pub(super) fn pour(cadrage: Cadrage, vp: Viewport) -> Self {
        if cadrage.vue.is_some() {
            return Self::Direct;
        }
        let niveau = Adresse::niveau_pour(vp.scale);
        let facteur = vp.scale / Adresse::echelle(niveau);
        if facteur == 1.0 {
            return Self::Exact;
        }
        // **Le mouvement, et non la tolerance de l'oeil.** La perception refuse de degrader
        // des que la vitesse passe sous celle de la poursuite -- donc sur toute la fin d'un
        // freinage, qui est precisement l'endroit ou repasser a trente millisecondes par
        // image ferait sauter le contenu de soixante pixels. Un agrandissement d'un facteur
        // un virgule trois se voit moins qu'un tel saut ; la finesse revient a l'arret.
        if cadrage.en_mouvement || cadrage.degradation_permise {
            return Self::Entre;
        }
        Self::Direct
    }
}

/// Ce que la passe par la grille a besoin d'emprunter au moteur, champ par champ.
///
/// Séparés parce que le moteur ne peut pas se prêter entier : rendre une tuile emprunte le
/// magasin et le modèle de coût en écriture pendant que le kit emprunte la typographie en
/// lecture. Le compilateur l'autorise sur des champs distincts, jamais à travers `&mut self`.
pub(super) struct Atelier<'a> {
    /// Le document : une tuile se rend depuis lui, comme n'importe quelle passe.
    pub store: &'a Store,
    pub magasin: &'a mut super::magasin::Magasin,
    pub cout: &'a mut Cout,
    pub tuiles: &'a mut Tuiles,
    pub index: &'a SpatialHash,
    pub kit: PaintKit<'a>,
}

/// Ce que la pose du fond a besoin d'emprunter, champ par champ.
///
/// Séparés parce que le moteur ne peut pas se prêter entier : la passe tient déjà son index
/// spatial en lecture, et `&mut self` serait refusé. C'est la même raison qui a fait naître
/// l'atelier de la grille.
pub(super) struct Fond<'a> {
    pub couverture: &'a mut Couverture,
    pub tuiles: &'a Tuiles,
    pub theme: &'a super::Theme,
}

/// Le fond du canevas et sa grille de points — **sauf si les tuiles vont tout recouvrir**,
/// auquel cas ces deux passes écrivent des pixels que personne ne lira jamais : un cinquième
/// d'une image sur 2560 × 1600.
///
/// C'est ici que l'écran se relève, une fois pour les deux questions qui en dépendent : « le
/// fond se verra-t-il ? », posée maintenant, et « que faut-il poser ? », posée par la passe
/// des images. Les séparer, c'était calculer les mêmes empreintes deux fois.
pub(super) fn poser_le_fond(
    fond: Fond<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
    cadrage: Cadrage,
    header_h: f32,
) {
    let (width, height) = (pixmap.width(), pixmap.height());
    relever(
        fond.couverture,
        fond.tuiles,
        store,
        pass,
        cadrage,
        (width as f32, height as f32),
    );
    let recouvert = fond.couverture.recouvre_tout();
    crate::perf::compteur("fond_saute", f64::from(u8::from(recouvert)));
    crate::perf::stage("releve");
    if recouvert {
        return;
    }
    pixmap.fill(fond.theme.bg_canvas);
    crate::perf::stage("clear");
    super::scene::grid::draw_grid(pixmap, &pass.vp, width, height, header_h);
    crate::perf::stage("grid");
}

/// Ce que l'écran porte en tuiles, **relevé une fois par image**.
///
/// # Pourquoi ce relevé existe, et pourquoi il est réutilisé
///
/// Deux questions ont besoin des mêmes empreintes : « le fond va-t-il se voir ? », qui se
/// pose **avant** d'effacer, et « que faut-il poser ? », qui se pose après. Les calculer deux
/// fois, c'est la géométrie calculée deux fois que la charte interdit — et c'est mesurable :
/// la vérification seule a fait monter le poste du fond de 0,98 à 1,11 ms, pour un gain nul
/// sur un document dont les photos ne pavent pas l'écran.
///
/// Le relevé est donc gardé d'une passe à l'autre, dans un tampon que le moteur réutilise :
/// aucune allocation par image (fiche 05 § 4.3).
#[derive(Default)]
pub(super) struct Couverture {
    /// Les tuiles de l'écran et ce que chacune porte, dans l'ordre où elles se poseront.
    tuiles: Vec<(Adresse, Empreinte)>,
    /// Le niveau dyadique de ce relevé, et le facteur d'agrandissement qui en découle.
    niveau: i32,
    /// Les tuiles relevées recouvrent-elles l'écran, sans laisser voir le fond ?
    recouvre_tout: bool,
}

impl Couverture {
    /// Un releve fabrique de toutes pieces, pour les tests.
    #[cfg(test)]
    pub(super) fn pour_test(tuiles: Vec<(Adresse, Empreinte)>, niveau: i32) -> Self {
        Self {
            tuiles,
            niveau,
            recouvre_tout: false,
        }
    }

    /// Les tuiles recouvrent-elles tout ce que le fond aurait rempli ?
    pub(super) fn recouvre_tout(&self) -> bool {
        self.recouvre_tout
    }
}

/// Relève ce que l'écran porte, et si cela suffit à cacher le fond.
///
/// # Pourquoi la réponse est exacte, et non prudente
///
/// Une tuile n'est comptée que si **son rendu est déjà là** — donc sa portée est connue, et
/// non supposée — et si cette portée est opaque **sur la tuile entière**. La pose d'une telle
/// tuile est alors un `Remplacer` sur toute sa surface, et les poses pavent l'écran sans trou
/// ni recouvrement, puisque le bord droit d'une tuile **est** le bord gauche de la suivante
/// (le partage des bords qui a supprimé les « croix noires »).
///
/// Une seule tuile manquante, transparente ou partielle, et la réponse est non : on efface,
/// comme hier. Il n'y a donc aucune zone à calculer, aucun bord à arrondir, et aucun pixel
/// périmé possible — le pire défaut qui soit, puisqu'il se voit sans qu'on sache d'où il
/// vient.
///
/// Ce que le test prouve, et c'est la bonne formulation : quand cette fonction dit oui,
/// **peindre sur un fond rouge ou sur un fond noir donne les mêmes pixels**.
pub(super) fn relever(
    releve: &mut Couverture,
    tuiles: &Tuiles,
    store: &Store,
    pass: ViewPass<'_>,
    cadrage: Cadrage,
    ecran: (f32, f32),
) {
    releve.tuiles.clear();
    releve.recouvre_tout = false;
    if Regime::pour(cadrage, pass.vp) == Regime::Direct {
        return;
    }
    let (niveau, adresses) = a_l_ecran(pass.vp, ecran);
    releve.niveau = niveau;
    let mut tout = true;
    for adresse in adresses {
        let empreinte = ce_que_porte(store, adresse, pass.visibles);
        // Opaque, et sur la tuile ENTIÈRE : une photo qui ne remplit pas sa tuile laisse
        // voir le fond autour d'elle.
        tout &= tuiles
            .portee_de(empreinte)
            .is_some_and(|p| p.opaque && p.boite == Some((0, 0, COTE_TUILE, COTE_TUILE)));
        releve.tuiles.push((adresse, empreinte));
    }
    releve.recouvre_tout = tout && !releve.tuiles.is_empty();
}

/// **Les tuiles de cet écran vont-elles le recouvrir entièrement ?**
///
/// # Pourquoi la question se pose avant d'effacer, et ce qu'elle épargne
///
/// Le fond se remplit d'une couleur unie, puis la grille de points s'y pose, puis les tuiles
/// se composent par-dessus. Sur un mur de photos, ces tuiles sont **opaques** et se posent
/// par `Melange::Remplacer` : chaque pixel du fond est écrasé sans jamais avoir été lu.
/// `bench_salissure` chiffre ce travail jeté à 1,1 ms par image sur 2560 × 1600 — un
/// cinquième de l'image, pour un fond que personne ne verra.
///
/// # Pourquoi c'est exact, et non prudent
///
/// Une tuile n'est comptée que si **son rendu est déjà là** — donc sa portée est connue, et
/// non supposée — et si cette portée est opaque **sur la tuile entière**. La pose d'une telle
/// tuile est alors un `Remplacer` sur toute sa surface, et les poses pavent l'écran sans trou
/// ni recouvrement, puisque le bord droit d'une tuile **est** le bord gauche de la suivante
/// (le partage des bords qui a supprimé les « croix noires »).
///
/// Une seule tuile manquante, transparente ou partielle, et la réponse est non : on efface,
/// comme hier. Il n'y a donc aucune zone à calculer, aucun bord à arrondir, et aucun pixel
/// périmé possible — le pire défaut qui soit, puisqu'il se voit sans qu'on sache d'où il
/// vient.
///
/// Ce que le test prouve, et c'est la bonne formulation : quand cette fonction dit oui,
/// **peindre sur un fond rouge ou sur un fond noir donne les mêmes pixels**.
/// Pose les images : par la grille quand la vue le permet, en direct sinon.
pub(super) fn poser_les_images(
    atelier: &mut Atelier<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
    cadrage: Cadrage,
    releve: &Couverture,
) {
    let regime = Regime::pour(cadrage, pass.vp);
    if regime == Regime::Direct {
        draw_images(
            atelier.magasin,
            atelier.cout,
            atelier.kit,
            pixmap,
            store,
            pass,
            PasseImages {
                degradation_permise: cadrage.degradation_permise,
                en_tuile: false,
            },
        );
        crate::perf::compteur("tuiles_peintes", 0.0);
        crate::perf::compteur("tuiles_reprises", 0.0);
        return;
    }
    let (peintes, reprises) = poser_par_la_grille(atelier, pixmap, store, pass, releve);
    crate::perf::compteur("tuiles_peintes", peintes as f64);
    crate::perf::compteur("tuiles_reprises", reprises as f64);
}

/// Pose les images de l'écran depuis la grille, et rend combien de tuiles ont été peintes et
/// reprises pendant cette image.
///
/// Le pixmap porte déjà le fond, la grille et les halos : les tuiles se **composent** dessus,
/// jamais ne le remplacent — une tuile est transparente partout où aucune photo ne passe.
fn poser_par_la_grille(
    atelier: &mut Atelier<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
    releve: &Couverture,
) -> (u64, u64) {
    let (largeur, hauteur) = (pixmap.width(), pixmap.height());
    let ecran = (largeur as f32, hauteur as f32);
    let clip = Boite::nouvelle(0.0, pass.header_h, ecran.0, ecran.1 - pass.header_h);
    let facteur = pass.vp.scale / Adresse::echelle(releve.niveau);
    let cote_ecran = f64::from(COTE_TUILE) * facteur;
    // **La grille interpole, toujours — et c'est ce qui retire a la pixelisation sa raison
    // d'etre.**
    //
    // Elle posait les tuiles au texel le plus proche des qu'on sortait d'une echelle
    // dyadique, parce que `bench_tuiles` mesurait 20,68 ms pour interpoler un ecran de
    // 2560 x 1600 contre 3,19 au plus proche. Ces vingt millisecondes n'etaient la limite de
    // rien : c'etait celle d'**un coeur sur seize**. Depuis que la composition se decoupe en
    // bandes, le meme ecran interpole coute 2,75 ms la ou le PIXELISE en coutait 8,25 avant
    // cette session -- plus beau ET plus rapide, ce qui ne laisse rien a arbitrer.
    //
    // Sur une machine qui n'a pas de coeurs a donner, le mecanisme reste entier : la finesse
    // repond au budget par `Cadrage::degradation_permise`, en amont, et c'est la qu'une
    // machine lente se fera entendre -- par ce qu'elle MESURE, pas par un materiel suppose.
    let filtre = Filtre::Lisse;

    // Ce qui est à l'écran reste hors d'atteinte de l'éviction, même quand aucune photo n'est
    // posée par la passe directe : sans cela, le magasin rendrait à la machine des images
    // qu'il faudrait redécoder à la première tuile invalidée.
    if let Some(board) = store.active_board() {
        for img in Visibles::nouvelles(pass.visibles, board).images() {
            if let Some(src) = img.src.as_deref() {
                atelier.magasin.reclamer(src, img.width);
            }
        }
    }

    let (peintes_avant, reprises_avant) = (atelier.tuiles.peintes(), atelier.tuiles.reprises());
    atelier.tuiles.ouvrir();
    let mut pixels: u64 = 0;
    // **Les postes se ferment l'un l'autre, et chacun porte le nom de ce qu'il mesure.** La
    // première version marquait « empreintes » avant la boucle et « grille » après : le
    // premier mesurait la réclamation des photos, le second absorbait les empreintes ET la
    // composition de toutes les tuiles -- 3,5 ms sur 429 photos, sans dire lesquelles.
    crate::perf::stage("reclamer");

    // **Peindre, puis composer — et non peindre-et-composer tuile par tuile.**
    //
    // Les deux travaux ne demandent pas la même chose du cache : peindre l'ÉCRIT, composer
    // le LIT. Tant qu'ils étaient entrelacés, la boucle entière tenait le cache en écriture,
    // et un seul fil pouvait y toucher. Séparés, la seconde moitié ne lit plus que des
    // pixels déjà là — et seize fils peuvent la faire ensemble (voir `composer_en_bandes`).
    //
    // Les empreintes viennent du relevé : elles ont déjà été lues avant le fond, et les
    // recalculer serait la géométrie calculée deux fois que la charte interdit.
    let provisoires = peindre_ce_qui_manque(atelier, releve);
    crate::perf::stage("tuile");

    let modele = Place {
        adresse: Adresse::contenant(releve.niveau, 0.0, 0.0),
        vp: pass.vp,
        cote_ecran,
        clip,
        filtre,
    };
    pixels += composer_en_bandes(pixmap, atelier.tuiles, releve, &provisoires, modele);
    atelier.tuiles.fermer();
    crate::perf::stage("grille");
    // Les ornements ne se dessinent plus ici : ils passent au-dessus de TOUT, apres les
    // annotations, sur les deux voies (ORNEMENTS-1). Les laisser ici aussi les composait deux
    // fois -- un trait de selection a 197 au lieu de 137, et c'est l'epreuve des deux voies
    // qui l'a vu.
    noter_ce_que_l_ecran_a_recu(pixmap, store, pass, pixels);
    (
        atelier.tuiles.peintes() - peintes_avant,
        atelier.tuiles.reprises() - reprises_avant,
    )
}

/// Peint les tuiles que le cache n'a pas encore, et rend celles qu'il ne faut **pas** garder.
///
/// Un seul fil : c'est la moitié du travail qui écrit dans le cache. Elle marque aussi, par
/// [`Tuiles::deja_peinte`], que chaque tuile a servi à cette image — ce qui borne le cache à
/// ce que l'écran demande, et doit donc se faire une fois, ici, et pas dans chaque bande.
fn peindre_ce_qui_manque(
    atelier: &mut Atelier<'_>,
    releve: &Couverture,
) -> Vec<(Adresse, Pixmap, Portee)> {
    let mut provisoires = Vec::new();
    for (adresse, empreinte) in &releve.tuiles {
        // Une tuile vide n'a rien à peindre : c'est le cas le plus fréquent d'un canevas
        // infini, et c'est lui qui rend le déplacement sur du vide gratuit.
        if *empreinte == Empreinte::vide() || atelier.tuiles.deja_peinte(*empreinte).is_some() {
            continue;
        }
        let Some((peinte, complete)) = rendre_une_tuile(atelier, *adresse) else {
            continue;
        };
        // Une tuile dont une photo manquait encore ne se garde pas : elle se repeindra à
        // l'image où les octets seront là, et le cadre « en chemin » n'aura pas survécu.
        if complete {
            atelier.tuiles.ranger(*empreinte, peinte);
        } else {
            let portee = Portee::de(&peinte);
            provisoires.push((*adresse, peinte, portee));
        }
    }
    provisoires
}

/// Rend une tuile dans son propre repère : les photos qui la traversent, et rien d'autre.
///
/// Rend aussi si **toutes** ces photos étaient décodées. Sinon, la tuile montre un cadre
/// en chemin, et l'appelant ne doit pas la garder.
fn rendre_une_tuile(atelier: &mut Atelier<'_>, adresse: Adresse) -> Option<(Pixmap, bool)> {
    let store = atelier.store;
    let cote = COTE_TUILE;
    let mut pixmap = Pixmap::new(cote, cote)?;
    let couverte = adresse.couvre();
    let cadrage = Cadrage::tuile(adresse.niveau, (couverte.left, couverte.top));
    let vp = cadrage.vue?;
    let (min_wx, min_wy) = screen_to_world(0.0, 0.0, &vp);
    let (max_wx, max_wy) = screen_to_world(f64::from(cote), f64::from(cote), &vp);
    let rangs = atelier
        .index
        .query_rect_ranks(min_wx, min_wy, max_wx, max_wy, 200.0);
    let pass = ViewPass {
        vp,
        visibles: &rangs,
        index: atelier.index,
        header_h: 0.0,
    };
    let complete = draw_images(
        atelier.magasin,
        atelier.cout,
        atelier.kit,
        &mut pixmap.as_mut(),
        store,
        pass,
        PasseImages {
            degradation_permise: false,
            en_tuile: true,
        },
    );
    Some((pixmap, complete))
}

/// Où et comment une tuile se pose à l'écran.
#[derive(Clone, Copy)]
struct Place {
    adresse: Adresse,
    vp: Viewport,
    cote_ecran: f64,
    clip: Boite,
    filtre: Filtre,
}

/// Ce qui n'appartient pas au document, par-dessus les tuiles : cadre de sélection,
/// poignées, jauge de domaines.
///
/// # Pourquoi la couche du DESSUS doit l'appeler elle-meme
///
/// Ils vivaient au bout de `poser_par_la_grille`, donc ils disparaissaient avec elle des que
/// la carte posait les photos : l'utilisateur voyait ses photos, mais plus une seule poignee
/// pour les redimensionner. Ils n'appartiennent pas a la pose des photos -- ils appartiennent
/// a ce qui se dessine PAR-DESSUS, et c'est la couche du dessus qui les porte.
pub(super) fn dessiner_les_ornements(
    kit: PaintKit<'_>,
    pixmap: &mut PixmapMut,
    store: &Store,
    pass: ViewPass<'_>,
) {
    let Some(board) = store.active_board() else {
        return;
    };
    let images: Vec<_> = Visibles::nouvelles(pass.visibles, board)
        .images()
        .map(|img| {
            let (wx, wy) =
                world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &pass.vp);
            let sw = (img.width * pass.vp.scale) as f32;
            let sh = (img.height * pass.vp.scale) as f32;
            (img, (wx as f32, wy as f32, sw, sh))
        })
        .collect();
    draw_image_ornaments(kit, pixmap, store, WorldScale::new(pass.vp.scale), &images);
}

/// Les compteurs de la chronique, tels que l'**écran** les vit — et non la dernière tuile.
///
/// Chaque tuile rendue passe par la passe directe, qui déclare ses propres compteurs : sans
/// cette reprise, la trace dirait que l'image a posé deux photos alors que l'écran en montre
/// quatre cents.
fn noter_ce_que_l_ecran_a_recu(pixmap: &PixmapMut, store: &Store, pass: ViewPass<'_>, pixels: u64) {
    let fenetre = f64::from(pixmap.width()) * f64::from(pixmap.height());
    let posees = store.active_board().map_or(0, |b| {
        Visibles::nouvelles(pass.visibles, b).images().count()
    });
    crate::perf::compteur("img_n", posees as f64);
    crate::perf::compteur("img_ecrans", pixels as f64 / fenetre.max(1.0));
    crate::perf::compteur("img_par_vignette", 0.0);
    crate::perf::compteur("img_pixelise", 0.0);
}

mod bandes;

use bandes::composer_en_bandes;

#[cfg(test)]
mod tests;
