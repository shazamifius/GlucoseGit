//! **Ce que la machine annonce d'elle-même** : la période de l'écran, la façon dont les images
//! se succèdent, et la carte graphique qui les dessine.
//!
//! Trois **faits de la machine**, pas des choix : les écrire dans le rapport est la seule façon
//! de ne pas relire une chronique en supposant l'un ou l'autre. Le mode de présentation a été
//! pris pour `Fifo` pendant toute l'histoire de ce dépôt alors qu'il valait `Immediate` ; et la
//! session du 07/10 a gelé une heure sur une carte qui n'affichait rien, sans qu'aucune ligne
//! ne le dise (ECRAN-1).

use std::time::Duration;

#[derive(Debug)]
pub struct Machine {
    /// La période de l'écran, telle qu'il l'annonce. Zéro tant qu'on ne l'a pas lue.
    pub(super) periode: Duration,
    /// Comment les images se succèdent devant la carte graphique, tel qu'elle l'a accepté.
    pub(super) presentation: &'static str,
    /// La carte qui dessine, et si c'est elle qui tient l'écran.
    pub(super) carte: Option<String>,
}

impl Default for Machine {
    fn default() -> Self {
        Self {
            periode: Duration::ZERO,
            presentation: "inconnue",
            carte: None,
        }
    }
}
