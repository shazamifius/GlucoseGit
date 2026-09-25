//! **La Time Machine, peinte hors écran** : le panneau dans ses trois états — le présent,
//! l'aperçu d'un geste passé (avec le liseré ambré), un jalon en train d'être nommé — écrits en
//! PNG pour être regardés.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example apercu_temps -- <dossier>
//! ```

use glucose_core::store::Store;
use glucose_desktop::dock::temps::{JalonVu, TempsUi};
use glucose_desktop::dock::{render_docks, DockManager, DockPass, TabId};
use glucose_desktop::interactions::text_entry::TextEntry;
use glucose_desktop::params::{Pointer, ScreenFrame};
use glucose_desktop::theme::Theme;
use glucose_desktop::typography::Typography;
use tiny_skia::Pixmap;

const ECRAN: (u32, u32) = (1440, 900);

fn temps(regarde: Option<usize>, nom: Option<&str>) -> TempsUi {
    let maintenant = 1_790_000_000_000i64;
    TempsUi {
        gestes: (0..42).map(|i| maintenant - (42 - i) * 95_000).collect(),
        jalons: vec![
            JalonVu {
                apres: 0,
                libelle: String::new(),
                nomme: false,
                instant: maintenant - 4_200_000,
            },
            JalonVu {
                apres: 17,
                libelle: "Avant la refonte des membranes".into(),
                nomme: true,
                instant: maintenant - 2_500_000,
            },
            JalonVu {
                apres: 35,
                libelle: String::new(),
                nomme: false,
                instant: maintenant - 600_000,
            },
        ],
        regarde,
        nom: nom.map(TextEntry::new),
        maintenant,
        glisse: false,
    }
}

fn main() {
    let dossier = std::path::PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    let typo = Typography::new();
    let theme = Theme::dark();
    let store = Store::new("apercu");
    for (nom, ui) in [
        ("temps-present", temps(None, None)),
        ("temps-apercu", temps(Some(17), None)),
        ("temps-jalon", temps(None, Some("Première version du conc"))),
    ] {
        let mut dock = DockManager::new();
        dock.toggle_tab(TabId::Temps);
        dock.temps = ui;
        let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("un tampon");
        pixmap.fill(theme.bg_canvas);
        render_docks(
            &mut pixmap.as_mut(),
            &dock,
            &store,
            &DockPass {
                typo: &typo,
                theme: &theme,
                screen: ScreenFrame {
                    width: ECRAN.0 as f32,
                    height: ECRAN.1 as f32,
                    header_h: 48.0,
                    scale: 1.0,
                },
                pointer: Pointer { x: -1.0, y: -1.0 },
                cache: None,
            },
        );
        let chemin = dossier.join(format!("{nom}.png"));
        pixmap.save_png(&chemin).expect("l'image s'écrit");
        println!("{}", chemin.display());
    }
}
