//! Ce que la boucle dit et montre d'un canevas partagé (fiche 117).

use super::*;

fn pair(nom: &str, couleur: u8, curseur: Option<(&str, f64, f64)>) -> Pair {
    Pair {
        nom: nom.into(),
        couleur,
        curseur: curseur.map(|(t, x, y)| (t.to_string(), x, y)),
        vu: Instant::now(),
    }
}

/// **Seuls se montrent les curseurs des autres qui regardent ce tableau-ci**, de leur couleur — pas
/// celui qui regarde un autre onglet, ni celui dont la souris n'a jamais bougé.
#[test]
fn les_curseurs_qui_se_montrent_sont_ceux_de_ce_tableau() {
    let (a, b, c) = (
        pair("Renard Bleu", 2, Some(("main", 10.0, 20.0))),
        pair("Ours Rouge", 0, Some(("autre", 1.0, 1.0))),
        pair("Loup Vert", 1, None),
    );
    let montres = curseurs_sur(&[&a, &b, &c], "main");
    assert_eq!(
        montres,
        [crate::params::CurseurDuPair {
            monde: (10.0, 20.0),
            couleur: identite::COULEURS[2],
            nom: "Renard Bleu".into(),
        }]
    );
}

/// **Ce qui n'a pas tenu, en une phrase** : un geste de lecteur défait, ou deux gestes croisés — et
/// rien quand tout a tenu.
#[test]
fn ce_qui_n_a_pas_tenu_se_dit_en_une_phrase() {
    assert_eq!(ce_qui_n_a_pas_tenu(false, 0), None);
    assert!(ce_qui_n_a_pas_tenu(true, 0).is_some_and(|p| p.contains("le plus récent")));
    assert!(ce_qui_n_a_pas_tenu(false, 2).is_some_and(|p| p.contains("ne fait que regarder")));
    assert!(ce_qui_n_a_pas_tenu(true, 1).is_some_and(|p| p.contains("ne fait que regarder")));
}

/// **Un code qui mène à une salle vide se dit** — dans la question de Collaborer, là où l'on a
/// rejoint — au lieu d'attendre en silence ; jamais par-dessus une autre question.
#[test]
fn un_code_qui_mene_a_une_salle_vide_se_dit() {
    let mut app = crate::app::GlucoseApp::new();
    let code = Code::neuf([9; 32]);
    let lien = lien::lancer(
        "http://127.0.0.1:9",
        code.clone(),
        0,
        crate::agents::Boite::nouvelle(std::sync::Arc::new(|| {})),
    );
    app.lancement.agents.partage = Some(CanevasPartage {
        lien,
        code,
        marque: "b0b".into(),
        nom: "Bob".into(),
        couleur: 0,
        entree: Entree::AttendLaBase,
        pairs: BTreeMap::new(),
        curseur_parti: None,
    });
    app.salle_a_jour(0);
    let Some((q, Suite::Collaborer)) = &app.ui.question else {
        panic!("la question de Collaborer");
    };
    assert!(q.texte.starts_with(SALLE_VIDE), "{}", q.texte);
    app.ui.question = Some((Question::default(), Suite::Telemetrie));
    app.salle_a_jour(0);
    assert!(
        matches!(app.ui.question, Some((_, Suite::Telemetrie))),
        "une autre question reste"
    );
}
