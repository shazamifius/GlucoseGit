use super::*;
use crate::text_anchors::resolve_text_sel;

/// **Un carnet, pas un par appel** : le même nom rend le même onglet ; le créer est un geste
/// qu'un `Ctrl+Z` défait ; l'onglet qu'il regarde ne change jamais.
#[test]
fn ouvrir_un_tableau_rend_le_meme_et_ne_change_pas_son_regard() {
    let (mut store, regarde) = son_tableau();
    let (id, cree) = store
        .ouvrir_un_tableau_pour_un_agent("Ce que Protéo a appris")
        .expect("ouvert");
    assert!(cree && id != regarde);
    assert_eq!(store.project.active_board_id, regarde, "son regard reste");
    let profondeur = store.undo_depth();
    let (encore, cree) = store
        .ouvrir_un_tableau_pour_un_agent("Ce que Protéo a appris")
        .expect("rouvert");
    assert_eq!((encore.as_str(), cree), (id.as_str(), false));
    assert_eq!(store.undo_depth(), profondeur, "rouvrir n'est pas un geste");
    assert_eq!(store.selected_annotation_ids, ["sien"], "AGENT-2");
    store.undo();
    assert!(
        store.onglets().all(|(i, _, _)| i != id),
        "un Ctrl+Z le défait"
    );
    // Pendant un geste de sa main, l'appel attend : son Ctrl+Z défairait les deux (AGENT-1).
    store.begin_live_edit();
    assert_eq!(
        store.ouvrir_un_tableau_pour_un_agent("Archives"),
        Err(Refus::GesteEnCours)
    );
    store.cancel_live_edit();
}

/// Un tableau où l'utilisateur a déjà travaillé : un texte à lui, choisi.
fn son_tableau() -> (Store, String) {
    let mut store = Store::new("agents");
    let board = store.project.active_board_id.clone();
    store.add_annotation(&board, Annotation::text("sien", 0.0, 0.0, "à moi"));
    store.select_annotation("sien".to_string(), false);
    store.journal.clear();
    (store, board)
}

fn poser(store: &mut Store, board: &str, ann: Annotation) -> Result<(), Refus> {
    store.geste_d_agent(|s| s.poser_pour_un_agent(board, ann))
}

/// Ce que l'ancre d'un côté de la flèche désigne, dans le texte de son nœud.
fn designe(store: &Store, board: &str, fleche: &str, cote_source: bool) -> Option<String> {
    let Annotation::Arrow {
        source_id,
        target_id,
        source_text_sel,
        target_text_sel,
        ..
    } = store.project.annotation(board, fleche)?
    else {
        return None;
    };
    let (noeud, sel) = if cote_source {
        (source_id.as_deref()?, source_text_sel)
    } else {
        (target_id.as_deref()?, target_text_sel)
    };
    let texte = store.project.annotation(board, noeud)?.own_text()?;
    let plage = *resolve_text_sel(&texte, sel.as_ref()).first()?;
    Some(texte[plage.start..plage.end].to_string())
}

#[test]
fn agent_1_un_appel_est_un_seul_geste_que_ctrl_z_defait() {
    let (mut store, board) = son_tableau();
    store
        .geste_d_agent(|s| {
            s.poser_pour_un_agent(&board, Annotation::text("a", 0.0, 300.0, "un"))?;
            s.poser_pour_un_agent(&board, Annotation::text("b", 0.0, 600.0, "deux"))
        })
        .expect("posés");
    assert_eq!(store.undo_depth(), 1, "deux nœuds, un seul geste");
    assert!(store.undo());
    assert!(store.project.annotation(&board, "a").is_none());
    assert!(store.project.annotation(&board, "b").is_none());
    assert!(store.project.annotation(&board, "sien").is_some());
}

#[test]
fn agent_1_un_appel_qui_echoue_ne_laisse_rien() {
    let (mut store, board) = son_tableau();
    let version = store.version;
    let refus = store.geste_d_agent(|s| {
        s.poser_pour_un_agent(&board, Annotation::text("a", 0.0, 300.0, "un"))?;
        s.relier_pour_un_agent(
            &board,
            Lien {
                id: "f",
                source: "a",
                cible: "sien",
                passage_source: Some("introuvable"),
                ..Lien::default()
            },
        )
    });
    assert!(matches!(refus, Err(Refus::PassageAbsent { .. })));
    assert!(
        store.project.annotation(&board, "a").is_none(),
        "le texte posé avant l'échec part"
    );
    assert_eq!(store.undo_depth(), 0);
    assert_eq!(store.version, version, "le document n'a pas changé");
}

#[test]
fn agent_1_rien_ne_s_ecrit_pendant_un_geste_de_l_utilisateur() {
    let (mut store, board) = son_tableau();
    store.begin_live_edit();
    let refus = poser(&mut store, &board, Annotation::text("a", 0.0, 300.0, "un"));
    assert_eq!(refus, Err(Refus::GesteEnCours));
    assert!(store.project.annotation(&board, "a").is_none());
    assert!(store.in_live_edit(), "son geste reste ouvert");
}

#[test]
fn agent_2_la_selection_de_l_utilisateur_reste_la_sienne() {
    let (mut store, board) = son_tableau();
    poser(&mut store, &board, Annotation::text("a", 0.0, 300.0, "un")).expect("posé");
    assert_eq!(store.selected_annotation_ids, vec!["sien".to_string()]);
    // Retirer ce qu'il avait choisi : la sélection ne garde pas un nœud disparu.
    store
        .geste_d_agent(|s| s.retirer_pour_un_agent(&board, &["sien"]))
        .expect("retiré");
    assert!(store.selected_annotation_ids.is_empty());
}

#[test]
fn agent_3_la_fleche_part_du_passage_cite_et_arrive_au_passage_cite() {
    let (mut store, board) = son_tableau();
    let rapport = "Le glucose est un sucre. Sa formule est C6H12O6 [1].";
    let source = "Formule brute : C6H12O6, masse 180 g/mol.";
    poser(
        &mut store,
        &board,
        Annotation::text("rapport", 0.0, 300.0, rapport),
    )
    .expect("posé");
    poser(
        &mut store,
        &board,
        Annotation::text("page", 600.0, 300.0, source),
    )
    .expect("posé");
    store
        .geste_d_agent(|s| {
            s.relier_pour_un_agent(
                &board,
                Lien {
                    id: "f",
                    source: "rapport",
                    cible: "page",
                    passage_source: Some("Sa formule est C6H12O6 [1]."),
                    passage_cible: Some("C6H12O6"),
                    relation: Some(ArrowPredicate::DependDe),
                    ..Lien::default()
                },
            )
        })
        .expect("relié");
    assert_eq!(
        designe(&store, &board, "f", true).as_deref(),
        Some("Sa formule est C6H12O6 [1].")
    );
    assert_eq!(
        designe(&store, &board, "f", false).as_deref(),
        Some("C6H12O6")
    );
}

#[test]
fn agent_3_un_passage_qui_n_est_pas_mot_pour_mot_est_refuse() {
    let (mut store, board) = son_tableau();
    poser(
        &mut store,
        &board,
        Annotation::text("page", 600.0, 0.0, "masse 180 g/mol"),
    )
    .expect("posé");
    let refus = store.geste_d_agent(|s| {
        s.relier_pour_un_agent(
            &board,
            Lien {
                id: "f",
                source: "sien",
                cible: "page",
                passage_cible: Some("masse 180 g / mol"),
                ..Lien::default()
            },
        )
    });
    assert_eq!(
        refus,
        Err(Refus::PassageAbsent {
            noeud: "page".to_string(),
            passage: "masse 180 g / mol".to_string()
        })
    );
    assert!(store.project.annotation(&board, "f").is_none());
}

#[test]
fn agent_4_poser_deux_fois_le_meme_identifiant_ne_double_rien() {
    let (mut store, board) = son_tableau();
    for texte in ["premier jet", "second jet"] {
        poser(
            &mut store,
            &board,
            Annotation::text("doc", 0.0, 300.0, texte),
        )
        .expect("posé");
    }
    let docs = store
        .decrire_pour_un_agent(&board)
        .expect("lu")
        .into_iter()
        .filter(|d| d.id == "doc")
        .count();
    assert_eq!(docs, 1);
    assert_eq!(
        store
            .project
            .annotation(&board, "doc")
            .and_then(Annotation::own_text)
            .as_deref(),
        Some("second jet")
    );
}

#[test]
fn agent_4_une_mise_a_jour_fait_suivre_les_passages_des_fleches() {
    let (mut store, board) = son_tableau();
    poser(
        &mut store,
        &board,
        Annotation::text("doc", 0.0, 300.0, "Le sucre [1]."),
    )
    .expect("posé");
    store
        .geste_d_agent(|s| {
            s.relier_pour_un_agent(
                &board,
                Lien {
                    id: "f",
                    source: "doc",
                    cible: "sien",
                    passage_source: Some("[1]"),
                    ..Lien::default()
                },
            )
        })
        .expect("relié");
    let neuf = "Introduction. Le sucre [1].";
    poser(
        &mut store,
        &board,
        Annotation::text("doc", 0.0, 300.0, neuf),
    )
    .expect("mis à jour");
    assert_eq!(designe(&store, &board, "f", true).as_deref(), Some("[1]"));
}

#[test]
fn agent_4_un_identifiant_d_une_autre_sorte_est_refuse() {
    let (mut store, board) = son_tableau();
    let refus = poser(
        &mut store,
        &board,
        Annotation::sticky("sien", 0.0, 0.0, "x"),
    );
    assert_eq!(
        refus,
        Err(Refus::AutreSorte {
            id: "sien".to_string(),
            est: "texte"
        })
    );
}

#[test]
fn ecrire_allonge_un_document_que_l_agent_remplit() {
    let (mut store, board) = son_tableau();
    poser(
        &mut store,
        &board,
        Annotation::text("doc", 0.0, 300.0, "# Rapport\n"),
    )
    .expect("posé");
    store
        .geste_d_agent(|s| s.ecrire_pour_un_agent(&board, "doc", "Première ligne.", true))
        .expect("écrit");
    assert_eq!(
        store
            .project
            .annotation(&board, "doc")
            .and_then(Annotation::own_text)
            .as_deref(),
        Some("# Rapport\nPremière ligne.")
    );
}

#[test]
fn le_prochain_noeud_se_pose_sous_le_contenu_de_sa_membrane_qui_grandit() {
    let (mut store, board) = son_tableau();
    poser(
        &mut store,
        &board,
        Annotation::membrane("bulle", 1000.0, 0.0, 400.0, 200.0),
    )
    .expect("posée");
    let (x, y) = store
        .place_pour_un_agent(&board, Some("bulle"))
        .expect("une place");
    assert_eq!(
        (x, y),
        (1000.0 + ECART, TITRE_DE_MEMBRANE),
        "vide : en haut à gauche"
    );
    let boite = Rect::new(x, y, 300.0, 500.0);
    store
        .geste_d_agent(|s| {
            s.agrandir_la_membrane_pour(&board, "bulle", boite)?;
            let mut texte = Annotation::text("t", x, y, "dedans");
            if let Annotation::Text { width, height, .. } = &mut texte {
                (*width, *height) = (Some(300.0), Some(500.0));
            }
            s.poser_pour_un_agent(&board, texte)
        })
        .expect("posé");
    let cadre = store
        .project
        .annotation(&board, "bulle")
        .expect("bulle")
        .bounds();
    assert!(
        cadre.bottom() >= boite.bottom() + ECART,
        "la bulle a grandi"
    );
    assert_eq!(
        store
            .project
            .annotation(&board, "t")
            .and_then(Annotation::membrane_id),
        Some("bulle"),
        "il appartient à la bulle dès sa naissance"
    );
    let (_, y2) = store
        .place_pour_un_agent(&board, Some("bulle"))
        .expect("une place");
    assert_eq!(y2, boite.bottom() + ECART, "le suivant se pose dessous");
}

#[test]
fn un_texte_qui_grandit_fait_grandir_sa_membrane() {
    let (mut store, board) = son_tableau();
    poser(
        &mut store,
        &board,
        Annotation::membrane("bulle", 0.0, 1000.0, 400.0, 300.0),
    )
    .expect("posée");
    let mut doc = Annotation::text("doc", 30.0, 1042.0, "court");
    if let Annotation::Text { width, height, .. } = &mut doc {
        (*width, *height) = (Some(300.0), Some(100.0));
    }
    poser(&mut store, &board, doc).expect("posé");
    assert_eq!(
        store
            .project
            .annotation(&board, "doc")
            .and_then(Annotation::membrane_id),
        Some("bulle")
    );
    store
        .geste_d_agent(|s| s.tenir_la_hauteur_pour_un_agent(&board, "doc", 900.0))
        .expect("tenu");
    let doc = store
        .project
        .annotation(&board, "doc")
        .expect("doc")
        .bounds();
    assert_eq!(doc.height, 900.0);
    let bulle = store
        .project
        .annotation(&board, "bulle")
        .expect("bulle")
        .bounds();
    assert!(
        bulle.bottom() >= doc.bottom() + ECART,
        "{bulle:?} contient {doc:?}"
    );
}

#[test]
fn sans_membrane_le_prochain_noeud_se_pose_a_droite_de_tout() {
    let (store, board) = son_tableau();
    let sien = store
        .project
        .annotation(&board, "sien")
        .expect("sien")
        .bounds();
    let place = store.place_pour_un_agent(&board, None).expect("une place");
    assert_eq!(place, (sien.right() + ECART, sien.top));
}

#[test]
fn deplacer_dans_une_membrane_l_y_rattache() {
    let (mut store, board) = son_tableau();
    poser(
        &mut store,
        &board,
        Annotation::membrane("bulle", 1000.0, 0.0, 600.0, 600.0),
    )
    .expect("posée");
    store
        .geste_d_agent(|s| s.deplacer_pour_un_agent(&board, "sien", (1100.0, 100.0)))
        .expect("déplacé");
    let sien = store.project.annotation(&board, "sien").expect("sien");
    assert_eq!((sien.x(), sien.y()), (1100.0, 100.0));
    assert_eq!(sien.membrane_id(), Some("bulle"));
    assert_eq!(store.selected_annotation_ids, vec!["sien".to_string()]);
}

#[test]
fn une_photo_dit_la_page_d_ou_elle_vient() {
    let (mut store, board) = son_tableau();
    let mut img = BoardImage::new("page", 0.0, 0.0, 320.0, 200.0);
    img.source_url = Some("https://fr.wikipedia.org/wiki/Glucose".to_string());
    store
        .geste_d_agent(|s| s.poser_une_photo_pour_un_agent(&board, img))
        .expect("posée");
    let decrit = store.decrire_pour_un_agent(&board).expect("lu");
    let page = decrit.iter().find(|d| d.id == "page").expect("la photo");
    assert_eq!(page.sorte, Sorte::Photo);
    assert_eq!(
        page.adresse.as_deref(),
        Some("https://fr.wikipedia.org/wiki/Glucose")
    );
    assert_eq!(page.boite, Rect::new(-160.0, -100.0, 320.0, 200.0));
}

#[test]
fn retirer_n_atteint_pas_un_autre_tableau_et_emporte_les_fleches_orphelines() {
    let (mut store, board) = son_tableau();
    let autre = store.add_board("autre");
    store.add_annotation(&autre, Annotation::text("ailleurs", 0.0, 0.0, "x"));
    poser(&mut store, &board, Annotation::text("a", 0.0, 300.0, "un")).expect("posé");
    store
        .geste_d_agent(|s| {
            s.relier_pour_un_agent(
                &board,
                Lien {
                    id: "f",
                    source: "a",
                    cible: "sien",
                    ..Lien::default()
                },
            )
        })
        .expect("relié");
    let n = store
        .geste_d_agent(|s| s.retirer_pour_un_agent(&board, &["a", "ailleurs"]))
        .expect("retiré");
    assert_eq!(n, 1);
    assert!(
        store.project.annotation(&board, "f").is_none(),
        "la flèche sans bout part"
    );
    assert!(store.project.annotation(&autre, "ailleurs").is_some());
}

#[test]
fn agent_4_une_mise_a_jour_garde_le_noeud_dans_sa_membrane() {
    let (mut store, board) = son_tableau();
    poser(
        &mut store,
        &board,
        Annotation::membrane("bulle", 0.0, 1000.0, 600.0, 600.0),
    )
    .expect("posée");
    poser(
        &mut store,
        &board,
        Annotation::text("doc", 50.0, 1100.0, "un"),
    )
    .expect("posé");
    assert_eq!(
        store
            .project
            .annotation(&board, "doc")
            .and_then(Annotation::membrane_id),
        Some("bulle")
    );
    poser(
        &mut store,
        &board,
        Annotation::text("doc", 50.0, 1100.0, "deux"),
    )
    .expect("mis à jour");
    assert_eq!(
        store
            .project
            .annotation(&board, "doc")
            .and_then(Annotation::membrane_id),
        Some("bulle")
    );
}

#[test]
fn agent_4_un_identifiant_pris_sur_un_autre_tableau_est_refuse() {
    let (mut store, board) = son_tableau();
    let autre = store.add_board("autre");
    store.add_annotation(&autre, Annotation::text("ailleurs", 0.0, 0.0, "x"));
    let refus = poser(
        &mut store,
        &board,
        Annotation::text("ailleurs", 0.0, 0.0, "y"),
    );
    assert_eq!(refus, Err(Refus::Pris("ailleurs".to_string())));
}

/// La place enregistrée, pas seulement le texte : Glucose retrouve un passage par sa citation
/// et son contexte même quand sa place est fausse — robuste, et invisible à une épreuve qui ne
/// regarde que le texte désigné. L'ancre elle-même doit avoir suivi : « Intro » effacé devant,
/// elle recule d'autant.
#[test]
fn agent_4_une_mise_a_jour_fait_reculer_la_place_enregistree_de_l_ancre() {
    let (mut store, board) = son_tableau();
    let avant = "Intro [1] [1]";
    poser(
        &mut store,
        &board,
        Annotation::text("doc", 0.0, 300.0, avant),
    )
    .expect("posé");
    let ancre = crate::text_anchors::create_anchor(avant, 10, 13).expect("le second");
    store
        .geste_d_agent(|s| {
            s.relier_pour_un_agent(
                &board,
                Lien {
                    id: "f",
                    source: "doc",
                    cible: "sien",
                    ..Lien::default()
                },
            )?;
            s.ancrer_la_fleche(&board, "f", (vec![ancre], Vec::new()));
            Ok(())
        })
        .expect("relié au second");
    poser(
        &mut store,
        &board,
        Annotation::text("doc", 0.0, 300.0, "[1] [1]"),
    )
    .expect("mis à jour");
    let Some(Annotation::Arrow {
        source_text_sel: Some(TextSelection::Anchors(ancres)),
        ..
    }) = store.project.annotation(&board, "f")
    else {
        panic!("la flèche et son ancre");
    };
    assert_eq!((ancres[0].start, ancres[0].end), (4, 7));
}
