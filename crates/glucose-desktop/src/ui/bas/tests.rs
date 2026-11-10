//! **Le bas de l'écran, de 320 à 2 560 points** : aucune barre ne sort de l'écran, ne touche la
//! minimap ni une autre barre ; aucun bouton ne déborde de sa barre (fiche 101).

use super::*;
use crate::typography::Typography;
use crate::ui::{action_bar, couleurs, options_de_fleche};
use glucose_core::groupe::Origine;
use glucose_core::store::Store;
use glucose_core::types::{Annotation, BoardImage};

type Rect = (f32, f32, f32, f32);

/// Les sélections qui font naître les barres : un texte (l'action et la couleur) ; douze photos
/// et trois textes (la barre d'action la plus large : le verrou et l'origine) ; une flèche et un
/// texte (les trois barres l'une sur l'autre).
fn selections() -> Vec<(&'static str, Store)> {
    let monter = |images: usize, textes: usize, fleche: bool| {
        let mut store = Store::new("bas");
        let board = store.project.active_board_id.clone();
        for k in 0..images {
            let photo = BoardImage::new(format!("i{k}"), 0.0, 0.0, 100.0, 100.0);
            store.add_image(&board, photo);
        }
        let mut ids: Vec<String> = (0..textes).map(|k| format!("t{k}")).collect();
        for id in &ids {
            store.add_annotation(&board, Annotation::text(id.clone(), 0.0, 0.0, "x"));
        }
        if fleche {
            store.add_annotation(&board, Annotation::arrow("f", 0.0, 0.0, 100.0, 0.0));
            ids.push("f".into());
        }
        store.set_selected_image_ids((0..images).map(|k| format!("i{k}")).collect());
        store.set_selected_annotation_ids(ids);
        store
    };
    vec![
        ("un texte", monter(0, 1, false)),
        ("douze photos et trois textes", monter(12, 3, false)),
        ("une flèche et un texte", monter(0, 1, true)),
    ]
}

/// Les barres posées, et ce que chacune porte.
fn barres(
    store: &Store,
    typo: &Typography,
    ecran: (f32, f32),
    k: f32,
) -> Vec<(&'static str, Rect, Vec<Rect>)> {
    let mut v = Vec::new();
    for origine in [Origine::Commune, Origine::Individuelle] {
        if let Some(b) = action_bar::layout_action_bar(store, typo, (ecran, k), origine) {
            v.push((
                "l'action",
                b.rect,
                b.buttons.iter().map(|b| b.rect).collect(),
            ));
        }
    }
    if let Some(b) = options_de_fleche::layout_options_de_fleche(store, typo, ecran, k) {
        v.push((
            "les flèches",
            b.rect,
            b.boutons.iter().map(|b| b.rect).collect(),
        ));
    }
    if let Some(b) = couleurs::layout_barre_de_couleur(store, typo, ecran, k) {
        v.push((
            "la couleur",
            b.rect,
            b.cases.iter().map(|c| c.rect).collect(),
        ));
    }
    v
}

/// Deux rectangles se recouvrent-ils (plus qu'un bord commun) ?
fn croisent((ax, ay, aw, ah): Rect, (bx, by, bw, bh): Rect) -> bool {
    ax < bx + bw - 0.01 && bx < ax + aw - 0.01 && ay < by + bh - 0.01 && by < ay + ah - 0.01
}

/// `a` contient-il `b` ?
fn contient((ax, ay, aw, ah): Rect, (bx, by, bw, bh): Rect) -> bool {
    bx >= ax - 0.01 && by >= ay - 0.01 && bx + bw <= ax + aw + 0.01 && by + bh <= ay + ah + 0.01
}

/// **De 320 à 2 560 points, aux échelles de 1 à 2,75** — un vieux téléphone, le sien (393 points
/// à 2,75), une fenêtre en demi-colonne, un bureau : chaque barre tient entre les bords de
/// l'écran, au-dessous du haut, hors de la minimap et des autres barres ; chaque bouton dans sa
/// barre, sans en chevaucher un autre.
#[test]
fn test_bas_1_aucune_barre_ne_sort_de_l_ecran_ni_ne_couvre_la_minimap() {
    let typo = Typography::new();
    for (nom, store) in selections() {
        for k in [1.0_f32, 1.25, 2.0, 2.75] {
            for points in (320..=2560).step_by(16) {
                let ecran = (points as f32 * k, 900.0 * k);
                let minimap = minimap::emprise(ecran, k);
                let posees = barres(&store, &typo, ecran, k);
                assert!(!posees.is_empty(), "{nom} : aucune barre");
                for (i, (barre, rect, boutons)) in posees.iter().enumerate() {
                    let ou = format!("{nom}, {points} points à {k} : {barre} {rect:?}");
                    let (x, y, w, _) = *rect;
                    let bord = minimap::BORD * k;
                    assert!(
                        x >= bord - 0.01 && x + w <= ecran.0 - bord + 0.01,
                        "{ou} sort de l'écran"
                    );
                    assert!(y >= 0.0, "{ou} passe le haut de l'écran");
                    assert!(
                        !croisent(*rect, minimap),
                        "{ou} couvre la minimap {minimap:?}"
                    );
                    // Les deux origines posent la même barre d'action : elles ne se comparent pas.
                    for (autre, r, _) in &posees[i + 1..] {
                        if autre != barre {
                            assert!(!croisent(*rect, *r), "{ou} couvre {autre} {r:?}");
                        }
                    }
                    for (j, b) in boutons.iter().enumerate() {
                        assert!(contient(*rect, *b), "{ou} : {b:?} déborde");
                        for c in &boutons[j + 1..] {
                            assert!(!croisent(*b, *c), "{ou} : {b:?} couvre {c:?}");
                        }
                    }
                }
            }
        }
    }
}

/// **Sur un bureau, rien ne change** : une rangée, la barre d'action à 14 points du bas, la barre
/// de couleur 8 points au-dessus — ce que Glucose montrait avant la fiche 101.
#[test]
fn test_bas_2_sur_un_bureau_les_barres_restent_en_bas_sur_une_rangee() {
    let typo = Typography::new();
    let (_, store) = selections().remove(0);
    let ecran = (1440.0, 900.0);
    let action = action_bar::layout_action_bar(&store, &typo, (ecran, 1.0), Origine::Commune)
        .expect("la barre d'action");
    assert_eq!(
        action.rect.1 + action.rect.3,
        900.0 - 14.0,
        "à 14 points du bas"
    );
    assert_eq!(
        action.rect.3,
        action.rangee + 2.0 * action_bar::PAD_Y,
        "une rangée"
    );
    let couleur =
        couleurs::layout_barre_de_couleur(&store, &typo, ecran, 1.0).expect("la barre de couleur");
    assert_eq!(
        couleur.rect.1 + couleur.rect.3,
        action.rect.1 - AU_DESSUS,
        "8 points au-dessus"
    );
    let cote = couleur.cases[0].rect.3;
    assert_eq!(couleur.rect.3, cote + 2.0 * action_bar::PAD_Y, "une rangée");
}

/// **Sur son téléphone, la couleur passe à la ligne et monte au-dessus de la minimap** : 1 080
/// pixels à 2,75 — le nuancier sur une rangée, « Par défaut » et la roue sur la suivante.
#[test]
fn test_bas_3_sur_son_telephone_la_couleur_passe_a_la_ligne_au_dessus_de_la_minimap() {
    let typo = Typography::new();
    let (_, store) = selections().remove(0);
    let k = 2.75;
    let ecran = (1080.0, 2400.0);
    let couleur = couleurs::layout_barre_de_couleur(&store, &typo, ecran, k).expect("la barre");
    let rangees: std::collections::BTreeSet<u32> = couleur
        .cases
        .iter()
        .map(|c| c.rect.1.round() as u32)
        .collect();
    assert_eq!(rangees.len(), 2, "deux rangées : {:?}", couleur.cases);
    // « Couleur » sur la première rangée, pas au milieu des deux.
    let premiere = couleur.cases[0].rect;
    let corps = action_bar::FONT * k;
    assert!(
        couleur.mot.1 >= premiere.1 && couleur.mot.1 + corps <= premiere.1 + premiere.3 + 0.01,
        "le mot {:?} hors de la première rangée {premiere:?}",
        couleur.mot
    );
    let (_, my, _, _) = minimap::emprise(ecran, k);
    let action = action_bar::layout_action_bar(&store, &typo, (ecran, k), Origine::Commune)
        .expect("la barre d'action");
    assert!(
        action.rect.1 + action.rect.3 <= my - AU_DESSUS * k + 0.01,
        "au-dessus de la minimap"
    );
}

/// **Basculer ne déplace rien sous la main** : l'origine commune ou individuelle, le verrou
/// ouvert ou fermé — chaque bouton garde sa place, et la barre sa taille.
#[test]
fn test_bas_4_un_bouton_a_deux_etats_ne_bouge_pas_quand_on_le_bascule() {
    let typo = Typography::new();
    let (_, mut store) = selections().remove(1);
    for ecran in [(1440.0, 900.0), (393.0, 873.0)] {
        let poser = |store: &Store, origine| {
            action_bar::layout_action_bar(store, &typo, (ecran, 1.0), origine).expect("la barre")
        };
        let commune = poser(&store, Origine::Commune);
        let individuelle = poser(&store, Origine::Individuelle);
        let rects = |b: &action_bar::ActionBar| {
            (b.rect, b.buttons.iter().map(|b| b.rect).collect::<Vec<_>>())
        };
        assert_eq!(
            rects(&commune),
            rects(&individuelle),
            "{ecran:?} : l'origine"
        );
        let board = store.project.active_board_id.clone();
        assert_eq!(
            store.toggle_lock_selection(&board),
            Some(true),
            "les photos se ferment"
        );
        let fermee = poser(&store, Origine::Commune);
        assert_eq!(fermee.buttons[0].label, "Verrouillé");
        assert_eq!(rects(&commune), rects(&fermee), "{ecran:?} : le verrou");
        store.toggle_lock_selection(&board);
    }
}

/// **Ranger, comme un texte** : tout sur une rangée quand cela tient ; sinon l'élément qui
/// dépasserait ouvre la suivante, sans son écart ; un élément trop large a sa rangée à lui.
#[test]
fn test_bas_5_ranger_passe_a_la_ligne_comme_un_texte() {
    let elements = [
        (0.0, 40.0),
        (10.0, 30.0),
        (5.0, 50.0),
        (5.0, 200.0),
        (5.0, 10.0),
    ];
    let (places, plus_longue) = ranger(&elements, 1000.0);
    assert!(places.iter().all(|&(r, _)| r == 0));
    assert_eq!(
        plus_longue,
        40.0 + 10.0 + 30.0 + 5.0 + 50.0 + 5.0 + 200.0 + 5.0 + 10.0
    );
    let (places, plus_longue) = ranger(&elements, 140.0);
    assert_eq!(
        places,
        vec![(0, 0.0), (0, 50.0), (0, 85.0), (1, 0.0), (2, 0.0)],
        "le quatrième dépasse, et le cinquième ne tient plus derrière lui"
    );
    assert_eq!(
        plus_longue, 200.0,
        "un élément trop large a sa rangée à lui"
    );
}

/// **Un téléphone couché, sa navigation à droite** : les barres évitent la minimap là où elle
/// est vraiment — décalée par la marge de droite —, et restent à gauche de la navigation.
/// L'écran visible ne retirait que le bas : les barres auraient cherché la minimap où elle
/// n'était pas.
#[test]
fn test_bas_6_couche_les_barres_evitent_la_minimap_decalee_par_la_navigation() {
    let typo = Typography::new();
    let (_, store) = selections().remove(0);
    let k = 2.75_f32;
    let mut ui = crate::ui::UiState::new();
    ui.marges = crate::plateforme::marges::Marges {
        droite: 48.0 * k,
        ..Default::default()
    };
    for points in (560..=1000).step_by(8) {
        let ecran = (points as f32 * k, 393.0 * k);
        let visible = ui.ecran_visible(ecran);
        let retrait = (ui.marges.droite, ui.marges.sous());
        let mm = crate::ui::layout_minimap(&store, ecran.0, ecran.1, k, retrait).expect("la carte");
        let minimap = (mm.mm_x, mm.mm_y, mm.mm_w, mm.mm_h);
        for (barre, rect, _) in barres(&store, &typo, visible, k) {
            let ou = format!("{points} points couché : {barre} {rect:?}");
            assert!(
                !croisent(rect, minimap),
                "{ou} couvre la minimap {minimap:?}"
            );
            assert!(
                rect.0 + rect.2 <= ecran.0 - ui.marges.droite,
                "{ou} passe sous la navigation"
            );
        }
    }
}

/// **Aucun filet n'ouvre une rangée** : sur un téléphone, la barre des flèches passe à la ligne
/// groupe par groupe, et le filet qui séparait deux groupes ne sépare plus rien en tête de
/// rangée — il n'est pas dessiné. Chaque filet a, sur sa rangée, un mot ou un bouton à sa gauche.
#[test]
fn test_bas_7_aucun_filet_n_ouvre_une_rangee() {
    let typo = Typography::new();
    let (_, store) = selections().remove(2);
    for (points, k) in [(320.0, 2.0), (393.0, 2.75), (480.0, 1.0)] {
        let ecran = (points * k, 900.0 * k);
        let b = options_de_fleche::layout_options_de_fleche(&store, &typo, ecran, k)
            .expect("la barre des flèches");
        let rangees: std::collections::BTreeSet<u32> =
            b.boutons.iter().map(|b| b.rect.1.round() as u32).collect();
        assert!(
            rangees.len() > 1,
            "{points} points : la barre devait passer à la ligne"
        );
        for &(fx, fy, fh) in &b.filets {
            let a_gauche = b
                .boutons
                .iter()
                .any(|o| (o.rect.1 - fy).abs() < 0.5 && o.rect.0 + o.rect.2 <= fx + 0.01)
                || b.mots
                    .iter()
                    .any(|&(_, (mx, my))| mx < fx && my >= fy && my < fy + fh);
            assert!(a_gauche, "{points} points : le filet {fx} ouvre sa rangée");
        }
    }
}
