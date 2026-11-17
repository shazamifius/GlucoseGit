//! **EXPORT-HTML-1 — la page autonome** (fiche 115) : l'export HTML de Glucose Tauri
//! (`toHtml.ts`) — l'image vectorielle de l'export ([`super::svg`]) dans **une seule page** qu'on
//! ouvre dans n'importe quel navigateur, sans rien installer ; les photos y portent leurs octets.
//!
//! * **On la parcourt** : glisser la déplace, la molette zoome autour de la main — un tiers d'octave
//!   par cran, la loi du canevas (`pan_zoom::geste`) —, et **deux doigts pincent** : sur un iPad,
//!   Safari est aujourd'hui le seul Glucose qu'on puisse lui montrer (ÉTAT § 5). Tauri ne savait que
//!   la souris ; le pincement de Safari zoomait la page entière, barre comprise.
//! * **Une flèche se lit** : un clic (ou un toucher) sur elle ouvre un panneau — sa relation, son
//!   étiquette, sa description longue ; chaque flèche du SVG est un groupe `class="fleche"` qui
//!   porte son identifiant (`renderer::pass::svg`).
//! * **Aucune dépendance** : le lecteur est quelques dizaines de lignes de JavaScript écrites ici,
//!   dans la page ; rien ne se télécharge à l'ouverture.

use crate::agents::json::Json;
use crate::app::GlucoseApp;
use glucose_core::types::{Annotation, Board};

/// **Ce que dit chaque flèche qui dit quelque chose** — sa relation en mots, son étiquette, sa
/// description longue — pour le panneau de la page ; les autres n'ont rien à montrer.
pub fn ce_que_disent_les_fleches(tableau: &Board) -> Json {
    let fleches = tableau.fleches().filter_map(|a| {
        let Annotation::Arrow {
            text,
            long_text,
            predicate,
            ..
        } = a
        else {
            return None;
        };
        let texte = |t: &Option<String>| t.as_deref().unwrap_or("").trim().to_string();
        let (etiquette, longue) = (texte(text), texte(long_text));
        let relation = predicate.map_or("", |p| p.label());
        if etiquette.is_empty() && longue.is_empty() && relation.is_empty() {
            return None;
        }
        Some(Json::objet([
            ("id", Json::texte(a.id())),
            ("relation", Json::texte(relation)),
            ("etiquette", Json::texte(etiquette)),
            ("longue", Json::texte(longue)),
        ]))
    });
    Json::Liste(fleches.collect())
}

/// Un texte posé dans du HTML.
fn echapper(texte: &str) -> String {
    texte
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// **La page** : son titre, l'image vectorielle, et ce que disent les flèches. Le JSON entre dans
/// un `<script>` : chaque `<` y devient `<`, et aucun texte ne peut le refermer. Les morceaux
/// se cousent **dans l'ordre des marques** du gabarit, sans relire ce qui est déjà posé : un nom de
/// document ou un texte du canevas qui contiendrait une marque reste un texte.
pub fn page(titre: &str, svg: &str, fleches: &Json) -> String {
    let titre = echapper(titre);
    let donnees = fleches.ecrire().replace('<', "\\u003c");
    let morceaux = [
        ("{{TITRE}}", titre.as_str()),
        ("{{TITRE}}", titre.as_str()),
        ("{{SVG}}", svg),
        ("{{FLECHES}}", donnees.as_str()),
    ];
    let mut sortie = String::with_capacity(GABARIT.len() + svg.len() + donnees.len());
    let mut reste = GABARIT;
    for (marque, valeur) in morceaux {
        let Some((avant, apres)) = reste.split_once(marque) else {
            break;
        };
        sortie.push_str(avant);
        sortie.push_str(valeur);
        reste = apres;
    }
    sortie.push_str(reste);
    sortie
}

impl GlucoseApp {
    /// **La page de ce qu'on exporte** — `None` pour un tableau vide : l'image vectorielle de
    /// l'export, et ce que disent les flèches de la même scène.
    pub fn rendre_la_page(&self) -> Option<String> {
        let scene = super::png::preparer(&self.store)?;
        let svg = self.rendre_le_svg()?;
        let fleches = scene
            .store
            .active_board()
            .map_or(Json::Liste(Vec::new()), ce_que_disent_les_fleches);
        Some(page(&self.document_label(), &svg, &fleches))
    }
}

/// Le gabarit : la barre (le titre, « Recadrer », ce que la main peut faire), la scène, le
/// panneau d'une flèche, et le lecteur. Les couleurs sont celles de la chrome de Glucose
/// (`style.md`) : monochrome.
const GABARIT: &str = include_str!("html/page.html");

#[cfg(test)]
mod tests;
