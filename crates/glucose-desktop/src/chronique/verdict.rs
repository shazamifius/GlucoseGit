//! Le verdict : ce qui cloche dans cette session, du plus grave au moins, et chiffré.
//!
//! # Pourquoi un rapport doit conclure et non lister
//!
//! L'utilisateur l'a dit dans ces termes : les traces sont « pas assez complètes », et surtout
//! « trop faciles et superficielles ». C'est exact, et le défaut n'est pas la quantité de
//! chiffres — il y en avait déjà beaucoup. C'est qu'**aucun ne disait lequel regarder**.
//!
//! Une chronique qui donne trente nombres corrects sans les ordonner oblige son lecteur à
//! refaire l'analyse à chaque fois, donc à la refaire différemment à chaque fois. Trois
//! sessions de travail ont été perdues à optimiser un poste qui n'était pas le goulot.
//!
//! # Comment la gravité se calcule, et pourquoi elle n'a pas de seuil choisi
//!
//! Chaque constat compare une mesure à une **référence qui existe déjà dans le projet** :
//!
//! * le plancher de la charte — dix millisecondes, et il ne se négocie pas ;
//! * la période de l'écran — lue, jamais supposée ;
//! * l'avance que le contenu devait parcourir pendant l'image — la seule échelle à laquelle
//!   un tressaut de position veuille dire quelque chose ;
//! * zéro, pour ce qui ne devrait pas exister du tout : du travail refait à l'identique.
//!
//! La gravité est le rapport de la mesure à sa référence. Elle n'a donc pas d'unité, et les
//! constats se comparent entre eux — ce qui est précisément ce qu'on veut savoir : lequel
//! d'abord.
//!
//! Un constat dont la gravité ne dépasse pas un ne paraît pas. Le rapport ne dit donc rien
//! quand tout va bien, ce qui est la seule façon pour que ce qu'il dit compte.

use super::Chronique;

/// Un défaut constaté, avec de quoi décider s'il passe avant un autre.
#[derive(Debug, Clone)]
pub struct Constat {
    /// Combien de fois la référence est dépassée. Un vaut « à la limite ».
    pub gravite: f64,
    /// Le mot qui nomme la nature du défaut, en un seul.
    pub nature: &'static str,
    /// Ce qui est mesuré, en une ligne.
    pub fait: String,
    /// Ce que ce fait implique — jamais une hypothèse de cause, seulement une conséquence.
    pub consequence: String,
}

impl Chronique {
    /// Ce qui cloche dans cette session, du plus grave au moins.
    ///
    /// Vide quand rien ne dépasse sa référence, et c'est voulu : un rapport qui trouve
    /// toujours quelque chose à dire n'apprend plus rien.
    pub fn verdict(&self) -> Vec<Constat> {
        let mut constats: Vec<Constat> = [
            self.constat_du_tressaut(),
            self.constat_de_la_cadence(),
            self.constat_du_travail_refait(),
            self.constat_de_la_finesse(),
            self.constat_de_la_latence(),
        ]
        .into_iter()
        .flatten()
        .filter(|c| c.gravite > 1.0)
        .collect();
        constats.sort_by(|a, b| b.gravite.total_cmp(&a.gravite));
        constats
    }

    /// Le contenu se pose-t-il là où il devrait ?
    ///
    /// La référence est l'**avance attendue** : sauter de trois pixels quand on en parcourt
    /// cent ne se voit pas ; sauter de trente quand on en parcourt dix fait reculer le
    /// contenu, et c'est exactement ce que l'utilisateur décrit par « ultra saccadé ».
    fn constat_du_tressaut(&self) -> Option<Constat> {
        let (saut_median, saut_p99, saut_pire) = self.rythme.sauts();
        let avance = self.rythme.avance_mediane().max(1);
        // **Le p99, et non la médiane.** Tout ce module répète que ce qui se ressent est le
        // pire centile, et la gravité du tressaut ne fait pas exception : un mouvement dont
        // une image sur cent se pose à soixante pixels de sa trajectoire est vu saccadé, quand
        // bien même les quatre-vingt-dix-neuf autres tombent juste.
        let gravite = f64::from(saut_p99) / f64::from(avance);
        let irreguliere = self.rythme.irregularite().unwrap_or(0.0);
        Some(Constat {
            gravite,
            nature: "tressaut",
            fait: format!(
                "le contenu se pose {saut_median} px a cote de sa trajectoire (p99 {saut_p99}, \
                 pire {saut_pire}), pour {avance} px d'avance attendue par image"
            ),
            consequence: format!(
                "le mouvement est integre sur un pas et montre pendant un autre ; {:.0} % des \
                 images changent de duree d'affichage",
                100.0 * irreguliere
            ),
        })
    }

    /// La cadence tient-elle le plancher de la charte ?
    fn constat_de_la_cadence(&self) -> Option<Constat> {
        let plancher = crate::cadence::BUDGET_TOTAL.as_micros() as u32;
        let ratees = self.images_au_dessus(plancher);
        let total = self.rendues();
        if total == 0 {
            return None;
        }
        let part = ratees as f64 / total as f64;
        // La référence est **une image sur cent** : au-delà, le plancher n'est plus un
        // plancher. La charte demande « constant, quoi qu'il se passe » ; un pour cent est
        // déjà une concession, pas un objectif.
        let gravite = part / 0.01;
        let (_, _, p99, pire) = self.rythme.intervalles();
        Some(Constat {
            gravite,
            nature: "cadence",
            fait: format!(
                "{ratees} images sur {total} ({:.0} %) depassent {:.1} ms",
                100.0 * part,
                f64::from(plancher) / 1000.0
            ),
            consequence: format!(
                "l'ecran garde parfois une image {:.1} ms (pire {:.1} ms) : c'est un gel, pas \
                 une cadence basse",
                f64::from(p99) / 1000.0,
                f64::from(pire) / 1000.0
            ),
        })
    }

    /// Combien du travail d'une image se refait à l'identique à la suivante ?
    ///
    /// La référence est **zéro** : une scène qui n'a pas changé ne devrait rien coûter. Comme
    /// zéro ne peut pas servir de dénominateur, la gravité se lit sur la part de la fenêtre
    /// repeinte alors qu'elle aurait pu ne pas l'être — soit l'exact complément des images
    /// évitées.
    fn constat_du_travail_refait(&self) -> Option<Constat> {
        let evitee = self.part_evitee()?;
        let refait = 1.0 - evitee;
        // Une image sur dix qui n'a rien à redessiner est le minimum qu'une scène immobile
        // devrait produire ; en deçà, quelque chose salit tout à chaque image.
        let gravite = refait / 0.9;
        let par_vignette = self.part_par_vignette().unwrap_or(0.0);
        Some(Constat {
            gravite,
            nature: "travail refait",
            fait: format!(
                "{:.0} % des images redessinent alors que l'image d'avant aurait pu suffire, et \
                 {:.0} % des photos passent par le chemin le moins cher",
                100.0 * refait,
                100.0 * par_vignette
            ),
            consequence: "chaque image repart de zero sur une scene qui a bouge de quelques pixels"
                .to_string(),
        })
    }

    /// La scène a-t-elle dû céder sur sa finesse ?
    fn constat_de_la_finesse(&self) -> Option<Constat> {
        let (part, facteur) = self.part_reduite()?;
        // Une image sur vingt rendue plus petite passe inaperçue ; au-delà, la dégradation
        // devient le régime normal, ce que la charte refuse — « net à quasi 100 % à l'arrêt ».
        let gravite = part / 0.05;
        Some(Constat {
            gravite,
            nature: "finesse",
            fait: format!(
                "{:.0} % des images se rendent plus petites, facteur moyen {facteur:.2}",
                100.0 * part
            ),
            consequence: format!(
                "a facteur {facteur:.2}, un pixel de l'ecran en montre {:.1} du canevas : c'est \
                 ce que l'utilisateur voit comme « trop pixelise »",
                facteur * facteur
            ),
        })
    }

    /// Le temps entre le geste et ce qu'on en voit.
    ///
    /// La référence est la **période de l'écran** : au-delà, la main attend au moins un
    /// balayage de plus que le minimum que la machine sait faire.
    fn constat_de_la_latence(&self) -> Option<Constat> {
        let periode = self.rythme.periode()?;
        let (mesurees, pire) = self.navigation.mesurees();
        if mesurees == 0 {
            return None;
        }
        let p99 = self.navigation.centile(0.99);
        let reference = periode.as_micros().max(1) as f64;
        // Deux périodes : une pour rendre, une pour présenter. C'est le minimum incompressible
        // d'un rendu qui n'attend rien, et non un confort.
        let gravite = f64::from(p99) / (2.0 * reference);
        Some(Constat {
            gravite,
            nature: "latence",
            fait: format!(
                "le geste met {:.1} ms a paraitre au p99 (pire {:.1} ms), pour un ecran qui bat \
                 toutes les {:.2} ms",
                f64::from(p99) / 1000.0,
                f64::from(pire) / 1000.0,
                reference / 1000.0
            ),
            consequence: "la main precede ce qu'elle voit, et cela se ressent comme de la mollesse"
                .to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::chronique::{Chronique, Instantane};

    /// Une session parfaite ne doit produire **aucun** constat.
    ///
    /// C'est ce qui donne du poids à ceux qu'elle produit : un verdict qui trouve toujours
    /// quelque chose à dire ne se lit plus.
    #[test]
    fn test_une_session_saine_ne_dit_rien() {
        let mut c = Chronique::nouvelle();
        for _ in 0..500 {
            c.enregistrer(Instantane {
                duree_us: 3_000,
                region_px: 0,
                fenetre_px: 1_000_000,
                reduction: 1,
                ..Default::default()
            });
        }
        let verdict = c.verdict();
        assert!(
            verdict.is_empty(),
            "rien ne cloche, le verdict doit se taire : {verdict:?}"
        );
    }

    /// Une session qui rate le plancher doit le dire, et le compter sur TOUTES les images.
    #[test]
    fn test_le_plancher_rate_se_compte_sur_toute_la_session() {
        let mut c = Chronique::nouvelle();
        for _ in 0..1000 {
            c.enregistrer(Instantane {
                duree_us: 40_000,
                region_px: 0,
                fenetre_px: 1_000_000,
                reduction: 1,
                ..Default::default()
            });
        }
        let cadence = c
            .verdict()
            .into_iter()
            .find(|v| v.nature == "cadence")
            .expect("la cadence doit etre mise en cause");
        assert!(
            cadence.fait.contains("1000"),
            "les mille images ratees doivent se compter, pas les trente-deux pires : {}",
            cadence.fait
        );
    }

    /// Les constats se rangent du plus grave au moins, sans quoi le rapport n'ordonne rien.
    #[test]
    fn test_le_verdict_est_trie_du_plus_grave_au_moins() {
        let mut c = Chronique::nouvelle();
        for _ in 0..200 {
            c.enregistrer(Instantane {
                duree_us: 40_000,
                region_px: 1_000_000,
                fenetre_px: 1_000_000,
                reduction: 4,
                ..Default::default()
            });
        }
        let verdict = c.verdict();
        assert!(verdict.len() >= 2, "plusieurs defauts sont presents");
        for paire in verdict.windows(2) {
            assert!(
                paire[0].gravite >= paire[1].gravite,
                "le verdict n'est pas trie : {:?} avant {:?}",
                paire[0].nature,
                paire[1].nature
            );
        }
    }
}
