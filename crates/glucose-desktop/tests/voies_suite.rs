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
use glucose_desktop::present::{banc_gpu, couches, fond_gpu, lueurs_gpu, scene_gpu};
use glucose_desktop::renderer::{Confie, Regard, Renderer};
use glucose_desktop::ui::UiState;
use tiny_skia::Pixmap;

/// L'écart admis entre les deux voies, en niveaux de couleur sur 255.
///
/// **Mesuré, puis élargi d'une marge nommée.** Sur cette scène et sur cette machine, le pire
/// écart vaut **3**, et *aucun* canal ne dépasse 3 — la borne théorique de la grille, sept,
/// n'est pas atteinte ici parce que la vue du témoin ne pose pas ses points sur les phases
/// les plus défavorables.
///
/// Cinq, donc : trois de mesure, deux pour qu'une autre carte graphique — qui n'arrondit pas
/// forcément au même bit — ne fasse pas échouer une épreuve où rien n'est faux. La charte
/// interdit d'exclure une machine, et cela vaut aussi de ses tests.
const ECART_ADMIS: u8 = 5;

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

/// Ce que le processeur produit pour la **voie graphique** : deux couches, et ce qu'il confie.
fn les_deux_couches(
    taille: (u32, u32),
    store: &glucose_core::store::Store,
) -> (Pixmap, Pixmap, Confie) {
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
            editing: None,
        },
        Regard::immobile(),
    );
    (dessous, dessus, confie)
}

/// La scène témoin composée par la **voie graphique**, en cinq temps, hors fenêtre.
fn par_la_carte(taille: (u32, u32), store: &glucose_core::store::Store) -> Option<Pixmap> {
    let (dessous, dessus, confie) = les_deux_couches(taille, store);
    let (peripherique, file) = banc_gpu::carte()?;
    let format = banc_gpu::FORMAT;
    let ecran = (taille.0 as f32, taille.1 as f32);

    let mut fond = fond_gpu::FondGpu::nouveau(&peripherique, format);
    let mut lueurs = lueurs_gpu::Lueurs::nouvelles(&peripherique, format);
    let mut scene = scene_gpu::SceneGpu::nouvelle(&peripherique, format);
    let mut deux = couches::Couches::nouvelles(&peripherique, format);

    fond.preparer(&file, ecran, confie.fond);
    lueurs.preparer(&peripherique, &file, ecran, &confie.lueurs);
    scene.ouvrir();
    let retenues = scene.preparer(&peripherique, &file, ecran, &confie.photos);
    let utile = confie.fond.is_none() || confie.dessous_porte_quelque_chose;
    deux.televerser(&peripherique, &file, (&dessous, utile), &dessus);

    let cible = banc_gpu::cible(&peripherique, taille);
    let vue = cible.create_view(&Default::default());
    let mut encodeur = peripherique.create_command_encoder(&Default::default());
    couches::composer(
        &mut encodeur,
        &vue,
        couches::Temps {
            fond: &fond,
            lueurs: &lueurs,
            couches: &deux,
            scene: &scene,
            retenues: &retenues,
        },
    );
    file.submit(Some(encodeur.finish()));
    banc_gpu::relire(&peripherique, &file, &cible, taille)
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

/// **Une photo en chemin se voit sur les deux voies.**
///
/// La scène témoin porte deux images sans octets. Sur la voie processeur elles se dessinent
/// comme un cadre gris portant leur identifiant ; sur la voie graphique la carte ne les
/// connaît pas, donc elle ne dessine rien — et pendant trois commits, **plus personne** ne les
/// dessinait.
///
/// Le test ne compare pas les deux voies, il compte : la couche du dessus doit porter de
/// l'encre là où une photo manque. Un test qui comparerait les images passerait aussi le jour
/// où les deux voies cesseraient de les dessiner.
#[test]
fn test_une_photo_en_chemin_se_dessine_sur_la_voie_graphique() {
    let taille = synth::WITNESS_SIZE;
    let store = synth::witness_selected();
    let (_, dessus, _) = les_deux_couches(taille, &store);

    // Le cadre d'une photo en chemin est peint avec `bg_hover`, un gris de la chrome, et il
    // est OPAQUE : c'est ce qui le distingue du texte et des poignees, qui sont clairs.
    let theme = glucose_desktop::theme::Theme::dark();
    let attendu = {
        let c = theme.bg_hover;
        [
            (c.red() * 255.0).round() as u8,
            (c.green() * 255.0).round() as u8,
            (c.blue() * 255.0).round() as u8,
            255,
        ]
    };
    let cadres = dessus
        .data()
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p == attendu)
        .count();
    assert!(
        cadres > 5_000,
        "la couche du dessus ne porte que {cadres} pixels de cadre : les photos en chemin ont \
         cesse d'etre dessinees"
    );
}
