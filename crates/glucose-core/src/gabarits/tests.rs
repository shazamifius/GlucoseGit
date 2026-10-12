//! PRESET-1 — les gabarits de Glucose Tauri, et ce qu'une zone montre.

use super::*;
use crate::types::{Preset, PresetSlot};

/// **Les six gabarits de Glucose Tauri**, dans son ordre, avec le nombre de leurs catégories —
/// Glucose Rust en avait quatre, et CharaDesign y perdait sa dixième.
#[test]
fn test_preset_1_les_six_gabarits_de_tauri() {
    let vus: Vec<(&str, usize)> = INTEGRES
        .iter()
        .map(|g| (g.id, g.categories.len()))
        .collect();
    assert_eq!(
        vus,
        [
            ("charadesign", 10),
            ("environment", 7),
            ("creature", 7),
            ("props", 5),
            ("storyboard", 6),
            ("moodboard", 6),
        ]
    );
    let profil = integre("charadesign")
        .and_then(|g| g.categories.iter().find(|c| c.id == "turn_s"))
        .expect("le profil");
    assert_eq!((profil.nom, profil.couleur), ("Profil", 0x34d399));
}

/// **Une zone nomme son gabarit** : `ref` existe dans cinq gabarits sur six, avec des phrases
/// différentes — l'identifiant complet dit laquelle.
#[test]
fn test_preset_1_une_zone_se_decrit_seule() {
    let projet = Project::new("p");
    let zone = |id: &str| BoardZone::new(id, 0.0, 0.0, 10.0, 10.0);
    let chara = ce_que_montre(&zone("charadesign/ref"), &projet);
    let creature = ce_que_montre(&zone("creature/ref"), &projet);
    assert_eq!(chara.nom, "Référence");
    assert_eq!(chara.description, "Images de référence, inspirations");
    assert_eq!(creature.description, "Animaux, nature, références");
}

/// Un gabarit du document — venu d'un document de Glucose Tauri — se lit aussi ; une zone que
/// rien ne décrit montre son identifiant, en gris, plutôt que de disparaître.
#[test]
fn test_preset_1_un_gabarit_du_document_et_une_zone_inconnue() {
    let mut projet = Project::new("p");
    projet.presets.push(Preset {
        id: "perso".into(),
        name: "Perso".into(),
        description: String::new(),
        slots: vec![PresetSlot {
            id: "x".into(),
            name: "Croquis".into(),
            color: "#123456".into(),
            description: "à moi".into(),
            order: 0,
        }],
        is_builtin: false,
        created_at: 0,
    });
    let montre = ce_que_montre(&BoardZone::new("perso/x", 0.0, 0.0, 1.0, 1.0), &projet);
    assert_eq!((montre.nom.as_str(), montre.couleur), ("Croquis", 0x123456));

    let inconnue = ce_que_montre(&BoardZone::new("ailleurs/y", 0.0, 0.0, 1.0, 1.0), &projet);
    assert_eq!(inconnue.nom, "ailleurs/y");
    assert_eq!(inconnue.couleur, GRIS);
}
