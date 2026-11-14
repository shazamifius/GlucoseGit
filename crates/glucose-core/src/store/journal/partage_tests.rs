//! PARTAGE-1 : deux participants qui font chacun un geste, chez eux, tout de suite — puis la salle
//! les numérote ; chacun rejoue l'ordre de la salle (le « rebase » : défaire ses gestes en attente,
//! appliquer ceux d'avant, refaire les siens) et **tous finissent avec le même canevas**.

use super::super::edit::{Bouts, Edit};
use super::super::{Slot, Transaction, Whole};
use super::{GestePartage, Rencontre};
use crate::types::{Annotation, BoardImage, Project};

fn image(id: &str, x: f64) -> BoardImage {
    BoardImage::new(id, x, 0.0, 100.0, 100.0)
}

/// Le canevas de départ, le même chez tous : trois photos.
fn depart() -> Project {
    let mut p = Project::new("partage");
    p.boards[0].images = vec![image("a", 0.0), image("b", 1.0), image("c", 2.0)];
    p
}

fn geste(edits: Vec<Edit>) -> Transaction {
    Transaction { edits }
}

fn img(slot: Slot<BoardImage>) -> Edit {
    Edit::Image {
        board: "main".into(),
        slot,
    }
}

/// Un participant fait son geste chez lui : la loi du journal, par les rangs.
fn faire(p: &mut Project, g: &Transaction) -> GestePartage {
    for e in &g.edits {
        assert!(e.apply(p), "le geste s'applique chez son auteur");
    }
    g.a_partager(p)
}

/// **Le rebase** : il défait son geste en attente, applique celui que la salle a numéroté avant,
/// puis refait le sien — tout par le nom.
fn rebaser(p: &mut Project, le_mien: &GestePartage, l_autre: &GestePartage) -> Vec<Rencontre> {
    le_mien.inverse().appliquer(p);
    let mut r = l_autre.appliquer(p).rencontres;
    r.extend(le_mien.appliquer(p).rencontres);
    r
}

fn xs(p: &Project) -> Vec<(String, f64)> {
    p.boards[0]
        .images
        .iter()
        .map(|i| (i.id.clone(), i.x))
        .collect()
}

/// Alice déplace b, Bob retire a et pose d — des nœuds différents. Dans les deux ordres de la
/// salle, les deux canevas sont les mêmes, et rien ne se dit en conflit. **L'ancienne loi**, par
/// les rangs, aurait écrit le b d'Alice sur le c de Bob.
#[test]
fn partage_1_deux_participants_convergent_quand_ils_touchent_des_noeuds_differents() {
    let ga = geste(vec![img(Slot::changed(
        1,
        image("b", 1.0),
        image("b", 50.0),
    ))]);
    let gb = geste(vec![
        img(Slot::removed(0, image("a", 0.0))),
        img(Slot::inserted(2, image("d", 3.0))),
    ]);
    for alice_d_abord in [true, false] {
        let (mut alice, mut bob) = (depart(), depart());
        let pa = faire(&mut alice, &ga);
        let pb = faire(&mut bob, &gb);
        let (ra, rb) = if alice_d_abord {
            // La salle : Alice, puis Bob. Bob rebase ; Alice applique.
            (
                pb.appliquer(&mut alice).rencontres,
                rebaser(&mut bob, &pb, &pa),
            )
        } else {
            (
                rebaser(&mut alice, &pa, &pb),
                pa.appliquer(&mut bob).rencontres,
            )
        };
        assert_eq!(alice, bob, "ordre {alice_d_abord}");
        assert_eq!(
            xs(&alice),
            [("b".into(), 50.0), ("c".into(), 2.0), ("d".into(), 3.0)]
        );
        assert!(ra.is_empty() && rb.is_empty(), "{ra:?} {rb:?}");
    }
    // La preuve à l'envers : chez Bob, le geste d'Alice rejoué par son rang (1) tombe sur c.
    let mut bob = depart();
    faire(&mut bob, &gb);
    for e in &ga.edits {
        e.apply(&mut bob);
    }
    assert_eq!(
        xs(&bob),
        [("b".into(), 1.0), ("b".into(), 50.0), ("d".into(), 3.0)]
    );
}

/// Les deux touchent b : le dernier numéro gagne partout, et celui qui arrive après le dit.
#[test]
fn partage_1_le_dernier_gagne_et_le_conflit_se_dit() {
    let ga = geste(vec![img(Slot::changed(
        1,
        image("b", 1.0),
        image("b", 50.0),
    ))]);
    let gb = geste(vec![img(Slot::changed(
        1,
        image("b", 1.0),
        image("b", 70.0),
    ))]);
    let (mut alice, mut bob) = (depart(), depart());
    let pa = faire(&mut alice, &ga);
    let pb = faire(&mut bob, &gb);
    // La salle : Alice, puis Bob.
    let chez_alice = pb.appliquer(&mut alice).rencontres;
    let chez_bob = rebaser(&mut bob, &pb, &pa);
    assert_eq!(alice, bob);
    assert_eq!(xs(&alice)[1], ("b".into(), 70.0));
    assert_eq!(chez_alice, [Rencontre::Conflit("b".into())]);
    assert_eq!(chez_bob, [Rencontre::Conflit("b".into())]);
}

/// Alice retire b ; Bob le déplaçait : sa modification ne le ressuscite pas, chez personne.
#[test]
fn partage_1_une_modification_ne_ressuscite_pas_un_retire() {
    let ga = geste(vec![img(Slot::removed(1, image("b", 1.0)))]);
    let gb = geste(vec![img(Slot::changed(
        1,
        image("b", 1.0),
        image("b", 70.0),
    ))]);
    let (mut alice, mut bob) = (depart(), depart());
    let pa = faire(&mut alice, &ga);
    let pb = faire(&mut bob, &gb);
    let chez_alice = pb.appliquer(&mut alice).rencontres;
    rebaser(&mut bob, &pb, &pa);
    assert_eq!(alice, bob);
    assert_eq!(xs(&alice), [("a".into(), 0.0), ("c".into(), 2.0)]);
    assert_eq!(chez_alice, [Rencontre::Absent("b".into())]);
}

/// Un déplacement part **nommé** : chez Bob, qui a posé un texte en tête de liste, il emporte les
/// mêmes nœuds — la flèche dont un seul bout suivait, ce bout seulement.
#[test]
fn partage_1_un_deplacement_part_nomme() {
    let fleche = Annotation::arrow("f", 0.0, 0.0, 9.0, 9.0);
    let base = || {
        let mut p = depart();
        p.boards[0].annotations = vec![Annotation::text("t", 0.0, 0.0, "x"), fleche.clone()];
        p
    };
    let ga = geste(vec![Edit::Translation {
        board: "main".into(),
        delta: (10.0, 5.0),
        images: vec![2],
        annotations: vec![0],
        folders: vec![],
        bouts: vec![(
            1,
            Bouts {
                origine: true,
                cible: false,
            },
        )],
    }]);
    let gb = geste(vec![Edit::Annotation {
        board: "main".into(),
        slot: Slot::inserted(0, Annotation::text("neuf", 0.0, 0.0, "y")),
    }]);
    let (mut alice, mut bob) = (base(), base());
    let pa = faire(&mut alice, &ga);
    let pb = faire(&mut bob, &gb);
    rebaser(&mut alice, &pa, &pb);
    pa.appliquer(&mut bob);
    assert_eq!(alice, bob);
    let t = &bob.boards[0];
    assert_eq!(xs(&bob)[2], ("c".into(), 12.0));
    let neuf = t
        .annotations
        .iter()
        .find(|a| a.id() == "neuf")
        .expect("le texte de Bob");
    assert_eq!(neuf.x(), 0.0, "le texte de Bob n'a pas bougé");
    let Some(Annotation::Arrow { x, x2, .. }) = t.annotations.iter().find(|a| a.id() == "f") else {
        panic!("la flèche");
    };
    assert_eq!((*x, *x2), (10.0, 9.0));
}

/// L'onglet regardé est à chacun : il ne part pas.
#[test]
fn partage_1_l_onglet_regarde_ne_part_pas() {
    let g = geste(vec![
        Edit::ActiveBoard {
            whole: Whole::new("main".into(), "autre".into()),
        },
        img(Slot::changed(0, image("a", 0.0), image("a", 4.0))),
    ]);
    let mut p = depart();
    let partage = g.a_partager(&p);
    assert_eq!(partage.editions.len(), 1);
    partage.appliquer(&mut p);
    assert_eq!(p.active_board_id, "main");
}

/// L'ordre des onglets : Alice met « deux » en tête, Bob ajoute « trois ». L'ordre d'Alice range
/// ceux qu'il nomme ; celui de Bob, qu'elle ne connaissait pas, vient après. Et deux noms donnés
/// au même onglet : le dernier gagne, et le conflit se dit.
#[test]
fn partage_1_l_ordre_des_onglets_garde_ceux_qu_un_autre_a_ajoutes() {
    let base = || {
        let mut p = depart();
        p.boards.push(crate::types::Board::new("deux", "Deux"));
        p
    };
    let ga = geste(vec![
        Edit::BoardOrder {
            whole: Whole::new(
                vec!["main".into(), "deux".into()],
                vec!["deux".into(), "main".into()],
            ),
        },
        Edit::BoardName {
            board: "main".into(),
            whole: Whole::new("Canvas Principal".into(), "Alice".into()),
        },
    ]);
    let gb = geste(vec![
        Edit::Board {
            slot: Slot::inserted(2, crate::types::Board::new("trois", "Trois")),
        },
        Edit::BoardName {
            board: "main".into(),
            whole: Whole::new("Canvas Principal".into(), "Bob".into()),
        },
    ]);
    let (mut alice, mut bob) = (base(), base());
    let pa = faire(&mut alice, &ga);
    let pb = faire(&mut bob, &gb);
    // La salle : Bob, puis Alice.
    let chez_alice = rebaser(&mut alice, &pa, &pb);
    let chez_bob = pa.appliquer(&mut bob).rencontres;
    assert_eq!(alice, bob);
    let ordre: Vec<&str> = bob.boards.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(ordre, ["deux", "main", "trois"]);
    assert_eq!(bob.boards[1].name, "Alice");
    assert_eq!(chez_alice, [Rencontre::Conflit("main".into())]);
    assert_eq!(chez_bob, [Rencontre::Conflit("main".into())]);
}

/// **La transaction réalisée** : celle que le journal recevra. Rejouée par les rangs sur l'état
/// d'avant, elle redonne le même canevas ; défaite, elle rend exactement l'état d'avant — la
/// valeur de l'autre comprise quand il y a eu conflit (JRN-2 tient).
#[test]
fn partage_1_la_transaction_realisee_se_rejoue_et_se_defait_par_les_rangs() {
    // Alice déplace b et retire c : chez elle, la liste est plus courte que chez Bob.
    let ga = geste(vec![
        img(Slot::changed(1, image("b", 1.0), image("b", 50.0))),
        img(Slot::removed(2, image("c", 2.0))),
    ]);
    let gb = geste(vec![
        img(Slot::removed(0, image("a", 0.0))),
        img(Slot::inserted(2, image("d", 3.0))),
        img(Slot::changed(1, image("b", 1.0), image("b", 70.0))),
    ]);
    let (mut alice, mut bob) = (depart(), depart());
    faire(&mut alice, &ga);
    let pb = faire(&mut bob, &gb);
    let avant = alice.clone();
    let application = pb.appliquer(&mut alice);
    assert_eq!(application.rencontres, [Rencontre::Conflit("b".into())]);
    let mut rejoue = avant.clone();
    for e in &application.realisee.edits {
        assert!(e.apply(&mut rejoue), "les rangs réalisés sont justes");
    }
    assert_eq!(rejoue, alice);
    let mut defait = application.realisee.clone();
    defait.edits.reverse();
    for e in &mut defait.edits {
        e.flip();
        assert!(e.apply(&mut rejoue));
    }
    assert_eq!(
        rejoue, avant,
        "défaite, elle rend le b d'Alice, pas celui d'avant Alice"
    );
}
