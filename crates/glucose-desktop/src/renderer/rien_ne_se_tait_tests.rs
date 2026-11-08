//! **GRIS-2 — de loin, rien ne se tait** (fiche 96). Sous le seuil de détail, neuf dessins se
//! taisaient (SCALE-2) : le nom des zones et des dossiers, les textes des cases, l'étiquette d'une
//! flèche, le mot d'un opérateur, la pastille d'un prédicat, le sigle d'un domaine, la lueur d'une
//! membrane et celle d'un dossier choisi. Sa parole du 8/11 : *« pas de LOD, pas de
//! simplification »*. Chaque épreuve regarde de loin (un dixième) ce que la vue montre, et exige
//! qu'il y soit.

use super::Renderer;
use crate::ui::UiState;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, ArrowPredicate, BoardZone, StickyOperator, Viewport};
use tiny_skia::Pixmap;

/// Le zoom où l'on regarde : un dixième, sous le seuil de détail.
const LOIN: f64 = 0.1;

/// Le tableau de ces annotations seules, vu de loin depuis l'origine.
fn de_loin(annotations: Vec<Annotation>) -> Pixmap {
    let mut store = Store::new("loin");
    let board = store.project.active_board_id.clone();
    if let Some(b) = store.active_board_mut() {
        b.annotations.clear();
    }
    for a in annotations {
        store.add_annotation(&board, a);
    }
    store.clear_selection();
    rendre(&store)
}

fn rendre(store: &Store) -> Pixmap {
    let mut pixmap = Pixmap::new(600, 400).expect("un tampon");
    let vue = Viewport {
        x: 0.0,
        y: 0.0,
        scale: LOIN,
    };
    let mut renderer = Renderer::new();
    renderer.rendre_pour_l_export(&mut pixmap.as_mut(), store, &mut UiState::new(), vue);
    pixmap
}

/// Une flèche horizontale de 3 000 unités, de (200, 1 000) à (3 200, 1 000).
fn fleche() -> Annotation {
    Annotation::arrow("f", 200.0, 1_000.0, 3_200.0, 1_000.0)
}

/// **L'étiquette d'une flèche se voit de loin** : une flèche étiquetée et la même sans étiquette ne
/// se dessinent pas pareil.
#[test]
fn gris_2_de_loin_l_etiquette_d_une_fleche_reste() {
    let mut etiquetee = fleche();
    if let Annotation::Arrow { text, .. } = &mut etiquetee {
        *text = Some("contredit l'idée reçue".into());
    }
    assert_ne!(
        de_loin(vec![etiquetee]).data(),
        de_loin(vec![fleche()]).data(),
        "de loin, l'étiquette montre le gris de son encre"
    );
}

/// **La pastille d'un prédicat se voit de loin.**
#[test]
fn gris_2_de_loin_la_pastille_d_un_predicat_reste() {
    let mut qualifiee = fleche();
    if let Annotation::Arrow { predicate, .. } = &mut qualifiee {
        *predicate = Some(ArrowPredicate::Contredit);
    }
    assert_ne!(
        de_loin(vec![qualifiee]).data(),
        de_loin(vec![fleche()]).data(),
        "de loin, la pastille d'un prédicat reste"
    );
}

/// La couleur moyenne des pixels `[x0, x1) × [y0, y1)`.
fn moyenne(p: &Pixmap, (x0, x1): (u32, u32), (y0, y1): (u32, u32)) -> [f64; 3] {
    let mut s = [0.0f64; 3];
    for y in y0..y1 {
        for x in x0..x1 {
            let c = p.pixel(x, y).expect("dans l'image").demultiply();
            for (v, c) in s.iter_mut().zip([c.red(), c.green(), c.blue()]) {
                *v += f64::from(c);
            }
        }
    }
    let n = f64::from((x1 - x0) * (y1 - y0));
    s.map(|v| v / n)
}

/// **Le mot d'un opérateur se voit de loin** : au milieu de la pilule, là où son mot s'écrit,
/// l'encre du mot s'ajoute à la teinte de la pilule ; près de son bout, il n'y a que la pilule.
#[test]
fn gris_2_de_loin_le_mot_d_un_operateur_reste() {
    let mut pilule = Annotation::sticky("op", 0.0, 0.0, "");
    if let Annotation::Sticky {
        operator,
        width,
        height,
        ..
    } = &mut pilule
    {
        *operator = Some(StickyOperator::Because);
        (*width, *height) = (Some(4_000.0), Some(1_000.0));
    }
    let p = de_loin(vec![pilule]);
    // La pilule : 400 × 100 pixels, son mot d'un corps de 1,4 pixel en son centre ; à la même
    // hauteur, près de son bout gauche, rien que la pilule.
    let milieu = moyenne(&p, (190, 210), (49, 51));
    let bout = moyenne(&p, (60, 80), (49, 51));
    let ecart: f64 = (0..3)
        .map(|k| (milieu[k] - bout[k]).abs())
        .fold(0.0, f64::max);
    assert!(
        ecart > 1.0,
        "de loin, le mot montre le gris de son encre : milieu {milieu:?}, bout {bout:?}"
    );
}

/// **La lueur d'une membrane se voit de loin** : juste hors de sa boîte, la vue n'est pas le fond.
#[test]
fn gris_2_de_loin_la_lueur_d_une_membrane_reste() {
    let sans_titre = |mut m: Annotation| {
        if let Annotation::Membrane { text, .. } = &mut m {
            *text = None;
        }
        m
    };
    let membrane = sans_titre(Annotation::membrane(
        "m", 1_000.0, 1_000.0, 2_000.0, 2_000.0,
    ));
    let p = de_loin(vec![membrane]);
    let fond = moyenne(&p, (500, 590), (300, 390));
    // La boîte : de 100 à 300 en x et en y. La lueur déborde de deux pixels (20 unités) ; le
    // contour, d'un dixième de pixel sur la rangée 99 : on lit la rangée 98, que la lueur seule
    // atteint.
    let lueur = moyenne(&p, (150, 250), (98, 99));
    assert_ne!(
        lueur.map(|v| v.round()),
        fond.map(|v| v.round()),
        "de loin, la lueur de la membrane reste"
    );
}

/// Ce qui écrit les textes d'une sorte d'élément sur une surface : `zone::textes`,
/// `storyboard::textes`.
type Ecrire =
    fn(&mut super::surface::Pixels, (&Store, crate::params::ViewPass<'_>), (f32, f32)) -> bool;

/// Les textes que `ecrire` pose, vus de loin, sur un tampon à eux ; rend vrai si un élément en a
/// reçu, et si le tampon a changé.
fn textes_de_loin(store: &Store, ecrire: Ecrire) -> (bool, bool) {
    let mut p = Pixmap::new(600, 400).expect("un tampon");
    let vierge = p.clone();
    let typographie = crate::typography::Typography::new();
    let board = store.active_board().expect("un tableau actif");
    let rangs = glucose_core::quadtree::tous_les_rangs(board);
    let mut index = glucose_core::quadtree::SpatialHash::new(1000.0);
    index.index_board(board);
    let pass = crate::params::ViewPass {
        vp: Viewport {
            x: 0.0,
            y: 0.0,
            scale: LOIN,
        },
        visibles: &rangs,
        index: &index,
        contournement: None,
        header_h: 0.0,
        densite: 1.0,
    };
    let mut tampon = p.as_mut();
    let mut surface = super::surface::Pixels {
        pixmap: &mut tampon,
        typographie: &typographie,
    };
    let recu = ecrire(&mut surface, (store, pass), (600.0, 400.0));
    (recu, p.data() != vierge.data())
}

/// **Le nom d'une zone s'écrit de loin**, en gris.
#[test]
fn gris_2_de_loin_le_nom_d_une_zone_reste() {
    let mut store = Store::new("zones");
    let board = store.project.active_board_id.clone();
    let zone = BoardZone::new("charadesign/detail", 100.0, 100.0, 3_400.0, 2_000.0);
    store.poser_les_zones(&board, vec![zone]);
    let (recu, peint) =
        textes_de_loin(&store, |p, quoi, ecran| super::zone::textes(p, quoi, ecran));
    assert!(
        recu && peint,
        "de loin, la zone écrit son nom ({recu}, {peint})"
    );
}

/// **Les textes des cases d'un storyboard s'écrivent de loin**, en gris.
#[test]
fn gris_2_de_loin_les_cases_d_un_storyboard_ecrivent() {
    use glucose_core::store::storyboard::{Format, Grille};
    let mut store = Store::new("cases");
    let board = store.project.active_board_id.clone();
    let grille = Grille {
        format: Format::Cinema,
        largeur: 3_000.0,
        colonnes: 2,
        ecart: 240.0,
    };
    store.poser_le_storyboard(&board, grille, (0.0, 0.0));
    let (recu, peint) = textes_de_loin(&store, |p, quoi, ecran| {
        super::storyboard::textes(p, quoi, ecran)
    });
    assert!(
        recu && peint,
        "de loin, les cases écrivent ({recu}, {peint})"
    );
}
