//! **La mémoire, étage par étage** (ETAGES-1, fiche 32) : ce que Glucose tient, ce qu'il a
//! laissé au système, et ce qui est allé et venu entre les deux.
//!
//! # Pourquoi la mémoire de travail ne suffisait pas
//!
//! La chronique disait « 1 550 Mo » à la fin de la longue session du 23/09, et rien de plus.
//! Il a fallu une demi-journée, deux bancs et un mauvais document pour établir que c'étaient
//! les images — les 243 photos décodées du document, 1 186 Mo à elles seules. Un chiffre
//! global ne dit pas **qui** : il envoie chercher, et on cherche au mauvais endroit.
//!
//! Depuis ETAGES-1, la question a une deuxième moitié : ce que Glucose **offre** au système
//! ne compte plus dans sa mémoire de travail, mais reste là tant que personne ne le réclame.
//! Les deux se lisent ici, ensemble — sans quoi une mémoire de travail qui baisse passerait
//! pour une économie, alors qu'elle dit seulement qui a le droit de reprendre.

use crate::renderer::magasin::Mouvements;

/// Ce que la veille relève, une fois par seconde.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Releve {
    /// Les octets des images que la mémoire vive tient — ce que l'écran montre, et la vue
    /// d'ensemble.
    pub tenus: u64,
    /// Ceux qu'elle a offerts au système : là tant qu'il ne les reprend pas.
    pub offerts: u64,
    /// Les lettres rastérisées que la typographie garde.
    pub lettres: u64,
    /// Ce que l'étagement a fait depuis le début.
    pub mouvements: Mouvements,
    /// Le plus grand cran commun des photos sur la carte (ETAGES-3).
    pub cran_pire: u32,
    /// Combien d'images ont dû se composer sur le processeur, la carte ne tenant pas l'écran.
    pub images_debordees: u64,
}

/// Le dernier relevé, et le pire de chaque étage.
#[derive(Debug, Default)]
pub struct Etages {
    dernier: Option<Releve>,
    tenus_pire: u64,
    offerts_pire: u64,
    lettres_pire: u64,
}

const MO: f64 = 1024.0 * 1024.0;

impl Etages {
    /// Range un relevé.
    pub fn noter(&mut self, r: Releve) {
        self.tenus_pire = self.tenus_pire.max(r.tenus);
        self.offerts_pire = self.offerts_pire.max(r.offerts);
        self.lettres_pire = self.lettres_pire.max(r.lettres);
        self.dernier = Some(r);
    }

    /// **Les lignes du rapport**, ou rien si aucun relevé n'a été pris — un zéro se lirait
    /// comme une mesure (fiche 17 § 3.1).
    pub fn ecrire(&self, t: &mut String) {
        let Some(r) = self.dernier else {
            return;
        };
        t.push_str(&format!(
            "  memoire vive des images : {:.0} Mo tenus et {:.0} Mo offerts au systeme a la fin -- au pire {:.0} Mo tenus, {:.0} Mo offerts\n",
            r.tenus as f64 / MO,
            r.offerts as f64 / MO,
            self.tenus_pire as f64 / MO,
            self.offerts_pire as f64 / MO
        ));
        let m = r.mouvements;
        t.push_str(&format!(
            "    niveaux offerts {}, repris {}, jetes par le systeme {} -- un niveau jete se redecode depuis son fichier\n",
            m.offerts, m.repris, m.perdus
        ));
        t.push_str(&format!(
            "  lettres gardees : {:.0} Mo a la fin, {:.0} Mo au pire\n",
            r.lettres as f64 / MO,
            self.lettres_pire as f64 / MO
        ));
        // Ce qui ne se voit que si la carte a manqué de place : zéro cran, c'est le cas normal,
        // et il ne mérite pas une ligne.
        if r.cran_pire > 0 || r.images_debordees > 0 {
            t.push_str(&format!(
                "  la carte manquait de place : photos reduites de {} cran(s) au plus, {} image(s) composee(s) par le processeur\n",
                r.cran_pire, r.images_debordees
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_le_rapport_se_tait_sans_releve_et_garde_le_pire_de_chaque_etage() {
        let mut e = Etages::default();
        let mut t = String::new();
        e.ecrire(&mut t);
        assert!(t.is_empty(), "sans relevé, rien ne s'écrit");

        let releve = |tenus: u64, offerts: u64| Releve {
            tenus: tenus << 20,
            offerts: offerts << 20,
            lettres: 3 << 20,
            mouvements: Mouvements {
                offerts: 12,
                repris: 4,
                perdus: 1,
            },
            cran_pire: 0,
            images_debordees: 0,
        };
        e.noter(releve(900, 10));
        e.noter(releve(120, 1060));
        e.ecrire(&mut t);
        assert!(t.contains("120 Mo tenus et 1060 Mo offerts"), "{t}");
        assert!(t.contains("au pire 900 Mo tenus, 1060 Mo offerts"), "{t}");
        assert!(t.contains("offerts 12, repris 4, jetes par le systeme 1"), "{t}");
        assert!(!t.contains("manquait de place"), "zero cran ne se dit pas : {t}");

        e.noter(Releve {
            cran_pire: 2,
            images_debordees: 5,
            ..releve(120, 1060)
        });
        let mut t = String::new();
        e.ecrire(&mut t);
        assert!(t.contains("reduites de 2 cran(s) au plus, 5 image(s)"), "{t}");
    }
}
