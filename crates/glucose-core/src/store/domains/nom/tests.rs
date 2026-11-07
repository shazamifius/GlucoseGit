use super::*;
use crate::types::Annotation;

fn allure() -> (String, String) {
    ("#60a5fa".to_string(), "SCI".to_string())
}

fn domaine(id: &str, nom: &str, defini_a: i64) -> Domain {
    Domain {
        id: id.into(),
        name: nom.into(),
        color: "#60a5fa".into(),
        icon: "SCI".into(),
        defini_a,
    }
}

#[test]
fn la_casse_et_les_blancs_ne_comptent_pas_les_accents_si() {
    assert_eq!(cle_de_domaine("  L'EAU \t pure\n"), "l'eau pure");
    assert_eq!(cle_de_domaine("ÉTÉ"), "été");
    assert_ne!(cle_de_domaine("pêche"), cle_de_domaine("péché"));
    assert_eq!(cle_de_domaine(" \n "), "");
}

/// **DOM-4, sa règle** : le même nom, écrit autrement, est refusé, et le refus demande de préciser.
/// Le mot précisé passe : c'est son exemple même.
#[test]
fn dom_4_on_n_inscrit_pas_deux_domaines_du_meme_nom() {
    let mut store = Store::new("P");
    store
        .creer_un_domaine("Python", allure(), 1)
        .expect("le premier");
    for meme in ["Python", "python", "  PYTHON  "] {
        let refus = store.creer_un_domaine(meme, allure(), 2);
        assert_eq!(refus, Err(CoreError::NomDeDomainePris("Python".into())));
    }
    let dit = CoreError::NomDeDomainePris("Python".into()).to_string();
    assert!(
        dit.contains("même sujet") && dit.contains("précise"),
        "{dit}"
    );
    assert_eq!(store.project.domains.len(), 1, "rien de plus au catalogue");
    store
        .creer_un_domaine("Python serpent", allure(), 3)
        .expect("le mot précisé");
    assert_eq!(store.project.domains.len(), 2);
}

#[test]
fn dom_4_un_nom_vide_est_refuse() {
    let mut store = Store::new("P");
    assert_eq!(
        store.creer_un_domaine("  \t ", allure(), 1),
        Err(CoreError::NomDeDomaineVide)
    );
    assert!(store.project.domains.is_empty());
}

/// **DOM-4 tient aussi au renommage** : on ne contourne pas la règle en créant un autre nom. Changer
/// seulement la casse de son propre nom reste permis.
#[test]
fn dom_4_on_ne_renomme_pas_vers_un_nom_pris() {
    let mut store = Store::new("P");
    let eau = store.creer_un_domaine("Eau", allure(), 1).unwrap();
    store.creer_un_domaine("Feu", allure(), 2).unwrap();
    let refus = store.try_update_domain(&eau, DomainPatchNom::vers("feu"));
    assert_eq!(refus, Err(CoreError::NomDeDomainePris("Feu".into())));
    assert_eq!(store.domain(&eau).unwrap().name, "Eau");
    store
        .try_update_domain(&eau, DomainPatchNom::vers("EAU"))
        .expect("sa propre casse");
    assert_eq!(store.domain(&eau).unwrap().name, "EAU");
    assert_eq!(
        store.try_update_domain(&eau, DomainPatchNom::vers("  ")),
        Err(CoreError::NomDeDomaineVide)
    );
}

struct DomainPatchNom;

impl DomainPatchNom {
    fn vers(nom: &str) -> crate::store::DomainPatch {
        crate::store::DomainPatch::new().with_name(nom)
    }
}

/// **DOM-5** : renommer ne change ni l'identité, ni ce qui la porte ; et la date dit quelle
/// définition est la plus récente.
#[test]
fn dom_5_le_nom_change_l_identite_reste() {
    let mut store = Store::new("P");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("a", 0.0, 0.0, "a"));
    let id = store.creer_un_domaine("Python", allure(), 10).unwrap();
    assert_eq!(id, "python@10", "son premier nom et sa naissance");
    store
        .try_assign_domain_to_node(&board, "a", &id, 0.5)
        .unwrap();
    store
        .try_update_domain(&id, DomainPatchNom::vers("Python code").at(20))
        .unwrap();
    let d = store.domain(&id).expect("la même identité");
    assert_eq!((d.name.as_str(), d.defini_a), ("Python code", 20));
    assert_eq!(lignee_de(d), "python@10");
    assert_eq!(store.node_domains(&board, "a").unwrap()[0].domain_id, id);
}

/// **Une date seule n'est pas un changement** (DOM-3) : aucun geste à défaire.
#[test]
fn une_date_seule_ne_laisse_aucun_geste() {
    let mut store = Store::new("P");
    let id = store.creer_un_domaine("Eau", allure(), 1).unwrap();
    let profondeur = store.undo_depth();
    store
        .try_update_domain(&id, crate::store::DomainPatch::new().at(99))
        .unwrap();
    assert_eq!(store.undo_depth(), profondeur);
    assert_eq!(store.domain(&id).unwrap().defini_a, 1);
}

/// **Les domaines d'avant** : `domain-3` ne veut rien dire d'un document à l'autre. Leur identité se
/// lit dans leur nom et leur date ; un identifiant qui n'en a que l'allure reste le sien.
#[test]
fn la_lignee_d_un_domaine_d_avant_vient_de_son_nom_et_de_sa_date() {
    assert_eq!(lignee_de(&domaine("domain-3", "Eau", 7)), "eau@7");
    assert_eq!(lignee_de(&domaine("domain-9", "EAU", 7)), "eau@7");
    assert_ne!(
        lignee_de(&domaine("domain-3", "Feu", 7)),
        lignee_de(&domaine("domain-3", "Eau", 7))
    );
    assert_eq!(lignee_de(&domaine("eau@5", "Eau", 7)), "eau@5");
    assert_eq!(lignee_de(&domaine("domain-", "Eau", 7)), "domain-");
    assert_eq!(lignee_de(&domaine("domain-x", "Eau", 7)), "domain-x");
}

#[test]
fn un_domaine_se_trouve_par_son_identifiant_puis_par_son_nom() {
    let mut store = Store::new("P");
    store.try_add_domain(domaine("d-1", "Eau", 0)).unwrap();
    store.try_add_domain(domaine("eau", "Autre", 0)).unwrap();
    // L'identifiant passe avant le nom : il ne désigne qu'un seul domaine.
    assert_eq!(store.trouver_un_domaine("eau").unwrap().name, "Autre");
    assert_eq!(store.trouver_un_domaine(" EAU ").unwrap().id, "d-1");
    assert_eq!(store.trouver_un_domaine("d-1").unwrap().name, "Eau");
    assert!(store.trouver_un_domaine("feu").is_none());
}

/// Un document d'avant la règle peut porter un domaine sans nom : rien ne doit le désigner.
#[test]
fn un_nom_vide_ne_designe_rien() {
    let mut store = Store::new("P");
    store.project.domains.push(domaine("d", "   ", 0));
    assert!(store.trouver_un_domaine("").is_none());
    assert!(store.trouver_un_domaine("  ").is_none());
    assert!(
        store.trouver_un_domaine("d").is_some(),
        "son identifiant, si"
    );
}

#[test]
fn le_compte_suit_l_ordre_du_catalogue_sur_tous_les_tableaux() {
    let mut store = Store::new("P");
    let premier = store.project.active_board_id.clone();
    let second = store.add_board("Second");
    store.add_annotation(&premier, Annotation::text("a", 0.0, 0.0, "a"));
    store.add_annotation(&second, Annotation::text("b", 0.0, 0.0, "b"));
    for (id, nom) in [("x", "X"), ("y", "Y"), ("z", "Z")] {
        store.try_add_domain(domaine(id, nom, 0)).unwrap();
    }
    for (board, noeud, d) in [
        (&premier, "a", "y"),
        (&second, "b", "y"),
        (&second, "b", "z"),
    ] {
        store
            .try_assign_domain_to_node(board, noeud, d, 1.0)
            .unwrap();
    }
    assert_eq!(store.porteurs_des_domaines(), [0, 2, 1]);
}

/// **Adopter** copie la définition, sous son identité, pour que le fichier reste entier ; adopter
/// deux fois ne double rien ; un domaine d'avant ne prend jamais la place d'un homonyme de numéro.
#[test]
fn adopter_copie_la_definition_une_seule_fois() {
    let mut store = Store::new("P");
    let venu = domaine("eau@5", "Eau", 9);
    assert_eq!(store.adopter_un_domaine(&venu).unwrap(), "eau@5");
    assert_eq!(store.project.domains[0], venu);
    let profondeur = store.undo_depth();
    assert_eq!(store.adopter_un_domaine(&venu).unwrap(), "eau@5");
    assert_eq!(store.project.domains.len(), 1);
    assert_eq!(store.undo_depth(), profondeur, "aucun geste de plus");

    store.project.domains.push(domaine("domain-3", "Feu", 1));
    let d_avant = domaine("domain-3", "Air", 2);
    let id = store
        .adopter_un_domaine(&d_avant)
        .expect("le sien, ailleurs");
    assert_eq!(
        id, "air@2",
        "l'identité qu'on lui lit, pas le numéro d'un autre"
    );
    assert_eq!(
        store.domain("domain-3").unwrap().name,
        "Feu",
        "l'autre reste"
    );
}

/// **DOM-4 à l'adoption** : le même nom sous une autre identité n'entre pas : c'est un homonyme.
#[test]
fn adopter_refuse_un_autre_sujet_de_meme_nom() {
    let mut store = Store::new("P");
    store.creer_un_domaine("Python", allure(), 1).unwrap();
    let serpent = domaine("python@2", "python", 2);
    assert_eq!(
        store.adopter_un_domaine(&serpent),
        Err(CoreError::NomDeDomainePris("Python".into()))
    );
    assert_eq!(store.project.domains.len(), 1);
}

/// **« Même sujet »** : un seul geste ; tout ce qui le porte, photo comprise, suit l'identité de
/// l'autre ; la définition la plus récente arrive ; `Ctrl+Z` rend tout, et `Ctrl+Y` refait tout.
#[test]
fn reunir_est_un_geste_et_tous_les_porteurs_suivent() {
    let mut store = Store::new("P");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("a", 0.0, 0.0, "a"));
    store.add_annotation(&board, Annotation::text("sans", 0.0, 0.0, "sans"));
    store.add_image(
        &board,
        crate::types::BoardImage::new("p", 0.0, 0.0, 10.0, 10.0),
    );
    let ici = store.creer_un_domaine("Eau", allure(), 1).unwrap();
    for n in ["a", "p"] {
        store
            .try_assign_domain_to_node(&board, n, &ici, 0.7)
            .unwrap();
    }
    store.journal.clear();

    let autre = domaine("eau@5", "eau", 9);
    assert_eq!(store.reunir_un_domaine(&ici, &autre), Ok(2));
    assert_eq!(store.undo_depth(), 1, "un seul geste");
    assert_eq!(store.project.domains, std::slice::from_ref(&autre));
    for n in ["a", "p"] {
        let porte = store.node_domains(&board, n).unwrap();
        assert_eq!(
            (porte[0].domain_id.as_str(), porte[0].weight),
            ("eau@5", 0.7)
        );
    }
    assert!(store.node_domains(&board, "sans").unwrap().is_empty());
    assert!(store.orphan_domain_references().is_empty(), "DOM-1");

    store.undo();
    assert_eq!(store.project.domains[0].id, ici);
    assert_eq!(store.node_domains(&board, "p").unwrap()[0].domain_id, ici);
    store.redo();
    assert_eq!(
        store.node_domains(&board, "a").unwrap()[0].domain_id,
        "eau@5"
    );
}

#[test]
fn reunir_refuse_ce_qui_casserait_le_catalogue() {
    let mut store = Store::new("P");
    let eau = store.creer_un_domaine("Eau", allure(), 1).unwrap();
    store.creer_un_domaine("Feu", allure(), 2).unwrap();
    assert_eq!(
        store.reunir_un_domaine("absent", &domaine("x@1", "X", 1)),
        Err(CoreError::DomainNotFound("absent".into()))
    );
    assert_eq!(
        store.reunir_un_domaine(&eau, &domaine("feu@9", "Feu", 9)),
        Err(CoreError::NomDeDomainePris("Feu".into()))
    );
    store.project.domains.push(domaine("deja@1", "Terre", 1));
    assert_eq!(
        store.reunir_un_domaine(&eau, &domaine("deja@1", "Eau", 9)),
        Err(CoreError::DuplicateDomainId("deja@1".into()))
    );
}

/// **Deux naissances dans la même milliseconde** : le nom d'attente repris aussitôt ne prend pas
/// l'identité du premier, qui porte déjà un autre nom.
#[test]
fn dom_5_deux_naissances_du_meme_nom_au_meme_instant() {
    let mut store = Store::new("P");
    let premier = store.creer_un_domaine("Domaine 1", allure(), 7).unwrap();
    store
        .try_update_domain(&premier, DomainPatchNom::vers("Python"))
        .unwrap();
    let second = store.creer_un_domaine("Domaine 1", allure(), 7).unwrap();
    assert_eq!(
        (premier.as_str(), second.as_str()),
        ("domaine 1@7", "domaine 1@8")
    );
    assert_eq!(store.domain(&second).unwrap().defini_a, 8);
}
