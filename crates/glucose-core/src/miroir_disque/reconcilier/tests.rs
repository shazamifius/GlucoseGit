//! La loi de réconciliation, sur un tableau fabriqué : chaque fichier du niveau `/d`.

use super::*;
use crate::miroir_disque::Sorte;
use crate::types::{AssetRef, DomainAssignment};

const RACINE: &str = "/d";

fn entree(nom: &str, sorte: Sorte, taille: u64, date: i64) -> Entree {
    Entree {
        nom: nom.to_string(),
        chemin: format!("{RACINE}/{nom}"),
        sorte,
        taille,
        date,
    }
}

fn photo_lue(nom: &str, taille: u64, date: i64) -> Entree {
    let sorte = Sorte::Photo {
        largeur: 4,
        hauteur: 3,
    };
    entree(nom, sorte, taille, date)
}

fn texte_lu(nom: &str, taille: u64, date: i64) -> Entree {
    entree(nom, Sorte::Lisible, taille, date)
}

fn photo(id: &str, nom: &str, taille: u64) -> BoardImage {
    let mut img = BoardImage::new(id, 0.0, 0.0, 40.0, 30.0);
    img.src = Some(format!("{RACINE}/{nom}"));
    img.asset = Some(AssetRef::Link {
        href: format!("{RACINE}/{nom}"),
        sha256: None,
        size_bytes: Some(taille),
    });
    img
}

fn texte(id: &str, nom: &str, contenu: &str) -> Annotation {
    let mut a = Annotation::text(id, 0.0, 0.0, contenu);
    if let Annotation::Text { source_file, .. } = &mut a {
        *source_file = Some(format!("{RACINE}/{nom}"));
    }
    a
}

fn vu(taille: u64, date: i64, empreinte: Option<u64>) -> Vu {
    Vu {
        taille,
        date,
        empreinte,
    }
}

fn vus(lignes: &[(&str, Vu)]) -> Vus {
    lignes
        .iter()
        .map(|(nom, v)| (format!("{RACINE}/{nom}"), *v))
        .collect()
}

fn ids(noeuds: &[Noeud]) -> Vec<&str> {
    noeuds
        .iter()
        .map(|n| match n {
            Noeud::Photo(id) | Noeud::Note(id) | Noeud::Dossier(id) => id.as_str(),
        })
        .collect()
}

fn noms(entrees: &[Entree]) -> Vec<&str> {
    entrees.iter().map(|e| e.nom.as_str()).collect()
}

#[test]
fn test_la_premiere_lecture_fait_tout_entrer_et_s_en_souvient() {
    let b = Board::new("miroir", "d");
    let lus = [photo_lue("a.png", 10, 1), texte_lu("b.md", 5, 2)];
    let frais = HashMap::from([(lus[1].chemin.clone(), 77)]);
    let plan = reconcilier(&b, RACINE, None, &lus, &frais);
    assert_eq!(noms(&plan.nouveaux), ["a.png", "b.md"]);
    assert!(plan.suivis.is_empty() && plan.partis.is_empty());
    assert_eq!(
        plan.vus,
        vus(&[("a.png", vu(10, 1, None)), ("b.md", vu(5, 2, Some(77)))])
    );
}

/// Un nœud retiré ne revient pas ; son fichier changé, il revient — un rendu refait sous le même nom.
#[test]
fn test_un_noeud_retire_ne_revient_pas_tant_que_son_fichier_ne_change_pas() {
    let b = Board::new("miroir", "d");
    let avant = vus(&[("a.png", vu(10, 1, None))]);
    let plan = reconcilier(
        &b,
        RACINE,
        Some(&avant),
        &[photo_lue("a.png", 10, 1)],
        &HashMap::new(),
    );
    assert!(plan.est_vide(), "refusé : {plan:?}");
    assert_eq!(plan.vus, avant, "et le miroir s'en souvient encore");
    let plan = reconcilier(
        &b,
        RACINE,
        Some(&avant),
        &[photo_lue("a.png", 12, 9)],
        &HashMap::new(),
    );
    assert_eq!(noms(&plan.nouveaux), ["a.png"]);
}

#[test]
fn test_une_photo_changee_suit_son_fichier_et_une_photo_identique_ne_bouge_pas() {
    let mut b = Board::new("miroir", "d");
    b.images = vec![photo("p", "a.png", 10), photo("q", "b.png", 20)];
    let avant = vus(&[("a.png", vu(10, 1, None)), ("b.png", vu(20, 1, None))]);
    let lus = [photo_lue("a.png", 11, 5), photo_lue("b.png", 20, 1)];
    let plan = reconcilier(&b, RACINE, Some(&avant), &lus, &HashMap::new());
    assert_eq!(plan.suivis.len(), 1);
    assert_eq!(plan.suivis[0].noeud, Noeud::Photo("p".into()));
    assert_eq!(plan.suivis[0].entree, lus[0]);
    assert!(plan.nouveaux.is_empty() && plan.partis.is_empty());
    // Sans mémoire, la taille que la photo porte suffit à le voir.
    let plan = reconcilier(&b, RACINE, None, &lus, &HashMap::new());
    assert_eq!(
        ids(&plan
            .suivis
            .iter()
            .map(|s| s.noeud.clone())
            .collect::<Vec<_>>()),
        ["p"]
    );
}

/// Un seul sens : un texte qu'il a récrit se détache de son fichier, pour toujours.
#[test]
fn test_un_texte_se_relit_sauf_s_il_l_a_recrit() {
    let pose = "### b.md\n\nAvant";
    let mut b = Board::new("miroir", "d");
    b.annotations = vec![texte("t", "b.md", pose)];
    let avant = vus(&[("b.md", vu(5, 1, Some(empreinte(pose))))]);
    let lus = [texte_lu("b.md", 6, 2)];
    let frais = HashMap::from([(lus[0].chemin.clone(), empreinte("### b.md\n\nAprès"))]);
    let plan = reconcilier(&b, RACINE, Some(&avant), &lus, &frais);
    assert_eq!(plan.suivis.len(), 1);
    assert!(
        plan.suivis[0].relire,
        "le fichier a changé, le nœud est resté le sien"
    );
    assert_eq!(
        plan.vus[&lus[0].chemin].empreinte,
        Some(frais[&lus[0].chemin])
    );

    b.annotations = vec![texte("t", "b.md", "### b.md\n\nRécrit de sa main")];
    let plan = reconcilier(&b, RACINE, Some(&avant), &lus, &frais);
    assert!(plan.est_vide(), "récrit : le disque ne le touche plus");
    assert_eq!(
        plan.vus[&lus[0].chemin].empreinte,
        Some(empreinte(pose)),
        "toujours détaché"
    );
}

/// Sans mémoire, un texte qui n'est plus ce que son fichier donne est tenu pour sien.
#[test]
fn test_sans_memoire_un_texte_different_est_a_lui() {
    let mut b = Board::new("miroir", "d");
    b.annotations = vec![texte("t", "b.md", "le sien")];
    let lus = [texte_lu("b.md", 6, 2)];
    let frais = HashMap::from([(lus[0].chemin.clone(), empreinte("le fichier"))]);
    let plan = reconcilier(&b, RACINE, None, &lus, &frais);
    assert!(plan.est_vide());
    // Le fichier changé ensuite ne le touche pas davantage.
    let encore = [texte_lu("b.md", 9, 3)];
    let frais = HashMap::from([(encore[0].chemin.clone(), empreinte("autre chose"))]);
    assert!(reconcilier(&b, RACINE, Some(&plan.vus), &encore, &frais).est_vide());
}

#[test]
fn test_un_fichier_renomme_garde_son_noeud() {
    let mut b = Board::new("miroir", "d");
    b.images = vec![photo("p", "a.png", 10)];
    let avant = vus(&[("a.png", vu(10, 1, None))]);
    let lus = [photo_lue("rendu final.png", 10, 1)];
    let plan = reconcilier(&b, RACINE, Some(&avant), &lus, &HashMap::new());
    assert_eq!(plan.suivis.len(), 1, "{plan:?}");
    assert_eq!(plan.suivis[0].noeud, Noeud::Photo("p".into()));
    assert_eq!(plan.suivis[0].entree.nom, "rendu final.png");
    assert!(plan.nouveaux.is_empty() && plan.partis.is_empty());
    assert_eq!(plan.vus, vus(&[("rendu final.png", vu(10, 1, None))]));
}

/// Deux candidats pareils : la loi ne devine pas. Le nœud part, les deux fichiers entrent.
#[test]
fn test_un_renommage_ambigu_ne_se_devine_pas() {
    let mut b = Board::new("miroir", "d");
    b.images = vec![photo("p", "a.png", 10)];
    let avant = vus(&[("a.png", vu(10, 1, None))]);
    let lus = [photo_lue("x.png", 10, 1), photo_lue("y.png", 10, 1)];
    let plan = reconcilier(&b, RACINE, Some(&avant), &lus, &HashMap::new());
    assert!(plan.suivis.is_empty());
    assert_eq!(noms(&plan.nouveaux), ["x.png", "y.png"]);
    assert_eq!(ids(&plan.partis), ["p"]);
    // Une autre extension n'est pas un renommage non plus.
    let plan = reconcilier(
        &b,
        RACINE,
        Some(&avant),
        &[photo_lue("a.jpg", 10, 1)],
        &HashMap::new(),
    );
    assert!(plan.suivis.is_empty() && plan.partis.len() == 1);
}

/// Un fichier disparu emporte son nœud, sauf ce que sa main y a mis.
#[test]
fn test_ce_que_sa_main_a_ajoute_ne_se_perd_pas_avec_le_fichier() {
    let pose = "### c.md\n\nposé";
    let mut b = Board::new("miroir", "d");
    let mut avec_domaine = photo("domaine", "b.png", 1);
    avec_domaine.domains = vec![DomainAssignment {
        domain_id: "eau".into(),
        weight: 1.0,
    }];
    let mut fleche = Annotation::arrow("f", 0.0, 0.0, 1.0, 1.0);
    if let Annotation::Arrow { source_id, .. } = &mut fleche {
        *source_id = Some("relie".into());
    }
    let mut tuile = Annotation::sticky("tuile", 0.0, 0.0, "renommée à la main");
    if let Annotation::Sticky { source_file, .. } = &mut tuile {
        *source_file = Some(format!("{RACINE}/e.zip"));
    }
    b.images = vec![
        photo("seule", "a.png", 1),
        avec_domaine,
        photo("relie", "d.png", 1),
    ];
    b.annotations = vec![
        texte("recrit", "c.md", "à lui"),
        fleche,
        tuile,
        texte("pose", "g.md", pose),
    ];
    let avant = vus(&[
        ("a.png", vu(1, 1, None)),
        ("b.png", vu(1, 1, None)),
        ("c.md", vu(1, 1, Some(empreinte(pose)))),
        ("d.png", vu(1, 1, None)),
        ("e.zip", vu(1, 1, None)),
        ("g.md", vu(1, 1, Some(empreinte(pose)))),
    ]);
    let plan = reconcilier(&b, RACINE, Some(&avant), &[], &HashMap::new());
    assert_eq!(ids(&plan.partis), ["seule", "pose"]);
    assert_eq!(
        plan.vus.len(),
        4,
        "les restés s'en souviennent : {:?}",
        plan.vus
    );
}

/// Ses textes, une photo collée là, une photo venue d'ailleurs : à lui, la loi n'y touche jamais.
#[test]
fn test_ce_qui_ne_vient_pas_de_ce_dossier_est_a_lui() {
    let mut b = Board::new("miroir", "d");
    let mut collee = BoardImage::new("collee", 0.0, 0.0, 1.0, 1.0);
    collee.src = Some("sha256:abcd".into());
    let mut ailleurs = BoardImage::new("ailleurs", 0.0, 0.0, 1.0, 1.0);
    ailleurs.src = Some("/autre/photo.png".into());
    b.images = vec![collee, ailleurs];
    b.annotations = vec![Annotation::text("note", 0.0, 0.0, "à moi")];
    let plan = reconcilier(&b, RACINE, Some(&Vus::new()), &[], &HashMap::new());
    assert!(plan.est_vide(), "{plan:?}");
}

/// Un sous-dossier déjà lu reste (son contenu peut porter son travail) ; un jamais lu part.
#[test]
fn test_un_sous_dossier_disparu_ne_part_que_s_il_n_a_jamais_ete_lu() {
    let mut b = Board::new("miroir", "d");
    let dossier = |id: &str, nom: &str, attend: bool| {
        let mut f = CanvasFolder::new(id, nom, format!("{id}-tableau"));
        let mut s = crate::miroir_disque::source_en_attente(&format!("{RACINE}/{nom}"), None);
        s.pending_scan = attend;
        f.mirror_source = Some(s);
        f
    };
    b.folders = vec![
        dossier("lu", "lu", false),
        dossier("jamais", "jamais", true),
    ];
    let plan = reconcilier(&b, RACINE, None, &[], &HashMap::new());
    assert_eq!(ids(&plan.partis), ["jamais"]);
}

#[test]
fn test_ce_qu_un_miroir_a_vu_s_ecrit_et_se_relit() {
    let mut v = vus(&[
        ("a.png", vu(10, -3, None)),
        ("b\tc.md", vu(5, 2, Some(u64::MAX))),
    ]);
    assert_eq!(lire_les_vus(&ecrire_les_vus(&v)), v);
    v.insert("/d/saut\nde ligne".into(), vu(1, 1, None));
    let relu = lire_les_vus(&ecrire_les_vus(&v));
    assert_eq!(relu.len(), 2, "un chemin à saut de ligne ne s'écrit pas");
    assert!(lire_les_vus("n'importe quoi\n1\t2\tzz\t/x\n").is_empty());
}
