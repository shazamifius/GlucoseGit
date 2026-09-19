//! Une distribution à mémoire constante, quelle que soit la durée de la session.
//!
//! # Pourquoi une distribution, et jamais une moyenne
//!
//! Cent images à 2 ms et une à 200 ms donnent une moyenne de 4 ms — excellente — alors que
//! l'utilisateur a vu un gel. **Ce qui se ressent est le pire centile**, pas le centre.
//!
//! # Pourquoi ce module existe
//!
//! Le même découpage logarithmique était écrit **deux fois**, à l'identique, dans la chronique
//! et dans la navigation — et la troisième copie allait s'écrire pour le rythme. Deux copies
//! d'une même loi finissent toujours par diverger, et celle-ci allait le faire au moment
//! précis où l'on s'apprêtait à comparer leurs résultats.
//!
//! # Le découpage, et la promesse qu'il tenait à moitié
//!
//! Quatre tranches par doublement : l'erreur relative sur un centile est alors d'au plus
//! `2^(1/4) − 1`, soit 19 %. Ce n'est pas un réglage de confort — c'est le compromis entre la
//! finesse et la mémoire, et il se calcule. Huit tranches donneraient 9 % pour deux fois plus
//! de compteurs.
//!
//! **Encore faut-il que les tranches soient géométriques, et elles ne l'étaient pas.** Les
//! deux copies d'origine découpaient l'octave en quatre parts **égales** : `base`,
//! `base·1,25`, `base·1,5`, `base·1,75`. Le rapport de la première vaut donc 1,25, soit une
//! erreur de **25 %** là où le commentaire en promettait 19 — et le test qui vérifiait cette
//! promesse sur quelques valeurs bien choisies passait quand même.
//!
//! Les bornes sont ici les puissances fractionnaires de deux, ce que la promesse exige :
//!
//! ```text
//!     base · 2^(0/4)   base · 2^(1/4)   base · 2^(2/4)   base · 2^(3/4)   2·base
//! ```
//!
//! Sans aucun flottant sur un chemin parcouru à chaque image : les rapports sont écrits une
//! fois pour toutes en fractions de `2^16`, et trois comparaisons entières suffisent.
//!
//! # La borne exacte, arrondi compris
//!
//! Les compteurs sont entiers, donc une borne se remonte à l'entier supérieur — sans quoi
//! elle ne contiendrait plus la valeur qu'elle classe, et un centile **sous-estimerait** la
//! lenteur. La garantie complète est donc :
//!
//! ```text
//!     borne(v) ≤ v · 2^(1/4) + 1
//! ```
//!
//! L'unité de plus ne compte que sur les toutes petites valeurs — une tranche partant de 1 ne
//! peut pas se découper en quatre, il n'y a pas d'entier entre 1 et 2. Dès la dizaine, elle
//! est noyée : à mille microsecondes, elle vaut un dixième de pour cent.

/// Combien de tranches par doublement de la grandeur mesurée.
const PAR_OCTAVE: usize = 4;

/// Le dénominateur des rapports géométriques, en puissance de deux pour que la division soit
/// un décalage.
const UNITE: u64 = 1 << 16;

/// Les bornes d'une octave, en fractions de [`UNITE`] : `2^(k/4)` pour `k` de 0 à 4.
///
/// Écrites plutôt que calculées, parce qu'elles ne changent pas et qu'une puissance
/// fractionnaire par image coûterait plus que tout ce que ce module mesure. Un test vérifie
/// qu'elles sont bien ces puissances-là : une table fausse rendrait le module muet sur son
/// propre défaut, ce qui vient précisément d'arriver.
const BORNES: [u64; PAR_OCTAVE + 1] = [65_536, 77_936, 92_682, 110_218, 131_072];

/// De 1 à 2^20, soit un peu plus d'un million. En microsecondes, cela fait une seconde ; en
/// pixels, tout un écran 8K. Au-delà, tout tombe dans la dernière tranche.
const OCTAVES: usize = 21;

/// Le nombre de compteurs d'un histogramme.
const TRANCHES: usize = PAR_OCTAVE * OCTAVES;

/// Une distribution de valeurs entières, de taille fixe.
#[derive(Debug, Clone)]
pub struct Histogramme {
    compteurs: [u32; TRANCHES],
    compte: u64,
    pire: u32,
    total: u64,
}

impl Default for Histogramme {
    fn default() -> Self {
        Self::nouveau()
    }
}

impl Histogramme {
    pub const fn nouveau() -> Self {
        Self {
            compteurs: [0; TRANCHES],
            compte: 0,
            pire: 0,
            total: 0,
        }
    }

    /// Note une valeur.
    pub fn ajouter(&mut self, valeur: u32) {
        self.compteurs[tranche(valeur)] += 1;
        self.compte += 1;
        self.pire = self.pire.max(valeur);
        self.total += u64::from(valeur);
    }

    /// Combien de valeurs ont été notées.
    pub fn compte(&self) -> u64 {
        self.compte
    }

    /// La plus grande valeur notée.
    pub fn pire(&self) -> u32 {
        self.pire
    }

    /// La moyenne des valeurs notées, ou zéro s'il n'y en a aucune.
    pub fn moyenne(&self) -> f64 {
        if self.compte == 0 {
            return 0.0;
        }
        self.total as f64 / self.compte as f64
    }

    /// La valeur sous laquelle tombe la part `p` des observations.
    ///
    /// Lue sur l'histogramme, donc juste à 19 % près — ce qui suffit largement pour distinguer
    /// une image de 2 ms d'une image de 80 ms, qui est la question posée.
    ///
    /// # Pourquoi elle ne dépasse jamais le maximum observé
    ///
    /// La borne haute d'une tranche majore ce qu'elle contient. Sans garde, le rapport
    /// affichait donc « p99 77,94 ms, pire 66,83 ms » — un centile plus grand que le maximum,
    /// ce qui n'a pas de sens et fait douter de toute la colonne. Le maximum, lui, est gardé
    /// exact : c'est la seule valeur de ce module qui ne soit pas approchée, et elle sert de
    /// plafond légitime à toutes les autres.
    pub fn centile(&self, p: f64) -> u32 {
        if self.compte == 0 {
            return 0;
        }
        let cible = (self.compte as f64 * p).ceil() as u64;
        let mut cumul = 0u64;
        for (i, n) in self.compteurs.iter().enumerate() {
            cumul += u64::from(*n);
            if cumul >= cible {
                return borne_haute(i).min(self.pire);
            }
        }
        self.pire
    }

    /// Combien d'observations dépassent strictement ce seuil.
    ///
    /// # Pourquoi cela ne pouvait pas se lire sur les pires images
    ///
    /// Le résumé annonçait « au moins 32 images au-dessus du plancher » **quelle que soit la
    /// session** : il comptait les images de la liste des trente-deux plus lentes, qui est
    /// bornée par construction. Une session qui rate mille images et une qui en rate trente-
    /// trois annonçaient le même nombre. Ici la question se pose à la distribution entière,
    /// qui les a toutes vues.
    ///
    /// La tranche qui contient le seuil est comptée en entier : la réponse est donc majorée,
    /// jamais minorée — c'est le sens prudent pour un compte de ratés.
    pub fn au_dessus(&self, seuil: u32) -> u64 {
        let premiere = tranche(seuil);
        self.compteurs
            .iter()
            .skip(premiere)
            .map(|n| u64::from(*n))
            .sum()
    }
}

/// L'indice de tranche d'une valeur.
///
/// `ilog2` donne l'octave ; la position **dans** l'octave se lit en comparant la valeur aux
/// bornes géométriques, ce qui évite un logarithme flottant sur un chemin parcouru à chaque
/// image.
fn tranche(valeur: u32) -> usize {
    if valeur == 0 {
        return 0;
    }
    let octave = valeur.ilog2() as usize;
    let base = 1u64 << octave;
    let echelle = u64::from(valeur) * UNITE;
    let sous = BORNES
        .iter()
        .skip(1)
        .position(|b| echelle < base * b)
        .unwrap_or(PAR_OCTAVE - 1);
    (octave * PAR_OCTAVE + sous).min(TRANCHES - 1)
}

/// La valeur maximale que contient cette tranche.
fn borne_haute(i: usize) -> u32 {
    let octave = i / PAR_OCTAVE;
    let sous = i % PAR_OCTAVE;
    let base = 1u64 << octave;
    // Arrondi vers le haut : la borne doit **contenir** toute valeur rangée dans la tranche,
    // sans quoi un centile sous-estimerait la lenteur — l'exact défaut qu'on veut éviter.
    let borne = (base * BORNES[sous + 1]).div_ceil(UNITE);
    borne.min(u64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_un_histogramme_vide_ne_ment_pas() {
        let h = Histogramme::nouveau();
        assert_eq!(h.compte(), 0);
        assert_eq!(h.centile(0.5), 0);
        assert_eq!(h.au_dessus(10), 0);
        assert_eq!(h.moyenne(), 0.0);
    }

    #[test]
    fn test_le_centile_encadre_la_valeur_a_dix_neuf_pour_cent() {
        let mut h = Histogramme::nouveau();
        for _ in 0..1000 {
            h.ajouter(10_000);
        }
        let median = h.centile(0.50);
        assert!(
            (10_000..=11_900).contains(&median),
            "le median doit encadrer 10 000 par au plus 19 % : {median}"
        );
    }

    /// **Un centile ne voit que ce qui est assez fréquent pour lui**, et c'est pour cela que
    /// le pire se garde à part.
    ///
    /// Ce test disait d'abord « le p99 doit voir le gel » sur une distribution qui n'en
    /// contient qu'un sur cent : le gel est alors au centile **cent**, pas quatre-vingt-dix-
    /// neuf, et l'assertion était fausse. Elle est réécrite pour dire ce qui est vrai, parce
    /// qu'un test qui se trompe de promesse finit par la faire tenir au code.
    #[test]
    fn test_le_pire_se_garde_a_part_parce_qu_aucun_centile_ne_le_voit() {
        let mut h = Histogramme::nouveau();
        for _ in 0..99 {
            h.ajouter(2_000);
        }
        h.ajouter(200_000);
        // La moyenne dirait 4 ms : excellente, et fausse.
        assert!(h.moyenne() < 5_000.0, "la moyenne noie bien le gel");
        assert!(
            h.centile(0.99) < 3_000,
            "un gel sur cent est au centile cent, pas au p99 : {}",
            h.centile(0.99)
        );
        assert_eq!(h.pire(), 200_000, "le pire, lui, le voit toujours");

        // Deux gels sur cent, en revanche, remontent jusqu'au p99.
        let mut plus = Histogramme::nouveau();
        for _ in 0..98 {
            plus.ajouter(2_000);
        }
        plus.ajouter(200_000);
        plus.ajouter(200_000);
        assert!(
            plus.centile(0.99) >= 100_000,
            "deux gels sur cent se voient au p99 : {}",
            plus.centile(0.99)
        );
    }

    /// La table des bornes **est** celle des puissances fractionnaires de deux.
    ///
    /// Sans ce test, une table approximative rendrait le module muet sur son propre défaut —
    /// exactement ce qui est arrivé au découpage linéaire qu'il remplace.
    #[test]
    fn test_les_bornes_sont_les_puissances_quartes_de_deux() {
        for (k, borne) in BORNES.iter().enumerate() {
            let exact = 2f64.powf(k as f64 / PAR_OCTAVE as f64) * UNITE as f64;
            let ecart = (*borne as f64 - exact).abs() / exact;
            assert!(
                ecart < 1e-5,
                "la borne 2^({k}/4) vaut {borne}, on attendait {exact:.0}"
            );
        }
    }

    /// **La promesse du module : 19 %, et sur toute l'étendue.**
    ///
    /// Le découpage linéaire d'origine en donnait 25 sur la première sous-tranche de chaque
    /// octave. Ce test balaie l'étendue entière plutôt que quelques valeurs bien choisies —
    /// c'est ce qui manquait pour que le défaut se voie.
    #[test]
    fn test_l_erreur_d_une_tranche_ne_depasse_jamais_dix_neuf_pour_cent() {
        let mut valeur = 1u32;
        while valeur < 1 << 20 {
            // Chaque valeur de l'etendue, et non quelques-unes bien choisies : c'est ce qui
            // manquait pour que le decoupage lineaire se denonce.
            let i = tranche(valeur);
            let haute = f64::from(borne_haute(i));
            assert!(
                haute >= f64::from(valeur),
                "{valeur} tombe en tranche {i}, bornee a {haute} : elle ne la contient pas"
            );
            // La promesse complete : le rapport geometrique, plus l'unite d'arrondi entier.
            let promise = f64::from(valeur) * 2f64.powf(0.25) + 1.0;
            assert!(
                haute <= promise,
                "{valeur} : borne {haute}, on promettait au plus {promise:.2}"
            );
            valeur = valeur + 1 + valeur / 64;
        }
    }

    /// **Aucun centile ne dépasse le maximum observé.**
    ///
    /// Le rapport affichait « p99 77,94 ms, pire 66,83 ms » : la borne haute d'une tranche
    /// majore ce qu'elle contient, et rien ne la ramenait au réel. Un centile plus grand que
    /// le maximum fait douter de toute la colonne, et à juste titre.
    #[test]
    fn test_aucun_centile_ne_depasse_le_maximum_observe() {
        let mut h = Histogramme::nouveau();
        // 66 830 µs tombe dans une tranche bornée nettement plus haut.
        for _ in 0..10 {
            h.ajouter(66_830);
        }
        for p in [0.50, 0.90, 0.99, 1.0] {
            assert!(
                h.centile(p) <= h.pire(),
                "le centile {p} vaut {} pour un maximum de {}",
                h.centile(p),
                h.pire()
            );
        }
    }

    /// Une valeur au-delà de l'étendue suivie retombe dans la dernière tranche, et le pire
    /// reste exact — c'est ce qui rend la borne de mémoire sans conséquence sur la lecture.
    #[test]
    fn test_au_dela_de_l_etendue_le_pire_reste_exact() {
        let mut h = Histogramme::nouveau();
        h.ajouter(50_000_000);
        assert_eq!(h.pire(), 50_000_000);
        assert!(
            h.centile(0.50) >= 1 << 20,
            "la derniere tranche a recu la valeur"
        );
    }

    #[test]
    fn test_le_compte_au_dessus_voit_toutes_les_images_et_pas_les_trente_deux_pires() {
        let mut h = Histogramme::nouveau();
        // Mille images ratées : le compte doit les voir toutes, pas s'arrêter à la liste
        // bornée des plus lentes.
        for _ in 0..1000 {
            h.ajouter(40_000);
        }
        for _ in 0..500 {
            h.ajouter(1_000);
        }
        let ratees = h.au_dessus(10_000);
        assert!(
            ratees >= 1000,
            "mille images au-dessus du plancher doivent se compter mille : {ratees}"
        );
        assert!(
            ratees < 1100,
            "et pas beaucoup plus : la tranche du seuil est comptee en entier, rien d'autre"
        );
    }

    #[test]
    fn test_les_tranches_sont_croissantes_et_couvrent_tout() {
        let mut precedente = 0;
        for i in 0..TRANCHES {
            let borne = borne_haute(i);
            assert!(borne >= precedente, "la tranche {i} recule");
            precedente = borne;
        }
        // Toute valeur retombe dans une tranche existante.
        for valeur in [0u32, 1, 3, 1_000, 999_999, u32::MAX] {
            assert!(
                tranche(valeur) < TRANCHES,
                "valeur hors tranches : {valeur}"
            );
        }
    }
}
