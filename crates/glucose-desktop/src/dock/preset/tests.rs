//! PRESET-1 — le panneau des Presets : les gabarits à poser, celui que le tableau suit, et ce
//! qu'un clic demande.

use super::*;
use glucose_core::types::BoardImage;

fn cadre() -> ScaledRect {
    ScaledRect {
        x: 20.0,
        y: 90.0,
        w: 280.0,
        h: hauteur_naturelle(),
        scale: 1.0,
    }
}

fn centre(r: WidgetRect) -> (f32, f32) {
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

/// **Les six gabarits de Tauri, et rien de suivi** sur un tableau neuf ; un clic sur une rangée
/// arme son gabarit, un clic ailleurs ne demande rien.
#[test]
fn test_preset_1_un_clic_arme_le_gabarit_de_sa_rangee() {
    let store = Store::new("p");
    let contenu = Contenu::du(&store);
    let layout = layout_presets_panel(cadre(), &contenu);
    assert!(layout.suivie.is_none());
    assert_eq!(layout.rows.len(), 6);
    let creature = &layout.rows[2];
    assert_eq!(
        click_presets_panel(&layout, &contenu, centre(creature.rect)),
        Some(GabaritIntent::Armer("creature".into()))
    );
    assert_eq!(
        click_presets_panel(&layout, &contenu, centre(layout.create_button)),
        None,
        "« Créer » est grisé"
    );
}

/// **Le gabarit que suit le tableau** : son nom, « Retirer », une pastille par catégorie —
/// pleine dès qu'une image est posée dans sa zone. Celle de Tauri ne s'allumait jamais.
#[test]
fn test_preset_1_la_section_du_gabarit_suivi() {
    let mut store = Store::new("p");
    let board = store.project.active_board_id.clone();
    let gabarit = glucose_core::gabarits::integre("props").expect("props");
    let cats: Vec<&str> = gabarit.categories.iter().map(|c| c.id).collect();
    store.poser_un_gabarit(&board, (gabarit.id, &cats), (0.0, 0.0));
    let sketch = store.zones(&board)[1].clone();
    store.add_image(
        &board,
        BoardImage::new("i", sketch.x + 50.0, sketch.y + 100.0, 40.0, 40.0),
    );

    let contenu = Contenu::du(&store);
    let (nom, pastilles) = contenu.suivi.clone().expect("un gabarit suivi");
    assert_eq!(nom, "Props & Items");
    let remplies: Vec<bool> = pastilles.iter().map(|p| p.remplie).collect();
    assert_eq!(
        remplies,
        [false, true, false, false, false],
        "le croquis seul"
    );

    let layout = layout_presets_panel(cadre(), &contenu);
    let section = layout.suivie.as_ref().expect("la section");
    assert_eq!(section.pastilles.len(), 5);
    assert_eq!(
        click_presets_panel(&layout, &contenu, centre(section.retirer)),
        Some(GabaritIntent::Retirer)
    );
}

/// **Le panneau a la hauteur de ce qu'il montre de plus grand** : le gabarit le plus long suivi,
/// et les six à choisir, tiennent dans sa hauteur naturelle — aucune rangée ne déborde.
#[test]
fn test_preset_1_tout_tient_dans_la_hauteur_naturelle() {
    let mut store = Store::new("p");
    let board = store.project.active_board_id.clone();
    let gabarit = glucose_core::gabarits::integre("charadesign").expect("charadesign");
    let cats: Vec<&str> = gabarit.categories.iter().map(|c| c.id).collect();
    store.poser_un_gabarit(&board, (gabarit.id, &cats), (0.0, 0.0));
    let cadre = cadre();
    let layout = layout_presets_panel(cadre, &Contenu::du(&store));
    let bas = layout.create_button.y + layout.create_button.h;
    assert!(bas <= cadre.y + cadre.h, "{bas} sous {}", cadre.y + cadre.h);
}

/// **Un nom ne s'écrit que s'il tient** dans sa case : sinon il chevaucherait sa voisine —
/// CharaDesign a dix catégories sur deux cent cinquante points.
#[test]
fn test_preset_1_un_nom_qui_ne_tient_pas_se_tait() {
    let typo = crate::typography::Typography::new();
    let theme = crate::theme::Theme::dark();
    let brush = crate::dock::paint::Brush::nouveau(
        (&typo, &theme),
        1.0,
        crate::params::Pointer { x: -1.0, y: -1.0 },
        (0.0, 0.0),
    );
    let ecrit = |largeur: f32| {
        let mut p = tiny_skia::Pixmap::new(200, 40).expect("un pixmap");
        let rect = WidgetRect::new(0.0, 0.0, largeur, 30.0);
        brush.text_centered_if_fits(&mut p.as_mut(), rect, "Référence", 7.5, theme.text_primary);
        p.data().iter().any(|&o| o != 0)
    };
    assert!(ecrit(150.0), "il tient : il s'écrit");
    assert!(!ecrit(12.0), "il ne tient pas : rien");
}
