//! **Q18 — ce que dit le cadre gris d'une photo partagée** (sa réponse à Q18 : *« une
//! fonctionnalité super intéressante et visible »* ; fiche 150 § 3).
//!
//! La salle ne garde aucune photo : ses octets viennent de quelqu'un qui l'a. Tant qu'il n'est pas
//! là, son cadre reste gris — et dit de qui elle est, et s'il est là. Qui l'a posée, l'application
//! le retient à l'arrivée de son geste ; qui est là, à chaque tour du partage. Hors partage, le
//! cadre garde son libellé ; une photo qu'on décode encore aussi : elle n'attend personne.

use super::Magasin;
use std::collections::{BTreeSet, HashMap};

/// Ce que le partage dit des photos qui manquent.
#[derive(Debug, Default)]
pub struct Attentes {
    /// Par clé de photo, qui l'a posée : sa marque et son nom.
    posees_par: HashMap<String, (String, String)>,
    /// Les marques de ceux qui sont là.
    presents: BTreeSet<String>,
}

impl Magasin {
    /// **Cette photo vient de quelqu'un** : la marque et le nom de qui l'a posée, d'un canevas
    /// partagé.
    pub fn posee_par(&mut self, cle: &str, marque: &str, nom: &str) {
        let attentes = self.attentes.get_or_insert_with(Attentes::default);
        let qui = (marque.to_string(), nom.to_string());
        if attentes.posees_par.get(cle) != Some(&qui) {
            attentes.posees_par.insert(cle.to_string(), qui);
            self.changements += 1;
        }
    }

    /// **Qui est là**, dans le canevas partagé — rien ne change s'il n'a pas changé.
    pub fn presents(&mut self, marques: BTreeSet<String>) {
        let attentes = self.attentes.get_or_insert_with(Attentes::default);
        if attentes.presents != marques {
            attentes.presents = marques;
            self.changements += 1;
        }
    }

    /// **Hors partage** : les cadres reprennent leur libellé.
    pub fn hors_partage(&mut self) {
        if self.attentes.take().is_some() {
            self.changements += 1;
        }
    }

    /// **Ce que dit le cadre de cette photo**, qui n'est pas là (hors introuvable, `cadre.rs`).
    pub(super) fn libelle(&self, src: Option<&str>, id: &str) -> String {
        let attend = |cle: &&str| self.echecs.contains(*cle);
        let (Some(a), Some(cle)) = (&self.attentes, src.filter(attend)) else {
            return format!("Image [{id}]");
        };
        match a.posees_par.get(cle) {
            Some((marque, nom)) if a.presents.contains(marque) => {
                format!("La photo {} arrive…", de(nom))
            }
            Some((_, nom)) => format!("La photo {}, hors ligne", de(nom)),
            None => "Cette photo attend quelqu'un qui l'a".to_string(),
        }
    }
}

/// « de Loup Émeraude », « d'Aigle Émeraude » : l'élision devant une voyelle (un *h* aspiré
/// comme celui de « Hibou » n'élide pas — aucun *h* ne le fait ici).
fn de(nom: &str) -> String {
    let voyelle = nom.chars().next().is_some_and(|c| {
        let c = c.to_lowercase().next().unwrap_or(c);
        "aeiouyàâäéèêëîïôöùûüÿæœ".contains(c)
    });
    if voyelle {
        format!("d'{nom}")
    } else {
        format!("de {nom}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLE: &str = "objet:chat";

    fn magasin_qui_attend() -> Magasin {
        let mut m = Magasin::nouveau();
        m.echecs.insert(CLE.into());
        m
    }

    #[test]
    fn q18_hors_partage_le_cadre_garde_son_libelle() {
        let m = magasin_qui_attend();
        assert_eq!(m.libelle(Some(CLE), "chat"), "Image [chat]");
    }

    #[test]
    fn q18_le_cadre_dit_de_qui_est_la_photo_et_s_il_est_la() {
        let mut m = magasin_qui_attend();
        m.presents(BTreeSet::new());
        assert_eq!(
            m.libelle(Some(CLE), "chat"),
            "Cette photo attend quelqu'un qui l'a"
        );
        let avant = m.changements;
        m.posee_par(CLE, "l0up", "Loup Émeraude");
        assert!(m.changements > avant, "le lot des photos se refait");
        assert_eq!(
            m.libelle(Some(CLE), "chat"),
            "La photo de Loup Émeraude, hors ligne"
        );
        let avant = m.changements;
        m.presents(BTreeSet::from(["l0up".to_string()]));
        assert!(m.changements > avant, "il arrive : le lot se refait");
        assert_eq!(
            m.libelle(Some(CLE), "chat"),
            "La photo de Loup Émeraude arrive…"
        );
        let avant = m.changements;
        m.presents(BTreeSet::from(["l0up".to_string()]));
        assert_eq!(m.changements, avant, "rien n'a changé : rien ne se refait");
        m.hors_partage();
        assert_eq!(m.libelle(Some(CLE), "chat"), "Image [chat]");
    }

    #[test]
    fn q18_le_nom_s_elide_devant_une_voyelle() {
        assert_eq!(de("Aigle Émeraude"), "d'Aigle Émeraude");
        assert_eq!(de("Écureuil Or"), "d'Écureuil Or");
        assert_eq!(de("Hibou Indigo"), "de Hibou Indigo");
        assert_eq!(de("Loup Émeraude"), "de Loup Émeraude");
    }

    #[test]
    fn q18_une_photo_qu_on_decode_encore_n_attend_personne() {
        let mut m = Magasin::nouveau();
        m.posee_par(CLE, "l0up", "Loup Émeraude");
        assert_eq!(m.libelle(Some(CLE), "chat"), "Image [chat]");
    }
}
