use super::*;
use glucose_core::geometry::Rect;
use glucose_core::store::agents::Sorte;

fn texte(id: &str, membrane: Option<&str>, t: &str) -> Decrit {
    Decrit {
        id: id.into(),
        sorte: Sorte::Texte,
        boite: Rect::new(0.0, 0.0, 10.0, 10.0),
        texte: Some(t.into()),
        membrane: membrane.map(str::to_string),
        source: None,
        cible: None,
        relation: None,
        adresse: None,
    }
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
    assert_eq!(premiere_phrase("  \n Bonjour. Suite"), Some("Bonjour."));
    assert_eq!(premiere_phrase(""), None);
    assert_eq!(
        phrases("v2.0 est sortie"),
        ["v2.0 est sortie"],
        "un point sans blanc ne coupe pas"
    );
}

#[test]
fn le_passage_est_la_phrase_qui_contient_les_mots() {
    let t = "Le pipeline lit le maillage. La commande polyDissolve n'existe pas dans Maya.";
    assert_eq!(
        passage(t, "POLYDISSOLVE").as_deref(),
        Some("La commande polyDissolve n'existe pas dans Maya.")
    );
    assert_eq!(passage(t, "Blender"), None);
    assert_eq!(passage(t, "  "), None, "chercher rien ne trouve rien");
}

#[test]
fn le_sommaire_d_abord_puis_ce_qu_on_choisit() {
    let n = [
        texte("manuel", None, "MeshForge. Mille lignes."),
        texte("note", Some("bulle"), "Une note sur Maya."),
        texte("bulle", None, "Bulle"),
    ];
    let d =
        |ids: Option<Vec<&'static str>>, cherche: Option<&'static str>, dans, complet| Demande {
            dans,
            ids,
            cherche,
            complet,
        };
    let ids_de = |v: Vec<(&Decrit, Rendu)>| {
        v.into_iter()
            .map(|(n, r)| (n.id.clone(), r))
            .collect::<Vec<_>>()
    };
    assert!(
        ids_de(choisir(&n, &d(None, None, None, false)))
            .iter()
            .all(|(_, r)| *r == Rendu::Sommaire),
        "par défaut, le sommaire"
    );
    assert!(
        ids_de(choisir(&n, &d(None, None, None, true)))
            .iter()
            .all(|(_, r)| *r == Rendu::Entier),
        "complet sur demande"
    );
    assert_eq!(
        ids_de(choisir(&n, &d(Some(vec!["manuel"]), None, None, false))),
        [("manuel".to_string(), Rendu::Entier)]
    );
    assert_eq!(
        ids_de(choisir(&n, &d(None, Some("maya"), None, false))),
        [(
            "note".to_string(),
            Rendu::Passage("Une note sur Maya.".into())
        )]
    );
    let dans: Vec<String> = choisir(&n, &d(None, None, Some("bulle"), false))
        .into_iter()
        .map(|(n, _)| n.id.clone())
        .collect();
    assert_eq!(
        dans,
        ["note", "bulle"],
        "une membrane et ce qu'elle contient"
    );
}

#[test]
fn l_empreinte_suit_le_texte_et_lui_seul() {
    let e = empreinte("Le portage existe.");
    assert_eq!(e.len(), 16, "un mot machine en hexadécimal");
    assert_eq!(e, empreinte("Le portage existe."), "le même texte, la même");
    assert_ne!(
        e,
        empreinte("Le portage existe !"),
        "un signe change, elle change"
    );
    assert_eq!(
        empreinte(""),
        "e3b0c44298fc1c14",
        "le début du sha256 du vide"
    );
}
