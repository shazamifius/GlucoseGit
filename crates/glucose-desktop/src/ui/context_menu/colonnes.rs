//! **Les colonnes du menu** (ACCES-1) : ses entrées rangées en colonnes qui tiennent dans
//! l'écran, puis posées colonne après colonne.

use super::{Ligne, MenuRow};

/// **Les rangées, posées colonne après colonne** depuis le coin `(x, y)` du menu. Chaque
/// colonne a la largeur de sa plus longue entrée ; ses entrées, cette largeur moins les marges
/// — la mesure d'une seule colonne, inchangée.
pub(super) fn poser(
    colonnes: Vec<Vec<Ligne>>,
    largeurs: &[f32],
    (x, y): (f32, f32),
    (pad, rangee, filet): (f32, f32, f32),
) -> Vec<MenuRow> {
    let mut rows = Vec::new();
    let mut gauche = x;
    for (colonne, &l) in colonnes.into_iter().zip(largeurs) {
        let mut curseur = y + pad;
        for def in colonne {
            match def {
                Some((action, label, shortcut)) => {
                    rows.push(MenuRow::Item {
                        action,
                        label,
                        shortcut,
                        rect: (gauche + pad, curseur, l - pad * 2.0, rangee),
                    });
                    curseur += rangee;
                }
                None => {
                    rows.push(MenuRow::Separator {
                        y: curseur + filet / 2.0,
                        x: gauche + pad,
                        largeur: l - pad * 2.0,
                    });
                    curseur += filet;
                }
            }
        }
        gauche += l;
    }
    rows
}

/// La hauteur d'une colonne d'entrées : une rangée par entrée, un filet par séparation.
pub(super) fn hauteur_de(colonne: &[Ligne], (rangee, filet): (f32, f32)) -> f32 {
    colonne
        .iter()
        .map(|d| if d.is_some() { rangee } else { filet })
        .sum()
}

/// **Les entrées, en colonnes qui tiennent dans `hauteur`** (ACCES-1) — la règle du rail :
/// colonne après colonne, quand la place manque. Un menu plus haut que l'écran en perdait le
/// bas : au doigt, sur un téléphone de cinq pouces, il n'en tient qu'une douzaine.
///
/// Une colonne se ferme **à une séparation** quand le groupe qui suit ne tiendrait pas entier
/// sous elle : les gestes d'un même groupe restent ensemble. Seul un groupe plus haut qu'une
/// colonne se coupe entre deux entrées. Une séparation ne commence ni ne finit une colonne.
pub(super) fn en_colonnes(
    defs: Vec<Ligne>,
    hauteur: f32,
    (rangee, filet): (f32, f32),
) -> Vec<Vec<Ligne>> {
    let groupes: Vec<&[Ligne]> = defs.split(Option::is_none).collect();
    let mut colonnes: Vec<Vec<Ligne>> = vec![Vec::new()];
    for groupe in groupes.into_iter().filter(|g| !g.is_empty()) {
        let colonne = colonnes.last_mut().expect("au moins une colonne");
        let sous = if colonne.is_empty() { 0.0 } else { filet };
        let deja = hauteur_de(colonne, (rangee, filet));
        if !colonne.is_empty() && deja + sous + hauteur_de(groupe, (rangee, filet)) > hauteur {
            colonnes.push(Vec::new());
        } else if !colonne.is_empty() {
            colonne.push(None);
        }
        for def in groupe {
            let colonne = colonnes.last_mut().expect("au moins une colonne");
            if !colonne.is_empty() && hauteur_de(colonne, (rangee, filet)) + rangee > hauteur {
                colonnes.push(Vec::new());
            }
            colonnes
                .last_mut()
                .expect("au moins une colonne")
                .push(def.clone());
        }
    }
    colonnes
}
