//! Ouvrir la fenêtre, et choisir par quoi l'image sera présentée.
//!
//! # Pourquoi c'est un sujet à part
//!
//! Tout ce qui suit n'arrive **qu'une fois**, au démarrage : lire la cadence de l'écran,
//! demander une carte graphique et se rabattre sur le processeur s'il n'y en a pas, dire où
//! la chronique s'écrira. Rien de cela ne se mêle à la boucle d'images, et l'y laisser faisait
//! du fichier de l'application un endroit où deux durées de vie se croisaient.

use super::GlucoseApp;
use crate::error::{DesktopError, DesktopResult};
use std::num::NonZeroU32;
use std::sync::Arc;
use tiny_skia::Pixmap;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

impl GlucoseApp {
    /// Crée la fenêtre et son framebuffer softbuffer ; toute erreur est propagée
    /// au lieu d'être avalée silencieusement (une fenêtre blanche sinon).
    pub(super) fn init_window(&mut self, event_loop: &ActiveEventLoop) -> DesktopResult<()> {
        let title = self.window_title();
        let attrs = WindowAttributes::default()
            .with_title(&title)
            .with_inner_size(LogicalSize::new(1440.0, 900.0));

        let window = event_loop
            .create_window(attrs)
            .map(Arc::new)
            .map_err(|e| DesktopError::WindowError(format!("create_window : {e}")))?;

        // La frequence se LIT sur l'ecran. Quand le systeme ne la donne pas -- bureau
        // distant, machine virtuelle -- on retient le plancher de la charte : mieux vaut viser
        // trop bas et tenir que l'inverse.
        self.cadence = window
            .current_monitor()
            .and_then(|m| m.refresh_rate_millihertz())
            .map_or_else(crate::cadence::Cadence::inconnue, |mhz| {
                crate::cadence::Cadence::depuis_millihertz(mhz)
            });
        println!(
            "[Glucose] cadence de l'ecran : {:.0} Hz, soit {:.2} ms par image",
            self.cadence.fps(),
            self.cadence.periode().as_secs_f64() * 1000.0
        );

        // La trajectoire se cale sur la grille de balayage de cet ecran : un pas qui n'est
        // pas un multiple de la periode decrit une duree d'affichage qui n'existe pas.
        self.horloge.accorder(self.cadence.periode());
        self.tempo.accorder(self.cadence.periode());

        let scale_factor = window.scale_factor();
        self.scale_factor = scale_factor;
        self.ui.scale_factor = scale_factor as f32;

        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let (w, h) = (
            NonZeroU32::new(width).unwrap_or(NonZeroU32::MIN),
            NonZeroU32::new(height).unwrap_or(NonZeroU32::MIN),
        );

        let mut presenter = ouvrir_la_presentation(&window, w, h)?;
        presenter.resize(w, h)?;
        // La chronique doit porter les deux faits de la machine : ce que l'ecran annonce de
        // lui-meme, et comment la carte a accepte de faire succeder les images. Sans eux, ses
        // durees ne se relisent pas -- une image de dix millisecondes ne dit pas la meme chose
        // selon qu'elle attendait un balayage ou non.
        self.chronique
            .rythme
            .observer_la_machine(self.cadence.periode(), presenter.rythme());
        println!(
            "[Glucose] succession des images : {} -- l'ecran bat toutes les {:.2} ms",
            presenter.rythme(),
            self.cadence.periode().as_secs_f64() * 1000.0
        );
        // Dit des le depart ou la chronique s'ecrira : la chercher apres coup dans un dossier
        // temporaire est decourageant, et une mesure qu'on ne retrouve pas ne sert a personne.
        println!(
            "[Glucose] chronique de cette session : {}",
            Self::chemin_de_la_chronique().display()
        );

        self.pixmap = Pixmap::new(width, height);
        window.set_cursor(winit::window::CursorIcon::Grab);
        self.window_title_cache = title;
        self.window = Some(window);
        self.presenter = Some(presenter);
        self.mark_dirty();
        Ok(())
    }
}

/// La carte graphique d'abord, le processeur s'il n'y en a pas.
///
/// Ce n'est pas un secours honteux : une machine virtuelle, un bureau distant ou un pilote
/// absent sont des cas de tous les jours, et l'application doit s'ouvrir quand même. C'est
/// aussi ce chemin-là que les tests et les bancs empruntent, faute de fenêtre — un repli qui
/// ne s'exécute jamais n'est pas un repli.
fn ouvrir_la_presentation(
    window: &Arc<winit::window::Window>,
    w: NonZeroU32,
    h: NonZeroU32,
) -> DesktopResult<Box<dyn crate::present::Presenter>> {
    match crate::present::GpuPresenter::new(window.clone(), w, h) {
        Ok(gpu) => {
            println!(
                "[Glucose] présentation par la carte graphique : {}",
                gpu.adaptateur()
            );
            Ok(Box::new(gpu))
        }
        Err(e) => {
            eprintln!(
                "[Glucose] pas de carte graphique disponible ({e}) — présentation par le processeur"
            );
            Ok(Box::new(crate::present::CpuPresenter::new(window.clone())?))
        }
    }
}
