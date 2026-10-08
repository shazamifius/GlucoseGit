//! **MAJ-ANDROID-1 — la mise à jour confiée au système** (fiche 58).
//!
//! Sous Windows et Linux, Glucose pose lui-même la version suivante. Sous Android, une
//! application ne se remplace pas : elle confie l'APK à l'installeur du système
//! (`PackageInstaller`), qui vérifie la clé de Glucose pour Android, demande à l'utilisateur —
//! sauf, à partir d'Android 12, quand Glucose se met à jour lui-même —, puis remplace Glucose.
//! L'APK est déjà vérifié par la clé de Glucose avant d'arriver ici, comme partout.
//!
//! Deux sens : Glucose **confie** l'APK (la porte, branchée par `glucose-android`) ; Android
//! **dit** où en est l'installation (la boîte aux lettres, relevée à chaque tour de boucle).
//! Ailleurs qu'au téléphone, la porte n'est jamais branchée.

use std::path::Path;
use std::sync::{Mutex, PoisonError};

/// Ce qu'Android dit de l'installation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Etat {
    /// Il faut autoriser Glucose à installer des applications : la page s'est ouverte, et
    /// l'installation repart au retour.
    Autoriser,
    /// Android demande la confirmation.
    Confirmer,
    /// L'installation a échoué, et pourquoi.
    Echec(String),
}

impl Etat {
    /// **Ce que Glucose en dit**, dans ses mots — Java ne choisit aucun mot.
    pub fn dire(&self) -> String {
        match self {
            Self::Autoriser => "autorise Glucose à installer des applications, puis reviens : \
                                elle reprendra"
                .to_string(),
            Self::Confirmer => "Android demande ta confirmation".to_string(),
            Self::Echec(pourquoi) => format!("Android ne l'a pas installée ({pourquoi})"),
        }
    }
}

/// Ce qui confie un APK au système.
pub type Confieur = std::sync::Arc<dyn Fn(&Path) + Send + Sync>;

/// La porte et la boîte aux lettres.
struct Boite {
    confieur: Option<Confieur>,
    etats: Vec<Etat>,
    reveil: Option<super::Reveil>,
}

static BOITE: Mutex<Boite> = Mutex::new(Boite {
    confieur: None,
    etats: Vec::new(),
    reveil: None,
});

/// **Branche ce qui confie un APK au système** — sous Android, au lancement.
pub fn installer_le_confieur(confieur: Confieur) {
    BOITE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .confieur = Some(confieur);
}

/// De quoi réveiller la boucle quand Android dit quelque chose, donné au lancement.
pub fn brancher(reveil: super::Reveil) {
    BOITE.lock().unwrap_or_else(PoisonError::into_inner).reveil = Some(reveil);
}

/// **Confie cet APK au système**, s'il y a un système à qui le confier. Le verrou se relâche
/// avant l'appel : Android peut répondre pendant qu'on l'appelle, sur le même fil
/// ([`recevoir`]), et un verrou tenu le bloquerait pour toujours.
pub fn confier(apk: &Path) -> Result<(), String> {
    let confieur = BOITE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .confieur
        .clone()
        .ok_or("aucun installeur du système à qui confier l'APK")?;
    confieur(apk);
    Ok(())
}

/// **Android dit où en est l'installation** — depuis un fil de Java. La boucle dort peut-être :
/// on la réveille.
pub fn recevoir(etat: Etat) {
    let mut boite = BOITE.lock().unwrap_or_else(PoisonError::into_inner);
    boite.etats.push(etat);
    if let Some(reveil) = &boite.reveil {
        reveil();
    }
}

/// Ce qu'Android a dit depuis la dernière fois.
pub fn relever() -> Vec<Etat> {
    std::mem::take(&mut BOITE.lock().unwrap_or_else(PoisonError::into_inner).etats)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// La boîte aux lettres est une seule pour tout le processus : les épreuves qui y passent
    /// se suivent, sinon l'une relèverait ce que l'autre attend.
    pub(crate) fn a_moi() -> std::sync::MutexGuard<'static, ()> {
        static TOUR: Mutex<()> = Mutex::new(());
        TOUR.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// **Android répond pendant qu'on l'appelle**, sur le même fil — « il faut autoriser » :
    /// rien ne se bloque, et la réponse attend la boucle. (Seule épreuve à brancher la porte :
    /// les autres n'y touchent pas.)
    #[test]
    fn test_maj_android_1_android_repond_pendant_qu_on_l_appelle() {
        let _tour = a_moi();
        let _ = relever();
        installer_le_confieur(std::sync::Arc::new(|_| recevoir(Etat::Autoriser)));
        assert_eq!(confier(Path::new("glucose.apk")), Ok(()));
        assert!(relever().contains(&Etat::Autoriser));
    }

    /// **Chaque état se dit dans les mots de Glucose**, et l'échec garde ce qu'Android a dit.
    #[test]
    fn test_maj_android_1_chaque_etat_se_dit() {
        assert!(Etat::Autoriser.dire().contains("autorise Glucose"));
        assert!(Etat::Confirmer.dire().contains("confirmation"));
        assert!(Etat::Echec("INSTALL_FAILED".into())
            .dire()
            .contains("INSTALL_FAILED"));
    }
}
