//! La fenêtre de l'éditeur du texte lié : ses boutons selon l'étape, et qu'elle se vise.

use super::*;
use crate::renderer::math::MathRenderer;
use crate::typography::Typography;

const TEXTE: &str = "bonjours\ntest\ntest\nbonjours";

fn ancrage(etape: Etape, choix: usize, deux_cartes: bool) -> Ancrage {
    let ancres: Vec<_> = [(0, 8), (19, 27)]
        .into_iter()
        .take(choix)
        .filter_map(|(a, b)| glucose_core::text_anchors::create_anchor(TEXTE, a, b))
        .collect();
    Ancrage {
        fleche: "f".into(),
        etape,
        cartes: (Some("a".into()), deux_cartes.then(|| "b".into())),
        source: ancres,
        cible: Vec::new(),
        glisse: None,
        defilement: 0.0,
    }
}

fn fenetre(a: &Ancrage, texte: &str) -> Fenetre {
    layout_ancrage(
        a,
        texte,
        (&Typography::new(), &MathRenderer::new()),
        (1440.0, 900.0),
        1.5,
    )
}

fn libelles(a: &Ancrage) -> Vec<&'static str> {
    fenetre(a, TEXTE)
        .boutons
        .iter()
        .map(|b| b.libelle)
        .collect()
}

/// **Les boutons disent l'étape** : « Passer » tant que rien n'est choisi, « Valider → Cible »
/// ensuite, « Terminer » à la dernière étape — le principal à droite, comme chez Tauri.
#[test]
fn test_ancre_ux_les_boutons_disent_l_etape() {
    assert_eq!(
        libelles(&ancrage(Etape::Source, 0, true)),
        ["Annuler", "Passer →"]
    );
    assert_eq!(
        libelles(&ancrage(Etape::Source, 1, true)),
        ["Annuler", "Valider → Cible"]
    );
    assert_eq!(
        libelles(&ancrage(Etape::Source, 1, false)),
        ["Annuler", "Terminer"],
        "sans carte au bout, la source est la dernière étape"
    );
    assert_eq!(
        libelles(&ancrage(Etape::Cible, 0, true)),
        ["Annuler", "Terminer"]
    );
}

/// **Tout tient dans la fenêtre, et chaque bouton se vise là où il est** : les boutons,
/// « Effacer », la croix de chaque puce.
#[test]
fn test_ancre_ux_la_fenetre_se_vise() {
    let a = ancrage(Etape::Source, 2, true);
    let f = fenetre(&a, TEXTE);
    let dedans = |r: fenetre::Rect4| {
        r.0 >= f.rect.0
            && r.1 >= f.rect.1
            && r.0 + r.2 <= f.rect.0 + f.rect.2 + 0.5
            && r.1 + r.3 <= f.rect.1 + f.rect.3 + 0.5
    };
    let milieu = |r: fenetre::Rect4| (r.0 + r.2 / 2.0, r.1 + r.3 / 2.0);
    for b in &f.boutons {
        assert!(dedans(b.rect), "{} dans la fenêtre", b.libelle);
        let (x, y) = milieu(b.rect);
        assert_eq!(f.action_sous(x, y), Some(b.action));
    }
    let choisi = f.choisi.as_ref().expect("deux passages choisis");
    assert!(dedans(choisi.rect));
    assert_eq!(choisi.titre.0, "Sélectionné (2) :");
    let (x, y) = milieu(choisi.effacer);
    assert_eq!(f.action_sous(x, y), Some(ActionDAncrage::Effacer));
    assert_eq!(choisi.puces.len(), 2);
    for p in &choisi.puces {
        assert!(dedans(p.rect));
        let (x, y) = milieu(p.croix);
        assert_eq!(f.action_sous(x, y), Some(ActionDAncrage::Retirer(p.rang)));
        let (x, y) = (p.rect.0 + 4.0, p.rect.1 + p.rect.3 / 2.0);
        assert_eq!(
            f.action_sous(x, y),
            None,
            "le texte d'une puce ne la retire pas"
        );
    }
    assert!(dedans(f.zone.rect));
}

/// **Une unité du monde, un point** : la zone pose le texte à l'échelle de l'interface, et
/// ramène un point de l'écran dans la carte virtuelle, défilement compris.
#[test]
fn test_ancre_ux_la_zone_est_a_l_echelle_d_un_point() {
    let mut a = ancrage(Etape::Source, 0, true);
    let long = "une ligne\n".repeat(60);
    let f = fenetre(&a, &long);
    let z = f.zone;
    assert_eq!(z.s, 1.5);
    assert!(
        z.defilement_max() > 0.0,
        "soixante lignes débordent de la zone"
    );
    assert_eq!(z.vers_le_monde((z.rect.0, z.rect.1)), (0.0, 0.0));
    a.defilement = 1e9;
    let z = fenetre(&a, &long).zone;
    assert_eq!(
        z.defilement,
        z.defilement_max(),
        "le défilement s'arrête au bas du texte"
    );
    let (_, y) = z.vers_le_monde((z.rect.0, z.rect.1));
    assert_eq!(y, z.defilement);
}

/// **Le numéro allumé est celui de l'étape en cours** ; une flèche dont un seul bout est une
/// carte n'en montre aucun.
#[test]
fn test_ancre_ux_le_numero_allume_est_l_etape() {
    let allumes = |a: &Ancrage| -> Vec<bool> {
        fenetre(a, TEXTE)
            .entete
            .etapes
            .iter()
            .map(|e| e.2)
            .collect()
    };
    assert_eq!(allumes(&ancrage(Etape::Source, 0, true)), [true, false]);
    assert_eq!(allumes(&ancrage(Etape::Cible, 0, true)), [false, true]);
    assert!(allumes(&ancrage(Etape::Source, 0, false)).is_empty());
}
