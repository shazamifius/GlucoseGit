//! **Copier, couper, coller des nœuds** (fiche 51 § 2) : la sélection entière, d'une fenêtre à
//! l'autre.
//!
//! # Le piège que ce module ferme
//!
//! `Ctrl+C` ne copiait que du **texte** — le contenu des cartes, le nom interne des images —, et
//! `Ctrl+X` supprimait pourtant **toute** la sélection : couper une sélection riche la détruisait
//! sans qu'aucun collage puisse la rendre. Le lot emporte désormais tout ce que la sélection
//! emporte ([`glucose_core::store::Store::extraire_la_selection`]), images comprises, et couper
//! ne retire rien tant que le lot n'est pas **dans** le presse-papiers.
//!
//! # Rien de lourd sur le fil qui dessine (COLLER-1)
//!
//! Un lot porte les octets de ses photos : les lire, les hacher, les assembler peut prendre des
//! dizaines de millisecondes. La copie se prépare donc sur un fil à part, comme le décodage du
//! collage ; le fil qui dessine ne fait que confier le résultat au presse-papiers. Un `Ctrl+V`
//! tapé avant que la copie soit prête attend son tour au lieu de coller l'ancien contenu.
//!
//! # D'où vient un lot
//!
//! Chaque lot est daté à la nanoseconde de sa copie ([`Project::created_at`], qui ne sert à rien
//! d'autre dans un lot). Une fenêtre reconnaît ainsi le sien : il vient de **ce document**, et ses
//! liens vers les onglets et les domaines d'ici tiennent. Tout autre lot vient d'ailleurs.

use crate::app::GlucoseApp;
use crate::interactions::presse_papiers;
use crate::persist::objets::Source;
use glucose_core::persist::GlucoseFile;
use glucose_core::store::{inventaire, Inventaire, Provenance};
use glucose_core::types::{AssetStore, Project};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;

/// Ce qui se prépare hors du fil qui dessine, et ce qu'il faut s'en rappeler.
#[derive(Default)]
pub struct Echanges {
    copie: Option<Copie>,
    collage: Option<Collage>,
    /// « Copier l'image » qui se prépare (fiche 51 § 3).
    pub(super) image: Option<Receiver<Result<Option<super::menu_image::ImagePosee>, String>>>,
    /// Un `Ctrl+V` arrivé pendant que la copie se préparait : où il visait.
    collage_attendu: Option<(String, (f64, f64))>,
    /// La date du dernier lot copié ici, et le document d'où il venait.
    derniere_copie: Option<(i64, Option<PathBuf>)>,
}

impl Echanges {
    /// Une copie ou un collage se prépare-t-il ?
    pub fn en_cours(&self) -> bool {
        self.copie.is_some() || self.collage.is_some() || self.image.is_some()
    }
}

struct Copie {
    recu: Receiver<Vec<u8>>,
    texte: Option<String>,
    compte: usize,
    /// Couper : ce qu'il faudra retirer, **une fois** le lot dans le presse-papiers.
    a_retirer: Option<(String, Vec<String>, Vec<String>)>,
}

struct Collage {
    recu: Receiver<Option<GlucoseFile>>,
    board: String,
    centre: (f64, f64),
}

/// Ce qu'un fil de fond a rendu, ou pas encore.
enum Reponse<T> {
    Prete(T),
    PasEncore,
    /// Le fil est tombé sans répondre.
    Perdue,
}

fn recevoir<T>(recu: &Receiver<T>, attendre: bool) -> Reponse<T> {
    if attendre {
        return recu.recv().map_or(Reponse::Perdue, Reponse::Prete);
    }
    match recu.try_recv() {
        Ok(t) => Reponse::Prete(t),
        Err(TryRecvError::Empty) => Reponse::PasEncore,
        Err(TryRecvError::Disconnected) => Reponse::Perdue,
    }
}

fn maintenant_en_nanos() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX))
}

impl GlucoseApp {
    /// `Ctrl+C` et `Ctrl+X` hors saisie : la sélection entière part vers le presse-papiers.
    pub(crate) fn copy_selection(&mut self, couper: bool) {
        let board = self.store.project.active_board_id.clone();
        let Some(mut lot) = self.store.extraire_la_selection(&board) else {
            return;
        };
        let date = maintenant_en_nanos();
        lot.created_at = date;
        self.echanges.derniere_copie = Some((date, self.disque.objets.document()));
        let Inventaire {
            images,
            annotations,
            cles,
        } = inventaire(&lot);
        let compte = images.len() + annotations.len();
        let objets = Arc::clone(&self.disque.objets);
        let (envoi, recu) = channel();
        std::thread::spawn(move || {
            let mut actifs = AssetStore::new();
            for cle in cles {
                if let Some(octets) = objets.lire_en_attendant(&cle) {
                    actifs.insert(cle, octets);
                }
            }
            let _ = envoi.send(glucose_core::persist::encode(&lot, &actifs, 0));
        });
        self.echanges.copie = Some(Copie {
            recu,
            texte: self.store.selection_as_text(),
            compte,
            a_retirer: couper.then_some((board, images, annotations)),
        });
    }

    /// `Ctrl+V` : pose le lot du presse-papiers, s'il en porte un. Rend `false` sinon, et le
    /// collage ordinaire — une image, un texte — prend la suite.
    pub(crate) fn coller_un_lot(&mut self, board: &str, centre: (f64, f64)) -> bool {
        if self.echanges.copie.is_some() {
            self.echanges.collage_attendu = Some((board.to_string(), centre));
            return true;
        }
        let Some(octets) = presse_papiers::ouvrir().ok().and_then(|mut a| a.lot()) else {
            return false;
        };
        let (envoi, recu) = channel();
        std::thread::spawn(move || {
            let _ = envoi.send(glucose_core::persist::decode(&octets).ok());
        });
        self.echanges.collage = Some(Collage {
            recu,
            board: board.to_string(),
            centre,
        });
        true
    }

    /// **Applique ce que le fond a fini.** `attendre` : les épreuves veulent le résultat ; la
    /// boucle, elle, repasse tant que [`Echanges::en_cours`] le dit.
    pub(crate) fn suivre_les_echanges(&mut self, attendre: bool) {
        if let Some(copie) = &self.echanges.copie {
            match recevoir(&copie.recu, attendre) {
                Reponse::PasEncore => return,
                Reponse::Prete(octets) => self.finir_la_copie(Some(octets)),
                Reponse::Perdue => self.finir_la_copie(None),
            }
        }
        if let Some((board, centre)) = self.echanges.collage_attendu.take() {
            self.coller_un_lot(&board, centre);
        }
        if let Some(collage) = &self.echanges.collage {
            match recevoir(&collage.recu, attendre) {
                Reponse::PasEncore => {}
                Reponse::Prete(fichier) => self.finir_le_collage(fichier),
                Reponse::Perdue => self.finir_le_collage(None),
            }
        }
        if let Some(image) = &self.echanges.image {
            let rendu = match recevoir(image, attendre) {
                Reponse::PasEncore => return,
                Reponse::Prete(rendu) => Some(rendu),
                Reponse::Perdue => None,
            };
            self.echanges.image = None;
            self.finir_l_image(rendu);
        }
    }

    /// Une copie dont le fil de fond n'a rien rendu — ce qu'une épreuve ne sait pas provoquer
    /// autrement.
    #[cfg(test)]
    pub(crate) fn finir_la_copie_pour_l_epreuve(&mut self, octets: Option<Vec<u8>>) {
        self.finir_la_copie(octets);
    }

    fn finir_la_copie(&mut self, octets: Option<Vec<u8>>) {
        let Some(copie) = self.echanges.copie.take() else {
            return;
        };
        let ecrit = octets
            .ok_or_else(|| "la copie n'a pas abouti".to_string())
            .and_then(|o| {
                presse_papiers::ouvrir().and_then(|mut a| a.ecrire_un_lot(copie.texte, o))
            });
        if let Err(e) = ecrit {
            // Rien n'est retiré : couper sans avoir copié perdrait la sélection.
            self.echec_presse_papiers(e);
            return;
        }
        let verbe = if copie.a_retirer.is_some() {
            "coupé"
        } else {
            "copié"
        };
        let s = if copie.compte > 1 { "s" } else { "" };
        self.ui
            .show_toast(format!("{} nœud{s} {verbe}{s}", copie.compte));
        if let Some((board, images, annotations)) = copie.a_retirer {
            self.store.set_selected_image_ids(images);
            self.store.set_selected_annotation_ids(annotations);
            self.store.delete_selected(&board);
        }
        self.mark_dirty();
    }

    fn finir_le_collage(&mut self, fichier: Option<GlucoseFile>) {
        let Some(collage) = self.echanges.collage.take() else {
            return;
        };
        let Some(fichier) = fichier else {
            self.echec_presse_papiers("ce lot de nœuds est illisible");
            return;
        };
        let provenance = self.provenance_du_lot(&fichier.project);
        let mut lot = fichier.project;
        self.accueillir_les_images(&mut lot, &fichier.assets, &fichier.manifest, provenance);
        // Sans message : le lot apparaît sous le curseur, sélectionné, et cela se regarde.
        self.store
            .coller_un_lot(&collage.board, lot, collage.centre, provenance);
        self.mark_dirty();
    }

    /// Ce lot est-il le dernier que cette fenêtre a copié, depuis ce même document ?
    fn provenance_du_lot(&self, lot: &Project) -> Provenance {
        match &self.echanges.derniere_copie {
            Some((date, document))
                if *date == lot.created_at && *document == self.disque.objets.document() =>
            {
                Provenance::CeDocument
            }
            _ => Provenance::Ailleurs,
        }
    }

    /// **Chaque image du lot reçoit une clé d'ici.** Une image de ce document garde la sienne ;
    /// des octets que le document porte déjà reprennent la clé qui les porte ; les autres
    /// entrent sous une clé faite de leur empreinte, et le scribe les scellera comme un dépôt.
    fn accueillir_les_images(
        &mut self,
        lot: &mut Project,
        actifs: &AssetStore,
        manifeste: &glucose_core::persist::manifest::Manifest,
        provenance: Provenance,
    ) {
        let objets = Arc::clone(&self.disque.objets);
        let empreintes: HashMap<&str, [u8; 32]> = manifeste
            .assets
            .iter()
            .map(|a| (a.key.as_str(), a.digest))
            .collect();
        let ici: HashMap<[u8; 32], String> = self
            .store
            .project
            .toutes_les_images()
            .filter_map(|i| i.src.clone())
            .filter_map(|src| Some((objets.empreinte(&src)?, src)))
            .collect();
        for img in lot.toutes_les_images_mut() {
            let Some(src) = img.src.clone() else {
                continue;
            };
            if provenance == Provenance::CeDocument && objets.source(&src).is_some() {
                continue;
            }
            let Some(empreinte) = empreintes.get(src.as_str()) else {
                continue;
            };
            let cle = ici.get(empreinte).cloned().unwrap_or_else(|| {
                let cle = format!("lot:{}", hexa(empreinte));
                if let (None, Some(octets)) = (objets.source(&cle), actifs.get(&src)) {
                    objets.poser(&cle, Source::Memoire(Arc::new(octets.to_vec())));
                }
                cle
            });
            img.src = Some(cle);
        }
    }
}

fn hexa(octets: &[u8; 32]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}
