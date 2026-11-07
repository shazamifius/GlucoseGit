use super::*;
use crate::store::{DomainPatch, Store};
use crate::types::{Annotation, BoardImage};

fn allure() -> (String, String) {
    ("#60a5fa".to_string(), "SCI".to_string())
}

/// Un document aux nœuds nommés.
fn document(noeuds: &[&str]) -> (Store, String) {
    let mut store = Store::new("P");
    let board = store.project.active_board_id.clone();
    for n in noeuds {
        store.add_annotation(
            &board,
            Annotation::text(*n, 0.0, 0.0, format!("Texte de {n}.")),
        );
    }
    (store, board)
}

fn porter(store: &mut Store, board: &str, noeud: &str, domaine: &str, poids: f64) {
    store
        .try_assign_domain_to_node(board, noeud, domaine, poids)
        .expect("assigné");
}

/// **Ce que sa vision demande** : trois blocs, deux fichiers, un seul domaine ; il sait où chacun vit.
#[test]
fn un_domaine_relie_les_noeuds_de_deux_fichiers() {
    let (mut a, tableau_a) = document(&["mouille", "h2o", "sans"]);
    let eau = a.creer_un_domaine("Eau", allure(), 10).unwrap();
    porter(&mut a, &tableau_a, "mouille", &eau, 1.0);
    porter(&mut a, &tableau_a, "h2o", &eau, 0.5);

    let (mut b, tableau_b) = document(&["jeu"]);
    let definition = a.domain(&eau).unwrap().clone();
    let ici = b.adopter_un_domaine(&definition).unwrap();
    porter(&mut b, &tableau_b, "jeu", &ici, 0.8);

    let mut r = Recensement::default();
    r.poser(recenser("/a.glucose", &a.project));
    r.poser(recenser("/b.glucose", &b.project));

    let vue = r.vue(None);
    assert_eq!(vue.len(), 1, "un seul domaine : {vue:?}");
    assert_eq!(vue[0].noeuds(), 3);
    assert_eq!(
        vue[0].fichiers,
        [
            DansUnFichier {
                chemin: "/a.glucose".into(),
                noeuds: 2
            },
            DansUnFichier {
                chemin: "/b.glucose".into(),
                noeuds: 1
            },
        ]
    );
    let trouves = r.noeuds_du_domaine(None, &vue[0].lignee);
    let places: Vec<_> = trouves
        .iter()
        .map(|t| (t.chemin, t.decrit.id.as_str(), t.poids))
        .collect();
    assert_eq!(
        places,
        [
            ("/a.glucose", "mouille", 1.0),
            ("/b.glucose", "jeu", 0.8),
            ("/a.glucose", "h2o", 0.5),
        ],
        "du plus lourd au plus léger"
    );
    assert_eq!(trouves[0].tableau, tableau_a);
    assert_eq!(
        trouves[0].decrit.texte.as_deref(),
        Some("Texte de mouille.")
    );
}

/// **RECENSE-1, sa règle** : deux créations séparées du même nom ne se réunissent pas en silence.
/// La preuve à l'envers : le même domaine adopté, lui, n'en fait qu'un.
#[test]
fn deux_creations_du_meme_nom_sont_deux_homonymes() {
    let (mut a, _) = document(&[]);
    a.creer_un_domaine("Python", allure(), 1).unwrap();
    let (mut b, _) = document(&[]);
    b.creer_un_domaine("python", allure(), 2).unwrap();
    let mut r = Recensement::default();
    r.poser(recenser("/a", &a.project));
    r.poser(recenser("/b", &b.project));
    let vue = r.vue(None);
    assert_eq!(vue.len(), 2, "deux lignées");
    assert_eq!(homonymes(&vue, " PYTHON ").len(), 2);
    assert_eq!(
        homonymes(&vue, "serpent").len(),
        0,
        "le mot précisé est libre"
    );
    assert_eq!(homonymes(&vue, "  ").len(), 0);

    let mut c = document(&[]).0;
    c.adopter_un_domaine(&a.project.domains[0]).unwrap();
    r.oublier("/b");
    r.poser(recenser("/c", &c.project));
    let vue = r.vue(None);
    assert_eq!(homonymes(&vue, "python").len(), 1, "adopté : le même");
    assert_eq!(vue[0].fichiers.len(), 2);
}

/// **La définition la plus récente l'emporte** : renommer dans un fichier se lit partout, quel que
/// soit l'ordre des fichiers ; l'identité ne bouge pas.
#[test]
fn la_definition_la_plus_recente_l_emporte() {
    let (mut a, _) = document(&[]);
    let id = a.creer_un_domaine("Python", allure(), 1).unwrap();
    let mut b = document(&[]).0;
    b.adopter_un_domaine(&a.project.domains[0]).unwrap();
    let recente = DomainPatch::new().with_name("Python code").at(5);
    a.try_update_domain(&id, recente).unwrap();

    for (premier, second) in [(&a, &b), (&b, &a)] {
        let mut r = Recensement::default();
        r.poser(recenser("/1", &premier.project));
        r.poser(recenser("/2", &second.project));
        let vue = r.vue(None);
        assert_eq!(vue.len(), 1);
        assert_eq!(vue[0].definition.name, "Python code");
        assert_eq!(vue[0].lignee, "python@1");
        assert!(
            homonymes(&vue, "python").is_empty(),
            "l'ancien nom est libéré"
        );
    }
}

/// **Le document ouvert passe devant ce qu'on avait lu de lui** : il est vivant.
#[test]
fn le_document_courant_remplace_ce_qu_on_en_avait_lu() {
    let (mut a, tableau) = document(&["x"]);
    let eau = a.creer_un_domaine("Eau", allure(), 1).unwrap();
    let mut r = Recensement::default();
    r.poser(recenser("/a", &a.project));
    porter(&mut a, &tableau, "x", &eau, 1.0);
    let courant = recenser("/a", &a.project);
    assert_eq!(r.vue(None)[0].noeuds(), 0, "ce qui a été lu");
    assert_eq!(r.vue(Some(&courant))[0].noeuds(), 1, "ce qui est");
    assert_eq!(r.vue(Some(&courant))[0].fichiers.len(), 1, "pas de double");
    assert_eq!(r.noeuds_du_domaine(Some(&courant), "eau@1").len(), 1);
}

/// **Un fichier disparu ne se compte plus**, et un domaine qu'il portait seul disparaît avec lui.
#[test]
fn oublier_un_fichier_retire_ce_qu_il_portait() {
    let (mut a, _) = document(&[]);
    a.creer_un_domaine("Eau", allure(), 1).unwrap();
    let mut r = Recensement::default();
    r.poser(recenser("/a", &a.project));
    assert_eq!(r.vue(None).len(), 1);
    r.oublier("/a");
    assert!(r.vue(None).is_empty());
}

/// **Les photos comptent, et les nœuds sans domaine ne pèsent rien.**
#[test]
fn seuls_les_noeuds_qui_portent_un_domaine_sont_gardes() {
    let (mut a, tableau) = document(&["sans", "avec"]);
    a.add_image(&tableau, BoardImage::new("photo", 0.0, 0.0, 10.0, 10.0));
    let eau = a.creer_un_domaine("Eau", allure(), 1).unwrap();
    porter(&mut a, &tableau, "avec", &eau, 1.0);
    porter(&mut a, &tableau, "photo", &eau, 1.0);
    let d = recenser("/a", &a.project);
    let mut ids: Vec<_> = d.noeuds.iter().map(|n| n.decrit.id.as_str()).collect();
    ids.sort_unstable();
    assert_eq!(ids, ["avec", "photo"]);
}

/// **Deux domaines d'avant** : le `domain-3` de chacun est un autre sujet, même numéro ou non. Sous
/// un même nom et à des dates différentes, ce sont deux homonymes ; la copie d'un fichier, elle, a
/// la même date, et reste le même domaine.
#[test]
fn les_numeros_d_avant_ne_se_confondent_pas() {
    let domaine = |nom: &str, date| Domain {
        id: "domain-3".into(),
        name: nom.into(),
        color: "#000000".into(),
        icon: "S".into(),
        defini_a: date,
    };
    let avec = |d: Domain| {
        let mut s = Store::new("P");
        s.project.domains.push(d);
        s
    };
    let mut r = Recensement::default();
    r.poser(recenser("/feu", &avec(domaine("Feu", 1)).project));
    r.poser(recenser("/eau", &avec(domaine("Eau", 2)).project));
    assert_eq!(r.vue(None).len(), 2, "le même numéro, deux sujets");
    r.poser(recenser("/copie", &avec(domaine("Eau", 2)).project));
    assert_eq!(r.vue(None).len(), 2, "la copie est le même");
    r.poser(recenser("/autre", &avec(domaine("eau", 9)).project));
    assert_eq!(
        homonymes(&r.vue(None), "EAU").len(),
        2,
        "une autre date : on demande"
    );
}
