//! CLIENT-1 et NOMMER-1 : deux Glucose — Alice et Bob — dans le même canevas, et une salle simulée
//! qui numérote leurs lettres sans les lire. Chacun fait ses gestes chez lui, tout de suite ; la
//! salle les ordonne ; chacun rejoue cet ordre — **et tous finissent avec le même canevas**.

use super::*;
use crate::store::journal::partage::{EditionPartagee, Rencontre};
use crate::store::journal::Edit;
use crate::types::{Annotation, BoardImage};

const T: &str = "main";

fn img(id: &str, x: f64) -> BoardImage {
    BoardImage::new(id, x, 0.0, 100.0, 100.0)
}

/// **La salle**, simulée : elle numérote les lettres dans l'ordre où elles arrivent.
#[derive(Default)]
struct Salle {
    lettres: Vec<(u64, Sortant)>,
}

impl Salle {
    /// Ce que ce Glucose a fait part (la boucle le fait à chaque image).
    fn poster(&mut self, qui: &mut Store) {
        qui.partager_les_gestes();
        for s in qui.prendre_les_sortants() {
            let n = self.lettres.len() as u64 + 1;
            self.lettres.push((n, s));
        }
    }

    /// Chacun reçoit tout ce que la salle garde ; il ignore ce qu'il a déjà appliqué.
    fn livrer(&self, qui: &mut Store) -> Vec<Rencontre> {
        for (n, s) in &self.lettres {
            let marque = s.id.rsplit_once('-').map_or("", |(m, _)| m).to_string();
            let main = match &s.main {
                Main::Agent(a) => Main::Agent(a.clone()),
                _ => Main::Participant {
                    nom: marque.to_uppercase(),
                    marque,
                },
            };
            let geste = match &s.envoi {
                Envoi::Geste(g) => g.clone(),
                Envoi::Base(_) => GestePartage::default(),
            };
            qui.recevoir(Arrivee {
                n: *n,
                id: s.id.clone(),
                main,
                geste,
            });
        }
        qui.appliquer_les_arrivees()
    }

    /// **Un participant entre** : la base (la première lettre), puis tout ce qui a suivi.
    fn rejoindre(&self, marque: &str, ecrivain: bool) -> Store {
        let (n, s) = &self.lettres[0];
        let Envoi::Base(p) = &s.envoi else {
            panic!("la base d'abord");
        };
        let mut st = Store::new("ailleurs");
        st.load_project((**p).clone());
        assert!(st.rejoindre(marque, ecrivain, *n));
        self.livrer(&mut st);
        st
    }
}

/// Alice partage un canevas de trois photos ; Bob le rejoint.
fn depart() -> (Salle, Store, Store) {
    let mut p = Project::new("partagé");
    p.boards[0].images = vec![img("a", 0.0), img("b", 1.0), img("c", 2.0)];
    let mut alice = Store::new("x");
    alice.load_project(p);
    assert!(alice.partager_ce_canevas("a7"));
    let mut salle = Salle::default();
    salle.poster(&mut alice);
    salle.livrer(&mut alice);
    let bob = salle.rejoindre("k3", true);
    (salle, alice, bob)
}

fn xs(s: &Store) -> Vec<(&str, f64)> {
    s.project.boards[0]
        .images
        .iter()
        .map(|i| (i.id.as_str(), i.x))
        .collect()
}

fn x_de(s: &Store, id: &str) -> Option<f64> {
    xs(s).into_iter().find(|(i, _)| *i == id).map(|(_, x)| x)
}

fn tous(salle: &Salle, qui: [&mut Store; 2]) {
    for s in qui {
        salle.livrer(s);
    }
}

/// **CLIENT-1** : Alice et Bob font chacun leurs gestes chez eux, en même temps, sur des nœuds
/// différents — un changement, un déplacement, un retrait. Dans les deux ordres de la salle, les
/// deux canevas sont les mêmes, et rien n'attend plus ; une salle qui répète ne rejoue rien.
#[test]
fn client_1_deux_participants_convergent_dans_l_ordre_de_la_salle() {
    for alice_d_abord in [true, false] {
        let (mut salle, mut alice, mut bob) = depart();
        alice.update_image(T, "b", |i| i.x = 50.0);
        bob.select_image("a".into(), false);
        bob.move_selected(T, 10.0, 0.0);
        bob.remove_images(T, &["c"]);
        if alice_d_abord {
            salle.poster(&mut alice);
            salle.poster(&mut bob);
        } else {
            salle.poster(&mut bob);
            salle.poster(&mut alice);
        }
        tous(&salle, [&mut alice, &mut bob]);
        assert_eq!(alice.project.boards, bob.project.boards, "{alice_d_abord}");
        assert_eq!(xs(&alice), [("a", 10.0), ("b", 50.0)]);
        let attente = |s: &Store| s.partage.as_ref().map(Participation::en_attente);
        assert_eq!((attente(&alice), attente(&bob)), (Some(0), Some(0)));
        // La salle répète tout : le déplacement ne se fait pas deux fois.
        tous(&salle, [&mut alice, &mut bob]);
        assert_eq!(xs(&bob), [("a", 10.0), ("b", 50.0)]);
    }
}

/// **CLIENT-1 — le dernier de la salle gagne partout, et le conflit se dit** : Alice et Bob
/// changent la même photo en même temps. Bob, numéroté après, l'emporte chez tous ; Alice voit que
/// Bob est passé sur son geste, Bob qu'il a recouvert celui d'Alice en rejouant le sien. **Sans
/// rejouer l'ordre de la salle**, Bob appliquerait le geste d'Alice par-dessus le sien, qu'elle ne
/// verrait jamais revenir : 50 chez lui, 70 chez elle.
#[test]
fn client_1_le_dernier_de_la_salle_gagne_partout_et_le_conflit_se_dit() {
    let (mut salle, mut alice, mut bob) = depart();
    alice.update_image(T, "b", |i| i.x = 50.0);
    bob.update_image(T, "b", |i| i.x = 70.0);
    salle.poster(&mut alice);
    salle.poster(&mut bob);
    let chez_alice = salle.livrer(&mut alice);
    let chez_bob = salle.livrer(&mut bob);
    assert_eq!(
        (x_de(&alice, "b"), x_de(&bob, "b")),
        (Some(70.0), Some(70.0))
    );
    assert_eq!(alice.project.boards, bob.project.boards);
    let conflit = Rencontre::Conflit("b".into());
    assert!(chez_alice.contains(&conflit), "{chez_alice:?}");
    assert!(chez_bob.contains(&conflit), "{chez_bob:?}");
}

/// **`Ctrl+Z` défait MON dernier geste**, pas le dernier du canevas (celui de Bob), et ce défaire
/// part comme un geste ; `Ctrl+Maj+Z` le refait, chez tous. Bob ne peut défaire que le sien.
#[test]
fn client_1_ctrl_z_defait_mon_dernier_geste_pas_celui_d_un_autre() {
    let (mut salle, mut alice, mut bob) = depart();
    alice.add_image(T, img("d", 3.0));
    salle.poster(&mut alice);
    tous(&salle, [&mut alice, &mut bob]);
    bob.update_image(T, "a", |i| i.x = 30.0);
    salle.poster(&mut bob);
    tous(&salle, [&mut alice, &mut bob]);
    assert_eq!((alice.undo_depth(), bob.undo_depth()), (1, 1));
    assert!(alice.undo(), "Alice défait sa photo");
    salle.poster(&mut alice);
    tous(&salle, [&mut alice, &mut bob]);
    for s in [&alice, &bob] {
        assert_eq!(
            xs(s),
            [("a", 30.0), ("b", 1.0), ("c", 2.0)],
            "le geste de Bob reste"
        );
    }
    assert!(alice.can_redo() && alice.redo());
    salle.poster(&mut alice);
    tous(&salle, [&mut alice, &mut bob]);
    assert_eq!(alice.project.boards, bob.project.boards);
    assert_eq!(x_de(&bob, "d"), Some(3.0), "rétablie chez Bob aussi");
}

/// **Rétablir rend ce que l'annulation a remplacé** (Figma) : Alice déplace b ; Bob le déplace
/// après elle ; Alice défait — b revient où elle l'avait pris ; elle rétablit — b revient où **Bob**
/// l'avait mis, l'état d'avant son annulation, et non le sien. Défaire puis tout rétablir ne change
/// rien.
#[test]
fn client_1_retablir_rend_ce_que_l_annulation_a_remplace() {
    let (mut salle, mut alice, mut bob) = depart();
    alice.update_image(T, "b", |i| i.x = 50.0);
    salle.poster(&mut alice);
    tous(&salle, [&mut alice, &mut bob]);
    bob.update_image(T, "b", |i| i.x = 70.0);
    salle.poster(&mut bob);
    tous(&salle, [&mut alice, &mut bob]);
    assert!(alice.undo());
    salle.poster(&mut alice);
    tous(&salle, [&mut alice, &mut bob]);
    assert_eq!((x_de(&alice, "b"), x_de(&bob, "b")), (Some(1.0), Some(1.0)));
    assert!(alice.redo());
    salle.poster(&mut alice);
    tous(&salle, [&mut alice, &mut bob]);
    assert_eq!(
        (x_de(&alice, "b"), x_de(&bob, "b")),
        (Some(70.0), Some(70.0))
    );
}

/// **Défaire un geste sur un nœud qu'un autre a retiré : rien** (Figma, Google Slides) — ni
/// résurrection, ni geste qui part.
#[test]
fn client_1_defaire_sur_un_noeud_retire_par_un_autre_ne_fait_rien() {
    let (mut salle, mut alice, mut bob) = depart();
    alice.update_image(T, "c", |i| i.x = 5.0);
    salle.poster(&mut alice);
    tous(&salle, [&mut alice, &mut bob]);
    bob.remove_images(T, &["c"]);
    salle.poster(&mut bob);
    tous(&salle, [&mut alice, &mut bob]);
    let version = alice.version;
    assert!(!alice.undo(), "c n'est plus là");
    assert!(alice.prendre_les_sortants().is_empty());
    assert_eq!((x_de(&alice, "c"), alice.version), (None, version));
}

/// **Rien n'arrive pendant un geste ouvert** : un glisser s'écrit par les rangs et doit pouvoir
/// s'abandonner ; le geste d'Alice attend que Bob lâche.
#[test]
fn client_1_un_geste_recu_attend_la_fin_d_un_geste_ouvert() {
    let (mut salle, mut alice, mut bob) = depart();
    alice.update_image(T, "b", |i| i.x = 50.0);
    salle.poster(&mut alice);
    bob.begin_live_edit();
    bob.update_image(T, "a", |i| i.x = 9.0);
    salle.livrer(&mut bob);
    assert_eq!(x_de(&bob, "b"), Some(1.0), "pas pendant le geste");
    bob.end_live_edit();
    bob.appliquer_les_arrivees();
    salle.poster(&mut bob);
    tous(&salle, [&mut alice, &mut bob]);
    assert_eq!(alice.project.boards, bob.project.boards);
    assert_eq!(xs(&alice), [("a", 9.0), ("b", 50.0), ("c", 2.0)]);
}

/// **NOMMER-1** : dans la même image, Alice déplace c (rang 2) puis retire a — c passe au rang 1,
/// et le rang 2 est désormais d. Le déplacement part nommé « c ». **Nommé dans l'état d'après les
/// deux gestes**, il aurait emporté d.
#[test]
fn nommer_1_un_deplacement_part_nomme_dans_l_etat_qui_le_suivait() {
    let (mut salle, mut alice, _bob) = depart();
    alice.add_image(T, img("d", 3.0));
    salle.poster(&mut alice);
    alice.select_image("c".into(), false);
    alice.move_selected(T, 5.0, 0.0);
    alice.remove_images(T, &["a"]);
    alice.partager_les_gestes();
    let sortants = alice.prendre_les_sortants();
    let Envoi::Geste(g) = &sortants[0].envoi else {
        panic!("un geste");
    };
    let EditionPartagee::Deplacement(d) = &g.editions[0] else {
        panic!("un déplacement");
    };
    assert_eq!(d.images, ["c"]);
    let par_les_rangs = Transaction {
        edits: vec![Edit::Translation {
            board: T.into(),
            delta: (5.0, 0.0),
            images: vec![2],
            annotations: vec![],
            folders: vec![],
            bouts: vec![],
        }],
    }
    .a_partager(&alice.project);
    let EditionPartagee::Deplacement(faux) = &par_les_rangs.editions[0] else {
        panic!("un déplacement");
    };
    assert_eq!(faux.images, ["d"], "l'ancienne loi nommait le voisin");
    assert_eq!(
        xs(&alice),
        [("b", 1.0), ("c", 7.0), ("d", 3.0)],
        "rien n'a bougé"
    );
}

/// **Qui signe un geste reçu** (MAIN-1) : l'humain d'Alice est un participant chez Bob — sa
/// flèche n'est pas celle d'un agent, l'IA de Glucose ne la jugera pas ; la flèche de l'agent
/// d'Alice l'est. Un geste reçu ne repart jamais.
#[test]
fn client_1_un_geste_recu_signe_de_son_auteur_et_ne_repart_pas() {
    let (mut salle, mut alice, mut bob) = depart();
    bob.journal.prendre_les_ecrits_signes();
    alice.add_annotation(T, Annotation::arrow("fl-a7-1", 0.0, 0.0, 9.0, 9.0));
    alice.journal.main = Main::Agent("claude".into());
    alice.add_annotation(T, Annotation::arrow("fl-a7-2", 0.0, 0.0, 5.0, 5.0));
    alice.journal.main = Main::Humain;
    salle.poster(&mut alice);
    salle.livrer(&mut bob);
    let ecrits = bob.ecrits_a_consigner();
    let mains: Vec<&Main> = ecrits.iter().map(|e| &e.main).collect();
    assert_eq!(
        mains,
        [
            &Main::Participant {
                marque: "a7".into(),
                nom: "A7".into()
            },
            &Main::Agent("claude".into())
        ]
    );
    assert!(!bob.journal.des_agents.contains("fl-a7-1"));
    assert!(bob.journal.des_agents.contains("fl-a7-2"));
    assert!(bob.prendre_les_sortants().is_empty(), "rien ne repart");
}

/// **Un lecteur regarde** : ce qu'il fait ne part pas et se défait aussitôt — il le sait.
#[test]
fn client_1_un_lecteur_ne_fait_que_regarder() {
    let (salle, _alice, _bob) = depart();
    let mut carol = salle.rejoindre("c9", false);
    carol.add_image(T, img("z", 9.0));
    carol.partager_les_gestes();
    assert!(carol.prendre_les_sortants().is_empty());
    assert_eq!(x_de(&carol, "z"), None);
    assert_eq!(
        carol.partage.as_ref().map(|p| p.defaits_au_lecteur),
        Some(1)
    );
}

/// **Quitter, ou ouvrir un autre document**, rend le canevas à soi seul : la pile repart du journal.
#[test]
fn client_1_un_autre_document_quitte_le_partage() {
    let (_salle, mut alice, _bob) = depart();
    alice.load_project(Project::new("autre"));
    assert!(alice.partage.is_none() && alice.participant.is_none());
    alice.add_image(T, img("e", 4.0));
    assert!(
        alice.undo() && x_de(&alice, "e").is_none(),
        "le journal reprend"
    );
}

/// **SELECTION-1** : la photo qu'Alice avait choisie, Bob la retire — elle sort de sa sélection ;
/// celle qu'Alice vient de poser et de choisir, encore en attente, est retirée puis remise par le
/// rebase — elle reste choisie (le résultat net du geste). Et seul, `Ctrl+Z` d'une pose retire ce
/// qu'on avait choisi.
#[test]
fn selection_1_ce_qu_un_geste_retire_sort_de_la_selection() {
    let (mut salle, mut alice, mut bob) = depart();
    alice.select_image("a".into(), false);
    bob.remove_images(T, &["a"]);
    salle.poster(&mut bob);
    alice.add_image(T, img("d", 3.0));
    alice.select_image("d".into(), true);
    salle.livrer(&mut alice);
    assert!(!alice.selected_image_ids.contains("a"), "retirée par Bob");
    assert!(
        alice.selected_image_ids.contains("d"),
        "retirée puis remise : choisie"
    );
    let mut seul = Store::new("seul");
    seul.add_image(T, img("e", 0.0));
    seul.select_image("e".into(), false);
    assert!(seul.undo());
    assert!(!seul.selected_image_ids.contains("e"));
}
