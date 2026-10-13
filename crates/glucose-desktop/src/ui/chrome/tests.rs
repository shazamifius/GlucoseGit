//! Les épreuves de CHROME-1 : l'ordre de la chrome, une fois ; ce que la carte en reçoit ; le
//! rectangle de la vue en filets.

use super::*;
use glucose_core::types::CanvasFolder;

/// Un téléphone debout, à l'échelle de son moto g54.
const TELEPHONE: (f32, f32) = (1080.0, 2400.0);

fn kit() -> (Typography, Theme) {
    (Typography::new(), Theme::dark())
}

/// Un projet où l'on est entré dans un dossier : le fil d'Ariane paraît.
fn dans_un_dossier(nom: &str) -> Store {
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    let mut f = CanvasFolder::new(format!("dossier-{nom}"), nom, String::new());
    f.x = 0.0;
    f.y = 0.0;
    store.create_folder(&board, f);
    let id = store
        .active_board()
        .and_then(|b| b.folders.last())
        .map(|f| f.id.clone())
        .expect("le dossier vient d'être créé");
    store.try_enter_folder(&id).expect("y entrer");
    store
}

/// Le rail déplié d'un téléphone.
fn au_telephone() -> UiState {
    let mut ui = UiState::new();
    ui.scale_factor = 2.75;
    ui.rail = super::super::rail::Rail {
        actif: true,
        ouvert: true,
    };
    ui
}

/// Les morceaux d'une image, par leur nom, dans l'ordre où ils viennent.
fn ordre(store: &Store, ui: &mut UiState) -> Vec<&'static str> {
    let (typo, theme) = kit();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let mut noms = Vec::new();
    parcourir(store, ui, (&typo, &theme), (TELEPHONE, dehors), |m| {
        noms.push(match m {
            Morceau::Tampon { nom, .. } => nom,
            Morceau::Cadre(_) => "cadre",
        })
    });
    noms
}

/// **L'ordre de la chrome, dit une fois** : la bande, le fil, le fond de la minimap, le
/// rectangle de la vue, le rail — qui passe sur la minimap, qu'il peut couvrir déplié.
#[test]
fn test_l_ordre_de_la_chrome() {
    let store = dans_un_dossier("Recherches");
    let mut ui = au_telephone();
    assert_eq!(
        ordre(&store, &mut ui),
        ["bande", "ariane", "minimap", "cadre", "rail"]
    );
    // À la racine, sans rail : ni fil, ni rail.
    let mut ui = UiState::new();
    assert_eq!(
        ordre(&Store::new("Projet"), &mut ui),
        ["bande", "minimap", "cadre"]
    );
    // Le mode référence : rien de permanent.
    ui.reference = true;
    assert!(ordre(&store, &mut ui).is_empty());
}

/// **Ce que la carte reçoit suit cet ordre** : sous le cadre, la bande, le fil et le fond de la
/// minimap ; par-dessus, le rail. Et leurs pixels sont ceux des caches, sans copie.
#[test]
fn test_la_carte_recoit_la_chrome_dans_l_ordre_et_sans_copie() {
    let (typo, theme) = kit();
    let store = dans_un_dossier("Recherches");
    let mut ui = au_telephone();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let confiee = ChromeConfiee::de(&store, &mut ui, (&typo, &theme), (TELEPHONE, dehors));
    let noms = |t: &[APoser]| t.iter().map(|t| t.identite.clone()).collect::<Vec<_>>();
    assert_eq!(
        noms(confiee.sous_le_cadre()),
        ["chrome:bande", "chrome:ariane", "chrome:minimap"]
    );
    assert_eq!(noms(confiee.sur_le_cadre()), ["chrome:rail"]);
    assert_eq!(confiee.cadre().len(), 5, "quatre filets et le voile");
    let rail = &confiee.sur_le_cadre()[0];
    let pixels = confiee.pixels(&rail.cle).expect("ses pixels, par sa clé");
    let cache = ui.rail_cache.as_ref().expect("le rail tenu");
    assert!(
        std::ptr::eq(pixels.data().as_ptr(), cache.pixmap.data().as_ptr()),
        "la carte lit le cache lui-même, pas une copie"
    );
    assert!(confiee.pixels("chrome:rail@0").is_none(), "une clé d'avant");
}

/// **Le fil d'Ariane change de clé quand on change de dossier**, et la garde quand rien ne
/// change : une clé qui resservirait pour d'autres pixels ferait poser l'ancien fil.
#[test]
fn test_le_fil_change_de_cle_avec_le_dossier() {
    let (typo, theme) = kit();
    let mut ui = UiState::new();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let cle = |store: &Store, ui: &mut UiState| {
        let c = ChromeConfiee::de(store, ui, (&typo, &theme), (TELEPHONE, dehors));
        let fil = c
            .sous_le_cadre()
            .iter()
            .find(|t| t.identite == "chrome:ariane");
        fil.map(|t| t.cle.clone()).expect("le fil")
    };
    let recherches = dans_un_dossier("Recherches");
    let premiere = cle(&recherches, &mut ui);
    assert_eq!(premiere, cle(&recherches, &mut ui), "rien n'a changé");
    let sources = dans_un_dossier("Sources");
    let seconde = cle(&sources, &mut ui);
    assert_ne!(premiere, seconde, "un autre dossier, la même clé");
    // Revenir au premier refait le fil : sa clé ne redevient jamais une ancienne.
    assert_ne!(cle(&recherches, &mut ui), premiere);
    // Et à la racine, le fil gardé ne se pose pas.
    assert!(
        !ordre(&Store::new("Projet"), &mut ui).contains(&"ariane"),
        "le fil d'un dossier qu'on a quitté se pose encore"
    );
}

/// **Le rectangle de la vue : quatre filets et un voile sur la grille, qui ne se recouvrent
/// pas** — les couleurs sont translucides, un coin peint deux fois serait plus clair —, d'au
/// moins un pixel d'épaisseur, et dans la minimap. Le trait garde un point, comme dans Tauri.
#[test]
fn test_le_rectangle_de_la_vue_en_filets() {
    let theme = Theme::dark();
    let mut store = Store::new("Projet");
    let board = store.project.active_board_id.clone();
    let membrane = glucose_core::types::Annotation::membrane("m", 0.0, 0.0, 6000.0, 4000.0);
    store.add_annotation(&board, membrane);
    for echelle in [1.0f32, 1.5, 2.75] {
        let mb = minimap::layout_minimap(&store, 1440.0, 900.0, echelle, (0.0, 0.0))
            .expect("une minimap");
        let filets = minimap::cadre_de_la_vue(&theme, &mb, echelle);
        assert_eq!(filets.len(), 5, "quatre filets et le voile");
        let epaisseur = filets[0].rect.height();
        assert_eq!(
            epaisseur,
            echelle.round().max(1.0),
            "un point, au pixel entier"
        );
        let mut couverts = std::collections::HashMap::<(i32, i32), u32>::new();
        for f in &filets {
            let r = f.rect;
            assert_eq!(
                (
                    r.left().fract(),
                    r.top().fract(),
                    r.right().fract(),
                    r.bottom().fract()
                ),
                (0.0, 0.0, 0.0, 0.0),
                "un filet hors de la grille : {r:?}"
            );
            assert!(r.width() >= 1.0 && r.height() >= 1.0, "{r:?}");
            assert!(
                r.left() >= mb.mm_x.floor() && r.right() <= (mb.mm_x + mb.mm_w).ceil(),
                "hors de la minimap : {r:?}"
            );
            for y in r.top() as i32..r.bottom() as i32 {
                for x in r.left() as i32..r.right() as i32 {
                    *couverts.entry((x, y)).or_default() += 1;
                }
            }
        }
        let doubles = couverts.values().filter(|n| **n > 1).count();
        assert_eq!(
            doubles, 0,
            "à l'échelle {echelle}, {doubles} pixels peints deux fois"
        );
    }
}
