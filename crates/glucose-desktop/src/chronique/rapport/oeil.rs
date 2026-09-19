//! Les sections du rapport qui parlent de ce que l'œil **reçoit**, et non de ce qu'une image
//! coûte : le verdict, la machine, et le rythme.
//!
//! # Pourquoi elles viennent en premier dans le rapport
//!
//! Tout le reste — les gestes, les postes, les images les plus lentes — répond à « où va le
//! temps ». C'est la bonne question une fois qu'on sait laquelle poser. Ces trois sections-ci
//! disent **laquelle poser**, et elles doivent donc être lues avant.

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
    }
}

fn ms(us: u32) -> f64 {
    f64::from(us) / 1000.0
}
