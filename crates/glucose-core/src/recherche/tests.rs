use super::*;
use crate::store::Store;
use crate::types::{Annotation, BoardImage};

/// **PLI-1** : sans majuscules ni accents, dans tout le latin ; ce qui n'est pas latin reste tel
/// quel, en minuscules.
#[test]
fn pli_1_ni_majuscules_ni_accents() {
    assert_eq!(pli("Été À l'ŒUVRE"), "ete a l'oeuvre");
    assert_eq!(pli("Ça déçoit"), "ca decoit");
    assert_eq!(pli("Straße, Æsir, Þór"), "strasse, aesir, thor");
    assert_eq!(pli("Łódź, Ğ, Ř, ž"), "lodz, g, r, z");
    // Un « é » écrit en deux : la lettre, puis son accent.
    assert_eq!(pli("e\u{301}te\u{301}"), "ete");
    assert_eq!(pli("Москва ΑΘΗΝΑ 東京"), "москва αθηνα 東京");
    assert_eq!(pli("× ÷"), "× ÷", "deux signes, pas des lettres");
}

#[test]
fn les_phrases_sont_l_unite() {
    assert_eq!(
        phrases("# Manuel\nMeshForge répare. Il simplifie aussi ! Et Maya ?\n\nFin"),
        [
            "# Manuel",
            "MeshForge répare.",
            "Il simplifie aussi !",
            "Et Maya ?",
            "Fin"
        ]
    );
    assert_eq!(
        phrases("La v2.0 sort."),
        ["La v2.0 sort."],
        "un point sans blanc"
    );
}

#[test]
fn le_passage_ignore_majuscules_et_accents() {
    let t = "Le café est noir. L'ÉTÉ est chaud.";
    assert_eq!(passage(t, "ete").as_deref(), Some("L'ÉTÉ est chaud."));
    assert_eq!(passage(t, "CAFE").as_deref(), Some("Le café est noir."));
    assert_eq!(passage(t, "  "), None);
    assert_eq!(passage(t, "hiver"), None);
}

/// Un document de deux tableaux : des textes, un post-it, une membrane, une flèche, une photo, un
/// dossier.
fn document() -> Store {
    let mut s = Store::new("P");
    let un = s.project.active_board_id.clone();
    let deux = s.add_board("Physique de l'eau");
    s.add_annotation(
        &un,
        Annotation::text("bureau", 0.0, 0.0, "Le bureau est rangé."),
    );
    s.add_annotation(
        &un,
        Annotation::text("mouille", 0.0, 0.0, "Ça mouille. L'eau, ça mouille."),
    );
    s.add_annotation(&deux, Annotation::sticky("h2o", 0.0, 0.0, "Eau = H2O"));
    let mut m = Annotation::membrane("bulle", 0.0, 0.0, 10.0, 10.0);
    if let Annotation::Membrane { text, .. } = &mut m {
        *text = Some("Chimie de l'eau".into());
    }
    s.add_annotation(&deux, m);
    let mut photo = BoardImage::new("vague", 0.0, 0.0, 10.0, 10.0);
    photo.tags = vec!["océan".into(), "vague".into()];
    photo.source_url = Some("https://exemple.org/eau-de-mer".into());
    s.add_image(&un, photo);
    s
}

fn ids(trouves: &[Trouve]) -> Vec<String> {
    trouves
        .iter()
        .map(|t| {
            t.id.clone()
                .unwrap_or_else(|| format!("tableau:{}", t.tableau))
        })
        .collect()
}

/// **TROUVE-1** : tout le document, tous les genres ; le début d'un mot d'abord (« eau » trouve
/// « l'eau » avant « bureau ») ; à égalité, l'ordre du document.
#[test]
fn trouve_1_le_debut_d_un_mot_d_abord() {
    let s = document();
    let deux = s.project.boards[1].id.clone();
    let trouves = chercher(&s.project, "eau");
    assert_eq!(
        ids(&trouves),
        [
            "vague".to_string(),
            "mouille".to_string(),
            format!("tableau:{deux}"),
            "h2o".to_string(),
            "bulle".to_string(),
            "bureau".to_string(),
        ]
    );
    let mouille = &trouves[1];
    assert_eq!(
        passage_du(&s.project, mouille, "eau"),
        "L'eau, ça mouille.",
        "la phrase du mot"
    );
    assert_eq!(mouille.genre, Genre::Noeud(Sorte::Texte));
    assert_eq!(trouves[2].genre, Genre::Tableau);
    assert_eq!(
        trouves.last().unwrap().debuts,
        0,
        "« bureau » le contient au milieu"
    );
}

/// **Chaque mot, n'importe où** : tous doivent y être ; l'ordre des mots ne compte pas.
#[test]
fn trouve_1_chaque_mot_doit_y_etre() {
    let s = document();
    assert_eq!(ids(&chercher(&s.project, "mouille eau")), ["mouille"]);
    assert_eq!(ids(&chercher(&s.project, "EAU   ça")), ["mouille"]);
    assert!(chercher(&s.project, "eau feu").is_empty());
    assert!(
        chercher(&s.project, "   ").is_empty(),
        "rien cherché, rien trouvé"
    );
}

/// **Les photos par leurs étiquettes et leur adresse**, les accents ignorés ; un dossier par son nom.
#[test]
fn les_photos_et_les_dossiers() {
    let mut s = document();
    let un = s.project.active_board_id.clone();
    let classeur = crate::types::CanvasFolder::new("classeur", "Archives de l'Océan", un.clone());
    s.project.boards[0].folders.push(classeur);
    assert_eq!(ids(&chercher(&s.project, "ocean")), ["vague", "classeur"]);
    let mer = chercher(&s.project, "mer");
    assert_eq!(ids(&mer), ["vague"]);
    assert_eq!(
        passage_du(&s.project, &mer[0], "mer"),
        "https://exemple.org/eau-de-mer"
    );
}

/// **L'index rend ce que rend une recherche faite d'un coup**, et il ne voit pas ce qui change après
/// lui : c'est à qui le garde de le refaire quand le document change.
#[test]
fn l_index_rend_la_meme_chose_et_se_refait() {
    let mut s = document();
    let index = Index::du(&s.project);
    for requete in ["eau", "mouille eau", "ocean", "feu", ""] {
        assert_eq!(
            index.chercher(requete),
            chercher(&s.project, requete),
            "{requete}"
        );
    }
    let un = s.project.active_board_id.clone();
    s.add_annotation(&un, Annotation::text("feu", 0.0, 0.0, "Le feu brûle."));
    assert!(
        index.chercher("feu").is_empty(),
        "l'ancien index ne le voit pas"
    );
    assert_eq!(Index::du(&s.project).chercher("feu").len(), 1);
}

/// Une phrase vide ou un nœud disparu ne donnent rien, sans tomber.
#[test]
fn une_phrase_introuvable_est_vide() {
    let s = document();
    let perdu = Trouve {
        tableau: "absent".into(),
        id: None,
        genre: Genre::Tableau,
        debuts: 0,
    };
    assert_eq!(passage_du(&s.project, &perdu, "eau"), "");
}
