//! Le banc de la **chrome** — ce que coûtent les panneaux, et ce qu'ils recalculent pour rien.
//!
//! ```text
//! cargo run --release -p glucose-desktop --example bench_chrome
//! ```
//!
//! # Pourquoi ce banc existe
//!
//! Mesurée sur l'application réelle, en `release`, sur un plateau **quasiment vide**, une image
//! coûtait de 5,8 à 14,2 ms. La décomposition disait où :
//!
//! ```text
//! frame #20  total=14.17ms  docks=3.96  ui=2.49  halos=2.13  blit=1.82  present=1.77
//! ```
//!
//! `docks` est le premier poste de l'application, devant tout le contenu réuni. C'est le même
//! défaut que la minimap avait — « elle redessinait un rectangle par nœud, à chaque image » —
//! et il a la même réponse. La leçon n'avait simplement pas été généralisée.
//!
//! # Ce que ce banc mesure, et pourquoi ce n'est pas le coût
//!
//! Un coût élevé ne justifie pas un cache : il justifie un code plus rapide. **Ce qui justifie
//! un cache, c'est la répétition** — recalculer à l'identique est du travail dont on peut
//! prouver qu'il est perdu, et cette preuve ne dépend d'aucune machine.
//!
//! Le banc rend donc chaque panneau plusieurs fois **sans rien changer entre deux images**, et
//! compare les tampons octet par octet. La colonne qui décide est « identiques » : si elle dit
//! que tout est identique, chaque image après la première est du travail jeté.
//!
//! La seconde question est celle du **survol**. Le rendu reçoit la position du pointeur, donc
//! un cache dont la clé la contiendrait serait vidé à chaque pixel de souris. Le banc mesure
//! par combien d'images distinctes un panneau passe quand le pointeur le traverse : si ce
//! nombre est petit devant le nombre de positions, alors la bonne clé n'est pas la position
//! mais **ce qui est survolé**.

use glucose_core::store::Store;
use glucose_core::synth::{self, Shape};
use glucose_desktop::dock::{render_docks, DockManager, TabId};
use glucose_desktop::params::{Pointer, ScreenFrame};
use glucose_desktop::theme::Theme;
use glucose_desktop::typography::Typography;
use tiny_skia::Pixmap;

/// La définition mesurée : celle de l'écran où le défaut a été constaté.
const ECRAN: (u32, u32) = (2560, 1600);

/// Hauteur du bandeau, reprise de l'application.
const HEADER_H: f32 = 44.0;

/// Combien d'images par panneau. Une seconde de rendu à cent images par seconde.
const IMAGES: usize = 100;

/// La même graine que les autres bancs : deux bancs sur des documents différents ne se
/// comparent pas.
const GRAINE: u64 = 0x91ac05e;

/// Un dock n'ouvrant que cet onglet — pour mesurer un panneau, et non leur somme.
fn dock_avec(tab: TabId) -> DockManager {
    let mut dock = DockManager::new();
    dock.top_tabs.clear();
    dock.bottom_tabs.clear();
    match tab.anchor() {
        glucose_desktop::dock::DockAnchor::TopLeft => dock.top_tabs.push(tab),
        glucose_desktop::dock::DockAnchor::BottomLeft => dock.bottom_tabs.push(tab),
    }
    dock
}

struct Pinceau {
    typo: Typography,
    theme: Theme,
    ecran: ScreenFrame,
}

impl Pinceau {
    fn new() -> Self {
        Self {
            typo: Typography::new(),
            theme: Theme::dark(),
            ecran: ScreenFrame {
                width: ECRAN.0 as f32,
                height: ECRAN.1 as f32,
                header_h: HEADER_H,
                scale: 1.0,
            },
        }
    }

    /// Rend la chrome seule, et rend le tampon.
    fn rendu(&self, dock: &DockManager, store: &Store, pointer: Pointer) -> Pixmap {
        let mut pixmap = Pixmap::new(ECRAN.0, ECRAN.1).expect("un tampon");
        render_docks(
            &mut pixmap.as_mut(),
            dock,
            store,
            &self.typo,
            &self.theme,
            self.ecran,
            pointer,
        );
        pixmap
    }
}

/// La médiane d'une série, en millisecondes.
fn mediane(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn main() {
    let pinceau = Pinceau::new();
    let store = synth::document(1_000, 60_000.0, Shape::Clustered, GRAINE);
    // Loin de tout panneau : aucun survol ne vient troubler la mesure de répétition.
    let dehors = Pointer { x: -1.0, y: -1.0 };

    println!(
        "Banc de la chrome — {} × {}, {IMAGES} images par panneau\n",
        ECRAN.0, ECRAN.1
    );
    println!(
        "{:<12} {:>10} {:>10} {:>12}   ce que ça veut dire",
        "panneau", "médiane", "pire", "identiques"
    );

    let mut total = 0.0;
    for tab in [
        TabId::Organize,
        TabId::Pomodoro,
        TabId::Storyboard,
        TabId::Plugins,
        TabId::Preset,
        TabId::Domains,
    ] {
        let dock = dock_avec(tab);
        let reference = pinceau.rendu(&dock, &store, dehors);
        let mut temps = Vec::with_capacity(IMAGES);
        let mut identiques = 0usize;
        for _ in 0..IMAGES {
            let t = std::time::Instant::now();
            let image = pinceau.rendu(&dock, &store, dehors);
            temps.push(t.elapsed().as_secs_f64() * 1000.0);
            if image.data() == reference.data() {
                identiques += 1;
            }
        }
        let med = mediane(temps.clone());
        let pire = temps.iter().copied().fold(0.0, f64::max);
        total += med;
        let verdict = if identiques == IMAGES {
            "tout est recalculé à l'identique"
        } else {
            "quelque chose change d'une image à l'autre"
        };
        println!(
            "{:<12} {med:>9.2}ms {pire:>9.2}ms {identiques:>7}/{IMAGES:<4}   {verdict}",
            tab.title()
        );
    }

    println!("\nLes six panneaux ouverts ensemble coûteraient {total:.2} ms par image.");

    // ── Le survol : combien d'images distinctes quand le pointeur traverse ? ──────────────
    let dock = dock_avec(TabId::Organize);
    let mut vues: Vec<Vec<u8>> = Vec::new();
    let y = ECRAN.1 as f32 - 160.0;
    let positions = 240;
    for i in 0..positions {
        let x = 8.0 + i as f32;
        let image = pinceau.rendu(&dock, &store, Pointer { x, y });
        let octets = image.data().to_vec();
        if !vues.contains(&octets) {
            vues.push(octets);
        }
    }
    println!(
        "\nEn traversant ORDONNER sur {positions} positions de souris, le panneau ne prend que \
         {} aspect(s) distinct(s).",
        vues.len()
    );
    println!(
        "La clé d'un cache n'est donc pas la position du pointeur — ce serait {positions} \
         invalidations pour {} images réellement différentes — mais ce qui est survolé.",
        vues.len()
    );
}
