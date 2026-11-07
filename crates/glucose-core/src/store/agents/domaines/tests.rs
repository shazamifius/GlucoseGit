use super::super::{Refus, Store};
use super::*;
use crate::types::{Annotation, BoardImage};

fn couleur(rang: usize) -> String {
    format!("#00000{rang}")
}

const NAISSANCE: Naissance = Naissance {
    couleur,
    instant: 42,
};

/// **Le sigle vient du nom** : « Eau » ne se montre pas « SCI ».
#[test]
fn le_sigle_vient_du_nom() {
    for (nom, sigle) in [
        ("Eau", "EAU"),
        ("Jeu vidéo", "JEU"),
        ("python code", "PYT"),
        ("été", "ÉTÉ"),
        ("C++", "C"),
        (" ++ ", "++"),
    ] {
        assert_eq!(sigle_de(nom), sigle, "{nom}");
    }
}

/// Deux textes et une photo ; le journal vide : chaque épreuve compte ses propres gestes.
fn tableau() -> (Store, String) {
    let mut store = Store::new("agents");
    let board = store.project.active_board_id.clone();
    store.add_annotation(
        &board,
        Annotation::text("a", 0.0, 0.0, "L'eau, ça mouille."),
    );
    store.add_annotation(&board, Annotation::text("b", 0.0, 100.0, "Eau = H2O."));
    store.add_image(&board, BoardImage::new("p", 0.0, 200.0, 50.0, 50.0));
    store.journal.clear();
    (store, board)
}

fn poser(
    store: &mut Store,
    board: &str,
    ids: &[&str],
    voulu: Voulu<'_>,
    poids: f64,
) -> Result<(Option<String>, bool), Refus> {
    store.geste_d_agent(|s| s.poser_un_domaine_pour_un_agent(board, ids, (voulu, poids), NAISSANCE))
}

fn nouveau(store: &mut Store, board: &str, ids: &[&str], nom: &str, poids: f64) -> Domain {
    let (id, ne) = poser(store, board, ids, Voulu::Nouveau(nom), poids).expect("créé");
    assert!(ne);
    store
        .domain(&id.expect("son identifiant"))
        .expect("au catalogue")
        .clone()
}

fn poids(store: &Store, board: &str, noeud: &str) -> Vec<(String, f64)> {
    store
        .node_domains(board, noeud)
        .expect("le nœud")
        .iter()
        .map(|a| (a.domain_id.clone(), a.weight))
        .collect()
}

/// **Un geste, pour tout l'appel** (AGENT-1) : un seul `Ctrl+Z` défait le domaine neuf et chacune de
/// ses assignations, une photo comprise.
#[test]
fn creer_et_poser_est_un_seul_geste() {
    let (mut store, board) = tableau();
    let d = nouveau(&mut store, &board, &["a", "b", "p"], "Eau", 0.8);
    assert_eq!(store.undo_depth(), 1, "un seul geste");
    assert_eq!(
        (d.name.as_str(), d.color.as_str(), d.icon.as_str()),
        ("Eau", "#000000", "EAU")
    );
    assert_eq!(d.defini_a, 42, "l'heure vient de l'appelant");
    assert_eq!(poids(&store, &board, "p"), [(d.id.clone(), 0.8)]);
    store.undo();
    assert!(store.project.domains.is_empty(), "le domaine est défait");
    assert!(
        poids(&store, &board, "a").is_empty(),
        "son assignation aussi"
    );
    assert!(poids(&store, &board, "p").is_empty(), "et sur la photo");
}

/// **AGENT-5, sa règle** : un nom pris ne se recrée pas ; rien ne bouge, et le refus demande de
/// préciser.
#[test]
fn creer_un_nom_pris_est_refuse_et_demande_de_preciser() {
    let (mut store, board) = tableau();
    nouveau(&mut store, &board, &["a"], "Python", 1.0);
    let profondeur = store.undo_depth();
    let Err(Refus::Domaine(question)) =
        poser(&mut store, &board, &["b"], Voulu::Nouveau(" python "), 0.5)
    else {
        panic!("le nom est pris : il devait être refusé");
    };
    assert!(
        question.contains("Python") && question.contains("précise"),
        "{question}"
    );
    assert_eq!(store.project.domains.len(), 1);
    assert!(poids(&store, &board, "b").is_empty(), "rien posé sur b");
    assert_eq!(store.undo_depth(), profondeur, "aucun geste");
    nouveau(&mut store, &board, &["b"], "Python serpent", 0.5);
}

/// **Prendre un domaine qui existe** : le poser encore ne double rien, et un appel qui ne change rien
/// ne laisse aucun geste à défaire ; un autre poids l'écrase.
#[test]
fn prendre_un_domaine_existant_ne_double_rien() {
    let (mut store, board) = tableau();
    let d = nouveau(&mut store, &board, &["a"], "Eau", 0.5);
    let profondeur = store.undo_depth();
    let (encore, ne) = poser(&mut store, &board, &["a"], Voulu::Existant(&d), 0.5).expect("repris");
    assert_eq!((encore.as_deref(), ne), (Some(d.id.as_str()), false));
    assert_eq!(store.project.domains.len(), 1);
    assert_eq!(
        store.undo_depth(),
        profondeur,
        "rien n'a changé : aucun geste"
    );
    poser(&mut store, &board, &["a"], Voulu::Existant(&d), 0.9).expect("un autre poids");
    assert_eq!(poids(&store, &board, "a"), [(d.id, 0.9)]);
    assert_eq!(store.undo_depth(), profondeur + 1);
}

/// **Traverser les fichiers** : un domaine connu d'un autre fichier entre au catalogue de celui-ci, et
/// le tout s'annule d'un seul geste.
#[test]
fn prendre_un_domaine_d_un_autre_fichier_l_adopte() {
    let (mut store, board) = tableau();
    let venu = Domain {
        id: "eau@7".into(),
        name: "Eau".into(),
        color: "#123456".into(),
        icon: "H2O".into(),
        defini_a: 7,
    };
    poser(&mut store, &board, &["a"], Voulu::Existant(&venu), 1.0).expect("adopté");
    assert_eq!(
        store.project.domains,
        std::slice::from_ref(&venu),
        "sa définition entière"
    );
    assert_eq!(poids(&store, &board, "a"), [("eau@7".to_string(), 1.0)]);
    assert_eq!(store.undo_depth(), 1);
    store.undo();
    assert!(store.project.domains.is_empty());
    assert!(poids(&store, &board, "a").is_empty());
}

/// **Rien à moitié** (AGENT-1) : un nœud inconnu au milieu de la liste, et rien ne reste — ni le
/// domaine neuf, ni les assignations déjà faites.
#[test]
fn un_noeud_inconnu_ne_laisse_rien() {
    let (mut store, board) = tableau();
    let refus = poser(
        &mut store,
        &board,
        &["a", "fantome"],
        Voulu::Nouveau("Eau"),
        0.5,
    );
    assert_eq!(refus, Err(Refus::NoeudInconnu("fantome".into())));
    assert!(store.project.domains.is_empty(), "pas de domaine neuf");
    assert!(poids(&store, &board, "a").is_empty(), "pas d'assignation");
    assert_eq!(store.undo_depth(), 0, "pas de geste");
}

#[test]
fn un_poids_hors_bornes_ne_laisse_rien_et_dit_pourquoi() {
    let (mut store, board) = tableau();
    for faux in [1.5, -0.1, f64::NAN, f64::INFINITY] {
        let Err(Refus::Domaine(pourquoi)) =
            poser(&mut store, &board, &["a"], Voulu::Nouveau("Eau"), faux)
        else {
            panic!("{faux} devait être refusé");
        };
        assert!(!pourquoi.is_empty());
        assert!(store.project.domains.is_empty(), "{faux} : rien ne reste");
    }
}

#[test]
fn un_tableau_inconnu_est_refuse_par_son_nom() {
    let (mut store, _) = tableau();
    let refus = poser(&mut store, "ailleurs", &["a"], Voulu::Nouveau("Eau"), 0.5);
    assert_eq!(refus, Err(Refus::TableauInconnu("ailleurs".into())));
}

#[test]
fn un_nom_vide_ne_cree_pas_de_domaine() {
    let (mut store, board) = tableau();
    let refus = poser(&mut store, &board, &["a"], Voulu::Nouveau("   "), 0.5);
    assert!(matches!(refus, Err(Refus::Domaine(_))), "{refus:?}");
    assert!(store.project.domains.is_empty());
}

/// **AGENT-6** : un poids nul retire, sans erreur pour le nœud qui ne le portait pas, et un second
/// retrait ne laisse aucun geste. Le catalogue garde le domaine.
#[test]
fn un_poids_nul_retire() {
    let (mut store, board) = tableau();
    let d = nouveau(&mut store, &board, &["a"], "Eau", 0.5);
    poser(&mut store, &board, &["a", "b"], Voulu::Existant(&d), 0.0).expect("retiré");
    assert!(poids(&store, &board, "a").is_empty());
    assert!(poids(&store, &board, "b").is_empty(), "b ne le portait pas");
    assert!(store.domain(&d.id).is_some(), "le catalogue le garde");
    let profondeur = store.undo_depth();
    poser(&mut store, &board, &["a"], Voulu::Existant(&d), 0.0).expect("déjà retiré");
    assert_eq!(store.undo_depth(), profondeur, "aucun geste de plus");
    store.undo();
    assert_eq!(
        poids(&store, &board, "a"),
        [(d.id, 0.5)],
        "un Ctrl+Z le rend"
    );
}

/// **AGENT-6** : retirer un domaine que ce document n'a pas ne le fait pas entrer.
#[test]
fn un_poids_nul_ne_fait_rien_entrer() {
    let (mut store, board) = tableau();
    let ailleurs = Domain {
        id: "eau@7".into(),
        name: "Eau".into(),
        color: "#123456".into(),
        icon: "H2O".into(),
        defini_a: 7,
    };
    let issue = poser(&mut store, &board, &["a"], Voulu::Existant(&ailleurs), 0.0).expect("rien");
    assert_eq!(issue, (None, false));
    assert!(store.project.domains.is_empty());
    let issue = poser(&mut store, &board, &["a"], Voulu::Nouveau("Feu"), 0.0).expect("rien");
    assert_eq!(issue, (None, false));
    assert!(store.project.domains.is_empty());
    assert_eq!(store.undo_depth(), 0);
}

/// **AGENT-2** : poser un domaine ne touche pas à ce que l'humain a sélectionné.
#[test]
fn poser_un_domaine_garde_sa_selection() {
    let (mut store, board) = tableau();
    store.select_annotation("b".to_string(), false);
    nouveau(&mut store, &board, &["a"], "Eau", 0.5);
    assert_eq!(store.selected_annotation_ids, ["b"]);
}

/// **Lu comme il est vu** : chaque nœud dit ses domaines, leur nom et leur poids, dans l'ordre du
/// catalogue (DOM-2), la photo comme le texte ; un nœud sans domaine n'en dit aucun.
#[test]
fn decrire_dit_les_domaines_de_chaque_noeud() {
    let (mut store, board) = tableau();
    let eau = nouveau(&mut store, &board, &["p"], "Eau", 0.5);
    let jeu = nouveau(&mut store, &board, &["a"], "Jeu vidéo", 0.3);
    poser(&mut store, &board, &["a"], Voulu::Existant(&eau), 0.8).expect("posé");
    let decrits = store.decrire_pour_un_agent(&board).expect("lu");
    let de = |id: &str| decrits.iter().find(|d| d.id == id).expect("décrit");
    let attendu = |d: &Domain, p| DomaineDecrit {
        id: d.id.clone(),
        nom: d.name.clone(),
        poids: p,
    };
    assert_eq!(de("a").domaines, [attendu(&eau, 0.8), attendu(&jeu, 0.3)]);
    assert_eq!(de("p").domaines, [attendu(&eau, 0.5)]);
    assert!(de("b").domaines.is_empty());
}

/// **Reposer ne jette rien** : quand un agent pose un identifiant qui existe (AGENT-4), les domaines
/// que l'humain y a mis restent. Le nœud entier était remplacé, et ils disparaissaient sans un mot.
/// La preuve à l'envers : le nœud posé, lui, n'en porte aucun.
#[test]
fn reposer_un_texte_garde_ses_domaines() {
    let (mut store, board) = tableau();
    let d = nouveau(&mut store, &board, &["a"], "Eau", 0.8);
    let neuf = Annotation::text("a", 0.0, 0.0, "L'eau mouille encore.");
    assert!(neuf.domains().is_empty(), "ce qu'il pose n'en dit rien");
    store
        .geste_d_agent(|s| s.poser_pour_un_agent(&board, neuf))
        .expect("reposé");
    let reposee = store.project.annotation(&board, "a").expect("a");
    assert_eq!(reposee.own_text().as_deref(), Some("L'eau mouille encore."));
    assert_eq!(poids(&store, &board, "a"), [(d.id, 0.8)]);
}

#[test]
fn reposer_une_photo_garde_ses_domaines() {
    let (mut store, board) = tableau();
    let d = nouveau(&mut store, &board, &["p"], "Eau", 0.8);
    let neuve = BoardImage::new("p", 10.0, 200.0, 50.0, 50.0);
    assert!(neuve.domains.is_empty(), "ce qu'il pose n'en dit rien");
    store
        .geste_d_agent(|s| s.poser_une_photo_pour_un_agent(&board, neuve))
        .expect("reposée");
    assert_eq!(poids(&store, &board, "p"), [(d.id, 0.8)]);
}
