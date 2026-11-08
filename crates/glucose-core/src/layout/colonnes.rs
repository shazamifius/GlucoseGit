//! **COLONNES-1 — disposer une membrane en colonnes de journal** (fiches 91 et 92).
//!
//! Un article de Wikipédia posé par les commandes s'empilait en une colonne de 60 000 unités de haut :
//! illisible pour un humain, quand VISION veut que la disposition en fasse *« un travail presque
//! d'illustration »*. Chaque membrane range ses enfants — textes, post-it, photos, sous-membranes déjà
//! rangées —, **dans l'ordre de lecture**, en colonnes contiguës, de bas en haut (la disposition « de
//! bas en haut » des graphes composés : Kasperowski et von Hanxleden, *Top-Down Drawings of Compound
//! Graphs*, ce qu'ils n'ont pas étant l'ordre de lecture).
//!
//! # Deux bissections, aucune constante à régler
//!
//! * **Pour k colonnes, la plus basse hauteur** : c'est le « partage des peintres ». Une hauteur
//!   donnée, remplir les colonnes dans l'ordre dit combien il en faut ([`colonnes_pour`]) ; ce nombre
//!   ne croît jamais quand la hauteur croît. Une bissection sur la hauteur rend donc, au flottant
//!   près, la plus basse qui tienne en k colonnes — l'optimum exact de la partition linéaire, en
//!   O(n) par pas plutôt qu'en O(k·n²).
//! * **Le nombre de colonnes** : plus il y en a, plus la membrane s'élargit et s'abaisse ; sa
//!   proportion ne décroît pas avec k. Une seconde bissection trouve le k dont la proportion approche
//!   le mieux celle qu'on vise — celle de l'écran, que l'application donne.
//!
//! Les marges sont celles d'une membrane de Glucose (son bord, l'écart entre deux nœuds, le bandeau de
//! son titre), reçues telles quelles : le module n'en choisit aucune.
//!
//! **Le titre d'une membrane grandit avec elle** (TITRE-1, fiche 95) et se pose au-dessus de son bord :
//! chaque sous-membrane titrée réserve au-dessus d'elle ce que son titre prend en plus d'un titre
//! ordinaire ([`crate::membrane_titre::surcroit`]), pour qu'aucun nom ne couvre le bloc du dessus.

use crate::geometry::Rect;

/// **Un bloc à disposer** : un nœud (sa taille), ou une membrane et ce qu'elle contient, dans l'ordre
/// de lecture.
#[derive(Debug, Clone, PartialEq)]
pub struct Bloc {
    pub id: String,
    pub largeur: f64,
    pub hauteur: f64,
    /// Vide pour un nœud ; pour une membrane, ce qu'elle contient. Une membrane vide garde sa taille.
    pub enfants: Vec<Bloc>,
    /// Une membrane qui porte un titre, posé au-dessus de son bord (TITRE-1).
    pub titree: bool,
}

/// **Les marges d'une membrane** : à gauche, à droite, en bas et entre deux blocs, `ecart` ; en haut,
/// le bandeau de son `titre`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Marges {
    pub ecart: f64,
    pub titre: f64,
}

/// **La boîte de chaque bloc**, la racine comprise, relative au coin haut-gauche de la racine.
pub fn disposer(racine: &Bloc, proportion: f64, m: Marges) -> Vec<(String, Rect)> {
    let d = disposer_bloc(racine, proportion, m);
    let mut boites = vec![(racine.id.clone(), Rect::new(0.0, 0.0, d.largeur, d.hauteur))];
    boites.extend(d.places);
    boites
}

/// Un bloc disposé : sa taille, la place que son titre prend au-dessus de lui en plus d'un titre
/// ordinaire, et la boîte de chacun de ses descendants, relative à son coin.
struct Dispose {
    largeur: f64,
    hauteur: f64,
    dessus: f64,
    places: Vec<(String, Rect)>,
}

/// Ce que le titre d'un bloc de cette taille prend au-dessus de lui (TITRE-1).
fn dessus(b: &Bloc, largeur: f64, hauteur: f64) -> f64 {
    match b.titree {
        true => crate::membrane_titre::surcroit(largeur, hauteur),
        false => 0.0,
    }
}

fn disposer_bloc(b: &Bloc, proportion: f64, m: Marges) -> Dispose {
    if b.enfants.is_empty() {
        return Dispose {
            largeur: b.largeur,
            hauteur: b.hauteur,
            dessus: dessus(b, b.largeur, b.hauteur),
            places: Vec::new(),
        };
    }
    let fils: Vec<Dispose> = b
        .enfants
        .iter()
        .map(|e| disposer_bloc(e, proportion, m))
        .collect();
    let hauteurs: Vec<f64> = fils.iter().map(|f| f.dessus + f.hauteur).collect();
    let coupes = meilleures_coupes(&fils, &hauteurs, proportion, m);
    let mut places = Vec::new();
    let mut x = m.ecart;
    let mut bas: f64 = 0.0;
    for (debut, fin) in coupes {
        let mut y = m.titre;
        let mut large: f64 = 0.0;
        for (e, f) in b.enfants[debut..fin].iter().zip(&fils[debut..fin]) {
            y += f.dessus;
            places.push((e.id.clone(), Rect::new(x, y, f.largeur, f.hauteur)));
            places.extend(f.places.iter().map(|(id, r)| {
                (
                    id.clone(),
                    Rect::new(r.left + x, r.top + y, r.width, r.height),
                )
            }));
            y += f.hauteur + m.ecart;
            large = large.max(f.largeur);
        }
        bas = bas.max(y);
        x += large + m.ecart;
    }
    Dispose {
        largeur: x,
        hauteur: bas,
        dessus: dessus(b, x, bas),
        places,
    }
}

/// La taille d'une membrane qui range `fils` selon ces coupes.
fn taille(fils: &[Dispose], hauteurs: &[f64], coupes: &[(usize, usize)], m: Marges) -> (f64, f64) {
    let large: f64 = coupes
        .iter()
        .map(|&(a, z)| fils[a..z].iter().map(|f| f.largeur).fold(0.0, f64::max))
        .sum();
    let haut = coupes
        .iter()
        .map(|&(a, z)| colonne(&hauteurs[a..z], m.ecart))
        .fold(0.0, f64::max);
    let n = coupes.len() as f64;
    (large + (n + 1.0) * m.ecart, m.titre + haut + m.ecart)
}

/// **Poser un bloc de hauteur `x` sous une colonne** qui monte déjà à `cumul` (rien : une colonne
/// neuve) : la seule façon d'additionner une colonne. Le remplissage ([`colonnes_pour`]) et la
/// hauteur d'une colonne ([`colonne`]) la partagent, dans le même ordre : ils s'accordent au bit
/// près. Deux ordres de somme diffèrent au dernier bit, et une colonne de trop naissait d'un
/// arrondi (fiche 96) — le remplissage ne tenait pas sous la hauteur de toute la suite.
fn empiler(cumul: Option<f64>, x: f64, ecart: f64) -> f64 {
    cumul.map_or(x, |c| c + ecart + x)
}

/// La hauteur d'une colonne : ses blocs et les écarts entre eux, empilés dans l'ordre.
fn colonne(h: &[f64], ecart: f64) -> f64 {
    h.iter()
        .fold(None, |c, &x| Some(empiler(c, x, ecart)))
        .unwrap_or(0.0)
}

/// **Les coupes dont la proportion approche le mieux celle qu'on vise** : la bissection sur le nombre
/// de colonnes, puis le meilleur des deux voisins de la frontière.
fn meilleures_coupes(
    fils: &[Dispose],
    hauteurs: &[f64],
    proportion: f64,
    m: Marges,
) -> Vec<(usize, usize)> {
    let ratio = |k: usize| {
        let c = coupes_pour(hauteurs, k, m.ecart);
        let (l, h) = taille(fils, hauteurs, &c, m);
        (c, l / h)
    };
    // Le plus petit k dont la proportion atteint celle qu'on vise.
    let (mut bas, mut haut) = (1, hauteurs.len());
    while bas < haut {
        let milieu = (bas + haut) / 2;
        if ratio(milieu).1 >= proportion {
            haut = milieu;
        } else {
            bas = milieu + 1;
        }
    }
    let ecart = |r: f64| (r / proportion).ln().abs();
    let (c, r) = ratio(bas);
    if bas > 1 {
        let (c0, r0) = ratio(bas - 1);
        if ecart(r0) <= ecart(r) {
            return c0;
        }
    }
    c
}

/// **Combien de colonnes il faut** pour que nulle ne dépasse `limite`, en les remplissant dans
/// l'ordre ; et leurs coupes. Chaque colonne monte exactement à sa [`colonne`] : à la hauteur de
/// toute la suite, une seule colonne suffit, au bit près.
fn colonnes_pour(h: &[f64], limite: f64, ecart: f64) -> Vec<(usize, usize)> {
    let mut coupes = Vec::new();
    let (mut debut, mut cumul) = (0, None);
    for (i, &x) in h.iter().enumerate() {
        let avec = empiler(cumul, x, ecart);
        if cumul.is_some() && avec > limite {
            coupes.push((debut, i));
            debut = i;
            cumul = Some(empiler(None, x, ecart));
        } else {
            cumul = Some(avec);
        }
    }
    coupes.push((debut, h.len()));
    coupes
}

/// **Les coupes en au plus k colonnes dont la plus haute est la plus basse possible** : la
/// bissection sur la hauteur, jusqu'à deux flottants voisins.
fn coupes_pour(h: &[f64], k: usize, ecart: f64) -> Vec<(usize, usize)> {
    let mut bas = h.iter().copied().fold(0.0, f64::max);
    let mut haut = colonne(h, ecart);
    if colonnes_pour(h, bas, ecart).len() <= k {
        return colonnes_pour(h, bas, ecart);
    }
    loop {
        let milieu = bas + (haut - bas) / 2.0;
        if milieu <= bas || milieu >= haut {
            return colonnes_pour(h, haut, ecart);
        }
        if colonnes_pour(h, milieu, ecart).len() <= k {
            haut = milieu;
        } else {
            bas = milieu;
        }
    }
}

#[cfg(test)]
mod tests;
