//! **TIROIR-5 — les tiroirs se rangent dans l'écran, quoi qu'il arrive** (fiche 126).
//!
//! TIROIR-4 (fiche 122) range chaque côté en colonnes, chaque tiroir dans la première où il tient
//! — le « first fit » du rangement en boîtes (D. S. Johnson, 1974). Quand l'écran n'avait pas la
//! largeur de toutes les colonnes, la dernière restait contre l'autre bord, et deux tiroirs se
//! croisaient : Storyboard et Domaines de 10 pixels à 1440 × 900, avec huit domaines (fiche 122
//! § 7). Deux lois le règlent, et aucune ne choisit de nombre :
//!
//! * **La largeur** : tant que les colonnes des deux côtés ne tiennent pas dans l'écran, la
//!   dernière colonne du côté qui en a le plus — à égalité, celui qui prend le plus de largeur —
//!   se fond dans la moins pleine de son côté. Une seule colonne par côté, encore trop large,
//!   reste contre son bord : rien de plus ne se peut.
//! * **La hauteur** : une colonne plus haute que la place sous la bande la **partage
//!   équitablement** (le partage max-min) — chaque tiroir garde sa hauteur tant qu'elle tient, et
//!   seuls les plus hauts cèdent, jusqu'à une hauteur commune. Le minuteur reste entier à côté
//!   d'une Time Machine qui rétrécit. Ce qu'un tiroir ne montre plus, il le fait défiler.

use super::{hauteur_naturelle, Cote, DockManager, PanelLayoutBox, TabId, GRIP_HEIGHT};
use glucose_core::store::Store;

/// **Les panneaux qui savent se montrer dans un cadre court** : Domaines cache ses dernières
/// rangées et les fait défiler d'une par cran, « + Nouveau domaine » toujours visible (DOM-UI-2,
/// fiche 88) — mieux qu'un défilement au pixel. Rangé et partagé comme les autres, il reçoit son
/// cadre visible, et la molette lui revient ; les autres, qui débordaient, défilent (TIROIR-6).
const SE_MONTRENT_SEULS: [TabId; 1] = [TabId::Domains];

/// **La place de chaque tiroir ouvert** (TIROIR-4 et 5) : de chaque côté, collés au bord sous la
/// bande, empilés dans l'ordre où on les a ouverts ; une colonne a la largeur de son tiroir le
/// plus large, et quand la hauteur manque, le suivant ouvre une colonne voisine, vers
/// l'intérieur — tant que l'écran en a la largeur ; sinon la colonne partage sa hauteur, et
/// chaque tiroir fait défiler ce qu'il ne montre plus (TIROIR-6). Deux tiroirs ne se posent
/// jamais l'un sur l'autre (TIROIR-3), sauf sur un écran plus étroit qu'une colonne par côté ;
/// là, aucun ne sort de l'écran, et celui qui se dessine le dernier — à droite — est dessus, et
/// prend le clic ([`super::panneau_sous`]).
pub fn compute_panel_layouts(
    dock: &DockManager,
    store: &Store,
    screen_w: f32,
    screen_h: f32,
    header_h: f32,
    scale: f32,
) -> Vec<PanelLayoutBox> {
    let s = crate::theme::clamp_ui_scale(scale);
    let grip_height = GRIP_HEIGHT * s;
    let a_ranger = [Cote::Gauche, Cote::Droite].map(|cote| {
        dock.tabs(cote)
            .iter()
            .map(|&tab| Tiroir {
                tab,
                largeur: tab.default_width() * s,
                naturelle: hauteur_naturelle(dock, store, tab, s),
            })
            .collect()
    });
    let mut layouts: Vec<PanelLayoutBox> = Vec::new();
    let ranges = ranger(a_ranger, screen_w, screen_h - header_h);
    for (cote, colonnes) in [Cote::Gauche, Cote::Droite].into_iter().zip(ranges) {
        let mut bord = 0.0;
        for colonne in colonnes {
            let width = colonne.largeur;
            // Une colonne de trop pour la largeur reste dans l'écran, contre l'autre bord.
            let x = match cote {
                Cote::Gauche => bord,
                Cote::Droite => screen_w - bord - width,
            }
            .clamp(0.0, (screen_w - width).max(0.0));
            let mut y = header_h;
            for (tiroir, height) in colonne.tiroirs {
                let tab = tiroir.tab;
                let dragging = dock.drag.as_ref().filter(|d| d.tab == tab);
                let naturelle = if SE_MONTRENT_SEULS.contains(&tab) {
                    height
                } else {
                    tiroir.naturelle
                };
                layouts.push(PanelLayoutBox {
                    tab,
                    x,
                    y,
                    width,
                    height,
                    naturelle,
                    defile: dock.defile(tab).clamp(0.0, (naturelle - height).max(0.0)),
                    grip_y: y + height - grip_height,
                    grip_height,
                    is_dragged: dragging.is_some(),
                    visual_offset_x: dragging.map_or(0.0, |d| d.current_x - d.start_x),
                    visual_offset_y: dragging.map_or(0.0, |d| d.current_y - d.start_y),
                });
                y += height;
            }
            bord += width;
        }
    }
    layouts
}

/// **Un tiroir à ranger** : son onglet, sa largeur, sa hauteur naturelle (poignée comprise).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Tiroir {
    pub tab: TabId,
    pub largeur: f32,
    pub naturelle: f32,
}

/// **Une colonne rangée** : la largeur de son tiroir le plus large, et ses tiroirs de haut en
/// bas, chacun avec la hauteur qu'il montre.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Colonne {
    pub largeur: f32,
    pub tiroirs: Vec<(Tiroir, f32)>,
}

/// **Les colonnes des deux côtés** — la gauche puis la droite, chacune de son bord vers
/// l'intérieur — pour un écran de cette largeur et cette place sous la bande.
pub(super) fn ranger(cotes: [Vec<Tiroir>; 2], largeur: f32, place: f32) -> [Vec<Colonne>; 2] {
    let mut cotes = cotes.map(|tiroirs| premier_qui_tient(&tiroirs, place));
    tenir_dans_la_largeur(&mut cotes, largeur);
    cotes.map(|colonnes| {
        colonnes
            .into_iter()
            .map(|tiroirs| Colonne {
                largeur: tiroirs.iter().map(|t| t.largeur).fold(0.0, f32::max),
                tiroirs: tiroirs
                    .iter()
                    .copied()
                    .zip(partager_la_hauteur(&tiroirs, place))
                    .collect(),
            })
            .collect()
    })
}

/// **Le « first fit »** (TIROIR-4) : chaque tiroir, dans l'ordre où on l'a ouvert, dans la
/// première colonne où il tient ; aucune ne l'a : il ouvre la suivante. Un tiroir plus haut que la
/// place compte pour la place : il ne tiendrait sous aucun autre.
fn premier_qui_tient(tiroirs: &[Tiroir], place: f32) -> Vec<Vec<Tiroir>> {
    let mut colonnes: Vec<(f32, Vec<Tiroir>)> = Vec::new();
    for &tiroir in tiroirs {
        let hauteur = tiroir.naturelle.min(place);
        match colonnes
            .iter_mut()
            .find(|(haut, _)| haut + hauteur <= place)
        {
            Some((haut, colonne)) => {
                colonne.push(tiroir);
                *haut += hauteur;
            }
            None => colonnes.push((hauteur, vec![tiroir])),
        }
    }
    colonnes.into_iter().map(|(_, colonne)| colonne).collect()
}

fn largeur_de(colonne: &[Tiroir]) -> f32 {
    colonne.iter().map(|t| t.largeur).fold(0.0, f32::max)
}

/// **La largeur** : tant que les colonnes ne tiennent pas dans l'écran, la dernière du côté qui
/// en a le plus se fond dans la moins pleine de son côté — ses tiroirs s'y posent dessous, dans
/// leur ordre.
fn tenir_dans_la_largeur(cotes: &mut [Vec<Vec<Tiroir>>; 2], largeur: f32) {
    loop {
        let prise = |c: &Vec<Vec<Tiroir>>| c.iter().map(|col| largeur_de(col)).sum::<f32>();
        if prise(&cotes[0]) + prise(&cotes[1]) <= largeur {
            return;
        }
        let (n0, n1) = (cotes[0].len(), cotes[1].len());
        let i = if (n1, prise(&cotes[1])) > (n0, prise(&cotes[0])) {
            1
        } else {
            0
        };
        if cotes[i].len() < 2 {
            return;
        }
        let derniere = cotes[i].pop().unwrap_or_default();
        let pleine = |col: &Vec<Tiroir>| col.iter().map(|t| t.naturelle).sum::<f32>();
        if let Some(moins_pleine) = cotes[i]
            .iter_mut()
            .min_by(|a, b| pleine(a).total_cmp(&pleine(b)))
        {
            moins_pleine.extend(derniere);
        }
    }
}

/// **Le partage max-min de la place** entre les tiroirs d'une colonne : la hauteur que chacun
/// montre. Les plus bas d'abord : un tiroir qui tient dans sa part égale garde sa hauteur, et la
/// place qu'il laisse revient aux autres ; ceux qui restent ont tous la même part. La somme ne
/// dépasse jamais la place, et vaut toute la place dès qu'un tiroir cède.
fn partager_la_hauteur(colonne: &[Tiroir], place: f32) -> Vec<f32> {
    let mut hauteurs: Vec<f32> = colonne.iter().map(|t| t.naturelle).collect();
    let mut ordre: Vec<usize> = (0..colonne.len()).collect();
    ordre.sort_by(|&a, &b| hauteurs[a].total_cmp(&hauteurs[b]));
    let mut reste = place.max(0.0);
    for (k, &i) in ordre.iter().enumerate() {
        let part = reste / (ordre.len() - k) as f32;
        if hauteurs[i] <= part {
            reste -= hauteurs[i];
        } else {
            for &j in &ordre[k..] {
                hauteurs[j] = part;
            }
            break;
        }
    }
    hauteurs
}

#[cfg(test)]
#[path = "rangement/tests.rs"]
mod tests;
