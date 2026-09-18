//! Ce qui se voit, et ce qui est caché derrière (OCCLUSION-2).
//!
//! # Pourquoi ce module vit dans le noyau
//!
//! Savoir quels pixels d'une image atteindront l'œil est une question **géométrique**. Elle ne
//! dépend ni du rastériseur, ni de la carte graphique, ni du système : les mêmes rectangles
//! donnent la même réponse sur un processeur, sur un GPU, sur un téléphone.
//!
//! Le noyau la calcule donc une fois, sans aucune dépendance, et chaque chemin de rendu
//! l'exécute à sa façon — un report de pixels d'un côté, un quadrilatère texturé de l'autre.
//! Mettre ce calcul dans le rendu processeur l'aurait rendu inutilisable par tout le reste, et
//! l'aurait fait réécrire à chaque nouveau chemin.
//!
//! # Ce que ce module répare
//!
//! Mesuré sur une session réelle : **cent cinq photos couvrant cinquante fois la surface de
//! l'écran**, deux millions de pixels visibles pour cent cinq millions écrits. Quatre-vingt-dix
//! huit pour cent du travail était recouvert avant d'atteindre l'œil.
//!
//! Le cas simple — une photo opaque qui remplit l'écran — se règle en comparant deux
//! rectangles. Le cas réel ne s'y ramène pas : les photos se chevauchent **partiellement**, et
//! aucune ne cache seule ce que plusieurs cachent ensemble.
//!
//! # Comment, et pourquoi c'est exact
//!
//! La fenêtre se découpe en tuiles. Pour chacune, on cherche le **dernier** calque opaque qui
//! la couvre entièrement : tout ce qui vient avant est invisible dans cette tuile.
//!
//! Un calque est alors à dessiner s'il touche au moins une tuile où il n'est pas recouvert, et
//! l'englobant de ces tuiles suffit à le clipper.
//!
//! La réponse est **conservatrice par construction** : on peut désigner un peu plus que le
//! strict nécessaire — une tuile partiellement couverte compte comme non couverte — jamais
//! moins. Une erreur fait donc dessiner en trop, ce qui coûte ; jamais en moins, ce qui se
//! verrait.

/// Un rectangle de l'écran, en pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Boite {
    pub x: f32,
    pub y: f32,
    pub largeur: f32,
    pub hauteur: f32,
}

impl Boite {
    pub fn nouvelle(x: f32, y: f32, largeur: f32, hauteur: f32) -> Self {
        Self {
            x,
            y,
            largeur,
            hauteur,
        }
    }

    pub fn droite(&self) -> f32 {
        self.x + self.largeur
    }

    pub fn bas(&self) -> f32 {
        self.y + self.hauteur
    }

    pub fn est_vide(&self) -> bool {
        self.largeur <= 0.0 || self.hauteur <= 0.0
    }

    /// Contient-elle entièrement l'autre ?
    pub fn contient(&self, autre: &Boite) -> bool {
        self.x <= autre.x
            && self.y <= autre.y
            && self.droite() >= autre.droite()
            && self.bas() >= autre.bas()
    }

    /// Se touchent-elles, ne serait-ce que d'un pixel ?
    pub fn touche(&self, autre: &Boite) -> bool {
        self.x < autre.droite()
            && autre.x < self.droite()
            && self.y < autre.bas()
            && autre.y < self.bas()
    }

    /// Leur partie commune, éventuellement vide.
    pub fn commune(&self, autre: &Boite) -> Boite {
        let x = self.x.max(autre.x);
        let y = self.y.max(autre.y);
        Boite {
            x,
            y,
            largeur: self.droite().min(autre.droite()) - x,
            hauteur: self.bas().min(autre.bas()) - y,
        }
    }

    /// La plus petite boîte qui les contient toutes les deux.
    pub fn englobant(&self, autre: &Boite) -> Boite {
        let x = self.x.min(autre.x);
        let y = self.y.min(autre.y);
        Boite {
            x,
            y,
            largeur: self.droite().max(autre.droite()) - x,
            hauteur: self.bas().max(autre.bas()) - y,
        }
    }
}

/// Une chose à dessiner, dans l'ordre où elle se dessine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calque {
    /// Là où elle se pose à l'écran.
    pub boite: Boite,
    /// Cache-t-elle **entièrement** ce qu'il y a dessous, sur toute sa boîte ?
    ///
    /// Se **constate**, jamais ne s'estime : une image dont un seul pixel est translucide, ou
    /// qui est tournée — sa boîte n'est alors plus ce qu'elle couvre — répond `false`.
    pub opaque: bool,
}

/// Combien de tuiles en travers de la fenêtre.
///
/// # Pourquoi un nombre de tuiles, et non une taille
///
/// Une taille en pixels vieillirait : elle serait fine sur un petit écran et grossière en 4K,
/// alors que ce qu'on découpe est **la scène**, pas la résolution. Un nombre fixe donne des
/// tuiles proportionnelles à la fenêtre, donc une précision relative constante — et un coût de
/// calcul constant, ce qui compte sur un téléphone.
///
/// Trente-deux en travers fait au plus 1 024 tuiles. Pour cent calques, cela représente cent
/// mille comparaisons de rectangles, soit environ un dixième de milliseconde — à comparer aux
/// centaines de millisecondes que le recouvrement coûte. Le calcul se paie mille fois.
const TUILES_EN_TRAVERS: usize = 32;

/// Ce qui reste a peindre, calque par calque.
///
/// # Pourquoi une LISTE de parties et non un rectangle
///
/// Ma premiere version rendait l'englobant des tuiles utiles. Elle ne gagnait **rien**, et la
/// mesure l'a dit sans appel : trente-six millions de pixels avant, trente-six millions apres.
///
/// La raison est geometrique. Une photo recouverte par une autre decalee en diagonale garde
/// une bande a gauche et une bande en haut : une forme en **L**. L'englobant d'un L est le
/// rectangle plein -- on n'avait donc rien retire du tout.
///
/// Un L se decrit avec deux rectangles. C'est pourquoi la reponse est une liste.
///
/// Les parties de tous les calques vivent dans un seul tableau, chaque calque possedant une
/// plage : aucune allocation par calque, sur un chemin parcouru a chaque image.
#[derive(Debug, Default)]
pub struct Visibles {
    parts: Vec<Boite>,
    plages: Vec<(u32, u32)>,
}

impl Visibles {
    /// Les morceaux a peindre pour le calque de ce rang. Vide veut dire **entierement cache**.
    pub fn parts(&self, rang: usize) -> &[Boite] {
        match self.plages.get(rang) {
            Some(&(d, f)) => &self.parts[d as usize..f as usize],
            None => &[],
        }
    }

    /// Combien de calques ont ete examines.
    pub fn calques(&self) -> usize {
        self.plages.len()
    }

    /// La surface totale qu'il reste a peindre, en pixels.
    ///
    /// C'est **la** grandeur qui decide du gain, et non le nombre de calques elimines : un
    /// calque qui ne garde qu'une bande de trois pixels ne coute presque rien, alors qu'il
    /// compte encore pour un.
    pub fn surface(&self) -> f64 {
        self.parts
            .iter()
            .map(|b| f64::from(b.largeur) * f64::from(b.hauteur))
            .sum()
    }

    fn vider(&mut self) {
        self.parts.clear();
        self.plages.clear();
    }
}

/// Calcule ce qui reste a peindre de chaque calque, dans l'ordre de dessin.
///
/// Le resultat est ecrit dans `sortie`, qui est vide d'abord. Le passer plutot que le rendre
/// evite une allocation par image.
pub fn ce_qui_se_voit(calques: &[Calque], fenetre: Boite, sortie: &mut Visibles) {
    sortie.vider();
    if calques.is_empty() || fenetre.est_vide() {
        sortie.plages.resize(calques.len(), (0, 0));
        return;
    }

    let (colonnes, lignes) = grille(&fenetre);
    let (pas_x, pas_y) = (
        fenetre.largeur / colonnes as f32,
        fenetre.hauteur / lignes as f32,
    );

    // Pour chaque tuile, le rang du dernier calque opaque qui la couvre entierement : tout ce
    // qui precede ce rang est invisible dans cette tuile.
    let mut seuils = vec![0usize; colonnes * lignes];
    for (rang, calque) in calques.iter().enumerate() {
        if !calque.opaque {
            continue;
        }
        let visible = calque.boite.commune(&fenetre);
        if visible.est_vide() {
            continue;
        }
        pour_chaque_tuile(
            &visible,
            &fenetre,
            (colonnes, lignes),
            (pas_x, pas_y),
            |t, tuile| {
                // « couvre entierement » : une tuile a moitie couverte ne cache rien, et compte
                // donc comme non couverte. C'est ce qui rend la reponse conservatrice.
                if calque.boite.contient(&tuile) {
                    seuils[t] = rang;
                }
            },
        );
    }

    for (rang, calque) in calques.iter().enumerate() {
        let debut = sortie.parts.len() as u32;
        let visible = calque.boite.commune(&fenetre);
        if !visible.est_vide() {
            decouper(
                &visible,
                &fenetre,
                (colonnes, lignes),
                (pas_x, pas_y),
                (&seuils, rang),
                &mut sortie.parts,
            );
        }
        sortie.plages.push((debut, sortie.parts.len() as u32));
    }
}

/// Decoupe la partie visible d'un calque en rectangles.
///
/// Le cas ou **rien** n'est occulte -- le plus frequent -- rend un seul rectangle : il ne faut
/// pas qu'un calque libre coute plus cher qu'avant sous pretexte qu'on sait decouper.
///
/// Sinon, une ligne de tuiles donne un rectangle par suite de tuiles contigues. Une forme en L
/// en donne quelques-uns, ce qui est exactement le but : leur somme est la surface reelle, la
/// ou un englobant aurait rendu le rectangle plein.
fn decouper(
    visible: &Boite,
    fenetre: &Boite,
    (colonnes, lignes): (usize, usize),
    (pas_x, pas_y): (f32, f32),
    (seuils, rang): (&[usize], usize),
    parts: &mut Vec<Boite>,
) {
    let indice = |v: f32, origine: f32, pas: f32, max: usize| {
        (((v - origine) / pas).floor() as isize).clamp(0, max as isize - 1) as usize
    };
    let c0 = indice(visible.x, fenetre.x, pas_x, colonnes);
    let c1 = indice(visible.droite() - f32::EPSILON, fenetre.x, pas_x, colonnes);
    let l0 = indice(visible.y, fenetre.y, pas_y, lignes);
    let l1 = indice(visible.bas() - f32::EPSILON, fenetre.y, pas_y, lignes);

    let utile = |l: usize, c: usize| seuils[l * colonnes + c] <= rang;
    let tout_utile = (l0..=l1).all(|l| (c0..=c1).all(|c| utile(l, c)));
    if tout_utile {
        parts.push(*visible);
        return;
    }

    for l in l0..=l1 {
        let mut segment: Option<usize> = None;
        for c in c0..=c1 {
            match (utile(l, c), segment) {
                (true, None) => segment = Some(c),
                (false, Some(d)) => {
                    parts.push(rect_de_tuiles(
                        visible,
                        fenetre,
                        (d, c - 1),
                        l,
                        (pas_x, pas_y),
                    ));
                    segment = None;
                }
                _ => {}
            }
        }
        if let Some(d) = segment {
            parts.push(rect_de_tuiles(visible, fenetre, (d, c1), l, (pas_x, pas_y)));
        }
    }
}

/// Le rectangle couvert par une suite de tuiles d'une meme ligne, ramene a la partie visible.
fn rect_de_tuiles(
    visible: &Boite,
    fenetre: &Boite,
    (c0, c1): (usize, usize),
    l: usize,
    (pas_x, pas_y): (f32, f32),
) -> Boite {
    let bloc = Boite {
        x: fenetre.x + c0 as f32 * pas_x,
        y: fenetre.y + l as f32 * pas_y,
        largeur: (c1 - c0 + 1) as f32 * pas_x,
        hauteur: pas_y,
    };
    bloc.commune(visible)
}

/// Le nombre de tuiles, en colonnes et en lignes.
///
/// La fenetre n'est pas carree : on garde des tuiles a peu pres carrees plutot qu'une grille
/// carree, pour qu'une tuile ait la meme finesse dans les deux directions.
fn grille(fenetre: &Boite) -> (usize, usize) {
    let cote = (fenetre.largeur.max(fenetre.hauteur) / TUILES_EN_TRAVERS as f32).max(1.0);
    let colonnes = ((fenetre.largeur / cote).ceil() as usize).clamp(1, TUILES_EN_TRAVERS);
    let lignes = ((fenetre.hauteur / cote).ceil() as usize).clamp(1, TUILES_EN_TRAVERS);
    (colonnes, lignes)
}

/// Appelle `f` pour chaque tuile que `zone` touche, avec son indice et sa boite.
fn pour_chaque_tuile(
    zone: &Boite,
    fenetre: &Boite,
    (colonnes, lignes): (usize, usize),
    (pas_x, pas_y): (f32, f32),
    mut f: impl FnMut(usize, Boite),
) {
    let colonne = |x: f32| {
        (((x - fenetre.x) / pas_x).floor() as isize).clamp(0, colonnes as isize - 1) as usize
    };
    let ligne = |y: f32| {
        (((y - fenetre.y) / pas_y).floor() as isize).clamp(0, lignes as isize - 1) as usize
    };

    // Le bord droit appartient a la tuile precedente : sans le recul d'un epsilon, une zone qui
    // s'arrete pile sur une frontiere reclamerait une tuile qu'elle ne touche pas.
    for l in ligne(zone.y)..=ligne(zone.bas() - f32::EPSILON) {
        for c in colonne(zone.x)..=colonne(zone.droite() - f32::EPSILON) {
            let tuile = Boite {
                x: fenetre.x + c as f32 * pas_x,
                y: fenetre.y + l as f32 * pas_y,
                largeur: pas_x,
                hauteur: pas_y,
            };
            f(l * colonnes + c, tuile);
        }
    }
}

#[cfg(test)]
mod tests;
