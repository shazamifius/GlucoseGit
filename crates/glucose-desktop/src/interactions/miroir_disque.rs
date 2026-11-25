//! **Un dossier de son PC dans Glucose** (DISQUE-1, fiche 164) : lâché sur la fenêtre, il devient un
//! dossier de Glucose qui montre ce qu'il contient — ses sous-dossiers, ses photos, ses textes, ses
//! fichiers en tuiles —, **un niveau à la fois** : un sous-dossier se lit quand on y entre.
//!
//! * **Les photos restent sur le disque** tant qu'elles vivent dans le tableau d'un miroir : l'écriture
//!   du document ne les scelle pas (`persist::ecriture`, [`glucose_core::miroir_disque::tableaux_du_disque`]).
//!   Une photo qui en sort — glissée, copiée sur son canevas — devient sienne, et se scelle. Leurs
//!   aperçus vivent dans le cache, nommés par leur taille et leur date (`renderer::apercu`).
//! * **Rien ne s'écrit sur son disque** : retirer un nœud du miroir ne supprime aucun fichier.
//! * **La lecture à l'entrée est un geste** de l'histoire (le noyau : `store::miroir_disque`), qui
//!   dit quand le disque a été lu ; elle n'a lieu **qu'au moment d'entrer** — un `Ctrl+Z` qui vide le
//!   dossier où l'on se trouve ne le relit pas aussitôt, et ne s'y enferme donc jamais.
//!
//! * **La première vue d'un miroir lu montre tout ce qu'il contient**, jamais plus grand que nature :
//!   sans elle, la caméra d'un tableau neuf posait l'origine sous la bande, et le haut du dossier s'y
//!   cachait (vu au pilote le 25/11) ; 600 photos n'en montraient que six.
//!
//! La loi (ce qui se tait, l'ordre, la disposition) est dans le noyau ; ici, le disque.

use crate::app::GlucoseApp;
use crate::interactions::vol::cadrage_du_contenu;
use glucose_core::file_kind;
use glucose_core::geometry::Rect;
use glucose_core::membrane_focus::ScreenSize;
use glucose_core::miroir_disque::{masquee, source_en_attente, trier, Entree, Sorte};
use glucose_core::store::miroir_disque::Niveau;
use glucose_core::types::{AssetRef, BoardImage, CanvasFolder, FolderSortMode, Viewport};
use std::path::Path;
use std::time::{Instant, UNIX_EPOCH};

/// **Le tableau regardé à l'image d'avant** : entrer, c'est le voir changer.
#[derive(Debug, Default)]
pub struct Miroirs {
    tableau_vu: String,
}

/// **Lit un niveau d'un dossier** : chaque entrée qu'on montre, rangée selon son tri. Une entrée qui
/// ne se lit pas (un lien cassé, une permission) est passée : le reste se montre.
pub(crate) fn lire_un_niveau(
    dossier: &Path,
    tri: Option<FolderSortMode>,
) -> std::io::Result<Vec<Entree>> {
    let mut entrees = Vec::new();
    for e in std::fs::read_dir(dossier)?.flatten() {
        let nom = e.file_name().to_string_lossy().into_owned();
        let chemin = e.path();
        // `metadata` suit les liens : un lien vers un dossier est un dossier.
        let Ok(meta) = std::fs::metadata(&chemin) else {
            continue;
        };
        let est_un_dossier = meta.is_dir();
        if masquee(&nom, est_un_dossier) {
            continue;
        }
        let date = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_secs() as i64);
        entrees.push(Entree {
            sorte: sorte_de(&chemin, &nom, est_un_dossier),
            nom,
            chemin: chemin.to_string_lossy().into_owned(),
            taille: if est_un_dossier { 0 } else { meta.len() },
            date,
        });
    }
    trier(&mut entrees, tri);
    Ok(entrees)
}

/// Ce qu'une entrée est : une photo se reconnaît à son en-tête, que le décodeur lit — comme le dépôt
/// (`drop.rs`), jamais à son extension.
fn sorte_de(chemin: &Path, nom: &str, est_un_dossier: bool) -> Sorte {
    if est_un_dossier {
        return Sorte::Dossier;
    }
    if file_kind::readable(nom).is_some() {
        return Sorte::Lisible;
    }
    match super::dimensions::dimensions_du_fichier(chemin) {
        Ok((largeur, hauteur)) => Sorte::Photo { largeur, hauteur },
        Err(_) => Sorte::Autre,
    }
}

/// **Tout le contenu, jamais plus grand que nature** : le cadrage de `Alt+F`, ramené à l'échelle 1
/// quand il grossirait — une seule photo ne remplit pas l'écran pour avoir été lue. Le centre du
/// contenu reste au centre de l'écran utile.
pub(crate) fn premiere_vue(contenu: Rect, ecran: ScreenSize, bandeau: f64) -> Viewport {
    let vue = cadrage_du_contenu(contenu, ecran, bandeau);
    if vue.scale <= 1.0 {
        return vue;
    }
    let centre = (
        contenu.left + contenu.width / 2.0,
        contenu.top + contenu.height / 2.0,
    );
    Viewport {
        scale: 1.0,
        x: vue.x + centre.0 * (vue.scale - 1.0),
        y: vue.y + centre.1 * (vue.scale - 1.0),
    }
}

fn maintenant() -> i64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

impl GlucoseApp {
    /// **Un dossier lâché devient son miroir**, son premier niveau déjà lu, dans le geste du dépôt.
    /// Rend `false` s'il ne se lit pas : le dépôt en fera une tuile qui y mène.
    pub(crate) fn poser_un_miroir(
        &mut self,
        board: &str,
        chemin: &Path,
        (x, y): (f64, f64),
    ) -> bool {
        let Ok(niveau) = self.lire_le_niveau(chemin, None) else {
            return false;
        };
        let nom = super::drop::file_name_of(chemin);
        let mut dossier = CanvasFolder::new(self.store.generate_id("folder"), nom, "");
        (dossier.x, dossier.y) = (x, y);
        dossier.mirror_source = Some(source_en_attente(&chemin.to_string_lossy(), None));
        let id = self
            .store
            .create_folder_with_content(board, dossier, Vec::new());
        self.remplir(board, &id, niveau)
    }

    /// **« Dossier du disque… »** : le sélecteur du système, et le dossier choisi se lâche là où
    /// l'on regarde — le même geste, le même message que s'il était déposé. Le téléphone n'a pas
    /// encore de quoi choisir un dossier (le SAF, fiche 164 § 4) : la commande n'y est pas offerte
    /// (`commandes::applique`).
    pub(crate) fn choisir_un_dossier_du_disque(&mut self) {
        self.demander_un_fichier(
            crate::dialogue::fichier(),
            crate::dialogue::Mode::Dossier,
            crate::persist::choix::Demande::DossierDuDisque,
        );
    }

    /// **Le dossier miroir où l'on vient d'entrer**, lu s'il attendait — appelé avant chaque image,
    /// il ne fait rien tant que le tableau regardé ne change pas.
    pub(crate) fn lire_le_miroir_ou_l_on_entre(&mut self) {
        if self.store.project.active_board_id == self.miroirs.tableau_vu {
            return;
        }
        self.miroirs.tableau_vu = self.store.project.active_board_id.clone();
        let Some((parent, id, source)) = self.store.miroir_a_lire() else {
            return;
        };
        let niveau = match self.lire_le_niveau(Path::new(&source.root_path), source.sort_by) {
            Ok(niveau) => niveau,
            Err(raison) => {
                // Il reste « à lire » : y revenir, le disque rebranché, le lira. Le message est
                // celui du dépôt — lire un sous-dossier à l'entrée, c'est la suite du dépôt du
                // dossier, différée (et le cliquet 2 ne compte pas un site de plus).
                let message = format!("« {} » ne se lit pas : {raison}", source.root_path);
                self.dire_le_depot(message);
                return;
            }
        };
        if self.remplir(&parent, &id, niveau) {
            self.mark_dirty();
        }
    }

    /// **Un niveau lu remplit son miroir**, et la caméra de son tableau prend sa première vue —
    /// au dépôt comme à l'entrée. Elle ne se pose qu'à la lecture qui le remplit : une vue qu'on
    /// s'est faite ne se reprend jamais.
    fn remplir(&mut self, parent: &str, dossier: &str, niveau: Niveau) -> bool {
        let proportion = self.proportion_de_l_ecran();
        let Some(tableau) =
            self.store
                .remplir_le_miroir(parent, dossier, niveau, proportion, maintenant())
        else {
            return false;
        };
        let Some(contenu) = self.store.content_bounds(&tableau) else {
            return true;
        };
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        let ecran = ScreenSize {
            width: f64::from(largeur),
            height: f64::from(hauteur),
        };
        let vue = premiere_vue(contenu, ecran, f64::from(self.ui.header_height()));
        self.store.set_viewport(&tableau, vue);
        true
    }

    /// **Un niveau du disque, lu en nœuds**, chacun né comme au dépôt ; la disposition est celle du
    /// noyau. Lu sur le fil de l'image, au dépôt comme à l'entrée : `GLUCOSE_PERF=1` dit ce que la
    /// lecture lui coûte (`miroir_lu`).
    fn lire_le_niveau(
        &mut self,
        dossier: &Path,
        tri: Option<FolderSortMode>,
    ) -> std::io::Result<Niveau> {
        let debut = Instant::now();
        let entrees = lire_un_niveau(dossier, tri)?;
        let mut niveau = Niveau::default();
        for e in &entrees {
            let chemin = Path::new(&e.chemin);
            match e.sorte {
                Sorte::Dossier => {
                    let mut f = CanvasFolder::new(self.store.generate_id("folder"), &*e.nom, "");
                    f.mirror_source = Some(source_en_attente(&e.chemin, tri));
                    niveau.dossiers.push(f);
                }
                Sorte::Photo { largeur, hauteur } => {
                    niveau.photos.push(self.photo_liee(e, (largeur, hauteur)));
                }
                Sorte::Lisible => match self.carte_lisible(chemin, &e.nom, (0.0, 0.0)) {
                    Some(carte) => niveau.textes.push(carte),
                    None => niveau.tuiles.push(self.tuile(chemin, &e.nom, (0.0, 0.0))),
                },
                Sorte::Autre => niveau.tuiles.push(self.tuile(chemin, &e.nom, (0.0, 0.0))),
            }
        }
        crate::perf::event("miroir_lu", debut);
        Ok(niveau)
    }

    /// **Une photo du disque, liée** : sa clé est son chemin, comme une photo déposée ; son `asset`
    /// dit qu'elle reste où elle est.
    fn photo_liee(&mut self, e: &Entree, (largeur, hauteur): (u32, u32)) -> BoardImage {
        let origine = (f64::from(largeur), f64::from(hauteur));
        let (w, h) = super::clipboard::taille_posee(origine);
        let mut img = BoardImage::new(self.store.generate_id("img"), 0.0, 0.0, w, h);
        (img.original_width, img.original_height) = origine;
        img.src = Some(e.chemin.clone());
        img.asset = Some(AssetRef::Link {
            href: e.chemin.clone(),
            sha256: None,
            size_bytes: Some(e.taille),
        });
        img
    }
}

#[cfg(test)]
mod tests;
