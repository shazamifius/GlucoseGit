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
use crate::plateforme::moisson::Moisson;

impl GlucoseApp {
    /// **Pose ce qu'un dépôt du système vient d'apporter.**
    ///
    /// Un appel par moisson : glisser huit images d'une page est **un** geste, et deux dépôts
    /// successifs en sont deux. C'est `deposer` qui en fait une entrée d'annulation, et qui
    /// rend l'unique compte-rendu.
    pub fn poser_le_depot(&mut self, recolte: &Moisson) {
        let ou = self.ecran_vers_client(recolte.ou);
        self.deposer(&recolte.chemins, &recolte.liens, ou);
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
