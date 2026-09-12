//! Les lois du dessin des dossiers. La première est celle qui manquait : qu'il y ait des
//! pixels.

use super::*;
use crate::renderer::domain::DomainTints;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::types::{CanvasFolder, Viewport};
use tiny_skia::Pixmap;

/// Un tableau portant un dossier à une place et une taille connues.
fn store_avec_dossier(x: f64, y: f64, w: f64, h: f64) -> Store {
    let mut store = Store::new("test");
    let mut f = CanvasFolder::new("fold-1", "Recherches", "board-enfant");
    f.x = x;
    f.y = y;
    f.width = w;
    f.height = h;
    f.color = "#60a5fa".into();
    if let Some(board) = store.project.boards.first_mut() {
        board.folders.push(f);
        board.viewport = Viewport { x: 0.0, y: 0.0, scale: 1.0 };
    }
    store
}

/// Dessine le tableau dans un pixmap noir et rend le nombre de pixels encrés.
fn encre(store: &Store) -> (Pixmap, usize) {
    let mut pixmap = Pixmap::new(800, 600).expect("pixmap");
    pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
    let typography = Typography::new();
    let theme = Theme::dark();
    let tints = DomainTints::default();
    let kit = PaintKit { typography: &typography, math: &crate::renderer::math::MathRenderer::new(), tints: &tints, theme: &theme };
    let vp = store.active_board().map(|b| b.viewport).unwrap_or_default();
    let vus: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let pass = ViewPass { vp, visible_ids: &vus, header_h: 0.0 };
    draw_folders(kit, &mut pixmap.as_mut(), store, pass);
    let n = pixmap
        .pixels()
        .iter()
        .filter(|p| p.red() > 0 || p.green() > 0 || p.blue() > 0)
        .count();
    (pixmap, n)
}

/// **Un dossier laisse des pixels.** C'est le test qui manquait : le renderer ne lisait pas
/// `board.folders`, et un « portail vers un univers complet » n'avait aucune existence visible.
#[test]
fn test_un_dossier_laisse_des_pixels() {
    let store = store_avec_dossier(100.0, 100.0, 300.0, 200.0);
    let (_, encres) = encre(&store);
    assert!(encres > 500, "{encres} pixels encrés : le dossier doit se voir");

    // Et un tableau sans dossier n'en laisse aucun : c'est bien le dossier qu'on voit.
    let vide = Store::new("vide");
    let (_, rien) = encre(&vide);
    assert_eq!(rien, 0);
}

/// Le cadre est là où le dossier est, et pas ailleurs : ses quatre bords portent de l'encre,
/// et le canevas autour reste vierge.
#[test]
fn test_le_cadre_tombe_sur_les_bords_du_dossier() {
    let store = store_avec_dossier(100.0, 100.0, 300.0, 200.0);
    let (pixmap, _) = encre(&store);
    let encre_en = |x: u32, y: u32| {
        let p = pixmap.pixel(x, y).expect("dans le pixmap");
        p.red() > 0 || p.green() > 0 || p.blue() > 0
    };

    // Le milieu de chaque bord : le pointillé peut tomber sur un vide, donc on regarde une
    // bande de quelques pixels plutôt qu'un point unique.
    let bande = |xs: std::ops::Range<u32>, y: u32| xs.into_iter().any(|x| encre_en(x, y));
    let colonne = |x: u32, ys: std::ops::Range<u32>| ys.into_iter().any(|y| encre_en(x, y));
    assert!(bande(240..260, 100), "bord haut");
    assert!(bande(240..260, 300), "bord bas");
    assert!(colonne(100, 190..210), "bord gauche");
    assert!(colonne(400, 190..210), "bord droit");

    // Loin du dossier, rien.
    assert!(!encre_en(50, 50), "au-dessus à gauche");
    assert!(!encre_en(700, 500), "en bas à droite");
}

/// Le bandeau se sépare du corps à la hauteur exacte que `hit_priority` utilise pour décider
/// qu'un clic tombe dessus. Dessin et clic lisent la même constante, donc ne peuvent pas
/// diverger.
#[test]
fn test_le_bandeau_partage_la_constante_du_picking() {
    assert_eq!(pick_consts::FOLDER_HEADER, 38.0);
    let store = store_avec_dossier(100.0, 100.0, 300.0, 200.0);
    let (pixmap, _) = encre(&store);
    // Le corps entier est teinté à 2,5 % : compter les pixels « encrés » ne distingue rien,
    // puisqu'ils le sont tous. C'est l'INTENSITÉ qu'il faut regarder — la séparation est un
    // trait à 14 %, six fois plus dense que le corps qu'elle traverse.
    let intensite = |y: u32| -> u32 {
        (110..390)
            .map(|x| pixmap.pixel(x, y).expect("dans le pixmap").blue() as u32)
            .sum()
    };
    let a_la_separation = intensite(100 + pick_consts::FOLDER_HEADER as u32);
    let dans_le_corps = intensite(250);
    assert!(
        a_la_separation > dans_le_corps * 3,
        "séparation {a_la_separation} contre corps {dans_le_corps} : le bandeau doit se voir"
    );
    assert!(dans_le_corps > 0, "le corps est bien teinté, lui aussi");
}

/// Les mesures du cadre sont celles de la fiche 06 § 8.1.
#[test]
fn test_les_metriques_sont_celles_de_la_fiche() {
    assert_eq!(CORNER_RADIUS, 10.0);
    assert_eq!((MIN_WIDTH, MIN_HEIGHT), (180.0, 120.0));
    assert_eq!(BORDER_WIDTH, (1.0, 1.4));
    assert_eq!(BORDER_DASH, (3.0, 5.0));
    assert_eq!(GLOW, (3.0, 13.0, 1.5, 89));
    assert_eq!(ICON, (12.0, 11.0, 16.0, 14.0));
    assert_eq!((TITLE, TITLE_MAX_CHARS), ((34.0, 23.0, 14.0), 28));
    assert_eq!(BADGE, (38.0, 11.0, 30.0, 16.0, 8.0));
    assert_eq!((BADGE_STROKE, BADGE_FONT), (0.8, 10.0));

    // Les opacités de la fiche sont en pourcentage ; le rendu les veut sur 255. La conversion
    // est vérifiée ici plutôt que recopiée de tête.
    let pourcent = |p: f64| (p * 255.0).round() as u8;
    assert_eq!(BODY_ALPHA, (pourcent(0.025), pourcent(0.05)));
    assert_eq!(BORDER_ALPHA, (pourcent(0.14), pourcent(0.45)));
    assert_eq!(ICON_ALPHA, (pourcent(0.45), pourcent(0.70)));
    assert_eq!(BADGE_ALPHA, (pourcent(0.18), pourcent(0.40)));
    assert_eq!(GLOW.3, pourcent(0.35));
}

/// Un dossier plus petit que le minimum de la fiche est dessiné au minimum : en deçà, le
/// bandeau de 38 px mangerait le corps.
#[test]
fn test_un_dossier_trop_petit_est_dessine_au_minimum() {
    let mut f = CanvasFolder::new("f", "n", "b");
    f.width = 10.0;
    f.height = 5.0;
    let layout = Layout::new(&f, WorldScale::new(1.0));
    assert_eq!((layout.width, layout.height), (MIN_WIDTH, MIN_HEIGHT));
    assert!(layout.header < layout.height, "le bandeau laisse de la place au corps");
}

/// Un titre trop long est tronqué sur les **caractères**, pas sur les octets : couper au milieu
/// d'un caractère accentué produirait une chaîne qui n'est plus du texte.
#[test]
fn test_le_titre_est_tronque_sur_les_caracteres_pas_les_octets() {
    assert_eq!(truncate("court", 28), "court");
    let long = "é".repeat(40);
    let coupe = truncate(&long, 28);
    assert_eq!(coupe.chars().count(), 28);
    assert!(coupe.ends_with('…'));
    assert_eq!(coupe.chars().filter(|&c| c == 'é').count(), 27);

    // Exactement à la limite, rien n'est coupé.
    let pile = "a".repeat(28);
    assert_eq!(truncate(&pile, 28), pile);
    assert_eq!(truncate(&"a".repeat(29), 28).chars().count(), 28);
}

/// Un dossier entièrement hors de l'écran n'est pas dessiné — la loi L1, appliquée ici sans
/// index spatial puisque l'index ne connaît pas les dossiers.
#[test]
fn test_un_dossier_hors_champ_n_est_pas_dessine() {
    let store = store_avec_dossier(50_000.0, 50_000.0, 300.0, 200.0);
    let (_, encres) = encre(&store);
    assert_eq!(encres, 0);
}

/// Un dossier sélectionné se distingue : plus d'encre qu'au repos, halo et trait plein compris.
#[test]
fn test_un_dossier_selectionne_se_distingue() {
    let store = store_avec_dossier(100.0, 100.0, 300.0, 200.0);
    let (_, au_repos) = encre(&store);

    let mut choisi = store.clone();
    choisi.selected_folder_id = Some("fold-1".into());
    let (_, selectionne) = encre(&choisi);
    assert!(
        selectionne > au_repos,
        "{selectionne} pixels sélectionné contre {au_repos} au repos"
    );
}

/// Sous le seuil de détail, le cadre reste et le texte disparaît : c'est SCALE-2, appliqué à
/// l'élément entier et non à chacune de ses parties.
#[test]
fn test_tres_dezoome_le_cadre_reste_et_le_texte_disparait() {
    let mut store = store_avec_dossier(10.0, 10.0, 3_000.0, 2_000.0);
    if let Some(board) = store.project.boards.first_mut() {
        board.viewport = Viewport { x: 0.0, y: 0.0, scale: 0.1 };
    }
    assert!(!WorldScale::new(0.1).draws_detail(), "0,1 est sous le seuil de détail");
    let (_, encres) = encre(&store);
    assert!(encres > 100, "{encres} : le cadre se dessine encore");
}

// ── La minimap ───────────────────────────────────────────────────────────────

/// Un tableau qui ne contient QUE des dossiers a quand même une minimap : ses bornes les
/// comptent. Sans cela, elles restaient infinies et la minimap disparaissait entièrement.
#[test]
fn test_un_tableau_de_dossiers_seuls_a_une_minimap() {
    let store = store_avec_dossier(1_000.0, 800.0, 300.0, 200.0);
    let mm = crate::ui::layout_minimap(&store, 1440.0, 900.0, 1.0)
        .expect("un tableau de dossiers doit avoir une minimap");
    // Les bornes englobent le dossier, marge de 200 px comprise.
    assert!(mm.min_x <= 1_000.0 && mm.max_x >= 1_300.0, "{:?}", (mm.min_x, mm.max_x));
    assert!(mm.min_y <= 800.0 && mm.max_y >= 1_000.0, "{:?}", (mm.min_y, mm.max_y));
}

/// Le dossier laisse une trace dans la minimap, à sa couleur — c'est ce qui le distingue d'une
/// membrane, dessinée en gris.
#[test]
fn test_un_dossier_laisse_une_trace_coloree_dans_la_minimap() {
    let store = store_avec_dossier(0.0, 0.0, 600.0, 400.0);
    let theme = Theme::dark();

    let trace = |store: &Store| {
        let mut pixmap = Pixmap::new(1_440, 900).expect("pixmap");
        pixmap.fill(Color::from_rgba8(0, 0, 0, 255));
        crate::ui::render_minimap(&mut pixmap.as_mut(), store, &theme, 1_440.0, 900.0, 1.0, &mut None);
        // La minimap occupe le coin bas-droit : on n'y regarde que là.
        let mut bleus = 0usize;
        for y in 760..890 {
            for x in 1_230..1_430 {
                let p = pixmap.pixel(x, y).expect("dans le pixmap");
                // #60a5fa est bien plus bleu que rouge ; aucun gris de la chrome ne l'est.
                if p.blue() as i32 - p.red() as i32 > 30 {
                    bleus += 1;
                }
            }
        }
        bleus
    };

    let avec = trace(&store);
    let sans = trace(&Store::new("vide"));
    assert!(avec > sans, "{avec} pixels bleus avec le dossier, {sans} sans");
    assert!(avec > 20, "{avec} : le contour du dossier doit se voir");
}
