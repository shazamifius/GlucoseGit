//! Les sections du rapport qui parlent de ce que l'œil **reçoit**, et non de ce qu'une image
//! coûte : le verdict, la machine, et le rythme.
//!
//! # Pourquoi elles viennent en premier dans le rapport
//!
//! Tout le reste — les gestes, les postes, les images les plus lentes — répond à « où va le
//! temps ». C'est la bonne question une fois qu'on sait laquelle poser. Ces trois sections-ci
//! disent **laquelle poser**, et elles doivent donc être lues avant.

use crate::chronique::entracte::Poste;
use crate::chronique::Chronique;

impl Chronique {
    /// Ce qui cloche, du plus grave au moins, avec son chiffre et sa conséquence.
    pub(super) fn ecrire_le_verdict(&self, t: &mut String) {
        let constats = self.verdict();
        if constats.is_empty() {
            t.push_str("  LE VERDICT -- rien ne depasse sa reference, cette session est saine\n\n");
            return;
        }
        t.push_str("  LE VERDICT -- ce qui cloche, du plus grave au moins\n\n");
        for (rang, c) in constats.iter().enumerate() {
            t.push_str(&format!(
                "  {}. [{}] x{:.1} au-dela de sa reference\n     {}\n     -> {}\n",
                rang + 1,
                c.nature,
                c.gravite,
                c.fait,
                c.consequence
            ));
        }
        t.push('\n');
    }

    /// Les deux faits de la machine sans lesquels aucune durée ne se relit.
    ///
    /// La succession des images, en particulier : les mêmes dix millisecondes ne veulent pas
    /// dire la même chose selon que la présentation attendait le balayage ou non. Ce dépôt a
    /// tourné sans synchronisation verticale pendant toute son histoire sans qu'aucune trace
    /// ne le porte.
    pub(super) fn ecrire_la_machine(&self, t: &mut String) {
        let Some(periode) = self.rythme.periode() else {
            return;
        };
        t.push_str("  La machine, telle qu'elle s'est annoncee\n\n");
        t.push_str(&format!(
            "  ecran : {:.0} Hz, soit une image toutes les {:.2} ms\n",
            1.0 / periode.as_secs_f64(),
            periode.as_secs_f64() * 1000.0
        ));
        t.push_str(&format!(
            "  succession des images : {}\n\n",
            self.rythme.presentation()
        ));
    }

    /// Le rythme : le temps que le mouvement intègre, comparé au temps que l'écran montre.
    ///
    /// # Ce que cette section dit et qu'aucune autre ne peut dire
    ///
    /// Un mouvement parfaitement régulier, rendu par une machine irrégulière, est **vu
    /// saccadé**. Les totaux, les centiles, la cadence moyenne et la latence restent tous
    /// excellents pendant ce temps. C'est la raison pour laquelle quatre tentatives
    /// successives ont cherché la cause dans le modèle du mouvement — qui est exact.
    pub(super) fn ecrire_le_rythme(&self, t: &mut String) {
        if self.rythme.comparees() == 0 {
            return;
        }
        t.push_str("  Le rythme -- ce que l'oeil recoit, et non ce que l'image coute\n\n");
        self.ecrire_les_intervalles(t);
        // Juste apres la ligne « dont a NE PAS dessiner », parce que c'est a elle que cette
        // section repond -- et qu'elle n'a jamais su repondre pendant trois sessions.
        self.ecrire_l_entracte(t);
        self.ecrire_le_tressaut(t);
        self.ecrire_les_balayages(t);
        t.push('\n');
    }

    /// L'intervalle entre deux images à l'écran, et la cadence qu'il donne réellement.
    fn ecrire_les_intervalles(&self, t: &mut String) {
        let (median, p90, p99, pire) = self.rythme.intervalles();
        t.push_str(&format!(
            "  une image reste a l'ecran : median {:.2}ms, p90 {:.2}ms, p99 {:.2}ms, pire {:.2}ms\n",
            ms(median),
            ms(p90),
            ms(p99),
            ms(pire),
        ));
        if let Some(vue) = self.rythme.cadence_vue() {
            t.push_str(&format!(
                "    soit {vue:.0} images par seconde entre deux images CONSECUTIVES -- la \
                 moyenne de la session, elle, compte le temps ou rien n'etait demande\n"
            ));
        }
        // **Un gel se voit pareil, qu'il vienne d'une image lente ou d'une attente.** Et il ne
        // se corrige pas du tout pareil : le premier lancement a montre un intervalle de
        // 706 ms alors que la pire image de la session en coutait seize.
        let (attente_med, attente_p99, attente_pire) = self.rythme.attentes();
        t.push_str(&format!(
            "  dont a NE PAS dessiner : median {:.2}ms, p99 {:.2}ms, pire {:.2}ms\n",
            ms(attente_med),
            ms(attente_p99),
            ms(attente_pire),
        ));
        let (quand, pire_gel, dont_attente) = self.rythme.pire_intervalle();
        if !pire_gel.is_zero() {
            t.push_str(&format!(
                "  le pire gel : {:.1}ms a la {:.1}e seconde, dont {:.1}ms a ne pas dessiner\n",
                pire_gel.as_secs_f64() * 1000.0,
                quand.as_secs_f64(),
                dont_attente.as_secs_f64() * 1000.0,
            ));
        }
    }

    /// De combien le contenu se pose à côté de sa trajectoire, et à quelle échelle.
    fn ecrire_le_tressaut(&self, t: &mut String) {
        let (median, p99, pire) = self.rythme.sauts();
        let avance = self.rythme.avance_mediane();
        t.push_str(&format!(
            "  le contenu se pose a cote de sa trajectoire : median {median} px, p99 {p99} px, \
             pire {pire} px\n"
        ));
        t.push_str(&format!(
            "    pour {avance} px d'avance attendue par image -- un saut plus grand que \
             l'avance fait RECULER le contenu\n"
        ));
        // **D'ou vient chaque saut** (TRESSAUT-1). Un gel en produit deux -- l'image figee,
        // puis celle qui rattrape -- et ils ne se corrigent pas du tout comme un rendu qui
        // change de duree. Les confondre a fait lire un gel comme un defaut de mouvement.
        for (quoi, (p99, pire, combien)) in [
            (
                "le temps a NE PAS dessiner -- les gels",
                self.rythme.sauts_par_l_attente(),
            ),
            (
                "le rendu qui change de duree -- le tressaut",
                self.rythme.sauts_par_le_rendu(),
            ),
        ] {
            if combien > 0 {
                t.push_str(&format!(
                    "    deplace par {quoi} : p99 {p99} px, pire {pire} px, sur {combien} images\n"
                ));
            }
        }
        if let Some((mediane, basse)) = self.rythme.fidelite() {
            t.push_str(&format!(
                "  fidelite temps integre / temps montre : mediane {mediane:.2}, la plus basse \
                 {basse:.2} (1,00 = exact)\n"
            ));
        }
    }

    /// Combien de balayages chaque image occupe — la mesure du judder, sans aucun seuil.
    fn ecrire_les_balayages(&self, t: &mut String) {
        let occupees = self.rythme.periodes_occupees();
        if occupees.is_empty() {
            return;
        }
        let Some(irreguliere) = self.rythme.irregularite() else {
            return;
        };
        t.push_str(&format!(
            "  {:.0} % des images n'occupent pas le meme nombre de balayages que la precedente\n",
            100.0 * irreguliere
        ));
        t.push_str(
            "    un mouvement fluide en occupe un nombre CONSTANT ; changer est la definition \
             du judder\n",
        );
        let total = self.rythme.comparees().max(1);
        for (balayages, images) in occupees.into_iter().take(6) {
            let part = images as f64 / total as f64;
            t.push_str(&format!(
                "      {balayages:>2} balayage(s) {:>6.1}%  {}\n",
                100.0 * part,
                super::barre(part)
            ));
        }
        self.ecrire_le_tempo(t);
    }

    /// Ce que le tempo VISAIT, en face de ce que l'écran a montré.
    ///
    /// Les deux doivent se ressembler. Sinon, le tempo vise juste et la machine ne suit pas —
    /// ou l'inverse — et c'est cet écart qu'il faut aller regarder.
    fn ecrire_le_tempo(&self, t: &mut String) {
        let (vises, attente_med, attente_pire) = self.tempo();
        if vises.is_empty() {
            return;
        }
        let en_mouvement: u64 = vises.iter().map(|(_, n)| n).sum();
        let parts: Vec<String> = vises
            .iter()
            .take(3)
            .map(|(k, n)| {
                format!(
                    "{k} balayage(s) {:.0} %",
                    100.0 * *n as f64 / en_mouvement.max(1) as f64
                )
            })
            .collect();
        t.push_str(&format!(
            "  le tempo, sur {en_mouvement} images en mouvement : {}\n",
            parts.join(", ")
        ));
        t.push_str(&format!(
            "    attente avant de soumettre : median {:.2}ms, pire {:.2}ms -- du temps libre, \
             pas du temps perdu\n",
            f64::from(attente_med) / 1000.0,
            f64::from(attente_pire) / 1000.0
        ));
    }

    /// **Ou va le temps a ne pas dessiner** (ENTRACTE-1).
    ///
    /// # La section qui manquait, et trois sessions l'ont demandee
    ///
    /// La ligne juste au-dessus chiffre cet intervalle depuis la fiche 19, et elle n'a jamais
    /// pu dire ce qu'il contenait : 185 ms au p99, puis 717, puis 748,7 sur un gel de 763.
    /// Deux hypotheses ecrites pour l'expliquer ont ete dementies, dont une par son propre
    /// compteur.
    ///
    /// Les cinq postes se somment a l'entracte entier, et c'est ce qui rend cette section
    /// lisible : un poste ne peut pas absorber ce qui le precede, faute que ce depot a payee
    /// quatre fois -- `occlusion`, `recolte`, `blit`, `minimap`.
    ///
    /// # L'ordre suit le PIRE, et c'est l'inverse du tableau des postes du rendu
    ///
    /// La fiche 24 § 2 a fait passer ce tableau-la de la mediane au p99, parce que le tempo
    /// se cale sur le centile. **Ici la bonne grandeur est encore une autre**, et la reprendre
    /// sans reflechir referait la meme faute a un cran de plus.
    ///
    /// Un poste du rendu se paie a CHAQUE image : son p99 dit ce qui fait geler. Un poste de
    /// l'entracte, lui, peut n'exister qu'une fois dans toute la session -- la bascule de
    /// carte a lieu au plus deux fois dans la vie du processus, et elle coute sept dixiemes
    /// de seconde. Son p99 est donc rigoureusement nul, et trier par lui reléguerait en
    /// dernier le seul poste qu'on cherchait.
    ///
    /// Et un poste rigoureusement nul ne parait pas : une ligne de zeros n'apprend rien et
    /// allonge le tableau d'autant.
    fn ecrire_l_entracte(&self, t: &mut String) {
        if self.entracte.comptes() == 0 {
            return;
        }
        let mut lignes: Vec<(&'static str, u32, u32, u32)> = Poste::TOUS
            .iter()
            .map(|p| {
                let (median, p99, pire) = self.entracte.poste(*p);
                (p.nom(), median, p99, pire)
            })
            .filter(|(_, _, _, pire)| *pire > 0)
            .collect();
        if lignes.is_empty() {
            return;
        }
        lignes.sort_by_key(|(_, _, p99, pire)| std::cmp::Reverse((*pire, *p99)));
        let plafond = lignes
            .iter()
            .map(|(_, _, _, pire)| *pire)
            .max()
            .unwrap_or(1);
        t.push_str(
            "    et voici OU il est alle -- l'application ne dessine pas, mais elle n'est pas \
             inactive\n",
        );
        t.push_str("      poste                median       p99      pire\n");
        for (nom, median, p99, pire) in lignes {
            t.push_str(&format!(
                "      {nom:<18} {:7.2}ms {:7.2}ms {:9.2}ms  {}\n",
                ms(median),
                ms(p99),
                ms(pire),
                super::barre(f64::from(pire) / f64::from(plafond.max(1)))
            ));
        }
        self.ecrire_les_gels(t);
    }

    /// **Chaque gel, decompose** : les seules lignes du rapport qui nomment un gel.
    ///
    /// Une distribution dit ce qui arrive d'ordinaire. Elle ne dit pas ce qu'UNE attente de
    /// sept dixiemes de seconde contenait. La premiere version ne gardait que la pire, et la
    /// premiere session reelle l'a montre insuffisant : c'etait le demarrage, et il cachait
    /// tous les autres (ENTRACTE-2). Chaque gel dit donc quand, combien, et les trois postes
    /// qui y ont pris le plus -- les autres n'ajoutent rien a ce qu'il faut corriger.
    fn ecrire_les_gels(&self, t: &mut String) {
        let (gels, tus) = self.entracte.gels();
        if gels.is_empty() {
            return;
        }
        let somme: f64 = gels.iter().map(|g| g.total.as_secs_f64() * 1000.0).sum();
        t.push_str(&format!(
            "    les attentes qui ont coute au moins une image -- {}, soit {somme:.0} ms :
",
            gels.len()
        ));
        for gel in gels {
            let detail: Vec<String> = gel
                .parts()
                .iter()
                .take(3)
                .map(|(p, d)| format!("{} {:.1}ms", p.nom(), d.as_secs_f64() * 1000.0))
                .collect();
            t.push_str(&format!(
                "      a {:5.1}s  {:7.1}ms -- {}
",
                gel.a.as_secs_f64(),
                gel.total.as_secs_f64() * 1000.0,
                detail.join(", ")
            ));
        }
        if tus > 0 {
            t.push_str(&format!(
                "      et {tus} autre(s), plus courte(s) que toutes celles-ci
"
            ));
        }
    }
}

fn ms(us: u32) -> f64 {
    f64::from(us) / 1000.0
}
