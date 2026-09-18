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
/// Soixante images par seconde. Ce n'est pas la cible — elle est à 400 — mais c'est le seuil
/// au-dessous duquel la charte considère que quelque chose ne va pas, donc le bon repère pour
/// compter les images ratées.
const BUDGET_PLANCHER_US: u32 = 1_000_000 / 60;

impl Chronique {
    /// Le rapport complet, en texte.
    pub fn rapport(&self) -> String {
        let mut t = String::new();
        self.ecrire_le_resume(&mut t);
        self.ecrire_les_gestes(&mut t);
        self.ecrire_les_pires(&mut t);
        t
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
