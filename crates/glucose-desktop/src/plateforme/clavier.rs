//! **Le clavier du système** (CLAVIER-1, fiche 57) : celui qui sort de l'écran d'un téléphone.
//!
//! Sur un bureau, le texte arrive par les touches (`KeyEvent::text`) et rien n'est à demander.
//! Sous Android, le clavier virtuel ne vient que si on l'appelle, et il **tient lui-même le
//! texte** qu'il modifie : `GameTextInput` garde un `Editable` Java — le texte entier, la
//! sélection, le mot en cours de composition — que les suggestions, la correction et la dictée
//! réécrivent. `winit` 0.30 (et la 0.31 en bêta) jette l'évènement qui le dit ; l'état, lui,
//! reste lisible. Glucose ne reçoit donc pas des frappes, mais des **états** à refléter
//! (`interactions::text_edit::miroir`).
//!
//! Cette porte ne dit que ce qu'un clavier du système sait faire ; `glucose-android` la branche
//! au lancement, et le bureau n'en a pas.
//!
//! # Les positions sont en unités UTF-16
//!
//! Lu dans le code livré (`games-activity` 4.4.0, `InputConnection`) : la sélection vient de
//! `Selection.getSelectionStart` d'un `Editable` — des `char` de Java —, et le C++ la recopie
//! telle quelle. Glucose compte en octets UTF-8 : la conversion se fait d'un seul côté, celui
//! du miroir, où elle s'éprouve sans téléphone.

use std::sync::{Mutex, PoisonError};

/// **Ce que tient le clavier** : le texte, la sélection, la composition — en unités UTF-16.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EtatDuClavier {
    pub texte: String,
    /// L'ancre puis la tête, comme `Selection.getSelectionStart` / `End` les rendent : le début
    /// peut dépasser la fin quand on a étendu vers l'arrière.
    pub selection: (usize, usize),
    /// Le mot que le clavier compose encore (souligné par Gboard), s'il y en a un.
    pub composition: Option<(usize, usize)>,
}

impl EtatDuClavier {
    /// Le même texte, et la même sélection. La composition ne compte pas : elle appartient au
    /// clavier, et Glucose n'a rien à en refaire.
    pub fn meme_saisie(&self, autre: &Self) -> bool {
        self.texte == autre.texte && self.selection == autre.selection
    }
}

/// **Un clavier du système.** Chaque demande part dans la file du fil de l'interface d'Android
/// (`write_work`, lu dans `GameActivity.cpp`) : elle n'est pas faite quand l'appel rend la main.
pub trait Clavier: Send {
    /// Sortir le clavier, sur cette saisie.
    fn montrer(&self, etat: &EtatDuClavier);
    /// Remplacer ce qu'il tient, sans le sortir : Glucose a changé la saisie lui-même.
    fn ecrire(&self, etat: &EtatDuClavier);
    /// Le rentrer : la saisie est finie.
    fn cacher(&self);
    /// Ce qu'il tient maintenant — peut-être pas encore ce qu'on vient de lui écrire.
    fn lire(&self) -> EtatDuClavier;
}

static CLAVIER: Mutex<Option<Box<dyn Clavier>>> = Mutex::new(None);

/// **Branche le clavier de cette plateforme**, avant le lancement.
pub fn installer(clavier: Box<dyn Clavier>) {
    *CLAVIER.lock().unwrap_or_else(PoisonError::into_inner) = Some(clavier);
}

/// Le clavier branché, que l'application prend pour elle : les épreuves, qui créent des
/// centaines d'applications, n'en reçoivent jamais.
pub fn prendre() -> Option<Box<dyn Clavier>> {
    CLAVIER
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .take()
}
