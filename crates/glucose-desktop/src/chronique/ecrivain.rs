//! **La chronique s'écrit hors du fil qui dessine** (JETER-1, fiche 87).
//!
//! Elle se sauve quand une image entre parmi les plus lentes de la session : c'est le moment où elle
//! a du neuf à dire, et le pire moment pour attendre le disque. Sur `strace`, un seul `close` y a
//! duré 166 ms, sur le fil qui dessine : la mesure aggravait ce qu'elle mesurait.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;

/// Ce qui attend le disque : où, et quoi.
type AEcrire = (PathBuf, String);

/// Le fil qui écrit, lancé à la première écriture. Il écrit dans l'ordre : la dernière gagne.
#[derive(Default)]
pub struct Ecrivain {
    fil: Option<(Sender<AEcrire>, JoinHandle<()>)>,
}

fn ecrire(chemin: &Path, texte: &str) -> std::io::Result<()> {
    if let Some(dossier) = chemin.parent() {
        std::fs::create_dir_all(dossier)?;
    }
    std::fs::write(chemin, texte)
}

impl Ecrivain {
    /// **Confie ce texte au fil**, sans rien attendre. Une chronique qui ne s'écrit pas n'empêche
    /// rien : elle se réessaiera à la prochaine image lente.
    pub fn confier(&mut self, chemin: PathBuf, texte: String) {
        let (envoi, _) = self.fil.get_or_insert_with(|| {
            let (envoi, recu) = channel::<AEcrire>();
            let fil = std::thread::spawn(move || {
                while let Ok((chemin, texte)) = recu.recv() {
                    let _ = ecrire(&chemin, &texte);
                }
            });
            (envoi, fil)
        });
        let _ = envoi.send((chemin, texte));
    }

    /// **Attend que tout soit écrit** : à la fermeture, plus rien ne tient la cadence, et ce qui
    /// s'écrit ensuite ne doit pas être recouvert par une écriture en retard.
    pub fn finir(&mut self) {
        if let Some((envoi, fil)) = self.fil.take() {
            drop(envoi);
            let _ = fil.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Tout s'écrit, dans l'ordre, et `finir` l'attend** : la dernière version est celle qui reste,
    /// sans quoi une chronique en retard recouvrirait celle de la fermeture.
    #[test]
    fn test_jeter_1_la_chronique_s_ecrit_dans_l_ordre_et_finir_l_attend() {
        let dossier = crate::epreuve::Dossier::nouveau("ecrivain");
        let chemin = dossier.join("sous/dossier/chronique.txt");
        let mut ecrivain = Ecrivain::default();
        for i in 0..50 {
            ecrivain.confier(chemin.clone(), format!("version {i}"));
        }
        ecrivain.finir();
        assert_eq!(
            std::fs::read_to_string(&chemin).expect("écrite"),
            "version 49"
        );
        // Après `finir`, l'écrivain repart : la suivante arrive aussi.
        ecrivain.confier(chemin.clone(), "encore".to_string());
        ecrivain.finir();
        assert_eq!(std::fs::read_to_string(&chemin).expect("écrite"), "encore");
    }
}
