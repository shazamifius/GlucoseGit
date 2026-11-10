//! **LIEN-3 — la fenêtre des liens** : chaque nœud lié amené près du centre, dans sa direction,
//! dans la membrane de sa provenance ; et rien n'atteint le document.

use super::*;
use crate::types::Viewport;

/// Une petite carte : « Sciences › Chimie » porte le texte « eau » ; il est lié à « h » (dans
/// Chimie), à « jeu » (dans « Jeux vidéo », très loin), à la photo « p », et à la membrane
/// « Hydrogène » (dans Sciences), qui porte le texte « h-t ». « seul » n'a aucun lien.
fn carte() -> Store {
    let mut store = Store::new("liens");
    let b = store.project.active_board_id.clone();
    let membrane = |id: &str,
                    x: f64,
                    y: f64,
                    w: f64,
                    h: f64,
                    titre: &str,
                    couleur: &str,
                    dans: Option<&str>| {
        let mut m = Annotation::membrane(id, x, y, w, h);
        if let Annotation::Membrane { text, color, .. } = &mut m {
            (*text, *color) = (Some(titre.to_string()), Some(couleur.to_string()));
        }
        m.set_membrane_id(dans.map(str::to_string));
        m
    };
    let texte = |id: &str, x: f64, y: f64, dans: Option<&str>| {
        let mut t = Annotation::text(id, x, y, format!("le texte {id}"));
        if let Annotation::Text { width, height, .. } = &mut t {
            (*width, *height) = (Some(200.0), Some(60.0));
        }
        t.set_membrane_id(dans.map(str::to_string));
        t
    };
    for a in [
        membrane(
            "sciences", 0.0, 0.0, 9000.0, 6000.0, "Sciences", "#336699", None,
        ),
        membrane(
            "chimie",
            100.0,
            100.0,
            3000.0,
            2000.0,
            "Chimie",
            "#cc6633",
            Some("sciences"),
        ),
        membrane(
            "hydrogene",
            5000.0,
            3000.0,
            2000.0,
            1500.0,
            "Hydrogène",
            "#66cc33",
            Some("sciences"),
        ),
        membrane(
            "jeux",
            60000.0,
            -40000.0,
            2000.0,
            1500.0,
            "Jeux vidéo",
            "#9933cc",
            None,
        ),
        texte("eau", 1000.0, 1000.0, Some("chimie")),
        texte("h", 2500.0, 1800.0, Some("chimie")),
        texte("h-t", 5200.0, 3200.0, Some("hydrogene")),
        texte("h-u", 5100.0, 3900.0, Some("hydrogene")),
        texte("jeu", 60200.0, -39800.0, Some("jeux")),
        texte("seul", -5000.0, 0.0, None),
    ] {
        store.add_annotation(&b, a);
    }
    store.add_image(&b, BoardImage::new("p", -3000.0, 5000.0, 300.0, 200.0));
    for (k, vers) in ["h", "jeu", "p", "hydrogene"].into_iter().enumerate() {
        let mut f = Annotation::arrow(format!("f{k}"), 0.0, 0.0, 1.0, 1.0);
        if let Annotation::Arrow {
            source_id,
            target_id,
            waypoints,
            ..
        } = &mut f
        {
            (*source_id, *target_id) = (Some("eau".into()), Some(vers.into()));
            waypoints.push(crate::types::Point2D { x: 7.0, y: 7.0 });
        }
        store.add_annotation(&b, f);
    }
    store.clear_selection();
    store
}

fn copie_de<'a>(store: &'a Store, original: &str) -> &'a Annotation {
    let f = store.fenetre_des_liens().expect("ouverte");
    f.tableau
        .annotations
        .iter()
        .find(|a| f.original(a.id()) == Some(original) && !a.id().starts_with(PROVENANCE))
        .expect("la copie")
}

/// Le milieu de la boîte de la copie de ce nœud — le point qui glisse sur la demi-droite.
fn milieu_de(store: &Store, id: &str) -> (f64, f64) {
    let f = store.fenetre_des_liens().expect("ouverte");
    let copie = copie_de(store, id).id();
    milieu(crate::arrow::node_rect(&f.tableau, copie).expect("une boîte"))
}

/// **Chaque nœud lié est amené**, plus près qu'il n'était, dans la direction de sa vraie place —
/// à des kilomètres compris — ; le centre reste à sa place ; le tableau qu'on regarde est la
/// fenêtre.
#[test]
fn test_lien_3_chaque_lie_est_amene_dans_sa_direction() {
    let mut store = carte();
    assert!(store.montrer_les_liens("eau"));
    assert!(store
        .active_board()
        .expect("un tableau")
        .id
        .starts_with(TABLEAU));
    let c = milieu_de(&store, "eau");
    assert_eq!(c, (1100.0, 1030.0), "le centre à sa place");
    let board = &store.project.boards[0];
    for v in ["h", "jeu", "hydrogene"] {
        let vrai = milieu(crate::arrow::node_rect(board, v).expect("une boîte"));
        let amene = milieu_de(&store, v);
        let (d_vrai, d_amene) = (
            (vrai.0 - c.0).hypot(vrai.1 - c.1),
            (amene.0 - c.0).hypot(amene.1 - c.1),
        );
        assert!(d_amene < 2_000.0, "{v} amené près : {d_amene}");
        if d_vrai > 2_000.0 {
            assert!(d_amene < d_vrai, "{v} rapproché");
        }
        let (a, b) = (
            (vrai.1 - c.1).atan2(vrai.0 - c.0),
            (amene.1 - c.1).atan2(amene.0 - c.0),
        );
        assert!(
            (a - b).abs() < 1e-6,
            "{v} garde sa direction : {a} contre {b}"
        );
    }
    let f = store.fenetre_des_liens().expect("ouverte");
    assert!(
        f.tableau
            .images
            .iter()
            .any(|i| f.original(&i.id) == Some("p")),
        "la photo"
    );
}

/// **La provenance** : chacun dans une membrane titrée du chemin de ses membranes, à la couleur de
/// la plus proche ; une membrane liée vient par son chemin et son nom, autour de son premier texte.
#[test]
fn test_lien_3_chacun_dit_sa_provenance() {
    let mut store = carte();
    assert!(store.montrer_les_liens("eau"));
    let f = store.fenetre_des_liens().expect("ouverte");
    let membrane_de = |id: &str| {
        let m = copie_de(&store, id)
            .membrane_id()
            .expect("dans une membrane")
            .to_string();
        match f.tableau.annotations.iter().find(|a| a.id() == m) {
            Some(Annotation::Membrane { text, color, .. }) => (text.clone(), color.clone()),
            _ => panic!("pas une membrane"),
        }
    };
    assert_eq!(
        membrane_de("eau"),
        (Some("Sciences › Chimie".into()), Some("#cc6633".into()))
    );
    assert_eq!(
        membrane_de("jeu"),
        (Some("Jeux vidéo".into()), Some("#9933cc".into()))
    );
    assert_eq!(
        membrane_de("h-t"),
        (Some("Sciences › Hydrogène".into()), Some("#66cc33".into()))
    );
    let photo = f
        .tableau
        .images
        .iter()
        .find(|i| f.original(&i.id) == Some("p"));
    assert!(
        photo.expect("la photo").membrane_id.is_none(),
        "la photo n'est dans rien"
    );
    assert!(
        f.tableau
            .annotations
            .iter()
            .all(|a| f.original(a.id()) != Some("h-u")),
        "le premier texte seul"
    );
    assert_eq!(
        f.original(&format!("{PROVENANCE}eau")),
        Some("eau"),
        "la provenance mène au nœud"
    );
}

/// **Les liens relient les copies, droits** ; chaque identité de la fenêtre dit son original.
#[test]
fn test_lien_3_les_liens_relient_les_copies() {
    let mut store = carte();
    assert!(store.montrer_les_liens("eau"));
    let f = store.fenetre_des_liens().expect("ouverte");
    let fleches: Vec<&Annotation> = f
        .tableau
        .annotations
        .iter()
        .filter(|a| matches!(a, Annotation::Arrow { .. }))
        .collect();
    assert_eq!(fleches.len(), 4);
    for a in fleches {
        let Annotation::Arrow {
            source_id,
            target_id,
            waypoints,
            ..
        } = a
        else {
            unreachable!()
        };
        assert!(waypoints.is_empty(), "droite");
        for bout in [source_id, target_id] {
            let b = bout.as_deref().expect("un bout");
            assert!(
                crate::arrow::node_rect(&f.tableau, b).is_some(),
                "{b} est dans la fenêtre"
            );
        }
        assert!(f.original(a.id()).is_some_and(|o| o.starts_with('f')));
    }
}

/// **Rien n'atteint le document** : ouvrir, zoomer, déplacer la vue, choisir et supprimer des
/// copies, fermer — le document reste tel qu'il était, sa vue comprise ; la vue du document est
/// celle d'où l'on regardait la fenêtre, et l'enregistrement ne la connaît pas.
#[test]
fn test_lien_3_rien_n_atteint_le_document() {
    let mut store = carte();
    let avant = store.project.clone();
    assert!(store.montrer_les_liens("eau"));
    store.pan(120.0, -40.0);
    store.zoom(2.0, 10.0, 10.0, (0.01, 10.0));
    let fenetre = store.active_board().expect("un tableau").id.clone();
    store.set_viewport(
        &fenetre,
        Viewport {
            x: 5.0,
            y: 6.0,
            scale: 0.5,
        },
    );
    let copies: Vec<String> = store
        .active_board()
        .expect("ouverte")
        .annotations
        .iter()
        .map(|a| a.id().to_string())
        .collect();
    store.set_selected_annotation_ids(copies.clone());
    let reels = store.project.active_board_id.clone();
    store.remove_annotations(
        &reels,
        &copies.iter().map(String::as_str).collect::<Vec<_>>(),
    );
    assert_eq!(store.project, avant, "le document, fenêtre ouverte");
    let vue = crate::persist::histoire::Vue::de(&store.project);
    assert!(vue
        .tableaux
        .iter()
        .all(|(id, _, _)| !id.starts_with(TABLEAU)));
    assert!(store.fermer_la_fenetre_des_liens());
    assert_eq!(
        store.viewport(),
        Viewport {
            x: 5.0,
            y: 6.0,
            scale: 0.5
        }
    );
    let mut attendu = avant;
    attendu.boards[0].viewport = Viewport {
        x: 5.0,
        y: 6.0,
        scale: 0.5,
    };
    assert_eq!(store.project, attendu, "fermée : seule la vue a suivi");
}

/// **Un nœud sans lien n'a rien à montrer**, et la fenêtre ne s'ouvre pas.
#[test]
fn test_lien_3_sans_lien_rien_a_montrer() {
    let mut store = carte();
    assert!(!store.montrer_les_liens("seul"));
    assert!(store.fenetre_des_liens().is_none());
    assert!(!store.fermer_la_fenetre_des_liens());
}

/// **Elle s'enchaîne** : refaite autour d'un nœud qu'elle montre, elle le garde là où elle le
/// montrait, amène ses liens à lui, et change d'identité (l'index des nœuds la voit changer).
#[test]
fn test_lien_3_elle_se_refait_autour_d_un_amene() {
    let mut store = carte();
    assert!(store.montrer_les_liens("eau"));
    let ici = milieu(
        store
            .active_board()
            .expect("ouverte")
            .annotations
            .iter()
            .find(|a| a.id() == format!("{PROVENANCE}h"))
            .expect("sa provenance")
            .encre(),
    );
    let premiere = store.active_board().expect("ouverte").id.clone();
    let copie = copie_de(&store, "h").id().to_string();
    // « h » n'a qu'un lien, vers « eau » : la fenêtre se refait autour de lui.
    assert!(store.montrer_les_liens(&copie));
    let f = store.fenetre_des_liens().expect("ouverte");
    assert_eq!(f.centre, "h");
    assert_ne!(f.tableau.id, premiere);
    let la = milieu(
        f.tableau
            .annotations
            .iter()
            .find(|a| a.id() == format!("{PROVENANCE}h"))
            .expect("sa provenance")
            .encre(),
    );
    assert!(
        (la.0 - ici.0).abs() < 1e-9 && (la.1 - ici.1).abs() < 1e-9,
        "{la:?} contre {ici:?}"
    );
}

/// **Un autre tableau, un autre document la ferment** : elle montrait les liens de l'ancien.
#[test]
fn test_lien_3_changer_de_tableau_ou_de_document_la_ferme() {
    let mut store = carte();
    assert!(store.montrer_les_liens("eau"));
    let autre = store.add_board("autre");
    assert!(store.try_set_active_board_id(autre).is_ok());
    assert!(store.fenetre_des_liens().is_none());
    let reels = store.project.boards[0].id.clone();
    assert!(store.try_set_active_board_id(reels).is_ok());
    assert!(store.montrer_les_liens("eau"));
    store.load_project(crate::types::Project::new("neuf"));
    assert!(store.fenetre_des_liens().is_none());
}

/// **Une membrane montrée se resserre sur sa place** : sa copie, autour de son premier texte, est
/// centrée sur le milieu de la vraie membrane — et les demi-droites en partent.
#[test]
fn test_lien_3_une_membrane_montree_se_resserre_sur_sa_place() {
    let mut store = carte();
    assert!(store.montrer_les_liens("hydrogene"));
    let vraie = crate::arrow::node_rect(&store.project.boards[0], "hydrogene").expect("une boîte");
    let ici = milieu_de(&store, "hydrogene");
    let la = milieu(vraie);
    assert!(
        (ici.0 - la.0).abs() < 1e-9 && (ici.1 - la.1).abs() < 1e-9,
        "{ici:?} contre {la:?}"
    );
}

/// **Ce que tous partagent ne se répète pas** : la carte entière dans une membrane « Monde », son
/// nom se tait dans chaque provenance ; chacun garde la membrane la plus proche, et une membrane
/// amenée son nom.
#[test]
fn test_lien_3_ce_que_tous_partagent_ne_se_repete_pas() {
    let mut store = carte();
    let b = store.project.active_board_id.clone();
    store.add_annotation(
        &b,
        Annotation::membrane("monde", -90_000.0, -90_000.0, 200_000.0, 200_000.0),
    );
    if let Some(board) = store.active_board_mut() {
        for a in &mut board.annotations {
            if ["sciences", "jeux"].contains(&a.id()) {
                a.set_membrane_id(Some("monde".into()));
            }
            if let Annotation::Membrane { id, text, .. } = a {
                if id == "monde" {
                    *text = Some("Monde".into());
                }
            }
        }
        for i in &mut board.images {
            i.membrane_id = Some("monde".into());
        }
    }
    assert!(store.montrer_les_liens("eau"));
    let f = store.fenetre_des_liens().expect("ouverte");
    let titre_autour = |id: &str| {
        let m = copie_de(&store, id).membrane_id().map(str::to_string);
        f.tableau.annotations.iter().find_map(|a| match a {
            Annotation::Membrane { id, text, .. } if Some(id) == m.as_ref() => text.clone(),
            _ => None,
        })
    };
    assert_eq!(titre_autour("eau").as_deref(), Some("Sciences › Chimie"));
    assert_eq!(titre_autour("jeu").as_deref(), Some("Jeux vidéo"));
    assert_eq!(titre_autour("h-t").as_deref(), Some("Sciences › Hydrogène"));
    let photo = f
        .tableau
        .images
        .iter()
        .find(|i| f.original(&i.id) == Some("p"))
        .expect("la photo");
    let autour = photo.membrane_id.as_deref().expect("dans sa provenance");
    let titre = f
        .tableau
        .annotations
        .iter()
        .find(|a| a.id() == autour)
        .and_then(|a| match a {
            Annotation::Membrane { text, .. } => text.clone(),
            _ => None,
        });
    assert_eq!(
        titre.as_deref(),
        Some("Monde"),
        "la plus proche, même commune"
    );
}
