//! EXIF-1 de bout en bout : une vraie photo marquée, décodée par l'atelier, et le nœud posé quand
//! elle arrivait couchée.

use super::*;
use crate::persist::objets::Source;
use glucose_core::types::BoardImage;
use std::sync::Arc;

const CLE: &str = "portrait-exif-6";

/// Une application sans rien dessus, où un nœud a été posé avant EXIF-1 sur une photo de 40 × 30
/// pixels couchés, marquée « 6 » : sa boîte et ses dimensions d'origine en paysage.
fn app_et_noeud_couche() -> GlucoseApp {
    let mut app = GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
        b.images.clear();
    }
    let jpeg = crate::renderer::photo::orientation::tests::jpeg_marque((40, 30), 6);
    app.disque
        .objets
        .poser(CLE, Source::Memoire(Arc::new(jpeg)));
    let board = app.store.project.active_board_id.clone();
    let mut img = BoardImage::new("i", 0.0, 0.0, 400.0, 300.0);
    img.src = Some(CLE.into());
    (img.original_width, img.original_height) = (40.0, 30.0);
    app.store.add_image(&board, img);
    app.store.journal.clear();
    app
}

/// La photo est décodée — debout — et entre dans le magasin.
fn arrivee(app: &mut GlucoseApp) {
    let magasin = &mut app.renderer.magasin;
    magasin.ouvrir();
    magasin.reclamer(CLE, 400.0);
    magasin.fermer();
    magasin.attendre_le_chantier();
}

fn taille(app: &GlucoseApp) -> (f64, f64) {
    let board = &app.store.project.active_board_id;
    let i = app.store.image(board, "i").expect("là");
    (i.width, i.height)
}

/// **La photo arrive debout, son nœud couché se redresse avant l'image suivante** — le réveil
/// repasse pour lui, puis se tait ; un `Ctrl+Z` le rend tel qu'il était.
#[test]
fn test_le_noeud_couche_se_redresse_a_l_arrivee_de_sa_photo() {
    let mut app = app_et_noeud_couche();
    // Le mot d'accueil réveillerait l'application de lui-même.
    app.ui.current_toast = None;
    app.poursuivre_les_attentes();
    arrivee(&mut app);
    assert_eq!(taille(&app), (400.0, 300.0), "pas encore regardé");
    assert!(app.prochain_reveil().is_some(), "le réveil repasse");
    app.poursuivre_les_attentes();
    assert_eq!(taille(&app), (300.0, 400.0), "debout");
    assert!(!app.redressement_en_attente(), "puis se tait");
    assert!(app.store.undo());
    assert_eq!(taille(&app), (400.0, 300.0), "défait");
}

/// **Un geste ouvert le fait attendre, sans réveiller la boucle** — une frappe de texte peut
/// durer — ; il s'applique quand le geste se ferme.
#[test]
fn test_un_geste_ouvert_fait_attendre_le_redressement_sans_reveiller() {
    let mut app = app_et_noeud_couche();
    app.poursuivre_les_attentes();
    arrivee(&mut app);
    app.store.begin_live_edit();
    assert!(!app.redressement_en_attente(), "la boucle dort");
    app.poursuivre_les_attentes();
    assert_eq!(taille(&app), (400.0, 300.0));
    app.store.end_live_edit();
    assert!(app.redressement_en_attente());
    app.poursuivre_les_attentes();
    assert_eq!(taille(&app), (300.0, 400.0));
}

/// **Un autre document qu'on ouvre se redresse sur les photos déjà là** : aucune n'arrive de
/// nouveau, le magasin les tient déjà.
#[test]
fn test_un_autre_document_se_redresse_sur_les_photos_deja_la() {
    let mut app = app_et_noeud_couche();
    app.poursuivre_les_attentes();
    arrivee(&mut app);
    app.poursuivre_les_attentes();
    let autre = {
        let mut projet = app.store.project.clone();
        for b in &mut projet.boards {
            for i in &mut b.images {
                (i.width, i.height) = (400.0, 300.0);
                (i.original_width, i.original_height) = (40.0, 30.0);
            }
        }
        projet
    };
    app.store.load_project(autre);
    assert_eq!(taille(&app), (400.0, 300.0));
    app.poursuivre_les_attentes();
    assert_eq!(
        taille(&app),
        (300.0, 400.0),
        "redressé dans l'autre document"
    );
}
