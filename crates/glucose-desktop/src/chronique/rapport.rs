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
mod panneaux;

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

/// Une colonne de la table des images lentes : son en-tete, et de quoi la lire.
///
/// Un alias et non le type ecrit en clair : clippy refuse un `Vec` de couples dont le second
/// membre est un objet-trait, et il a raison -- la signature disait trois fois la meme chose
/// et ne se lisait plus.
type Colonne<'a> = (&'a str, &'a dyn Fn(&Instantane) -> u32);

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
        self.ecrire_les_panneaux(&mut t);
        self.ecrire_l_empreinte(&mut t);
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

    /// **Ce que le processus coûte a la machine**, et surtout quand on ne le touche pas
    /// (EMPREINTE-1).
    ///
    /// # La seule section qui ne parle pas d'une image
    ///
    /// Tout le reste de ce rapport mesure ce qu'une **image** coûte. Un logiciel peut tenir
    /// cent images par seconde et prendre huit cents mébioctets à l'ouverture d'un document
    /// vide, ou occuper un dixième de cœur pendant qu'il dort — ce qui empêche un portable de
    /// se rendormir, et c'est de l'autonomie en moins. Aucune durée d'image ne le dit.
    ///
    /// La ligne qui compte est celle du repos : c'est elle qui répond à *« économe »*.
    fn ecrire_l_empreinte(&self, t: &mut String) {
        if !self.veille.a_mesure() {
            return;
        }
        let (maintenant, pire) = self.veille.memoire();
        t.push_str(
            "  Ce que Glucose coute a la machine, et non ce que ses images coutent

",
        );
        t.push_str(&format!(
            "  memoire de travail : {:.0} Mo a la fin, {:.0} Mo au pire de la session
",
            mo(maintenant),
            mo(pire)
        ));
        if let Some(c) = self.veille.carte() {
            t.push_str(&format!(
                "  memoire graphique : {:.0} Mo a la fin, {:.0} Mo au pire, dont {:.0} Mo gardes hors de l'ecran au plus -- pour un budget accorde de {:.0} Mo ({:.0} Mo au plus bas)
",
                mo(c.utilisee),
                mo(c.utilisee_pire),
                mo(c.cache_pire),
                mo(c.budget),
                mo(c.budget_plus_bas)
            ));
        }
        self.veille.etages.ecrire(t);
        for (quoi, part) in [
            ("pendant qu'on ne le touche pas", self.veille.au_repos()),
            ("pendant qu'on s'en sert", self.veille.a_l_usage()),
        ] {
            let Some(p) = part else {
                continue;
            };
            t.push_str(&format!(
                "  processeur {quoi} : {:.1} % d'un coeur, sur {:.0} s observees
",
                p.coeurs * 100.0,
                p.sur.as_secs_f64()
            ));
        }
        let (cachees, perdues) = self.refusees();
        if cachees + perdues > 0 {
            t.push_str(&format!(
                "  images que la surface a refusees : {perdues} perdue(s), {cachees} cachee(s)
"
            ));
        }
        // **Le nombre qui accuse** : une image rendue alors que personne ne regarde empeche le
        // processeur de descendre dans ses etats de sommeil profond. Zero est la seule bonne
        // reponse, et toute autre valeur designe un reveil a expliquer.
        if let Some(ips) = self.veille.images_sans_la_main() {
            t.push_str(&format!(
                "  et il dessine {ips:.1} image(s) par seconde pendant ce temps -- zero est la seule bonne reponse
"
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
            "  D'ou viennent les images -- la main, une raison de se reveiller, ou le systeme

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
        // Chaque image a au moins une provenance -- le systeme quand rien de connu ne l'a
        // demandee --, donc une liste vide ne veut dire qu'une chose : aucune image.
        if raisons.is_empty() {
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
    /// est relative au plus coûteux **au p99**, pas à un tout, donc personne ne peut la sommer.
    ///
    /// # Pourquoi le p99 et non la médiane
    ///
    /// Parce que c'est le p99 que le tempo suit : le nombre de balayages par image se cale sur
    /// le centile de la distribution, jamais sur son typique. Trier par la médiane rangeait en
    /// tête ce qui ne décide de rien, et **cachait purement et simplement** les postes à pic —
    /// voir [`postes_a_montrer`], qui dit le défaut en entier.
    ///
    /// Le pire est affiché à côté du typique : c'est lui qui désigne un gel, et la section des
    /// images les plus lentes le décompose.
    fn ecrire_la_repartition(&self, t: &mut String) {
        t.push_str(
            "  Ou va le temps de chaque geste -- au centile, et TRIE PAR LE P99\n    \
             c'est lui que le tempo suit : un poste peut etre nul en median et geler l'image\n\n",
        );
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
            let retenus = postes_a_montrer(parts);
            let tete = retenus.first().map_or(1, |(_, h)| h.centile(0.99).max(1));
            for (nom, h) in retenus {
                t.push_str(&format!(
                    "      {:<14} {:>7.2}ms {:>7.2}ms {:>9.2}ms  {}\n",
                    nom,
                    ms(h.centile(0.5)),
                    ms(h.centile(0.99)),
                    ms(h.pire()),
                    barre(f64::from(h.centile(0.99)) / f64::from(tete))
                ));
            }
            t.push('\n');
        }
    }

    /// Les images les plus lentes, avec tout leur contexte : c'est là que se lit la cause.
    ///
    /// # Pourquoi les colonnes vont et viennent
    ///
    /// Elles étaient vingt-deux, toujours les mêmes, et **la moitié valait zéro sur toute une
    /// session**. Sur la voie graphique, `vign`, `file`, `perim`, `pret`, `orph`, `abdn`,
    /// `tuiles` et `reprises` comptent des mécanismes de la voie processeur qui ne s'exécutent
    /// pas : huit colonnes de zéros occupant la moitié de la largeur, pendant que `text` et
    /// `kpx` — celles qui désignaient la cause — se lisaient à l'autre bout de la ligne.
    ///
    /// Une colonne dont **aucune** des images retenues ne porte de valeur ne dit rien, et
    /// l'espace qu'elle prend est pris à celles qui disent quelque chose. Aucun seuil n'a eu
    /// à être choisi : zéro partout, ou bien elle reste.
    fn ecrire_les_pires(&self, t: &mut String) {
        let pires = self.pires();
        if pires.is_empty() {
            return;
        }
        let colonnes: Vec<Colonne<'_>> = vec![
            ("noeuds", &|p: &Instantane| p.noeuds),
            ("photos", &|p: &Instantane| p.photos),
            // En centièmes d'écran, comme la structure la garde : c'est ce chiffre qui a
            // désigné REPORT-1 en annonçant 14 930 pour soixante-seize photos empilées.
            ("ecran%", &|p: &Instantane| p.surcouverture),
            ("vign/px", &|p: &Instantane| u32::from(p.par_vignette)),
            ("vign", &|p: &Instantane| u32::from(p.vignettes)),
            ("file", &|p: &Instantane| u32::from(p.vignettes_en_attente)),
            ("perim", &|p: &Instantane| u32::from(p.vignettes_perimees)),
            ("pret", &|p: &Instantane| u32::from(p.vignettes_pretes)),
            ("orph", &|p: &Instantane| u32::from(p.vignettes_orphelines)),
            ("abdn", &|p: &Instantane| u32::from(p.vignettes_abandonnees)),
            ("tuiles", &|p: &Instantane| u32::from(p.tuiles_peintes)),
            ("repris", &|p: &Instantane| u32::from(p.tuiles_reprises)),
            ("text", &|p: &Instantane| u32::from(p.textures_faites)),
            ("kpx", &|p: &Instantane| u32::from(p.textures_kpx)),
            // En microsecondes : la part du poste `textures` passée à RENDRE, le reste étant
            // l'envoi à la carte.
            ("rendu_us", &|p: &Instantane| p.textures_rendu_us),
            ("report", &|p: &Instantane| u32::from(p.textures_reportees)),
            ("dock", &|p: &Instantane| u32::from(p.dock_rendus)),
            ("bande", &|p: &Instantane| u32::from(p.bande_refaite)),
            ("direct", &|p: &Instantane| u32::from(p.cartes_entieres)),
            ("surf", &|p: &Instantane| u32::from(p.surfaces_refaites)),
            ("envoiMo", &|p: &Instantane| u32::from(p.blit_mo)),
            ("cacheMo", &|p: &Instantane| p.images_mo),
        ];
        let retenues: Vec<&Colonne<'_>> = colonnes
            .iter()
            .filter(|(_, lire)| pires.iter().take(12).any(|p| lire(p) > 0))
            .collect();
        t.push_str(
            "  Les images les plus lentes, et ce qu'elles faisaient

",
        );
        t.push_str(&format!(
            "  {:>7} {:>8} {:<18} {:>9}",
            "a", "duree", "geste", "redessine"
        ));
        for (nom, _) in &retenues {
            t.push_str(&format!(" {nom:>7}"));
        }
        t.push('\n');
        for p in pires.iter().take(12) {
            t.push_str(&format!(
                "  {:>6.1}s {:>6.2}ms {:<18} {:>8.0}%",
                f64::from(p.instant_ms) / 1000.0,
                f64::from(p.duree_us) / 1000.0,
                nom_du_geste(p),
                100.0 * p.part_redessinee(),
            ));
            for (_, lire) in &retenues {
                t.push_str(&format!(" {:>7}", lire(p)));
            }
            t.push('\n');
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

/// **Les postes qu'un geste doit montrer** : les six plus gros en median, et les six plus
/// gros au p99 -- leur union, triee par le p99.
///
/// # Le defaut que cette fonction repare, et il rendait le rapport aveugle
///
/// Le tableau triait par **mediane** et coupait a six. Un poste dont la mediane est nulle et
/// le p99 vaut vingt millisecondes -- exactement celui qu'on cherche quand on demande
/// pourquoi le tempo ne descend pas -- n'y paraissait jamais. La chronique du 22/09 en donne
/// la preuve : `textures` est absent des trois gestes, et les douze images les plus lentes de
/// la session sont toutes dominees par lui, jusqu'a 24,88 ms.
///
/// C'est la faute de la fiche 20 § 4.5 **prise a l'envers**, et elle avait deja ete corrigee
/// une fois : la fiche 23 § 6 raconte le meme filtre jete de `bench_texte` la veille. Repare
/// dans le banc, laisse dans l'instrument principal -- celui qui tourne chez l'utilisateur.
///
/// # Pourquoi l'union, et pourquoi le p99 decide de l'ordre
///
/// Les deux lectures servent et ne se remplacent pas : la mediane dit ce qu'une image coute
/// **d'ordinaire**, le p99 dit ce qui la fait **geler**. Prendre l'union ne demande aucun
/// seuil de duree -- un seuil serait une constante arbitraire, et la charte les refuse quand
/// elles peuvent disparaitre.
///
/// L'ordre, lui, suit le p99, parce que c'est **lui que le tempo suit** : le nombre de
/// balayages par image se cale sur le centile de la distribution, jamais sur son typique. Un
/// rapport qui trie par la mediane range donc en tete ce qui ne decide de rien.
fn postes_a_montrer<'a>(
    parts: Vec<(&'static str, &'a super::Histogramme)>,
) -> Vec<(&'static str, &'a super::Histogramme)> {
    const COMBIEN: usize = 6;
    let mut par_median = parts.clone();
    par_median.sort_by_key(|(_, h)| std::cmp::Reverse(h.centile(0.5)));
    let mut retenus: Vec<(&'static str, &'a super::Histogramme)> =
        par_median.into_iter().take(COMBIEN).collect();
    let mut par_p99 = parts;
    par_p99.sort_by_key(|(_, h)| std::cmp::Reverse(h.centile(0.99)));
    for (nom, h) in par_p99.into_iter().take(COMBIEN) {
        if !retenus.iter().any(|(deja, _)| *deja == nom) {
            retenus.push((nom, h));
        }
    }
    retenus.sort_by_key(|(_, h)| std::cmp::Reverse(h.centile(0.99)));
    retenus
}

/// Des octets, en mébioctets — l'unité dans laquelle un utilisateur lit une mémoire.
fn mo(octets: u64) -> f64 {
    octets as f64 / (1024.0 * 1024.0)
}
