//! **Un dossier de son PC dans Glucose** (DISQUE-1 et DISQUE-2, fiche 164) : lâché sur la fenêtre, il
//! devient un dossier de Glucose qui montre ce qu'il contient — ses sous-dossiers, ses photos, ses
//! textes, ses fichiers en tuiles —, **un niveau à la fois**, lu quand on y entre ; et **vivant** :
//! relu à chaque entrée, et quand la fenêtre reprend la main.
//!
//! * **Le fil de l'image ne lit jamais le disque** : l'atelier ([`atelier`]) lit le niveau, le noyau
//!   le réconcilie avec ce que le tableau montre ([`glucose_core::miroir_disque::reconcilier`]), le
//!   magasin l'applique en un geste signé « Glucose ». Le dossier lâché n'est lu qu'à l'entrée,
//!   comme ses sous-dossiers : une seule loi pour tous les niveaux, et un `Ctrl+Z` après le dépôt
//!   l'emporte entier.
//! * **Les photos restent sur le disque** tant qu'elles montrent un fichier du miroir : l'écriture
//!   du document ne les scelle pas (`persist::ecriture`, [`glucose_core::miroir_disque::reste_dehors`]).
//!   Une photo qui en sort — glissée, copiée sur son canevas —, ou qu'il y colle, est sienne, et se
//!   scelle. Leurs aperçus vivent dans le cache, nommés par leur taille et leur date
//!   (`renderer::apercu`) ; un fichier changé se redécode.
//! * **Rien ne s'écrit sur son disque** : retirer un nœud du miroir ne supprime aucun fichier — et
//!   le nœud ne revient pas, tant que son fichier ne change pas : ce que chaque miroir a vu se garde
//!   dans le dossier de l'application (`miroirs/`), par document.
//! * **La première vue d'un miroir lu montre tout ce qu'il contient**, jamais plus grand que nature :
//!   sans elle, la caméra d'un tableau neuf posait l'origine sous la bande, et le haut du dossier s'y
//!   cachait (vu au pilote le 25/11) ; 600 photos n'en montraient que six.

use crate::app::GlucoseApp;
use crate::interactions::vol::cadrage_du_contenu;
use glucose_core::file_kind;
use glucose_core::geometry::Rect;
use glucose_core::membrane_focus::ScreenSize;
use glucose_core::miroir_disque::reconcilier::{ecrire_les_vus, empreinte, reconcilier, Vus};
use glucose_core::miroir_disque::{masquee, source_en_attente, trier, Entree, Sorte};
use glucose_core::types::{CanvasFolder, FolderSortMode, Viewport};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

mod appliquer;
pub(crate) mod atelier;

/// **Les miroirs, vus du fil de l'image.**
#[derive(Debug, Default)]
pub struct Miroirs {
    /// Le tableau regardé à l'image d'avant : entrer, c'est le voir changer.
    tableau_vu: String,
    atelier: Option<atelier::Atelier>,
    /// Les tableaux dont une lecture est en route : une seule à la fois chacun.
    en_lecture: HashSet<String>,
    /// Une lecture revenue pendant un geste ouvert : elle attend qu'il se ferme.
    en_attente: Vec<atelier::Lu>,
    /// Ce que chaque miroir de ce document a vu, par tableau.
    vus: HashMap<String, Vus>,
    /// Le document auquel ce qui précède appartient.
    document: String,
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
    /// **Un dossier lâché devient son miroir**, à lire quand on y entrera — comme ses sous-dossiers.
    /// Rend `false` s'il ne s'ouvre pas : le dépôt en fera une tuile qui y mène.
    pub(crate) fn poser_un_miroir(
        &mut self,
        board: &str,
        chemin: &Path,
        (x, y): (f64, f64),
    ) -> bool {
        if std::fs::read_dir(chemin).is_err() {
            return false;
        }
        let nom = super::drop::file_name_of(chemin);
        let mut dossier = CanvasFolder::new(self.store.generate_id("folder"), nom, "");
        (dossier.x, dossier.y) = (x, y);
        dossier.mirror_source = Some(source_en_attente(&chemin.to_string_lossy(), None));
        self.store
            .create_folder_with_content(board, dossier, Vec::new());
        true
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

    /// **Entrer dans un miroir le relit** — appelé avant chaque image, il ne fait rien tant que le
    /// tableau regardé ne change pas : un `Ctrl+Z` qui vide le miroir où l'on est ne le relit pas
    /// aussitôt, et ne s'y enferme donc jamais.
    pub(crate) fn lire_le_miroir_ou_l_on_entre(&mut self) {
        if self.store.active_board_id() == self.miroirs.tableau_vu {
            return;
        }
        self.miroirs.tableau_vu = self.store.active_board_id().to_string();
        self.relire_le_miroir();
    }

    /// **Relit le miroir où l'on se trouve**, s'il y en a un et qu'aucune lecture n'est en route :
    /// à l'entrée, et quand la fenêtre reprend la main (on revient peut-être de l'explorateur, ou
    /// d'un rendu). Le disque se lit sur l'atelier ; [`Self::suivre_les_miroirs`] applique ce qu'il rend.
    pub(crate) fn relire_le_miroir(&mut self) {
        let Some(miroir) = self.store.miroir_ouvert() else {
            return;
        };
        let document = self.document_des_miroirs();
        if miroir.tableau != self.store.active_board_id()
            || !self.miroirs.en_lecture.insert(miroir.tableau.clone())
        {
            return;
        }
        let demande = atelier::Demande {
            vus: self.miroirs.vus.remove(&miroir.tableau),
            fichier: self.fichier_des_vus(&miroir.tableau),
            document,
            miroir,
        };
        if self.miroirs.atelier.is_none() {
            self.miroirs.atelier = atelier::Atelier::ouvrir(self.lancement.reveil.clone());
        }
        let tableau = demande.miroir.tableau.clone();
        if !self
            .miroirs
            .atelier
            .as_ref()
            .is_some_and(|a| a.lire(demande))
        {
            self.miroirs.en_lecture.remove(&tableau);
        }
    }

    /// **Ce que l'atelier a lu s'applique** — appelé avant chaque image ; pendant un geste ouvert,
    /// la lecture attend qu'il se ferme.
    pub(crate) fn suivre_les_miroirs(&mut self) {
        if let Some(a) = &self.miroirs.atelier {
            self.miroirs.en_attente.extend(a.recu.try_iter());
        }
        if self.miroirs.en_attente.is_empty() || self.store.in_live_edit() {
            return;
        }
        for lu in std::mem::take(&mut self.miroirs.en_attente) {
            self.appliquer_la_lecture(lu);
        }
    }

    /// **Le document des miroirs** : celui qu'on regarde. Un autre ouvert, ce qu'on savait des
    /// miroirs de l'autre s'oublie (il est gardé dans leurs fichiers).
    fn document_des_miroirs(&mut self) -> String {
        let document = self.fichier_de_ce_document();
        if document != self.miroirs.document {
            self.miroirs.vus.clear();
            self.miroirs.document = document.clone();
        }
        document
    }

    /// **Où ce miroir garde ce qu'il a vu** : dans le dossier de l'application, sous l'empreinte de
    /// son document et de son tableau. Un document sans fichier ne le garde qu'en mémoire.
    fn fichier_des_vus(&self, tableau: &str) -> Option<PathBuf> {
        let document = &self.disque.ecriture.as_ref()?.chemin;
        let cle = empreinte(&format!("{}\n{tableau}", document.display()));
        Some(self.disque.miroirs.join(format!("{cle:016x}")))
    }

    /// **Une lecture revenue** : réconciliée avec ce que le tableau montre maintenant, appliquée en
    /// un geste, et ce que le miroir a vu retenu — en mémoire, et dans son fichier.
    fn appliquer_la_lecture(&mut self, lu: atelier::Lu) {
        let miroir = &lu.demande.miroir;
        self.miroirs.en_lecture.remove(&miroir.tableau);
        if lu.demande.document != self.document_des_miroirs() {
            return;
        }
        let entrees = match &lu.entrees {
            Ok(e) => e,
            Err(raison) => return self.dire_l_illisible(lu.demande.miroir, lu.vus, raison),
        };
        let Some(tableau) = self.store.board(&miroir.tableau) else {
            return;
        };
        let frais: HashMap<String, u64> = lu
            .textes
            .iter()
            .map(|(c, m)| (c.clone(), empreinte(m)))
            .collect();
        let racine = &miroir.source.root_path;
        let plan = reconcilier(tableau, racine, lu.vus.as_ref(), entrees, &frais);
        let a_oublier = self.photos_qui_changent(&miroir.tableau, &plan);
        let r = self.reconciliation(&miroir.tableau, &plan, &lu.textes, miroir.source.sort_by);
        let proportion = self.proportion_de_l_ecran();
        let fait = self.store.reconcilier_le_miroir(
            &miroir.parent,
            &miroir.dossier,
            r,
            proportion,
            maintenant(),
        );
        if let Some(premiere) = fait {
            self.mark_dirty();
            self.oublier_les_pixels(&a_oublier);
            if premiere {
                self.poser_la_premiere_vue(&miroir.tableau);
            }
        }
        let change = lu.vus.as_ref() != Some(&plan.vus);
        self.retenir_les_vus(&lu.demande, plan.vus, change);
    }

    /// Un miroir qui ne se lit pas (un disque débranché, une permission) : ses nœuds restent tels
    /// qu'ils sont — rien n'a disparu, on ne sait rien. S'il n'a jamais été lu, il le dit, et reste
    /// « à lire » : y revenir, le disque rebranché, le lira. Le message est celui du dépôt : lire un
    /// miroir à l'entrée, c'est la suite du dépôt, différée (et le cliquet 2 ne compte pas un site
    /// de plus).
    fn dire_l_illisible(
        &mut self,
        miroir: glucose_core::store::miroir_disque::MiroirOuvert,
        vus: Option<Vus>,
        raison: &str,
    ) {
        if let Some(v) = vus {
            self.miroirs.vus.insert(miroir.tableau.clone(), v);
        }
        if miroir.source.pending_scan {
            let message = format!("« {} » ne se lit pas : {raison}", miroir.source.root_path);
            self.dire_le_depot(message);
        }
    }

    /// Retient ce que le miroir a vu : en mémoire, et dans son fichier s'il a changé.
    fn retenir_les_vus(&mut self, demande: &atelier::Demande, vus: Vus, change: bool) {
        let tableau = demande.miroir.tableau.clone();
        if let (true, Some(fichier), Some(a)) = (change, &demande.fichier, &self.miroirs.atelier) {
            a.ecrire(fichier.clone(), ecrire_les_vus(&vus));
        }
        self.miroirs.vus.insert(tableau, vus);
    }

    /// **La première vue d'un miroir qu'une lecture vient de remplir** : une vue qu'on s'est
    /// faite ne se reprend jamais — elle ne se pose qu'à la lecture qui le remplit.
    fn poser_la_premiere_vue(&mut self, tableau: &str) {
        let Some(contenu) = self.store.content_bounds(tableau) else {
            return;
        };
        let (largeur, hauteur) = self.taille_de_la_fenetre();
        let ecran = ScreenSize {
            width: f64::from(largeur),
            height: f64::from(hauteur),
        };
        let vue = premiere_vue(contenu, ecran, f64::from(self.ui.header_height()));
        self.store.set_viewport(tableau, vue);
    }
}

#[cfg(test)]
mod tests;
