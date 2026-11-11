//! Les épreuves de l'habit d'une carte : PAPIER-1.

use glucose_core::types::{Annotation, Viewport};

/// Deux pixels rendus par l'application, voie du processeur : le papier d'un texte — loin de ses
/// lettres —, et le canevas ou la membrane loin du texte. Le texte peut porter une couleur ; une
/// membrane de la couleur donnée peut le contenir.
fn rendre(couleur: Option<&str>, membrane: Option<&str>) -> [(u8, u8, u8); 2] {
    let mut app = crate::app::GlucoseApp::new();
    app.ui.current_toast = None;
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let tableau = app.store.project.active_board_id.clone();
    if let Some(teinte) = membrane {
        let mut m = Annotation::membrane("m", -400.0, -400.0, 1200.0, 900.0);
        if let Annotation::Membrane { color, .. } = &mut m {
            *color = Some(teinte.to_string());
        }
        app.store.add_annotation(&tableau, m);
    }
    let mut texte = Annotation::text("t", 0.0, 0.0, "Eau");
    if let Annotation::Text { color, .. } = &mut texte {
        *color = couleur.map(str::to_string);
    }
    app.store.add_annotation(&tableau, texte);
    app.store.clear_selection();
    let vue = Viewport {
        x: 40.0,
        y: 300.0,
        scale: 1.0,
    };
    app.store.set_viewport(&tableau, vue);
    app.une_image_sans_fenetre((800, 600));
    let image = app.pixmap.as_ref().expect("une image");
    let lire = |x: usize, y: usize| {
        let p = image.pixels()[y * image.width() as usize + x].demultiply();
        (p.red(), p.green(), p.blue())
    };
    // Le papier, à droite du mot ; le monde en (500, −150), loin du texte et entre deux points
    // de la grille.
    [lire(220, 340), lire(540, 150)]
}

/// **PAPIER-1 — le papier d'un texte est le gris sombre et neutre des blocs, partout** (fiche 105 ;
/// sa parole du 11/11/2026, *« TOUJOURS gris »*). Sur le canevas nu, dans une membrane verte, avec
/// une couleur choisie, les deux à la fois : le même gris, au niveau près.
///
/// **La preuve à l'envers** : là où le texte est posé, la membrane est vraiment colorée — l'ancienne
/// loi (LUEUR-3, le papier vu au travers des membranes) aurait donné cette couleur au papier, et la
/// couleur choisie l'aurait teinté (COULEUR-3, fiche 99).
#[test]
fn papier_1_le_papier_d_un_texte_est_le_gris_des_blocs_partout() {
    // Le jeton du thème, lu directement : comparer l'image à `papier` comparerait la fonction à
    // elle-même (le sabotage « un autre gris du thème » passait).
    let c = crate::theme::Theme::dark().card_paper.to_color_u8();
    let gris = (c.red(), c.green(), c.blue());
    let (r, g, b) = gris;
    assert!(r == g && g == b, "le papier est neutre : {gris:?}");
    for (couleur, membrane) in [
        (None, None),
        (Some("#f472b6"), None),
        (None, Some("#22c55e")),
        (Some("#f472b6"), Some("#22c55e")),
    ] {
        let [papier, autour] = rendre(couleur, membrane);
        assert_eq!(
            papier, gris,
            "le papier d'un texte {couleur:?} dans une membrane {membrane:?}"
        );
        if membrane.is_some() {
            let ecart = [autour.0, autour.1, autour.2]
                .iter()
                .zip([r, g, b])
                .map(|(a, b)| a.abs_diff(b))
                .max();
            assert!(
                ecart >= Some(12),
                "la membrane autour du texte doit être colorée : {autour:?} contre {gris:?}"
            );
        }
    }
}

/// Un pixel de la lueur d'un texte « Eau » à la couleur `couleur` : juste sous son bloc, où la lueur
/// déborde (voie du processeur).
fn lueur_rendue(couleur: Option<&str>) -> (u8, u8, u8) {
    let mut app = crate::app::GlucoseApp::new();
    app.ui.current_toast = None;
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let tableau = app.store.project.active_board_id.clone();
    let mut texte = Annotation::text("t", 0.0, 0.0, "Eau");
    if let Annotation::Text { color, .. } = &mut texte {
        *color = couleur.map(str::to_string);
    }
    let (_, h) = texte.size().expect("une taille");
    app.store.add_annotation(&tableau, texte);
    app.store.clear_selection();
    let vue = Viewport {
        x: 40.0,
        y: 300.0,
        scale: 1.0,
    };
    app.store.set_viewport(&tableau, vue);
    app.une_image_sans_fenetre((800, 600));
    let image = app.pixmap.as_ref().expect("une image");
    let y = (300.0 + h + 12.0) as usize;
    let p = image.pixels()[y * image.width() as usize + 120].demultiply();
    (p.red(), p.green(), p.blue())
}

/// **COULEUR-4 — la couleur choisie d'un texte va à sa lueur** (fiche 106 ; sa parole du 11/11/2026 :
/// le geste qui colore un texte change *« le halo lumineux autour du bloc — PAS son background »*).
/// Rose, la lueur sous le bloc tire vers le rose : son rouge passe devant son vert. **La preuve à
/// l'envers** : sans couleur choisie, ce texte seul garde la lueur grise de son bloc — l'IA ne lui
/// connaît aucun parent, et ne lie sa couleur à rien (PARENTE-2).
#[test]
fn couleur_4_la_couleur_choisie_d_un_texte_va_a_sa_lueur() {
    let rose = lueur_rendue(Some("#f472b6"));
    let sans = lueur_rendue(None);
    assert_ne!(rose, sans, "la couleur choisie change la lueur");
    assert!(
        sans.0.abs_diff(sans.1) <= 1 && sans.1.abs_diff(sans.2) <= 1,
        "sans couleur choisie, la lueur d'un texte seul est grise : {sans:?}"
    );
    assert!(
        rose.0 > rose.1 + 2 && rose.2 > rose.1,
        "la lueur d'un texte rose tire vers le rose : {rose:?}"
    );
}
