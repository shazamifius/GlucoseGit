//! Le rythme : le temps que l'écran **montre**, comparé au temps que le mouvement **intègre**.
//!
//! # Le défaut que toute la chronique était incapable de voir
//!
//! L'utilisateur le décrit depuis des semaines, dans les mêmes termes : « quand on freine
//! progressivement, on voit tout en genre quatre images par seconde », alors que la chronique
//! en affiche cent. Quatre tentatives ont cherché la cause dans le modèle du mouvement, et
//! toutes ont échoué — parce que ce modèle est exact.
//!
//! Ce qui ne l'est pas, c'est son **horloge**.
//!
//! # La démonstration, et elle tient en trois lignes
//!
//! La caméra avance de `v · pas`, où `pas` est l'intervalle entre deux **débuts de rendu**.
//! L'image obtenue reste à l'écran pendant `intervalle`, qui est l'intervalle entre deux
//! **présentations**. Or ces deux durées ne sont pas la même :
//!
//! ```text
//!     intervalle = pas + (durée du rendu − durée du rendu précédent)
//! ```
//!
//! La vitesse **apparente** — celle que l'œil mesure, en pixels par seconde d'affichage —
//! vaut donc `v · pas / intervalle`. Elle est constante si et seulement si la durée du rendu
//! l'est. Elle ne l'est pas : la chronique de terrain donne 6 ms en médiane et 67 ms au pire,
//! pour un même geste.
//!
//! **Un mouvement parfaitement régulier, rendu par une machine irrégulière, est vu saccadé.**
//! Aucune mesure de coût ne peut le dire : les totaux, les centiles, la cadence et la latence
//! sont tous excellents pendant que le contenu tressaute.
//!
//! # Ce que ce module mesure, et pourquoi c'est directement lisible
//!
//! Trois grandeurs, et aucune n'a de constante choisie.
//!
//! * **la fidélité** `pas / intervalle` — sans unité. Un vaut « le temps montré est le temps
//!   intégré ». Un tiers veut dire que le contenu n'a avancé que du tiers de ce que cette
//!   durée d'affichage demandait : il paraît figé. Trois veut dire qu'il a sauté ;
//! * **le saut, en pixels** — `vitesse × |intervalle − pas|`. C'est l'écart entre où le
//!   contenu est montré et où il devrait l'être. Comparé à l'avance attendue pendant la même
//!   image, il dit si le tressaut est petit devant le mouvement ou du même ordre ;
//! * **les périodes d'écran par image** — combien de balayages chaque image occupe. Un
//!   mouvement fluide en occupe un nombre **constant** ; c'est la définition même du judder
//!   que d'en changer, et elle ne dépend d'aucun seuil.
//!
//! # Pourquoi l'intervalle se mesure entre présentations, et jamais autrement
//!
//! C'est le seul instant de la boucle qui corresponde à quelque chose que l'œil reçoive. Le
//! début du rendu, la fin du rendu, le réveil de la boucle sont des faits internes : deux
//! d'entre eux peuvent varier du simple au décuple sans que l'écran change de rythme, et
//! inversement.

use super::histogramme::Histogramme;
use std::time::{Duration, Instant};

/// Le nombre de périodes d'écran qu'une image peut occuper avant d'être comptée « au-delà ».
///
/// Trente-deux périodes valent 133 ms sur un écran à 240 Hz, et une demi-seconde à 60 Hz :
/// bien au-delà de tout ce qui se distingue encore d'un gel. La borne existe pour que le
/// tableau ait une largeur, pas pour trancher quoi que ce soit.
const PERIODES_SUIVIES: usize = 32;

/// La fidélité d'une image, en millièmes — mille vaut « exacte ».
///
/// En millièmes parce que l'histogramme compte des entiers, et que trois chiffres suffisent
/// largement : une fidélité de 0,997 et une de 0,998 ne se distinguent pas à l'œil.
const FIDELITE_EXACTE: u32 = 1_000;

/// Ce qu'une image a montré du mouvement, par opposition à ce qu'elle a coûté.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Image {
    /// L'intervalle depuis la présentation précédente : la durée pendant laquelle l'image
    /// **précédente** est restée sous les yeux.
    pub intervalle: Duration,
    /// Le pas de temps avec lequel le mouvement a été intégré pour cette image.
    pub pas: Duration,
    /// La vitesse apparente de la vue, en pixels physiques par seconde.
    pub vitesse_px_s: f64,
}

impl Image {
    /// Le rapport entre le temps intégré et le temps montré, en millièmes.
    ///
    /// Mille est l'exactitude. Au-dessous, le contenu a moins avancé que sa durée d'affichage
    /// ne le demandait — il traîne. Au-dessus, il a sauté.
    fn fidelite_millieme(&self) -> Option<u32> {
        let intervalle = self.intervalle.as_secs_f64();
        if intervalle <= 0.0 {
            return None;
        }
        let rapport = self.pas.as_secs_f64() / intervalle * f64::from(FIDELITE_EXACTE);
        Some(rapport.clamp(0.0, f64::from(u32::MAX)) as u32)
    }

    /// De combien de pixels le contenu est montré à côté de là où il devrait être.
    ///
    /// C'est la grandeur que l'œil voit vraiment : un écart de temps ne se perçoit pas, un
    /// contenu qui se pose deux centimètres trop loin, si.
    fn saut_px(&self) -> f64 {
        let ecart = self.intervalle.as_secs_f64() - self.pas.as_secs_f64();
        self.vitesse_px_s * ecart.abs()
    }

    /// De combien de pixels le contenu devait avancer pendant cette image.
    ///
    /// Sert d'échelle au précédent : sauter de trois pixels quand on en parcourt cent ne se
    /// voit pas ; sauter de trente quand on en parcourt dix est exactement ce dont
    /// l'utilisateur parle.
    fn avance_px(&self) -> f64 {
        self.vitesse_px_s * self.pas.as_secs_f64()
    }
}

/// Ce que la session a montré, image après image.
#[derive(Debug)]
pub struct Rythme {
    /// La période de l'écran, telle qu'il l'annonce. Zéro tant qu'on ne l'a pas lue.
    periode: Duration,
    /// Comment les images se succèdent devant la carte graphique, tel qu'elle l'a accepté.
    presentation: &'static str,
    /// L'instant de la présentation précédente.
    precedente: Option<Instant>,
    /// L'instant de la toute première présentation, pour dater les gels.
    origine: Option<Instant>,
    /// Combien de périodes d'écran l'image précédente a occupées.
    periodes_precedentes: Option<usize>,
    /// L'intervalle entre deux présentations, en microsecondes.
    intervalles: Histogramme,
    /// L'écart entre le temps montré et le temps intégré, en pixels.
    sauts_px: Histogramme,
    /// Ce que le contenu devait parcourir, en pixels, sur les mêmes images.
    avances_px: Histogramme,
    /// La fidélité, en millièmes, sur les seules images où la vue bougeait.
    fidelites: Histogramme,
    /// **Le saut, rangé selon ce qui a fait bouger l'intervalle** (TRESSAUT-1).
    ///
    /// # Le défaut que cette séparation ferme
    ///
    /// Depuis la fiche 19, le pas d'une image vaut l'intervalle de la précédente. Un gel
    /// produit donc **deux** sauts géants : l'image figée, dont le contenu s'arrête, puis
    /// l'image de rattrapage, dont le contenu bondit. Dès qu'un pour cent des intervalles gèle,
    /// le p99 du saut tombe forcément dans les gels — et le verdict, qui juge le tressaut sur ce
    /// p99, annonçait un défaut de mouvement là où il n'y avait qu'un gel compté une deuxième
    /// fois. Sur la session du 22/09 au soir : « tressaut x85 », premier du verdict, pour un
    /// pire gel de 763 ms dont 748 à ne pas dessiner.
    ///
    /// Aucun seuil ne sépare les deux : chaque intervalle se décompose en ce que l'application
    /// a passé à **ne pas dessiner** et ce qu'elle a passé à **dessiner**, et c'est la part qui
    /// a le plus changé depuis l'image précédente qui a déplacé le contenu. Deux mesures qu'on
    /// compare, pas une constante qu'on choisit.
    sauts_par_l_attente: Histogramme,
    sauts_par_le_rendu: Histogramme,
    /// Ce que l'image précédente a passé à ne pas dessiner, puis à dessiner.
    parts_precedentes: Option<(Duration, Duration)>,
    /// **Le temps perdu à ne pas dessiner alors qu'une image était attendue**, au-delà du
    /// plancher de la charte.
    ///
    /// C'est la grandeur qu'ARBITRE-2 a dû apprendre pour `present` : un gel de 480 ms n'est
    /// pas une image ratée, c'est quarante-sept images perdues. La même loi vaut ici, pour la
    /// même raison, avec le même plancher.
    perdu_a_ne_pas_dessiner: Duration,
    /// Ce que l'application a passé à **ne pas dessiner**, entre deux images.
    ///
    /// # La question que la première version ne pouvait pas poser
    ///
    /// Un premier lancement a montré un intervalle de 706 ms alors que la pire image de la
    /// session en coûtait 16. Les sept dixièmes de seconde manquants n'étaient dans aucune
    /// durée de rendu : ils étaient **entre** deux images, là où rien ne mesurait rien.
    ///
    /// Un gel se voit exactement pareil, qu'il vienne d'une image lente ou d'une attente
    /// interminable — mais il ne se corrige pas du tout pareil.
    attentes: Histogramme,
    /// À quelle seconde de la session le pire intervalle est tombé.
    ///
    /// Un gel dans les trois premières secondes est une initialisation ; le même gel à la
    /// trentième est autre chose. La distribution ne dit pas lequel, et le savoir change ce
    /// qu'il faut aller regarder.
    pire_a_ms: u32,
    /// La durée du pire intervalle, pour savoir à quoi `pire_a_ms` se rapporte.
    pire_intervalle: Duration,
    /// Ce que l'image du pire intervalle a passé à ne pas dessiner.
    pire_attente: Duration,
    /// Combien d'images ont occupé `i` périodes d'écran.
    periodes: [u64; PERIODES_SUIVIES + 1],
    /// Combien d'images ont occupé un nombre de périodes **différent** de la précédente.
    changements: u64,
    /// Combien d'images ont pu être comparées à la précédente.
    comparees: u64,
    /// Combien d'images montraient un mouvement, seules à porter une fidélité.
    en_mouvement: u64,
}

impl Default for Rythme {
    fn default() -> Self {
        Self::nouveau()
    }
}

impl Rythme {
    pub fn nouveau() -> Self {
        Self {
            periode: Duration::ZERO,
            presentation: "inconnue",
            precedente: None,
            origine: None,
            periodes_precedentes: None,
            intervalles: Histogramme::nouveau(),
            sauts_px: Histogramme::nouveau(),
            avances_px: Histogramme::nouveau(),
            fidelites: Histogramme::nouveau(),
            sauts_par_l_attente: Histogramme::nouveau(),
            sauts_par_le_rendu: Histogramme::nouveau(),
            parts_precedentes: None,
            perdu_a_ne_pas_dessiner: Duration::ZERO,
            attentes: Histogramme::nouveau(),
            pire_a_ms: 0,
            pire_intervalle: Duration::ZERO,
            pire_attente: Duration::ZERO,
            periodes: [0; PERIODES_SUIVIES + 1],
            changements: 0,
            comparees: 0,
            en_mouvement: 0,
        }
    }

    /// Ce que l'écran a annoncé de lui-même, et ce que la carte graphique a accepté.
    ///
    /// Les deux sont des **faits de la machine**, pas des choix : les écrire dans le rapport
    /// est la seule façon de ne pas relire une chronique en supposant l'un ou l'autre. Le mode
    /// de présentation, en particulier, décide si l'image attend le balayage — et il a été
    /// pris pour `Fifo` pendant toute l'histoire de ce dépôt alors qu'il valait `Immediate`.
    pub fn observer_la_machine(&mut self, periode: Duration, presentation: &'static str) {
        self.periode = periode;
        self.presentation = presentation;
    }

    /// La période de l'écran, ou `None` si elle n'a pas été lue.
    pub fn periode(&self) -> Option<Duration> {
        (!self.periode.is_zero()).then_some(self.periode)
    }

    /// Comment les images se succèdent.
    pub fn presentation(&self) -> &'static str {
        self.presentation
    }

    /// L'image vient d'être présentée : note ce qu'elle a montré.
    ///
    /// `debut_du_rendu` sépare l'intervalle en deux — ce que l'application a passé à **ne pas
    /// dessiner**, puis ce qu'elle a passé à dessiner. Un gel se voit exactement pareil dans
    /// les deux cas, et ne se corrige pas du tout pareil.
    ///
    /// Rend ce qui vient d'être mesuré, pour que l'instantané de cette image le porte.
    /// `attendue` dit si l'image precedente avait **demande** celle-ci -- une animation, un
    /// geste, un decodage en cours. Sinon l'application dormait, et l'intervalle est du
    /// repos : le mesurer comme un gel a fait paraitre « 3 092 ms a la 7,7e seconde » sur une
    /// session ou personne ne touchait a rien.
    ///
    /// `due` est l'**echeance** de cette image -- l'instant ou elle est devenue necessaire
    /// (GEL-1). Ce que l'oeil voit reste a l'ecran depuis la presentation precedente, mais
    /// il n'attend rien tant qu'aucune image n'est due : un toast qui dort jusqu'a son fondu
    /// demande bien l'image de son reveil, et les 1,8 s de sommeil, pendant lesquelles l'ecran
    /// etait juste, se lisaient « pire 1 874 ms » dans la ligne « une image reste a
    /// l'ecran ». L'histogramme compte donc depuis la plus tardive des deux. Pendant un
    /// mouvement, l'image suivante est due des la precedente -- le tempo attend APRES le
    /// rendu -- et rien n'y change.
    pub fn presentee(
        &mut self,
        maintenant: Instant,
        debut_du_rendu: Instant,
        pas: Duration,
        vitesse_px_s: f64,
        (attendue, due): (bool, Option<Instant>),
    ) -> Mesure {
        let precedente = self.precedente.replace(maintenant);
        let Some(avant) = precedente else {
            self.origine.get_or_insert(maintenant);
            return Mesure::default();
        };
        if !attendue {
            self.periodes_precedentes = None;
            self.parts_precedentes = None;
            return Mesure::default();
        }
        let image = Image {
            intervalle: maintenant.saturating_duration_since(avant),
            pas,
            vitesse_px_s,
        };
        let attente = debut_du_rendu.saturating_duration_since(avant);
        let parts = (
            attente,
            maintenant.saturating_duration_since(debut_du_rendu),
        );
        let precedentes = self.parts_precedentes.replace(parts);
        let depart = due.map_or(avant, |due| due.max(avant));
        self.intervalles
            .ajouter(micros(maintenant.saturating_duration_since(depart)).unwrap_or(u32::MAX));
        // **Les balayages ne se comparent que quand la vue bouge.** Le judder est la
        // definition d'un MOUVEMENT irregulier ; un toast qui s'estompe a un rythme
        // quelconque n'en est pas un, et le compter faisait lire « 45 % d'images
        // irregulieres » sur une session ou rien ne bougeait.
        if vitesse_px_s > 0.0 {
            self.noter_les_periodes(image.intervalle);
        } else {
            self.periodes_precedentes = None;
        }
        // **Les images immobiles sont écartées de la fidélité, et il le faut.** Une vue qui ne
        // bouge pas a un pas de temps sans signification pour l'œil : la compter ferait
        // paraître régulier un rythme qui ne montre rien.
        if vitesse_px_s <= 0.0 || pas.is_zero() {
            return Mesure {
                intervalle_us: micros(image.intervalle).unwrap_or(0),
                ..Mesure::default()
            };
        }
        self.en_mouvement += 1;
        let saut = image.saut_px();
        let avance = image.avance_px();
        self.sauts_px.ajouter(entier(saut));
        self.attribuer_le_saut(entier(saut), parts, precedentes);
        self.avances_px.ajouter(entier(avance));
        let fidelite = image.fidelite_millieme();
        if let Some(f) = fidelite {
            self.fidelites.ajouter(f);
        }
        Mesure {
            intervalle_us: micros(image.intervalle).unwrap_or(0),
            saut_px: entier(saut).min(u32::from(u16::MAX)) as u16,
            fidelite_millieme: fidelite.unwrap_or(0).min(u32::from(u16::MAX)) as u16,
        }
    }

    /// La boucle a été tenue hors de tout rendu — un dialogue natif, le plus souvent.
    ///
    /// L'utilisateur regardait le dialogue, pas le canevas : l'intervalle qui suit n'est ni
    /// un gel ni un mouvement. Sans cela, une ouverture de fichier se lisait « le pire gel :
    /// 19 836 ms à la 21,2e seconde, dont 19 775 ms à ne pas dessiner » -- vrai, et sans
    /// aucun intérêt, pendant qu'il cachait le vrai pire gel de la session.
    pub fn oublier(&mut self) {
        self.precedente = None;
        self.periodes_precedentes = None;
        self.parts_precedentes = None;
    }

    /// **Range ce saut du côté de ce qui a fait bouger l'intervalle** (TRESSAUT-1).
    ///
    /// L'intervalle d'une image est la somme de ce qu'elle a passé à ne pas dessiner et de ce
    /// qu'elle a passé à dessiner ; son saut vient de ce que cet intervalle diffère du
    /// précédent. La part qui a le plus changé est celle qui a déplacé le contenu.
    ///
    /// Une image sans précédente connue — la première après un repos ou un dialogue — n'est
    /// rangée nulle part : on ne sait pas ce qui a bougé, et le deviner serait choisir.
    fn attribuer_le_saut(
        &mut self,
        saut: u32,
        (attente, rendu): (Duration, Duration),
        precedentes: Option<(Duration, Duration)>,
    ) {
        let Some((attente_avant, rendu_avant)) = precedentes else {
            return;
        };
        let par_l_attente = attente.abs_diff(attente_avant) > rendu.abs_diff(rendu_avant);
        if par_l_attente {
            self.sauts_par_l_attente.ajouter(saut);
        } else {
            self.sauts_par_le_rendu.ajouter(saut);
        }
    }

    /// Le saut des images dont l'intervalle a bougé parce que l'application **ne dessinait
    /// pas** : `(p99, pire, combien)`.
    pub fn sauts_par_l_attente(&self) -> (u32, u32, u64) {
        let h = &self.sauts_par_l_attente;
        (h.centile(0.99), h.pire(), h.compte())
    }

    /// Le saut des images dont l'intervalle a bougé parce que le **rendu** a changé de durée :
    /// `(p99, pire, combien)`. C'est le seul tressaut que le mouvement et le tempo commandent.
    pub fn sauts_par_le_rendu(&self) -> (u32, u32, u64) {
        let h = &self.sauts_par_le_rendu;
        (h.centile(0.99), h.pire(), h.compte())
    }

    /// Le temps perdu à ne pas dessiner alors qu'une image était attendue, au-delà du
    /// plancher de la charte, et sur combien d'intervalles il se compte.
    pub fn perdu_a_ne_pas_dessiner(&self) -> (Duration, u64) {
        (self.perdu_a_ne_pas_dessiner, self.attentes.compte())
    }

    /// **Le retard de cette image sur l'instant où elle était due** (GEL-1).
    ///
    /// # Le défaut que cette mesure ferme, et il a mis un faux gel en tête du verdict
    ///
    /// Le gel se mesurait depuis la **présentation précédente**, dès que l'image était
    /// « attendue ». Or « attendue » se décidait au dernier moment : l'utilisateur quitte
    /// Glucose pour son navigateur, y reste onze secondes, glisse une image — et le dépôt, en
    /// lançant un décodage, rend l'image suivante « attendue ». Les onze secondes passées
    /// dans le navigateur devenaient un gel de onze secondes, « attendre Windows », premier
    /// du verdict à x310. C'est la dette de la fiche 24 § 7 — *un sommeil n'est pas un gel* —,
    /// et le constat du gel l'avait portée en tête.
    ///
    /// Un gel est le temps entre l'instant où une image est **devenue nécessaire** — un
    /// geste, un dépôt, une animation qui demande la suivante, un réveil programmé — et
    /// l'instant où elle paraît. Aucune constante : l'échéance est un fait que l'application
    /// connaît, puisque c'est elle qui la pose.
    ///
    /// Et c'est ce qui rend « attendre Windows » décisif : une attente chez Windows **alors
    /// qu'une image était due** veut dire que Windows tenait le fil de Glucose, et non que
    /// personne ne demandait rien.
    ///
    /// À appeler **avant** [`Self::presentee`], qui oublie la présentation précédente. Après
    /// un dialogue natif, il n'y a pas de précédente : ce qui suit n'est pas un gel.
    pub fn en_retard(
        &mut self,
        maintenant: Instant,
        debut_du_rendu: Instant,
        due: Option<Instant>,
    ) {
        let (Some(avant), Some(due)) = (self.precedente, due) else {
            return;
        };
        let depart = due.max(avant);
        let attente = debut_du_rendu.saturating_duration_since(depart);
        self.perdu_a_ne_pas_dessiner += attente.saturating_sub(crate::cadence::BUDGET_TOTAL);
        self.attentes.ajouter(micros(attente).unwrap_or(u32::MAX));
        self.noter_le_pire(
            maintenant,
            maintenant.saturating_duration_since(depart),
            attente,
        );
    }

    /// Retient le pire intervalle, quand il est tombé, et ce qui l'a composé.
    fn noter_le_pire(&mut self, maintenant: Instant, intervalle: Duration, attente: Duration) {
        if intervalle <= self.pire_intervalle {
            return;
        }
        self.pire_intervalle = intervalle;
        self.pire_attente = attente;
        self.pire_a_ms = self
            .origine
            .map(|o| maintenant.saturating_duration_since(o).as_millis())
            .unwrap_or(0)
            .min(u128::from(u32::MAX)) as u32;
    }

    /// Le pire intervalle : `(quand, sa duree, ce qu'il a passe a ne pas dessiner)`.
    ///
    /// Les trois ensemble, parce qu'aucun ne se lit seul : un gel de sept dixiemes de seconde
    /// a la deuxieme seconde d'une session, dont sept dixiemes passes hors du rendu, ne
    /// designe pas le meme coupable que le meme gel a la trentieme, passe a dessiner.
    pub fn pire_intervalle(&self) -> (Duration, Duration, Duration) {
        (
            Duration::from_millis(u64::from(self.pire_a_ms)),
            self.pire_intervalle,
            self.pire_attente,
        )
    }

    /// Ce que l'application passe a NE PAS dessiner, entre deux images : `(median, p99, pire)`.
    pub fn attentes(&self) -> (u32, u32, u32) {
        (
            self.attentes.centile(0.50),
            self.attentes.centile(0.99),
            self.attentes.pire(),
        )
    }

    /// Range cette image dans le compte des périodes d'écran, et note si elle a changé.
    fn noter_les_periodes(&mut self, intervalle: Duration) {
        let Some(periode) = self.periode() else {
            return;
        };
        // Arrondi au plus proche : une image qui dure 1,9 période en a bien occupé deux, et la
        // tronquer ferait paraître régulier un rythme qui alterne entre une et deux.
        let brut = (intervalle.as_secs_f64() / periode.as_secs_f64()).round();
        let combien = (brut.clamp(0.0, PERIODES_SUIVIES as f64) as usize).min(PERIODES_SUIVIES);
        self.periodes[combien] += 1;
        if let Some(avant) = self.periodes_precedentes.replace(combien) {
            self.comparees += 1;
            if avant != combien {
                self.changements += 1;
            }
        }
    }

    /// L'intervalle entre deux images à l'écran : `(médian, p90, p99, pire)` en microsecondes.
    pub fn intervalles(&self) -> (u32, u32, u32, u32) {
        (
            self.intervalles.centile(0.50),
            self.intervalles.centile(0.90),
            self.intervalles.centile(0.99),
            self.intervalles.pire(),
        )
    }

    /// La cadence réellement vue, en images par seconde — l'inverse de l'intervalle médian.
    ///
    /// # Pourquoi ce n'est pas « images ÷ durée »
    ///
    /// La moyenne de la session compte les instants où personne ne demandait rien : une
    /// application qui dort dix secondes puis rend cent images en une seconde annonce neuf
    /// images par seconde, et aucune de ces neuf n'a existé. La cadence qui se ressent est
    /// celle des images **consécutives**.
    pub fn cadence_vue(&self) -> Option<f64> {
        let median = self.intervalles.centile(0.50);
        (median > 0).then(|| 1_000_000.0 / f64::from(median))
    }

    /// Le saut de position : `(médian, p99, pire)` en pixels.
    pub fn sauts(&self) -> (u32, u32, u32) {
        (
            self.sauts_px.centile(0.50),
            self.sauts_px.centile(0.99),
            self.sauts_px.pire(),
        )
    }

    /// Ce que le contenu devait parcourir par image, en pixels — l'échelle du saut.
    pub fn avance_mediane(&self) -> u32 {
        self.avances_px.centile(0.50)
    }

    /// La fidélité `temps intégré / temps montré` : `(médiane, la plus basse observée)`.
    ///
    /// La plus basse est celle qui se voit : c'est l'image où le contenu a le plus traîné.
    pub fn fidelite(&self) -> Option<(f64, f64)> {
        (self.en_mouvement > 0).then(|| {
            (
                f64::from(self.fidelites.centile(0.50)) / f64::from(FIDELITE_EXACTE),
                f64::from(self.fidelites.centile(0.01)) / f64::from(FIDELITE_EXACTE),
            )
        })
    }

    /// La part des images qui n'ont pas occupé le même nombre de balayages que la précédente.
    ///
    /// **C'est la mesure du judder, et elle n'a pas de seuil.** Un mouvement rendu à cadence
    /// parfaitement régulière donne zéro, quelle que soit cette cadence — y compris une image
    /// sur trois. Ce qui se voit n'est pas la lenteur, c'est l'irrégularité.
    pub fn irregularite(&self) -> Option<f64> {
        (self.comparees > 0).then(|| self.changements as f64 / self.comparees as f64)
    }

    /// Combien d'images ont occupé chaque nombre de balayages, du plus fréquent au moins.
    ///
    /// Ne rend que ce qui a été observé : une ligne à zéro n'apprend rien et allonge le
    /// tableau d'autant.
    pub fn periodes_occupees(&self) -> Vec<(usize, u64)> {
        let mut vues: Vec<(usize, u64)> = self
            .periodes
            .iter()
            .enumerate()
            .filter(|(_, n)| **n > 0)
            .map(|(i, n)| (i, *n))
            .collect();
        vues.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        vues
    }

    /// Combien d'images ont été comparées à la précédente.
    pub fn comparees(&self) -> u64 {
        self.comparees
    }
}

/// Ce qu'une image a montré, tel que son instantané le portera.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mesure {
    /// La durée pendant laquelle l'image précédente est restée à l'écran.
    pub intervalle_us: u32,
    /// De combien de pixels le contenu s'est montré à côté de sa trajectoire.
    pub saut_px: u16,
    /// Le rapport `temps intégré / temps montré`, en millièmes. Mille vaut « exact ».
    pub fidelite_millieme: u16,
}

/// Une durée en microsecondes, bornée au type qui la porte.
fn micros(d: Duration) -> Option<u32> {
    u32::try_from(d.as_micros()).ok()
}

/// Un nombre réel positif, arrondi et borné.
fn entier(valeur: f64) -> u32 {
    if !valeur.is_finite() || valeur <= 0.0 {
        return 0;
    }
    valeur.min(f64::from(u32::MAX)) as u32
}

#[cfg(test)]
mod tests;
