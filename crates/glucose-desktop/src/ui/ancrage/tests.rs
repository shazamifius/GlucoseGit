//! Le panneau de l'éditeur d'ancres : ses boutons selon l'étape.

use super::*;

fn ancrage(etape: Etape, avec_un_choix: bool, deux_cartes: bool) -> Ancrage {
    let ancre = glucose_core::text_anchors::create_anchor("bonjours", 0, 8);
    Ancrage {
        fleche: "f".into(),
        etape,
        cartes: (Some("a".into()), deux_cartes.then(|| "b".into())),
        source: if avec_un_choix {
            ancre.into_iter().collect()
        } else {
            Vec::new()
        },
        cible: Vec::new(),
        glisse: None,
    }
}

fn libelles(a: &Ancrage) -> Vec<&'static str> {
    layout_ancrage(a, &Typography::new(), (1440.0, 900.0), 1.0)
        .boutons
        .iter()
        .map(|b| b.libelle)
        .collect()
}

/// **Les boutons disent l'étape** : « Passer » tant que rien n'est choisi, « Valider → Cible »
/// ensuite, « Terminer » à la dernière étape ; « Effacer » seulement s'il y a à effacer.
#[test]
fn test_fleche_4_les_boutons_disent_l_etape() {
    assert_eq!(
        libelles(&ancrage(Etape::Source, false, true)),
        ["Passer →", "Annuler"]
    );
    assert_eq!(
        libelles(&ancrage(Etape::Source, true, true)),
        ["Effacer", "Valider → Cible", "Annuler"]
    );
    assert_eq!(
        libelles(&ancrage(Etape::Source, true, false)),
        ["Effacer", "Terminer", "Annuler"],
        "sans carte au bout, la source est la dernière étape"
    );
    assert_eq!(
        libelles(&ancrage(Etape::Cible, false, true)),
        ["Terminer", "Annuler"]
    );
}

/// **Les boutons tiennent dans le panneau, sans se chevaucher**, et un clic y trouve le sien.
#[test]
fn test_fleche_4_le_panneau_se_vise() {
    let a = ancrage(Etape::Source, true, true);
    let p = layout_ancrage(&a, &Typography::new(), (1440.0, 900.0), 1.0);
    let (x, y, w, h) = p.rect;
    for (i, b) in p.boutons.iter().enumerate() {
        let (action, r) = (b.action, b.rect);
        assert!(r.0 >= x && r.1 >= y && r.0 + r.2 <= x + w && r.1 + r.3 <= y + h);
        assert_eq!(
            action_sous(&p, r.0 + r.2 / 2.0, r.1 + r.3 / 2.0),
            Some(action)
        );
        if let Some(suivant) = p.boutons.get(i + 1) {
            assert!(r.0 + r.2 <= suivant.rect.0);
        }
    }
}
