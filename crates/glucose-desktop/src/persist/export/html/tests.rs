//! EXPORT-HTML-1 : la page autonome — sa forme, la sûreté de ce qu'elle porte, et l'export par le
//! vrai chemin.

use super::*;
use crate::bench::vitrine::vitrine_de_l_export;
use crate::epreuve::Dossier;
use glucose_core::types::ArrowPredicate;

const SVG: &str = r#"<svg width="10" height="10"><g class="fleche" data-id="f"></g></svg>"#;

/// Ce que dit une flèche, comme [`ce_que_disent_les_fleches`] l'écrit.
fn dite(longue: &str) -> Json {
    Json::Liste(vec![Json::objet([
        ("id", Json::texte("f")),
        ("relation", Json::texte("illustre")),
        ("etiquette", Json::texte("borne")),
        ("longue", Json::texte(longue)),
    ])])
}

/// **La page porte le SVG une fois, le titre deux fois** (l'onglet et la barre), échappé ; une
/// description qui contient `</script>` ne referme pas le lecteur ; aucune marque ne reste.
#[test]
fn export_html_1_la_page_et_ce_qu_elle_porte() {
    let p = page(
        "La carte <de> l'eau",
        SVG,
        &dite("</script><script>alert(1)</script>"),
    );
    assert!(p.starts_with("<!DOCTYPE html>"));
    assert_eq!(p.matches(SVG).count(), 1, "le SVG, une fois");
    assert_eq!(
        p.matches("La carte &lt;de&gt; l'eau").count(),
        2,
        "le titre échappé"
    );
    assert_eq!(
        p.matches("</script>").count(),
        1,
        "un seul script, le lecteur"
    );
    assert!(p.contains("\\u003c/script>"), "la description, neutralisée");
    assert!(!p.contains("{{"), "aucune marque oubliée");
}

/// **Un titre qui porte une marque reste un titre** : les morceaux se cousent dans l'ordre, sans
/// relire ce qui est posé — le SVG n'entre pas dans le titre.
#[test]
fn export_html_1_une_marque_dans_un_titre_reste_un_texte() {
    let p = page("{{SVG}} et {{FLECHES}}", SVG, &dite(""));
    assert_eq!(p.matches(SVG).count(), 1);
    assert_eq!(p.matches("{{SVG}} et {{FLECHES}}").count(), 2);
}

/// **Seules les flèches qui disent quelque chose** entrent dans le panneau : leur relation en mots,
/// leur étiquette et leur description, sans blancs autour ; une flèche muette n'y est pas.
#[test]
fn export_html_1_ce_que_disent_les_fleches() {
    let mut tableau = glucose_core::types::Board::new("t", "t");
    let mut parle = Annotation::arrow("parle", 0.0, 0.0, 10.0, 0.0);
    if let Annotation::Arrow {
        long_text,
        predicate,
        ..
    } = &mut parle
    {
        *long_text = Some("  L'eau **mouille**.  ".into());
        *predicate = Some(ArrowPredicate::Illustre);
    }
    tableau.annotations.push(parle);
    tableau
        .annotations
        .push(Annotation::arrow("muette", 0.0, 0.0, 10.0, 0.0));
    let dites = ce_que_disent_les_fleches(&tableau).ecrire();
    assert_eq!(
        dites,
        r#"[{"id":"parle","relation":"illustre","etiquette":"","longue":"L'eau **mouille**."}]"#
    );
}

/// **Par le vrai chemin** : exporter la vitrine sous `carte.html` écrit une page qui porte l'image
/// vectorielle, chaque flèche en groupe nommé, et ce que dit la flèche de la vitrine ; le message
/// nomme le format.
#[test]
fn export_html_1_par_le_vrai_chemin() {
    let dossier = Dossier::nouveau("export-html");
    let mut app = vitrine_de_l_export(&dossier);
    let chemin = dossier.join("carte.html");
    app.exporter_vers(chemin.clone());
    let p = std::fs::read_to_string(&chemin).expect("la page écrite");
    assert!(
        p.starts_with("<!DOCTYPE html>") && p.contains("<svg"),
        "la page et son image"
    );
    assert!(
        p.contains(r#"<g class="fleche" data-id="s-borne">"#),
        "la flèche, nommée"
    );
    assert!(
        p.contains(r#""relation":"illustre","etiquette":"borne""#),
        "ce qu'elle dit"
    );
    let dit = app.ui.current_toast.as_ref().map(|t| t.message.clone());
    assert!(
        dit.is_some_and(|m| m.contains("Page web")),
        "le message nomme le format"
    );
}
