//! Le presse-papiers d'Android, éprouvé sur le PC par une porte à la main (fiche 171).

use super::*;
use crate::interactions::clipboard::ImagePosee;
use std::sync::Mutex;

/// Ce qu'on a écrit : l'étiquette, le texte, l'image.
type Ecrit = (String, Option<String>, Option<PathBuf>);

/// **Une porte à la main** : elle retient ce qu'on écrit, rend la dernière étiquette écrite,
/// et lit ce qu'on lui a confié.
#[derive(Default)]
struct PorteDEpreuve {
    ecrits: Mutex<Vec<Ecrit>>,
    a_lire: Mutex<Option<Lu>>,
}

impl Porte for PorteDEpreuve {
    fn ecrire(&self, e: &str, texte: Option<&str>, image: Option<&Path>) -> Result<(), String> {
        let ecrit = (
            e.to_string(),
            texte.map(str::to_string),
            image.map(Path::to_path_buf),
        );
        self.ecrits.lock().expect("verrou").push(ecrit);
        Ok(())
    }

    fn etiquette(&self) -> Option<String> {
        self.ecrits
            .lock()
            .expect("verrou")
            .last()
            .map(|e| e.0.clone())
    }

    fn lire(&self, _ouvrir: bool) -> Option<Lu> {
        self.a_lire.lock().expect("verrou").take()
    }
}

/// Branche une porte neuve sur un dossier des partages neuf.
fn brancher(nom: &str) -> (Arc<PorteDEpreuve>, PathBuf) {
    let partages = std::env::temp_dir().join(format!("glucose-presse-android-{nom}"));
    let _ = std::fs::remove_dir_all(&partages);
    let porte = Arc::new(PorteDEpreuve::default());
    installer(porte.clone(), partages.clone());
    (porte, partages)
}

fn formes(date: i64, texte: Option<&str>, image: bool) -> Formes {
    Formes {
        lot: format!("le lot {date}").into_bytes(),
        texte: texte.map(str::to_string),
        image: image.then(|| ImagePosee {
            png: vec![0x89, b'P', b'N', b'G'],
            largeur: 1,
            hauteur: 1,
            rgba: vec![0; 4],
        }),
        fichiers: Vec::new(),
        date,
    }
}

fn noms(dossier: &Path) -> Vec<String> {
    let mut v: Vec<_> = (std::fs::read_dir(dossier).expect("dossier").flatten())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    v.sort();
    v
}

/// **L'étiquette reconnaît un lot de Glucose, et lui seul** : celle d'une autre application, ou
/// d'un texte copié dans Glucose, ne dit aucune date.
#[test]
fn presse_android_l_etiquette_reconnait_un_lot_de_glucose_et_lui_seul() {
    assert_eq!(
        date_du_lot(&etiquette_du_lot(1_764_000_000)),
        Some(1_764_000_000)
    );
    for autre in ["Glucose", "Glucose1", "Chrome 12", "Glucose douze", ""] {
        assert_eq!(date_du_lot(autre), None, "{autre:?}");
    }
}

/// **Une copie écrit son lot et son image, sous une seule étiquette** — un seul élément, le texte
/// et l'image ensemble — et efface la copie d'avant.
#[test]
fn presse_android_une_copie_ecrit_un_seul_element_et_remplace_la_precedente() {
    let (porte, partages) = brancher("copie");
    ecrire_la_selection(formes(7, Some("bonjour"), true)).expect("copie");
    let image = partages.join("presse-papiers-7.png");
    assert_eq!(
        noms(&partages),
        ["presse-papiers-7.glucose", "presse-papiers-7.png"]
    );
    let premier = porte.ecrits.lock().expect("verrou")[0].clone();
    assert_eq!(
        premier,
        ("Glucose 7".into(), Some("bonjour".into()), Some(image))
    );
    ecrire_la_selection(formes(8, None, false)).expect("copie");
    assert_eq!(
        noms(&partages),
        ["presse-papiers-8.glucose"],
        "l'autre copie s'efface"
    );
    let second = porte.ecrits.lock().expect("verrou")[1].clone();
    assert_eq!(second, ("Glucose 8".into(), None, None));
}

/// **Un lot se relit par son étiquette** — rien pour celle d'une autre application, rien si son
/// fichier a disparu.
#[test]
fn presse_android_un_lot_se_relit_par_son_etiquette() {
    let (porte, partages) = brancher("lot");
    assert!(lire_un_lot().is_none(), "un presse-papiers vide");
    ecrire_la_selection(formes(9, None, false)).expect("copie");
    let lot = lire_un_lot().expect("le lot de Glucose")();
    assert_eq!(lot.as_deref(), Some(&b"le lot 9"[..]));
    ecrire_le_texte("un texte écrit dans Glucose").expect("texte");
    assert!(lire_un_lot().is_none(), "un texte seul n'est pas un lot");
    porte.ecrire("Glucose 9", None, None).expect("étiquette");
    std::fs::remove_file(partages.join("presse-papiers-9.glucose")).expect("effacé");
    assert!(lire_un_lot().is_none(), "son fichier a disparu");
}

/// **Un texte revient à l'appelant** ; un presse-papiers vide ne pose rien ; sans porte, le
/// collage suit le chemin du bureau.
#[test]
fn presse_android_un_texte_revient_a_l_appelant() {
    assert_eq!(coller((0.0, 0.0)), None, "sans porte");
    let (porte, _) = brancher("texte");
    assert_eq!(coller((0.0, 0.0)), Some(Colle::Rien));
    *porte.a_lire.lock().expect("verrou") = Some(Lu {
        texte: Some("bonjour".into()),
        ..Lu::default()
    });
    assert_eq!(coller((0.0, 0.0)), Some(Colle::Texte("bonjour".into())));
}

/// **Les fichiers d'un collage deviennent un partage posé au point du geste** : un illisible se
/// compte, et aucun texte ne part chercher une image.
#[test]
fn presse_android_un_collage_de_fichiers_devient_un_partage_au_point_du_geste() {
    let (_, partages) = brancher("fichiers");
    std::fs::create_dir_all(&partages).expect("dossier");
    let chemin = partages.join("une-image");
    std::fs::write(&chemin, b"ENTETE et les octets").expect("écrite");
    let ouvert = std::fs::File::open(&chemin).expect("ouverte");
    let collage = partage_du_collage(vec![Some((ouvert, 7, Some(6))), None], &[], (12.0, 34.0));
    assert_eq!(collage.ou, Some((12.0, 34.0)));
    assert_eq!(collage.illisibles, 1);
    assert_eq!(collage.texte, None);
    let octets: Vec<_> = collage.recus.iter().map(|r| r.octets.clone()).collect();
    assert_eq!(octets, [b"et les".to_vec()], "à son début, sur sa longueur");
}

/// **Coller dans Glucose pose le texte d'une autre application** au point du geste, par la
/// porte — sans lire le presse-papiers de Glucose.
#[test]
fn presse_android_coller_pose_le_texte_d_une_autre_application() {
    let mut app = crate::app::GlucoseApp::new();
    if let Some(b) = app.store.active_board_mut() {
        b.annotations.clear();
    }
    let (porte, _) = brancher("app");
    *porte.a_lire.lock().expect("verrou") = Some(Lu {
        texte: Some("  venu de Chrome \n".into()),
        ..Lu::default()
    });
    app.paste_from_clipboard();
    let board = app.store.active_board().expect("tableau");
    let textes: Vec<_> = (board.annotations.iter())
        .filter_map(|a| match a {
            glucose_core::types::Annotation::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(textes, ["venu de Chrome"]);
}

/// **« Coller » du menu pose là où le menu s'est ouvert** (fiche 171 § 4), pas sous l'entrée
/// touchée : au téléphone, le menu s'ouvre en haut de l'écran, loin du doigt.
#[test]
fn presse_android_coller_par_le_menu_pose_au_point_du_menu() {
    use crate::params::{Pointer, ScreenFrame};
    use crate::ui::context_menu::{layout_context_menu, MenuAction, MenuRow};
    const ECRAN: ScreenFrame = ScreenFrame {
        width: 1440.0,
        height: 900.0,
        header_h: 0.0,
        scale: 1.0,
    };
    let ouvert_a = (900.0_f32, 600.0_f32);
    let pose = |par_le_menu: bool| {
        let mut app = crate::app::GlucoseApp::new();
        if let Some(b) = app.store.active_board_mut() {
            b.annotations.clear();
        }
        let (porte, _) = brancher("menu");
        *porte.a_lire.lock().expect("verrou") = Some(Lu {
            texte: Some("ici".into()),
            ..Lu::default()
        });
        app.mouse_pos = (f64::from(ouvert_a.0), f64::from(ouvert_a.1));
        if par_le_menu {
            app.ui.context_menu_at = Some(ouvert_a);
            let menu = layout_context_menu(
                &app.store,
                (&app.renderer.typography, &app.ui),
                (ouvert_a, None),
                (ECRAN.width, ECRAN.height),
                ECRAN.scale,
                false,
            )
            .expect("un menu");
            let (x, y, w, h) = (menu.rows.iter())
                .find_map(|r| match r {
                    MenuRow::Item { action, rect, .. } if *action == MenuAction::Paste => {
                        Some(*rect)
                    }
                    _ => None,
                })
                .expect("« Coller » est proposé");
            let doigt = Pointer {
                x: x + w / 2.0,
                y: y + h / 2.0,
            };
            app.mouse_pos = (f64::from(doigt.x), f64::from(doigt.y));
            assert!(
                app.click_context_menu(doigt, ECRAN),
                "le menu prend le clic"
            );
            assert_eq!(app.mouse_pos, (f64::from(doigt.x), f64::from(doigt.y)));
        } else {
            app.paste_from_clipboard();
        }
        let board = app.store.active_board().expect("tableau");
        let [glucose_core::types::Annotation::Text { x, y, .. }] = board.annotations.as_slice()
        else {
            panic!("un seul texte collé");
        };
        (*x, *y)
    };
    assert_eq!(pose(true), pose(false));
}
