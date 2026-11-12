//! Ce que les cartes texturées garantissent : rendues hors contexte, elles donnent les mêmes
//! pixels qu'en place.

use super::*;

/// Le palier dyadique le plus proche l'est au sens du logarithme : entre deux puissances de
/// deux, la frontière est leur moyenne géométrique.
#[test]
fn test_le_palier_dyadique_est_le_plus_proche_au_sens_du_logarithme() {
    assert_eq!(palier_dyadique(1.0), 1.0);
    assert_eq!(palier_dyadique(2.0), 2.0);
    assert_eq!(palier_dyadique(0.5), 0.5);
    // racine de deux est la frontiere : juste en dessous, un ; juste au-dessus, deux.
    assert_eq!(palier_dyadique(1.40), 1.0);
    assert_eq!(palier_dyadique(1.42), 2.0);
    assert_eq!(palier_dyadique(0.72), 1.0);
    assert_eq!(palier_dyadique(0.70), 0.5);
    assert_eq!(
        palier_dyadique(0.0),
        1.0,
        "une echelle nulle ne fait pas paniquer"
    );
    assert_eq!(palier_dyadique(f64::NAN), 1.0);
}

use crate::renderer::card::TextCard;
use crate::renderer::Renderer;
use glucose_core::types::BoardImage;
use tiny_skia::Pixmap;

/// Le composant d'une photo en chemin, rendu à part et reposé sur un pixel entier, donne les
/// pixels que le processeur écrit en place — au bit près, puisque c'est le même code à la
/// même phase.
fn ecart_photo_en_chemin(x: f64, y: f64, rotation: f64) -> (u8, usize) {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let mut img = BoardImage::new("p".to_string(), x, y, 180.0, 120.0);
    img.rotation = rotation;
    let vp = Viewport {
        scale: 1.0,
        x: 20.5,
        y: 10.25,
    };
    let taille = (400u32, 300u32);
    let regime = Regime::de((vp, 1.0), Regard::immobile(), taille, 0.0);

    // En place, comme la voie processeur.
    let mut en_place = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let (sx, sy) = world_to_screen(img.x - img.width / 2.0, img.y - img.height / 2.0, &vp);
    draw_missing_image(
        kit.typography,
        kit.theme,
        &mut en_place.as_mut(),
        (sx as f32, sy as f32),
        (
            (img.width * vp.scale) as f32,
            (img.height * vp.scale) as f32,
        ),
        &img.id,
        img.rotation,
    );

    // A part, puis repose la ou la pose le dit.
    let composant = regime
        .photo_en_chemin(kit, &img)
        .expect("un composant")
        .seule();
    let texture = composant.rendre(kit).expect("une texture");
    let mut reposee = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let (px, py) = (composant.pose.x, composant.pose.y);
    assert_eq!(px, px.floor(), "a l'arret, la pose est entiere");
    assert_eq!(py, py.floor(), "a l'arret, la pose est entiere");
    crate::composition::poser(
        &mut reposee.as_mut(),
        &texture,
        (px, py),
        glucose_core::report::Melange::Composer,
    );

    let pire = en_place
        .data()
        .iter()
        .zip(reposee.data())
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap_or(0);
    let canaux = en_place
        .data()
        .iter()
        .zip(reposee.data())
        .filter(|(a, b)| a.abs_diff(**b) > 1)
        .count();
    (pire, canaux)
}

#[test]
fn test_une_photo_en_chemin_droite_se_repose_au_bit_pres() {
    let (pire, canaux) = ecart_photo_en_chemin(150.3, 100.7, 0.0);
    assert!(
        pire <= 1 && canaux == 0,
        "photo droite : pire {pire}, {canaux} canaux au-dela de 1"
    );
}

#[test]
fn test_une_photo_en_chemin_penchee_se_repose_au_bit_pres() {
    let (pire, canaux) = ecart_photo_en_chemin(150.3, 100.7, std::f64::consts::FRAC_PI_8);
    assert!(
        pire <= 1 && canaux == 0,
        "photo penchee : pire {pire}, {canaux} canaux au-dela de 1"
    );
}

/// Une saisie en cours sur cette carte, avec le curseur a cet offset et cette phase.
fn saisie(corps: &str, tete: usize, curseur_visible: bool) -> crate::renderer::TextEditSession {
    crate::renderer::TextEditSession {
        ann_id: "c".to_string(),
        buffer: corps.to_string(),
        selection: glucose_core::text::Selection::at(tete),
        goal_x: None,
        blink_timer: std::time::Instant::now(),
        curseur_visible,
    }
}

/// Une vue et un regime communs aux epreuves de la saisie.
fn regime_temoin() -> (Viewport, Regime) {
    let vp = Viewport {
        scale: 1.0,
        x: 20.5,
        y: 10.25,
    };
    (
        vp,
        Regime::de((vp, 1.0), Regard::immobile(), (500, 300), 0.0),
    )
}

/// Un composant de carte, rendu à part et reposé sous ses ornements, comparé à la carte
/// entière dessinée en place par le processeur : les deux voies.
fn ecart_carte(selectionnee: bool, densite: f32) -> (u8, usize) {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let vp = Viewport {
        scale: 1.0,
        x: 20.5,
        y: 10.25,
    };
    let taille = (500u32, 300u32);
    let regime = Regime::de((vp, densite), Regard::immobile(), taille, 0.0);
    let (x, y, w, h) = (60.3, 40.7, 240.0f32, 60.0f32);
    let corps = "Accents : éàçùôêîï — « guillemets »
Et un lien.";
    let teinte = (96, 165, 250);
    let carte = || TextCard {
        origin: (x, y),
        size: (w, h),
        body: corps,
        tint: teinte,
        fond: crate::renderer::card::papier(&crate::theme::Theme::dark()),
        eclaires: None,
        selected: selectionnee,
        editing: None,
    };

    let mut en_place = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        mises_en_page: kit.mises_en_page,
        tints: kit.tints,
        theme: kit.theme,
        vp,
        scale: WorldScale::new(vp.scale, densite),
        clip: Clip {
            width: taille.0 as f32,
            height: taille.1 as f32,
            top: 0.0,
        },
        poignees: true,
    };
    crate::renderer::card::draw_text_card(&ctx, &mut en_place.as_mut(), carte());
    let composant = regime
        .carte(kit, "c", (x, y, w, h), (corps, teinte), (None, None))
        .expect("un composant")
        .seule();
    let texture = composant.rendre(kit).expect("une texture");
    let mut reposee = Pixmap::new(taille.0, taille.1).expect("pixmap");
    crate::composition::poser(
        &mut reposee.as_mut(),
        &texture,
        (composant.pose.x, composant.pose.y),
        glucose_core::report::Melange::Composer,
    );
    crate::renderer::card::draw_card_ornements(&ctx, &mut reposee.as_mut(), carte());
    ecart(&en_place, &reposee)
}

/// **Une carte au repos se repose au bit près** : même code, même mise en page, même phase.
#[test]
fn test_une_carte_se_repose_au_bit_pres() {
    assert_eq!(ecart_carte(false, 1.0), (0, 0), "carte au repos");
}

/// **Une carte sélectionnée se repose au bit près, elle aussi** — à 100 % comme à 150 %.
///
/// Elle se reposait « à un cran de couverture près » : jusqu'à vingt-six niveaux sur deux cents
/// canaux, et ce cran n'était pas à nous. L'anneau de sélection était un trait anticrénelé de
/// `tiny-skia`, peint DANS la texture, et `tiny-skia` accumule ses bords en virgule fixe le long
/// de chaque ligne : la même forme translatée d'un nombre entier de pixels ne donnait pas
/// toujours la même couverture. Depuis COMPOSANT-4, l'anneau se pose au-dessus, au même endroit
/// sur les deux voies, et la brume se peint par une loi évaluée pixel par pixel : la tolérance
/// disparaît avec sa cause.
#[test]
fn test_une_carte_selectionnee_se_repose_au_bit_pres() {
    assert_eq!(ecart_carte(true, 1.0), (0, 0), "a 100 %");
    // DPI-1 : à 150 %, l'anneau a l'épaisseur de celui qu'on dessine en place.
    assert_eq!(ecart_carte(true, 1.5), (0, 0), "a 150 %");
}

// -- COMPOSANT-2 : la carte qu'on edite --------------------------------------

/// **La carte qu'on edite est un composant**, et elle ne l'etait pas.
///
/// Le terrain du 22/09 la chiffre a 9,74 ms en median sur le geste « editer du texte », dont
/// l'image mediane coute 19,48 ms : cinquante et une images par seconde pendant qu'on ecrit.
/// Ce n'est pas la frappe qui coute, c'est de refaire a l'identique entre deux touches.
#[test]
fn test_la_carte_qu_on_edite_est_un_composant() {
    let renderer = Renderer::new();
    let (_, regime) = regime_temoin();
    let corps = "Une note qu'on est en train d'ecrire.";
    let e = saisie(corps, 4, true);
    assert!(
        regime
            .carte(
                renderer.kit(),
                "c",
                (60.0, 40.0, 240.0, 60.0),
                (corps, (96, 165, 250)),
                (Some(&e), None,),
            )
            .is_some(),
        "une carte en saisie doit devenir une texture"
    );
}

/// **La clé ne suit plus le curseur, mais elle suit une sélection étendue** (COMPOSANT-3).
///
/// Le curseur se pose au-dessus de la texture : sa phase et sa place n'y changent rien, et
/// n'ont donc pas à la refaire. Une sélection étendue, elle, se peint **sous** le texte, dans
/// la texture : son étendue doit changer la clé, sans quoi on verrait une sélection périmée.
#[test]
fn test_composant_3_la_cle_ignore_le_curseur_et_suit_la_selection() {
    let renderer = Renderer::new();
    let (_, regime) = regime_temoin();
    let corps = "Une note qu'on est en train d'ecrire.";
    let cle = |selection: glucose_core::text::Selection, visible: bool| {
        let e = crate::renderer::TextEditSession {
            selection,
            ..saisie(corps, 0, visible)
        };
        regime
            .carte(
                renderer.kit(),
                "c",
                (60.0, 40.0, 240.0, 60.0),
                (corps, (96, 165, 250)),
                (Some(&e), None),
            )
            .expect("un composant")
            .seule()
            .cle
    };
    let a = glucose_core::text::Selection::at;
    assert_eq!(
        cle(a(4), true),
        cle(a(4), false),
        "la phase du curseur n'y est pas"
    );
    assert_eq!(cle(a(4), true), cle(a(9), true), "sa place non plus");
    let etendue = |debut, fin| glucose_core::text::Selection {
        anchor: debut,
        head: fin,
    };
    assert_ne!(
        cle(etendue(2, 9), true),
        cle(a(4), true),
        "une sélection étendue, si"
    );
    assert_ne!(
        cle(etendue(2, 9), true),
        cle(etendue(2, 12), true),
        "et son étendue"
    );
}

/// **L'identite ne bouge pas, elle.**
///
/// Sans quoi CASCADE-2 ne pourrait plus garder l'ancienne texture posee pendant que la
/// nouvelle se rend : chaque frappe ferait disparaitre la carte le temps d'une image.
#[test]
fn test_l_identite_d_une_carte_en_saisie_ne_bouge_pas() {
    let renderer = Renderer::new();
    let (_, regime) = regime_temoin();
    let identite = |corps: &str, visible: bool| {
        let e = saisie(corps, 1, visible);
        regime
            .carte(
                renderer.kit(),
                "c",
                (60.0, 40.0, 240.0, 60.0),
                (corps, (96, 165, 250)),
                (Some(&e), None),
            )
            .expect("un composant")
            .seule()
            .identite
    };
    assert_eq!(identite("ab", true), identite("abc", false));
}

/// Une carte en saisie peinte par les deux voies : la carte entiere en place au processeur
/// (curseur allume, puis eteint), et la texture reposee sous les ornements de la couche du
/// dessus.
struct DeuxVoies {
    en_place: Pixmap,
    eteint: Pixmap,
    reposee: Pixmap,
}

/// La carte d'epreuve : a une position fractionnaire, pour que la texture ait une phase.
const CARTE: (f64, f64, f32, f32) = (60.3, 40.7, 240.0, 60.0);

fn deux_voies_en_saisie(corps: &str, tete: usize) -> DeuxVoies {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let (vp, regime) = regime_temoin();
    let taille = (500u32, 300u32);
    let (x, y, w, h) = CARTE;
    let teinte = (96, 165, 250);
    let (e, eteint) = (saisie(corps, tete, true), saisie(corps, tete, false));
    let carte = |editing| TextCard {
        origin: (x, y),
        size: (w, h),
        body: corps,
        tint: teinte,
        fond: crate::renderer::card::papier(&crate::theme::Theme::dark()),
        eclaires: None,
        selected: false,
        editing,
    };
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        mises_en_page: kit.mises_en_page,
        tints: kit.tints,
        theme: kit.theme,
        vp,
        scale: WorldScale::new(vp.scale, 1.0),
        clip: Clip {
            width: taille.0 as f32,
            height: taille.1 as f32,
            top: 0.0,
        },
        poignees: true,
    };
    let en_place_avec = |saisie| {
        let mut pixmap = Pixmap::new(taille.0, taille.1).expect("pixmap");
        crate::renderer::card::draw_text_card(&ctx, &mut pixmap.as_mut(), carte(Some(saisie)));
        pixmap
    };

    let composant = regime
        .carte(kit, "c", (x, y, w, h), (corps, teinte), (Some(&e), None))
        .expect("une carte en saisie est toujours un composant")
        .seule();
    let texture = composant.rendre(kit).expect("une texture");
    let mut reposee = Pixmap::new(taille.0, taille.1).expect("pixmap");
    crate::composition::poser(
        &mut reposee.as_mut(),
        &texture,
        (composant.pose.x, composant.pose.y),
        glucose_core::report::Melange::Composer,
    );
    crate::renderer::card::draw_card_ornements(&ctx, &mut reposee.as_mut(), carte(Some(&e)));
    DeuxVoies {
        en_place: en_place_avec(&e),
        eteint: en_place_avec(&eteint),
        reposee,
    }
}

/// Le pire ecart entre deux images, et le nombre de canaux qui different de plus d'un niveau.
fn ecart(a: &Pixmap, b: &Pixmap) -> (u8, usize) {
    let paires = || a.data().iter().zip(b.data());
    let pire = paires().map(|(p, q)| p.abs_diff(*q)).max().unwrap_or(0);
    let canaux = paires().filter(|(p, q)| p.abs_diff(**q) > 1).count();
    (pire, canaux)
}

/// **Les deux voies donnent les memes pixels**, au bit pres.
///
/// Une carte en saisie se reposait « dans le cran de l'anneau » -- vingt-six niveaux, pour la
/// meme cause que la carte selectionnee. L'anneau n'est plus dans la texture (COMPOSANT-4).
fn au_bit_pres(voies: &DeuxVoies, quoi: &str) {
    assert_eq!(ecart(&voies.en_place, &voies.reposee), (0, 0), "{quoi}");
}

/// **Une carte en saisie donne les memes pixels sur les deux voies**, curseur compris : la
/// carte entiere dessinee en place par le processeur, contre la texture reposee puis le
/// curseur que la couche du dessus pose (COMPOSANT-3).
///
/// C'est l'epreuve qui compte : une texture qui montrerait autre chose que ce que le
/// processeur dessine ferait diverger les deux voies sans qu'aucun test de passe le voie --
/// c'est exactement la forme des quatre regressions de l'etape 1 (fiche 22 § 5).
#[test]
fn test_une_carte_en_saisie_se_repose_au_bit_pres() {
    let voies = deux_voies_en_saisie("Une note qu'on ecrit, avec des accents : eac.", 12);
    // **L'epreuve voit le curseur.** Sans ce temoin, elle passerait aussi si aucune des deux
    // voies ne le peignait -- la forme exacte d'une regression que personne ne verrait.
    assert_ne!(
        voies.en_place.data(),
        voies.eteint.data(),
        "le curseur allume doit se voir"
    );
    au_bit_pres(&voies, "carte en saisie");
}

/// **Une formule sous le curseur : la carte reste une texture, et sa previsualisation se pose
/// au-dessus** (COMPOSANT-3).
///
/// La pastille qui montre la formule qu'on ecrit se pose a DROITE de la carte, hors de sa
/// boite, et son placement lit la largeur du clip. Dans le contenu, elle ne tenait pas dans une
/// texture, et la carte entiere retombait au processeur a chaque image tant que le curseur
/// traversait une formule. C'est un ornement : elle suit le curseur, comme lui.
#[test]
fn test_une_formule_sous_le_curseur_reste_une_texture_et_sa_pastille_se_pose_au_dessus() {
    let voies = deux_voies_en_saisie("$$x^2$$", 3);
    au_bit_pres(&voies, "formule en saisie");
    // **L'epreuve voit la pastille** : de l'encre a droite de la carte, que la carte seule n'a
    // pas. Sans ce temoin, une pastille oubliee par les deux voies passerait.
    let (x, _, w, _) = CARTE;
    let bord = (x + f64::from(w) + 20.5) as u32 + 4;
    let encre_a_droite = |p: &Pixmap| {
        let largeur = p.width() as usize;
        p.data()
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(i, c)| i % largeur > bord as usize && c[3] > 0)
            .count()
    };
    assert!(
        encre_a_droite(&voies.reposee) > 100,
        "la pastille de la formule doit se voir a droite de la carte, sur la voie graphique"
    );
}

/// **Cent images sans une frappe ne rendent qu'une texture.**
///
/// C'est le gain de COMPOSANT-2, et il se prouve **sans chronometre** : une texture se refait
/// exactement quand sa cle change, donc compter les cles distinctes sur une seconde de saisie
/// immobile dit le nombre de rendus, sur n'importe quelle machine et sans bruit de mesure.
///
/// Un curseur clignote a deux hertz. Sur cent images -- une seconde a cent images par seconde,
/// le plancher de la charte -- COMPOSANT-2 en faisait **deux** etats, donc deux rendus, la ou
/// le processeur en payait cent. Depuis COMPOSANT-3, le curseur se pose au-dessus : **un
/// seul** rendu, quel que soit le temps qu'on laisse la carte ouverte.
///
/// La forme de ce test est celle que la fiche 20 § 5.2 demande : un cache qui repeindrait tout
/// rendrait les memes pixels et passerait toutes les epreuves d'aspect. Seul un compte le
/// distingue d'un cache qui sert.
#[test]
fn test_cent_images_de_saisie_immobile_ne_font_qu_une_texture() {
    let renderer = Renderer::new();
    let (_, regime) = regime_temoin();
    let corps = "Une note qu'on laisse ouverte sans y toucher.";
    let mut vues = std::collections::BTreeSet::new();
    for image in 0..100 {
        // Le curseur s'allume et s'eteint toutes les vingt-cinq images : deux hertz a cent
        // images par seconde, ce que la boucle de reveil calcule deja pour BLINK-1.
        let e = saisie(corps, 12, (image / 25) % 2 == 0);
        let c = regime
            .carte(
                renderer.kit(),
                "c",
                (60.0, 40.0, 240.0, 60.0),
                (corps, (96, 165, 250)),
                (Some(&e), None),
            )
            .expect("un composant")
            .seule();
        vues.insert(c.cle);
    }
    assert_eq!(
        vues.len(),
        1,
        "une saisie immobile ne refait jamais sa texture : le curseur clignote au-dessus"
    );
}

/// **Une frappe refait la texture, et une seule.**
///
/// Le pendant du test precedent, et il est aussi necessaire : un composant dont la cle
/// ignorerait le tampon de saisie passerait le test des deux textures **en montrant du texte
/// perime**. C'est la faute que la fiche 20 § 5.4 nomme -- un test qui prouve une egalite ne
/// prouve pas un choix.
#[test]
fn test_chaque_frappe_donne_une_texture_et_une_seule() {
    let renderer = Renderer::new();
    let (_, regime) = regime_temoin();
    let mut vues = std::collections::BTreeSet::new();
    let frappe = "Bonjour";
    for n in 1..=frappe.len() {
        let corps = &frappe[..n];
        let e = saisie(corps, n, true);
        let c = regime
            .carte(
                renderer.kit(),
                "c",
                (60.0, 40.0, 240.0, 60.0),
                (corps, (96, 165, 250)),
                (Some(&e), None),
            )
            .expect("un composant")
            .seule();
        vues.insert(c.cle);
    }
    assert_eq!(
        vues.len(),
        frappe.len(),
        "sept caracteres tapes donnent sept images differentes"
    );
}

/// **REPOS-4 — un mouvement près du repos garde ses textures** (fiches 100 et 109).
///
/// Au clic molette, la vue avançait de fractions de pixel : chaque nœud changeait de phase,
/// passait au palier du mouvement, et toutes les textures visibles se refaisaient au premier
/// mouvement (fiche 100) ; à la molette, chaque cran les refaisait deux fois — au palier, puis
/// exactes au repos (fiche 109). Près du repos, la texture du repos garde sa clé, et se pose par
/// le passage exact du repos à l'écran : une translation entière au pixel entier près, une
/// fraction à sa place exacte, un cran de zoom agrandie d'autant. **La preuve à l'envers** :
/// au-delà d'une demi-octave, ou sans repos connu, le palier — une autre clé.
#[test]
fn test_repos_4_un_mouvement_pres_du_repos_garde_ses_textures() {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let pose = |vp: Viewport, regard: Regard| {
        let composant = Regime::de((vp, 1.0), regard, (500, 300), 0.0)
            .carte(
                kit,
                "t",
                (60.3, 40.7, 240.0, 60.0),
                ("Un texte qui se déplace", (96, 165, 250)),
                (None, None),
            )
            .and_then(|p| p.posee)
            .expect("une carte posée");
        (composant.cle, composant.pose)
    };
    let repos = Viewport {
        scale: 1.37,
        x: 20.5,
        y: 10.25,
    };
    let (cle_du_repos, p0) = pose(repos, Regard::immobile());
    let en_mouvement = Regard {
        degradation_permise: true,
        en_mouvement: true,
        repos: Some(repos),
    };
    let ailleurs = |dx: f64, dy: f64, zoom: f64| Viewport {
        scale: repos.scale * zoom,
        x: repos.x + dx,
        y: repos.y + dy,
    };
    let (cle, p) = pose(ailleurs(37.0, -12.0, 1.0), en_mouvement);
    assert_eq!(cle, cle_du_repos, "une translation entière");
    assert_eq!(
        (p.x - p0.x, p.y - p0.y, p.largeur),
        (37.0, -12.0, p0.largeur),
        "posée au pixel entier près"
    );
    let (cle, p) = pose(ailleurs(0.5, 0.0, 1.0), en_mouvement);
    assert_eq!(cle, cle_du_repos, "une fraction");
    assert!((p.x - p0.x - 0.5).abs() < 1e-4, "{} pour {}", p.x, p0.x);
    // Un cran de molette : le monde garde sa place sous le point fixe du zoom.
    let zoom = 1.25;
    let vue = ailleurs(-40.0, -30.0, zoom);
    let (cle, p) = pose(vue, en_mouvement);
    assert_eq!(cle, cle_du_repos, "un cran de zoom");
    let monde = (
        (p0.x as f64 - repos.x) / repos.scale,
        (p0.y as f64 - repos.y) / repos.scale,
    );
    let attendu = (monde.0 * vue.scale + vue.x, monde.1 * vue.scale + vue.y);
    assert!(
        (f64::from(p.x) - attendu.0).abs() < 1e-3 && (f64::from(p.y) - attendu.1).abs() < 1e-3,
        "le coin du repos, vu d'ici : {:?} pour {attendu:?}",
        (p.x, p.y)
    );
    assert!((p.largeur - p0.largeur * zoom as f32).abs() < 1e-3);

    // Hors de l'écran du repos, à l'écran maintenant : il se pose — l'écran se lit à l'écran.
    let hors_du_repos = |vp: Viewport, regard: Regard| {
        Regime::de((vp, 1.0), regard, (500, 300), 0.0)
            .carte(
                kit,
                "u",
                (400.0, 40.7, 240.0, 60.0),
                ("Un texte qui entre", (96, 165, 250)),
                (None, None),
            )
            .is_some()
    };
    assert!(
        !hors_du_repos(repos, Regard::immobile()),
        "hors de l'écran au repos"
    );
    assert!(
        hors_du_repos(ailleurs(-200.0, 0.0, 1.0), en_mouvement),
        "à l'écran après le geste"
    );

    let (loin, _) = pose(ailleurs(0.0, 0.0, 1.5), en_mouvement);
    let sans_repos = Regard {
        repos: None,
        ..en_mouvement
    };
    let (inconnu, _) = pose(ailleurs(0.5, 0.0, 1.0), sans_repos);
    assert!(
        loin != cle_du_repos && inconnu != cle_du_repos,
        "au-delà d'une demi-octave, ou sans repos : le palier"
    );
}

/// **REPOS-4 — chaque sorte de composant se pose par le passage exact** (fiche 109) : une carte, le
/// titre d'une membrane et une photo en chemin calculent chacun leur coin ; pendant un cran de zoom
/// près du repos, chacun garde la clé du repos, et son coin et sa taille à l'écran sont ceux du
/// repos vus d'ici — `(repos − vue du repos) / échelle du repos × échelle + vue`.
#[test]
fn test_repos_4_chaque_composant_se_pose_par_le_passage_exact() {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let repos = Viewport {
        scale: 0.83,
        x: 31.4,
        y: -12.6,
    };
    let vue = Viewport {
        scale: 0.83 * 1.3,
        x: -55.2,
        y: 4.1,
    };
    let en_mouvement = Regard {
        degradation_permise: true,
        en_mouvement: true,
        repos: Some(repos),
    };
    let img = BoardImage::new("p".to_string(), 150.3, 100.7, 180.0, 120.0);
    let titre = titre::TitreDuMonde {
        membrane: "m",
        texte: "Hydrologie",
        coin: (100.3, 260.7),
        mesures: [20.0, 20.0, 22.5],
        teintes: (
            (96, 165, 250),
            tiny_skia::Color::from_rgba8(18, 18, 20, 255),
        ),
    };
    let composants = |vp: Viewport, regard: Regard| {
        let regime = Regime::de((vp, 1.0), regard, (2400, 1200), 0.0);
        let carte = regime.carte(
            kit,
            "t",
            (60.3, 40.7, 240.0, 60.0),
            ("Un texte", (96, 165, 250)),
            (None, None),
        );
        let photo = regime.photo_en_chemin(kit, &img);
        [
            carte.and_then(|p| p.posee).expect("la carte"),
            regime.titre(kit, &titre),
            photo.and_then(|p| p.posee).expect("la photo en chemin"),
        ]
    };
    let k = vue.scale / repos.scale;
    for (a, b) in composants(repos, Regard::immobile())
        .iter()
        .zip(&composants(vue, en_mouvement))
    {
        assert_eq!(a.cle, b.cle, "{} : la clé du repos", a.identite);
        let vu_d_ici = |x: f32, origine: f64, fin: f64| {
            (f64::from(x) - origine) / repos.scale * vue.scale + fin
        };
        let attendu = (
            vu_d_ici(a.pose.x, repos.x, vue.x),
            vu_d_ici(a.pose.y, repos.y, vue.y),
        );
        assert!(
            (f64::from(b.pose.x) - attendu.0).abs() < 1e-3
                && (f64::from(b.pose.y) - attendu.1).abs() < 1e-3,
            "{} : {:?} pour {attendu:?}",
            a.identite,
            (b.pose.x, b.pose.y)
        );
        assert!(
            (f64::from(b.pose.largeur) - f64::from(a.pose.largeur) * k).abs() < 1e-3,
            "{} : agrandie d'autant",
            a.identite
        );
    }
}
