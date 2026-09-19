//! Lire la chronique : ce qui coûte, ce qui a gelé, et pourquoi.
//!
//! # Ce que ce rapport doit répondre
//!
//! Trois questions, dans cet ordre, parce que c'est l'ordre où elles servent :
//!
//! 1. **Est-ce fluide ?** Pas en moyenne — au pire centile, qui est ce qui se ressent.
//! 2. **Quel geste coûte ?** Déplacer la vue, glisser un nœud, zoomer : ils n'ont ni le même
//!    coût ni la même cause, et les confondre a déjà fait optimiser dans le vide.
//! 3. **Pourquoi celui-là ?** Le temps d'un geste se répartit entre ses postes, et la
//!    répartition d'un geste lent ne ressemble pas à celle d'un geste rapide.
//!
//! Puis les images les plus lentes, en entier : un histogramme dit *combien* d'images ont
//! gelé, jamais *pourquoi*.

use super::{Chronique, Geste, Instantane};

/// Le budget d'une image à la cadence plancher de la charte, en microsecondes.
///
/// **Cent** images par seconde, et non soixante : la charte a été révisée le 18/09/2026 et ne
/// négocie plus ce plancher — « si on est en dessous, alors go pixeliser tout ». Ce rapport
/// annonçait donc 16,7 ms comme « le plancher de la charte » alors que la charte en demandait
/// dix, et comptait pour réussies des images qui ne l'étaient pas.
///
/// La valeur ne se redéclare pas ici : c'est [`crate::cadence::BUDGET_TOTAL`], la même que
/// celle qui borne le travail de fond. Deux copies d'un même plancher finissent toujours par
/// diverger, et celle-ci avait déjà divergé.
const BUDGET_PLANCHER_US: u32 = crate::cadence::BUDGET_TOTAL.as_micros() as u32;

impl Chronique {
    /// Le rapport complet, en texte.
    pub fn rapport(&self) -> String {
        let mut t = String::new();
        self.ecrire_le_resume(&mut t);
        self.ecrire_les_gestes(&mut t);
        self.ecrire_la_navigation(&mut t);
        self.ecrire_les_reveils(&mut t);
        self.ecrire_les_pires(&mut t);
        t
    }

    /// Ce que la main a demandé, et le temps qu'il a fallu pour que l'écran le montre.
    ///
    /// # Pourquoi cette section existe à part
    ///
    /// Tout le reste de ce rapport mesure ce qu'une image **coûte**. Rien n'y disait le délai
    /// entre le geste et ce qu'on en voit — et c'est pourtant lui qui se ressent. Une
    /// application à cent images par seconde dont chaque image montre l'état d'il y a
    /// cinquante millisecondes paraît molle, et aucune durée d'image ne l'explique.
    fn ecrire_la_navigation(&self, t: &mut String) {
        let c = self.navigation.comptes();
        let (mesurees, pire) = self.navigation.mesurees();
        if c.total() == 0 {
            return;
        }
        t.push_str("  Ce que la main demande, et le temps que l'ecran met a le montrer\n\n");
        t.push_str(&format!(
            "  {} pincement(s), {} zoom(s) au clavier, {} deplacement(s), {} pris pour un cran de souris\n",
            c.pincements, c.zooms, c.pans, c.crans
        ));
        // Le pincement est le geste que Glucose ne recevait pas du tout : tant que ce compte
        // reste nul alors qu'on a pince, le pont de plateforme ne sert pas, et c'est cela
        // qu'il faut corriger -- pas la cadence.
        if c.pincements == 0 && c.pans > 0 {
            t.push_str(
                "    aucun pincement reconnu : si on a pince, le systeme ne le marque pas ici\n",
            );
        }
        // Le pont de plateforme et la boucle d'evenements doivent voir le meme nombre de
        // pincements. Un ecart veut dire qu'un message marque n'a pas donne d'evenement -- ou
        // l'inverse -- et c'est exactement ce qui faisait dezoomer des translations pures.
        let marquees = crate::interactions::pincement::marques();
        if marquees != c.pincements {
            t.push_str(&format!(
                "    {marquees} message(s) marque(s) par le systeme pour {} pincement(s) traite(s)\n",
                c.pincements
            ));
        }
        if c.crans > 0 && c.pans > 0 {
            t.push_str(
                "    un cran suppose au milieu d'un glissement coute un saut d'echelle visible\n",
            );
        }
        if mesurees > 0 {
            t.push_str(&format!(
                "  latence geste -> ecran : median {:.1}ms, p90 {:.1}ms, p99 {:.1}ms, pire {:.1}ms\n",
                ms(self.navigation.centile(0.50)),
                ms(self.navigation.centile(0.90)),
                ms(self.navigation.centile(0.99)),
                ms(pire),
            ));
        }
        t.push('\n');
    }

    /// Pourquoi l'application ne dort pas, et ce que ça lui coûte.
    ///
    /// # La question que le reste du rapport ne pose jamais
    ///
    /// Tout ce qui précède mesure ce qu'une image **coûte**. Rien n'y disait si elle avait
    /// lieu d'être. Une image inutile est pourtant la plus chère de toutes : elle se paie en
    /// entier et ne montre rien de neuf.
    ///
    /// Deux lignes y répondent. Celle des images évitées dit combien de fois l'image
    /// précédente était encore exacte — c'est la part du travail qui a servi deux fois.
    /// Celle des réveils dit ce qui empêchait de dormir, raison par raison : une raison qui
    /// tient l'application éveillée sur la quasi-totalité des images est celle à supprimer,
    /// et aucune durée ne l'aurait désignée.
    fn ecrire_les_reveils(&self, t: &mut String) {
        let Some(evitee) = self.part_evitee() else {
            return;
        };
        t.push_str(
            "  Pourquoi l'application ne dort pas

",
        );
        t.push_str(&format!(
            "  {:.0} % des images n'ont RIEN eu a redessiner -- l'image d'avant suffisait
",
            100.0 * evitee
        ));
        let total = self.rendues().max(1);
        let mut raisons: Vec<(&'static str, u64)> = self
            .reveils()
            .filter(|(_, images)| *images > 0)
            .map(|(r, images)| (r.nom(), images))
            .collect();
        if raisons.is_empty() {
            t.push_str(
                "    aucune : chaque image a ete demandee par un geste

",
            );
            return;
        }
        raisons.sort_by_key(|(_, images)| std::cmp::Reverse(*images));
        for (nom, images) in raisons {
            let part = images as f64 / total as f64;
            t.push_str(&format!(
                "      {:<22} {:>5.1}%  {}
",
                nom,
                100.0 * part,
                barre(part)
            ));
        }
        t.push('\n');
    }

    fn ecrire_le_resume(&self, t: &mut String) {
        let secondes = self.duree().as_secs_f64().max(0.001);
        t.push_str("=== Chronique de la session ===\n\n");
        t.push_str(&format!(
            "  duree {:.0} s, {} images, {:.0} par seconde en moyenne\n",
            secondes,
            self.rendues(),
            self.rendues() as f64 / secondes
        ));

        let ratees = self
            .pires()
            .iter()
            .filter(|p| p.duree_us > BUDGET_PLANCHER_US)
            .count();
        if ratees > 0 {
            t.push_str(&format!(
                "  au moins {ratees} image(s) au-dessus de {:.1} ms -- le plancher de la charte\n",
                f64::from(BUDGET_PLANCHER_US) / 1000.0
            ));
        }
        // Se lit sur TOUTES les images, et jamais sur les plus lentes : celles-ci sont
        // justement celles où la vignette a manqué, donc y lire cette part reviendrait à
        // conclure d'un échantillon qui ne peut contenir que des échecs.
        if let Some(part) = self.part_par_vignette() {
            t.push_str(&format!(
                "  {:.0} % des photos se posent depuis une vignette -- le chemin le moins cher\n",
                100.0 * part
            ));
        }
        // Le résidu est la seule ligne de ce résumé qui apprenne quelque chose de neuf : elle
        // dit ce que le modèle de coût ne comprend pas encore de cette machine.
        if let Some(justesse) = self.justesse_du_modele() {
            t.push_str(&format!(
                "  le modele prevoit a {:.0} % de ce qui arrive -- l'ecart est ce qu'il ignore\n",
                100.0 * justesse
            ));
        }
        // La scene rendue plus petite est le levier le plus puissant, et le seul qui borne
        // TOUTES les passes a la fois : il faut donc savoir a quel point on s'en sert.
        if let Some((part, moyen)) = self.part_reduite().filter(|(p, _)| *p > 0.0) {
            t.push_str(&format!(
                "  {:.0} % des images se sont rendues plus petites -- facteur moyen {moyen:.2}\n",
                100.0 * part
            ));
        }
        if self.part_pixelisee().is_some_and(|p| p > 0.0) {
            let part = self.part_pixelisee().unwrap_or(0.0);
            t.push_str(&format!(
                "  {:.0} % des images se sont pixelisees pour tenir les cent par seconde\n",
                100.0 * part
            ));
        }
        t.push('\n');
    }

    /// Un geste par ligne, avec sa distribution. C'est le tableau qui dit où chercher.
    fn ecrire_les_gestes(&self, t: &mut String) {
        t.push_str("  Ce que chaque geste coute -- au centile, pas en moyenne\n\n");
        t.push_str(&format!(
            "  {:<18} {:>8} {:>9} {:>9} {:>9} {:>9}\n",
            "geste", "images", "median", "p90", "p99", "pire"
        ));
        for geste in Geste::TOUS {
            let poste = self.poste_du_geste(geste);
            if poste.0 == 0 {
                continue;
            }
            t.push_str(&format!(
                "  {:<18} {:>8} {:>7.2}ms {:>7.2}ms {:>7.2}ms {:>7.2}ms\n",
                geste.nom(),
                poste.0,
                ms(poste.1),
                ms(poste.2),
                ms(poste.3),
                ms(poste.4),
            ));
        }
        t.push('\n');
        self.ecrire_la_repartition(t);
    }

    /// Où va le temps, geste par geste et poste par poste.
    fn ecrire_la_repartition(&self, t: &mut String) {
        t.push_str("  Ou va le temps de chaque geste\n\n");
        for geste in Geste::TOUS {
            let Some(parts) = self.parts_du_geste(geste) else {
                continue;
            };
            t.push_str(&format!("  {} :\n", geste.nom()));
            let total: u64 = parts.iter().map(|(_, us)| *us).sum();
            if total == 0 {
                continue;
            }
            let mut tries = parts;
            tries.sort_by_key(|(_, us)| std::cmp::Reverse(*us));
            for (nom, us) in tries.into_iter().take(6) {
                let part = us as f64 / total as f64;
                if part < 0.01 {
                    break;
                }
                t.push_str(&format!(
                    "      {:<14} {:>5.1}%  {}\n",
                    nom,
                    100.0 * part,
                    barre(part)
                ));
            }
            t.push('\n');
        }
    }

    /// Les images les plus lentes, avec tout leur contexte : c'est là que se lit la cause.
    fn ecrire_les_pires(&self, t: &mut String) {
        if self.pires().is_empty() {
            return;
        }
        t.push_str("  Les images les plus lentes, et ce qu'elles faisaient\n\n");
        t.push_str(&format!(
            "  {:>9} {:>10} {:<18} {:>7} {:>7} {:>5} {:>8} {:>10} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>7} {:>8}\n",
            "a",
            "duree",
            "geste",
            "noeuds",
            "photos",
            "mip",
            "ecrans",
            "redessine",
            "vign",
            "file",
            "perim",
            "pret",
            "orph",
            "abdn",
            "envoi",
            "cache"
        ));
        for p in self.pires().iter().take(12) {
            t.push_str(&format!(
                "  {:>7.1}s {:>8.2}ms {:<18} {:>7} {:>7} {:>5} {:>7.1}x {:>9.0}% {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>4} Mo {:>5} Mo\n",
                f64::from(p.instant_ms) / 1000.0,
                f64::from(p.duree_us) / 1000.0,
                nom_du_geste(p),
                p.noeuds,
                p.photos,
                p.par_vignette,
                f64::from(p.surcouverture) / 100.0,
                100.0 * p.part_redessinee(),
                p.vignettes,
                p.vignettes_en_attente,
                p.vignettes_perimees,
                p.vignettes_pretes,
                p.vignettes_orphelines,
                p.vignettes_abandonnees,
                p.blit_mo,
                p.images_mo,
            ));
            self.ecrire_les_postes(t, p);
        }
    }

    /// Le détail d'une image lente : les trois postes qui l'ont dominée.
    fn ecrire_les_postes(&self, t: &mut String, p: &Instantane) {
        let mut postes: Vec<(&'static str, u32)> = p
            .postes_us
            .iter()
            .enumerate()
            .filter_map(|(i, us)| (*us > 0).then_some(*us).zip(self.nom_du_poste(i)))
            .map(|(us, nom)| (nom, us))
            .collect();
        if postes.is_empty() {
            return;
        }
        postes.sort_by_key(|(_, us)| std::cmp::Reverse(*us));
        let detail: Vec<String> = postes
            .into_iter()
            .take(3)
            .map(|(nom, us)| format!("{nom} {:.2}ms", f64::from(us) / 1000.0))
            .collect();
        t.push_str(&format!("            dont {}\n", detail.join(", ")));
    }

    /// `(images, median, p90, p99, pire)` pour ce geste, en microsecondes.
    fn poste_du_geste(&self, geste: Geste) -> (u64, u32, u32, u32, u32) {
        (
            self.rendues_du_geste(geste),
            self.centile_du_geste(geste, 0.50),
            self.centile_du_geste(geste, 0.90),
            self.centile_du_geste(geste, 0.99),
            self.pire_du_geste(geste),
        )
    }
}

fn ms(us: u32) -> f64 {
    f64::from(us) / 1000.0
}

fn nom_du_geste(p: &Instantane) -> &'static str {
    Geste::TOUS
        .get(p.geste as usize)
        .copied()
        .unwrap_or(Geste::Repos)
        .nom()
}

/// Une barre proportionnelle, en douze caractères.
fn barre(part: f64) -> String {
    let pleins = (part * 12.0).round().clamp(0.0, 12.0) as usize;
    let mut s = String::with_capacity(12);
    for i in 0..12 {
        s.push(if i < pleins { '#' } else { '.' });
    }
    s
}
