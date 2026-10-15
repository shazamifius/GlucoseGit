//! **Une forme en courbes du second degré, prête pour la carte graphique** (P8, fiche 73) — la
//! moitié « données » de l'algorithme **Slug** d'Eric Lengyel.
//!
//! # Ce qu'il a vu
//!
//! Le 15/10, de près, le texte devenait flou, en gros pixels, *« absolument illisible et
//! impossible à travailler »*, et ne redevenait net qu'au bout d'un moment. La carte graphique
//! posait un nœud de texte comme une **image** que le processeur avait peinte ; de près, l'image
//! se découpait en carrés repeints un à un, et en attendant on voyait l'image entière agrandie
//! dix ou vingt fois (fiche 72 § 5). Tant que le texte est une image, un zoom la refait.
//!
//! # La réponse : dessiner la lettre depuis son contour, à chaque pixel
//!
//! Slug (Lengyel, *GPU-Centered Font Rendering Directly from Glyph Outlines*, JCGT 2017 ; *A
//! Decade of Slug*, 2026) : pour chaque pixel, la carte graphique lance un rayon horizontal et
//! un rayon vertical, cherche où ils croisent les courbes du contour — une équation du second
//! degré par courbe —, et en tire la part du pixel que la lettre couvre. Aucune image n'est
//! jamais faite : la lettre est nette à toute taille, et un zoom ne coûte rien de plus. Le brevet
//! est dans le domaine public depuis le 17/03/2026, les nuanceurs de référence sous licence MIT
//! (le dépôt `EricLengyel/Slug`).
//!
//! Deux idées le rendent rapide et sûr, et ce module les tient :
//!
//! * **les bandes** : la boîte de la forme est coupée en bandes horizontales et verticales, et
//!   chacune ne nomme que les courbes qui la traversent — un pixel ne lit que celles de sa bande ;
//! * **les courbes triées** : dans une bande horizontale, de la plus à droite à la plus à gauche ;
//!   un pixel s'arrête dès qu'une courbe passe tout entière à sa gauche.
//!
//! # Ce que ce module écrit
//!
//! * [`Contours`] bâtit les contours fermés d'une forme en **quadratiques**, depuis les gestes
//!   d'un tracé (aller, ligne, quadratique, cubique, fermer) — ceux des tables d'une police ;
//! * [`emballage::emballer`] les range comme la carte les lit ;
//! * [`couverture::couverture`] est **la loi** que le nuanceur applique en chaque pixel, écrite
//!   ici en Rust : le nuanceur la recopie fonction pour fonction (comme les membranes,
//!   `membrane_forme`), et l'épreuve des deux voies compare la carte à elle.
//!
//! # CUBIQUE-1 — une cubique devient des quadratiques, à la précision de sa propre grille
//!
//! Slug ne résout que le second degré. Les polices TrueType (Inter droit, JetBrains Mono, Noto
//! Emoji, les fontes de KaTeX) n'ont que des quadratiques : elles passent **exactement**. Les
//! polices CFF (Inter Italic) et les formes de KaTeX ont des cubiques : chacune se coupe en `n`
//! morceaux égaux, et chaque morceau devient la quadratique dont le contrôle est
//! `(3(c₁ + c₂) − (p₀ + p₃)) / 4`. L'écart entre un morceau et sa quadratique est borné par
//! `√3/36 · |p₃ − 3c₂ + 3c₁ − p₀|` (la borne classique de cette approximation, vérifiée par
//! l'épreuve), et la troisième différence d'un morceau vaut celle de la cubique divisée par
//! `n³` : **`n` se calcule**, il ne se cherche pas.
//!
//! **La tolérance n'est pas choisie** : c'est la moitié de l'unité de la grille où la forme a été
//! dessinée — l'unité de la police pour une lettre (son dessinateur a posé chaque point sur cette
//! grille, à une demi-unité près), la moitié de ce que la sortie peut dire pour une forme posée à
//! l'écran. C'est aussi ce que subit la version TrueType de toute police CFF (`cu2qu`, l'outil
//! de Google Fonts, convertit à une unité près).

pub mod couverture;
pub mod emballage;

#[cfg(test)]
mod tests;

/// Un point du plan de la forme.
pub type Point = [f64; 2];

/// **Une courbe du second degré** : ses deux extrémités et son point de contrôle.
///
/// Une ligne en est une, son contrôle au milieu : son terme du second degré est nul, et la loi
/// la résout comme une droite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quadratique {
    pub debut: Point,
    pub controle: Point,
    pub fin: Point,
}

impl Quadratique {
    /// Le point de la courbe au paramètre `t`.
    pub fn point(&self, t: f64) -> Point {
        let u = 1.0 - t;
        let [a, b, c] = [u * u, 2.0 * t * u, t * t];
        [
            a * self.debut[0] + b * self.controle[0] + c * self.fin[0],
            a * self.debut[1] + b * self.controle[1] + c * self.fin[1],
        ]
    }
}

/// `√3/36` : la borne de l'écart entre une cubique et sa quadratique du milieu, par unité de
/// sa troisième différence (CUBIQUE-1).
const BORNE_DU_MILIEU: f64 = 0.048_112_522_432_468_816;

/// **Les contours fermés d'une forme**, bâtis geste après geste.
///
/// Chaque contour est une suite de quadratiques **jointes** : la fin de l'une est, au bit près,
/// le début de la suivante, et la dernière finit où la première commence. C'est ce qui permet à
/// l'emballage de ne ranger chaque extrémité qu'une fois (Slug la partage entre deux courbes).
#[derive(Debug, Clone)]
pub struct Contours {
    faits: Vec<Vec<Quadratique>>,
    courant: Vec<Quadratique>,
    depart: Point,
    plume: Point,
    /// L'écart toléré entre une cubique et ses quadratiques (CUBIQUE-1).
    tolerance: f64,
}

impl Contours {
    /// Des contours dont les cubiques s'approchent à `tolerance` près, dans l'unité de la forme.
    pub fn nouveaux(tolerance: f64) -> Self {
        Self {
            faits: Vec::new(),
            courant: Vec::new(),
            depart: [0.0; 2],
            plume: [0.0; 2],
            tolerance,
        }
    }

    /// Commence un contour en `p` ; ferme celui d'avant, comme tout remplissage le ferme.
    pub fn aller(&mut self, p: Point) {
        self.fermer();
        self.depart = p;
        self.plume = p;
    }

    /// Une ligne jusqu'à `p`. Une ligne de longueur nulle n'existe pas.
    pub fn ligne(&mut self, p: Point) {
        if p == self.plume {
            return;
        }
        let milieu = [(self.plume[0] + p[0]) / 2.0, (self.plume[1] + p[1]) / 2.0];
        self.pousser(milieu, p);
    }

    /// Une quadratique de contrôle `c` jusqu'à `p`.
    pub fn quadratique(&mut self, c: Point, p: Point) {
        if p == self.plume && c == self.plume {
            return;
        }
        self.pousser(c, p);
    }

    /// **Une cubique** de contrôles `c1`, `c2` jusqu'à `p`, en autant de quadratiques que sa
    /// tolérance l'exige (CUBIQUE-1).
    pub fn cubique(&mut self, c1: Point, c2: Point, p: Point) {
        let p0 = self.plume;
        let n = morceaux_d_une_cubique([p0, c1, c2, p], self.tolerance);
        for i in 0..n {
            let (t0, t1) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
            let morceau = sous_cubique([p0, c1, c2, p], t0, t1);
            // Le dernier morceau finit exactement en `p` : `B(1)` le rend au bit près, mais
            // `p` est ce que la plume doit valoir pour que le geste suivant s'y joigne.
            let fin = if i + 1 == n { p } else { morceau[3] };
            let controle = [
                (3.0 * (morceau[1][0] + morceau[2][0]) - (morceau[0][0] + morceau[3][0])) / 4.0,
                (3.0 * (morceau[1][1] + morceau[2][1]) - (morceau[0][1] + morceau[3][1])) / 4.0,
            ];
            self.pousser(controle, fin);
        }
    }

    /// Ferme le contour en cours : une ligne jusqu'à son départ, s'il n'y est pas déjà.
    pub fn fermer(&mut self) {
        if self.courant.is_empty() {
            return;
        }
        let depart = self.depart;
        self.ligne(depart);
        self.faits.push(std::mem::take(&mut self.courant));
        self.plume = depart;
    }

    /// **Les contours finis**, chacun fermé.
    pub fn finir(mut self) -> Vec<Vec<Quadratique>> {
        self.fermer();
        self.faits
    }

    fn pousser(&mut self, controle: Point, fin: Point) {
        self.courant.push(Quadratique {
            debut: self.plume,
            controle,
            fin,
        });
        self.plume = fin;
    }
}

/// **En combien de morceaux égaux une cubique se coupe** pour que chacun s'écarte de sa
/// quadratique du milieu de `tolerance` au plus (CUBIQUE-1) : `√3/36 · |Δ³| / n³ ≤ tolerance`.
pub fn morceaux_d_une_cubique([p0, c1, c2, p3]: [Point; 4], tolerance: f64) -> usize {
    let d = [
        p3[0] - 3.0 * c2[0] + 3.0 * c1[0] - p0[0],
        p3[1] - 3.0 * c2[1] + 3.0 * c1[1] - p0[1],
    ];
    let ecart = BORNE_DU_MILIEU * d[0].hypot(d[1]);
    if tolerance.is_nan() || tolerance <= 0.0 || ecart <= tolerance {
        return 1;
    }
    (ecart / tolerance).cbrt().ceil() as usize
}

/// Le morceau `[t0, t1]` d'une cubique, lui-même une cubique : ses extrémités, et ses
/// contrôles tirés des tangentes (la forme de Hermite, exacte).
pub fn sous_cubique(c: [Point; 4], t0: f64, t1: f64) -> [Point; 4] {
    let (a, b) = (point_de_cubique(&c, t0), point_de_cubique(&c, t1));
    let (da, db) = (derivee_de_cubique(&c, t0), derivee_de_cubique(&c, t1));
    let h = (t1 - t0) / 3.0;
    [
        a,
        [a[0] + h * da[0], a[1] + h * da[1]],
        [b[0] - h * db[0], b[1] - h * db[1]],
        b,
    ]
}

/// Le point d'une cubique au paramètre `t`.
pub fn point_de_cubique(c: &[Point; 4], t: f64) -> Point {
    let u = 1.0 - t;
    let k = [u * u * u, 3.0 * t * u * u, 3.0 * t * t * u, t * t * t];
    let mut p = [0.0; 2];
    for (ki, ci) in k.iter().zip(c) {
        p[0] += ki * ci[0];
        p[1] += ki * ci[1];
    }
    p
}

fn derivee_de_cubique(c: &[Point; 4], t: f64) -> Point {
    let u = 1.0 - t;
    let k = [3.0 * u * u, 6.0 * t * u, 3.0 * t * t];
    let mut d = [0.0; 2];
    for (i, ki) in k.iter().enumerate() {
        d[0] += ki * (c[i + 1][0] - c[i][0]);
        d[1] += ki * (c[i + 1][1] - c[i][1]);
    }
    d
}
