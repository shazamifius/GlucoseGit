//! Tests du journal d'éditions.
//!
//! Les six premiers vérifient le comportement (défaire, refaire, atomicité, ordre).
//! Le dernier vérifie **la loi de coût JRN-1 elle-même** : c'est celui qui compte, parce que
//! c'est la propriété que les snapshots ne pouvaient pas tenir et la raison d'être du module.

use super::*;
use crate::types::{Annotation, BoardImage, Project};

const BOARD: &str = "main";

fn project_with(images: usize) -> Project {
    let mut p = Project::new("test");
    let board = p
        .boards
        .iter_mut()
        .find(|b| b.id == BOARD)
        .expect("board main");
    for i in 0..images {
        board.images.push(BoardImage::new(
            format!("img-{i}"),
            i as f64,
            0.0,
            10.0,
            10.0,
        ));
    }
    p
}

fn images(p: &Project) -> Vec<String> {
    p.boards
        .iter()
        .find(|b| b.id == BOARD)
        .expect("board main")
        .images
        .iter()
        .map(|i| i.id.clone())
        .collect()
}

fn insert(index: usize, id: &str) -> Edit {
    Edit::Image {
        board: BOARD.to_string(),
        slot: Slot::inserted(index, BoardImage::new(id, 0.0, 0.0, 10.0, 10.0)),
    }
}

fn remove(index: usize, id: &str) -> Edit {
    Edit::Image {
        board: BOARD.to_string(),
        slot: Slot::removed(index, BoardImage::new(id, 0.0, 0.0, 10.0, 10.0)),
    }
}

/// Applique une édition au projet comme le ferait un site de mutation, puis l'enregistre.
/// C'est l'ordre réel : le store écrit, le journal note ce qui vient d'être écrit.
fn do_edit(journal: &mut Journal, project: &mut Project, edit: Edit) {
    assert!(edit.apply(project), "l'édition doit s'appliquer");
    journal.record(edit);
}

#[test]
fn inserer_puis_defaire_retire_l_element() {
    let mut p = project_with(2);
    let mut j = Journal::new(200);

    do_edit(&mut j, &mut p, insert(2, "neuf"));
    assert_eq!(images(&p), ["img-0", "img-1", "neuf"]);

    assert!(j.undo(&mut p));
    assert_eq!(images(&p), ["img-0", "img-1"]);

    assert!(j.redo(&mut p));
    assert_eq!(images(&p), ["img-0", "img-1", "neuf"]);
}

#[test]
fn supprimer_puis_defaire_remet_a_la_meme_place() {
    let mut p = project_with(3);
    let mut j = Journal::new(200);

    // On retire celui du milieu : c'est le cas qui attrape les erreurs de décalage d'index.
    do_edit(&mut j, &mut p, remove(1, "img-1"));
    assert_eq!(images(&p), ["img-0", "img-2"]);

    assert!(j.undo(&mut p));
    assert_eq!(
        images(&p),
        ["img-0", "img-1", "img-2"],
        "l'élément revient à son rang exact"
    );
}

#[test]
fn modifier_puis_defaire_restaure_la_valeur_precedente() {
    let mut p = project_with(2);
    let mut j = Journal::new(200);

    let before = BoardImage::new("img-1", 1.0, 0.0, 10.0, 10.0);
    let mut after = before.clone();
    after.x = 999.0;

    do_edit(
        &mut j,
        &mut p,
        Edit::Image {
            board: BOARD.to_string(),
            slot: Slot::changed(1, before, after),
        },
    );

    let x = |p: &Project| p.boards[0].images[1].x;
    assert_eq!(x(&p), 999.0);
    assert!(j.undo(&mut p));
    assert_eq!(x(&p), 1.0);
    assert!(j.redo(&mut p));
    assert_eq!(x(&p), 999.0);
}

#[test]
fn une_transaction_se_defait_en_un_seul_geste_et_dans_l_ordre_inverse() {
    let mut p = project_with(4);
    let mut j = Journal::new(200);

    // Deux suppressions dans le même geste. Les défaire dans le mauvais ordre remettrait les
    // éléments aux mauvaises places — c'est précisément ce que `Transaction::invert` évite.
    j.begin();
    do_edit(&mut j, &mut p, remove(1, "img-1"));
    do_edit(&mut j, &mut p, remove(2, "img-3")); // après le premier retrait, img-3 est en 2
    j.end();

    assert_eq!(images(&p), ["img-0", "img-2"]);
    assert_eq!(j.depth(), 1, "un geste, une entrée");

    assert!(j.undo(&mut p));
    assert_eq!(images(&p), ["img-0", "img-1", "img-2", "img-3"]);
    assert!(!j.can_undo(), "un seul Ctrl+Z suffisait");
}

#[test]
fn une_transaction_vide_ne_laisse_aucune_trace() {
    let p = project_with(1);
    let mut j = Journal::new(200);

    j.begin();
    j.end();

    assert!(
        !j.can_undo(),
        "un clic qui ne bouge rien n'est pas un geste"
    );
    assert_eq!(images(&p).len(), 1);
}

#[test]
fn annuler_une_transaction_ouverte_defait_ce_qu_elle_avait_ecrit() {
    let mut p = project_with(2);
    let mut j = Journal::new(200);

    j.begin();
    do_edit(&mut j, &mut p, insert(2, "provisoire"));
    assert_eq!(images(&p).len(), 3);

    assert!(j.cancel(&mut p));
    assert_eq!(
        images(&p),
        ["img-0", "img-1"],
        "Échap remet l'état d'avant le geste"
    );
    assert!(!j.can_undo(), "et ne laisse rien dans la pile");
}

#[test]
fn une_edition_nouvelle_efface_la_pile_de_retablissement() {
    let mut p = project_with(1);
    let mut j = Journal::new(200);

    do_edit(&mut j, &mut p, insert(1, "a"));
    assert!(j.undo(&mut p));
    assert!(j.can_redo());

    do_edit(&mut j, &mut p, insert(1, "b"));
    assert!(!j.can_redo(), "l'histoire repart d'ici");
}

#[test]
fn la_profondeur_est_bornee() {
    let mut p = project_with(0);
    let mut j = Journal::new(3);

    for i in 0..10 {
        do_edit(&mut j, &mut p, insert(i, &format!("img-{i}")));
    }

    assert_eq!(
        j.depth(),
        3,
        "au-delà de la profondeur, les gestes les plus anciens sortent"
    );
}

#[test]
fn un_index_devenu_faux_vide_le_journal_plutot_que_de_mentir() {
    // JRN-2 : si une édition contourne le journal, la pile ne décrit plus le document.
    let mut p = project_with(3);
    let mut j = Journal::new(200);

    do_edit(&mut j, &mut p, remove(2, "img-2"));

    // Quelqu'un vide la liste dans le dos du journal (chargement de projet, compaction…).
    p.boards[0].images.clear();

    assert!(!j.undo(&mut p), "l'undo échoue proprement");
    assert!(
        !j.can_undo() && !j.can_redo(),
        "et la pile est vidée, pas laissée fausse"
    );
}

/// **Le test qui compte.** La loi JRN-1 dit : `T(n, k) = α + β·k` avec `∂T/∂n = 0`.
///
/// On ne mesure pas des millisecondes — trop bruitées en CI — mais la grandeur dont elles
/// dépendent : le poids du journal. Si ce poids est identique pour le même geste dans un
/// document de 10 et dans un document de 100 000 nœuds, alors la dérivée par rapport à `n`
/// est nulle *par construction*, et aucune mesure de temps ne peut la démentir.
///
/// Pour comparaison, le mécanisme par snapshots donnait ici un rapport de 10 000.
#[test]
fn le_poids_d_un_geste_ne_depend_pas_de_la_taille_du_document() {
    fn weight_of_one_move(document_size: usize) -> usize {
        let mut p = project_with(document_size);
        let mut j = Journal::new(200);

        let before = p.boards[0].images[0].clone();
        let mut after = before.clone();
        after.x += 1.0;

        do_edit(
            &mut j,
            &mut p,
            Edit::Image {
                board: BOARD.to_string(),
                slot: Slot::changed(0, before, after),
            },
        );
        j.weight()
    }

    let petit = weight_of_one_move(10);
    let moyen = weight_of_one_move(10_000);
    let grand = weight_of_one_move(100_000);

    assert_eq!(petit, moyen);
    assert_eq!(
        moyen, grand,
        "∂T/∂n = 0 — déplacer une carte coûte pareil partout"
    );
    assert!(petit > 0, "et le geste pèse bien quelque chose");
}

/// Corollaire : le poids croît proportionnellement au nombre d'éléments touchés, et pas
/// autrement. C'est le `β·k` de la loi.
#[test]
fn le_poids_croit_lineairement_avec_la_taille_du_geste() {
    fn weight_of_k_moves(k: usize) -> usize {
        let mut p = project_with(1_000);
        let mut j = Journal::new(200);
        j.begin();
        for i in 0..k {
            let before = p.boards[0].images[i].clone();
            let mut after = before.clone();
            after.x += 1.0;
            do_edit(
                &mut j,
                &mut p,
                Edit::Image {
                    board: BOARD.to_string(),
                    slot: Slot::changed(i, before, after),
                },
            );
        }
        j.end();
        j.weight()
    }

    let w1 = weight_of_k_moves(1);
    let w10 = weight_of_k_moves(10);
    let w100 = weight_of_k_moves(100);

    assert_eq!(w10, w1 * 10);
    assert_eq!(w100, w1 * 100, "le coût est la taille du geste, exactement");
}

#[test]
fn les_annotations_suivent_la_meme_mecanique_que_les_images() {
    // Le mécanisme ne doit rien savoir du type qu'il transporte : si l'un marche, tous marchent.
    let mut p = Project::new("test");
    let mut j = Journal::new(200);

    let ann = Annotation::Text {
        id: "t1".to_string(),
        x: 0.0,
        y: 0.0,
        width: None,
        height: None,
        text: "bonjour".to_string(),
        font_size: None,
        color: None,
        cursor_pos: None,
        source_file: None,
        membrane_id: None,
        domains: Vec::new(),
        mirror_of: None,
        temporal_anchor: None,
    };

    do_edit(
        &mut j,
        &mut p,
        Edit::Annotation {
            board: BOARD.to_string(),
            slot: Slot::inserted(0, ann),
        },
    );
    assert_eq!(p.boards[0].annotations.len(), 1);

    assert!(j.undo(&mut p));
    assert!(p.boards[0].annotations.is_empty());
}

/// Le journal ne pèse que les gestes qu'il contient, jamais le document sous lui.
#[test]
fn un_geste_ne_pese_que_ce_qu_il_touche() {
    let mut p = project_with(50_000);
    let mut j = Journal::new(200);

    let before = p.boards[0].images[0].clone();
    let mut after = before.clone();
    after.x += 1.0;
    do_edit(
        &mut j,
        &mut p,
        Edit::Image { board: BOARD.to_string(), slot: Slot::changed(0, before, after) },
    );

    assert!(
        j.weight() < 4 * std::mem::size_of::<BoardImage>(),
        "un geste sur un document de 50 000 nœuds pèse deux images, pas 50 000"
    );
}

// ── Fiche 09 § 2.1 — « consommation mémoire bornée » ─────────────────────────────────

/// La borne de la spécification est en **gestes** (`UNDO_DEPTH`), pas en octets. Ce que le
/// mécanisme garantit vraiment — et ce que les clichés ne pouvaient pas tenir — c'est que le
/// poids d'une pile pleine ne dépend pas du document : 200 déplacements pèsent pareil dans
/// un canevas de 10 cartes et dans un de 100 000. Avec 200 clichés, le rapport était 10 000.
#[test]
fn test_une_pile_pleine_ne_pese_pas_la_taille_du_document() {
    fn weight_of_a_full_stack(n: usize) -> usize {
        let mut p = project_with(n);
        let mut j = Journal::new(UNDO_DEPTH);
        // Deux fois la profondeur : la moitié des gestes sort par le fond.
        for step in 0..(2 * UNDO_DEPTH) {
            let i = step % n;
            let before = p.boards[0].images[i].clone();
            let mut after = before.clone();
            after.x += 1.0;
            do_edit(
                &mut j,
                &mut p,
                Edit::Image {
                    board: BOARD.to_string(),
                    slot: Slot::changed(i, before, after),
                },
            );
        }
        assert_eq!(j.depth(), UNDO_DEPTH, "pleine, et pas plus");
        j.weight()
    }

    let petit = weight_of_a_full_stack(10);
    let grand = weight_of_a_full_stack(100_000);
    assert_eq!(petit, grand, "∂(poids de la pile)/∂n = 0");
    assert_eq!(
        petit,
        UNDO_DEPTH * 2 * std::mem::size_of::<BoardImage>(),
        "et c'est exactement 200 gestes de deux images chacun — rien d'autre"
    );
}

// ── Une édition qui ne change rien n'est pas une édition ─────────────────────────────

/// Valider un texte sans l'avoir modifié, ou un geste revenu à son point de départ, ne doit
/// ni consommer un niveau d'annulation, ni détruire ce qui restait à rétablir.
#[test]
fn test_une_edition_sans_effet_ne_laisse_aucune_trace() {
    let mut p = project_with(3);
    let mut j = Journal::new(UNDO_DEPTH);

    do_edit(&mut j, &mut p, remove(2, "img-2"));
    assert!(j.undo(&mut p));
    assert!(j.can_redo());

    // Une « modification » hors transaction, identique avant et après.
    let same = p.boards[0].images[0].clone();
    j.record(Edit::Image {
        board: BOARD.to_string(),
        slot: Slot::changed(0, same.clone(), same.clone()),
    });
    assert_eq!(j.depth(), 0, "rien à annuler");
    assert!(j.can_redo(), "et rien de perdu");

    // La même, dans un geste ouvert : la transaction reste vide et se referme sans trace.
    j.begin();
    j.record(Edit::Image {
        board: BOARD.to_string(),
        slot: Slot::changed(0, same.clone(), same),
    });
    j.record(Edit::BoardName {
        board: BOARD.to_string(),
        whole: Whole::new("main".to_string(), "main".to_string()),
    });
    j.end();
    assert_eq!(j.depth(), 0);
    assert!(j.can_redo());
    assert!(j.redo(&mut p), "le rétablissement marche toujours");
    assert_eq!(images(&p).len(), 2);
}
