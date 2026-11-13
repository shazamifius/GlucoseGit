//! **Un nœud, tel qu'un agent le lit en JSON** : sa sorte, sa boîte, ce qu'il dit, ce qu'il porte —
//! et ce que l'humain voit de lui sans le lire : la parenté de son halo (PARENTE-3), sa date
//! (DATE-1). Sorti d'`executer.rs` quand il a atteint ses 600 lignes (fiche 114).

use super::domaines;
use crate::agents::json::Json;
use crate::agents::lecture::{empreinte, premiere_phrase, Rendu};
use glucose_core::geometry::Rect;
use glucose_core::store::agents::Decrit;
use glucose_core::timeline::format_anchor;
use glucose_core::types::TemporalAnchor;

pub(super) fn boite_en_json(b: Rect) -> Json {
    Json::objet([
        ("x", coordonnee(b.left)),
        ("y", coordonnee(b.top)),
        ("largeur", coordonnee(b.width)),
        ("hauteur", coordonnee(b.height)),
    ])
}

/// **COORD-1 — une coordonnée, au plus court sans rien perdre** (fiche 90) : le document place tout à
/// 1/2⁸ d'unité près (`glucose_core::fixed`) ; avec d décimales, 10^d ≥ 2⁸, la valeur qu'un agent
/// relit et renvoie retombe sur le même point. d = 3, déduit et non choisi. Une hauteur mesurée en
/// `f32` s'écrivait « 51.599998474121094 » : des milliers de caractères perdus sur un grand tableau.
pub(super) fn coordonnee(v: f64) -> Json {
    let decimales = (f64::from(glucose_core::fixed::FRAC_BITS) * 2f64.log10()).ceil();
    let pas = 10f64.powf(decimales);
    Json::Nombre((v * pas).round() / pas)
}

/// **PARENTE-3 — ce que l'IA de Glucose dit d'un texte**, comme l'humain le voit à son halo (fiche
/// 106 ; VISION § 1 : *« ce qu'un humain voit, une IA doit pouvoir le lire »*) : le numéro de sa
/// famille — des textes de la même famille se ressemblent sûrement, leur halo a la même couleur —,
/// rien s'il est seul ; et la sûreté qu'il a un parent, de 0 à 1, au centième. D'une flèche entre
/// deux textes (PARENTE-4, fiche 107) : aucune famille, et la sûreté que ses deux textes parlent de
/// la même chose — ce que l'humain voit à sa force quand les liens abondent.
pub(super) fn parente_en_json(avis: Option<&crate::parente::AvisDUnNoeud>, noeud: &mut Json) {
    let (Some(a), Json::Objet(champs)) = (avis, noeud) else {
        return;
    };
    let famille = a.famille.map_or(Json::Nul, |f| Json::Nombre(f as f64));
    let surete = Json::Nombre((f64::from(a.surete) * 100.0).round() / 100.0);
    let parente = Json::objet([("famille", famille), ("surete", surete)]);
    champs.push(("parente".to_string(), parente));
}

/// Un nœud pour l'agent : son texte entier, ou sa première phrase et sa taille (le sommaire), ou
/// le passage trouvé.
pub(super) fn decrit_en_json(d: &Decrit, rendu: &Rendu) -> Json {
    let mut champs = vec![
        ("id", Json::from(d.id.as_str())),
        ("sorte", Json::from(d.sorte.nom())),
        ("boite", boite_en_json(d.boite)),
    ];
    if let Some(t) = d.texte.as_deref() {
        champs.push(("empreinte", Json::from(empreinte(t).as_str())));
    }
    if let (Rendu::Sommaire | Rendu::Passage(_), Some(t)) = (rendu, d.texte.as_deref()) {
        champs.push(("taille", Json::Nombre(t.chars().count() as f64)));
        if let Some(p) = premiere_phrase(t) {
            champs.push(("debut", Json::from(p)));
        }
    }
    if let Rendu::Passage(p) = rendu {
        champs.push(("passage", Json::from(p.as_str())));
    }
    let entier = (*rendu == Rendu::Entier)
        .then_some(d.texte.as_deref())
        .flatten();
    let facultatifs = [
        ("texte", entier),
        ("membrane", d.membrane.as_deref()),
        ("source", d.source.as_deref()),
        ("cible", d.cible.as_deref()),
        ("relation", d.relation.map(|r| r.as_str())),
        ("operateur", d.operateur.map(|o| o.id())),
        ("couleur", d.couleur.as_deref()),
        ("adresse", d.adresse.as_deref()),
    ];
    champs.extend(
        facultatifs
            .into_iter()
            .filter_map(|(cle, v)| v.map(|v| (cle, Json::from(v)))),
    );
    if !d.domaines.is_empty() {
        champs.push(("domaines", domaines::domaines_en_json(&d.domaines)));
    }
    if let Some(date) = &d.date {
        champs.push(("date", date_en_json(date)));
    }
    Json::objet(champs)
}

/// **DATE-1 — la date d'un nœud** : le mot que sa pastille montre à l'humain (« 1789 – 1799 »,
/// « Renaissance », « 12.3 ka »), et ses années exactes — le mot arrondit ce qui précède -10 000
/// (« 12.3 ka » pour -12 345). Sans an zéro : « -500 » est « 500 av. J.-C. ».
fn date_en_json(date: &TemporalAnchor) -> Json {
    Json::objet([
        ("texte", Json::from(format_anchor(date).as_str())),
        ("debut", Json::Nombre(date.start as f64)),
        ("fin", Json::Nombre(date.end as f64)),
    ])
}
