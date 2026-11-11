//! PARENTE-3 : une IA lit la parenté que l'IA de Glucose donne aux textes — ce que l'humain voit à
//! leur halo (fiche 106).

use super::{appeler, board};
use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::types::Annotation;

/// Cinq textes de musique, et des textes qui ne parlent de rien de commun.
const TEXTES: [&str; 11] = [
    "Le piano est un instrument de musique à cordes frappées.",
    "Une partition note la mélodie, le rythme et l'harmonie d'un morceau.",
    "Le violon et le violoncelle jouent dans l'orchestre symphonique.",
    "La guitare électrique a transformé la musique rock.",
    "Mozart a composé ses premières symphonies dans son enfance.",
    "Le chat dort sur le canapé tout l'après-midi.",
    "Il faut acheter du pain et des œufs demain matin.",
    "Le train pour Lyon part à huit heures.",
    "La Révolution française commence en 1789.",
    "Le béton armé résiste mieux à la traction.",
    "Le vaccin entraîne le système immunitaire.",
];

/// La parenté d'un nœud lu : sa famille et sa sûreté.
fn parente(lu: &Json, id: &str) -> (Option<f64>, f64) {
    let noeud = lu
        .get("noeuds")
        .and_then(Json::as_liste)
        .and_then(|l| {
            l.iter()
                .find(|n| n.get("id").and_then(Json::as_str) == Some(id))
        })
        .unwrap_or_else(|| panic!("{id} est lu"));
    let p = noeud
        .get("parente")
        .unwrap_or_else(|| panic!("{id} dit sa parenté : {}", noeud.ecrire()));
    let famille = p.get("famille").and_then(Json::as_f64);
    let surete = p.get("surete").and_then(Json::as_f64).expect("une sûreté");
    (famille, surete)
}

/// **PARENTE-3 — un agent lit la parenté** : les textes de musique dont l'IA est sûre ont la même
/// famille ; les textes sans parent n'en ont pas. **La preuve à l'envers** : avant que Glucose ait
/// lu le tableau, aucun texte ne dit de parenté, ni un autre tableau qui porterait les mêmes
/// identifiants — l'agent ne reçoit jamais un avis inventé, ou celui d'un voisin.
#[test]
fn parente_3_un_agent_lit_la_parente() {
    let mut app = GlucoseApp::new();
    app.ui.current_toast = None;
    let b = board(&app);
    if let Some(t) = app.store.active_board_mut() {
        t.annotations.clear();
    }
    for (k, t) in TEXTES.iter().enumerate() {
        let texte = Annotation::text(format!("t{k}"), 0.0, 200.0 * k as f64, *t);
        app.store.add_annotation(&b, texte);
    }
    let avant = appeler(&mut app, "glucose_lire", "{}");
    assert!(
        avant.ecrire().find("parente").is_none(),
        "{}",
        avant.ecrire()
    );
    app.une_image_sans_fenetre((800, 600));
    let lu = appeler(&mut app, "glucose_lire", "{}");
    let musique: Vec<(Option<f64>, f64)> = (0..5).map(|k| parente(&lu, &format!("t{k}"))).collect();
    let sures: Vec<Option<f64>> = musique
        .iter()
        .filter(|(_, s)| *s > 0.5)
        .map(|(f, _)| *f)
        .collect();
    assert!(
        sures.len() >= 2,
        "deux textes de musique sûrs : {musique:?}"
    );
    assert!(
        sures.iter().all(|f| f.is_some() && *f == sures[0]),
        "{musique:?}"
    );
    let seuls = (5..11).filter(|k| parente(&lu, &format!("t{k}")).0.is_none());
    assert!(seuls.count() >= 4, "{}", lu.ecrire());
    let autre = appeler(&mut app, "glucose_ouvrir_tableau", r#"{"nom":"autre"}"#);
    let autre = autre
        .get("tableau")
        .and_then(Json::as_str)
        .expect("un tableau")
        .to_string();
    app.store
        .add_annotation(&autre, Annotation::text("t0", 0.0, 0.0, TEXTES[0]));
    let ailleurs = appeler(
        &mut app,
        "glucose_lire",
        &format!(r#"{{"tableau":"{autre}"}}"#),
    );
    assert!(
        ailleurs.ecrire().find("parente").is_none(),
        "{}",
        ailleurs.ecrire()
    );
}
