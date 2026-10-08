//! **La batterie, là où le système ne la laisse lire que par lui** (fiche 58).
//!
//! Sous Linux, Glucose la lit dans `/sys/class/power_supply` (`boite_noire::sondes`). Android
//! ferme ce dossier aux applications : le journal technique de son Redmi 9 l'envoyait toujours
//! vide (`batterie_pct` nul, fiche 57 § 5). Android la dit à tous par une intention qu'il garde
//! (`ACTION_BATTERY_CHANGED`) : le niveau, l'échelle, et ce qui la charge. `MainActivity` la lit
//! (`lireLaBatterie`), et confie les trois nombres ; Glucose en fait un pourcentage ici.

use std::sync::{Mutex, PoisonError};

/// Ce que le système dit de la batterie : son niveau, l'échelle de ce niveau, et la prise qui
/// la charge (0 : aucune).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Brut {
    pub niveau: i32,
    pub echelle: i32,
    pub prise: i32,
}

impl Brut {
    /// Le pourcentage, s'il se lit : un niveau dans son échelle.
    pub fn pourcentage(self) -> Option<u8> {
        if self.echelle <= 0 || self.niveau < 0 || self.niveau > self.echelle {
            return None;
        }
        u8::try_from(i64::from(self.niveau) * 100 / i64::from(self.echelle)).ok()
    }

    /// Branchée ? Une prise négative est une prise que le système ne dit pas.
    pub fn en_charge(self) -> Option<bool> {
        (self.prise >= 0).then_some(self.prise > 0)
    }
}

/// Ce qui lit la batterie chez le système.
pub type Lecteur = Box<dyn Fn() -> Option<Brut> + Send>;

static LECTEUR: Mutex<Option<Lecteur>> = Mutex::new(None);

/// **Branche ce qui lit la batterie** — sous Android, au lancement.
pub fn installer(lecteur: Lecteur) {
    *LECTEUR.lock().unwrap_or_else(PoisonError::into_inner) = Some(lecteur);
}

/// Ce que le système dit de la batterie maintenant, s'il y a de quoi le lui demander.
pub fn lire() -> Option<Brut> {
    LECTEUR
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .and_then(|l| l())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Un niveau dans son échelle fait un pourcentage** ; une échelle nulle, un niveau
    /// négatif ou hors de l'échelle n'en font aucun ; une prise négative ne se sait pas.
    #[test]
    fn test_la_batterie_d_android_se_lit_en_pourcentage() {
        let brut = |niveau, echelle, prise| Brut {
            niveau,
            echelle,
            prise,
        };
        assert_eq!(brut(87, 100, 0).pourcentage(), Some(87));
        assert_eq!(brut(3, 4, 0).pourcentage(), Some(75));
        assert_eq!(brut(5, 0, 0).pourcentage(), None);
        assert_eq!(brut(-1, 100, 0).pourcentage(), None);
        assert_eq!(brut(101, 100, 0).pourcentage(), None);
        assert_eq!(brut(50, 100, 0).en_charge(), Some(false));
        assert_eq!(brut(50, 100, 2).en_charge(), Some(true));
        assert_eq!(brut(50, 100, -1).en_charge(), None);
    }
}
