//! Ce qu'on fait d'un dépôt venu d'un navigateur (DEPOT-WEB-1).
//!
//! # Il n'y a presque rien ici, et c'est le but
//!
//! [`crate::plateforme::depot_windows`] rend une moisson faite de **fichiers** : les octets
//! qu'une page promettait sont déjà écrits sur le disque quand ce module les voit. Poser une
//! image glissée depuis un site ne demande donc aucun chemin de code nouveau — c'est
//! [`super::drop`] qui route, exactement comme pour un fichier venu de l'explorateur, et lui
//! seul décide qu'un lot tient dans une entrée d'annulation.
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
}

impl GlucoseApp {
    /// **Ce que le pont de dépôt fait parvenir** : une annonce, ou une livraison.
    pub fn recevoir_le_depot(&mut self, depot: Depot) {
        match depot {
            Depot::EnChemin { numero, ou, hote } => self.annoncer_l_arrivage(numero, ou, hote),
            Depot::Pose { numero, moisson } => self.poser_le_depot(numero, &moisson),
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
        let origine = match annonce {
            Some(a) => a.monde,
            None => self.drop_origin(self.ecran_vers_client(recolte.ou)),
        };
        self.deposer(&recolte.chemins, &recolte.liens, origine);
        self.mark_dirty();
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
