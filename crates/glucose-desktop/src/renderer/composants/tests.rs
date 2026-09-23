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
    let regime = Regime::de(vp, Regard::immobile(), taille, 0.0);

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
    let composant = regime.photo_en_chemin(&img).expect("un composant").seule();
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
    (vp, Regime::de(vp, Regard::immobile(), (500, 300), 0.0))
}

/// Un composant de carte, rendu à part et reposé, comparé à la carte dessinée en place.
fn ecart_carte(selectionnee: bool) -> (u8, usize) {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let vp = Viewport {
        scale: 1.0,
        x: 20.5,
        y: 10.25,
    };
    let taille = (500u32, 300u32);
    let regime = Regime::de(vp, Regard::immobile(), taille, 0.0);
    let (x, y, w, h) = (60.3, 40.7, 240.0f32, 60.0f32);
    let corps = "Accents : éàçùôêîï — « guillemets »
Et un lien.";
    let teinte = (96, 165, 250);

    let mut en_place = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        tints: kit.tints,
        theme: kit.theme,
        vp,
        scale: WorldScale::new(vp.scale),
        clip: Clip {
            width: taille.0 as f32,
            height: taille.1 as f32,
            top: 0.0,
        },
    };
    draw_card_contenu(
        &ctx,
        &mut en_place.as_mut(),
        TextCard {
            origin: (x, y),
            size: (w, h),
            body: corps,
            tint: teinte,
            selected: selectionnee,
            editing: None,
        },
    );
    let composant = regime
        .carte(kit, "c", (x, y, w, h), (corps, teinte, selectionnee), None)
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

/// **Une carte au repos se repose au bit près** : même code, même mise en page, même phase.
#[test]
fn test_une_carte_se_repose_au_bit_pres() {
    let (pire, canaux) = ecart_carte(false);
    assert!(
        pire <= 1 && canaux == 0,
        "carte au repos : pire {pire}, {canaux} canaux au-dela de 1"
    );
}

/// **Une carte sélectionnée se repose à un cran de couverture près**, et ce cran n'est pas
/// à nous.
///
/// Le trait de sélection est un anneau anti-crénelé de deux pixels, et `tiny-skia` accumule
/// ses bords en virgule fixe le long de chaque ligne : la même forme, translatée d'un nombre
/// **entier** de pixels, ne donne pas toujours la même couverture aux points où la tangente
/// d'un coin arrondi frôle une frontière de sous-pixel. L'écart vaut un cran de son
/// suréchantillonnage — au plus une vingtaine de niveaux, sur quelques dizaines de pixels — et
/// il existe déjà sur la voie processeur seule, entre deux images d'un glissement.
///
/// Ce n'est donc pas une différence de loi entre les voies : c'est la sensibilité du
/// rastériseur à la position absolue, mesurée et bornée ici pour qu'elle ne grandisse pas.
#[test]
fn test_une_carte_selectionnee_se_repose_a_un_cran_de_couverture_pres() {
    let (pire, canaux) = ecart_carte(true);
    assert!(
        pire <= 26,
        "carte selectionnee : pire {pire} -- plus qu'un cran de couverture"
    );
    assert!(
        canaux <= 200,
        "carte selectionnee : {canaux} canaux au-dela de 1 -- ce n'est plus un coin, c'est un          bord entier"
    );
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
                (corps, (96, 165, 250), false),
                Some(&e),
            )
            .is_some(),
        "une carte en saisie doit devenir une texture"
    );
}

/// **La cle suit le curseur, et c'est ce qui le fait clignoter.**
///
/// Si la phase du curseur n'entrait pas dans l'empreinte, la texture ne se referait jamais
/// entre deux frappes et le curseur cesserait purement et simplement de clignoter -- une
/// regression qu'aucune epreuve d'aspect ne verrait, puisque chaque image prise isolement
/// serait juste. C'est la meme forme que BLINK-1, prise a l'envers.
#[test]
fn test_la_cle_change_avec_la_phase_du_curseur_et_avec_sa_position() {
    let renderer = Renderer::new();
    let (_, regime) = regime_temoin();
    let corps = "Une note qu'on est en train d'ecrire.";
    let cle = |tete: usize, visible: bool| {
        let e = saisie(corps, tete, visible);
        regime
            .carte(
                renderer.kit(),
                "c",
                (60.0, 40.0, 240.0, 60.0),
                (corps, (96, 165, 250), false),
                Some(&e),
            )
            .expect("un composant")
            .seule()
            .cle
    };
    assert_ne!(
        cle(4, true),
        cle(4, false),
        "la phase du curseur doit changer la cle"
    );
    assert_ne!(cle(4, true), cle(9, true), "sa position aussi");
    assert_eq!(cle(4, true), cle(4, true), "et le meme etat donne la meme");
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
                (corps, (96, 165, 250), false),
                Some(&e),
            )
            .expect("un composant")
            .seule()
            .identite
    };
    assert_eq!(identite("ab", true), identite("abc", false));
}

/// **Une carte dont le curseur est pose sur une formule reste au processeur.**
///
/// Sa previsualisation se pose hors de sa boite et son placement lit la largeur du clip, qui
/// vaut l'ecran dans une passe et la texture dans un composant : elle ne tiendrait pas
/// dedans, et elle basculerait a gauche au lieu de se poser a droite.
#[test]
fn test_une_formule_sous_le_curseur_garde_la_carte_au_processeur() {
    let renderer = Renderer::new();
    let (_, regime) = regime_temoin();
    let corps = "$$x^2$$";
    let composant = |tete: usize| {
        let e = saisie(corps, tete, true);
        regime
            .carte(
                renderer.kit(),
                "c",
                (60.0, 40.0, 240.0, 60.0),
                (corps, (96, 165, 250), false),
                Some(&e),
            )
            .is_some()
    };
    assert!(
        !composant(3),
        "une previsualisation de formule ne rentre pas dans une texture"
    );
    // **Et le refus est bien cible** : la meme carte, dont le texte n'est pas une formule,
    // redevient un composant. Sans ce second cas, le test passerait aussi si le refus etait
    // devenu inconditionnel -- et toutes les cartes en saisie seraient retombees au
    // processeur sans que rien ne le dise.
    let ordinaire = saisie("Une note ordinaire.", 3, true);
    assert!(
        regime
            .carte(
                renderer.kit(),
                "c",
                (60.0, 40.0, 240.0, 60.0),
                ("Une note ordinaire.", (96, 165, 250), false),
                Some(&ordinaire),
            )
            .is_some(),
        "une carte en saisie sans formule reste une texture"
    );
}

/// **Une carte en saisie rendue a part donne les memes pixels qu'en place**, curseur compris.
///
/// C'est l'epreuve qui compte : une texture qui montrerait autre chose que ce que le
/// processeur dessine ferait diverger les deux voies sans qu'aucun test de passe le voie --
/// c'est exactement la forme des quatre regressions de l'etape 1 (fiche 22 § 5).
#[test]
fn test_une_carte_en_saisie_se_repose_au_bit_pres() {
    let renderer = Renderer::new();
    let kit = renderer.kit();
    let (vp, regime) = regime_temoin();
    let taille = (500u32, 300u32);
    let (x, y, w, h) = (60.3, 40.7, 240.0f32, 60.0f32);
    let corps = "Une note qu'on ecrit, avec des accents : eac.";
    let teinte = (96, 165, 250);
    let e = saisie(corps, 12, true);

    let mut en_place = Pixmap::new(taille.0, taille.1).expect("pixmap");
    let ctx = Pass {
        typography: kit.typography,
        math: kit.math,
        tints: kit.tints,
        theme: kit.theme,
        vp,
        scale: WorldScale::new(vp.scale),
        clip: Clip {
            width: taille.0 as f32,
            height: taille.1 as f32,
            top: 0.0,
        },
    };
    draw_card_contenu(
        &ctx,
        &mut en_place.as_mut(),
        TextCard {
            origin: (x, y),
            size: (w, h),
            body: corps,
            tint: teinte,
            selected: false,
            editing: Some(&e),
        },
    );
    let composant = regime
        .carte(kit, "c", (x, y, w, h), (corps, teinte, false), Some(&e))
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
    // **La meme borne que la carte selectionnee, et c'est la meme cause.**
    //
    // `draw_card_frame` traite une carte en saisie comme une carte selectionnee : son cadre
    // fin devient l'anneau de deux pixels. On retrouve donc exactement le cran de couverture
    // que `test_une_carte_selectionnee_se_repose_a_un_cran_de_couverture_pres` mesure, et
    // pour la meme raison -- `tiny-skia` accumule ses bords en virgule fixe le long de chaque
    // ligne, et la meme forme a une position absolue differente ne donne pas toujours la meme
    // couverture la ou la tangente d'un coin arrondi frole une frontiere de sous-pixel.
    //
    // La mesure le dit sans ambiguite : le meme test sans saisie donne **zero**, et le seul
    // changement entre les deux est l'epaisseur de ce trait. Une seconde borne pour la meme
    // cause finirait par diverger de la premiere : c'est la sienne qu'on reprend.
    assert!(pire <= 26, "carte en saisie : pire {pire}");
    assert!(
        canaux <= 200,
        "carte en saisie : {canaux} canaux au-dela de 1 -- ce n'est plus un coin"
    );
}

/// **Cent images sans une frappe ne rendent plus cent textures.**
///
/// C'est le gain de COMPOSANT-2, et il se prouve **sans chronometre** : une texture se refait
/// exactement quand sa cle change, donc compter les cles distinctes sur une seconde de saisie
/// immobile dit le nombre de rendus, sur n'importe quelle machine et sans bruit de mesure.
///
/// Un curseur clignote a deux hertz. Sur cent images -- une seconde a cent images par seconde,
/// le plancher de la charte -- cela fait **deux** etats, donc deux rendus, la ou le processeur
/// en payait cent. Le terrain du 22/09 chiffre chacun a 9,74 ms.
///
/// La forme de ce test est celle que la fiche 20 § 5.2 demande : un cache qui repeindrait tout
/// rendrait les memes pixels et passerait toutes les epreuves d'aspect. Seul un compte le
/// distingue d'un cache qui sert.
#[test]
fn test_cent_images_de_saisie_immobile_ne_font_que_deux_textures() {
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
                (corps, (96, 165, 250), false),
                Some(&e),
            )
            .expect("un composant")
            .seule();
        vues.insert(c.cle);
    }
    assert_eq!(
        vues.len(),
        2,
        "une saisie immobile ne montre que deux images : curseur allume, curseur eteint"
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
                (corps, (96, 165, 250), false),
                Some(&e),
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
