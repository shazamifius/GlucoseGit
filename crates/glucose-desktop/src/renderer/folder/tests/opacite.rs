//! OPACITE-1 — l'opacité d'un dossier, au dessin (fiche 133) : le corps prend celle que le
//! dossier porte, le bandeau jamais plus que celle de sa naissance ; choisi, rien ne change (Q12,
//! fiche 137 : il était peint deux fois).

use super::*;

/// Le dossier de `store_avec_dossier`, à cette opacité, choisi ou non.
fn dossier_a(opacite: u8, choisi: bool) -> Store {
    let mut store = store_avec_dossier(100.0, 100.0, 300.0, 200.0);
    if let Some(f) = store
        .project
        .boards
        .first_mut()
        .and_then(|b| b.folders.first_mut())
    {
        f.color = glucose_core::types::couleur_de_dossier("#60a5fa", opacite);
    }
    if choisi {
        store.select_folder("fold-1".into(), false);
    }
    store
}

/// Le bleu d'un point du dossier, sur le fond noir : la teinte (bleu 250) fois l'opacité.
fn bleu(store: &Store, (x, y): (u32, u32)) -> u8 {
    let (pixmap, _) = encre(store);
    pixmap.pixel(x, y).expect("dans le pixmap").blue()
}

/// Un point du corps, loin du bord ; un point du bandeau, loin du titre, de l'icône et du compte.
const CORPS: (u32, u32) = (250, 250);
const BANDEAU: (u32, u32) = (300, 120);

#[test]
fn opacite_1_la_loi_du_bandeau_et_du_corps() {
    let n = f32::from(OPACITE_DE_NAISSANCE) / 255.0;
    assert_eq!(opacites(OPACITE_DE_NAISSANCE), (n, n), "la naissance");
    assert_eq!(opacites(255), (n, 1.0), "le bandeau ne monte pas");
    assert_eq!(opacites(0), (0.0, 0.0), "il descend avec le corps");
}

/// **Opaque, le corps est la teinte ; le bandeau garde son voile** — sans quoi le titre, écrit
/// dans la teinte, s'effacerait sur elle. Preuve à l'envers : peindre tout le cadre à l'opacité
/// du corps (la loi d'avant appliquée telle quelle) rendrait le bandeau aussi bleu que le corps.
#[test]
fn opacite_1_opaque_le_corps_est_la_teinte_le_bandeau_reste_voile() {
    let store = dossier_a(255, false);
    assert!(bleu(&store, CORPS) >= 249, "{}", bleu(&store, CORPS));
    assert!(bleu(&store, BANDEAU) <= 8, "{}", bleu(&store, BANDEAU));
}

/// **Le corps se compose exactement** : sur le noir, son bleu est celui de la teinte fois son
/// opacité, au demi-octet près — choisi ou non, la même (Q12, fiche 137).
#[test]
fn opacite_1_le_corps_a_l_opacite_que_le_dossier_porte() {
    for (opacite, choisi) in [
        (128, false),
        (128, true),
        (40, false),
        (OPACITE_DE_NAISSANCE, true),
    ] {
        let attendu = 250.0 * opacites(opacite).1;
        let lu = f32::from(bleu(&dossier_a(opacite, choisi), CORPS));
        assert!(
            (lu - attendu).abs() <= 1.5,
            "{opacite} choisi={choisi} : {lu} contre {attendu}"
        );
    }
}

/// À la naissance, rien ne change : le corps et le bandeau ont le même voile.
#[test]
fn opacite_1_a_la_naissance_un_seul_voile() {
    let store = dossier_a(OPACITE_DE_NAISSANCE, false);
    assert_eq!(bleu(&store, CORPS), bleu(&store, BANDEAU));
}

/// **La mini-carte dessine un dossier à sa teinte, quelle que soit son opacité** : un `#rrggbbaa`
/// lu comme une couleur inconnue y serait gris.
#[test]
fn opacite_1_la_mini_carte_lit_la_teinte() {
    let store = dossier_a(0x80, false);
    let mut pixmap = Pixmap::new(1_440, 900).expect("pixmap");
    pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
    crate::ui::minimap::render_minimap(
        &mut pixmap.as_mut(),
        &store,
        &Theme::dark(),
        (1_440.0, 900.0),
        (1.0, (0.0, 0.0)),
        &mut None,
    );
    let mut bleus = 0usize;
    for y in 760..890 {
        for x in 1_230..1_430 {
            let p = pixmap.pixel(x, y).expect("dans le pixmap");
            if p.blue() as i32 - p.red() as i32 > 30 {
                bleus += 1;
            }
        }
    }
    assert!(
        bleus > 20,
        "{bleus} pixels bleus : le contour du dossier, à sa teinte"
    );
}
