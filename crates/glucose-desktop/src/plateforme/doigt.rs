//! **Ce que le système dit du doigt** (fiche 57) : combien de temps un doigt posé et immobile
//! met à devenir un appui long, et la vibration qui dit qu'il a pris.
//!
//! # Le délai n'est pas à Glucose
//!
//! Android le règle pour chacun : « Délai de pression prolongée » (court, moyen, long), dans
//! l'accessibilité — une main qui tremble le veut long. `ViewConfiguration.getLongPressTimeout`
//! le lit (dans AOSP : le réglage `LONG_PRESS_TIMEOUT`, 400 ms à défaut). Le prendre chez le
//! système, à chaque doigt posé, fait **disparaître** la constante au lieu de la choisir : chaque
//! téléphone garde la main de son propriétaire.

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

/// **Le délai d'Android lui-même** (`ViewConfiguration.DEFAULT_LONG_PRESS_TIMEOUT`), là où le
/// système n'en dit aucun — un écran tactile sous Linux. C'est le seul système qui le fixe pour un
/// doigt sur un canevas, et il est mesuré pour cela.
const APPUI_LONG_PAR_DEFAUT: Duration = Duration::from_millis(400);

/// Ce que la plateforme sait du doigt.
pub trait Doigt: Send {
    /// Le délai de l'appui long, s'il se lit.
    fn appui_long(&self) -> Option<Duration>;
    /// Faire sentir que l'appui long a pris.
    fn sentir_l_appui(&self);
}

static DOIGT: Mutex<Option<Box<dyn Doigt>>> = Mutex::new(None);

/// **Branche ce que le système dit du doigt**, avant le lancement.
pub fn installer(doigt: Box<dyn Doigt>) {
    *DOIGT.lock().unwrap_or_else(PoisonError::into_inner) = Some(doigt);
}

/// Le délai de l'appui long, lu à l'instant : l'utilisateur a pu le changer entre deux doigts.
pub fn appui_long() -> Duration {
    DOIGT
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .and_then(|d| d.appui_long())
        .unwrap_or(APPUI_LONG_PAR_DEFAUT)
}

/// La vibration de l'appui long, là où le système en donne une.
pub fn sentir_l_appui() {
    if let Some(d) = DOIGT
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
    {
        d.sentir_l_appui();
    }
}
