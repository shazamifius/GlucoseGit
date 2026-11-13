//! **Les deux voies rendent la même image** — la garantie centrale de la fiche 21, tenue sur
//! une scène entière et non sur une passe isolée.
//!
//! # Pourquoi ce fichier existe, et ce qu'il aurait évité
//!
//! L'étape 1 — les photos sur la carte — a produit **trois** régressions visibles en trois
//! commits, et chacune s'est vue à l'écran avant qu'un test ne la voie :
//!
//! * un écran noir, parce que la pose déplacée avait emporté la **réclamation** des photos au
//!   magasin ;
//! * les poignées et le cadre de sélection disparus, parce qu'elle avait emporté les
//!   **ornements** ;
//! * les photos **en chemin** qui ne se dessinaient plus du tout — celle-là a survécu à trois
//!   commits et n'a été vue qu'en comparant deux captures côte à côte.
//!
//! Les trois ont la même forme : une passe qui cesse d'être appelée. Aucun test de passe ne
//! peut les voir, parce qu'un test de passe appelle la passe. **Seule une comparaison de
//! l'image entière** les attrape, et c'est ce que ce fichier fait.
//!
//! # Ce qui est comparé, et pourquoi pas au bit près
//!
//! La charte demande que deux voies d'une même opération produisent le même résultat, « au
//! bit près quand c'est possible, à un écart borné et mesuré sinon ». Ici l'écart existe et il
//! est connu, passe par passe :
//!
//! * la **grille**, jusqu'à sept niveaux : le processeur quantifie la position d'un point sur
//!   seize phases sous-pixel, la carte ne le fait pas (`present::fond_gpu`) ;
//! * les **lueurs**, jusqu'à deux niveaux : le processeur échantillonne `Φ` dans une table, la
//!   carte l'évalue analytiquement (`present::lueurs_gpu`) ;
//! * la **composition**, un niveau : la voie graphique compose en cinq temps, donc une chaîne
//!   d'arrondis huit bits différente de celle du processeur, qui peint tout dans un tampon.
//!
//! Aucune de ces trois causes n'est une dégradation : ce sont trois endroits où la carte est
//! **plus juste**. Ce qu'un écart plus grand signalerait, c'est autre chose — une passe qui
//! manque, un ordre inversé, une couche qui remplace au lieu de composer.
//!
//! Sur une machine sans carte utilisable, ces épreuves se **sautent** : l'absence de matériel
//! n'est pas un défaut du code.

use glucose_core::synth;
use glucose_desktop::params::{Pointer, SceneOverlay};
use glucose_desktop::present::banc_gpu;
use glucose_desktop::renderer::{Confie, Regard, Renderer};
use glucose_desktop::typography::glyph::SUBPIXEL_PHASES;
use glucose_desktop::ui::UiState;
use tiny_skia::Pixmap;

/// L'écart admis entre les deux voies, en niveaux de couleur sur 255.
///
/// **Mesuré, puis élargi d'une marge nommée.** Sur cette scène et sur cette machine, le pire
/// écart vaut **25**, et il n'est pas à nous : c'est un cran de couverture de `tiny-skia`
/// sur le trait de sélection d'une carte (`renderer::composants::tests`). Le rastériseur
/// accumule ses bords en virgule fixe le long de chaque ligne, et la même forme translatée
/// d'un nombre **entier** de pixels ne donne pas toujours la même couverture là où la
/// tangente d'un coin arrondi frôle une frontière de sous-pixel. Une carte rendue dans sa
/// texture, puis posée, est exactement cette forme translatée.
///
/// Tout le reste tient sous **trois** niveaux, sur cette scène et zéro canal au-delà, comme
/// avant les composants. Un pire écart seul ne dit pas si un pixel est en cause ou un
/// million : c'est [`PART_MAX_POUR_MILLE`] qui attrape une passe manquante, et il n'a pas
/// bougé.
const ECART_ADMIS: u8 = 26;

/// Au-delà de cet écart, un pixel n'est plus une question de phase ni d'arrondi.
const ECART_COURANT: u8 = 3;

/// Combien de canaux, au plus, ont le droit de dépasser [`ECART_COURANT`], en millièmes.
///
/// Mesuré à **zéro** sur cette machine. Un pour mille est la marge d'une autre carte, et cela
/// reste trois ordres de grandeur sous ce que laisserait une passe manquante : les deux cadres
/// de photos en chemin, à eux seuls, pèsent soixante mille pixels sur un million six cent
/// mille — soit trente-sept pour mille.
///
/// Les deux grandeurs se lisent ensemble : le pire écart seul ne dit pas si un pixel est en
/// cause ou un million, et c'est la leçon que la chronique a coûté cher à apprendre (fiche 20
/// § 4.5).
const PART_MAX_POUR_MILLE: usize = 1;

/// La scène témoin rendue par la **voie processeur** : tout dans un seul tampon.
fn par_le_processeur(taille: (u32, u32), store: &glucose_core::store::Store) -> Pixmap {
    let mut pixmap = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    renderer.render(
        &mut pixmap.as_mut(),
        store,
        &mut ui,
        SceneOverlay::sans_rien(&guides),
        Pointer { x: 0.0, y: 0.0 },
        Regard::immobile(),
    );
    pixmap
}

/// Ce que le processeur produit pour la **voie graphique** : deux couches, ce qu'il confie,
/// et le moteur qui saura rendre un composant dont la texture manque.
fn les_deux_couches(
    taille: (u32, u32),
    store: &glucose_core::store::Store,
) -> (Renderer, Pixmap, Pixmap, Confie) {
    les_deux_couches_en_editant(taille, store, None)
}

/// La meme chose, avec une saisie en cours sur l'une des cartes.
fn les_deux_couches_en_editant(
    taille: (u32, u32),
    store: &glucose_core::store::Store,
    editing: Option<&glucose_desktop::renderer::TextEditSession>,
) -> (Renderer, Pixmap, Pixmap, Confie) {
    let mut dessous = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    let mut dessus = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    dessous.fill(tiny_skia::Color::TRANSPARENT);
    dessus.fill(tiny_skia::Color::TRANSPARENT);
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let confie = renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        store,
        (&mut ui, Pointer { x: 0.0, y: 0.0 }),
        SceneOverlay {
            editing,
            ..SceneOverlay::sans_rien(&guides)
        },
        Regard::immobile(),
    );
    (renderer, dessous, dessus, confie)
}

/// La scène témoin composée par la **voie graphique**, en cinq temps, hors fenêtre.
///
/// Le moteur reste sous la main : une carte de texte dont la texture manque se rend à la
/// demande, comme la présentation le fait (CARTE-GPU-1).
fn par_la_carte(taille: (u32, u32), store: &glucose_core::store::Store) -> Option<Pixmap> {
    let (renderer, dessous, dessus, confie) = les_deux_couches(taille, store);
    let (peripherique, file) = banc_gpu::carte()?;
    banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        taille,
        &confie,
        (&dessous, &dessus),
        &|cle| confie.pixels(&renderer, cle),
    )
}

/// **La scène entière rend la même image des deux côtés.**
///
/// C'est l'épreuve que les trois régressions de l'étape 1 auraient toutes échouée : une passe
/// qui cesse d'être appelée laisse un trou que rien d'autre ne remplit.
#[test]
fn test_les_deux_voies_rendent_la_meme_scene() {
    let taille = synth::WITNESS_SIZE;
    let store = synth::witness_selected();
    let Some(carte) = par_la_carte(taille, &store) else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let processeur = par_le_processeur(taille, &store);

    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "les deux voies divergent de {pire} niveaux (admis {ECART_ADMIS}) : ce n'est plus un \
         arrondi, c'est une passe qui manque ou un ordre inverse"
    );

    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    let canaux = processeur.data().len();
    assert!(
        larges * 1000 <= canaux * PART_MAX_POUR_MILLE,
        "l'ecart depasse {ECART_COURANT} sur {larges} canaux sur {canaux} : une passe entiere \
         manque, ou une couche remplace la ou elle devrait composer"
    );
}

/// **Une photo en chemin se pose à son rang, comme un composant.**
///
/// La scène témoin porte trois images sans octets. Sur la voie processeur elles se dessinent
/// comme un cadre gris portant leur identifiant ; sur la voie graphique elles sont des
/// composants rendus à la demande (COMPOSANT-1), et pendant trois commits **plus personne** ne
/// les dessinait.
///
/// Le test ne compare pas les deux voies, il compte : ce que le processeur confie à la carte
/// doit porter une texture par photo en chemin, et cette texture doit porter de l'encre. Un
/// test qui comparerait les images passerait aussi le jour où les deux voies cesseraient de
/// les dessiner.
#[test]
fn test_une_photo_en_chemin_est_un_composant_qui_porte_de_l_encre() {
    let taille = synth::WITNESS_SIZE;
    let store = synth::witness_selected();
    let (renderer, _, _, confie) = les_deux_couches(taille, &store);
    let en_chemin: Vec<_> = confie
        .photos
        .iter()
        .filter(|t| t.cle.nom().is_some_and(|n| n.starts_with("chemin:")))
        .collect();
    assert_eq!(
        en_chemin.len(),
        3,
        "le temoin porte trois photos sans octets, et chacune est une texture a son rang"
    );
    for posee in en_chemin {
        let composant = confie
            .composant(&posee.cle)
            .expect("une photo en chemin sait se rendre");
        let texture = composant.rendre(renderer.kit()).expect("une texture");
        let encre = texture
            .data()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] > 0)
            .count();
        assert!(
            encre > 5_000,
            "la texture de {:?} ne porte que {encre} pixels : le cadre en chemin ne se dessine              plus",
            posee.cle
        );
    }
}

/// Combien de pixels d'une couche portent de l'encre.
fn encre(p: &Pixmap) -> usize {
    p.data()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|c| c[3] > 0)
        .count()
}

/// **La carte qu'on ecrit est portee par la carte graphique, et le processeur ne la redessine
/// plus** (COMPOSANT-2).
///
/// # Pourquoi ce test compte au lieu de comparer
///
/// Deux fautes opposees sont possibles, et une comparaison d'images n'en verrait qu'une :
///
/// * si personne ne la dessine, elle **disparait** -- c'est la forme exacte des quatre
///   regressions de l'etape 1, et la fiche 22 § 5 rappelle qu'aucun test de passe ne peut les
///   voir, puisqu'un test de passe appelle la passe ;
/// * si les deux la dessinent, elle se compose **deux fois** sur elle-meme, ce qui ne se voit
///   presque pas sur un fond sombre et fausse pourtant l'epreuve des deux voies.
///
/// La couche du dessus porte la chrome et les ornements, et rien de plus. Ouvrir une saisie
/// n'y change donc rien : si son encre grandit, c'est que le processeur y a repeint la carte.
#[test]
fn test_la_carte_qu_on_edite_est_portee_par_la_carte_et_non_repeinte() {
    let taille = synth::WITNESS_SIZE;
    let store = synth::witness_selected();
    let board = store.active_board().expect("un tableau");
    let (id, texte) = board
        .annotations
        .iter()
        .find_map(|a| match a {
            glucose_core::types::Annotation::Text { id, text, .. } => {
                Some((id.clone(), text.clone()))
            }
            _ => None,
        })
        .expect("le temoin porte au moins une carte de texte");

    let session = glucose_desktop::renderer::TextEditSession {
        ann_id: id.clone(),
        buffer: texte,
        selection: glucose_core::text::Selection::at(1),
        goal_x: None,
        blink_timer: std::time::Instant::now(),
        curseur_visible: true,
    };
    let (_, _, sans, _) = les_deux_couches_en_editant(taille, &store, None);
    let (_, _, avec, confie) = les_deux_couches_en_editant(taille, &store, Some(&session));

    let prefixe = format!("carte:{id}:");
    assert!(
        confie
            .cartes
            .iter()
            .any(|t| t.cle.nom().is_some_and(|n| n.starts_with(&prefixe))),
        "la carte en saisie doit etre une texture que la carte pose, et elle ne l'est pas"
    );
    assert_eq!(
        confie
            .cartes
            .last()
            .map(|t| t.cle.nom().is_some_and(|n| n.starts_with(&prefixe))),
        Some(true),
        "elle se pose en DERNIER, donc au-dessus des autres cartes"
    );
    // **Ce test echoue sur l'implementation d'avant**, et c'est la seule preuve qui compte :
    // en y retablissant `au_processeur = editing.is_some()`, la couche du dessus passe de
    // 205 918 a 260 930 pixels d'encre -- cinquante-cinq mille de plus, soit exactement une
    // carte repeinte par-dessus celle que la carte graphique pose deja.
    //
    // **La borne est la carte elle-meme**, et non une part de la couche : celle-ci portait la
    // chrome, que la carte pose desormais (CHROME-1) -- 38 250 pixels d'encre au lieu de
    // 205 918, et les deux mille du curseur depassaient alors les « cinq pour cent » d'avant.
    // Une carte repeinte ajoute sa surface ; le curseur, un trait. La moitie de la carte les
    // separe d'un facteur dix de chaque cote.
    let pose = confie.cartes.last().expect("la carte en saisie").pose;
    let carte = (pose.largeur * pose.hauteur) as usize;
    let (avant, apres) = (encre(&sans), encre(&avec));
    assert!(
        apres <= avant + carte / 2,
        "la couche du dessus passe de {avant} a {apres} pixels d'encre, pour une carte de \
         {carte} : le processeur y repeint la carte que la carte graphique porte deja"
    );
}

/// **Sélectionner une carte ne refait pas sa texture** (COMPOSANT-4) : l'anneau qui la désigne
/// se pose dans la couche du dessus.
///
/// Sa session du 25/09 à 22 h 19 donne 12,35 ms de `textures` au p99 du geste « sélectionner » :
/// l'anneau était peint dans la texture, et la sélection entrait dans sa clé. Deux images du
/// même document, avant et après un clic sur une carte, doivent demander les mêmes textures —
/// et la couche du dessus doit porter l'anneau.
#[test]
fn test_selectionner_une_carte_ne_refait_pas_sa_texture() {
    let taille = synth::WITNESS_SIZE;
    let mut store = synth::witness();
    store.clear_selection();
    let id = store
        .active_board()
        .expect("un tableau")
        .annotations
        .iter()
        .find(|a| matches!(a, glucose_core::types::Annotation::Text { .. }))
        .map(|a| a.id().to_string())
        .expect("le temoin porte une carte de texte");
    let (_, _, dessus_avant, avant) = les_deux_couches(taille, &store);
    store.select_annotation(id.clone(), false);
    let (_, _, dessus_apres, apres) = les_deux_couches(taille, &store);

    let cles = |c: &Confie| c.cartes.iter().map(|t| t.cle.clone()).collect::<Vec<_>>();
    assert!(
        cles(&avant).iter().any(|c| c
            .nom()
            .is_some_and(|n| n.starts_with(&format!("carte:{id}:")))),
        "la carte est une texture : sans elle, l'epreuve ne dirait rien"
    );
    assert_eq!(
        cles(&avant),
        cles(&apres),
        "selectionner une carte ne doit refaire aucune texture"
    );
    assert!(
        encre(&dessus_apres) > encre(&dessus_avant),
        "l'anneau et les poignees de la carte selectionnee se posent dans la couche du dessus"
    );
}

/// **FLECHE-1 — une flèche nue montre ses ornements dans la couche du dessus** (fiches 107 et
/// 108) : ses poignées quand on la choisit, sa pastille quand on écrit son étiquette.
///
/// Sur la voie graphique, la carte peint le trait ; la couche du dessus ne porte d'une flèche
/// que ses ornements, et une flèche qui n'en porte aucun ne s'y calcule plus
/// (`porte_des_ornements`). L'épreuve de la fonction seule ne voit pas un appel qui lui
/// passerait la mauvaise chose, et le témoin des deux voies non plus : sa flèche choisie porte
/// un badge, qui la ferait calculer quand même. Ici, la flèche est nue.
#[test]
fn test_fleche_1_une_fleche_nue_montre_ses_ornements_dans_la_couche_du_dessus() {
    use glucose_core::types::Annotation;
    let taille = synth::WITNESS_SIZE;
    let mut store = glucose_core::store::Store::new("Une flèche nue");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = glucose_core::types::Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        };
    }
    store.add_annotation(
        &board,
        Annotation::arrow("f-nue", 200.0, 200.0, 700.0, 450.0),
    );
    store.clear_selection();
    let (_, _, repos, _) = les_deux_couches(taille, &store);

    store.select_annotation("f-nue".into(), false);
    let (_, _, choisie, _) = les_deux_couches(taille, &store);
    store.clear_selection();

    // Écrire dans une flèche qui n'a pas d'étiquette : la pastille, vide, montre qu'on écrit.
    let session = glucose_desktop::renderer::TextEditSession {
        ann_id: "f-nue".into(),
        buffer: String::new(),
        selection: glucose_core::text::Selection::at(0),
        goal_x: None,
        blink_timer: std::time::Instant::now(),
        curseur_visible: true,
    };
    let (_, _, saisie, _) = les_deux_couches_en_editant(taille, &store, Some(&session));

    let (r, c, s) = (encre(&repos), encre(&choisie), encre(&saisie));
    assert!(
        c > r,
        "choisie, la flèche nue doit poser ses poignées ({c} pixels d'encre contre {r} au repos)"
    );
    assert!(
        s > r,
        "en saisie, la flèche nue doit poser sa pastille ({s} pixels d'encre contre {r} au repos)"
    );
}

/// **Sur la voie graphique, le curseur de la carte qu'on écrit se pose dans la couche du
/// dessus** (COMPOSANT-3) — et rien d'autre ne change quand il clignote.
///
/// La texture ne le porte plus : si la passe oubliait de le poser, il disparaîtrait de l'écran
/// sans qu'aucune épreuve de texture le voie, puisque chacune appelle elle-même les ornements.
/// Trouvé en sabotant : sans ce test, retirer l'appel ne faisait rien tomber.
#[test]
fn test_le_curseur_de_la_carte_qu_on_ecrit_clignote_dans_la_couche_du_dessus() {
    let taille = synth::WITNESS_SIZE;
    let store = synth::witness_selected();
    let board = store.active_board().expect("un tableau");
    let (id, texte) = board
        .annotations
        .iter()
        .find_map(|a| match a {
            glucose_core::types::Annotation::Text { id, text, .. } => {
                Some((id.clone(), text.clone()))
            }
            _ => None,
        })
        .expect("le temoin porte au moins une carte de texte");
    let saisie = |curseur_visible| glucose_desktop::renderer::TextEditSession {
        ann_id: id.clone(),
        buffer: texte.clone(),
        selection: glucose_core::text::Selection::at(1),
        goal_x: None,
        blink_timer: std::time::Instant::now(),
        curseur_visible,
    };
    let (_, _, allume, confie_allume) =
        les_deux_couches_en_editant(taille, &store, Some(&saisie(true)));
    let (_, _, eteint, confie_eteint) =
        les_deux_couches_en_editant(taille, &store, Some(&saisie(false)));

    let differents = allume
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .zip(eteint.data().as_chunks::<4>().0)
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differents > 0,
        "le curseur allume doit se voir dans la couche du dessus"
    );
    // Un trait de deux pixels sur la hauteur d'une ligne : quelques dizaines de pixels, pas une
    // carte. La borne est la surface du curseur, large : tout ce qui la depasse serait autre
    // chose que lui.
    assert!(
        differents < 200,
        "{differents} pixels changent quand le curseur clignote : ce n'est plus un curseur"
    );
    let cles = |c: &Confie| c.cartes.iter().map(|t| t.cle.clone()).collect::<Vec<_>>();
    assert_eq!(
        cles(&confie_allume),
        cles(&confie_eteint),
        "un clignotement ne doit refaire aucune texture"
    );
}

// ── RECADRAGE-1 : les deux voies cadrent pareil ───────────────────────────────────────────

/// Une photo dont le quart gauche est une bande noire, écrite sur le disque pour que le
/// magasin la décode comme il décoderait la vraie.
///
/// Sur le disque et non fabriquée en mémoire : l'entrée de test du magasin n'est pas visible
/// d'un test d'intégration, et c'est très bien ainsi — ce test emprunte le chemin de
/// l'application, décodage compris.
fn photo_a_bande() -> std::path::PathBuf {
    photo_d_epreuve("bande", (64, 64), |x, _| if x < 64 / 4 { 0 } else { 255 })
}

/// **Une photo d'épreuve, nommée par son contenu — et jamais réécrite.**
///
/// Les épreuves de ce fichier tournent en parallèle, et chacune réécrivait la même photo, sous
/// le même nom, à chaque appel : l'atelier d'une épreuve lisait parfois un fichier qu'une autre
/// venait de vider pour le réécrire. Un damier lu de travers donnait une texture de 65 × 65
/// au lieu de 64, ou des gris qui n'étaient pas ceux du processeur — « l'épreuve de la carte
/// qui tombe au hasard » (fiche 35 § 4). Ni la carte, ni le processeur : un fichier partagé.
///
/// Nommée par l'empreinte de ses pixels, une photo qui existe est déjà la bonne. Deux
/// écritures simultanées posent chacune le même fichier entier, par renommage.
fn photo_d_epreuve(
    nom: &str,
    (l, h): (u32, u32),
    gris: impl Fn(u32, u32) -> u8,
) -> std::path::PathBuf {
    static ECRITURES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let mut octets = Vec::with_capacity((l * h * 4) as usize);
    for y in 0..h {
        for x in 0..l {
            let v = gris(x, y);
            octets.extend_from_slice(&[v, v, v, 255]);
        }
    }
    let empreinte = glucose_core::hash::hex_of(&glucose_core::hash::sha256(&octets));
    let chemin = std::env::temp_dir().join(format!("glucose-voies-{nom}-{}.png", &empreinte[..16]));
    if !chemin.is_file() {
        let rang = ECRITURES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let a_cote = chemin.with_extension(format!("{}-{rang}.tmp", std::process::id()));
        image::save_buffer_with_format(
            &a_cote,
            &octets,
            l,
            h,
            image::ExtendedColorType::Rgba8,
            image::ImageFormat::Png,
        )
        .expect("ecriture de la photo d'epreuve");
        // Posée entre-temps par une autre épreuve ? La sienne vaut la nôtre.
        if std::fs::rename(&a_cote, &chemin).is_err() {
            let _ = std::fs::remove_file(&a_cote);
        }
    }
    chemin
}

/// Un document d'une seule photo à bande, cadrée d'un quart à gauche ou entière.
fn document_cadre(crop: glucose_core::types::Recadrage) -> glucose_core::store::Store {
    let mut store = glucose_core::store::Store::new("Recadrage");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = glucose_core::types::Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        };
    }
    let mut img = glucose_core::types::BoardImage::new("i0", 400.0, 300.0, 200.0, 200.0);
    img.src = Some(photo_a_bande().to_string_lossy().into_owned());
    img.crop = crop;
    store.add_image(&board, img);
    store.clear_selection();
    store
}

/// Les deux couches et ce que le processeur confie, avec un magasin qui a **fini** de décoder.
///
/// Le premier rendu réclame la photo au chantier ; on attend qu'il la livre ; le second rendu
/// est celui qu'on compare. C'est le chemin de l'application, en deux images au lieu de
/// quelques dizaines.
fn les_deux_couches_decodees(
    taille: (u32, u32),
    store: &glucose_core::store::Store,
) -> (Renderer, Pixmap, Pixmap, Confie) {
    les_deux_couches_sous_un_budget(taille, store, (None, None))
}

/// Les deux couches, quand la carte ne laisse aux photos que `part` octets (ETAGES-3), et
/// n'accepte pas de texture de plus de `plafond` pixels de côté (PLAFOND-1).
fn les_deux_couches_sous_un_budget(
    taille: (u32, u32),
    store: &glucose_core::store::Store,
    (part, plafond): (Option<u64>, Option<u32>),
) -> (Renderer, Pixmap, Pixmap, Confie) {
    let mut renderer = Renderer::new();
    renderer.carte.part_des_photos = part;
    renderer.carte.plafond_des_textures = plafond;
    renderer.sync_spatial_index(store);
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let rendre = |renderer: &mut Renderer, ui: &mut UiState| {
        let mut dessous = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        let mut dessus = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        dessous.fill(tiny_skia::Color::TRANSPARENT);
        dessus.fill(tiny_skia::Color::TRANSPARENT);
        let confie = renderer.rendre_les_couches(
            &mut dessous.as_mut(),
            &mut dessus.as_mut(),
            store,
            (ui, Pointer { x: 0.0, y: 0.0 }),
            SceneOverlay::sans_rien(&guides),
            Regard::immobile(),
        );
        (dessous, dessus, confie)
    };
    let _ = rendre(&mut renderer, &mut ui);
    renderer.magasin.attendre_le_chantier();
    let (dessous, dessus, confie) = rendre(&mut renderer, &mut ui);
    (renderer, dessous, dessus, confie)
}

/// La scène processeur, avec le même magasin décodé.
fn par_le_processeur_decode(taille: (u32, u32), store: &glucose_core::store::Store) -> Pixmap {
    let mut renderer = Renderer::new();
    renderer.sync_spatial_index(store);
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    let rendre = |renderer: &mut Renderer, ui: &mut UiState| {
        let mut pixmap = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        renderer.render(
            &mut pixmap.as_mut(),
            store,
            ui,
            SceneOverlay::sans_rien(&guides),
            Pointer { x: 0.0, y: 0.0 },
            Regard::immobile(),
        );
        pixmap
    };
    let _ = rendre(&mut renderer, &mut ui);
    renderer.magasin.attendre_le_chantier();
    rendre(&mut renderer, &mut ui)
}

fn pixel(p: &Pixmap, x: u32, y: u32) -> [u8; 4] {
    let i = (y * p.width() + x) as usize * 4;
    let d = p.data();
    [d[i], d[i + 1], d[i + 2], d[i + 3]]
}

/// **Une photo cadrée d'un quart se rend pareil des deux côtés**, et la bande a disparu des
/// deux.
///
/// La voie graphique cadre par la fenêtre de texture du quad ; la voie processeur par une
/// source posée plus grande et clippée à la boîte. Deux mécanismes sans rien de commun, et
/// c'est ce qui rend l'égalité probante : un même résultat obtenu par deux chemins n'a aucune
/// raison d'être le même s'il est faux.
///
/// Le contrepoint est dans le même test : sans recadrage, la bande est là des deux côtés.
#[test]
fn test_les_deux_voies_cadrent_pareil_et_la_bande_a_disparu() {
    let taille = (800u32, 600u32);
    let quart = glucose_core::types::Recadrage::depuis_les_marges(0.25, 0.0, 0.0, 0.0);
    let store = document_cadre(quart);
    let (renderer, dessous, dessus, confie) = les_deux_couches_decodees(taille, &store);
    let Some((peripherique, file)) = banc_gpu::carte() else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    // La source est celle de l'application (`app/peinture.rs`) : un composant se rend a la
    // demande, une PHOTO vient du magasin. La premiere version de ce test ne donnait que les
    // composants, et la carte ne montrait rien du tout -- une image a cote d'une autre l'a
    // dit en une lecture, la ou le pixel seul accusait le recadrage.
    let carte = banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        taille,
        &confie,
        (&dessous, &dessus),
        &|cle| confie.pixels(&renderer, cle),
    )
    .expect("la composition en cinq temps");
    let processeur = par_le_processeur_decode(taille, &store);

    // La boîte va de x = 300 à 500, y = 200 à 400. Un point dans ce qui était la bande.
    let (x, y) = (324u32, 300u32);
    assert_eq!(
        pixel(&processeur, x, y),
        [255, 255, 255, 255],
        "processeur : la bande doit avoir disparu"
    );
    assert_eq!(
        pixel(&carte, x, y),
        [255, 255, 255, 255],
        "carte : la bande doit avoir disparu"
    );

    // Et les deux voies restent d'accord sur l'écran entier, à l'écart déjà admis.
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "les deux voies divergent de {pire} niveaux sur une photo cadree"
    );
    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    let canaux = processeur.data().len();
    assert!(
        larges * 1000 <= canaux * PART_MAX_POUR_MILLE,
        "l'ecart depasse {ECART_COURANT} sur {larges} canaux sur {canaux}"
    );

    // Le contrepoint : sans recadrage, la bande est là — des deux côtés.
    let entiere = document_cadre(glucose_core::types::Recadrage::ENTIER);
    let temoin = par_le_processeur_decode(taille, &entiere);
    assert_eq!(
        pixel(&temoin, x, y),
        [0, 0, 0, 255],
        "sans recadrage, la bande noire doit etre la : sinon ce test ne regarde pas la photo"
    );
}

// ── De près : une carte plus grande que l'écran ───────────────────────────────────────────

/// Un document d'une seule carte de texte, vue à `echelle` avec son coin haut-gauche posé en
/// `coin` à l'écran.
fn document_de_pres(echelle: f64, coin: (f64, f64)) -> glucose_core::store::Store {
    let mut store = glucose_core::store::Store::new("De pres");
    let board = store.project.active_board_id.clone();
    let (x, y) = (100.0, 100.0);
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = glucose_core::types::Viewport {
            x: coin.0 - x * echelle,
            y: coin.1 - y * echelle,
            scale: echelle,
        };
    }
    store.add_annotation(
        &board,
        glucose_core::types::Annotation::Text {
            id: "de-pres".to_string(),
            x,
            y,
            width: Some(240.0),
            height: Some(60.0),
            text: "Un texte qu'on lit de tres pres, accents compris : éàçù.".to_string(),
            font_size: Some(14.0),
            color: None,
            cursor_pos: None,
            source_file: None,
            membrane_id: None,
            domains: Vec::new(),
            mirror_of: None,
            temporal_anchor: None,
        },
    );
    store.clear_selection();
    store
}

/// **Une carte plus grande que l'écran se dessine sur les deux voies.**
///
/// C'est le défaut que l'utilisateur a montré le 23/09 : *« lorsqu'on est trop proche d'un
/// texte, celui-ci ne veut tout simplement pas s'afficher »*. La voie graphique refusait d'en
/// faire une texture — elle aurait dépassé l'écran — et personne ne la dessinait à la place ;
/// il ne restait que sa lueur et la grille. La fiche 24 § 13 l'avait prédit sans pouvoir
/// l'établir, faute de cette épreuve-ci.
///
/// # SLUG-1 — de près, les deux voies ne diffèrent qu'au bord des lettres (fiche 73)
///
/// De près, la carte graphique dessine la carte depuis ses formes : chaque lettre par Slug, à sa
/// place exacte, filtrée le long de deux rayons. Le processeur la pose au quart de pixel
/// (GLYPH-1) et la filtre par son aire. Ce sont deux anticrénelages fidèles — aucun ne grossit
/// ni n'amincit la lettre —, mais ils ne tombent pas au bit près sur un bord : un huitième de
/// pixel de place, un dixième de couverture sur un bord oblique (mesurés : écart signé moyen
/// nul, tous les écarts sur un bord). La loi tient donc ce qui compte : **un écart de plus de
/// trois niveaux ne tombe que sur un pixel que l'une des deux voies couvre en partie** — un
/// mélange de deux de ses voisins opposés ([`couvert_en_partie`]). Une carte absente, une
/// lettre manquante ou décalée d'un pixel donnent des pixels pleins d'un côté et vides de
/// l'autre : ceux-là tombent. Ailleurs, la règle de toute la suite.
///
/// Deux vues : une où le coin de la carte est à l'écran, une où elle le couvre entièrement.
#[test]
fn test_une_carte_plus_grande_que_l_ecran_se_dessine_sur_les_deux_voies() {
    let taille = (800u32, 600u32);
    for (echelle, coin) in [(6.0, (150.37, 180.61)), (14.0, (-900.2, -500.9))] {
        let store = document_de_pres(echelle, coin);
        let Some(carte) = par_la_carte(taille, &store) else {
            eprintln!("aucune carte utilisable : epreuve sautee");
            return;
        };
        let processeur = par_le_processeur(taille, &store);
        slug_1(&format!("a l'echelle {echelle}"), &processeur, &carte);
    }
}

/// **SLUG-1** : un écart de plus de trois niveaux ne tombe que sur un pixel que l'une des voies
/// couvre en partie — au plus la part de la suite ailleurs —, et l'encre est là des deux côtés.
fn slug_1(nom: &str, processeur: &Pixmap, carte: &Pixmap) {
    let encre = encre_des_lettres(processeur);
    let mut hors_des_bords = 0;
    let largeur = processeur.width() as usize;
    for (i, (a, b)) in processeur.pixels().iter().zip(carte.pixels()).enumerate() {
        let ecarts = [
            a.red().abs_diff(b.red()),
            a.green().abs_diff(b.green()),
            a.blue().abs_diff(b.blue()),
        ];
        let larges = ecarts.iter().filter(|&&e| e > ECART_COURANT).count();
        let (x, y) = ((i % largeur) as u32, (i / largeur) as u32);
        if larges > 0 && !couvert_en_partie(processeur, x, y) && !couvert_en_partie(carte, x, y) {
            hors_des_bords += larges;
        }
    }
    let canaux = processeur.data().len();
    assert!(
        hors_des_bords * 1000 <= canaux * PART_MAX_POUR_MILLE,
        "{nom}, {hors_des_bords} canaux s'ecartent de plus de {ECART_COURANT} hors du bord des \
         lettres : une forme manque sur une des deux voies"
    );
    // Et les lettres sont à leur place : à un huitième de pixel près (GLYPH-1).
    let borne = 1.0 / (2.0 * f64::from(SUBPIXEL_PHASES));
    let [dx, dy] = deplacement(processeur, carte);
    assert!(
        dx.abs() <= borne && dy.abs() <= borne,
        "{nom} : la carte se deplace de ({dx:.3}, {dy:.3}) pixel, au-dela de {borne}"
    );
    // L'épreuve regarde bien des lettres : l'encre est là, des deux côtés.
    let pleins = |image: &Pixmap| {
        image
            .pixels()
            .iter()
            .filter(|p| [p.red(), p.green(), p.blue()] == encre)
            .count()
    };
    assert!(
        pleins(processeur) > 2000 && pleins(carte) > 2000,
        "{nom} : l'encre manque"
    );
}

/// **De près, un passage qui brille et une sélection qu'on écrit se dessinent pareil sur les
/// deux voies** (SLUG-1, fiche 73) : le fond et le liseré du passage, le surlignage de la
/// sélection, que le dessin vectoriel a reçus en devenant celui de la carte. Leurs couleurs ne
/// sont ni le fond ni l'encre : ils tiennent sous trois niveaux, comme le reste.
#[test]
fn test_slug_1_de_pres_un_passage_et_une_selection_se_dessinent_pareil() {
    use glucose_desktop::params::Eclairage;
    let taille = (800u32, 600u32);
    let store = document_de_pres(6.0, (150.37, 180.61));
    let texte = "Un texte qu'on lit de tres pres, accents compris : éàçù.";
    let debut = texte.find("texte").expect("le passage");
    let eclairages = [Eclairage {
        carte: "de-pres".to_string(),
        plages: vec![(debut, debut + 5)],
        teinte: None,
    }];
    let session = glucose_desktop::renderer::TextEditSession {
        ann_id: "de-pres".to_string(),
        buffer: texte.to_string(),
        selection: glucose_core::text::Selection::spanning(3, 14),
        goal_x: None,
        blink_timer: std::time::Instant::now(),
        curseur_visible: false,
    };
    let guides = glucose_core::smart_align::SnapGuides::default();
    let lumiere = SceneOverlay {
        eclairages: &eclairages,
        ..SceneOverlay::sans_rien(&guides)
    };
    let saisie = SceneOverlay {
        editing: Some(&session),
        ..SceneOverlay::sans_rien(&guides)
    };
    for (nom, overlay) in [("le passage", lumiere), ("la selection", saisie)] {
        let Some((processeur, carte)) = deux_voies_sous(taille, &store, overlay) else {
            eprintln!("aucune carte utilisable : epreuve sautee");
            return;
        };
        slug_1(nom, &processeur, &carte);
    }
}

/// La même scène par les deux voies, sous ce que l'interface y pose.
fn deux_voies_sous(
    taille: (u32, u32),
    store: &glucose_core::store::Store,
    overlay: SceneOverlay<'_>,
) -> Option<(Pixmap, Pixmap)> {
    let mut processeur = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    Renderer::new().render(
        &mut processeur.as_mut(),
        store,
        &mut UiState::new(),
        overlay,
        Pointer { x: 0.0, y: 0.0 },
        Regard::immobile(),
    );
    let mut dessous = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    let mut dessus = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    let mut renderer = Renderer::new();
    let confie = renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        store,
        (&mut UiState::new(), Pointer { x: 0.0, y: 0.0 }),
        overlay,
        Regard::immobile(),
    );
    let (peripherique, file) = banc_gpu::carte()?;
    let carte = banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        taille,
        &confie,
        (&dessous, &dessus),
        &|cle| confie.pixels(&renderer, cle),
    )?;
    Some((processeur, carte))
}

/// **De combien `b` est déplacée par rapport à `a`**, en pixels, sur chaque axe : la translation
/// `δ` qui explique le mieux `b ≈ a − δ · ∇a`, par les moindres carrés (Lucas et Kanade, sur
/// l'image entière, la luminance somme des trois canaux).
///
/// # Pourquoi une loi de place, en plus de la loi des bords
///
/// Une loi qui excuse un écart « sur un pixel couvert en partie » excuse aussi une lettre
/// décalée d'un pixel : les pixels qu'elle déplace sont eux aussi des pixels de bord. Le
/// sabotage l'a montré (fiche 73). La place, elle, a une borne qui ne se choisit pas : le
/// processeur pose chaque lettre au quart de pixel ([`SUBPIXEL_PHASES`] phases par axe), donc
/// à un huitième de pixel au plus de sa place exacte — et une moyenne de ces erreurs aussi.
/// Mesuré : 0,047 et 0,021 pixel ; une lettre décalée d'un pixel en donne près d'un.
fn deplacement(a: &Pixmap, b: &Pixmap) -> [f64; 2] {
    let (l, h) = (a.width() as usize, a.height() as usize);
    let lum = |p: &Pixmap, x: usize, y: usize| {
        let q = p.pixels()[y * l + x];
        f64::from(q.red()) + f64::from(q.green()) + f64::from(q.blue())
    };
    let (mut xx, mut xy, mut yy, mut ex, mut ey) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for y in 1..h - 1 {
        for x in 1..l - 1 {
            let gx = (lum(a, x + 1, y) - lum(a, x - 1, y)) / 2.0;
            let gy = (lum(a, x, y + 1) - lum(a, x, y - 1)) / 2.0;
            let e = lum(b, x, y) - lum(a, x, y);
            (xx, xy, yy) = (xx + gx * gx, xy + gx * gy, yy + gy * gy);
            (ex, ey) = (ex + e * gx, ey + e * gy);
        }
    }
    let det = xx * yy - xy * xy;
    [-(yy * ex - xy * ey) / det, -(xx * ey - xy * ex) / det]
}

/// **L'encre des lettres**, lue sur l'image : la plus fréquente des couleurs claires — le cœur
/// des lettres.
fn encre_des_lettres(image: &Pixmap) -> [u8; 3] {
    let mut comptes = std::collections::HashMap::<[u8; 3], usize>::new();
    for p in image.pixels() {
        *comptes.entry([p.red(), p.green(), p.blue()]).or_default() += 1;
    }
    comptes
        .into_iter()
        .filter(|(c, _)| c.iter().map(|&v| u32::from(v)).sum::<u32>() > 3 * 128)
        .max_by_key(|&(_, n)| n)
        .map(|(c, _)| c)
        .expect("une image qui a des pixels clairs")
}

/// **Ce pixel est-il couvert en partie par une forme ?** — un mélange de deux de ses voisins
/// opposés (à gauche et à droite, en haut et en bas, ou sur une diagonale), à trois niveaux
/// près, qui n'est ni l'un ni l'autre.
///
/// C'est ce que « couvert en partie » veut dire, quels que soient le fond et l'encre : une
/// lettre blanche sur la brume, une lettre teintée sur le fond d'un passage, une lettre sur le
/// surlignage d'une sélection, le bord d'un liseré. Lire le fond et l'encre sur toute l'image
/// ne reconnaissait que le premier cas — et comptait tous les autres comme des fautes.
fn couvert_en_partie(image: &Pixmap, x: u32, y: u32) -> bool {
    let (l, h) = (image.width() as i64, image.height() as i64);
    let lire = |x: i64, y: i64| {
        (x >= 0 && y >= 0 && x < l && y < h)
            .then(|| pixel(image, x as u32, y as u32))
            .map(|p| [p[0], p[1], p[2]].map(f32::from))
    };
    let Some(c) = lire(x.into(), y.into()) else {
        return false;
    };
    let (x, y) = (i64::from(x), i64::from(y));
    let tolerance = f32::from(ECART_COURANT);
    [(1, 0), (0, 1), (1, 1), (1, -1)].iter().any(|&(dx, dy)| {
        let (Some(a), Some(b)) = (lire(x - dx, y - dy), lire(x + dx, y + dy)) else {
            return false;
        };
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        // Deux voisins égaux n'ont pas de segment : `t` n'est pas un nombre, et rien ne passe.
        let longueur = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let t = ((c[0] - a[0]) * d[0] + (c[1] - a[1]) * d[1] + (c[2] - a[2]) * d[2])
            / (longueur * longueur);
        let sur_le_segment = (0..3).all(|k| (c[k] - (a[k] + t * d[k])).abs() <= tolerance);
        let marge = tolerance / longueur;
        sur_le_segment && t > marge && t < 1.0 - marge
    })
}

// ── NIVEAU-GPU-1 : la carte reçoit le niveau qui couvre la taille posée ────────────────────

/// Un damier d'un pixel, 512 × 512, écrit sur le disque : le pire cas du crénelage. Lu un
/// texel sur huit, il donne des motifs ; moyenné comme la pyramide le fait, du gris.
fn photo_damier() -> std::path::PathBuf {
    photo_d_epreuve(
        "damier",
        (512, 512),
        |x, y| {
            if (x + y) % 2 == 0 {
                0
            } else {
                255
            }
        },
    )
}

/// Le damier posé en 200 × 200 dans le monde, vu à l'échelle `echelle`.
fn document_damier(echelle: f64) -> glucose_core::store::Store {
    let mut store = glucose_core::store::Store::new("Niveau");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = glucose_core::types::Viewport {
            x: 400.0 - 400.0 * echelle,
            y: 300.0 - 300.0 * echelle,
            scale: echelle,
        };
    }
    let mut img = glucose_core::types::BoardImage::new("damier", 400.0, 300.0, 200.0, 200.0);
    img.src = Some(photo_damier().to_string_lossy().into_owned());
    store.add_image(&board, img);
    store.clear_selection();
    store
}

/// **Une photo posée petite part au niveau qui la couvre, et non en natif** (NIVEAU-GPU-1).
///
/// Soixante pixels à l'écran : le niveau de soixante-quatre, huit fois réduit. La voie
/// graphique envoyait les 512 × 512 natifs — pour une épingle en original, 27 Mo et treize
/// millisecondes de processeur par envoi (`bench_televersement`). Et la texture garde le
/// fichier pour identité : quand le niveau change, l'ancien reste posé pendant que le nouveau
/// se téléverse.
#[test]
fn test_une_photo_posee_petite_part_au_niveau_qui_la_couvre() {
    let store = document_damier(0.3);
    let (renderer, _, _, confie) = les_deux_couches_decodees((800, 600), &store);
    let cle = &confie.photos.first().expect("la photo est posee").cle;
    let texture = confie.pixels(&renderer, cle).expect("ses pixels");
    let texture = texture.vue();
    assert_eq!(
        (texture.width(), texture.height()),
        (64, 64),
        "soixante pixels a l'ecran : le niveau de soixante-quatre, pas les 512 natifs"
    );
    let src = photo_damier().to_string_lossy().into_owned();
    let numero = renderer
        .magasin
        .cache
        .numero_connu(&src)
        .expect("son numéro");
    let identite = confie
        .textures()
        .into_iter()
        .find(|t| &t.cle == cle)
        .map(|t| t.identite);
    assert_eq!(
        identite,
        Some(glucose_desktop::present::scene_gpu::Cle::Photo(numero)),
        "l'identite reste le fichier : un changement de niveau garde l'ancien pose"
    );
}

/// **PLAFOND-1 — jamais une texture plus grande que la carte n'accepte.** Le damier posé à
/// 800 pixels voudrait ses 512 natifs ; une carte qui n'accepte que 128 de côté reçoit le
/// niveau de 128 — la photo se pose, réduite, au lieu d'une texture refusée. À l'envers : sans
/// plafond, les 512 natifs.
#[test]
fn test_plafond_1_jamais_une_texture_plus_grande_que_la_carte() {
    let store = document_damier(4.0);
    let cote = |plafond: Option<u32>| {
        let (renderer, _, _, confie) =
            les_deux_couches_sous_un_budget((800, 600), &store, (None, plafond));
        let cle = &confie.photos.first().expect("la photo est posee").cle;
        let texture = confie.pixels(&renderer, cle).expect("ses pixels");
        let vue = texture.vue();
        vue.width().max(vue.height())
    };
    assert_eq!(cote(None), 512, "sans plafond, les natifs");
    assert_eq!(cote(Some(128)), 128, "le plus grand niveau qui tient");
    assert_eq!(cote(Some(200)), 128, "200 ne tient pas 256 : 128");
}

/// **Une photo réduite se rend pareil sur les deux voies** — ce que la texture native, lue un
/// texel sur huit par le filtre bilinéaire, ne pouvait pas faire : le damier y devenait des
/// motifs là où le processeur, qui part du niveau, montre du gris.
#[test]
fn test_une_photo_reduite_se_rend_pareil_sur_les_deux_voies() {
    let taille = (800u32, 600u32);
    let store = document_damier(0.3);
    let (renderer, dessous, dessus, confie) = les_deux_couches_decodees(taille, &store);
    let Some((peripherique, file)) = banc_gpu::carte() else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let carte = banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        taille,
        &confie,
        (&dessous, &dessus),
        &|cle| confie.pixels(&renderer, cle),
    )
    .expect("la composition en cinq temps");
    let processeur = par_le_processeur_decode(taille, &store);
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    let canaux = processeur.data().len();
    assert!(
        larges * 1000 <= canaux * PART_MAX_POUR_MILLE,
        "l'ecart depasse {ECART_COURANT} sur {larges} canaux sur {canaux} (pire {pire}) : la \
         carte ne lit pas le niveau que le processeur lit"
    );
    assert!(pire <= ECART_ADMIS, "pire ecart {pire}");
}

/// **Quand la carte manque de place, la photo perd un cran** (ETAGES-3) — par le vrai rendu,
/// sur ce que la carte reçoit.
///
/// Soixante pixels à l'écran demandent le niveau de 64 : 16 Kio. Une part d'un octet de
/// moins, et c'est celui de 32 qui part. Une part nulle, et même le pixel ne tient pas : la
/// carte le dit, et l'image se composera sur le processeur.
#[test]
fn test_quand_la_carte_manque_de_place_la_photo_perd_un_cran() {
    let store = document_damier(0.3);
    let niveau_recu = |part: Option<u64>| {
        let (renderer, _, _, confie) =
            les_deux_couches_sous_un_budget((800, 600), &store, (part, None));
        let cle = confie
            .photos
            .first()
            .expect("la photo est posee")
            .cle
            .clone();
        let texture = confie.pixels(&renderer, &cle).expect("ses pixels");
        let texture = texture.vue();
        ((texture.width(), texture.height()), renderer.carte.debordee)
    };
    assert_eq!(
        niveau_recu(None),
        ((64, 64), false),
        "sans budget, la regle d'avant"
    );
    assert_eq!(
        niveau_recu(Some(64 * 64 * 4)),
        ((64, 64), false),
        "juste assez"
    );
    assert_eq!(
        niveau_recu(Some(64 * 64 * 4 - 1)),
        ((32, 32), false),
        "un cran"
    );
    let (renderer, _, _, _) = les_deux_couches_sous_un_budget((800, 600), &store, (Some(0), None));
    assert!(renderer.carte.debordee, "rien ne tient : la carte le dit");
}

// ── Une carte posée sur une photo (LUEUR-3) ────────────────────────────────────────────────

/// La photo à bande entière — de x = 300 à 500, y = 200 à 400, blanche à droite de x = 350 —,
/// et, si on la demande, une carte de texte posée dessus.
fn document_carte_sur_photo(avec_carte: bool) -> glucose_core::store::Store {
    let mut store = document_cadre(glucose_core::types::Recadrage::ENTIER);
    if avec_carte {
        let board = store.project.active_board_id.clone();
        store.add_annotation(
            &board,
            glucose_core::types::Annotation::text("c", 360.0, 230.0, "Lisible"),
        );
        store.clear_selection();
    }
    store
}

/// **Une carte posée sur une photo la cache, et sa lueur passe devant elle** — sur les deux
/// voies.
///
/// Sa capture du 26/09 : une carte sur une peinture claire, son texte blanc illisible, et
/// « la zone de blur derrière les images ». Les deux étaient vrais : la lueur se posait sous
/// les photos, et l'intérieur de la carte ne portait que 3 % de sa teinte — la photo passait
/// au travers. La carte peint désormais son papier (le gris des blocs, PAPIER-1), et sa lueur
/// se pose entre les photos et les cartes.
#[test]
fn test_lueur_3_une_carte_sur_une_photo_la_cache_et_sa_lueur_passe_devant() {
    let taille = (800u32, 600u32);
    let avec = document_carte_sur_photo(true);
    let processeur = par_le_processeur_decode(taille, &avec);
    let photo_seule = par_le_processeur_decode(taille, &document_carte_sur_photo(false));

    // Dans la carte, loin de son texte et de ses coins, au-dessus du blanc de la photo : le
    // fond de la carte, sombre — et non la photo.
    let (dedans, dessous) = ((470u32, 272u32), (450u32, 300u32));
    assert_eq!(
        pixel(&photo_seule, dedans.0, dedans.1),
        [255, 255, 255, 255],
        "sans la carte, la photo est blanche ici : sinon ce test ne regarde pas la photo"
    );
    let fond = pixel(&processeur, dedans.0, dedans.1);
    assert!(
        fond[..3].iter().all(|&c| c < 60),
        "dans la carte, la photo se voit encore : {fond:?}"
    );
    // Juste sous la carte, sur la photo : la lueur passe devant l'image.
    assert_ne!(
        pixel(&processeur, dessous.0, dessous.1),
        pixel(&photo_seule, dessous.0, dessous.1),
        "la lueur de la carte reste cachee sous la photo"
    );

    // Et la voie graphique pose la même chose, à l'écart admis.
    let (renderer, dessous_c, dessus_c, confie) = les_deux_couches_decodees(taille, &avec);
    let Some((peripherique, file)) = banc_gpu::carte() else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let carte = banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        taille,
        &confie,
        (&dessous_c, &dessus_c),
        &|cle| confie.pixels(&renderer, cle),
    )
    .expect("la composition en cinq temps");
    let fond_gpu = pixel(&carte, dedans.0, dedans.1);
    assert!(
        fond_gpu[..3].iter().all(|&c| c < 60),
        "carte graphique : la photo se voit dans la carte : {fond_gpu:?}"
    );
    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    let canaux = processeur.data().len();
    assert!(
        larges * 1000 <= canaux * PART_MAX_POUR_MILLE,
        "l'ecart depasse {ECART_COURANT} sur {larges} canaux sur {canaux}"
    );
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(pire <= ECART_ADMIS, "pire ecart {pire}");
}

/// Une membrane violette, et, si on la demande, une carte bleue posée en son milieu.
fn document_carte_dans_une_membrane(avec_carte: bool) -> glucose_core::store::Store {
    use glucose_core::types::Annotation;
    let mut store = glucose_core::store::Store::new("Contenant");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = glucose_core::types::Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        };
    }
    let mut membrane = Annotation::membrane("m", 100.0, 100.0, 600.0, 400.0);
    if let Annotation::Membrane { color, .. } = &mut membrane {
        *color = Some("#b050ff".to_string());
    }
    store.add_annotation(&board, membrane);
    if avec_carte {
        let mut carte = Annotation::text("c", 280.0, 260.0, "Dedans");
        if let Annotation::Text { color, .. } = &mut carte {
            *color = Some("#20c0ff".to_string());
        }
        store.add_annotation(&board, carte);
    }
    store.clear_selection();
    store
}

/// **PAPIER-1 — le papier d'un texte est le gris des blocs, sur les deux voies** (fiche 105).
///
/// Une carte cache ce qu'elle recouvre ; elle cachait aussi, depuis LUEUR-3, ce qui la contient —
/// son papier partait de ce que la membrane peignait là, puis COULEUR-3 le teintait de la couleur
/// choisie. Sa parole du 11/11/2026 : le fond d'un texte est *« TOUJOURS gris »*, *« le gris sombre
/// et neutre des blocs »*. Une carte d'une couleur choisie, dans une membrane violette : son papier
/// est le jeton du thème, sur le processeur comme sur la carte graphique.
///
/// **La preuve à l'envers** : sans la carte, la membrane teinte vraiment ce pixel — l'ancienne loi
/// l'aurait donné au papier.
#[test]
fn test_papier_1_le_papier_d_une_carte_dans_une_membrane_est_le_gris_des_blocs() {
    let taille = (800u32, 600u32);
    let avec = par_le_processeur(taille, &document_carte_dans_une_membrane(true));
    let sans = par_le_processeur(taille, &document_carte_dans_une_membrane(false));
    let gris = glucose_desktop::theme::Theme::dark()
        .card_paper
        .to_color_u8();
    let gris = [gris.red(), gris.green(), gris.blue()];
    // Dans la carte, loin de son texte et de ses coins ; et un pixel sans point de grille :
    // ses deux voisins disent la même chose que lui.
    let p = (470u32, 300u32);
    let la = pixel(&sans, p.0, p.1);
    assert_eq!(
        la,
        pixel(&sans, p.0 + 3, p.1),
        "un point de grille sous la mesure"
    );
    assert!(
        la[0].abs_diff(gris[0]) + la[2].abs_diff(gris[2]) > 10,
        "la membrane ne teinte pas ce pixel ({la:?}) : sinon ce test ne la regarde pas"
    );
    let vu = pixel(&avec, p.0, p.1);
    assert_eq!(&vu[..3], &gris[..], "le papier de la carte, au processeur");
    let Some(carte) = par_la_carte(taille, &document_carte_dans_une_membrane(true)) else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let vu_gpu = pixel(&carte, p.0, p.1);
    for k in 0..3 {
        assert!(
            vu_gpu[k].abs_diff(vu[k]) <= ECART_COURANT,
            "carte graphique : {vu_gpu:?}, processeur : {vu:?}"
        );
    }
}

// ── Les membranes en carte (REGION-1) ──────────────────────────────────────────────────────

/// Une mère rouge, une fille bleue dedans, une case de storyboard dans la mère — vues au double,
/// la grille de points bien visible.
fn document_regions() -> glucose_core::store::Store {
    use glucose_core::types::Annotation;
    let mut store = glucose_core::store::Store::new("Régions");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = glucose_core::types::Viewport {
            x: 300.0,
            y: 250.0,
            scale: 2.0,
        };
    }
    for (id, boite, couleur) in [
        ("m", (-300.0, -200.0, 600.0, 400.0), "#ef4444"),
        ("f", (-40.0, -120.0, 300.0, 200.0), "#3b82f6"),
    ] {
        let (x, y, l, h) = boite;
        let mut m = Annotation::membrane(id, x, y, l, h);
        if let Annotation::Membrane { color, .. } = &mut m {
            *color = Some(couleur.to_string());
        }
        store.add_annotation(&board, m);
    }
    let grille = glucose_core::store::storyboard::Grille {
        largeur: 80.0,
        colonnes: 1,
        ..Default::default()
    };
    store.poser_le_storyboard(&board, grille, (-250.0, 60.0));
    store.clear_selection();
    store
}

/// **Les deux voies posent le sol des régions, puis la grille et les zones, puis les frontières**
/// (REGION-1, fiche 105) : la même image, sur toute la surface — la grille de points dans les deux
/// membranes, la case dans la mère, les frontières par-dessus. Une passe à son ancien rang (la
/// grille ou les zones sous le sol, sur la carte graphique) laisse des centaines de points de
/// trente à quarante niveaux d'écart.
#[test]
fn test_region_1_les_deux_voies_posent_le_sol_la_grille_et_les_frontieres_dans_le_meme_ordre() {
    let taille = (800u32, 600u32);
    let store = document_regions();
    let processeur = par_le_processeur(taille, &store);
    let Some(carte) = par_la_carte(taille, &store) else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    let canaux = processeur.data().len();
    assert!(
        larges * 1000 <= canaux * PART_MAX_POUR_MILLE,
        "l'ecart depasse {ECART_COURANT} sur {larges} canaux sur {canaux}"
    );
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(pire <= ECART_ADMIS, "pire ecart {pire}");
}

// ── Un passage qui brille (PASSAGE-2) ──────────────────────────────────────────────────────

/// **Un passage qui brille se rend pareil sur les deux voies** : son fond, son liseré et ses
/// lettres teintées sont dans la texture de la carte, sa lueur au-dessus — et la place que son
/// cadre ouvre est la même des deux côtés. Et la texture change avec lui : survoler une flèche
/// refait la texture de sa carte.
#[test]
fn test_passage_2_un_passage_qui_brille_se_rend_pareil_sur_les_deux_voies() {
    use glucose_desktop::params::Eclairage;
    let taille = (800u32, 600u32);
    let mut store = glucose_core::store::Store::new("Passage");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = glucose_core::types::Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.5,
        };
    }
    let texte = "le chat mangeait la souris";
    store.add_annotation(
        &board,
        glucose_core::types::Annotation::text("c", 60.0, 80.0, texte),
    );
    store.clear_selection();
    let debut = texte.find("mange").expect("le passage");
    let eclairages = [Eclairage {
        carte: "c".to_string(),
        plages: vec![(debut + 2, debut + 5)],
        teinte: None,
    }];
    let guides = glucose_core::smart_align::SnapGuides::default();
    let overlay = SceneOverlay {
        eclairages: &eclairages,
        ..SceneOverlay::sans_rien(&guides)
    };

    let mut processeur = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    Renderer::new().render(
        &mut processeur.as_mut(),
        &store,
        &mut UiState::new(),
        overlay,
        Pointer { x: 0.0, y: 0.0 },
        Regard::immobile(),
    );
    let couches = |overlay: SceneOverlay<'_>| {
        let mut dessous = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        let mut dessus = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        let mut renderer = Renderer::new();
        let confie = renderer.rendre_les_couches(
            &mut dessous.as_mut(),
            &mut dessus.as_mut(),
            &store,
            (&mut UiState::new(), Pointer { x: 0.0, y: 0.0 }),
            overlay,
            Regard::immobile(),
        );
        (renderer, dessous, dessus, confie)
    };
    let (renderer, dessous, dessus, confie) = couches(overlay);
    let (_, _, _, sans) = couches(SceneOverlay::sans_rien(&guides));
    let cles = |c: &Confie| c.cartes.iter().map(|t| t.cle.clone()).collect::<Vec<_>>();
    assert_ne!(
        cles(&confie),
        cles(&sans),
        "la texture de la carte ne change pas quand son passage brille"
    );
    let Some((peripherique, file)) = banc_gpu::carte() else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let carte = banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        taille,
        &confie,
        (&dessous, &dessus),
        &|cle| confie.pixels(&renderer, cle),
    )
    .expect("la composition en cinq temps");
    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    let canaux = processeur.data().len();
    assert!(
        larges * 1000 <= canaux * PART_MAX_POUR_MILLE,
        "l'ecart depasse {ECART_COURANT} sur {larges} canaux sur {canaux}"
    );
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(pire <= ECART_ADMIS, "pire ecart {pire}");
}

/// **PANNEAUX-1 — un panneau posé par la carte donne l'image de la voie processeur.**
///
/// Sur la voie graphique, la Time Machine n'est plus composée dans la couche du dessus : la
/// carte la pose par-dessus, comme une texture. Sur la voie processeur, elle se compose sur
/// l'image finie. Les deux doivent rendre la même image, et le panneau doit s'y voir — une
/// épreuve qui comparerait seulement passerait aussi le jour où aucune voie ne le poserait.
#[test]
fn test_panneaux_1_un_panneau_se_pose_pareil_sur_les_deux_voies() {
    use glucose_desktop::dock::{
        confier_les_docks, render_docks, DockCache, DockManager, DockPass, TabId,
    };
    use glucose_desktop::params::ScreenFrame;
    let taille = synth::WITNESS_SIZE;
    let store = synth::witness_selected();
    let mut dock = DockManager::new();
    dock.top_tabs.clear();
    dock.bottom_tabs.clear();
    dock.right_tabs.clear();
    dock.toggle_tab(TabId::Temps);
    let (typo, theme) = (
        glucose_desktop::typography::Typography::new(),
        glucose_desktop::theme::Theme::dark(),
    );
    let passe = |cache| DockPass {
        typo: &typo,
        theme: &theme,
        screen: ScreenFrame {
            width: taille.0 as f32,
            height: taille.1 as f32,
            header_h: UiState::new().header_height(),
            scale: 1.0,
        },
        pointer: Pointer { x: -1.0, y: -1.0 },
        cache,
    };

    let (renderer, dessous, mut dessus, mut confie) = les_deux_couches(taille, &store);
    let cache = DockCache::new();
    confie.panneaux = confier_les_docks(&mut dessus.as_mut(), &dock, &store, &passe(Some(&cache)));
    assert_eq!(confie.panneaux.len(), 1);
    let Some((peripherique, file)) = banc_gpu::carte() else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let carte = banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        taille,
        &confie,
        (&dessous, &dessus),
        &|cle| {
            confie.pixels(&renderer, cle).or_else(|| {
                cache
                    .pixels(cle.nom()?)
                    .map(glucose_desktop::present::scene_gpu::Pixels::Rendues)
            })
        },
    )
    .expect("une image");

    let sans_panneau = par_le_processeur(taille, &store);
    let mut processeur = sans_panneau.clone();
    render_docks(&mut processeur.as_mut(), &dock, &store, &passe(None));

    let pose = confie.panneaux[0].pose;
    let (cx, cy) = (
        (pose.x + pose.largeur / 2.0) as u32,
        (pose.y + pose.hauteur / 2.0) as u32,
    );
    assert_ne!(
        pixel(&carte, cx, cy),
        pixel(&sans_panneau, cx, cy),
        "le panneau se voit sur la carte"
    );
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    assert!(
        pire <= ECART_ADMIS && larges * 1000 <= processeur.data().len() * PART_MAX_POUR_MILLE,
        "les deux voies divergent : pire {pire}, {larges} canaux au-dela de {ECART_COURANT}"
    );
}

/// **PRESET-1 — les zones d'un gabarit se posent pareil sur les deux voies** : sous tout le
/// contenu, par la couche du dessous que la carte pose sous ses photos. Et elles se voient —
/// une épreuve qui compare deux voies passerait aussi le jour où aucune ne les dessinerait.
#[test]
fn test_preset_1_les_zones_se_posent_pareil_sur_les_deux_voies() {
    let taille = synth::WITNESS_SIZE;
    let sans = synth::witness_selected();
    let mut store = sans.clone();
    let board = store.project.active_board_id.clone();
    let gabarit = glucose_core::gabarits::integre("props").expect("un gabarit intégré");
    let categories: Vec<&str> = gabarit.categories.iter().map(|c| c.id).collect();
    let vp = store.viewport();
    let centre = (
        (f64::from(taille.0) / 2.0 - vp.x) / vp.scale,
        (f64::from(taille.1) / 2.0 - vp.y) / vp.scale,
    );
    store.poser_un_gabarit(&board, (gabarit.id, &categories), centre);

    let processeur = par_le_processeur(taille, &store);
    assert!(
        banc_gpu::pire_ecart(&processeur, &par_le_processeur(taille, &sans)) > 0,
        "les zones se voient"
    );
    let Some(carte) = par_la_carte(taille, &store) else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "les deux voies divergent de {pire} niveaux"
    );
    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    assert!(
        larges * 1000 <= processeur.data().len() * PART_MAX_POUR_MILLE,
        "l'ecart depasse {ECART_COURANT} sur {larges} canaux"
    );
}

/// **STORYBOARD-1 — les cases se posent pareil sur les deux voies** : le cadre, son bord, la
/// bande et le badge en rectangles que la carte pose au premier rang ; le numéro et la
/// description dans la couche du dessous. Comparées là où elles sont, sans aucun canal au-delà
/// d'un arrondi : un badge oublié par la carte passerait sous une part tolérée de l'écran.
#[test]
fn test_storyboard_1_les_cases_se_posent_pareil_sur_les_deux_voies() {
    use glucose_core::store::storyboard::{Format, Grille, DESCRIPTION};
    let taille = synth::WITNESS_SIZE;
    let sans = synth::witness_selected();
    let mut store = sans.clone();
    let board = store.project.active_board_id.clone();
    let vp = store.viewport();
    let centre = (
        (f64::from(taille.0) / 2.0 - vp.x) / vp.scale,
        (f64::from(taille.1) / 2.0 - vp.y) / vp.scale,
    );
    let grille = Grille {
        format: Format::Cinema,
        largeur: 260.0,
        colonnes: 3,
        ecart: 24.0,
    };
    let (l, h) = grille.etendue(grille.nombre_initial());
    let sb = store
        .poser_le_storyboard(&board, grille, (centre.0 - l / 2.0, centre.1 - h / 2.0))
        .expect("posé");
    let premiere = store.cases(&board, &sb)[0].id.clone();
    store.ecrire_dans_la_case(
        &board,
        &premiere,
        glucose_core::store::storyboard::Texte::Description,
        "Plan large, la ville au loin",
    );

    let processeur = par_le_processeur(taille, &store);
    assert!(
        banc_gpu::pire_ecart(&processeur, &par_le_processeur(taille, &sans)) > 0,
        "les cases se voient"
    );
    let Some(carte) = par_la_carte(taille, &store) else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let largeur = processeur.width() as usize * 4;
    let mut ecarts = 0;
    for case in store.cases(&board, &sb) {
        let (x0, y0) = glucose_desktop::canvas::world_to_screen(case.x, case.y, &vp);
        let (w, h) = (
            case.width * vp.scale,
            (case.height + DESCRIPTION) * vp.scale,
        );
        let (x0, y0) = (x0.max(0.0) as usize, y0.max(0.0) as usize);
        let x1 = ((x0 as f64 + w) as usize).min(processeur.width() as usize);
        let y1 = ((y0 as f64 + h) as usize).min(processeur.height() as usize);
        for y in y0..y1 {
            let ligne = y * largeur;
            ecarts += (ligne + x0 * 4..ligne + x1 * 4)
                .filter(|&i| processeur.data()[i].abs_diff(carte.data()[i]) > ECART_COURANT)
                .count();
        }
    }
    assert_eq!(
        ecarts, 0,
        "les cases diffèrent entre les deux voies sur {ecarts} canaux"
    );
}

/// **LOT-1 par équivalence** : en balayant l'échelle au travers de plusieurs niveaux, chaque image
/// qui reprend le lot demande exactement ce qu'une image complète aurait demandé. Une seule photo
/// gardée à un mauvais niveau, une seule qui devrait entrer ou sortir, et l'épreuve tombe.
#[test]
fn test_lot_1_une_image_reprise_est_celle_qu_une_image_complete_aurait_demandee() {
    use glucose_desktop::renderer::Renderer;
    let mut store = glucose_core::store::Store::new("Lot");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
    }
    for (i, (l, h)) in [(200.0, 200.0), (130.0, 170.0), (90.0, 60.0)]
        .iter()
        .enumerate()
    {
        let mut img = glucose_core::types::BoardImage::new(
            format!("damier-{i}"),
            120.0 + 220.0 * i as f64,
            300.0,
            *l,
            *h,
        );
        img.src = Some(photo_damier().to_string_lossy().into_owned());
        store.add_image(&board, img);
    }
    store.clear_selection();
    let taille = (800, 600);
    let guides = glucose_core::smart_align::SnapGuides::default();
    let image = |renderer: &mut Renderer, ui: &mut UiState, store: &glucose_core::store::Store| {
        let mut dessous = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        let mut dessus = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        renderer.rendre_les_couches(
            &mut dessous.as_mut(),
            &mut dessus.as_mut(),
            store,
            (ui, Pointer { x: 0.0, y: 0.0 }),
            SceneOverlay::sans_rien(&guides),
            Regard::immobile(),
        )
    };
    let (mut lot, mut complet) = (Renderer::new(), Renderer::new());
    let (mut ui_lot, mut ui_complet) = (UiState::new(), UiState::new());
    for r in [&mut lot, &mut complet] {
        r.sync_spatial_index(&store);
    }
    // Tout décodé, des deux côtés, avant de comparer.
    let _ = image(&mut lot, &mut ui_lot, &store);
    let _ = image(&mut complet, &mut ui_complet, &store);
    lot.magasin.attendre_le_chantier();
    complet.magasin.attendre_le_chantier();

    let (mut reprises, mut completes) = (0, 0);
    let mut poses: Vec<glucose_desktop::renderer::voies::APoser> = Vec::new();
    for pas in 0..240 {
        // De 0,02 à 2 : plus de six octaves, à un pas plus fin que l'écart entre deux niveaux.
        let echelle = 0.02 * 1.02_f64.powi(pas);
        if let Some(b) = store.active_board_mut() {
            b.viewport = glucose_core::types::Viewport {
                x: 400.0 - 400.0 * echelle,
                y: 300.0 - 300.0 * echelle,
                scale: echelle,
            };
        }
        let c = image(&mut lot, &mut ui_lot, &store);
        match c.lot {
            Some(en_place) => {
                reprises += 1;
                assert!(
                    c.photos.is_empty(),
                    "reprendre le lot, c'est n'en rien redire"
                );
                assert_eq!(en_place.nombre, poses.len());
            }
            None => {
                completes += 1;
                poses = c.photos.clone();
                lot.carte.lot_de_la_carte = c.lot_propose;
            }
        }
        lot.magasin.attendre_le_chantier();
        let temoin = image(&mut complet, &mut ui_complet, &store);
        complet.magasin.attendre_le_chantier();
        let cles = |p: &[glucose_desktop::renderer::voies::APoser]| {
            p.iter()
                .map(|t| {
                    t.cle.nom().map_or_else(
                        || {
                            format!("{:?}", t.cle)
                                .split_once(',')
                                .map_or(String::new(), |(_, f)| f.to_string())
                        },
                        str::to_string,
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(cles(&poses), cles(&temoin.photos), "à l'échelle {echelle}");
    }
    assert!(
        reprises > 100,
        "seulement {reprises} images reprises : l'épreuve doit les exercer"
    );
    assert!(
        completes > 3,
        "seulement {completes} images complètes : aucun niveau franchi ?"
    );
}

/// **Une seule requête spatiale par image** (PHOTO-1) : le dessous, ce qui est confié à la carte et
/// le dessus regardent la même vue du même document, et la refaisaient chacun : huit mille rangs
/// triés trois fois, à chaque image. **Et cette mémoire ne survit pas à l'image** : un nœud posé
/// ensuite, à la même vue, doit se voir, même par une question posée hors d'une image.
#[test]
fn test_photo_1_une_seule_requete_spatiale_par_image() {
    let mut store = document_damier(1.0);
    let (taille, guides) = ((800, 600), glucose_core::smart_align::SnapGuides::default());
    let mut renderer = Renderer::new();
    renderer.sync_spatial_index(&store);
    let mut ui = UiState::new();
    let header = ui.header_height();
    let mut image = |renderer: &mut Renderer, store: &glucose_core::store::Store| {
        let mut dessous = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        let mut dessus = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        renderer.rendre_les_couches(
            &mut dessous.as_mut(),
            &mut dessus.as_mut(),
            store,
            (&mut ui, Pointer { x: 0.0, y: 0.0 }),
            SceneOverlay::sans_rien(&guides),
            Regard::immobile(),
        )
    };
    let avant = renderer.requetes_spatiales;
    let _ = image(&mut renderer, &store);
    assert_eq!(
        renderer.requetes_spatiales - avant,
        1,
        "une seule, pour toute l'image"
    );
    renderer.magasin.attendre_le_chantier();
    let confie = image(&mut renderer, &store);
    assert_eq!(confie.photos.len(), 1, "la photo est à l'écran");
    assert_eq!(renderer.noeuds_visibles, 1);

    // Une question hors d'une image, puis la même après un changement : si la première restait en
    // mémoire, la seconde lirait le document d'avant.
    renderer.rejuger_la_carte(&store, taille, header);
    let board = store.project.active_board_id.clone();
    store.add_image(
        &board,
        glucose_core::types::BoardImage::new("voisine", 420.0, 320.0, 50.0, 50.0),
    );
    renderer.rejuger_la_carte(&store, taille, header);
    assert_eq!(
        renderer.noeuds_visibles, 2,
        "la vue est la même, le document non"
    );
}

/// **Une image qui reprend le lot ne touche pas au magasin** (LOT-1) : elle ne réclame aucune photo, et
/// si l'étagement passait quand même, il lirait des voulus vides, et offrirait au système tout ce que
/// l'écran montre. Rien ne doit partir, ni bouger, image après image.
#[test]
fn test_lot_1_une_image_reprise_ne_touche_pas_au_magasin() {
    let store = document_damier(0.3);
    let (taille, guides) = ((800, 600), glucose_core::smart_align::SnapGuides::default());
    let mut renderer = Renderer::new();
    renderer.sync_spatial_index(&store);
    let mut ui = UiState::new();
    let mut image = |renderer: &mut Renderer| {
        let mut dessous = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        let mut dessus = Pixmap::new(taille.0, taille.1).expect("un pixmap");
        let confie = renderer.rendre_les_couches(
            &mut dessous.as_mut(),
            &mut dessus.as_mut(),
            &store,
            (&mut ui, Pointer { x: 0.0, y: 0.0 }),
            SceneOverlay::sans_rien(&guides),
            Regard::immobile(),
        );
        renderer.magasin.attendre_le_chantier();
        confie
    };
    // Jusqu'à ce que tout soit décodé et réglé : plus rien ne bouge d'une image complète à l'autre.
    let mut propose = None;
    let mut vus = u64::MAX;
    for _ in 0..50 {
        propose = image(&mut renderer).lot_propose;
        if renderer.magasin.changements() == vus {
            break;
        }
        vus = renderer.magasin.changements();
    }
    assert!(propose.is_some(), "une image complète propose son lot");
    renderer.carte.lot_de_la_carte = propose;
    let offerts = renderer.magasin.mouvements().offerts;
    let numero = renderer.magasin.image();
    for _ in 0..4 {
        assert!(
            image(&mut renderer).lot.is_some(),
            "rien ne change : il se reprend"
        );
        assert_eq!(
            renderer.magasin.image(),
            numero,
            "elle reprend le numéro de l'image qui a réclamé ses photos"
        );
    }
    assert_eq!(
        renderer.magasin.mouvements().offerts,
        offerts,
        "rien n'est parti"
    );
    assert_eq!(renderer.magasin.changements(), vus, "et rien n'a bougé");
}

// ── TITRE-3 : le titre d'une membrane est un composant (fiche 108) ────────────────────────

/// Un document d'une membrane titrée, à cette vue — et, si on la donne, une photo blanche
/// (centre et taille dans le monde) posée sur son titre.
fn document_titre(
    titre: &str,
    (largeur, hauteur): (f64, f64),
    vue: glucose_core::types::Viewport,
    photo: Option<(f64, f64, f64, f64)>,
) -> glucose_core::store::Store {
    let mut store = glucose_core::store::Store::new("Titres");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = vue;
    }
    let mut m = glucose_core::types::Annotation::membrane("m", 0.0, 0.0, largeur, hauteur);
    if let glucose_core::types::Annotation::Membrane { text, color, .. } = &mut m {
        *text = Some(titre.to_string());
        *color = Some("#60a5fa".to_string());
    }
    store.add_annotation(&board, m);
    if let Some((x, y, l, h)) = photo {
        let mut img = glucose_core::types::BoardImage::new("blanche", x, y, l, h);
        let blanche = photo_d_epreuve("blanche", (64, 64), |_, _| 255);
        img.src = Some(blanche.to_string_lossy().into_owned());
        store.add_image(&board, img);
    }
    store.clear_selection();
    store
}

/// Un pixel du bleu vif du titre : ni le canevas, ni le voile de la membrane, ni le blanc d'une
/// photo ne l'atteignent.
fn bleu_vif(p: &Pixmap, x: u32, y: u32) -> bool {
    let [r, g, b, _] = pixel(p, x, y);
    b > 170 && r < 150 && g > 110
}

/// **TITRE-3 — sur la voie graphique, la carte pose le titre, et la couche du dessous reste
/// vide** : rien ne s'y dessine, rien ne part sur le bus. Le titre qu'on écrit, lui, y reste —
/// sa sélection et son curseur bougent à chaque frappe —, et la carte ne le pose pas.
#[test]
fn test_titre_3_la_carte_pose_le_titre_et_la_couche_du_dessous_reste_vide() {
    let taille = (800u32, 600u32);
    let vue = glucose_core::types::Viewport {
        x: 40.0,
        y: 300.0,
        scale: 1.0,
    };
    let store = document_titre("Hydrologie", (600.0, 250.0), vue, None);
    let (_, dessous, _, confie) = les_deux_couches(taille, &store);
    assert_eq!(confie.titres.len(), 1, "la carte pose le titre");
    assert_eq!(encre(&dessous), 0, "la couche du dessous ne le peint pas");
    assert!(
        !confie.dessous_porte_quelque_chose,
        "une couche du dessous vide ne part pas sur le bus"
    );

    let saisie = glucose_desktop::renderer::TextEditSession {
        ann_id: "m".into(),
        buffer: "Hydrologie".into(),
        selection: glucose_core::text::Selection::at(4),
        goal_x: None,
        blink_timer: std::time::Instant::now(),
        curseur_visible: true,
    };
    let (_, dessous, _, confie) = les_deux_couches_en_editant(taille, &store, Some(&saisie));
    assert!(
        confie.titres.is_empty(),
        "le titre qu'on écrit n'est pas une texture"
    );
    assert!(encre(&dessous) > 0, "il s'écrit dans la couche du dessous");
}

/// **TITRE-3 — une photo passe sur un titre, sur les deux voies** : les titres se posent sous les
/// photos, juste après les frontières, comme le processeur les écrit. Le contrepoint dans la même
/// épreuve : sans la photo, son rectangle porte bien le bleu du titre.
#[test]
fn test_titre_3_une_photo_passe_sur_un_titre_par_les_deux_voies() {
    let taille = (800u32, 600u32);
    let vue = glucose_core::types::Viewport {
        x: 40.0,
        y: 300.0,
        scale: 1.0,
    };
    // Le titre de la loi pour cette membrane : un corps de 23 pixels, sa plume 26 pixels au-dessus
    // du bord haut — ses lettres entre les lignes 274 et 300 de l'écran, à partir de x = 63. La
    // photo couvre x de 60 à 200 et y de 262 à 302.
    let membrane = (600.0, 250.0);
    let sur_le_titre = Some((90.0, -18.0, 140.0, 40.0));
    let (gauche, haut, droite, bas) = (63u32, 265u32, 197u32, 299u32);
    let dans_la_photo = || (gauche..droite).flat_map(|x| (haut..bas).map(move |y| (x, y)));

    let sans = document_titre("Hydrologie", membrane, vue, None);
    let temoin = par_le_processeur(taille, &sans);
    assert!(
        dans_la_photo().any(|(x, y)| bleu_vif(&temoin, x, y)),
        "sans la photo, son rectangle doit porter le titre : sinon l'épreuve ne regarde rien"
    );

    let store = document_titre("Hydrologie", membrane, vue, sur_le_titre);
    let (renderer, dessous, dessus, confie) = les_deux_couches_decodees(taille, &store);
    let Some((peripherique, file)) = banc_gpu::carte() else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let carte = banc_gpu::composer_les_cinq_temps(
        (&peripherique, &file),
        taille,
        &confie,
        (&dessous, &dessus),
        &|cle| confie.pixels(&renderer, cle),
    )
    .expect("la composition en cinq temps");
    let processeur = par_le_processeur_decode(taille, &store);
    for (nom, image) in [("processeur", &processeur), ("carte", &carte)] {
        let bleus = dans_la_photo()
            .filter(|&(x, y)| bleu_vif(image, x, y))
            .count();
        assert_eq!(
            bleus, 0,
            "{nom} : le titre passe sur la photo ({bleus} pixels)"
        );
    }
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "les deux voies divergent de {pire} niveaux sur un titre sous une photo"
    );
    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    let canaux = processeur.data().len();
    assert!(
        larges * 1000 <= canaux * PART_MAX_POUR_MILLE,
        "l'ecart depasse {ECART_COURANT} sur {larges} canaux sur {canaux}"
    );
}

/// Ce que la voie graphique confie pendant un zoom (`en_mouvement`), à cette vue.
fn confie_en_zoom(
    taille: (u32, u32),
    store: &glucose_core::store::Store,
    vue: glucose_core::types::Viewport,
) -> Confie {
    let mut store = store.clone();
    let board = store.project.active_board_id.clone();
    store.set_viewport(&board, vue);
    let mut dessous = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    let mut dessus = Pixmap::new(taille.0, taille.1).expect("un pixmap");
    let mut renderer = Renderer::new();
    let mut ui = UiState::new();
    let guides = glucose_core::smart_align::SnapGuides::default();
    renderer.rendre_les_couches(
        &mut dessous.as_mut(),
        &mut dessus.as_mut(),
        &store,
        (&mut ui, Pointer { x: 0.0, y: 0.0 }),
        SceneOverlay::sans_rien(&guides),
        Regard {
            degradation_permise: true,
            en_mouvement: true,
            repos: None,
        },
    )
}

/// **TITRE-3 — de loin, pendant un zoom, la texture d'un titre réduit garde sa clé** : le titre
/// est ramené à la largeur de sa membrane **dans le monde**. Mesuré à l'écran, où ses lettres
/// font moins d'un pixel et se mesurent comme d'un pixel, l'ajustement changeait avec le zoom —
/// et la texture se refaisait à chaque image.
///
/// Une membrane neuve (un corps de 16 dans le monde), à ×0,03 et ×0,033 : le même palier (2⁻⁵),
/// des lettres d'un demi-pixel. La première version de cette épreuve regardait ×0,004 et ×0,0042
/// — sous le plus petit zoom (`Viewport::MIN_SCALE`), donc deux fois la même vue : le sabotage de
/// l'ajustement passait. Elle vérifie désormais qu'elle regarde deux vues.
#[test]
fn test_titre_3_de_loin_un_titre_reduit_garde_sa_cle_pendant_un_zoom() {
    let taille = (800u32, 600u32);
    let titre = "Une très longue histoire de l'hydrologie continentale et de ses bassins versants";
    let vue = |scale| glucose_core::types::Viewport {
        x: 300.0,
        y: 300.0,
        scale,
    };
    let (zooms, naissance) = ([0.03, 0.033], glucose_core::membrane_titre::NAISSANCE);
    assert!(zooms
        .iter()
        .all(|&z| z >= glucose_core::types::Viewport::MIN_SCALE && 16.0 * z < 1.0));
    let store = document_titre(titre, naissance, vue(zooms[0]), None);
    let titres: Vec<_> = zooms
        .into_iter()
        .map(|scale| {
            let confie = confie_en_zoom(taille, &store, vue(scale));
            assert_eq!(confie.titres.len(), 1, "à ×{scale}, la carte pose le titre");
            confie.titres[0].clone()
        })
        .collect();
    assert_ne!(
        titres[0].pose.largeur, titres[1].pose.largeur,
        "deux vues différentes : sinon l'épreuve compare une vue à elle-même"
    );
    assert_eq!(
        titres[0].cle, titres[1].cle,
        "le même palier, la même texture"
    );
}

// ── La date des nœuds (DATE-1, fiche 113) ─────────────────────────────────────────────────

/// Un texte, une flèche, une membrane et une photo en chemin, **rien de choisi** — le cas où la
/// couche du dessus de la carte graphique n'a, sans les dates, aucun ornement à porter
/// (ARCHITECTURE § 4) ; datés chacun, ou aucun.
fn document_date(datee: bool) -> glucose_core::store::Store {
    use glucose_core::types::{Annotation, BoardImage, TemporalAnchor};
    let mut store = glucose_core::store::Store::new("Dates");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
        b.viewport = glucose_core::types::Viewport {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        };
    }
    store.add_annotation(&board, Annotation::text("t", 100.0, 100.0, "La Révolution"));
    store.add_annotation(&board, Annotation::arrow("f", 420.0, 120.0, 680.0, 300.0));
    store.add_annotation(&board, Annotation::membrane("m", 80.0, 340.0, 300.0, 200.0));
    store.add_image(&board, BoardImage::new("i", 500.0, 380.0, 160.0, 120.0));
    if datee {
        for (id, start, end) in [
            ("t", 1789, 1799),
            ("f", 1400, 1600),
            ("m", -500, -500),
            ("i", 1969, 1969),
        ] {
            let ancre = TemporalAnchor {
                start,
                end,
                label: None,
            };
            store.ancrer_dans_le_temps(&board, &[id.to_string()], Some(&ancre));
        }
    }
    store.clear_selection();
    store
}

/// Les pixels **chauds** (le rouge passe le bleu) autour du coin en bas à droite de ce nœud, là
/// où la pastille déborde : son ambre, l'encre `#fde68a` comme le filet `#fbbf24`. Rien d'autre
/// n'est chaud dans la scène — les membranes sans couleur sont neutres, la photo en chemin
/// grise, un texte sans parent garde le contour gris de son bloc (`renderer::date::tests`).
fn chaud_au_coin(image: &Pixmap, store: &glucose_core::store::Store, id: &str) -> usize {
    let board = store.active_board().expect("le tableau");
    let b = glucose_core::arrow::emprise(board, id).expect("sa boîte");
    // La vue est à l'origine, à l'échelle 1 : le monde est l'écran.
    let (cx, cy) = ((b.left + b.width) as i64, (b.top + b.height) as i64);
    let mut n = 0;
    for y in (cy - 14).max(0)..(cy + 14).min(image.height() as i64) {
        for x in (cx - 90).max(0)..(cx + 14).min(image.width() as i64) {
            let [r, _, b, _] = pixel(image, x as u32, y as u32);
            if r > b {
                n += 1;
            }
        }
    }
    n
}

/// **DATE-1 — la date paraît sur les deux voies, rien de choisi** (fiche 113).
///
/// La pastille se pose dans la moitié que les deux voies partagent (`dessiner_sur_les_photos`) ;
/// sur la voie graphique, c'est la couche du dessus, qui ne part à l'écran que par les bandes
/// qu'elle a réellement écrites (BANDE-1) et que la chrome a quittée (CHROME-1) — sans rien de
/// choisi, elle était vide. Trois choses à prouver : la carte graphique montre l'ambre au coin de
/// **chaque** sorte de nœud daté ; rien sans date (la preuve à l'envers : l'ambre vient bien de la
/// date) ; et les deux voies rendent la même image.
#[test]
fn test_date_1_la_date_parait_sur_les_deux_voies_sans_rien_de_choisi() {
    let taille = (800u32, 600u32);
    let datee = document_date(true);
    let Some(carte) = par_la_carte(taille, &datee) else {
        eprintln!("aucune carte utilisable : epreuve sautee");
        return;
    };
    let nue = document_date(false);
    let carte_nue = par_la_carte(taille, &nue).expect("la même carte");
    for id in ["t", "f", "m", "i"] {
        let vu = chaud_au_coin(&carte, &datee, id);
        assert!(
            vu > 30,
            "{id} daté : {vu} pixels d'ambre sur la carte graphique"
        );
        let sans = chaud_au_coin(&carte_nue, &nue, id);
        assert_eq!(
            sans, 0,
            "{id} sans date : de l'ambre sur la carte graphique"
        );
    }
    let processeur = par_le_processeur(taille, &datee);
    let pire = banc_gpu::pire_ecart(&processeur, &carte);
    assert!(
        pire <= ECART_ADMIS,
        "les deux voies divergent de {pire} niveaux sur les dates (admis {ECART_ADMIS})"
    );
    let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
    let canaux = processeur.data().len();
    assert!(
        larges * 1000 <= canaux * PART_MAX_POUR_MILLE,
        "l'ecart depasse {ECART_COURANT} sur {larges} canaux sur {canaux} : une date manque d'un côté"
    );
}
