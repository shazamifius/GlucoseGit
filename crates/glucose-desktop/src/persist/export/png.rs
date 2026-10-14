//! **L'export en image** (EXPORT-PNG-1, fiche 62).
//!
//! # Ce qu'on exporte : la sélection, sinon le tableau
//!
//! La règle d'Ordonner : une sélection vide veut dire tout le tableau. Choisir trois images et
//! exporter donne **ces trois images, dans leur disposition** — son idée du 09/10 (fiche 59) ;
//! ce qui n'est pas choisi n'y est pas, même posé entre elles. Glucose Tauri n'exportait que le
//! tableau entier.
//!
//! # À quelle taille
//!
//! **À 100 %** : une unité du canevas, un pixel — la composition telle qu'on la voit au zoom
//! 100 %, comme PureRef, où 100 % est la taille réelle des images. Une image se pose à sa taille
//! réelle jusqu'à six cents unités (`taille_posee`) : jusque-là, elle sort pixel pour pixel.
//! Glucose Tauri rendait deux fois plus grand, plafonné à 8 000 pixels de côté et à 40 millions
//! de pixels — trois nombres réglés à l'œil. Ici, aucun plafond : la taille est celle du
//! contenu, et la mémoire ne la borne pas (plus bas).
//!
//! La marge autour du contenu n'est pas choisie non plus (Tauri : 120) : c'est **ce que le
//! contenu peint au-delà de ses boîtes**. Une photo ne déborde pas de la sienne : des photos
//! seules sortent bord à bord. Une annotation peut déborder — la lueur d'une carte, la courbe
//! ou la pointe d'une flèche — et la portée de la lueur au repos, au-delà de laquelle plus aucun
//! pixel ne change, borne ce débord.
//!
//! # Par bandes, au fil de l'eau, sur un fil à soi
//!
//! L'image se dessine par **bandes de la surface d'un écran** — le travail pour lequel le
//! moteur de rendu est fait —, et chaque bande s'écrit dans le fichier dès qu'elle est faite
//! (`png`, decisions/12) : la mémoire tient une bande, quelle que soit l'image. Le tout sur un
//! fil à lui, avec son propre moteur : l'écran garde sa cadence pendant l'export, et rien de ce
//! que l'export décode ne chasse ce que l'écran tient. Le fil réveille la boucle à chaque bande,
//! qui dit où il en est ; le fichier ne prend sa place qu'entier (SAVE-1).
//!
//! # Ce qu'il ne porte pas
//!
//! Ni la grille de points, ni le cadre de la sélection, ni les poignées : du contenu, seulement.

use crate::app::GlucoseApp;
use crate::persist::atomic;
use crate::plateforme::Reveil;
use crate::renderer::atelier::Partage;
use crate::renderer::Renderer;
use crate::theme::Theme;
use crate::ui::UiState;
use glucose_core::store::Store;
use glucose_core::types::Viewport;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc};

/// **À 100 %** : une unité du canevas, un pixel.
pub const ECHELLE: f64 = 1.0;

/// Ce qu'un export dessine : un document réduit à ce qu'on exporte, la vue qui place son contenu
/// dans l'image, et la taille de l'image.
pub struct Scene {
    pub(super) store: Store,
    pub(super) vue: Viewport,
    pub largeur: u32,
    pub hauteur: u32,
}

/// **Ce qu'on exporte** : la sélection si elle existe, sinon tout le tableau ; `None` pour un
/// tableau vide.
pub fn preparer(store: &Store) -> Option<Scene> {
    let tableau = store.project.active_board_id.clone();
    let choisi = store.selection_bounds(&tableau);
    let region = match choisi {
        Some(r) => r,
        None => store.content_bounds(&tableau)?,
    };
    let scene = store.copie_a_exporter(choisi.is_some());
    let marge = if scene.a_des_annotations() {
        crate::renderer::export::debord(ECHELLE)
    } else {
        0.0
    };
    let cote = |l: f64| (l * ECHELLE + 2.0 * marge).ceil().max(1.0) as u32;
    Some(Scene {
        store: scene,
        vue: Viewport {
            scale: ECHELLE,
            x: marge - region.left * ECHELLE,
            y: marge - region.top * ECHELLE,
        },
        largeur: cote(region.width),
        hauteur: cote(region.height),
    })
}

/// Ce que le moteur de l'export emprunte à celui de l'écran : le thème, et de quoi lire les
/// images comme lui.
pub struct Peintre {
    pub theme: Theme,
    pub atelier: Partage,
}

/// Ce que le fil de l'export dit à la boucle.
pub enum Nouvelle {
    /// Tant de rangées sont écrites.
    Rangees(u32),
    /// Fini : le poids du fichier, ou ce qui l'a empêché.
    Fini(Result<u64, String>),
}

/// **Un export en cours** : son fichier, combien de rangées il écrira en tout, ce qu'il dira
/// de lui à la fin — « 1192 × 843 pixels », « 5 planches » —, et ce que son fil dit.
pub struct EnCours {
    recu: mpsc::Receiver<Nouvelle>,
    pub chemin: PathBuf,
    total: u32,
    resume: String,
}

impl EnCours {
    pub fn new(
        recu: mpsc::Receiver<Nouvelle>,
        chemin: PathBuf,
        (total, resume): (u32, String),
    ) -> Self {
        Self {
            recu,
            chemin,
            total,
            resume,
        }
    }

    /// Les nouvelles arrivées depuis la dernière fois, sans attendre.
    pub fn nouvelles(&self) -> Vec<Nouvelle> {
        self.recu.try_iter().collect()
    }
}

/// **Un travail d'export sur un fil à lui** — une image, ou des planches (fiche 68) : il dit
/// les rangées écrites, puis sa fin, et réveille la boucle à chaque nouvelle.
pub fn lancer_le_travail(
    reveil: Option<Reveil>,
    travail: impl FnOnce(&mut dyn FnMut(u32)) -> std::io::Result<u64> + Send + 'static,
) -> mpsc::Receiver<Nouvelle> {
    let (envoi, recu) = mpsc::channel();
    let dire = move |nouvelle| {
        let _ = envoi.send(nouvelle);
        if let Some(r) = &reveil {
            r();
        }
    };
    let fil = std::thread::Builder::new()
        .name("glucose-export".into())
        .spawn(move || {
            let mut rangees = |n| dire(Nouvelle::Rangees(n));
            let fin = travail(&mut rangees);
            dire(Nouvelle::Fini(fin.map_err(|e| e.to_string())));
        });
    // Un fil que le système refuse dit son échec par le même chemin qu'un export qui échoue.
    match fil {
        Ok(_) => recu,
        Err(e) => {
            let (envoi, recu) = mpsc::channel();
            let _ = envoi.send(Nouvelle::Fini(Err(e.to_string())));
            recu
        }
    }
}

/// **Le moteur d'un export** : le sien, avec le thème de l'écran et son atelier prêté.
pub fn moteur(peintre: Peintre) -> Renderer {
    let mut moteur = Renderer::new();
    moteur.theme = peintre.theme;
    moteur.magasin.atelier.partager_avec(peintre.atelier);
    moteur
}

/// Dessine la scène bande après bande et l'écrit au fil de l'eau ; rend le poids du fichier.
fn ecrire(
    scene: Scene,
    peintre: Peintre,
    chemin: &Path,
    surface: u64,
    rangees: &mut dyn FnMut(u32),
) -> std::io::Result<u64> {
    let Scene {
        store,
        vue,
        largeur,
        hauteur,
    } = scene;
    let mut moteur = moteur(peintre);
    let bande = (surface / u64::from(largeur)).clamp(1, u64::from(hauteur)) as u32;
    atomic::ecrire_en_flux(chemin, |sortie| {
        rendre_en_png(
            &mut moteur,
            &store,
            (vue, largeur, hauteur),
            bande,
            sortie,
            rangees,
        )
    })?;
    Ok(std::fs::metadata(chemin)?.len())
}

/// **Rend cette vue en PNG, bande après bande, dans `sortie`** — un fichier, ou la mémoire d'une
/// planche (fiche 68) : rouge, vert, bleu sur huit bits, la forme qu'un PDF pose telle quelle.
/// `rangees` dit combien de rangées sont écrites.
pub fn rendre_en_png(
    moteur: &mut Renderer,
    store: &Store,
    (vue, largeur, hauteur): (Viewport, u32, u32),
    bande: u32,
    sortie: impl Write,
    rangees: &mut dyn FnMut(u32),
) -> std::io::Result<()> {
    let mut ui = UiState::new();
    let mut encodeur = png::Encoder::new(sortie, largeur, hauteur);
    encodeur.set_color(png::ColorType::Rgb);
    encodeur.set_depth(png::BitDepth::Eight);
    let mut ecrivain = encodeur.write_header().map_err(std::io::Error::other)?;
    let mut flux = ecrivain.stream_writer().map_err(std::io::Error::other)?;
    let mut y = 0;
    while y < hauteur {
        let h = bande.min(hauteur - y);
        let mut pixmap = tiny_skia::Pixmap::new(largeur, h)
            .ok_or_else(|| std::io::Error::other("une bande trop grande pour la mémoire"))?;
        let ici = Viewport {
            y: vue.y - f64::from(y),
            ..vue
        };
        moteur.rendre_pour_l_export(&mut pixmap.as_mut(), store, &mut ui, ici);
        flux.write_all(&en_rvb(&pixmap))?;
        y += h;
        rangees(y);
    }
    flux.finish().map_err(std::io::Error::other)
}

/// Les pixels prémultipliés d'une bande, en rouge, vert, bleu : le fond est opaque, donc
/// l'image aussi.
fn en_rvb(pixmap: &tiny_skia::Pixmap) -> Vec<u8> {
    pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue()]
        })
        .collect()
}

impl GlucoseApp {
    /// **Exporte en image** la sélection, ou le tableau, à `chemin` — sur un fil à soi ; la
    /// boucle relève où il en est ([`Self::suivre_l_export`]).
    pub fn exporter_en_image(&mut self, chemin: PathBuf) {
        if self.un_export_attend() {
            return;
        }
        let Some(scene) = preparer(&self.store) else {
            self.ui.show_toast("Rien à exporter : ce tableau est vide");
            return;
        };
        let resume = format!("{} × {} pixels", scene.largeur, scene.hauteur);
        let total = scene.hauteur;
        let (peintre, surface) = (self.peintre_de_l_export(), self.surface_d_une_bande());
        let ici = chemin.clone();
        let travail =
            move |rangees: &mut dyn FnMut(u32)| ecrire(scene, peintre, &ici, surface, rangees);
        let recu = lancer_le_travail(self.reveil_de_l_export(), travail);
        self.attentes.export = Some(EnCours::new(recu, chemin, (total, resume)));
        self.dire_l_avancee(0, total);
    }

    /// **Un export est-il déjà en cours ?** Il le dit — un seul à la fois, une image ou des
    /// planches.
    pub(crate) fn un_export_attend(&mut self) -> bool {
        let attend = self.attentes.export.is_some();
        if attend {
            self.ui.show_toast("Un export est déjà en cours");
        }
        attend
    }

    /// Ce que le moteur d'un export emprunte à celui de l'écran.
    pub(crate) fn peintre_de_l_export(&self) -> Peintre {
        Peintre {
            theme: self.renderer.theme.clone(),
            atelier: self.renderer.magasin.atelier.a_partager(),
        }
    }

    /// **Une bande d'export a la surface de la fenêtre** : le travail pour lequel le moteur est
    /// fait.
    pub(crate) fn surface_d_une_bande(&self) -> u64 {
        let (l, h) = self.taille_de_la_fenetre();
        (f64::from(l) * f64::from(h)).max(1.0) as u64
    }

    /// De quoi réveiller la boucle depuis le fil de l'export.
    pub(crate) fn reveil_de_l_export(&self) -> Option<Reveil> {
        self.lancement.reveil.clone().map(|proxy| -> Reveil {
            Arc::new(move || {
                let _ = proxy.send_event(());
            })
        })
    }

    /// Un export en image est-il en cours ?
    pub fn exporte_encore(&self) -> bool {
        self.attentes.export.is_some()
    }

    /// **Ce que dit le fil de l'export** — relevé à chaque réveil de la boucle : où il en est,
    /// puis ce qu'il a écrit, ou ce qui l'en a empêché.
    pub fn suivre_l_export(&mut self) {
        let Some(export) = self.attentes.export.as_ref() else {
            return;
        };
        let (nouvelles, total) = (export.nouvelles(), export.total);
        let resume = export.resume.clone();
        let nom = export
            .chemin
            .file_name()
            .map_or_else(|| "l'image".into(), |n| n.to_string_lossy().into_owned());
        let mut faites = None;
        for nouvelle in nouvelles {
            match nouvelle {
                Nouvelle::Rangees(n) => faites = Some(n),
                Nouvelle::Fini(fin) => {
                    let chemin = self.attentes.export.take().map(|e| e.chemin);
                    if let (Ok(_), Some(chemin)) = (&fin, &chemin) {
                        self.offrir_si_partage(chemin);
                    }
                    self.ui.show_toast(match fin {
                        Ok(octets) => format!(
                            "{nom} exporté — {resume}, {}",
                            crate::persist::human_size(octets as usize)
                        ),
                        Err(e) => format!("L'export n'a pas pu s'écrire : {e}"),
                    });
                    return;
                }
            }
        }
        if let Some(n) = faites {
            self.dire_l_avancee(n, total);
        }
    }

    /// Où en est l'export : la part des rangées écrites — au départ, zéro.
    pub(crate) fn dire_l_avancee(&mut self, ecrites: u32, total: u32) {
        let pourcent = u64::from(ecrites) * 100 / u64::from(total.max(1));
        self.ui.show_toast(format!("Export : {pourcent} %"));
    }
}

#[cfg(test)]
mod tests;
