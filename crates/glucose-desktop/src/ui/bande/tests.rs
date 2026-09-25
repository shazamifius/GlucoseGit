//! Ce que le cache de la bande promet : les mêmes pixels, et beaucoup moins de dessins.
//!
//! # Le piège que ces tests évitent
//!
//! Un cache qui redessinerait tout à chaque image rendrait exactement les mêmes pixels et
//! passerait toutes les épreuves d'aspect. On ne saurait qu'il est inutile qu'au chronomètre,
//! c'est-à-dire jamais de façon déterministe — la leçon des vignettes, qui ont servi à un
//! pour cent pendant cinq sessions. Le compte des dessins est donc la première chose que ces
//! tests regardent.

use super::*;
use crate::typography::Typography;
use glucose_core::types::Board;

const ECRAN: (u32, u32) = (1600, 200);

fn document() -> Store {
    let mut store = Store::new("bande");
    let premier = store.project.active_board_id.clone();
    let mut autre = Board::new("b2", "Deuxieme");
    autre.viewport = store.viewport();
    store.project.boards.push(autre);
    let _ = premier;
    store
}

struct Pinceau {
    typo: Typography,
    theme: Theme,
}

impl Pinceau {
    fn new() -> Self {
        Self {
            typo: Typography::new(),
            theme: Theme::dark(),
        }
    }

    /// Rend la bande par le chemin de l'application, et rend le tampon plein écran.
    fn rendu(&self, ui: &mut UiState, store: &Store, pointer: Pointer) -> Pixmap {
        let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("un tampon");
        render_bande(
            &mut pixmap.as_mut(),
            store,
            ui,
            &self.typo,
            &self.theme,
            ECRAN.0 as f32,
            pointer,
        );
        pixmap
    }

    /// La même bande, dessinée **directement** dans le tampon plein écran, sans détour.
    fn rendu_direct(&self, ui: &UiState, store: &Store, pointer: Pointer) -> Pixmap {
        let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("un tampon");
        let largeur = ECRAN.0 as f32;
        let barre = layout_topbar(largeur, ui, &self.typo, store.nombre_d_images());
        let onglets = layout_tabs(store, ui, &self.typo);
        let survole_btn = barre
            .buttons
            .iter()
            .position(|b| contient(b.x, b.y, b.w, b.h, pointer));
        let survole_tab = onglets
            .iter()
            .position(|t| contient(t.x, t.y, t.width, t.height, pointer));
        let mut vue = pixmap.as_mut();
        render_topbar(
            &mut vue,
            ui,
            &self.typo,
            &self.theme,
            largeur,
            &barre,
            survole_btn,
        );
        render_board_tabs(
            &mut vue,
            ui,
            &self.typo,
            &self.theme,
            largeur,
            &onglets,
            survole_tab,
        );
        pixmap
    }
}

fn dessins(ui: &UiState) -> u64 {
    ui.bande_cache.as_ref().map_or(0, |c| c.dessins)
}

/// **Le détour par le tampon ne change aucun pixel.**
///
/// La bande commence à l'origine de l'écran : rendue dans un tampon de sa taille, chaque
/// primitive reçoit exactement les coordonnées qu'elle recevait dans l'écran entier. Aucune
/// phase de glyphe ne se décale (GLYPH-1 ne se pose pas), et la bande est opaque, donc elle
/// se remplace.
#[test]
fn test_la_bande_en_cache_est_identique_au_bit_pres_au_rendu_direct() {
    let pinceau = Pinceau::new();
    let store = document();
    let mut ui = UiState::new();
    let dehors = Pointer { x: -1.0, y: -1.0 };

    let direct = pinceau.rendu_direct(&ui, &store, dehors);
    let par_le_cache = pinceau.rendu(&mut ui, &store, dehors);
    // La bande n'occupe que le haut ; le reste du tampon est vierge dans les deux cas.
    let hauteur = ui.header_height().ceil() as usize;
    let n = hauteur * ECRAN.0 as usize * 4;
    assert_eq!(
        &direct.data()[..n],
        &par_le_cache.data()[..n],
        "le détour par le tampon a changé des pixels"
    );
}

/// **Une image qui ne change rien ne redessine rien**, et rend pourtant les mêmes pixels.
#[test]
fn test_cent_images_immobiles_ne_dessinent_la_bande_qu_une_fois() {
    let pinceau = Pinceau::new();
    let store = document();
    let mut ui = UiState::new();
    let dehors = Pointer { x: -1.0, y: -1.0 };

    let reference = pinceau.rendu(&mut ui, &store, dehors);
    assert_eq!(dessins(&ui), 1, "la première image dessine");
    for _ in 0..100 {
        let image = pinceau.rendu(&mut ui, &store, dehors);
        assert_eq!(image.data(), reference.data(), "les pixels ont changé");
    }
    assert_eq!(dessins(&ui), 1, "la bande a été redessinée sans raison");
}

/// **Bouger la souris ne redessine pas la bande ; entrer dans un bouton, si.**
///
/// C'est ce que `bench_chrome` avait établi pour les panneaux et que la clé applique ici : la
/// clé n'est pas la position du pointeur — ce serait une invalidation par pixel parcouru —
/// mais ce qui est survolé.
#[test]
fn test_la_souris_ne_redessine_que_lorsqu_elle_change_ce_qui_est_survole() {
    let pinceau = Pinceau::new();
    let store = document();
    let mut ui = UiState::new();
    let typo = Typography::new();
    let barre = layout_topbar(ECRAN.0 as f32, &ui, &typo, 0);
    // Un bouton qui n'est PAS déjà actif : l'état actif masque le survol, et le test ne
    // verrait alors rien changer -- ce qui est exact, et ne prouverait rien.
    let bouton = barre
        .buttons
        .iter()
        .find(|b| !b.active)
        .expect("un bouton inactif");
    let (bx, by) = (bouton.x + bouton.w / 2.0, bouton.y + bouton.h / 2.0);

    pinceau.rendu(&mut ui, &store, Pointer { x: -1.0, y: -1.0 });
    let depart = dessins(&ui);
    // Cent positions hors de tout bouton : le survol ne change pas.
    for i in 0..100 {
        let x = 4.0 + i as f32 * 0.01;
        pinceau.rendu(&mut ui, &store, Pointer { x, y: 2.0 });
    }
    assert_eq!(
        dessins(&ui),
        depart,
        "cent positions de souris hors de tout bouton ont redessiné la bande"
    );
    // Entrer dans un bouton change ce qui est survolé, donc l'aspect.
    let survol = pinceau.rendu(&mut ui, &store, Pointer { x: bx, y: by });
    assert_eq!(dessins(&ui), depart + 1, "le survol doit redessiner");
    let sorti = pinceau.rendu(&mut ui, &store, Pointer { x: -1.0, y: -1.0 });
    assert_ne!(
        survol.data(),
        sorti.data(),
        "un bouton survolé doit se voir : sinon ce n'est pas la bonne clé"
    );
}

/// **Ce que la bande montre la fait redessiner** : l'outil actif, le badge du nombre
/// d'images, le nom d'un tableau, celui qui est actif.
///
/// C'est l'autre moitié du contrat : un cache qui ne se redessine jamais serait parfait au
/// chronomètre et faux à l'écran.
#[test]
fn test_ce_qui_change_l_aspect_redessine_la_bande() {
    let pinceau = Pinceau::new();
    let mut store = document();
    let mut ui = UiState::new();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    let mut precedent = pinceau.rendu(&mut ui, &store, dehors);
    let mut compte = dessins(&ui);

    let mut change = |ui: &mut UiState, store: &Store, quoi: &str| {
        let image = pinceau.rendu(ui, store, dehors);
        assert_eq!(dessins(ui), compte + 1, "{quoi} n'a pas redessiné la bande");
        assert_ne!(image.data(), precedent.data(), "{quoi} ne se voit pas");
        compte = dessins(ui);
        precedent = image;
    };

    ui.active_tool = ActiveTool::Text;
    change(&mut ui, &store, "changer d'outil");

    ui.smart_align = !ui.smart_align;
    change(&mut ui, &store, "l'alignement intelligent");

    let board = store.project.active_board_id.clone();
    let mut img = glucose_core::types::BoardImage::new("photo", 0.0, 0.0, 100.0, 100.0);
    img.src = Some("x.png".into());
    store.add_image(&board, img);
    store.clear_selection();
    change(&mut ui, &store, "une photo de plus au badge");

    store.project.boards[1].name = "Renomme".into();
    change(&mut ui, &store, "renommer un tableau");

    store.project.active_board_id = store.project.boards[1].id.clone();
    change(&mut ui, &store, "changer de tableau actif");
}

/// Une fenêtre qui change de largeur refait la bande à sa taille.
#[test]
fn test_la_bande_suit_la_largeur_de_la_fenetre() {
    let pinceau = Pinceau::new();
    let store = document();
    let mut ui = UiState::new();
    let dehors = Pointer { x: -1.0, y: -1.0 };
    pinceau.rendu(&mut ui, &store, dehors);
    let largeur = ui
        .bande_cache
        .as_ref()
        .map(|c| c.pixmap.width())
        .expect("un cache");
    assert_eq!(largeur, ECRAN.0);

    let mut etroit = Pixmap::new(900, 200).expect("un tampon");
    render_bande(
        &mut etroit.as_mut(),
        &store,
        &mut ui,
        &pinceau.typo,
        &pinceau.theme,
        900.0,
        dehors,
    );
    assert_eq!(
        ui.bande_cache.as_ref().map(|c| c.pixmap.width()),
        Some(900),
        "la bande doit suivre la fenêtre"
    );
}
