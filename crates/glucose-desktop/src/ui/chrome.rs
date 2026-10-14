//! **La chrome permanente — la bande, le fil d'Ariane, la minimap, le rail — écrite une fois,
//! posée par deux voies** (CHROME-1, fiche 64).
//!
//! # Ce que la mesure a dit
//!
//! Sur la voie graphique, la chrome se recomposait **à chaque image** dans la couche du dessus :
//! la bande, la minimap et le rail recopiés depuis leurs caches, puis la couche effacée,
//! relevée et envoyée à la carte. Au téléphone (1080 × 2400, interface à 275 %), 556 lignes
//! sur 2 400 rail replié, 1 865 rail déplié — **2,4 à 8 Mo par image**, pour une chrome qui
//! n'avait pas changé (fiche 63 § 7). Sur son PC, effacer, relever et envoyer faisaient la
//! moitié du travail d'une image où l'on déplace la vue (sa chronique du 13/10).
//!
//! # Ce que font les meilleurs
//!
//! Chromium sépare l'*invalidation* — ce qui doit être rastérisé de nouveau — du *dommage* —
//! ce que l'écran doit recomposer : déplacer une couche ou changer son opacité crée du dommage
//! et **aucun travail de rastérisation** (`cc/README.md`). Android garde une liste d'affichage
//! par `RenderNode`, rejouée telle quelle tant que son contenu ne change pas ; Flutter isole ce
//! qui ne bouge pas derrière un `RepaintBoundary`, dont la texture se réutilise.
//!
//! Glucose fait de même : chaque morceau de la chrome est une **texture** que la carte détient
//! et pose à chaque image, et que le processeur ne refait que quand son cache change. Ce qui
//! bouge à chaque image — le rectangle de la vue sur la minimap — est un **nombre** : quatre
//! filets que la carte pose sans qu'aucune image ne parte.
//!
//! # Une seule liste, deux exécutants
//!
//! [`parcourir`] dit l'ordre — et les marques de mesure — une fois : la voie processeur compose
//! chaque morceau dans son tampon ([`Morceau::poser`]), la voie graphique les reçoit
//! ([`ChromeConfiee`]). Deux listes finiraient par ne plus poser la même chose.

use super::{breadcrumb, minimap, rail, UiState};
use crate::params::Pointer;
use crate::present::scene_gpu::Pose;
use crate::renderer::voies::APoser;
use crate::renderer::zone::Rectangle;
use crate::theme::Theme;
use crate::typography::Typography;
use glucose_core::report::Melange;
use glucose_core::store::Store;
use std::rc::Rc;
use tiny_skia::{Pixmap, PixmapMut};

/// **Un morceau de la chrome**, dans l'ordre où il se pose.
pub enum Morceau<'a> {
    /// Un tampon tenu à jour par son cache : ce qu'il montre ne change que quand `dessins`
    /// change.
    Tampon {
        nom: &'static str,
        dessins: u64,
        pixmap: &'a Rc<Pixmap>,
        origine: (f32, f32),
        melange: Melange,
    },
    /// Des filets sur la grille de pixels : le rectangle de la vue, sur la minimap.
    Cadre(Vec<Rectangle>),
}

impl Morceau<'_> {
    /// **La voie processeur** : le morceau composé dans le tampon de l'image.
    pub fn poser(&self, pixmap: &mut PixmapMut) {
        match self {
            Self::Tampon {
                pixmap: tampon,
                origine,
                melange,
                ..
            } => {
                crate::composition::poser(pixmap, tampon, *origine, *melange);
            }
            Self::Cadre(filets) => {
                for f in filets {
                    crate::renderer::scale::fill_crisp(pixmap, f.rect, f.couleur);
                }
            }
        }
    }
}

/// **Tient chaque morceau de la chrome à jour, et le donne à `recevoir`, dans l'ordre** : la
/// bande, le fil d'Ariane, la minimap — son fond puis le rectangle de la vue —, le rail.
///
/// L'ordre est celui du dessin d'avant CHROME-1, et il compte : le rail déplié passe sur la
/// minimap quand l'écran est court ou la fenêtre étroite — pas sur son téléphone debout, où il
/// s'arrête bien au-dessus (`tests/chrome_suite.rs`).
pub fn parcourir(
    store: &Store,
    ui: &mut UiState,
    (typo, theme): (&Typography, &Theme),
    (taille, pointer): ((f32, f32), Pointer),
    mut recevoir: impl FnMut(Morceau<'_>),
) {
    // Un message expiré s'en va avant de décider quoi que ce soit.
    if ui.current_toast.as_ref().is_some_and(|t| t.is_expired()) {
        ui.current_toast = None;
    }
    // Le mode référence : rien de permanent. Le toast et le menu, qui attendent une décision,
    // se posent après (`ui::decision`).
    if ui.sans_interface() {
        return;
    }
    let (w, _) = taille;
    // **Refaite, ou seulement reposée ?** Le poste `bande` valait 0,22 ms en médian et 6,95 ms
    // sur une image de zoom du terrain : un écart que seul un redessin explique.
    let avant = ui.bande_cache.as_ref().map_or(0, |c| c.dessins);
    super::bande::tenir_la_bande(store, ui, typo, theme, w, pointer);
    let apres = ui.bande_cache.as_ref().map_or(0, |c| c.dessins);
    crate::perf::compteur("bande_refaite", f64::from(u8::from(apres != avant)));
    if let Some(m) = super::bande::morceau(ui) {
        recevoir(m);
    }
    crate::perf::stage("bande");

    let echelle = (w, ui.header_height(), ui.scale_factor);
    breadcrumb::tenir_le_fil(store, (typo, theme), echelle, &mut ui.ariane_cache);
    if let Some(m) = breadcrumb::morceau(store, &ui.ariane_cache) {
        recevoir(m);
    }
    // **Le fil d'Ariane a sa marque** : sans elle, il tombait dans `minimap` — la quatrième
    // marque de ce dépôt à absorber ce qui la précède (fiche 19 § 4.4, 22 § 5.4, 23 § 2).
    crate::perf::stage("ariane");

    let retrait = (ui.marges.droite, ui.marges.sous());
    let mise = (ui.scale_factor, retrait);
    if let Some(mb) = minimap::tenir_la_minimap(store, theme, taille, mise, &mut ui.minimap_cache) {
        let morceaux = minimap::morceaux(&ui.minimap_cache, theme, (&mb, ui.scale_factor));
        morceaux.into_iter().flatten().for_each(&mut recevoir);
    }
    rail::tenir_le_rail(ui, typo, theme, taille);
    if let Some(m) = rail::morceau(ui) {
        recevoir(m);
    }
    crate::perf::stage("minimap");
}

/// **La chrome telle que la carte la pose** (CHROME-1) : des textures sous le rectangle de la
/// vue, ce rectangle, puis des textures par-dessus — et de quoi lire leurs pixels sans copie.
#[derive(Debug, Default, Clone)]
pub struct ChromeConfiee {
    sous_le_cadre: Vec<APoser>,
    cadre: Vec<Rectangle>,
    sur_le_cadre: Vec<APoser>,
    /// Le cadre est-il passé ? Ce qui vient après lui se pose par-dessus.
    cadre_recu: bool,
    /// Les pixels de chaque texture, par sa clé : ceux des caches eux-mêmes, partagés.
    pixels: Vec<(String, Rc<Pixmap>)>,
}

impl ChromeConfiee {
    /// **Tient la chrome à jour et la confie à la carte.**
    pub fn de(
        store: &Store,
        ui: &mut UiState,
        kit: (&Typography, &Theme),
        ecran: ((f32, f32), Pointer),
    ) -> Self {
        let mut confiee = Self::default();
        parcourir(store, ui, kit, ecran, |m| confiee.recevoir(m));
        confiee
    }

    fn recevoir(&mut self, morceau: Morceau<'_>) {
        match morceau {
            Morceau::Tampon {
                nom,
                dessins,
                pixmap,
                origine,
                ..
            } => {
                let cle = format!("chrome:{nom}@{dessins}");
                // La position que la voie processeur emploie : arrondie, d'où le chemin exact
                // (`composition::poser`). Le mélange n'a pas à suivre : la carte compose tout,
                // et la bande, qui remplace, est opaque.
                let pose = Pose {
                    x: origine.0.round(),
                    y: origine.1.round(),
                    largeur: pixmap.width() as f32,
                    hauteur: pixmap.height() as f32,
                    opacite: 1.0,
                    angle: 0.0,
                    fenetre: Pose::TOUT,
                    bornes: Pose::PARTOUT,
                };
                let a_poser = APoser {
                    identite: format!("chrome:{nom}"),
                    cle: cle.clone(),
                    pose,
                    repli: None,
                };
                self.pixels.push((cle, Rc::clone(pixmap)));
                if self.cadre_recu {
                    self.sur_le_cadre.push(a_poser);
                } else {
                    self.sous_le_cadre.push(a_poser);
                }
            }
            Morceau::Cadre(filets) => {
                self.cadre.extend(filets);
                self.cadre_recu = true;
            }
        }
    }

    /// Les textures posées sous le rectangle de la vue : la bande, le fil, le fond de la minimap.
    pub fn sous_le_cadre(&self) -> &[APoser] {
        &self.sous_le_cadre
    }

    /// Le rectangle de la vue, en filets.
    pub fn cadre(&self) -> &[Rectangle] {
        &self.cadre
    }

    /// Les textures posées par-dessus : le rail.
    pub fn sur_le_cadre(&self) -> &[APoser] {
        &self.sur_le_cadre
    }

    /// Les pixels de la texture que cette clé désigne, s'ils sont de cette image.
    pub fn pixels(&self, cle: &str) -> Option<tiny_skia::PixmapRef<'_>> {
        self.pixels
            .iter()
            .find(|(c, _)| c == cle)
            .map(|(_, p)| p.as_ref().as_ref())
    }
}

#[cfg(test)]
mod tests;
