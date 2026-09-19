//! Ce que l'export doit garantir — sans fenêtre, sans disque, sans dialogue.
//!
//! Tout ce qui se teste ici est **pur** : lire un format dans un nom, compléter ce qui manque,
//! et rendre un tableau. L'écriture elle-même est celle des enregistrements (`atomic`), déjà
//! prouvée par sa propre suite.

use super::*;
use glucose_core::types::{Annotation, Board};

fn tableau_avec_une_carte() -> Board {
    let mut board = Board::new("b1", "Essai");
    board
        .annotations
        .push(Annotation::text("a1", 10.0, 20.0, "Un titre"));
    board
}

/// L'extension **est** le choix de format, et la casse ne doit pas y changer quoi que ce soit.
#[test]
fn le_format_se_lit_dans_le_nom_quelle_que_soit_la_casse() {
    assert_eq!(
        Format::du_chemin(Path::new("carte.md")),
        Some(Format::Markdown)
    );
    assert_eq!(Format::du_chemin(Path::new("carte.svg")), Some(Format::Svg));
    assert_eq!(Format::du_chemin(Path::new("CARTE.SVG")), Some(Format::Svg));
    assert_eq!(
        Format::du_chemin(Path::new("carte.Md")),
        Some(Format::Markdown)
    );
}

/// Un nom qui n'annonce rien de connu ne doit jamais faire échouer l'export en silence.
#[test]
fn un_nom_sans_extension_connue_prend_le_premier_format() {
    assert_eq!(Format::du_chemin(Path::new("carte")), None);
    assert_eq!(Format::du_chemin(Path::new("carte.txt")), None);

    let (chemin, format) = destination(PathBuf::from("carte"));
    assert_eq!(format, Format::TOUS[0]);
    assert_eq!(chemin, PathBuf::from("carte.md"));

    // Et le chemin rendu annonce bien ce qui sera écrit : le nom ne peut pas mentir.
    assert_eq!(Format::du_chemin(&chemin), Some(format));
}

/// Un nom qui annonce un format connu se garde **tel quel** — extension comprise.
#[test]
fn un_nom_qui_annonce_un_format_le_garde() {
    let (chemin, format) = destination(PathBuf::from("dossier/carte.svg"));
    assert_eq!(format, Format::Svg);
    assert_eq!(chemin, PathBuf::from("dossier/carte.svg"));
}

/// Chaque format a son extension à lui : deux formats qui la partageraient rendraient
/// [`Format::du_chemin`] ambigu, et le premier gagnerait en silence.
#[test]
fn deux_formats_ne_partagent_jamais_une_extension() {
    for (rang, format) in Format::TOUS.into_iter().enumerate() {
        for autre in Format::TOUS.into_iter().skip(rang + 1) {
            assert_ne!(
                format.extension(),
                autre.extension(),
                "{} et {} se disputent .{}",
                format.nom(),
                autre.nom(),
                format.extension()
            );
        }
    }
}

/// Les moteurs sont branchés : chacun produit quelque chose qui porte sa signature, et le
/// contenu du tableau s'y retrouve. C'est la preuve que le module appelle bien le bon moteur.
#[test]
fn chaque_format_rend_sa_propre_forme() {
    let board = tableau_avec_une_carte();

    let markdown = Format::Markdown.rendre(&board);
    assert!(
        markdown.contains("Un titre"),
        "le markdown porte le texte de la carte : {markdown}"
    );
    assert!(
        !markdown.contains("<svg"),
        "le markdown n'est pas du svg : {markdown}"
    );

    let svg = Format::Svg.rendre(&board);
    assert!(svg.starts_with("<?xml"), "un svg commence par son prologue");
    assert!(svg.contains("<svg"), "et porte sa balise racine");
}

/// **Le chemin complet, jusqu'au disque** — tout sauf le dialogue natif.
///
/// Ce test remplace celui qui vérifiait que le bouton « Exporter » ne *prétendait* pas avoir
/// exporté. Cette garde avait un sens tant que le bouton ne faisait rien ; maintenant qu'il
/// fait, ce qu'il faut prouver est qu'il **fait**, et que l'octet arrive sur le disque.
#[test]
fn un_export_arrive_vraiment_sur_le_disque() {
    let mut app = crate::app::GlucoseApp::new();
    app.store
        .project
        .boards
        .first_mut()
        .expect("un projet neuf a un tableau")
        .annotations
        .push(Annotation::text(
            "a1",
            10.0,
            20.0,
            "Une phrase reconnaissable",
        ));

    let chemin = std::env::temp_dir().join(format!(
        "glucose-export-{}-{}.md",
        std::process::id(),
        line!()
    ));
    let octets = app
        .try_export(&chemin, Format::Markdown)
        .expect("l'export doit aboutir");

    let relu = std::fs::read_to_string(&chemin).expect("le fichier doit exister");
    // Le temporaire part avant toute assertion : un échec ne doit pas laisser de trace
    // derrière lui (fiche 17 § 5 — `coller_image` fait cette faute, ne la refaisons pas).
    let _ = std::fs::remove_file(&chemin);

    assert_eq!(relu.len(), octets, "le compte rendu dit ce qui a ete ecrit");
    assert!(
        relu.contains("Une phrase reconnaissable"),
        "le contenu du tableau traverse jusqu'au disque : {relu}"
    );
}
