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

mod oeil;

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
    /// Le rapport complet, en texte.
    ///
    /// # L'ordre n'est pas décoratif
    ///
    /// Le verdict vient en premier parce que tout ce qui suit répond à « où va le temps », ce
    /// qui est la bonne question **une fois qu'on sait laquelle poser**. La machine vient
    /// ensuite : sans elle, aucune durée ne se relit — les mêmes dix millisecondes ne disent
    /// pas la même chose selon que la présentation attendait un balayage. Puis le rythme, qui
    /// est la seule section à parler de ce que l'œil reçoit. Le reste suit.
    pub fn rapport(&self) -> String {
        let mut t = String::new();
        t.push_str(
            "=== Chronique de la session ===

",
        );
        self.ecrire_le_verdict(&mut t);
        self.ecrire_la_machine(&mut t);
        self.ecrire_le_resume(&mut t);
        self.ecrire_le_rythme(&mut t);
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
        t.push_str(&format!(
            "  duree {:.0} s, {} images rendues\n",
            secondes,
            self.rendues(),
        ));
        // **Et non « images ÷ durée »**, qui compte le temps où personne ne demandait rien :
        // une application qui dort dix secondes puis rend cent images en une annonce neuf
        // images par seconde, et aucune de ces neuf n'a existé. Ce qui se ressent est le
        // rythme des images consécutives.
        if let Some(vue) = self.rythme.cadence_vue() {
            t.push_str(&format!(
                "  {vue:.0} images par seconde entre deux images consecutives\n"
            ));
        }

        // **Sur toutes les images, et non sur les trente-deux plus lentes.** Cette ligne
        // annonçait « au moins 32 » quelle que soit la session, parce qu'elle comptait les
        // membres d'une liste bornée par construction : une session ratant mille images et
        // une en ratant trente-trois disaient le même nombre.
        let ratees = self.images_au_dessus(BUDGET_PLANCHER_US);
        if ratees > 0 {
            t.push_str(&format!(
                "  {ratees} image(s) sur {} au-dessus de {:.1} ms -- le plancher de la charte\n",
                self.rendues(),
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
        // La grille de tuiles : ce qu'elle a epargne. C'est la mesure que les vignettes n'ont
        // jamais su donner, et c'est elle qui dit si TUILE-1 sert.
        if let Some(part) = self.part_de_tuiles_reprises() {
            t.push_str(&format!(
                "  {:.0} % des tuiles ont servi telles quelles -- {} peintes, {} reprises\n",
                100.0 * part,
                self.tuiles_peintes(),
                self.tuiles_reprises()
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

    /// Où va le temps, geste par geste et poste par poste — **au centile, comme le reste**.
    ///
    /// # Pourquoi il n'y a plus de pourcentage
    ///
    /// Il y en avait un, et il mentait deux fois. D'abord parce qu'il venait d'une **somme**,
    /// qu'une seule image aberrante suffisait à décider (voir `Poste::postes`). Ensuite parce
    /// qu'un pourcentage invite à additionner, et **des centiles ne s'additionnent pas** : la
    /// médiane d'une somme n'est pas la somme des médianes, et les postes d'un geste n'ont
    /// aucune raison d'être médians sur la même image.
    ///
    /// Restent des millisecondes, qui sont vraies sans rien supposer, et la durée médiane du
    /// geste en en-tête, qui donne l'échelle. La barre compare les postes **entre eux** — elle
    /// est relative au plus coûteux, pas à un tout, donc personne ne peut la sommer.
    ///
    /// Le pire est affiché à côté du typique : c'est lui qui désigne un gel, et la section des
    /// images les plus lentes le décompose.
    fn ecrire_la_repartition(&self, t: &mut String) {
        t.push_str("  Ou va le temps de chaque geste -- au centile, pas en moyenne\n\n");
        for geste in Geste::TOUS {
            let Some(parts) = self.parts_du_geste(geste) else {
                continue;
            };
            let reference = self.median_du_geste(geste);
            // Un geste dont aucune marque n'a jamais rien coute n'a pas d'en-tete a ecrire :
            // sans cette garde, le rapport annoncait un geste puis ne disait rien de lui.
            if reference == 0 || parts.is_empty() {
                continue;
            }
            t.push_str(&format!(
                "  {} (image mediane {:.2}ms) :\n",
                geste.nom(),
                ms(reference)
            ));
            t.push_str("      poste            median       p99      pire\n");
            let mut tries = parts;
            tries.sort_by_key(|(_, h)| std::cmp::Reverse(h.centile(0.5)));
            let tete = tries.first().map_or(1, |(_, h)| h.centile(0.5).max(1));
            for (nom, h) in tries.into_iter().take(6) {
                let median = h.centile(0.5);
                t.push_str(&format!(
                    "      {:<14} {:>7.2}ms {:>7.2}ms {:>9.2}ms  {}\n",
                    nom,
                    ms(median),
                    ms(h.centile(0.99)),
                    ms(h.pire()),
                    barre(f64::from(median) / f64::from(tete))
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
            "  {:>9} {:>10} {:<18} {:>7} {:>7} {:>5} {:>8} {:>10} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8} {:>6} {:>7} {:>6} {:>5} {:>7} {:>8}
",
            "a",
            "duree",
            "geste",
            "noeuds",
            "photos",
            // Cette colonne a porte l'en-tete « mip » pendant toute son existence alors
            // qu'elle compte les photos posees DEPUIS UNE VIGNETTE. Un en-tete faux est pire
            // qu'une colonne absente : il a fait chercher un defaut de pyramide la ou il n'y
            // en avait pas.
            "vign/px",
            "ecrans",
            "redessine",
            "vign",
            "file",
            "perim",
            "pret",
            "orph",
            "abdn",
            "tuiles",
            "reprises",
            "text",
            "kpx",
            "report",
            "surf",
            "envoi",
            "cache"
        ));
        for p in self.pires().iter().take(12) {
            t.push_str(&format!(
                "  {:>7.1}s {:>8.2}ms {:<18} {:>7} {:>7} {:>5} {:>7.1}x {:>9.0}% {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>8} {:>6} {:>7} {:>6} {:>5} {:>4} Mo {:>5} Mo\n",
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
                p.tuiles_peintes,
                p.tuiles_reprises,
                p.textures_faites,
                p.textures_kpx,
                p.textures_reportees,
                p.surfaces_refaites,
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
