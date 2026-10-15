//! **La place du disque où Glucose écrit** (REPRISE-1, fiche 72).
//!
//! Le 15/10, un tableau de 6 000 photos s'est fermé seul au téléphone et n'est pas revenu. Une
//! des deux causes possibles est le disque plein : les photos sont copiées dans le document, et
//! 6 000 photos de téléphone pèsent des gigaoctets. Sous Android, `MainActivity` lit la place du
//! dossier privé (`lireLaPlace`, `StatFs`) ; ailleurs, rien ne la lit encore — absente, jamais
//! zéro (cliquet 9).

use std::sync::{Mutex, PoisonError};

/// La place d'un disque, en octets : ce qui reste libre, et le total.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    pub libre: u64,
    pub total: u64,
}

/// Ce qui lit la place chez le système.
pub type Lecteur = Box<dyn Fn() -> Option<Place> + Send>;

static LECTEUR: Mutex<Option<Lecteur>> = Mutex::new(None);

/// **Branche ce qui lit la place** — sous Android, au lancement.
pub fn installer(lecteur: Lecteur) {
    *LECTEUR.lock().unwrap_or_else(PoisonError::into_inner) = Some(lecteur);
}

/// La place du disque de Glucose maintenant, s'il y a de quoi la demander.
pub fn lire() -> Option<Place> {
    LECTEUR
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .and_then(|l| l())
}

/// **La place, depuis les deux nombres que Java rend** : un nombre négatif n'est pas une place.
pub fn depuis_le_tableau(v: &[i64]) -> Option<Place> {
    match v {
        [libre, total] => Some(Place {
            libre: u64::try_from(*libre).ok()?,
            total: u64::try_from(*total).ok()?,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Deux nombres font une place** ; un nombre négatif, ou un autre compte, n'en font aucune.
    #[test]
    fn test_reprise_1_la_place_de_java_se_lit() {
        assert_eq!(
            depuis_le_tableau(&[5_000, 64_000]),
            Some(Place {
                libre: 5_000,
                total: 64_000
            })
        );
        assert_eq!(depuis_le_tableau(&[-1, 64_000]), None);
        assert_eq!(depuis_le_tableau(&[5_000]), None);
    }
}
