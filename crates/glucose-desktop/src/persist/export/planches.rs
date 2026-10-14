//! **L'export en planches** (P2, proposition 7, fiche 68) : le storyboard courant, planche après
//! planche — une image par planche, ou un PDF.
//!
//! # Ce que le canevas montre est la planche
//!
//! Un storyboard est rangé par sa grille : sa planche de rang `p` est **le rectangle du canevas**
//! qui porte ses rangées `[p·R, (p+1)·R)` — toutes ses colonnes, avec leurs cadres, leurs bandes,
//! leurs numéros, leurs sigles et leurs images (`Disposition::planches`). Elle se rend par
//! l'écrivain de l'export en image (EXPORT-PNG-1), au même 100 % : ce qu'on imprime est ce qu'on
//! voit. Storyboarder et Krita composent leurs pages par un second moteur, qui doit refaire
//! chaque choix du premier ; ici, aucun. Et comme chez Storyboarder, une planche ne mélange
//! jamais deux storyboards.
//!
//! # PNG ou PDF
//!
//! Le nom choisi le dit, comme pour Exporter : `.pdf`, un fichier, une page par planche
//! ([`super::pdf`]) ; `.png`, une image par planche — `nom-1.png`, `nom-2.png`… quand il y en a
//! plusieurs. Au téléphone, le PDF part à la feuille de partage : un seul fichier, que Fichiers,
//! une conversation ou une imprimante prennent.

use super::png::{self, EnCours, Peintre, ECHELLE};
use crate::app::GlucoseApp;
use crate::persist::atomic;
use glucose_core::store::Store;
use glucose_core::types::Viewport;
use std::io;
use std::path::{Path, PathBuf};

/// **Ce qu'un export en planches dessine** : un document copié, la vue de chaque planche, et
/// leur taille commune en pixels.
pub struct Planches {
    store: Store,
    vues: Vec<Viewport>,
    pub largeur: u32,
    pub hauteur: u32,
}

/// **Les planches du storyboard `sb`** du tableau actif — `None` s'il n'existe pas.
pub fn preparer(store: &Store, sb: &str) -> Option<Planches> {
    let board = store.project.active_board_id.clone();
    let boites = store.disposition_actuelle(&board, sb)?.planches();
    let (_, _, l, h) = *boites.first()?;
    let cote = |v: f64| (v * ECHELLE).ceil().max(1.0) as u32;
    let vues = boites
        .iter()
        .map(|&(x, y, _, _)| Viewport {
            scale: ECHELLE,
            x: -x * ECHELLE,
            y: -y * ECHELLE,
        })
        .collect();
    Some(Planches {
        store: store.copie_du_storyboard(sb),
        vues,
        largeur: cote(l),
        hauteur: cote(h),
    })
}

/// Un PDF, d'après son nom.
fn est_un_pdf(chemin: &Path) -> bool {
    chemin
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
}

/// **Le fichier de la planche `k`** : le nom choisi quand il n'y en a qu'une, sinon le nom suivi
/// de son numéro — `planches-1.png`, `planches-2.png`…
pub fn chemin_de_la_planche(chemin: &Path, k: usize, nombre: usize) -> PathBuf {
    if nombre <= 1 {
        return chemin.to_path_buf();
    }
    let nom = chemin
        .file_stem()
        .map_or_else(|| "planche".into(), |n| n.to_string_lossy().into_owned());
    chemin.with_file_name(format!("{nom}-{}.png", k + 1))
}

/// **Dessine chaque planche et l'écrit** : un PDF, ou une image par planche. Rend le poids de
/// ce qui est écrit.
fn ecrire(
    planches: Planches,
    peintre: Peintre,
    (chemin, surface): (&Path, u64),
    rangees: &mut dyn FnMut(u32),
) -> io::Result<u64> {
    let Planches {
        store,
        vues,
        largeur,
        hauteur,
    } = planches;
    let mut moteur = png::moteur(peintre);
    let bande = (surface / u64::from(largeur)).clamp(1, u64::from(hauteur)) as u32;
    let nombre = vues.len();
    let mut pages = Vec::new();
    let mut poids = 0;
    for (k, vue) in vues.into_iter().enumerate() {
        let taille = (vue, largeur, hauteur);
        let mut avancer = |n: u32| rangees(k as u32 * hauteur + n);
        if est_un_pdf(chemin) {
            let mut octets = Vec::new();
            png::rendre_en_png(
                &mut moteur,
                &store,
                taille,
                bande,
                &mut octets,
                &mut avancer,
            )?;
            let page = super::pdf::planche(&octets)
                .ok_or_else(|| io::Error::other("une planche qui n'est pas un PNG RVB"))?;
            pages.push(page);
        } else {
            let page = chemin_de_la_planche(chemin, k, nombre);
            atomic::ecrire_en_flux(&page, |sortie| {
                png::rendre_en_png(&mut moteur, &store, taille, bande, sortie, &mut avancer)
            })?;
            poids += std::fs::metadata(&page)?.len();
        }
    }
    if est_un_pdf(chemin) {
        atomic::ecrire_en_flux(chemin, |sortie| {
            super::pdf::ecrire(sortie, &pages).map(drop)
        })?;
        poids = std::fs::metadata(chemin)?.len();
    }
    Ok(poids)
}

impl GlucoseApp {
    /// **Exporte le storyboard courant en planches** : au PC, le sélecteur demande où — un PDF,
    /// ou des images ; au téléphone, un PDF part à la feuille de partage.
    ///
    /// Le bouton du panneau et l'entrée du menu n'existent qu'avec un storyboard : sans lui, il
    /// n'y a rien à dire, et rien n'est fait.
    pub(crate) fn exporter_les_planches(&mut self) {
        if self.storyboard_courant().is_none() {
            return;
        }
        let nom = format!("{} — storyboard", self.document_label());
        if crate::plateforme::partager::disponible() {
            let chemin = self.a_partager(&format!("{nom}.pdf"));
            self.exporter_les_planches_vers(chemin);
            return;
        }
        let sortie = crate::dialogue::fichier()
            .filtre("PDF", &["pdf"])
            .filtre("Images", &["png"])
            .nom(format!("{nom}.pdf"));
        let demande = crate::persist::choix::Demande::Planches;
        self.demander_un_fichier(sortie, crate::dialogue::Mode::Enregistrer, demande);
    }

    /// **Le choix revenu : les planches s'écrivent là**, sur un fil à elles — un PDF si le nom
    /// le dit ou n'en dit rien, des images sinon.
    pub fn exporter_les_planches_vers(&mut self, choisi: PathBuf) {
        if self.un_export_attend() {
            return;
        }
        let planches = self
            .storyboard_courant()
            .and_then(|sb| preparer(&self.store, &sb));
        let Some(planches) = planches else {
            return;
        };
        let chemin = match choisi.extension() {
            Some(e) if e.eq_ignore_ascii_case("png") || e.eq_ignore_ascii_case("pdf") => choisi,
            _ => choisi.with_extension("pdf"),
        };
        let nombre = planches.vues.len();
        let total = planches.hauteur * nombre as u32;
        let resume = if nombre == 1 {
            "une planche".to_string()
        } else {
            format!("{nombre} planches")
        };
        let peintre = self.peintre_de_l_export();
        let surface = self.surface_d_une_bande();
        let ici = chemin.clone();
        let travail =
            move |rangees: &mut dyn FnMut(u32)| ecrire(planches, peintre, (&ici, surface), rangees);
        let recu = png::lancer_le_travail(self.reveil_de_l_export(), travail);
        self.attentes.export = Some(EnCours::new(recu, chemin, (total, resume)));
    }
}

#[cfg(test)]
mod tests;
