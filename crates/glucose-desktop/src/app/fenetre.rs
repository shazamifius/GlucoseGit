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
    /// **Ce que la machine annonce d'elle-meme**, ecrit une fois au demarrage.
    ///
    /// Une fonction a part parce qu'`init_window` cree la fenetre, lit la cadence, accorde
    /// les horloges et ouvre la presentation : ce qui **etablit** et ce qui **annonce** ne
    /// changent pas pour les memes raisons, et le cliquet des quatre-vingts lignes a raison.
    fn annoncer_la_machine(
        &self,
        presenter: &dyn crate::present::Presenter,
        (largeur, hauteur): (u32, u32),
        echelle: f64,
    ) {
        println!(
            "[Glucose] succession des images : {} -- l'ecran bat toutes les {:.2} ms",
            presenter.rythme(),
            self.cadence.periode().as_secs_f64() * 1000.0
        );
        // **Ce que la chaine garde en vol**, et il n'etait ecrit nulle part. Une attente a
        // l'acquisition -- 19,5 ms en mediane sur la session du 21/09 au soir -- ne se
        // comprend pas sans ce nombre : si la chaine n'a pas d'image libre, la demander
        // attend qu'il s'en libere une. Le terrain a depuis tranche : voir
        // `succession::images_demandees`.
        let en_vol = presenter.images_en_vol();
        if en_vol > 0 {
            println!("[Glucose] images gardees en vol par la chaine : {en_vol}");
        }
        // **La surface et l'echelle de l'interface**, sans lesquelles la chrome ne se relit
        // pas. Les postes `bande`, `minimap`, `ariane` et `ui` couvrent des rectangles dont
        // la taille est proportionnelle au CARRE de cette echelle : a 175 %, la minimap
        // occupe trois fois plus de pixels qu'a 100 %. `bench_texte` la mesure a 0,05 ms et
        // le terrain a 0,86 -- l'ecart ne se comprend pas sans ce nombre, et aucune trace ne
        // le portait. C'est la lecon de la fiche 19 § 6.1 : une mesure qui ne dit pas ou elle
        // a ete prise ne se relit pas.
        println!(
            "[Glucose] fenetre : {largeur} x {hauteur} pixels, interface a {:.0} % ({:.0} x {:.0} points)",
            echelle * 100.0,
            f64::from(largeur) / echelle,
            f64::from(hauteur) / echelle,
        );
        // Dit des le depart ou la chronique s'ecrira : la chercher apres coup dans un dossier
        // temporaire est decourageant, et une mesure qu'on ne retrouve pas ne sert a personne.
        println!(
            "[Glucose] chronique de cette session : {}",
            Self::chemin_de_la_chronique().display()
        );
    }

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
        // **L'arbitre ne s'installe que si personne n'a tranche a sa place** (ARBITRE-1). Sa
        // fenetre d'observation vaut une seconde de CET ecran : elle se deduit de la cadence
        // qu'on vient de lire, elle n'est pas choisie.
        self.arbitre = crate::present::gpu::succession::carte_imposee().map_or_else(
            || {
                Some(crate::present::arbitre::Arbitre::nouveau(
                    crate::present::arbitre::Preference::Econome,
                    self.cadence.periode(),
                ))
            },
            |_| None,
        );
        self.annoncer_la_machine(presenter.as_ref(), (width, height), scale_factor);

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
