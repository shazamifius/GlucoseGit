//! **La liste des documents connus, telle qu'elle s'écrit** (fiche 88) : une ligne par document, du
//! plus récent au plus ancien — son chemin, puis, après une tabulation, le numéro que son système
//! lui donne ([`crate::plateforme::numero`]), par lequel la liste le retrouve s'il a bougé
//! (RECENTS-1, fiche 174).
//!
//! Une ligne d'avant, un chemin seul, se lit toujours ; son numéro s'ajoute la première fois que la
//! liste des documents se compose. Un nom peut contenir une tabulation sous Linux : seule une fin
//! `\tdisque-numéro`, en chiffres, est un numéro — tout le reste est le chemin.

use std::path::{Path, PathBuf};

/// **Un document connu** : son chemin, et son numéro s'il en a un.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Connu {
    pub chemin: PathBuf,
    pub numero: Option<String>,
}

impl Connu {
    /// Ce document, avec le numéro que son système lui donne maintenant.
    pub fn numerote(chemin: &Path) -> Self {
        Self {
            chemin: chemin.to_path_buf(),
            numero: crate::plateforme::numero::de(chemin),
        }
    }
}

/// **La liste écrite**, lue : une ligne par document, les lignes vides passées.
pub fn lire(texte: &str) -> Vec<Connu> {
    texte
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|ligne| match ligne.rsplit_once('\t') {
            Some((chemin, numero)) if est_un_numero(numero) => Connu {
                chemin: PathBuf::from(chemin),
                numero: Some(numero.to_string()),
            },
            _ => Connu {
                chemin: PathBuf::from(ligne),
                numero: None,
            },
        })
        .collect()
}

/// **La liste à écrire.**
pub fn ecrire(liste: &[Connu]) -> String {
    liste
        .iter()
        .map(|c| match &c.numero {
            Some(numero) => format!("{}\t{numero}\n", c.chemin.display()),
            None => format!("{}\n", c.chemin.display()),
        })
        .collect()
}

/// Ce document en tête, sans doublon.
pub fn en_tete(liste: Vec<Connu>, connu: Connu) -> Vec<Connu> {
    let chemin = connu.chemin.clone();
    std::iter::once(connu)
        .chain(liste.into_iter().filter(|c| c.chemin != chemin))
        .collect()
}

/// `disque-numéro`, ou `disque-numéro-naissance`, en chiffres : ce que
/// [`crate::plateforme::numero::de`] écrit.
fn est_un_numero(texte: &str) -> bool {
    let parts: Vec<&str> = texte.split('-').collect();
    (2..=3).contains(&parts.len())
        && parts
            .iter()
            .all(|t| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **La liste se relit telle qu'elle s'écrit** ; une ligne d'avant, sans numéro, se lit ; une
    /// tabulation dans un nom reste au nom.
    #[test]
    fn recents_1_la_liste_se_relit_numeros_compris() {
        let liste = vec![
            Connu {
                chemin: "/a/un.glucose".into(),
                numero: Some("2049-1234".into()),
            },
            Connu {
                chemin: "/b/deux\tnoms.glucose".into(),
                numero: None,
            },
            Connu {
                chemin: "/c/trois\tx.glucose".into(),
                numero: Some("7-8-1795657446127617834".into()),
            },
        ];
        assert_eq!(lire(&ecrire(&liste)), liste);
        let d_avant = lire("/a\n\n/b\n");
        assert_eq!(d_avant.len(), 2);
        assert!(d_avant.iter().all(|c| c.numero.is_none()));
    }

    #[test]
    fn la_liste_garde_le_plus_recent_en_tete_sans_doublon() {
        let liste = lire("/a\n\n/b\n");
        let liste = en_tete(liste, Connu::numerote(Path::new("/b")));
        let chemins: Vec<&Path> = liste.iter().map(|c| c.chemin.as_path()).collect();
        assert_eq!(chemins, [Path::new("/b"), Path::new("/a")]);
    }
}
