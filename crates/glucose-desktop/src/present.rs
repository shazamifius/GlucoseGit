//! La présentation : ce qui met l'image rendue devant les yeux (PRESENT-1).
//!
//! # Le poste que rien d'algorithmique ne réduit
//!
//! Mesuré sur l'application, en `release`, sur un écran de 2560 × 1600 :
//!
//! ```text
//! frame #1  clear=4.37  docks=3.40  blit=2.76  present=1.22  ui=1.54
//! frame #4  clear=1.40  docks=1.33  blit=1.78  present=1.22  ui=0.88
//! ```
//!
//! `blit` et `present` sont un coût de **surface pure** : quatre millions de pixels à
//! convertir, puis seize mégaoctets à remettre au système. Aucun cache, aucun culling, aucune
//! structure spatiale ne les diminue — ils ne dépendent que du nombre de pixels de la fenêtre.
//! À eux deux, ils tiennent le budget d'un écran à 240 Hz, qui est de 4,16 ms.
//!
//! # Ce que la mesure a donné, et ce qu'elle a failli faire croire
//!
//! `bench_presentation` présente la même image deux cents fois par chemin, sur une fenêtre de
//! 2560 × 1600, et sépare ce qui est **travail** de ce qui est **attente** :
//!
//! ```text
//! chemin                       travail   attente     total
//! carte graphique                0.79ms     3.92ms     4.71ms
//! processeur                     3.00ms     0.00ms     3.00ms
//! ```
//!
//! Lu trop vite, ce tableau condamne la carte graphique : 4,71 ms contre 3,00. C'est ce que
//! la première version du banc affichait, et c'était faux — elle chronométrait l'appel entier
//! sans distinguer les deux.
//!
//! Ce qui l'a démenti tient en une mesure : **à quatre fois moins de pixels, le chemin
//! processeur tombe de 3,00 à 0,62 ms, et le chemin graphique ne bouge pas.** Un coût qui
//! ignore la surface n'est pas un coût de surface. Les 3,92 ms sont le compositeur qui
//! régule à la fréquence de l'écran — et pendant ce temps le fil dort, il ne consomme rien.
//!
//! Le chiffre qui compte est donc la première colonne : **0,79 ms contre 3,00**, soit deux
//! millisecondes et deux dixièmes rendues au budget de chaque image. Sur un écran à 240 Hz,
//! dont le budget est de 4,16 ms, c'est plus de la moitié.
//!
//! # Ce que le GPU supprime, et ce qu'il ne supprime pas
//!
//! Il ne rend pas la scène : `tiny-skia` continue de le faire, sur le processeur. Il change
//! seulement ce qui arrive **après**, et sur deux points précis :
//!
//! * **la conversion disparaît**. `tiny-skia` rend du RGBA huit bits, et c'est exactement ce
//!   qu'une texture `Rgba8Unorm` attend : les octets partent tels quels. Le chemin CPU, lui,
//!   doit les réécrire un par un dans le `0RGB` que la fenêtre réclame ;
//! * **la remise au système change de nature**. Au lieu d'une copie vers GDI, l'image devient
//!   une texture que le compositeur affiche — et le transfert peut se recouvrir avec la suite.
//!
//! Sur un processeur graphique intégré, où la mémoire est partagée avec le processeur, il n'y
//! a même pas de transfert : l'écriture se fait dans la même mémoire.
//!
//! # Pourquoi le chemin processeur reste, et reste testé
//!
//! Un repli n'est un repli que s'il fonctionne le jour où on en a besoin. Celui-ci n'est pas
//! du code de secours jamais exécuté : c'est lui que les tests et les bancs utilisent, faute
//! de fenêtre, et c'est lui qui sert partout où aucun adaptateur graphique ne répond —
//! machine virtuelle, bureau distant, pilote absent. Le jour où le GPU échoue, l'application
//! ne s'arrête pas : elle le dit et continue.

pub mod banc_gpu;
pub mod bandes;
pub mod couches;
pub mod fond_gpu;
pub mod gpu;
pub mod lueurs_gpu;
pub mod scene_gpu;

pub use gpu::GpuPresenter;

use crate::error::{DesktopError, DesktopResult};
use std::num::NonZeroU32;
use std::sync::Arc;
use tiny_skia::Pixmap;
use winit::window::Window;

/// Ce qui met une image à l'écran.
///
/// La frontière est volontairement étroite — deux gestes, redimensionner et présenter. Tout
/// ce qui distingue les deux chemins vit derrière ; rien n'en sort.
pub trait Presenter {
    /// Accorde la surface à la taille de la fenêtre.
    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> DesktopResult<()>;

    /// Met cette image à l'écran.
    fn present(&mut self, pixmap: &Pixmap) -> DesktopResult<()>;

    /// **Cette présentation sait-elle poser les photos elle-même ?**
    ///
    /// Quand elle le sait, l'application lui donne la scène en trois temps plutôt qu'en une
    /// image finie : ce qui passe sous les photos, les photos, ce qui passe dessus. Le
    /// filtrage est alors câblé dans le silicium, et il ne coûte rien à aucune échelle.
    fn pose_les_photos(&self) -> bool {
        false
    }

    /// Présente la scène en cinq temps. N'est appelée que si [`Presenter::pose_les_photos`].
    ///
    /// L'ordre est tout, et il est celui du modèle : **le fond**, **les lueurs**, la couche du
    /// dessous (membranes et dossiers), **les photos**, la couche du dessus. Les intervertir
    /// mettrait une membrane par-dessus la photo qu'elle contient.
    ///
    /// `source` donne les pixels d'une photo que la carte ne connaît pas encore : elle ne les
    /// demande qu'une fois, au premier affichage, et jamais plus.
    ///
    /// Le **budget** est ce qui sépare cette image du plancher de la charte : ce qu'on
    /// s'autorise à rendre de textures manquantes avant de reporter le reste (CASCADE-2).
    fn presenter_en_couches(
        &mut self,
        _dessous: &Pixmap,
        (_confie, _budget): (&crate::renderer::Confie, std::time::Duration),
        _source: &dyn Fn(&str) -> Option<Pixmap>,
        _dessus: &Pixmap,
    ) -> DesktopResult<()> {
        Err(DesktopError::WindowError(
            "cette presentation ne pose pas les photos".into(),
        ))
    }

    /// Comment cette présentation s'appelle, pour le dire à qui veut le savoir.
    fn nom(&self) -> &'static str;

    /// Comment les images se succèdent devant l'écran.
    ///
    /// # Pourquoi cela remonte jusqu'à la chronique
    ///
    /// Les mêmes durées ne veulent pas dire la même chose selon que la présentation attend le
    /// balayage ou non. Une chronique qui ne le dit pas ne se relit pas — et c'est exactement
    /// ce qui est arrivé : l'application a tourné sans synchronisation verticale pendant toute
    /// son histoire, la bannière de démarrage le disait, et aucune trace ne le portait.
    fn rythme(&self) -> &'static str {
        "presentation par le systeme"
    }
}

/// Un pixel de `tiny-skia` dans le format que la fenêtre attend.
///
/// # Une opération par pixel plutôt que six
///
/// La version précédente lisait trois octets et les recomposait à coups de décalages et de
/// `ou`. Celle-ci dit la même chose en une fois : un pixel vaut `r,g,b,a` en mémoire, donc
/// `a<<24 | b<<16 | g<<8 | r` lu comme un mot ; l'échanger bout à bout donne
/// `r<<24 | g<<16 | b<<8 | a`, et un décalage de huit bits laisse exactement `r<<16 | g<<8 | b`.
/// Le processeur a une instruction pour l'échange d'octets, et le compilateur peut la
/// vectoriser — ce qu'une recomposition octet par octet lui interdit.
///
/// Mesuré : **7,83 → 5,47 ms en 4K**, 1,58 → 1,06 ms en 1080p.
pub fn pixel_fenetre(px: [u8; 4]) -> u32 {
    u32::from_le_bytes(px).swap_bytes() >> 8
}

/// La présentation par le processeur : convertir chaque pixel, puis remettre le tampon.
pub struct CpuPresenter {
    // `context` n'est jamais relu, mais la surface en dépend : le lâcher la briserait.
    _context: softbuffer::Context<Arc<Window>>,
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
}

impl CpuPresenter {
    pub fn new(window: Arc<Window>) -> DesktopResult<Self> {
        let context = softbuffer::Context::new(window.clone())
            .map_err(|e| DesktopError::WindowError(format!("softbuffer::Context : {e}")))?;
        let surface = softbuffer::Surface::new(&context, window)
            .map_err(|e| DesktopError::WindowError(format!("softbuffer::Surface : {e}")))?;
        Ok(Self {
            _context: context,
            surface,
        })
    }
}

impl Presenter for CpuPresenter {
    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> DesktopResult<()> {
        self.surface
            .resize(width, height)
            .map_err(|e| DesktopError::WindowError(format!("surface.resize : {e}")))
    }

    fn present(&mut self, pixmap: &Pixmap) -> DesktopResult<()> {
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|e| DesktopError::WindowError(format!("buffer_mut : {e}")))?;
        let (src, _) = pixmap.data().as_chunks::<4>();
        for (dst, chunk) in buffer.iter_mut().zip(src) {
            *dst = pixel_fenetre(*chunk);
        }
        crate::perf::stage("blit");
        buffer
            .present()
            .map_err(|e| DesktopError::WindowError(format!("present : {e}")))?;
        crate::perf::stage("present");
        Ok(())
    }

    fn nom(&self) -> &'static str {
        "processeur"
    }

    /// `softbuffer` remet le tampon au système, qui l'affiche quand il l'entend : rien ici ne
    /// décide du balayage, et prétendre le contraire serait une mesure inventée.
    fn rythme(&self) -> &'static str {
        "remise au systeme de fenetrage"
    }
}

#[cfg(test)]
mod tests;
