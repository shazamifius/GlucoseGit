//! **Le storyboard d'un tableau** (STORYBOARD-1, fiche 64 ; STORYBOARD-2, fiche 65) : une
//! **suite** de cases numérotées, rangées par une grille, une description sous chacune, et des
//! images ajustées dedans.
//!
//! # Ce que Glucose Tauri faisait, et ce qui lui manquait
//!
//! Son panneau posait une grille de `colonnes × 2` cases, centrée là où l'on cliquait (Glucose
//! la pose désormais par son coin haut gauche, FANTOME-1), et
//! « Mettre à jour » rangeait les cases existantes à la nouvelle grille ; chaque case portait un
//! numéro et une description. Mais une case ne pouvait **rien contenir** — son `imageId` n'était
//! jamais écrit —, ni se prendre, ni se déplacer (`StoryboardControls.tsx`, `StoryboardLayer.ts`).
//!
//! # Une suite, et une seule loi pour la ranger
//!
//! Boords renumérote seul quand on réordonne ; Storyboarder et Toon Boom tiennent une **suite**
//! de plans, pas des cadres épars. Ici, le storyboard est une suite rangée par sa grille
//! ([`Disposition`] : la grille, l'origine, le nombre) : la case de rang `i` est toujours à la
//! place `i` ([`Grille::coin`]). Poser d'un clic, tracer un rectangle, ajouter, retirer,
//! déplacer une case ne font que **changer la suite ou la disposition** — et [`Store::ranger`],
//! seul, en tire où va chaque case et ses images. Le numéro suit donc toujours l'ordre qu'on
//! voit, sans bouton à presser.
//!
//! # Ce qu'un rectangle tracé décide
//!
//! Sa demande (fiche 65) : *« le storyboard se remplit automatiquement, et crée ou supprime des
//! cases selon le rectangle dessiné »*. Le rectangle donne les colonnes et les rangées dont
//! l'étendue est la plus proche de la sienne ([`Grille::tient_dans`]) ; une case qui porte
//! quelque chose — une description, une image — ne se retire jamais ainsi : la grille
//! s'allonge plutôt.
//!
//! # Ce qui suit une case qui bouge
//!
//! Une image appartient à la case où son centre est posé, comme à une zone (PRESET-1). Quand
//! une case change de place ou de taille, ses images suivent par **la similitude qui envoie
//! l'ancien cadre dans le nouveau** — contenu, centré ([`suivre_le_cadre`]) : à taille égale,
//! une translation ; à taille changée, la composition de la case est gardée.

use super::journal::{Edit, Slot, Whole};
use super::Store;
use crate::types::{BoardImage, StoryboardPanel};

pub use crate::types::{Format, Grille, DESCRIPTION};

/// **Où vont les cases** : la grille qui les range, le coin de la première, et combien.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Disposition {
    pub grille: Grille,
    pub origine: (f64, f64),
    pub nombre: usize,
}

impl Disposition {
    /// **Le cadre de chaque case**, dans l'ordre : `(x, y, largeur, hauteur)`, bande non
    /// comprise. Le fantôme qui suit la main et le geste qui pose le lisent tous deux.
    pub fn cadres(&self) -> Vec<(f64, f64, f64, f64)> {
        let (l, h) = (self.grille.largeur, self.grille.hauteur());
        (0..self.nombre)
            .map(|i| {
                let (x, y) = self.grille.coin(self.origine, i);
                (x, y, l, h)
            })
            .collect()
    }

    /// **La boîte du storyboard** : `(x, y, largeur, hauteur)`, de la première case au bas de la
    /// dernière bande — son coin bas droit porte la poignée qui le retrace.
    pub fn boite(&self) -> (f64, f64, f64, f64) {
        let (largeur, hauteur) = self.grille.etendue(self.nombre);
        (self.origine.0, self.origine.1, largeur, hauteur)
    }

    /// **La place sous ce point** : la case de cette colonne et de cette rangée, bornée aux
    /// cases de la suite. Ce qui tombe hors de la grille va à la place la plus proche.
    pub fn rang_sous(&self, (x, y): (f64, f64)) -> usize {
        let (pas_x, pas_y) = self.grille.pas();
        let colonnes = self.grille.colonnes();
        let colonne = ((x - self.origine.0) / pas_x).floor().max(0.0) as usize;
        let rangee = ((y - self.origine.1) / pas_y).floor().max(0.0) as usize;
        (rangee * colonnes + colonne.min(colonnes - 1)).min(self.nombre.saturating_sub(1))
    }
}

/// **Où une image d'une case va quand sa case bouge** : la similitude qui envoie l'ancien
/// cadre dans le nouveau, contenu et centré — `k = min(l'/l, h'/h)`, le centre de l'image gardé
/// à sa place relative. À taille égale, `k = 1` : une translation exacte.
fn suivre_le_cadre(image: &mut BoardImage, ancien: &StoryboardPanel, neuf: &StoryboardPanel) {
    let k = (neuf.width / ancien.width).min(neuf.height / ancien.height);
    let centre = |c: &StoryboardPanel| (c.x + c.width / 2.0, c.y + c.height / 2.0);
    let (a, n) = (centre(ancien), centre(neuf));
    image.x = n.0 + (image.x - a.0) * k;
    image.y = n.1 + (image.y - a.1) * k;
    image.width *= k;
    image.height *= k;
}

/// Une boîte : sa gauche, son haut, sa largeur, sa hauteur.
pub type Boite = (f64, f64, f64, f64);

/// **L'ordre de lecture** de ces boîtes — des cases, ou des images qu'on lâche : par rangée, de
/// haut en bas, puis de gauche à droite. Une seule loi pour tout ce qui se lit (P2, fiche 66).
///
/// Une rangée commence à la plus haute des boîtes restantes, et prend toutes celles dont le
/// milieu tombe dans sa hauteur : une case glissée un peu plus bas que ses voisines reste dans
/// leur rangée. Rend les rangs dans la liste, dans l'ordre.
pub fn ordre_de_lecture(boites: &[Boite]) -> Vec<usize> {
    let mut restantes: Vec<usize> = (0..boites.len()).collect();
    restantes.sort_by(|&a, &b| boites[a].1.total_cmp(&boites[b].1));
    let mut ordre = Vec::with_capacity(boites.len());
    while let Some(&premiere) = restantes.first() {
        let bas = boites[premiere].1 + boites[premiere].3;
        let (mut rangee, reste): (Vec<usize>, Vec<usize>) = restantes
            .into_iter()
            .partition(|&i| boites[i].1 + boites[i].3 / 2.0 <= bas);
        rangee.sort_by(|&a, &b| boites[a].0.total_cmp(&boites[b].0));
        ordre.extend(rangee);
        restantes = reste;
    }
    ordre
}

impl Store {
    /// Les cases de ce tableau, dans l'ordre de leurs numéros.
    pub fn cases(&self, board_id: &str) -> Vec<&StoryboardPanel> {
        let mut cases: Vec<&StoryboardPanel> = self
            .project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .map(|b| b.panels.iter().collect())
            .unwrap_or_default();
        cases.sort_by_key(|c| c.order);
        cases
    }

    /// La grille du storyboard de ce tableau, s'il en a une.
    pub fn grille_du_storyboard(&self, board_id: &str) -> Option<Grille> {
        self.project
            .boards
            .iter()
            .find(|b| b.id == board_id)?
            .storyboard
    }

    /// **Combien de cases un rangement doit garder** : jusqu'à la dernière qui porte quelque
    /// chose — une description, ou une image. Zéro si aucune.
    pub fn cases_a_garder(&self, board_id: &str) -> usize {
        self.cases(board_id)
            .iter()
            .rposition(|c| {
                !c.description.trim().is_empty()
                    || !self.images_de_la_case(board_id, &c.id).is_empty()
            })
            .map_or(0, |rang| rang + 1)
    }

    /// **La disposition du storyboard tel qu'il est** : sa grille (ou `reglee`, pour des cases
    /// d'avant qu'elle ne s'enregistre), le coin de sa première case, son nombre de cases.
    pub fn disposition_actuelle(&self, board_id: &str, reglee: Grille) -> Option<Disposition> {
        let cases = self.cases(board_id);
        let premiere = cases.first()?;
        Some(Disposition {
            grille: self.grille_du_storyboard(board_id).unwrap_or(reglee),
            origine: (premiere.x, premiere.y),
            nombre: cases.len(),
        })
    }

    /// **La disposition d'un clic** : deux rangées sur un tableau qui n'en a pas, sinon autant
    /// que de cases — le « Mettre à jour » de Tauri —, **le coin haut gauche de la première sous
    /// le clic** (FANTOME-1, fiche 66).
    ///
    /// Tauri centrait la grille sur le clic, et le tracé, lui, part du point d'appui : le
    /// fantôme qu'on survolait sautait d'une demi-grille dès que la main commençait à glisser.
    /// Sa demande du 13/10 : *« toujours en haut à gauche de la souris »*. Le fantôme, le clic
    /// qui pose et le glisser qui commence partent désormais du même point.
    pub fn disposition_au_clic(
        &self,
        board_id: &str,
        grille: Grille,
        coin: (f64, f64),
    ) -> Disposition {
        let n = self.cases(board_id).len();
        let nombre = if n == 0 { grille.nombre_initial() } else { n };
        Disposition {
            grille,
            origine: coin,
            nombre,
        }
    }

    /// **La disposition d'un rectangle tracé de `depart` à `point`** : les colonnes et les
    /// rangées les plus proches de lui, ancrées au coin d'où la main est partie ; jamais moins
    /// de cases que [`Self::cases_a_garder`] — la grille s'allonge plutôt.
    pub fn disposition_du_trace(
        &self,
        board_id: &str,
        reglee: Grille,
        (depart, point): ((f64, f64), (f64, f64)),
    ) -> Disposition {
        let taille = ((point.0 - depart.0).abs(), (point.1 - depart.1).abs());
        let (colonnes, rangees) = reglee.tient_dans(taille);
        let grille = Grille { colonnes, ..reglee };
        let nombre = (colonnes * rangees).max(self.cases_a_garder(board_id));
        let (largeur, hauteur) = grille.etendue(nombre);
        let vers = |d: f64, p: f64, l: f64| if p < d { d - l } else { d };
        Disposition {
            grille,
            origine: (
                vers(depart.0, point.0, largeur),
                vers(depart.1, point.1, hauteur),
            ),
            nombre,
        }
    }

    /// **Range le storyboard à cette disposition** : ses cases dans leur ordre, les neuves à la
    /// suite, celles de trop retirées — un geste.
    pub fn disposer(&mut self, board_id: &str, d: Disposition) {
        let mut suite: Vec<StoryboardPanel> = self.cases(board_id).into_iter().cloned().collect();
        suite.truncate(d.nombre);
        while suite.len() < d.nombre {
            let id = self.generate_id("case");
            let (x, y) = d.grille.coin(d.origine, suite.len());
            let hauteur = d.grille.hauteur();
            suite.push(StoryboardPanel::new(id, 0, x, y, d.grille.largeur, hauteur));
        }
        self.ranger(board_id, d, suite, None);
    }

    /// **Pose le storyboard d'un clic, sa première case au `coin`** ([`Self::disposition_au_clic`]).
    pub fn poser_le_storyboard(&mut self, board_id: &str, grille: Grille, coin: (f64, f64)) {
        let d = self.disposition_au_clic(board_id, grille, coin);
        self.disposer(board_id, d);
    }

    /// **Ajoute une case après la dernière** — le « + Ajouter » de Tauri. Sans storyboard, rien.
    pub fn ajouter_une_case(&mut self, board_id: &str, reglee: Grille) -> Option<String> {
        let mut d = self.disposition_actuelle(board_id, reglee)?;
        d.nombre += 1;
        self.disposer(board_id, d);
        self.cases(board_id).last().map(|c| c.id.clone())
    }

    /// **Retire une case** : les suivantes avancent d'une place, leurs images avec elles, et la
    /// suite reste continue. Retirer la dernière retire le storyboard. Un geste.
    pub fn retirer_la_case(&mut self, board_id: &str, id: &str) {
        let Some(mut d) = self.disposition_actuelle(board_id, Grille::default()) else {
            return;
        };
        let suite: Vec<StoryboardPanel> = self
            .cases(board_id)
            .into_iter()
            .filter(|c| c.id != id)
            .cloned()
            .collect();
        if suite.is_empty() {
            self.effacer_le_storyboard(board_id);
            return;
        }
        d.nombre = suite.len();
        self.ranger(board_id, d, suite, None);
    }

    /// **Déplace une case à la place `rang`** : celles d'entre les deux places glissent d'un
    /// cran, et chacune emporte ses images. `origine` est le coin de la première place au début
    /// du geste — la case qu'on tient a pu être celle-là ; `portees`, les images qu'elle
    /// emportait au départ — lâchée sur une autre case, elle la recouvre, et ce qu'elle porte
    /// ne se lit plus à la place. Dans le geste ouvert, s'il y en a un.
    pub fn deplacer_la_case(
        &mut self,
        board_id: &str,
        (id, portees): (&str, &[String]),
        rang: usize,
        origine: (f64, f64),
    ) {
        let Some(mut d) = self.disposition_actuelle(board_id, Grille::default()) else {
            return;
        };
        let mut suite: Vec<StoryboardPanel> = self.cases(board_id).into_iter().cloned().collect();
        let Some(depuis) = suite.iter().position(|c| c.id == id) else {
            return;
        };
        let tenue = suite.remove(depuis);
        suite.insert(rang.min(suite.len()), tenue);
        d.origine = origine;
        self.ranger(board_id, d, suite, Some((id, portees)));
    }

    /// **Efface le storyboard** : toutes les cases et sa grille, en un geste. Les images restent.
    pub fn effacer_le_storyboard(&mut self, board_id: &str) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let avant = std::mem::take(&mut b.panels);
        let grille = b.storyboard.take();
        let mut edits = diff_des_cases(board_id, &avant, &[]);
        edits.push(Edit::Storyboard {
            board: board_id.to_string(),
            whole: Whole::new(grille, None),
        });
        self.record_as_one_gesture(edits);
    }

    /// **Renumérote les cases dans l'ordre de lecture**, et les range à leur nouvelle place
    /// ([`ordre_de_lecture`]) — ce qui remet en suite un storyboard venu d'ailleurs, aux cases
    /// éparses. Sur une suite déjà rangée, rien ne change. Un geste.
    pub fn renumeroter_les_cases(&mut self, board_id: &str) {
        let Some(mut d) = self.disposition_actuelle(board_id, Grille::default()) else {
            return;
        };
        let cases: Vec<StoryboardPanel> = self.cases(board_id).into_iter().cloned().collect();
        let boites: Vec<Boite> = cases
            .iter()
            .map(|c| (c.x, c.y, c.width, c.height))
            .collect();
        let suite: Vec<StoryboardPanel> = ordre_de_lecture(&boites)
            .into_iter()
            .map(|i| cases[i].clone())
            .collect();
        d.origine = (suite[0].x, suite[0].y);
        self.ranger(board_id, d, suite, None);
    }

    /// **La suite, rangée à la disposition** : chaque case à sa place et à sa taille, numérotée
    /// de un en un ; ses images la suivent ([`suivre_le_cadre`]) ; la grille s'enregistre. Ce
    /// qui change entre au journal en un geste — dans le geste ouvert, s'il y en a un.
    ///
    /// `tenue` : la case qu'une main a portée, et les images qu'elle emportait — les siennes,
    /// quoi qu'elle recouvre ; les autres images cherchent leur case sans elle.
    fn ranger(
        &mut self,
        board_id: &str,
        d: Disposition,
        suite: Vec<StoryboardPanel>,
        tenue: Option<(&str, &[String])>,
    ) {
        let neuves: Vec<StoryboardPanel> = suite
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let (x, y) = d.grille.coin(d.origine, i);
                let (width, height) = (d.grille.largeur, d.grille.hauteur());
                StoryboardPanel {
                    order: i as i32,
                    x,
                    y,
                    width,
                    height,
                    ..c.clone()
                }
            })
            .collect();
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        // Chaque image suit la première case de la suite où son centre est posé — lue à sa
        // place d'avant, une seule fois —, sauf celles que la case tenue emportait.
        let case_de = |image: &BoardImage| match tenue {
            Some((id, emportees)) if emportees.contains(&image.id) => {
                suite.iter().position(|c| c.id == id)
            }
            _ => suite.iter().position(|c| {
                tenue.is_none_or(|(id, _)| c.id != id) && c.contient((image.x, image.y))
            }),
        };
        let mut edits = Vec::new();
        for (k, image) in b.images.iter_mut().enumerate() {
            let Some(rang) = case_de(image) else {
                continue;
            };
            if image.locked {
                continue;
            }
            let avant = image.clone();
            suivre_le_cadre(image, &suite[rang], &neuves[rang]);
            edits.push(Edit::Image {
                board: board_id.to_string(),
                slot: Slot::changed(k, avant, image.clone()),
            });
        }
        let avant = std::mem::replace(&mut b.panels, neuves);
        edits.extend(diff_des_cases(board_id, &avant, &b.panels));
        let grille = b.storyboard.replace(d.grille);
        edits.push(Edit::Storyboard {
            board: board_id.to_string(),
            whole: Whole::new(grille, Some(d.grille)),
        });
        self.record_as_one_gesture(edits);
    }

    /// **Pose cette case à ce coin** — pendant un glisser, dans le geste ouvert.
    pub fn placer_la_case(&mut self, board_id: &str, id: &str, (x, y): (f64, f64)) {
        let mut cases: Vec<StoryboardPanel> = self.cases(board_id).into_iter().cloned().collect();
        if let Some(c) = cases.iter_mut().find(|c| c.id == id) {
            (c.x, c.y) = (x, y);
        }
        self.remplacer_les_cases(board_id, cases);
    }

    /// **Glisse une case et les images qu'elle porte** à `total` de leur départ, dans le geste
    /// ouvert — en deux éditions, quelle que soit la durée du geste : la case, **réécrite à sa
    /// place** dans le geste (FONDRE-1 ne fond que des éditions voisines, et les pas des images
    /// les séparent), puis ses images par le glisser (GLISSER-1), dont les pas deviennent une
    /// translation à la fin. Défaire rend les images, puis la case. Hors geste, rien.
    pub fn glisser_la_case(
        &mut self,
        board_id: &str,
        (id, depart): (&str, (f64, f64)),
        images: &[String],
        total: (f64, f64),
    ) {
        let Some((_, edits)) = self.journal.en_cours() else {
            return;
        };
        let deja = edits.iter().enumerate().find_map(|(rang, e)| match e {
            Edit::Panel { board, slot } if board == board_id => slot
                .after
                .as_ref()
                .filter(|c| c.id == id)
                .and(slot.before.as_deref().cloned())
                .map(|avant| (rang, avant)),
            _ => None,
        });
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let Some(index) = b.panels.iter().position(|c| c.id == id) else {
            return;
        };
        let actuelle = b.panels[index].clone();
        let mut neuve = actuelle.clone();
        (neuve.x, neuve.y) = (depart.0 + total.0, depart.1 + total.1);
        b.panels[index] = neuve.clone();
        let board = board_id.to_string();
        match deja {
            Some((rang, avant)) => {
                let slot = Slot::changed(index, avant, neuve);
                let edit = Edit::Panel { board, slot };
                self.journal
                    .remplacer_dans_le_geste(rang..rang + 1, Some(edit));
            }
            None => {
                let slot = Slot::changed(index, actuelle, neuve);
                self.journal.record(Edit::Panel { board, slot });
            }
        }
        self.glisser_ces_images(board_id, images, total);
    }

    /// **Les images que cette case porte** : celles dont le centre tombe dans son cadre.
    pub fn images_de_la_case(&self, board_id: &str, id: &str) -> Vec<String> {
        let Some(b) = self.project.boards.iter().find(|b| b.id == board_id) else {
            return Vec::new();
        };
        let Some(c) = b.panels.iter().find(|c| c.id == id) else {
            return Vec::new();
        };
        b.images
            .iter()
            .filter(|i| c.contient((i.x, i.y)))
            .map(|i| i.id.clone())
            .collect()
    }

    /// **La case dont le cadre contient ce point** — la plus haute, la dernière posée, si deux se
    /// recouvrent. La description, sous le cadre, n'en est pas.
    pub fn case_sous(&self, board_id: &str, point: (f64, f64)) -> Option<&StoryboardPanel> {
        self.cases(board_id)
            .into_iter()
            .rev()
            .find(|c| c.contient(point))
    }

    /// **Remplace les cases du tableau**, et consigne chaque case qui change, entre, ou part —
    /// le tout en un geste.
    fn remplacer_les_cases(&mut self, board_id: &str, neuves: Vec<StoryboardPanel>) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let avant = std::mem::replace(&mut b.panels, neuves);
        let apres = b.panels.clone();
        let edits = diff_des_cases(board_id, &avant, &apres);
        self.record_as_one_gesture(edits);
    }
}

/// **Ce qui change d'une liste de cases à l'autre**, en éditions qui se rejouent dans l'ordre.
///
/// Quand la liste garde ses cases aux mêmes rangs — un glisser, une description, un rangement —
/// seules celles qui changent sont consignées : un glisser n'écrit pas toutes les cases à chaque
/// image. Sinon, on retire l'ancien de la fin vers le début, puis on insère le neuf.
fn diff_des_cases(board: &str, avant: &[StoryboardPanel], apres: &[StoryboardPanel]) -> Vec<Edit> {
    let memes_rangs =
        avant.len() == apres.len() && avant.iter().zip(apres).all(|(a, b)| a.id == b.id);
    if memes_rangs {
        return avant
            .iter()
            .zip(apres)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (a, b))| Edit::Panel {
                board: board.to_string(),
                slot: Slot::changed(i, a.clone(), b.clone()),
            })
            .collect();
    }
    let retraits = avant.iter().enumerate().rev().map(|(i, c)| Edit::Panel {
        board: board.to_string(),
        slot: Slot::removed(i, c.clone()),
    });
    let ajouts = apres.iter().enumerate().map(|(i, c)| Edit::Panel {
        board: board.to_string(),
        slot: Slot::inserted(i, c.clone()),
    });
    retraits.chain(ajouts).collect()
}

mod remplir;

#[cfg(test)]
mod tests;
