//! Ce qu'on fait d'un dépôt venu d'un navigateur (DEPOT-WEB-1).
//!
//! # Il n'y a presque rien ici, et c'est le but
//!
//! [`crate::plateforme::depot_windows`] rend une moisson : les fichiers de l'explorateur, et ce
//! qu'une page a livré, en mémoire (DEPOT-4). C'est [`super::drop`] qui route l'un et l'autre,
//! et lui seul décide qu'un lot tient dans une entrée d'annulation.
//!
//! Ce module ne fait que deux choses que personne d'autre ne peut faire : **traduire la
//! position** que le système donne en pixels de l'écran vers les pixels de la zone de dessin,
//! et **poser une adresse** quand la page n'a rien promis de mieux.
//!
//! # L'adresse seule, et pourquoi on la pose quand même
//!
//! Un lien glissé depuis la barre d'adresse, ou une image que le navigateur refuse de livrer,
//! ne donne qu'une adresse. On pourrait ne rien faire ; ce serait exactement le geste sans
//! effet que DEPOT-WEB-1 corrige. On pose donc une carte portant l'adresse, que
//! [`super::links`] rend cliquable — et qui refuse déjà tout ce qui n'est ni `http://` ni
//! `https://`, donc rien de ce qu'une page dépose ici ne peut faire exécuter quoi que ce soit.

mod apercu;
mod relance;

pub use relance::{liens_choisis, Relances};

use crate::app::GlucoseApp;
use crate::params::Arrivage;
use crate::plateforme::moisson::{Depot, Moisson};

/// **Ce qui arrive du système par glisser-déposer**, et n'est pas encore posé.
#[derive(Default)]
pub struct Arrivees {
    /// Les fichiers déposés sur la fenêtre, en attente d'être posés **ensemble**.
    ///
    /// winit émet un `DroppedFile` **par fichier** : un lot de huit donne huit événements,
    /// tous poussés par le même appel système et donc tous présents avant le prochain
    /// `about_to_wait`. Les accumuler jusque-là reconstitue le lot — sans quoi chaque
    /// fichier se poserait comme s'il était seul, tous au même point, et le geste entier
    /// laisserait huit entrées d'annulation au lieu d'une.
    pub fichiers: Vec<std::path::PathBuf>,
    /// **Ce que le système dépose sur la fenêtre**, quand un pont natif existe (DEPOT-WEB-1).
    ///
    /// `winit` ne transmet que `CF_HDROP` — les chemins de l'explorateur — et un navigateur
    /// n'en donne jamais. La cible de dépôt du projet lit aussi les fichiers qu'une page
    /// PROMET, et la position où le curseur a lâché, que `winit` reçoit et jette.
    pub pont: Option<crate::plateforme::Depots>,
    /// **Les images annoncées**, pas encore livrées : leur marqueur est à l'écran
    /// (DEPOT-WEB-5).
    pub en_chemin: Vec<Arrivage>,
    /// Où va ce qu'une page livre et qui n'est pas une image (DEPOT-4) : les téléchargements
    /// de l'utilisateur, que **seul le vrai lancement** donne ([`GlucoseApp::habiter`]) — une
    /// épreuve n'écrit jamais chez lui.
    pub telechargements: Option<std::path::PathBuf>,
    /// **Les liens qui repartent chercher leur image** (DEPOT-WEB-6), au clic droit.
    pub relances: Relances,
    /// Les images que le dernier dépôt a posées, dans l'ordre : ce que la pose rend.
    pub posees: Vec<String>,
    /// **Les copies posées qui attendent leur original** (fiche 53 § 9), par numéro de
    /// rapatriement.
    pub copies: apercu::Copies,
}

impl GlucoseApp {
    /// **Ce que le pont natif et les relances ont apporté**, posé.
    ///
    /// Le pont écrit depuis la boucle de messages de Windows, au milieu d'un geste ; on pose
    /// ici, où le document n'est lu par personne. Un lot par dépôt : glisser huit images d'une
    /// page est **un** geste.
    ///
    /// **Jamais pendant un geste de la main** (DEPOT-WEB-6) : une image rapatriée arrive une
    /// seconde après le lâcher, souvent pendant qu'on glisse déjà autre chose — et sa pose,
    /// qui ouvre et ferme son propre geste, refermait ce glisser-là. Elle attend dans son
    /// canal que la main ait fini.
    pub(crate) fn relever_les_depots(&mut self) {
        if self.store.in_live_edit() {
            return;
        }
        let du_pont = self.depot.pont.as_ref().map(|d| d.recolter());
        let relances = self.depot.relances.recoltees();
        for depot in du_pont.unwrap_or_default().into_iter().chain(relances) {
            self.recevoir_le_depot(depot);
            self.provenance.noter_un_depot();
        }
    }

    /// **Ce que le pont de dépôt fait parvenir** : une annonce, ou une livraison.
    pub fn recevoir_le_depot(&mut self, depot: Depot) {
        match depot {
            Depot::EnChemin { numero, ou, hote } => self.annoncer_l_arrivage(numero, ou, hote),
            Depot::Pose { numero, moisson } => self.poser_le_depot(numero, &moisson),
            Depot::Ameliore { numero, recu } => self.remplacer_la_copie(numero, recu),
        }
    }

    /// **Un téléchargement commence** : son marqueur paraît au point de dépôt, et ce point
    /// se fige dans le monde — l'image s'y posera même si la vue bouge d'ici là.
    fn annoncer_l_arrivage(&mut self, numero: u64, ou: Option<(f64, f64)>, hote: String) {
        let monde = self.drop_origin(self.ecran_vers_client(ou));
        self.depot.en_chemin.push(Arrivage {
            numero,
            monde,
            hote,
        });
        self.mark_dirty();
    }

    /// **Pose ce qu'un dépôt du système vient d'apporter.**
    ///
    /// Un appel par moisson : glisser huit images d'une page est **un** geste, et deux dépôts
    /// successifs en sont deux. C'est `deposer` qui en fait une entrée d'annulation, et qui
    /// rend l'unique compte-rendu.
    ///
    /// Une livraison annoncée se pose au point que son annonce a figé, et retire son marqueur
    /// — dans la même image, pour qu'aucune ne montre ni l'un ni l'autre.
    pub fn poser_le_depot(&mut self, numero: Option<u64>, recolte: &Moisson) {
        let annonce = numero
            .and_then(|n| self.depot.en_chemin.iter().position(|a| a.numero == n))
            .map(|i| self.depot.en_chemin.remove(i));
        if self.livrer_une_relance(numero, recolte, annonce.as_ref().map(|a| a.monde)) {
            return;
        }
        let client = self.ecran_vers_client(recolte.ou);
        let posees = self.poser_une_moisson(recolte, client, annonce.map(|a| a.monde));
        if let (Some(numero), true) = (numero, recolte.apercu) {
            self.retenir_la_copie(numero, posees.first());
        }
    }

    /// **Pose une moisson lâchée en ce point de la fenêtre** — ou au point qu'une annonce a
    /// figé. À part de la conversion depuis l'écran, qui demande une fenêtre : c'est la
    /// décision, et elle s'éprouve sans.
    pub(crate) fn poser_une_moisson(
        &mut self,
        recolte: &Moisson,
        client: Option<(f64, f64)>,
        annonce: Option<(f64, f64)>,
    ) -> Vec<String> {
        let sur_les_onglets = annonce.is_none() && self.sur_les_onglets(client);
        let origine = annonce.unwrap_or_else(|| self.drop_origin(client));
        // Un lot glissé depuis une autre fenêtre de Glucose se colle là où l'on lâche.
        if let Some(lot) = &recolte.lot {
            let board = self.store.project.active_board_id.clone();
            self.coller_ces_octets(&board, origine, lot.clone());
            self.mark_dirty();
            return Vec::new();
        }
        // **Lâchés sur la barre d'onglets, les documents s'ajoutent** dans des onglets neufs
        // (BOARDS-2) ; le reste du lot se pose sur le canevas, comme d'habitude.
        let (documents, reste): (Vec<_>, Vec<_>) = recolte
            .chemins
            .iter()
            .cloned()
            .partition(|p| sur_les_onglets && est_un_document(p));
        for document in &documents {
            self.ajouter_un_document(document);
        }
        let lot = crate::interactions::drop::Lot::de(&reste, recolte.recus.clone(), &recolte.liens)
            .en_apercu(recolte.apercu)
            .avec_des_illisibles(recolte.illisibles);
        let posees = self.deposer_le_lot(lot, origine, recolte.echec.as_deref());
        self.mark_dirty();
        posees
    }

    /// Ce point, en pixels de la fenêtre, tombe-t-il dans la barre d'onglets ?
    fn sur_les_onglets(&self, client: Option<(f64, f64)>) -> bool {
        client.is_some_and(|(_, y)| {
            let y = y as f32;
            y >= self.ui.topbar_height() && y < self.ui.header_height()
        })
    }

    /// **Les pixels de l'écran, vus depuis le coin de la zone de dessin.**
    ///
    /// `IDropTarget` donne la position en pixels de l'écran entier ; `mouse_pos` et tout ce
    /// qui convertit vers le monde comptent depuis la fenêtre. Sans cette soustraction, une
    /// image déposée apparaîtrait d'autant plus loin que la fenêtre est basse et à droite —
    /// un décalage qui ne se voit pas sur une fenêtre maximisée à l'origine de l'écran, et qui
    /// est le genre de défaut qu'on ne trouve que chez quelqu'un d'autre.
    fn ecran_vers_client(&self, ecran: Option<(f64, f64)>) -> Option<(f64, f64)> {
        let (x, y) = ecran?;
        let coin = self.window.as_ref()?.inner_position().ok()?;
        Some((x - f64::from(coin.x), y - f64::from(coin.y)))
    }
}

/// Un document de Glucose — de Glucose Rust ou de Glucose Tauri, qui partagent l'extension.
fn est_un_document(chemin: &std::path::Path) -> bool {
    chemin
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(glucose_core::persist::FILE_EXTENSION))
}
