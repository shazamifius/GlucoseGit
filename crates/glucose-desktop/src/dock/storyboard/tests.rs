//! Les épreuves du panneau STORYBOARD (STORYBOARD-1) : chaque bouton dit ce qu'il demande, et
//! la liste des cases se lit page par page.

use super::*;

/// Un cadre de panneau : 260 de large, `hauteur` de haut, à l'échelle 1.
fn cadre(hauteur: f32) -> ScaledRect {
    ScaledRect {
        x: 0.0,
        y: 0.0,
        w: 260.0,
        h: hauteur,
        scale: 1.0,
    }
}

fn au_centre(r: WidgetRect) -> Pointer {
    Pointer {
        x: r.x + r.w / 2.0,
        y: r.y + r.h / 2.0,
    }
}

/// `n` cases, décrites ou non.
fn contenu(n: usize) -> Contenu {
    Contenu {
        cases: (0..n)
            .map(|i| (format!("case-{i}"), i as i32, format!("plan {i}")))
            .collect(),
    }
}

/// **Sans cases : « Activer » arme la grille réglée**, et ni croix, ni liste, ni rien à
/// renuméroter.
#[test]
fn test_sans_cases_activer_arme_la_grille() {
    let mut state = StoryboardState::default();
    let vide = Contenu::default();
    let layout = layout_storyboard_panel(cadre(600.0), &state, &vide);
    assert!(layout.effacer.is_none() && layout.liste.is_none() && layout.renumeroter.is_none());
    assert_eq!(layout.cells.len(), 8, "l'aperçu : deux rangées de quatre");
    let demande = click_storyboard_panel(
        &mut state,
        (&layout, &vide),
        au_centre(layout.activate_button),
    );
    assert_eq!(demande, Some(StoryboardIntent::Armer(state.grille())));
}

/// **Le format se choisit au clic**, en tournant parmi les cinq ; il ne part pas au canevas.
#[test]
fn test_le_format_tourne() {
    let mut state = StoryboardState::default();
    let vide = Contenu::default();
    let layout = layout_storyboard_panel(cadre(600.0), &state, &vide);
    let au_format = au_centre(layout.format_button);
    for attendu in [1, 2, 3, 4, 0] {
        assert_eq!(
            click_storyboard_panel(&mut state, (&layout, &vide), au_format),
            None
        );
        assert_eq!(state.format_idx, attendu);
    }
    state.format_idx = 4;
    assert_eq!(state.grille().format, Format::Vertical);
}

/// **Avec des cases : chaque bouton dit sa demande** — effacer, ajouter, décrire une case,
/// la retirer, renuméroter.
#[test]
fn test_chaque_bouton_dit_sa_demande() {
    let mut state = StoryboardState::default();
    let cases = contenu(3);
    let layout = layout_storyboard_panel(cadre(600.0), &state, &cases);
    let mut cliquer = |p: Pointer| click_storyboard_panel(&mut state, (&layout, &cases), p);
    let effacer = layout.effacer.expect("la croix");
    assert_eq!(cliquer(au_centre(effacer)), Some(StoryboardIntent::Effacer));
    let (_, ajouter) = layout.liste.expect("la liste");
    assert!(matches!(
        cliquer(au_centre(ajouter)),
        Some(StoryboardIntent::Ajouter(_))
    ));
    assert_eq!(layout.lignes.len(), 3);
    let deuxieme = &layout.lignes[1];
    assert_eq!(
        cliquer(au_centre(deuxieme.description)),
        Some(StoryboardIntent::Decrire("case-1".into()))
    );
    assert_eq!(
        cliquer(au_centre(deuxieme.retirer)),
        Some(StoryboardIntent::Retirer("case-1".into()))
    );
    let renumeroter = layout.renumeroter.expect("le bouton");
    assert_eq!(
        cliquer(au_centre(renumeroter)),
        Some(StoryboardIntent::Renumeroter)
    );
}

/// **Toutes les cases restent joignables** : quand elles ne tiennent pas, des pages ; la
/// dernière montre la dernière case, et la page ne sort jamais de la liste.
#[test]
fn test_les_cases_se_lisent_par_pages() {
    let mut state = StoryboardState::default();
    let cases = contenu(40);
    let layout = layout_storyboard_panel(cadre(600.0), &state, &cases);
    let (precedente, suivante, page, pages) = layout.pages.expect("des pages");
    assert_eq!(page, 0);
    assert!(pages > 1);
    let par_page = layout.lignes.len();
    // L'application refait la mise en page à chaque image : chaque clic lit la page montrée.
    for _ in 0..pages + 3 {
        let montree = layout_storyboard_panel(cadre(600.0), &state, &cases);
        click_storyboard_panel(&mut state, (&montree, &cases), au_centre(suivante));
    }
    assert_eq!(state.page, pages - 1, "la page s'arrête à la dernière");
    let derniere = layout_storyboard_panel(cadre(600.0), &state, &cases);
    let rangs: Vec<usize> = derniere.lignes.iter().map(|l| l.rang).collect();
    assert_eq!(rangs.last(), Some(&39), "la dernière case se voit");
    assert!(rangs.len() <= par_page);
    click_storyboard_panel(&mut state, (&derniere, &cases), au_centre(precedente));
    assert_eq!(state.page, pages - 2);
    // Toutes les lignes tiennent au-dessus de « Re-numéroter ».
    let bas = derniere.renumeroter.expect("le bouton").y;
    assert!(derniere.lignes.iter().all(|l| l.ligne.y + l.ligne.h <= bas));
}
