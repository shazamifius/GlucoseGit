//! **RECADRAGE-1** : ce qu'on retire des bords d'une image, sans toucher à ses octets.
//!
//! # Ce qui manquait, et ce que ce n'est pas
//!
//! Glucose sait **redimensionner** une image — changer la place qu'elle occupe — et ne savait
//! pas la **recadrer** — changer ce qu'elle montre. Les deux gestes se ressemblent à la souris
//! et n'ont rien à voir : l'un étire, l'autre coupe.
//!
//! # Non destructif, et ce n'est pas une précaution — c'est la seule façon juste
//!
//! Recadrer ne réécrit pas le fichier. Les octets de la photo restent ceux qu'ils étaient, et
//! le recadrage vit dans le document, à côté de la position et de la rotation. Trois raisons,
//! et la dernière suffirait :
//!
//! * **on peut revenir en arrière**, des mois plus tard, sans avoir rien perdu ;
//! * **une photo posée deux fois** peut être cadrée différemment de chaque côté, alors qu'elle
//!   ne porte qu'un fichier ;
//! * **c'est ce que la charte exige du rendu** : le renderer *lit*, il ne calcule pas
//!   (fiche 05 § 4.4). Un recadrage qui produirait de nouveaux octets ferait du dessin un
//!   endroit où des données naissent.
//!
//! C'est aussi ce que font tous les canevas qui ont ce geste — le recadrage y est une fenêtre
//! sur la source, jamais une coupe dans le fichier.
//!
//! # Des fractions, et pas des pixels
//!
//! Les quatre nombres sont des **parts de l'original**, entre zéro et un. En pixels, ils
//! dépendraient de la définition de la photo : remplacer une image par la même en plus grand
//! déplacerait le cadrage, et une vignette ne se cadrerait pas comme sa source. Une fraction
//! ne dépend de rien.
//!
//! # Pas d'`Option`, et c'est une ambiguïté en moins
//!
//! « Pas de recadrage » et « un recadrage qui garde tout » sont le **même** état. Les
//! représenter par deux valeurs distinctes — `None` et `Some(ENTIER)` — ferait exister une
//! comparaison qui ment, et il faudrait s'en souvenir partout. Il n'y a donc qu'un type, et
//! [`Recadrage::ENTIER`] est son neutre.

/// Ce qu'on retire d'une image sur chacun de ses quatre bords, en fractions de l'original.
///
/// # Invariant RECADRAGE-1 — il reste toujours quelque chose
///
/// Les quatre parts sont dans `[0, 1[`, et deux parts opposées laissent toujours une bande
/// visible : `gauche + droite < 1` et `haut + bas < 1`. Un recadrage qui ne laisserait rien
/// n'est pas un cadrage serré, c'est une image disparue — et une division par sa largeur
/// visible donnerait un infini au rendu.
///
/// L'invariant se tient à la **construction** ([`Recadrage::depuis_les_marges`]), qui est le
/// seul chemin : les champs ne sortent pas du noyau, et la lecture d'un document passe par le
/// même constructeur qu'une détection de bordures.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Recadrage {
    pub(crate) gauche: f64,
    pub(crate) haut: f64,
    pub(crate) droite: f64,
    pub(crate) bas: f64,
}

impl Default for Recadrage {
    fn default() -> Self {
        Self::ENTIER
    }
}

impl Recadrage {
    /// L'image entière : rien n'est retiré. C'est le neutre, et l'état de toute image qu'on
    /// n'a jamais cadrée.
    pub const ENTIER: Self = Self {
        gauche: 0.0,
        haut: 0.0,
        droite: 0.0,
        bas: 0.0,
    };

    /// **Un recadrage depuis ce qu'on retire de chaque bord**, ramené dans son domaine.
    ///
    /// Il ne refuse pas : il **ramène**. Un appelant qui proposerait des marges impossibles —
    /// une détection de bordures sur une image entièrement noire, une poignée tirée trop loin —
    /// attend une image, pas une erreur à traiter. Ce qui est trop grand est réduit à ce qui
    /// laisse une bande visible, à parts égales de chaque côté, ce qui est le seul partage qui
    /// ne privilégie aucun bord.
    pub fn depuis_les_marges(gauche: f64, haut: f64, droite: f64, bas: f64) -> Self {
        let (gauche, droite) = Self::paire(gauche, droite);
        let (haut, bas) = Self::paire(haut, bas);
        Self {
            gauche,
            haut,
            droite,
            bas,
        }
    }

    /// Deux marges opposées, ramenées à ce qui laisse une bande visible.
    ///
    /// La bande minimale est **un pour cent** de l'original, et ce n'est pas une préférence :
    /// c'est le plus petit cadrage qu'un geste puisse viser sans que la division par la largeur
    /// visible ne fasse exploser l'échelle de rendu. Un pour cent d'une photo de quatre mille
    /// pixels fait quarante pixels, ce qui est déjà plus serré que tout cadrage réel.
    fn paire(avant: f64, apres: f64) -> (f64, f64) {
        const RESTE_MINIMAL: f64 = 0.01;
        // Un nombre qui n'est pas fini ne vient pas d'un geste : il vient d'un fichier abime.
        // Il ne retire rien, plutot que de faire naitre un NaN au moment de le ramener -- le
        // test l'a attrape au premier essai : l'infini fois le facteur nul fait NaN, et une
        // largeur visible NaN passe tous les `> 0` sans en satisfaire aucun.
        let saine = |m: f64| if m.is_finite() { m.max(0.0) } else { 0.0 };
        let avant = saine(avant);
        let apres = saine(apres);
        let retire = avant + apres;
        let permis = 1.0 - RESTE_MINIMAL;
        if retire <= permis {
            return (avant, apres);
        }
        // Tout est réduit du même facteur : les deux bords gardent leur proportion, et un
        // cadrage très décentré ne se recentre pas tout seul.
        let facteur = permis / retire;
        (avant * facteur, apres * facteur)
    }

    /// Rien n'est retiré : c'est l'image entière.
    pub fn est_entier(self) -> bool {
        self == Self::ENTIER
    }

    /// Ce qu'on retire à gauche, en haut, à droite, en bas — en fractions de l'original.
    pub fn marges(self) -> (f64, f64, f64, f64) {
        (self.gauche, self.haut, self.droite, self.bas)
    }

    /// La part de la largeur de l'original qui reste visible. Toujours strictement positive.
    pub fn largeur_visible(self) -> f64 {
        1.0 - self.gauche - self.droite
    }

    /// La part de la hauteur de l'original qui reste visible. Toujours strictement positive.
    pub fn hauteur_visible(self) -> f64 {
        1.0 - self.haut - self.bas
    }

    /// **Où l'image ENTIÈRE se poserait pour que sa fenêtre visible coïncide avec `boite`.**
    ///
    /// C'est l'inverse de la fenêtre, et c'est ce qui rend le recadrage gratuit sur la voie
    /// processeur : la boîte du nœud reste la boîte, le clip du report reste le clip, et seule
    /// la **pose de la source** change — plus grande, décalée vers le haut et la gauche de ce
    /// qu'on retire. Le report n'itère que sur le clip (REPORT-1), donc une source posée plus
    /// grande ne coûte pas un pixel de plus.
    ///
    /// `boite` est `(x, y, largeur, hauteur)` dans n'importe quelle unité ; le résultat est
    /// dans la même.
    pub fn source_pour(
        self,
        (x, y, largeur, hauteur): (f64, f64, f64, f64),
    ) -> (f64, f64, f64, f64) {
        let pleine_largeur = largeur / self.largeur_visible();
        let pleine_hauteur = hauteur / self.hauteur_visible();
        (
            x - self.gauche * pleine_largeur,
            y - self.haut * pleine_hauteur,
            pleine_largeur,
            pleine_hauteur,
        )
    }

    /// **Le plus serré des deux** : chaque marge est la plus grande des deux.
    ///
    /// C'est ce qu'il faut quand une détection de bordures rencontre un recadrage que
    /// l'utilisateur a déjà posé : elle ne doit jamais **relâcher** ce qu'il a serré, et il n'y
    /// a aucune raison qu'elle garde une bande qu'il avait laissée.
    pub fn le_plus_serre(self, autre: Self) -> Self {
        Self::depuis_les_marges(
            self.gauche.max(autre.gauche),
            self.haut.max(autre.haut),
            self.droite.max(autre.droite),
            self.bas.max(autre.bas),
        )
    }

    /// **La boîte que le nœud doit occuper pour que ce qui reste visible ne bouge pas**, quand
    /// son recadrage passe de `self` à `nouveau`.
    ///
    /// Recadrer ne doit pas déplacer les pixels qu'on garde : une bande retirée à gauche fait
    /// reculer le bord gauche de la boîte, et rien d'autre. Le calcul est une composition de
    /// [`Self::source_pour`] — où la source entière se posait — et de la fenêtre que le nouveau
    /// recadrage y découpe. Sans rotation ; l'appelant tourne le décalage du centre s'il y a
    /// lieu.
    ///
    /// `boite` est `(x, y, largeur, hauteur)`, coin haut-gauche, dans n'importe quelle unité.
    pub fn boite_apres(self, nouveau: Self, boite: (f64, f64, f64, f64)) -> (f64, f64, f64, f64) {
        let (sx, sy, sw, sh) = self.source_pour(boite);
        (
            sx + nouveau.gauche * sw,
            sy + nouveau.haut * sh,
            sw * nouveau.largeur_visible(),
            sh * nouveau.hauteur_visible(),
        )
    }

    /// **Le rapport largeur sur hauteur que ce cadrage donne** à une image d'origine `(l, h)`.
    ///
    /// C'est ce qui permet de garder la forme de ce qu'on montre quand on cadre : couper les
    /// bandes noires d'une image en boîte aux lettres change son rapport, et laisser sa boîte
    /// inchangée l'écraserait.
    pub fn rapport(self, largeur: f64, hauteur: f64) -> f64 {
        let l = largeur * self.largeur_visible();
        let h = hauteur * self.hauteur_visible();
        if h <= 0.0 {
            return 1.0;
        }
        l / h
    }
}

#[cfg(test)]
mod tests;
