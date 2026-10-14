//! Les épreuves de la lecture dans le panneau (fiche 67) : chaque bouton dit sa demande, et
//! rien n'existe sans storyboard ni case choisie.

use super::super::{click_storyboard_panel, layout_storyboard_panel, Contenu, StoryboardState};
use super::*;

fn cadre() -> ScaledRect {
    ScaledRect {
        x: 0.0,
        y: 0.0,
        w: 260.0,
        h: 900.0,
        scale: 1.0,
    }
}

fn au_centre(r: WidgetRect) -> Pointer {
    Pointer {
        x: r.x + r.w / 2.0,
        y: r.y + r.h / 2.0,
    }
}

/// Trois cases ; la deuxième choisie, au plan moyen, avec sa durée à elle si `propre`.
fn contenu(propre: bool) -> Contenu {
    Contenu {
        cases: (0..3)
            .map(|i| (format!("case-{i}"), i, String::new()))
            .collect(),
        lecture: Some(Lecture {
            champs: vec!["Dialogue".into(), "Action".into(), "Notes".into()],
            duree_ms: 2000,
            case: Some(Choisie {
                id: "case-1".into(),
                numero: 2,
                description: "Elle ouvre".into(),
                textes: vec![String::new(), "elle court".into(), String::new()],
                plan: Some(Plan::Moyen),
                duree_ms: propre.then_some(800),
            }),
        }),
    }
}

/// **Chaque bouton de la lecture dit sa demande** : lire depuis la case choisie, rendre sa
/// durée au storyboard, le plan suivant, écrire chacun de ses textes, nommer un champ, en
/// ajouter un.
#[test]
fn test_chaque_bouton_de_la_lecture_dit_sa_demande() {
    let mut state = StoryboardState::default();
    let contenu = contenu(true);
    let layout = layout_storyboard_panel(cadre(), &state, &contenu);
    let l = layout.lecture.clone().expect("un storyboard : la lecture");
    let case = l.case.clone().expect("une case choisie");
    let mut cliquer =
        |r: WidgetRect| click_storyboard_panel(&mut state, (&layout, &contenu), au_centre(r));
    assert_eq!(
        cliquer(l.lire),
        Some(StoryboardIntent::Lire(Some("case-1".into())))
    );
    let rendre = case.rendre.expect("elle a sa durée : « ↺ »");
    assert_eq!(
        cliquer(rendre),
        Some(StoryboardIntent::RendreLaDuree("case-1".into()))
    );
    assert_eq!(
        cliquer(case.plan),
        Some(StoryboardIntent::Plan(
            "case-1".into(),
            Some(Plan::Americain)
        )),
        "le plan d'après, en tournant"
    );
    assert_eq!(case.textes.len(), 4, "la description et les trois champs");
    assert_eq!(
        cliquer(case.textes[0].ecrit),
        Some(StoryboardIntent::Ecrire(
            "case-1".into(),
            Texte::Description
        ))
    );
    assert_eq!(
        cliquer(case.textes[0].nom),
        Some(StoryboardIntent::Ecrire(
            "case-1".into(),
            Texte::Description
        )),
        "la description n'est pas un champ : son nom ne se renomme pas"
    );
    assert_eq!(
        cliquer(case.textes[2].ecrit),
        Some(StoryboardIntent::Ecrire("case-1".into(), Texte::Champ(1)))
    );
    assert_eq!(
        cliquer(case.textes[2].nom),
        Some(StoryboardIntent::NommerLeChamp(Some(1)))
    );
    assert_eq!(
        cliquer(case.ajouter_un_champ),
        Some(StoryboardIntent::NommerLeChamp(None))
    );
}

/// **« ↺ » n'existe que pour une case qui a sa durée** ; sans case choisie, ni durée ni textes ;
/// sans storyboard, pas de lecture.
#[test]
fn test_la_lecture_n_existe_que_pour_ce_qui_existe() {
    let state = StoryboardState::default();
    let sans_la_sienne = layout_storyboard_panel(cadre(), &state, &contenu(false));
    let case = sans_la_sienne
        .lecture
        .and_then(|l| l.case)
        .expect("la case");
    assert!(case.rendre.is_none());

    let mut aucune = contenu(false);
    if let Some(l) = &mut aucune.lecture {
        l.case = None;
    }
    let sans_case = layout_storyboard_panel(cadre(), &state, &aucune);
    assert!(sans_case.lecture.expect("la lecture").case.is_none());

    let rien = Contenu {
        lecture: None,
        ..contenu(false)
    };
    assert!(layout_storyboard_panel(cadre(), &state, &rien)
        .lecture
        .is_none());
}

/// **La liste des cases passe sous la lecture**, sans la chevaucher.
#[test]
fn test_la_liste_passe_sous_la_case_choisie() {
    let state = StoryboardState::default();
    let layout = layout_storyboard_panel(cadre(), &state, &contenu(true));
    let case = layout.lecture.and_then(|l| l.case).expect("la case");
    let (haut, _) = layout.liste.expect("la liste");
    let bas = case.ajouter_un_champ.y + case.ajouter_un_champ.h;
    assert!(
        haut > bas,
        "le titre de la liste ({haut}) sous « + Champ » ({bas})"
    );
}
