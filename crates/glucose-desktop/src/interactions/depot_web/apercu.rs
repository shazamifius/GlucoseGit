//! **La copie d'abord, l'original ensuite** (fiche 53 § 9).
//!
//! # Ce qu'il a demandé
//!
//! *« Que ce soit moins long, et qu'on retrouve quasi instantanément l'image choisie. »* Sa
//! session du 07/10 au soir l'a mesuré : ses épingles portaient déjà l'adresse de leur image —
//! aucune page à lire —, et l'original a pourtant mis de 0,8 à **12,4 s**. Un original de
//! Pinterest peut être un PNG de plusieurs mégaoctets ; ses copies réduites (236, 474, 736
//! pixels) arrivent en une demi-seconde.
//!
//! # Ce qu'on fait
//!
//! Les variantes d'une image courent ensemble ([`crate::plateforme::rapatrier`]). La première
//! copie qui arrive pendant que l'original se fait attendre **se pose tout de suite**, à la
//! taille qu'aura l'original ([`crate::interactions::clipboard::taille_d_un_apercu`]) ; quand
//! l'original arrive, il **prend sa place** : même nœud, même endroit, même taille — sa
//! source seule change, et sa netteté. Rien ne saute à l'écran.
//!
//! Ce qui a pu changer entre-temps se respecte : un nœud effacé ne revient pas ; un nœud que
//! l'utilisateur a agrandi garde sa taille. Le recadrage, en proportions, reste juste.
//!
//! Le remplacement est un geste du journal, après celui de la pose : un `Ctrl+Z` juste après
//! rend la copie, un second retire l'image.

use crate::app::GlucoseApp;
use crate::interactions::clipboard::{taille_d_un_apercu, taille_posee};
use crate::plateforme::moisson::Recu;
use std::collections::HashMap;

/// Une copie posée : son tableau, son nœud, et la taille où elle s'est posée.
#[derive(Debug, Clone, PartialEq)]
pub struct Copie {
    board: String,
    id: String,
    taille: (f64, f64),
}

/// Les copies qui attendent leur original, par numéro de rapatriement.
pub type Copies = HashMap<u64, Copie>;

impl GlucoseApp {
    /// **Retient la copie qu'un rapatriement vient de poser**, pour l'original qui suivra.
    pub(super) fn retenir_la_copie(&mut self, numero: u64, id: Option<&String>) {
        let board = self.store.project.active_board_id.clone();
        let Some(image) = id.and_then(|id| self.store.image(&board, id)) else {
            return;
        };
        let copie = Copie {
            board,
            id: image.id.clone(),
            taille: (image.width, image.height),
        };
        self.depot.copies.insert(numero, copie);
    }

    /// **L'original prend la place de sa copie** — ou rien, quand la copie était déjà la
    /// meilleure. Dans tous les cas, l'attente de ce numéro est close.
    pub(super) fn remplacer_la_copie(&mut self, numero: u64, recu: Option<Recu>) {
        let Some(copie) = self.depot.copies.remove(&numero) else {
            return;
        };
        let Some(recu) = recu else {
            return;
        };
        // Effacée entre-temps : l'utilisateur n'en voulait plus.
        let Some(image) = self.store.image(&copie.board, &copie.id) else {
            return;
        };
        let intacte = (image.width, image.height) == copie.taille;
        let Ok((cle, (w, h))) = self.sceller_une_image(recu, 0) else {
            return;
        };
        // Celle qu'aurait eue l'original posé seul — la même que la copie, sauf s'il est plus
        // petit que ce qu'elle présumait.
        let taille = taille_posee((w, h));
        debug_assert!(taille_d_un_apercu((w, h)).0 >= taille.0);
        self.store.update_image(&copie.board, &copie.id, |image| {
            image.src = Some(cle);
            image.original_width = w;
            image.original_height = h;
            if intacte {
                (image.width, image.height) = taille;
            }
        });
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests;
