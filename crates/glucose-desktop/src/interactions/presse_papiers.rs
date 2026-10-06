//! **Quel presse-papiers** : celui du système pour l'application, un à soi pour chaque fil
//! d'épreuve.
//!
//! Les épreuves jouent `Ctrl+C`, `Ctrl+X` et `Ctrl+V` sur le vrai [`crate::app::GlucoseApp`].
//! Par le presse-papiers du système, chaque `cargo test` remplaçait ce que l'utilisateur venait
//! de copier — et un collage lisait ce qu'il y avait mis. Le presse-papiers du système se prend
//! donc au lancement de l'application, comme son dossier (`main.rs`) ; sans cela, chaque fil a
//! le sien, en mémoire, et deux épreuves qui tournent ensemble ne se lisent pas l'une l'autre.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

static DU_SYSTEME: AtomicBool = AtomicBool::new(false);

/// **L'application prend le presse-papiers du système**, pour tout le processus.
pub fn prendre_celui_du_systeme() {
    DU_SYSTEME.store(true, Ordering::Relaxed);
}

/// Le presse-papiers est-il celui du système ? Un fil de fond qui veut l'écrire lui-même doit
/// le savoir : celui d'une épreuve vit sur le fil qui l'a ouvert.
pub fn est_celui_du_systeme() -> bool {
    DU_SYSTEME.load(Ordering::Relaxed)
}

/// Ce que porte le presse-papiers à soi d'un fil. Écrire l'une de ses formes efface les
/// autres, comme dans celui du système.
#[derive(Default)]
struct Contenu {
    texte: Option<String>,
    /// Un lot de nœuds (fiche 51 § 2) : comme celui du système, écrire un texte seul l'efface.
    lot: Option<Vec<u8>>,
    /// Une image copiée par le menu (fiche 51 § 3), en PNG.
    image: Option<Vec<u8>>,
}

thread_local! {
    /// Le presse-papiers à soi d'un fil.
    static A_SOI: RefCell<Contenu> = RefCell::new(Contenu::default());
}

/// Un presse-papiers ouvert.
pub enum Acces {
    Systeme(arboard::Clipboard),
    ASoi,
}

/// **Ouvre le presse-papiers** — celui du système si l'application l'a pris.
pub fn ouvrir() -> Result<Acces, String> {
    if DU_SYSTEME.load(Ordering::Relaxed) {
        arboard::Clipboard::new()
            .map(Acces::Systeme)
            .map_err(|e| e.to_string())
    } else {
        Ok(Acces::ASoi)
    }
}

impl Acces {
    /// Le texte qu'il porte.
    pub fn texte(&mut self) -> Result<String, String> {
        match self {
            Acces::Systeme(c) => c.get_text().map_err(|e| e.to_string()),
            Acces::ASoi => A_SOI.with(|c| c.borrow().texte.clone().ok_or_else(|| RIEN.into())),
        }
    }

    /// L'image qu'il porte.
    pub fn image(&mut self) -> Result<arboard::ImageData<'static>, String> {
        match self {
            Acces::Systeme(c) => c.get_image().map_err(|e| e.to_string()),
            Acces::ASoi => {
                let png = A_SOI.with(|c| c.borrow().image.clone()).ok_or(RIEN)?;
                let pixels = image::load_from_memory(&png)
                    .map_err(|e| e.to_string())?
                    .to_rgba8();
                Ok(arboard::ImageData {
                    width: pixels.width() as usize,
                    height: pixels.height() as usize,
                    bytes: pixels.into_raw().into(),
                })
            }
        }
    }

    /// Y écrit ce texte.
    pub fn ecrire(&mut self, texte: String) -> Result<(), String> {
        match self {
            Acces::Systeme(c) => c.set_text(texte).map_err(|e| e.to_string()),
            Acces::ASoi => {
                A_SOI.with(|c| {
                    *c.borrow_mut() = Contenu {
                        texte: Some(texte),
                        ..Contenu::default()
                    }
                });
                Ok(())
            }
        }
    }

    /// Y écrit un lot de nœuds, et le texte que les autres logiciels colleront à sa place.
    pub fn ecrire_un_lot(&mut self, texte: Option<String>, lot: Vec<u8>) -> Result<(), String> {
        match self {
            Acces::Systeme(_) => {
                crate::plateforme::presse_papiers::ecrire_un_lot(texte.as_deref(), &lot)
            }
            Acces::ASoi => {
                A_SOI.with(|c| {
                    *c.borrow_mut() = Contenu {
                        texte,
                        lot: Some(lot),
                        image: None,
                    }
                });
                Ok(())
            }
        }
    }

    /// Y écrit une image (fiche 51 § 3).
    pub fn ecrire_une_image(
        &mut self,
        image: crate::interactions::clipboard::ImagePosee,
    ) -> Result<(), String> {
        match self {
            Acces::Systeme(_) => crate::plateforme::presse_papiers::ecrire_une_image(&image),
            Acces::ASoi => {
                A_SOI.with(|c| {
                    *c.borrow_mut() = Contenu {
                        image: Some(image.png),
                        ..Contenu::default()
                    }
                });
                Ok(())
            }
        }
    }

    /// Le lot de nœuds qu'il porte, s'il en porte un.
    pub fn lot(&mut self) -> Option<Vec<u8>> {
        match self {
            Acces::Systeme(_) => crate::plateforme::presse_papiers::lire_un_lot(),
            Acces::ASoi => A_SOI.with(|c| c.borrow().lot.clone()),
        }
    }
}

/// Ce qu'un presse-papiers vide répond.
const RIEN: &str = "le presse-papiers ne porte rien de tel";

#[cfg(test)]
mod tests {
    use super::*;

    /// **Une épreuve n'a jamais le presse-papiers du système** : ce qu'elle écrit reste à son
    /// fil, et un autre fil ne le lit pas.
    #[test]
    fn test_chaque_fil_d_epreuve_a_son_presse_papiers() {
        assert!(
            matches!(ouvrir(), Ok(Acces::ASoi)),
            "jamais celui du système"
        );
        ouvrir()
            .and_then(|mut a| a.ecrire("à moi".into()))
            .expect("écrit");
        assert_eq!(ouvrir().and_then(|mut a| a.texte()).as_deref(), Ok("à moi"));
        let ailleurs = std::thread::spawn(|| ouvrir().and_then(|mut a| a.texte()))
            .join()
            .expect("fil");
        assert!(ailleurs.is_err(), "un autre fil ne le lit pas");
    }
}
