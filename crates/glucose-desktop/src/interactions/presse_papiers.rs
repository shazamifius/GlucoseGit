//! **Quel presse-papiers** : celui du système pour l'application, un à soi pour chaque fil
//! d'épreuve.
//!
//! Les épreuves jouent `Ctrl+C`, `Ctrl+X` et `Ctrl+V` sur le vrai [`crate::app::GlucoseApp`].
//! Par le presse-papiers du système, chaque `cargo test` remplaçait ce que l'utilisateur venait
//! de copier — et un collage lisait ce qu'il y avait mis. Le presse-papiers du système se prend
//! donc au lancement de l'application, comme son dossier (`main.rs`) ; sans cela, chaque fil a
//! le sien, en mémoire, et deux épreuves qui tournent ensemble ne se lisent pas l'une l'autre.
//!
//! **Sous Android**, `arboard` ne sait rien faire : le presse-papiers est celui de Glucose seul
//! (fiche 54) — copier et coller entre ses nœuds marche, pas encore avec les autres
//! applications, qui demanderont celui du système par JNI.

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
    /// L'image seule d'une sélection copiée (COPIER-1), en PNG.
    image: Option<Vec<u8>>,
}

thread_local! {
    /// Le presse-papiers à soi d'un fil.
    static A_SOI: RefCell<Contenu> = RefCell::new(Contenu::default());
}

/// **Des pixels RGBA**, rangée après rangée — ce que le presse-papiers donne d'une image. Un
/// type à Glucose : celui d'`arboard` n'existe pas là où `arboard` ne compile pas (Android).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixels {
    pub width: usize,
    pub height: usize,
    pub bytes: Vec<u8>,
}

/// Un presse-papiers ouvert.
pub enum Acces {
    #[cfg(not(target_os = "android"))]
    Systeme(arboard::Clipboard),
    ASoi,
}

/// **Ouvre le presse-papiers** — celui du système si l'application l'a pris, et si le système
/// en a un que Glucose sait lire.
pub fn ouvrir() -> Result<Acces, String> {
    #[cfg(not(target_os = "android"))]
    if DU_SYSTEME.load(Ordering::Relaxed) {
        return arboard::Clipboard::new()
            .map(Acces::Systeme)
            .map_err(|e| e.to_string());
    }
    Ok(Acces::ASoi)
}

impl Acces {
    /// Le texte qu'il porte.
    pub fn texte(&mut self) -> Result<String, String> {
        match self {
            #[cfg(not(target_os = "android"))]
            Acces::Systeme(c) => c.get_text().map_err(|e| e.to_string()),
            Acces::ASoi => A_SOI.with(|c| c.borrow().texte.clone().ok_or_else(|| RIEN.into())),
        }
    }

    /// L'image qu'il porte.
    pub fn image(&mut self) -> Result<Pixels, String> {
        match self {
            #[cfg(not(target_os = "android"))]
            Acces::Systeme(c) => c
                .get_image()
                .map(|i| Pixels {
                    width: i.width,
                    height: i.height,
                    bytes: i.bytes.into_owned(),
                })
                .map_err(|e| e.to_string()),
            Acces::ASoi => {
                let png = A_SOI.with(|c| c.borrow().image.clone()).ok_or(RIEN)?;
                let pixels = image::load_from_memory(&png)
                    .map_err(|e| e.to_string())?
                    .to_rgba8();
                Ok(Pixels {
                    width: pixels.width() as usize,
                    height: pixels.height() as usize,
                    bytes: pixels.into_raw(),
                })
            }
        }
    }

    /// Y écrit ce texte.
    pub fn ecrire(&mut self, texte: String) -> Result<(), String> {
        match self {
            #[cfg(not(target_os = "android"))]
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

    /// **Y confie la sélection**, sous toutes ses formes (COPIER-1, fiche 59) : celui du système
    /// les porte toutes ; celui d'une épreuve garde le lot, le texte et l'image seule — ce qu'un
    /// collage relit.
    pub fn ecrire_la_selection(
        &mut self,
        formes: crate::interactions::clipboard::Formes,
    ) -> Result<(), String> {
        match self {
            #[cfg(not(target_os = "android"))]
            Acces::Systeme(_) => crate::plateforme::presse_papiers::ecrire_la_selection(formes),
            Acces::ASoi => {
                A_SOI.with(|c| {
                    *c.borrow_mut() = Contenu {
                        texte: formes.texte,
                        lot: Some(formes.lot),
                        image: formes.image.map(|i| i.png),
                    }
                });
                Ok(())
            }
        }
    }

    /// Le lot de nœuds qu'il porte, s'il en porte un.
    pub fn lot(&mut self) -> Option<Vec<u8>> {
        match self {
            #[cfg(not(target_os = "android"))]
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
