//! **Transformer la sélection entière**, dans le geste ouvert (fiche 53 § 10) : la pose de
//! départ de chaque nœud se relève à l'appui, et chaque mouvement réécrit tout depuis elle.
//!
//! Ce qui se transforme est ce qu'un déplacement emporte ([`Store::ce_qu_emporte_la_selection`]) :
//! les images non verrouillées, les annotations, le dossier — et ce que les membranes possèdent
//! (MEMB-1). Une flèche choisie se transforme en entier ; un bout de flèche accroché à un nœud
//! transformé suit la transformation **de ce nœud**, comme il suit sa translation (JRN-4).
//!
//! Ce qui ne tourne pas — une carte, un pense-bête, une membrane, un dossier — voit son centre
//! tourner avec le groupe, et garde son angle droit : le modèle ne lui en donne pas.

use super::Store;
use crate::geometry::Rect;
use crate::groupe::{deplacer_le_point, transformer, Origine, Pose, Transformation};
use crate::types::Annotation;

/// Un nœud de la sélection, et ce qu'il était à l'appui.
#[derive(Debug, Clone, PartialEq)]
pub enum Depart {
    Image {
        id: String,
        pose: Pose,
    },
    /// Une carte, un pense-bête, une membrane.
    Boite {
        id: String,
        pose: Pose,
    },
    Dossier {
        id: String,
        pose: Pose,
    },
    /// Une flèche : ses points (les deux bouts, puis les points de passage), et ce qui décide
    /// de chacun de ses deux bouts — elle-même si elle est choisie, le nœud auquel il
    /// s'accroche sinon.
    Fleche {
        id: String,
        points: Vec<(f64, f64)>,
        centre: (f64, f64),
        choisie: bool,
        bouts: [Option<String>; 2],
    },
}

/// La pose d'une boîte droite.
fn pose_de(r: Rect) -> Pose {
    Pose {
        centre: (r.left + r.width / 2.0, r.top + r.height / 2.0),
        taille: (r.width, r.height),
        rotation: 0.0,
    }
}

/// La boîte droite d'une pose.
fn boite_de(p: Pose) -> Rect {
    Rect::new(
        p.centre.0 - p.taille.0 / 2.0,
        p.centre.1 - p.taille.1 / 2.0,
        p.taille.0,
        p.taille.1,
    )
}

/// **La transformation qu'un nœud reçoit** : celle du groupe, ou la même autour de son centre.
fn pour_le_noeud(t: Transformation, origine: Origine, centre: (f64, f64)) -> Transformation {
    match (origine, t) {
        (Origine::Commune, t) => t,
        (Origine::Individuelle, Transformation::Echelle { facteur, .. }) => {
            Transformation::Echelle {
                facteur,
                ancre: centre,
            }
        }
        (Origine::Individuelle, Transformation::Rotation { angle, .. }) => {
            Transformation::Rotation {
                angle,
                pivot: centre,
            }
        }
    }
}

/// La dernière réponse de [`Store::emprise_du_groupe`], et ce qui la rend valable :
/// `(version du document, tableau, empreinte de la sélection)`.
#[derive(Debug, Clone, Default)]
pub(super) struct CadreGarde {
    repere: Option<(u64, String, u64)>,
    valeur: Option<Rect>,
}

impl Store {
    /// **Le cadre du groupe** : l'emprise de la sélection quand elle compte deux nœuds ou plus
    /// qui se transforment — sinon rien (fiche 53 § 10). La même réponse que l'arbitre de clic
    /// ([`crate::hit_priority::emprise_du_groupe`]), gardée tant que ni le document ni la
    /// sélection ne changent : le dessin la demande à chaque image, et la calculer parcourt le
    /// tableau.
    pub fn emprise_du_groupe(&self, board_id: &str) -> Option<Rect> {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        (
            &self.selected_image_ids,
            &self.selected_annotation_ids,
            &self.selected_folder_id,
        )
            .hash(&mut h);
        let repere = (self.version, board_id.to_string(), h.finish());
        let mut garde = self.groupe.borrow_mut();
        if garde.repere.as_ref() != Some(&repere) {
            let b = self.project.boards.iter().find(|b| b.id == board_id)?;
            let entree = crate::hit_priority::PickInput {
                wx: 0.0,
                wy: 0.0,
                scale: 1.0,
                images: &b.images,
                annotations: &b.annotations,
                folders: &b.folders,
                selected_image_ids: &self.selected_image_ids,
                selected_annotation_ids: &self.selected_annotation_ids,
                selected_folder_id: self.selected_folder_id.as_deref(),
                noeuds: None,
            };
            garde.valeur = crate::hit_priority::emprise_du_groupe(&entree);
            garde.repere = Some(repere);
        }
        garde.valeur
    }

    /// **La pose de départ de tout ce que la sélection transforme** — à relever à l'appui.
    pub fn depart_de_la_selection(&self, board_id: &str) -> Vec<Depart> {
        let emport = self.ce_qu_emporte_la_selection(board_id);
        let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) else {
            return Vec::new();
        };
        let mut departs: Vec<Depart> = b
            .images
            .iter()
            .filter(|i| emport.images.contains(&i.id) && !i.locked)
            .map(|i| Depart::Image {
                id: i.id.clone(),
                pose: Pose {
                    centre: (i.x, i.y),
                    taille: (i.width, i.height),
                    rotation: i.rotation,
                },
            })
            .collect();
        departs.extend(
            b.folders
                .iter()
                .filter(|f| self.selected_folder_id.as_deref() == Some(f.id.as_str()))
                .map(|f| Depart::Dossier {
                    id: f.id.clone(),
                    pose: pose_de(f.rect()),
                }),
        );
        let transformes = |id: &Option<String>| -> Option<String> {
            id.clone()
                .filter(|id| emport.images.contains(id) || emport.annotations.contains(id))
        };
        for a in &b.annotations {
            let choisie = emport.annotations.contains(a.id());
            match a {
                Annotation::Arrow {
                    x,
                    y,
                    x2,
                    y2,
                    waypoints,
                    source_id,
                    target_id,
                    ..
                } => {
                    let bouts = [transformes(source_id), transformes(target_id)];
                    if !choisie && bouts.iter().all(Option::is_none) {
                        continue;
                    }
                    let mut points = vec![(*x, *y), (*x2, *y2)];
                    points.extend(waypoints.iter().map(|p| (p.x, p.y)));
                    let r = a.bounds();
                    departs.push(Depart::Fleche {
                        id: a.id().to_string(),
                        points,
                        centre: (r.left + r.width / 2.0, r.top + r.height / 2.0),
                        choisie,
                        bouts,
                    });
                }
                _ if choisie => {
                    if let Some(r) = a.rect() {
                        departs.push(Depart::Boite {
                            id: a.id().to_string(),
                            pose: pose_de(r),
                        });
                    }
                }
                _ => {}
            }
        }
        departs
    }

    /// **Écrit la sélection transformée** depuis sa pose de départ, dans le geste ouvert : la
    /// même transformation du groupe, autour de son origine ou de celle de chaque nœud.
    pub fn transformer_la_selection(
        &mut self,
        board_id: &str,
        departs: &[Depart],
        t: Transformation,
        origine: Origine,
    ) {
        for depart in departs {
            match depart {
                Depart::Image { id, pose } => {
                    let p = transformer(*pose, t, origine);
                    self.update_image(board_id, id, |i| {
                        (i.x, i.y) = p.centre;
                        (i.width, i.height) = p.taille;
                        i.rotation = crate::rotate::normalize(p.rotation);
                    });
                }
                Depart::Boite { id, pose } => {
                    let p = transformer(*pose, t, origine);
                    self.set_annotation_rect(board_id, id, boite_de(p));
                }
                Depart::Dossier { id, pose } => {
                    let p = transformer(*pose, t, origine);
                    self.set_folder_rect(board_id, id, boite_de(p));
                }
                Depart::Fleche { id, .. } => {
                    let nouveaux = points_de_la_fleche(depart, departs, t, origine);
                    self.update_annotation(board_id, id, |a| {
                        if let Annotation::Arrow {
                            x,
                            y,
                            x2,
                            y2,
                            waypoints,
                            ..
                        } = a
                        {
                            (*x, *y) = nouveaux[0];
                            (*x2, *y2) = nouveaux[1];
                            for (w, p) in waypoints.iter_mut().zip(&nouveaux[2..]) {
                                (w.x, w.y) = *p;
                            }
                        }
                    });
                }
            }
        }
    }
}

/// Le centre de départ du nœud `id`, pour les bouts de flèche qui s'y accrochent.
fn centre_de(departs: &[Depart], id: &str) -> Option<(f64, f64)> {
    departs.iter().find_map(|d| match d {
        Depart::Image { id: i, pose }
        | Depart::Boite { id: i, pose }
        | Depart::Dossier { id: i, pose }
            if i == id =>
        {
            Some(pose.centre)
        }
        _ => None,
    })
}

/// **Les points d'une flèche après la transformation** : tous, si elle est choisie ; sinon ses
/// seuls bouts accrochés à un nœud transformé, chacun par la transformation de ce nœud.
fn points_de_la_fleche(
    fleche: &Depart,
    departs: &[Depart],
    t: Transformation,
    origine: Origine,
) -> Vec<(f64, f64)> {
    let Depart::Fleche {
        points,
        centre,
        choisie,
        bouts,
        ..
    } = fleche
    else {
        return Vec::new();
    };
    let propre = pour_le_noeud(t, origine, *centre);
    let du_bout = |k: usize| -> Option<Transformation> {
        if *choisie {
            return Some(propre);
        }
        let noeud = bouts[k].as_deref()?;
        Some(pour_le_noeud(t, origine, centre_de(departs, noeud)?))
    };
    points
        .iter()
        .enumerate()
        .map(|(k, p)| match (k, du_bout(k.min(1))) {
            (0 | 1, Some(tb)) => deplacer_le_point(tb, *p),
            (2.., Some(_)) if *choisie => deplacer_le_point(propre, *p),
            _ => *p,
        })
        .collect()
}
