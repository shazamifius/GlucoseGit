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
        .filter(|(cle, _)| cle.starts_with("chemin:"))
        .collect();
    assert_eq!(
        en_chemin.len(),
        3,
        "le temoin porte trois photos sans octets, et chacune est une texture a son rang"
    );
    for (cle, _) in en_chemin {
        let composant = confie
            .composant(cle)
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
            "la texture de {cle} ne porte que {encre} pixels : le cadre en chemin ne se dessine              plus"
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
            .any(|(cle, _)| cle.starts_with(&prefixe)),
        "la carte en saisie doit etre une texture que la carte pose, et elle ne l'est pas"
    );
    assert_eq!(
        confie
            .cartes
            .last()
            .map(|(cle, _)| cle.starts_with(&prefixe)),
        Some(true),
        "elle se pose en DERNIER, donc au-dessus des autres cartes"
    );
    // **Ce test echoue sur l'implementation d'avant**, et c'est la seule preuve qui compte :
    // en y retablissant `au_processeur = editing.is_some()`, la couche du dessus passe de
    // 205 918 a 260 930 pixels d'encre -- cinquante-cinq mille de plus, soit exactement une
    // carte repeinte par-dessus celle que la carte graphique pose deja.
    let (avant, apres) = (encre(&sans), encre(&avec));
    assert!(
        apres <= avant + avant / 20,
        "la couche du dessus passe de {avant} a {apres} pixels d'encre : le processeur y \
         repeint la carte que la carte graphique porte deja"
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

    let cles = |c: &Confie| {
        c.cartes
            .iter()
            .map(|(cle, _)| cle.clone())
            .collect::<Vec<_>>()
    };
    assert!(
        cles(&avant)
            .iter()
            .any(|c| c.starts_with(&format!("carte:{id}:"))),
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
    let cles = |c: &Confie| {
        c.cartes
            .iter()
            .map(|(cle, _)| cle.clone())
            .collect::<Vec<_>>()
    };
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
    les_deux_couches_sous_un_budget(taille, store, None)
}

/// Les deux couches, quand la carte ne laisse aux photos que `part` octets (ETAGES-3).
fn les_deux_couches_sous_un_budget(
    taille: (u32, u32),
    store: &glucose_core::store::Store,
    part: Option<u64>,
) -> (Renderer, Pixmap, Pixmap, Confie) {
    let mut renderer = Renderer::new();
    renderer.carte.part_des_photos = part;
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
        let _ = carte.save_png(format!(r"C:/Users/ADMINI~1/AppData/Local/Temp/claude/c--Users-Administrator-Documents-GlucoseGit-main/3b3cd992-dff0-4aaf-9a68-e8d42ba227b1/scratchpad/depres-carte-{echelle}.png"));
        let larges = banc_gpu::canaux_hors_tolerance(&processeur, &carte, ECART_COURANT);
        let canaux = processeur.data().len();
        assert!(
            larges * 1000 <= canaux * PART_MAX_POUR_MILLE,
            "a l'echelle {echelle}, l'ecart depasse {ECART_COURANT} sur {larges} canaux sur \
             {canaux} : la carte manque sur une des deux voies"
        );
    }
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
    let (cle, _) = confie.photos.first().expect("la photo est posee");
    let texture = confie.pixels(&renderer, cle).expect("ses pixels");
    let texture = texture.vue();
    assert_eq!(
        (texture.width(), texture.height()),
        (64, 64),
        "soixante pixels a l'ecran : le niveau de soixante-quatre, pas les 512 natifs"
    );
    let src = photo_damier().to_string_lossy().into_owned();
    let identite = confie
        .textures()
        .into_iter()
        .find(|t| &t.cle == cle)
        .map(|t| t.identite);
    assert_eq!(
        identite,
        Some(src),
        "l'identite reste le fichier : un changement de niveau garde l'ancien pose"
    );
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
        let (renderer, _, _, confie) = les_deux_couches_sous_un_budget((800, 600), &store, part);
        let (cle, _) = confie.photos.first().expect("la photo est posee").clone();
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
    let (renderer, _, _, _) = les_deux_couches_sous_un_budget((800, 600), &store, Some(0));
    assert!(renderer.carte.debordee, "rien ne tient : la carte le dit");
}
