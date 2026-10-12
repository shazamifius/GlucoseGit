//! **Les gabarits de tableau** — les presets de Glucose Tauri (`defaultPresets.ts`), et ce
//! qu'une zone posée en dit (PRESET-1, fiche 62).
//!
//! Un gabarit est une suite de **catégories** — référence, croquis, face, profil… — chacune
//! avec sa couleur et une phrase qui dit ce qu'on y range. Poser un gabarit sur un tableau y
//! pose une **zone** par catégorie ; une image appartient à la zone où elle est posée.
//!
//! # Une zone nomme son gabarit
//!
//! Glucose Tauri retenait le gabarit du tableau (`board.presetId`), et la zone le seul nom de sa
//! catégorie — `ref`, `final` —, que cinq gabarits sur six emploient. Ici la zone nomme les
//! deux : `charadesign/ref`. L'identifiant devient unique dans tout Glucose, une zone se décrit
//! seule, et le format du document n'a pas eu à changer.
//!
//! # Les données, telles quelles
//!
//! Les six gabarits sont ceux de Glucose Tauri, recopiés de sa source mot pour mot et couleur
//! pour couleur : la fiche 06 en fait la spécification. Glucose Rust en avait une copie à lui,
//! quatre gabarits sur six et d'autres couleurs.

use crate::types::{BoardZone, Project};

/// Une catégorie d'un gabarit : ce qu'une zone montre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Categorie {
    pub id: &'static str,
    pub nom: &'static str,
    /// `0xRRGGBB`.
    pub couleur: u32,
    pub description: &'static str,
}

/// Un gabarit : son nom, ce qu'il sert à faire, et ses catégories dans l'ordre.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gabarit {
    pub id: &'static str,
    pub nom: &'static str,
    pub description: &'static str,
    pub categories: &'static [Categorie],
}

const fn c(
    id: &'static str,
    nom: &'static str,
    couleur: u32,
    description: &'static str,
) -> Categorie {
    Categorie {
        id,
        nom,
        couleur,
        description,
    }
}

/// **Les six gabarits de Glucose Tauri.**
pub const INTEGRES: [Gabarit; 6] = [
    Gabarit {
        id: "charadesign",
        nom: "CharaDesign",
        description: "Workflow complet de création de personnage",
        categories: &[
            c(
                "ref",
                "Référence",
                0x3b82f6,
                "Images de référence, inspirations",
            ),
            c("sketch", "Sketch", 0x8b5cf6, "Croquis exploratoires"),
            c("lineart", "Lineart", 0x6366f1, "Dessin au propre"),
            c("turn_f", "Face", 0x059669, "Vue de face"),
            c("turn_3q", "3/4", 0x10b981, "Vue 3/4"),
            c("turn_s", "Profil", 0x34d399, "Vue de profil"),
            c("turn_b", "Dos", 0x6ee7b7, "Vue de dos"),
            c("pose", "Poses", 0xf59e0b, "Poses dynamiques"),
            c("detail", "Détails", 0xef4444, "Gros plans, accessoires"),
            c("final", "Final", 0xffffff, "Rendu final"),
        ],
    },
    Gabarit {
        id: "environment",
        nom: "Environment",
        description: "Conception d'environnement et décors",
        categories: &[
            c("ref", "Référence", 0x3b82f6, "Photos, références réelles"),
            c("mood", "Mood", 0x8b5cf6, "Ambiance, palette"),
            c("thumb", "Thumbnails", 0x6366f1, "Miniatures de composition"),
            c("layout", "Layout", 0x059669, "Plan et structure"),
            c("detail", "Détails", 0xf59e0b, "Textures, props"),
            c("light", "Lumière", 0xfbbf24, "Études de lumière"),
            c("final", "Final", 0xffffff, "Rendu final"),
        ],
    },
    Gabarit {
        id: "creature",
        nom: "Creature Design",
        description: "Conception de créature / monstre",
        categories: &[
            c("ref", "Référence", 0x3b82f6, "Animaux, nature, références"),
            c("silho", "Silhouette", 0x8b5cf6, "Explorations de formes"),
            c("anatomy", "Anatomie", 0xef4444, "Structure et squelette"),
            c("skin", "Texture", 0x059669, "Peau, écailles, fourrure"),
            c("turn", "Turnaround", 0x10b981, "Vues multiples"),
            c("action", "Action", 0xf59e0b, "Poses dynamiques"),
            c("final", "Final", 0xffffff, "Rendu final"),
        ],
    },
    Gabarit {
        id: "props",
        nom: "Props & Items",
        description: "Design d'objets, armes, accessoires",
        categories: &[
            c("ref", "Référence", 0x3b82f6, "Références réelles"),
            c("sketch", "Sketch", 0x8b5cf6, "Croquis"),
            c("ortho", "Orthogonal", 0x059669, "Face / profil / dessus"),
            c("detail", "Détails", 0xf59e0b, "Détails et matériaux"),
            c("final", "Final", 0xffffff, "Rendu final"),
        ],
    },
    Gabarit {
        id: "storyboard",
        nom: "Storyboard",
        description: "Plans de scène pour animation",
        categories: &[
            c("ref", "Référence", 0x3b82f6, "Style visuel, références"),
            c("plan_e", "Ensemble", 0x8b5cf6, "Plans larges (EL, TL)"),
            c("plan_m", "Moyen", 0x6366f1, "Plans moyens (PM, PA)"),
            c("plan_g", "Gros plan", 0x059669, "Gros plans (GP, TGP)"),
            c("insert", "Insert", 0xf59e0b, "Détails, inserts"),
            c("mouv", "Mouvement", 0xef4444, "Plans en mouvement"),
        ],
    },
    Gabarit {
        id: "moodboard",
        nom: "Mood Board",
        description: "Exploration libre d'ambiance et de style",
        categories: &[
            c(
                "color",
                "Couleurs",
                0xef4444,
                "Palettes, ambiances colorées",
            ),
            c("light", "Lumière", 0xfbbf24, "Études de lumière"),
            c("style", "Style", 0x8b5cf6, "Style graphique, rendu"),
            c("comp", "Compo", 0x3b82f6, "Compositions, cadrage"),
            c("texture", "Texture", 0x059669, "Matières, textures"),
            c("inspi", "Inspiration", 0x6366f1, "Coups de cœur"),
        ],
    },
];

/// Le gabarit intégré de cet identifiant.
pub fn integre(id: &str) -> Option<&'static Gabarit> {
    INTEGRES.iter().find(|g| g.id == id)
}

/// **L'identifiant d'une zone** : son gabarit et sa catégorie, `gabarit/categorie`.
pub fn identifiant(gabarit: &str, categorie: &str) -> String {
    format!("{gabarit}/{categorie}")
}

/// **Un gabarit qu'on peut choisir** — intégré, ou venu du document : son identifiant, son
/// nom, sa phrase, et ses catégories (identifiant, nom, couleur).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choix<'a> {
    pub id: &'a str,
    pub nom: &'a str,
    pub description: &'a str,
    pub categories: Vec<(&'a str, &'a str, u32)>,
}

/// **Les gabarits qu'on peut poser** : les six de Glucose Tauri, puis ceux du document — ce que
/// Tauri appelait `getAllPresets`.
pub fn a_choisir(projet: &Project) -> Vec<Choix<'_>> {
    let integres = INTEGRES.iter().map(|g| Choix {
        id: g.id,
        nom: g.nom,
        description: g.description,
        categories: g
            .categories
            .iter()
            .map(|c| (c.id, c.nom, c.couleur))
            .collect(),
    });
    let du_document = projet.presets.iter().map(|p| Choix {
        id: p.id.as_str(),
        nom: p.name.as_str(),
        description: p.description.as_str(),
        categories: p
            .slots
            .iter()
            .map(|s| {
                (
                    s.id.as_str(),
                    s.name.as_str(),
                    couleur_de(&s.color).unwrap_or(GRIS),
                )
            })
            .collect(),
    });
    integres.chain(du_document).collect()
}

/// **Le gabarit que ces zones suivent** : celui que toutes nomment, ou aucun.
pub fn suivi(zones: &[BoardZone]) -> Option<&str> {
    let premier = zones.first()?.slot_id.split_once('/')?.0;
    zones
        .iter()
        .all(|z| z.slot_id.split_once('/').is_some_and(|(g, _)| g == premier))
        .then_some(premier)
}

/// Ce qu'une zone montre : le nom de sa catégorie, sa couleur, sa phrase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Montre {
    pub nom: String,
    /// `0xRRGGBB`.
    pub couleur: u32,
    pub description: String,
}

/// **Ce que montre cette zone** : sa catégorie, cherchée dans les gabarits intégrés, puis dans
/// ceux du document. Une zone que rien ne décrit — un gabarit que ce Glucose ne connaît pas —
/// montre son identifiant, en gris : elle reste visible et déplaçable, jamais perdue.
pub fn ce_que_montre(zone: &BoardZone, projet: &Project) -> Montre {
    let (gabarit, categorie) = zone
        .slot_id
        .split_once('/')
        .unwrap_or(("", zone.slot_id.as_str()));
    if let Some(cat) =
        integre(gabarit).and_then(|g| g.categories.iter().find(|c| c.id == categorie))
    {
        return Montre {
            nom: cat.nom.to_string(),
            couleur: cat.couleur,
            description: cat.description.to_string(),
        };
    }
    let du_document = projet
        .presets
        .iter()
        .filter(|p| gabarit.is_empty() || p.id == gabarit)
        .find_map(|p| p.slots.iter().find(|s| s.id == categorie));
    match du_document {
        Some(slot) => Montre {
            nom: slot.name.clone(),
            couleur: couleur_de(&slot.color).unwrap_or(GRIS),
            description: slot.description.clone(),
        },
        None => Montre {
            nom: zone.slot_id.clone(),
            couleur: GRIS,
            description: String::new(),
        },
    }
}

/// Le gris d'une zone que rien ne décrit.
const GRIS: u32 = 0x888888;

/// `#rrggbb` en `0xRRGGBB`.
fn couleur_de(texte: &str) -> Option<u32> {
    let hex = texte.strip_prefix('#')?;
    (hex.len() == 6).then(|| u32::from_str_radix(hex, 16).ok())?
}

#[cfg(test)]
mod tests;
