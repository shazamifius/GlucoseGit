//! **« Toujours, dans ce document »** (AGENTS-TOUJOURS, fiche 131) : les documents où tout
//! programme de cette machine travaille sans qu'on le lui redemande, même après un redémarrage.
//!
//! Le 23/11/2026 au matin, après le redémarrage du PC, il a dû cliquer « Autoriser » trois fois à
//! la main dans son Glucose, pour que le relais reprenne ; la veille, cinq fois. L'accord ne vivait
//! que jusqu'à la fermeture de Glucose, et par nom d'agent.
//!
//! **L'accord durable porte sur le document, pas sur un nom.** Le nom qu'un agent se donne n'est
//! pas une identité : n'importe quel programme peut se dire « claude-code » (et toute commande d'un
//! terminal se dit « le terminal »). Ce qui distingue un agent de la machine, c'est d'atteindre la
//! porte locale, que le système n'ouvre qu'à l'utilisateur qui a lancé Glucose (AGENTS-2). Un accord
//! « pour ce nom-là, toujours » ne protégerait de rien et le ferait cliquer pour chaque nom neuf.
//!
//! **Retenu par Glucose, dans son dossier — jamais dans le `.glucose`** : un fichier envoyé à
//! quelqu'un n'emporte pas l'accord. Une identité de document par ligne ([`crate::app`] la tient
//! dans `store.identite` : le chemin du fichier, fiche 80). Les IA du nuage n'en relèvent jamais :
//! elles ont leur propre accord, la clé que Glucose leur donne après son clic (PUBLIQUE-1).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Le fichier, dans le dossier de Glucose sur cette machine.
pub fn chemin() -> PathBuf {
    crate::present::souvenir::dossier().join("agents-toujours.txt")
}

/// Les documents retenus. Un fichier absent ou illisible : aucun — la question revient, rien
/// n'est accordé par défaut.
pub fn lire(chemin: &Path) -> BTreeSet<String> {
    std::fs::read_to_string(chemin)
        .map(|texte| {
            texte
                .lines()
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/// Écrit la liste d'un coup, par la loi de Glucose (`persist::atomic`) : jamais une liste à
/// moitié, jamais l'ancienne perdue.
pub fn ecrire(chemin: &Path, documents: &BTreeSet<String>) -> std::io::Result<()> {
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier)?;
    }
    let texte: String = documents.iter().map(|d| format!("{d}\n")).collect();
    crate::persist::atomic::ecrire_d_un_bloc(chemin, texte.as_bytes())
}

/// **Les documents où les agents de la machine travaillent toujours**, et le fichier qui les
/// garde. Sans fichier (une épreuve, le téléphone), rien ne se retient, et le choix ne s'offre pas.
#[derive(Default)]
pub struct Toujours {
    fichier: Option<PathBuf>,
    documents: BTreeSet<String>,
    /// La dernière écriture ratée : l'accord vaut alors pour cette session seulement, et la
    /// question « Retirer l'accord des agents… » le dit.
    pub(crate) rate: Option<String>,
}

impl Toujours {
    /// Lu au lancement, dans ce fichier.
    pub fn depuis(fichier: PathBuf) -> Self {
        Self {
            documents: lire(&fichier),
            fichier: Some(fichier),
            rate: None,
        }
    }

    /// Ce document a-t-il l'accord durable ?
    pub fn vaut(&self, identite: &str) -> bool {
        self.documents.contains(identite)
    }

    /// **Le choix peut-il s'offrir ?** Seulement pour un document qui vit dans un fichier : celui
    /// d'avant tout enregistrement n'a pas d'identité qui survive à la fermeture — l'offrir serait
    /// un bouton qui ment.
    pub fn peut_retenir(&self, identite: &str) -> bool {
        self.fichier.is_some() && !identite.contains('\n') && Path::new(identite).is_file()
    }

    /// Retient ce document — en mémoire d'abord, puis sur le disque.
    pub fn retenir(&mut self, identite: &str) {
        self.documents.insert(identite.to_string());
        self.garder();
    }

    /// Le retire.
    pub fn oublier(&mut self, identite: &str) {
        if self.documents.remove(identite) {
            self.garder();
        }
    }

    fn garder(&mut self) {
        let Some(fichier) = &self.fichier else {
            return;
        };
        self.rate = ecrire(fichier, &self.documents)
            .err()
            .map(|e| e.to_string());
        if let Some(e) = &self.rate {
            crate::dire!("[Glucose] accord des agents : pas retenu sur le disque ({e})");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dossier(nom: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("glucose-toujours-{nom}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("dossier");
        d
    }

    #[test]
    fn agents_toujours_retenu_survit_a_la_relecture_et_s_oublie() {
        let d = dossier("relu");
        let fichier = d.join("agents-toujours.txt");
        let document = d.join("GLUCOSE PROJET.glucose");
        std::fs::write(&document, "{}").expect("document");
        let identite = document.display().to_string();
        let mut t = Toujours::depuis(fichier.clone());
        assert!(t.peut_retenir(&identite));
        t.retenir(&identite);
        assert!(Toujours::depuis(fichier.clone()).vaut(&identite), "relu");
        assert!(!Toujours::depuis(fichier.clone()).vaut("/un/autre.glucose"));
        t.oublier(&identite);
        assert!(!Toujours::depuis(fichier).vaut(&identite), "oublié");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn agents_toujours_ne_s_offre_pas_sans_fichier_ni_document_sur_le_disque() {
        let d = dossier("offre");
        let t = Toujours::depuis(d.join("agents-toujours.txt"));
        assert!(
            !t.peut_retenir("charge-123-0"),
            "un document jamais enregistré"
        );
        assert!(!t.peut_retenir(&d.join("absent.glucose").display().to_string()));
        let document = d.join("doc.glucose");
        std::fs::write(&document, "{}").expect("document");
        assert!(
            !Toujours::default().peut_retenir(&document.display().to_string()),
            "sans fichier"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn agents_toujours_une_ecriture_ratee_vaut_pour_la_session_et_se_dit() {
        let d = dossier("rate");
        std::fs::write(d.join("pas-un-dossier"), "").expect("fichier");
        let mut t = Toujours::depuis(d.join("pas-un-dossier").join("agents-toujours.txt"));
        t.retenir("/doc.glucose");
        assert!(t.vaut("/doc.glucose"), "pour cette session");
        assert!(t.rate.is_some(), "et elle le dit");
        let _ = std::fs::remove_dir_all(&d);
    }
}
