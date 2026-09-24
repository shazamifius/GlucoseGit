//! **Ouvrir un document de Glucose Tauri** : le lire, le traduire, montrer ses images tout de
//! suite, et les sceller dans le premier fichier qu'il aura.
//!
//! Le document importé n'a **pas de nom** : il n'est pas encore un document de Glucose Rust.
//! Son fichier Tauri n'est jamais réécrit — Glucose Tauri doit pouvoir le rouvrir tel qu'il
//! l'a laissé. `Ctrl+S` demande où l'enregistrer ; le premier geste ouvre un brouillon.
//!
//! Les images restent d'abord **là où Tauri les rangeait** — son magasin, ou le dossier
//! `objects/` d'un document portable — et s'affichent depuis là. Le scribe les copie dans le
//! fichier de Glucose Rust dès qu'il existe (voir [`super::ecriture`]).

use super::objets::Source;
use crate::app::GlucoseApp;
use crate::error::{DesktopError, DesktopResult};
use glucose_core::persist::tauri::{self, projet::Provenance};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Ce qu'un import a trouvé, et ce qu'il n'a pas pu garder.
#[derive(Debug, Default, PartialEq)]
pub struct Importe {
    pub nb_images: usize,
    pub nb_annotations: usize,
    /// Images dont les octets sont introuvables sur cette machine.
    pub manquantes: usize,
    /// Ce que le modèle de Glucose Rust ne porte pas encore.
    pub omis: Vec<(&'static str, usize)>,
    /// Octets de fin ignorés : un enregistrement de Tauri interrompu.
    pub fin_ignoree: usize,
}

impl GlucoseApp {
    /// Importe le document Tauri `octets`, lu depuis `chemin`.
    pub(crate) fn importer_de_tauri(
        &mut self,
        chemin: &Path,
        octets: &[u8],
    ) -> DesktopResult<Importe> {
        let (valeur, fin_ignoree) = tauri::lire(octets).map_err(DesktopError::from)?;
        let (mut projet, rapport) = tauri::projet::traduire(&valeur);
        if !self.fermer_le_document() {
            self.disque.ecriture = None;
        }
        self.disque.objets.vider();
        let dossier = chemin.parent();
        let magasin = crate::tauri::magasin_tauri();
        let provenances: HashMap<&str, &Provenance> = rapport
            .provenances
            .iter()
            .map(|(id, p)| (id.as_str(), p))
            .collect();
        let mut importe = Importe {
            fin_ignoree,
            omis: rapport.omis.iter().map(|(k, v)| (*k, *v)).collect(),
            ..Importe::default()
        };
        importe.nb_annotations = projet.nombre_d_annotations();
        for img in projet.toutes_les_images_mut() {
            importe.nb_images += 1;
            let prov = provenances.get(img.id.as_str()).copied();
            let cle = self.placer(&img.id, prov, dossier, magasin.as_deref());
            importe.manquantes += usize::from(cle.is_none());
            img.src = cle;
        }
        self.adopter_un_import(projet);
        Ok(importe)
    }

    /// La clé d'une image importée, et d'où ses octets se liront en attendant d'être scellés.
    fn placer(
        &mut self,
        id: &str,
        provenance: Option<&Provenance>,
        dossier: Option<&Path>,
        magasin: Option<&Path>,
    ) -> Option<String> {
        match provenance? {
            Provenance::Octets(octets) => {
                let cle = format!("tauri:{id}");
                let octets = Arc::new(octets.clone());
                self.disque
                    .objets
                    .poser(&cle, Source::Memoire(Arc::clone(&octets)));
                self.disque.a_sceller.push((cle.clone(), octets));
                Some(cle)
            }
            p => {
                let chemin = crate::tauri::localiser(p, dossier, magasin)?;
                Some(chemin.to_string_lossy().into_owned())
            }
        }
    }

    /// Fait du projet importé le document courant, sans fichier.
    fn adopter_un_import(&mut self, projet: glucose_core::types::Project) {
        self.editing_session = None;
        self.historique_du_texte.oublier();
        self.selection_box = None;
        self.dock_manager.domains.reset();
        self.store.load_project(projet);
        self.store.journal.prendre_les_ecrits();
        self.fit_all_text_cards();
        self.store.bump_version();
        self.disque.depart = Some(self.store.project.clone());
        self.project_path = None;
        // Un document importé n'est pas encore enregistré en Glucose Rust : il est « modifié »,
        // et fermer sans l'enregistrer posera la question.
        self.saved_version = self.store.version.wrapping_sub(1);
    }
}

/// Le toast d'un import : ce qui est venu, ce qui manque, ce qui n'a pas pu venir.
pub fn message_d_import(nom: &str, i: &Importe) -> String {
    let mut msg = format!(
        "« {nom} » importé de Glucose Tauri — {} image(s), {} annotation(s)",
        i.nb_images, i.nb_annotations
    );
    if i.manquantes > 0 {
        msg.push_str(&format!(
            ", {} image(s) introuvable(s) sur cette machine",
            i.manquantes
        ));
    }
    for (quoi, n) in &i.omis {
        msg.push_str(&format!(", {n} × {quoi} non repris"));
    }
    if i.fin_ignoree > 0 {
        msg.push_str(", la fin d'un enregistrement interrompu ignorée");
    }
    msg.push_str(" — Ctrl+S pour l'enregistrer en Glucose Rust");
    msg
}
