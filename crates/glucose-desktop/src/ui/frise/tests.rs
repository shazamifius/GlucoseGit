//! FRISE-1, sa mise en page et son dessin : une bande du bas qui pousse la minimap et les barres
//! au-dessus d'elle ; la main trouve la poignée la plus proche ; la piste montre la fenêtre ambre
//! et la répartition des dates du document.

use super::*;
use crate::bench::render_frame;
use crate::renderer::Renderer;
use crate::ui::UiState;
use glucose_core::types::{Annotation, TemporalAnchor};

const ECRAN: (f32, f32) = (1440.0, 900.0);

fn ouverte(vue: TemporalFilter) -> UiState {
    let mut ui = UiState::new();
    ui.frise = Some(Frise { vue, tenue: None });
    ui
}

/// **Une bande du bas** : dans l'écran, sur toute sa largeur moins les bords ; **la minimap se pose
/// au-dessus d'elle**, comme au-dessus du clavier d'Android, et une barre de sélection aussi.
#[test]
fn frise_1_elle_tient_le_bas_et_pousse_la_minimap() {
    let ui = ouverte(TemporalFilter {
        start: 1700,
        end: 2000,
    });
    let s = ui.scale();
    let p = placer(ui.ecran_du_systeme(ECRAN), s);
    let (x, y, l, h) = p.rect;
    assert!(
        x >= 0.0 && y >= 0.0 && x + l <= ECRAN.0 && y + h <= ECRAN.1,
        "{:?}",
        p.rect
    );
    assert_eq!(ECRAN.1 - (y + h), BORD * s, "au bord du bas");
    let (_, my, _, mh) = crate::ui::minimap::emprise(ui.ecran_visible(ECRAN), s);
    assert!(
        my + mh <= y,
        "la minimap au-dessus : {} contre {y}",
        my + mh
    );
    let barre = crate::ui::bas::poser(
        ui.ecran_visible(ECRAN),
        s,
        (300.0, 37.0),
        ui.ecran_visible(ECRAN).1,
    );
    assert!(barre.1 + 37.0 <= y, "une barre au-dessus");
    // Fermée, le bas revient au système.
    assert_eq!(UiState::new().ecran_visible(ECRAN), ECRAN);
}

/// **La main trouve la poignée la plus proche**, de loin comme au doigt (24 points) ; entre les
/// deux, la fenêtre ; la croix ; ailleurs dans la frise ; rien hors d'elle.
#[test]
fn frise_1_ce_qui_est_sous_la_main() {
    let vue = TemporalFilter {
        start: 1700,
        end: 2099,
    };
    let fenetre = TemporalFilter {
        start: 1800,
        end: 1899,
    };
    let p = placer(ECRAN, 1.0);
    let (a, b) = bords(fenetre, vue, p.piste);
    let y = p.piste.1 + p.piste.3 / 2.0;
    let sous = |x: f32, y: f32| endroit(&p, (vue, fenetre), (x, y));
    assert_eq!(
        sous(a + 20.0, y),
        Some(Endroit::Prise(Prise::Debut)),
        "à 20 points du début"
    );
    assert_eq!(sous(b - 3.0, y), Some(Endroit::Prise(Prise::Fin)));
    assert_eq!(sous((a + b) / 2.0, y), Some(Endroit::Prise(Prise::Fenetre)));
    assert_eq!(
        sous(a - 60.0, y),
        Some(Endroit::Ailleurs),
        "hors de la fenêtre, loin des poignées"
    );
    let croix = (p.fermer.0 + 2.0, p.fermer.1 + 2.0);
    assert_eq!(sous(croix.0, croix.1), Some(Endroit::Fermer));
    assert_eq!(sous(ECRAN.0 / 2.0, 100.0), None, "hors d'elle");
    // Une fenêtre d'un an : ses deux bords sont à moins d'un point — le plus proche gagne.
    let un_an = TemporalFilter {
        start: 1900,
        end: 1900,
    };
    let (a, b) = bords(un_an, vue, p.piste);
    let proche = |x| endroit(&p, (vue, un_an), (x, y));
    assert_eq!(proche(a - 10.0), Some(Endroit::Prise(Prise::Debut)));
    assert_eq!(proche(b + 10.0), Some(Endroit::Prise(Prise::Fin)));
}

/// **Une année et son abscisse se répondent.**
#[test]
fn frise_1_l_annee_et_l_abscisse() {
    let vue = TemporalFilter {
        start: -500,
        end: 1499,
    };
    let piste = (100.0, 0.0, 1000.0, 40.0);
    assert_eq!(x_de(-500, vue, piste), 100.0);
    assert_eq!(
        x_de(1500, vue, piste),
        1100.0,
        "la fin de la dernière année"
    );
    for an in [-500, 0, 476, 1499] {
        assert!((an_de(x_de(an, vue, piste), vue, piste) - an as f64).abs() < 1e-6);
    }
}

/// Le texte « a » daté de 1969, ou rien.
fn store(datee: bool) -> glucose_core::store::Store {
    let mut store = glucose_core::store::Store::new("frise");
    let b = store.project.active_board_id.clone();
    store.add_annotation(&b, Annotation::text("a", 0.0, 0.0, "la Lune"));
    if datee {
        let lune = TemporalAnchor {
            start: 1969,
            end: 1969,
            label: None,
        };
        store.ancrer_dans_le_temps(&b, &["a".to_string()], Some(&lune));
    }
    store.clear_selection();
    store
}

/// Les pixels qui diffèrent entre deux images, dans cette boîte.
fn ecarts(a: &tiny_skia::Pixmap, b: &tiny_skia::Pixmap, (x, y, l, h): Boite) -> usize {
    let mut n = 0;
    for py in y as u32..(y + h) as u32 {
        for px in x as u32..(x + l) as u32 {
            if a.pixel(px, py) != b.pixel(px, py) {
                n += 1;
            }
        }
    }
    n
}

/// **La piste dit où sont les dates** : avec un nœud daté de 1969, la colonne de 1969 porte une
/// barre — et rien ailleurs ne change ; **la fenêtre est ambre**.
#[test]
fn frise_1_la_piste_montre_les_dates_et_la_fenetre() {
    let vue = TemporalFilter {
        start: 1900,
        end: 2049,
    };
    let rendre = |store: &glucose_core::store::Store, fenetre: TemporalFilter| {
        let mut ui = ouverte(vue);
        let mut store = store.clone();
        store.set_temporal_filter(Some(fenetre));
        let mut renderer = Renderer::new();
        render_frame(
            &mut renderer,
            &mut ui,
            &store,
            ECRAN.0 as u32,
            ECRAN.1 as u32,
        )
    };
    let p = placer(ECRAN, UiState::new().scale());
    let (avec, sans) = (rendre(&store(true), vue), rendre(&store(false), vue));
    let pas = tick_step(vue.end - vue.start);
    let colonne = |debut: i64| {
        let (a, b) = (x_de(debut, vue, p.piste), x_de(debut + pas, vue, p.piste));
        (a + 1.0, p.piste.1, b - a - 2.0, p.piste.3)
    };
    let dix = 1960 - 1960 % pas;
    assert!(ecarts(&avec, &sans, colonne(dix)) > 20, "la barre de 1969");
    assert_eq!(ecarts(&avec, &sans, colonne(1900)), 0, "rien en 1900");
    // La fenêtre resserrée sur 1950-1979 : l'ambre est dedans, pas à côté.
    let fenetre = TemporalFilter {
        start: 1950,
        end: 1979,
    };
    let image = rendre(&store(true), fenetre);
    let chaud = |x: f32| {
        let px = image
            .pixel(x as u32, (p.piste.1 + 2.0) as u32)
            .expect("dans l'image");
        px.red() > px.blue()
    };
    let (a, b) = bords(fenetre, vue, p.piste);
    assert!(chaud((a + b) / 2.0), "dans la fenêtre, l'ambre");
    assert!(!chaud(x_de(1910, vue, p.piste)), "hors d'elle, aucun ambre");
}

/// **Un nom d'ère qui ne tient pas se coupe**, avec « … », et ce qui en reste tient ; entier s'il
/// tient ; rien s'il ne tient pas même une lettre.
#[test]
fn frise_1_un_nom_coupe_garde_son_debut() {
    let typo = Typography::new();
    let corps = PETIT;
    let nom = "Révolution industrielle";
    let entier = typo.measure_text(nom, corps, Face::Regular).0;
    assert_eq!(
        ce_qui_tient(&typo, nom, corps, entier + 1.0).as_deref(),
        Some(nom)
    );
    let coupe = ce_qui_tient(&typo, nom, corps, entier / 2.0).expect("un début");
    assert!(
        coupe.starts_with("Révol") && coupe.ends_with('…'),
        "{coupe}"
    );
    assert!(typo.measure_text(&coupe, corps, Face::Regular).0 <= entier / 2.0);
    assert_eq!(ce_qui_tient(&typo, nom, corps, 1.0), None);
}

/// **FRISE-2 — chaque ère dans la rangée de son couloir**, dans la vraie image : de 1700 à 2000,
/// la révolution industrielle (le troisième couloir) est peinte dès 1770 — la loi d'avant y
/// montrait les Lumières — et sa rangée est vide avant elle ; les Lumières dans le deuxième,
/// l'Holocène dans le premier. Les trois couloirs de la vue sont **serrés contre la piste** : la
/// révolution industrielle sur la dernière rangée, et la rangée au-dessus de l'Holocène reste au
/// fond de la frise.
#[test]
fn frise_2_chaque_ere_dans_sa_rangee() {
    let vue = TemporalFilter {
        start: 1700,
        end: 2000,
    };
    let mut ui = ouverte(vue);
    let mut store = store(false);
    store.set_temporal_filter(Some(vue));
    let mut renderer = Renderer::new();
    let image = render_frame(
        &mut renderer,
        &mut ui,
        &store,
        ECRAN.0 as u32,
        ECRAN.1 as u32,
    );
    let p = placer(ECRAN, UiState::new().scale());
    let derniere = rangee_d_ere(&p, profondeur(DEFAULT_ERAS) - 1);
    assert!(
        derniere.1 + derniere.3 <= p.piste.1,
        "toutes les rangées tiennent au-dessus de la piste"
    );
    let eres = eres_de_la_vue(vue, DEFAULT_ERAS);
    let rangee = rangees(&eres);
    assert_eq!(
        rangee(2),
        profondeur(DEFAULT_ERAS) - 1,
        "serrées contre la piste"
    );
    let au = |rangee: usize, an: i64| {
        let r = rangee_d_ere(&p, rangee);
        image
            .pixel((x_de(an, vue, p.eres) + 1.0) as u32, (r.1 + 1.0) as u32)
            .expect("dans l'image")
    };
    let vide = au(rangee(0) - 1, 1770);
    assert_ne!(
        au(rangee(2), 1770),
        vide,
        "la révolution industrielle, dès 1770"
    );
    assert_eq!(
        au(rangee(2), 1720),
        vide,
        "rien avant elle dans son couloir"
    );
    assert_ne!(au(rangee(1), 1720), vide, "les Lumières");
    assert_ne!(au(rangee(0), 1720), vide, "l'Holocène");
}

/// **Ouverte, la frise attend une décision** — sans rien de choisi : c'est ce qui la fait peindre
/// sur la voie graphique (`attend_une_decision`, DECISION-1). Fermée, rien n'attend.
#[test]
fn frise_1_ouverte_elle_attend_une_decision() {
    let store = store(false);
    assert!(!UiState::new().attend_une_decision(&store), "rien n'attend");
    let ouverte = ouverte(TemporalFilter {
        start: 1900,
        end: 2000,
    });
    assert!(
        ouverte.attend_une_decision(&store),
        "la frise attend, sans rien de choisi"
    );
}
