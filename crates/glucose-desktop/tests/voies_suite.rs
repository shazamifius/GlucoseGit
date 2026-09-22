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
        SceneOverlay {
            guides: &guides,
            selection_box: None,
            editing: None,
        },
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
            guides: &guides,
            selection_box: None,
            editing,
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
        &|cle| confie.composant(cle).and_then(|c| c.rendre(renderer.kit())),
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
