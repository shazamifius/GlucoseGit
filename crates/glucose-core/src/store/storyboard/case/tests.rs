//! Les épreuves de ce qu'une case dit pour la lecture (schéma 4, fiche 67) : un geste par
//! réglage, rien d'écrit quand rien ne change, la loi de la durée, et le champ retiré de
//! chaque case en même temps que son nom.

use super::super::{Grille, Plan, Storyboard};
use super::Texte;
use crate::store::Store;

/// Un storyboard de huit cases, et l'identifiant de chacune dans l'ordre.
fn storyboard() -> (Store, String, Vec<String>) {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    store.poser_le_storyboard(&board, Grille::default(), (0.0, 0.0));
    let ids = store.cases(&board).iter().map(|c| c.id.clone()).collect();
    (store, board, ids)
}

/// **Un storyboard naît aux réglages de Storyboarder** : deux secondes par case, et ses trois
/// champs — Dialogue, Action, Notes.
#[test]
fn test_un_storyboard_nait_aux_reglages_de_naissance() {
    let (store, board, _) = storyboard();
    let s = store.storyboard(&board).expect("posé");
    assert_eq!(s.duree_ms, 2000);
    assert_eq!(s.champs, ["Dialogue", "Action", "Notes"]);
}

/// **La durée d'une case : la sienne, sinon celle du storyboard** — et régler celle du
/// storyboard change toutes les cases qui n'ont pas la leur, aucune autre.
#[test]
fn test_la_duree_d_une_case_est_la_sienne_sinon_celle_du_storyboard() {
    let (mut store, board, ids) = storyboard();
    store.regler_la_duree(&board, &ids[1], Some(750));
    store.regler_la_duree_du_storyboard(&board, 4000);
    let s = store.storyboard(&board).expect("posé").clone();
    let cases = store.cases(&board);
    assert_eq!(
        s.duree(cases[0]),
        4000,
        "sans la sienne : celle du storyboard"
    );
    assert_eq!(s.duree(cases[1]), 750, "la sienne");

    store.regler_la_duree(&board, &ids[1], None);
    let cases = store.cases(&board);
    assert_eq!(s.duree(cases[1]), 4000, "rendue au storyboard");
}

/// **Une durée nulle ne montrerait pas la case** : elle vaut au moins une milliseconde, pour la
/// case comme pour le storyboard.
#[test]
fn test_une_duree_nulle_vaut_une_milliseconde() {
    let (mut store, board, ids) = storyboard();
    store.regler_la_duree(&board, &ids[0], Some(0));
    store.regler_la_duree_du_storyboard(&board, 0);
    assert_eq!(store.cases(&board)[0].duree_ms, Some(1));
    assert_eq!(store.storyboard(&board).expect("posé").duree_ms, 1);
}

/// **Chaque réglage est un geste, et un réglage qui ne change rien n'en est pas un** : un
/// `Ctrl+Z` défait exactement le dernier.
#[test]
fn test_chaque_reglage_est_un_geste_et_rien_ne_s_ecrit_sans_changement() {
    let (mut store, board, ids) = storyboard();
    let depart = store.undo_depth();
    store.ecrire_dans_la_case(&board, &ids[2], Texte::Champ(1), "il court");
    store.choisir_le_plan(&board, &ids[2], Some(Plan::Gros));
    store.regler_la_duree(&board, &ids[2], Some(1200));
    store.nommer_le_champ(&board, 0, "Réplique");
    assert_eq!(store.undo_depth(), depart + 4, "quatre gestes");

    store.ecrire_dans_la_case(&board, &ids[2], Texte::Champ(1), "il court");
    store.choisir_le_plan(&board, &ids[2], Some(Plan::Gros));
    store.regler_la_duree(&board, &ids[2], Some(1200));
    store.nommer_le_champ(&board, 0, "Réplique");
    store.regler_la_duree_du_storyboard(&board, 2000);
    assert_eq!(
        store.undo_depth(),
        depart + 4,
        "rien n'a changé : rien n'est écrit"
    );

    assert!(store.undo());
    assert_eq!(
        store.storyboard(&board).expect("posé").champs[0],
        "Dialogue"
    );
    assert_eq!(
        store.cases(&board)[2].duree_ms,
        Some(1200),
        "le reste tient"
    );
}

/// **Écrire un champ garde la forme unique** : les rangs d'avant comblés de vide, aucun vide
/// au bout — une case qu'on vide redevient égale à une case neuve.
#[test]
fn test_ecrire_un_champ_garde_la_forme_unique() {
    let (mut store, board, ids) = storyboard();
    store.ecrire_dans_la_case(&board, &ids[0], Texte::Champ(2), "pluie");
    assert_eq!(store.cases(&board)[0].textes, ["", "", "pluie"]);
    store.ecrire_dans_la_case(&board, &ids[0], Texte::Champ(2), "");
    assert!(store.cases(&board)[0].textes.is_empty());
    assert!(!store.cases(&board)[0].porte_quelque_chose());
}

/// **Retirer un champ retire son texte de chaque case, en un geste** — et les textes des champs
/// d'après glissent avec leur nom. `Ctrl+Z` rend le nom et chaque texte.
#[test]
fn test_retirer_un_champ_retire_son_texte_de_chaque_case_en_un_geste() {
    let (mut store, board, ids) = storyboard();
    store.ecrire_dans_la_case(&board, &ids[0], Texte::Champ(0), "« Halte ! »");
    store.ecrire_dans_la_case(&board, &ids[0], Texte::Champ(1), "il s'arrête");
    store.ecrire_dans_la_case(&board, &ids[3], Texte::Champ(1), "elle tombe");
    store.ecrire_dans_la_case(&board, &ids[3], Texte::Champ(2), "nuit");
    let avant = store.project.clone();
    let depart = store.undo_depth();

    store.retirer_le_champ(&board, 1);
    assert_eq!(store.undo_depth(), depart + 1, "un geste");
    assert_eq!(
        store.storyboard(&board).expect("posé").champs,
        ["Dialogue", "Notes"]
    );
    let cases = store.cases(&board);
    assert_eq!(cases[0].textes, ["« Halte ! »"], "le vide au bout s'en va");
    assert_eq!(cases[3].textes, ["", "nuit"], "les Notes suivent leur nom");

    assert!(store.undo());
    assert_eq!(store.project, avant, "le nom et chaque texte reviennent");
}

/// **Ajouter un champ** : au bout, vide dans chaque case — aucune case ne change.
#[test]
fn test_ajouter_un_champ_ne_change_aucune_case() {
    let (mut store, board, _) = storyboard();
    let cases = store.cases(&board).into_iter().cloned().collect::<Vec<_>>();
    assert_eq!(store.ajouter_un_champ(&board, "Son"), Some(3));
    assert_eq!(store.storyboard(&board).expect("posé").champs[3], "Son");
    let apres: Vec<_> = store.cases(&board).into_iter().cloned().collect();
    assert_eq!(apres, cases);
}

/// **Une case qui porte un plan, une durée ou un texte ne se retire jamais d'un tracé** — comme
/// celle qui porte une description ou une image (STORYBOARD-2). À l'envers : avant le schéma 4,
/// seules la description et les images comptaient, et la case partait avec son plan.
#[test]
fn test_un_trace_garde_la_case_qui_porte_un_plan_ou_un_texte() {
    for regler in [
        |s: &mut Store, b: &str, id: &str| s.choisir_le_plan(b, id, Some(Plan::Moyen)),
        |s: &mut Store, b: &str, id: &str| s.regler_la_duree(b, id, Some(500)),
        |s: &mut Store, b: &str, id: &str| s.ecrire_dans_la_case(b, id, Texte::Champ(2), "x"),
    ] {
        let (mut store, board, ids) = storyboard();
        regler(&mut store, &board, &ids[5]);
        assert_eq!(store.cases_a_garder(&board), 6, "jusqu'à la sixième");
        let d = store.disposition_du_trace(&board, Grille::default(), ((0.0, 0.0), (1.0, 1.0)));
        assert_eq!(d.nombre, 6, "le tracé d'une case en garde six");
    }
}

/// **Poser de nouveau la grille garde la durée et les champs du storyboard** : « Mettre à jour »
/// ne change que ce qui range les cases.
#[test]
fn test_ranger_garde_la_duree_et_les_champs() {
    let (mut store, board, _) = storyboard();
    store.regler_la_duree_du_storyboard(&board, 900);
    store.nommer_le_champ(&board, 2, "Son");
    let colonnes = Grille {
        colonnes: 2,
        ..Grille::default()
    };
    store.poser_le_storyboard(&board, colonnes, (10.0, 10.0));
    let s = store.storyboard(&board).expect("posé");
    assert_eq!(
        *s,
        Storyboard {
            grille: colonnes,
            duree_ms: 900,
            champs: vec!["Dialogue".into(), "Action".into(), "Son".into()],
        }
    );
}
