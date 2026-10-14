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

use super::journal::{Edit, Slot};
use super::Store;
use crate::types::{BoardImage, StoryboardPanel};

pub use crate::types::{Format, Grille, Plan, Storyboard, DESCRIPTION};

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

    /// **Les planches de ce storyboard** (P2, proposition 7, fiche 68) : des rectangles du
    /// monde de même taille, l'un sous l'autre — la planche `p` porte les rangées
    /// `[p·R, (p+1)·R)`, toutes les colonnes de la grille, cadres et bandes, bordées d'un écart
    /// tout autour : la marge d'une planche est l'espace entre deux cases.
    ///
    /// **`R` se déduit du format A** (ISO 216) : une feuille dont le grand côté vaut √2 fois le
    /// petit, la seule qui garde sa proportion pliée en deux. La planche à l'italienne de `C`
    /// colonnes mesure `C·p_x + e` de large ; `R` est le nombre de rangées dont la hauteur
    /// `R·p_y + e` est la plus proche de cette largeur sur √2 — au moins une, au plus les
    /// rangées du storyboard : plus court qu'une feuille, il tient sur une planche à sa mesure.
    /// Aucun nombre choisi : imprimée, une planche remplit sa feuille, A4 comme A3.
    pub fn planches(&self) -> Vec<Boite> {
        let (pas_x, pas_y) = self.grille.pas();
        let e = self.grille.ecart;
        let colonnes = self.grille.colonnes();
        let largeur = colonnes as f64 * pas_x + e;
        let rangees = self.nombre.max(1).div_ceil(colonnes);
        let format_a = ((largeur / std::f64::consts::SQRT_2 - e) / pas_y)
            .round()
            .max(1.0) as usize;
        // Un storyboard plus court qu'une feuille ne garde pas de rangées vides (vu sur
        // l'aperçu : deux rangées, une planche de quatre).
        let par_planche = format_a.min(rangees);
        let hauteur = par_planche as f64 * pas_y + e;
        (0..rangees.div_ceil(par_planche))
            .map(|p| {
                let y = self.origine.1 + (p * par_planche) as f64 * pas_y - e;
                (self.origine.0 - e, y, largeur, hauteur)
            })
            .collect()
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
    /// **Les storyboards de ce tableau**, dans l'ordre où ils sont nés (fiche 68).
    pub fn storyboards(&self, board_id: &str) -> &[Storyboard] {
        self.project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .map_or(&[], |b| b.storyboards.as_slice())
    }

    /// Ce storyboard de ce tableau, s'il existe.
    pub fn storyboard(&self, board_id: &str, sb: &str) -> Option<&Storyboard> {
        self.storyboards(board_id).iter().find(|s| s.id == sb)
    }

    /// **Le storyboard de cette case.**
    pub fn storyboard_de_la_case(&self, board_id: &str, case: &str) -> Option<&Storyboard> {
        let c = self
            .toutes_les_cases(board_id)
            .iter()
            .find(|c| c.id == case)?;
        self.storyboard(board_id, &c.storyboard)
    }

    /// **Toutes les cases du tableau**, de tous ses storyboards — ce que le dessin et le clic
    /// parcourent.
    pub fn toutes_les_cases(&self, board_id: &str) -> &[StoryboardPanel] {
        self.project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .map_or(&[], |b| b.panels.as_slice())
    }

    /// Les cases de ce storyboard, dans l'ordre de leurs numéros.
    pub fn cases(&self, board_id: &str, sb: &str) -> Vec<&StoryboardPanel> {
        let mut cases: Vec<&StoryboardPanel> = self
            .toutes_les_cases(board_id)
            .iter()
            .filter(|c| c.storyboard == sb)
            .collect();
        cases.sort_by_key(|c| c.order);
        cases
    }

    /// La grille de ce storyboard, s'il existe.
    pub fn grille_du_storyboard(&self, board_id: &str, sb: &str) -> Option<Grille> {
        self.storyboard(board_id, sb).map(|s| s.grille)
    }

    /// **Combien de cases un rangement doit garder** : jusqu'à la dernière qui porte quelque
    /// chose — ce qu'on y a écrit ou choisi ([`StoryboardPanel::porte_quelque_chose`]), ou une
    /// image. Zéro si aucune, ou pour un storyboard qui n'existe pas encore.
    pub fn cases_a_garder(&self, board_id: &str, sb: Option<&str>) -> usize {
        let Some(sb) = sb else {
            return 0;
        };
        self.cases(board_id, sb)
            .iter()
            .rposition(|c| {
                c.porte_quelque_chose() || !self.images_de_la_case(board_id, &c.id).is_empty()
            })
            .map_or(0, |rang| rang + 1)
    }

    /// **La disposition de ce storyboard tel qu'il est** : sa grille, le coin de sa première
    /// case, son nombre de cases.
    pub fn disposition_actuelle(&self, board_id: &str, sb: &str) -> Option<Disposition> {
        let grille = self.grille_du_storyboard(board_id, sb)?;
        let cases = self.cases(board_id, sb);
        let premiere = cases.first()?;
        Some(Disposition {
            grille,
            origine: (premiere.x, premiere.y),
            nombre: cases.len(),
        })
    }

    /// **La disposition d'un clic** : deux rangées pour un storyboard qui naît (`cible` vide),
    /// sinon autant que les cases de la cible — le « Mettre à jour » de Tauri —, **le coin haut
    /// gauche de la première sous le clic** (FANTOME-1, fiche 66).
    ///
    /// Tauri centrait la grille sur le clic, et le tracé, lui, part du point d'appui : le
    /// fantôme qu'on survolait sautait d'une demi-grille dès que la main commençait à glisser.
    /// Sa demande du 13/10 : *« toujours en haut à gauche de la souris »*. Le fantôme, le clic
    /// qui pose et le glisser qui commence partent désormais du même point.
    pub fn disposition_au_clic(
        &self,
        (board_id, cible): (&str, Option<&str>),
        grille: Grille,
        coin: (f64, f64),
    ) -> Disposition {
        let n = cible.map_or(0, |sb| self.cases(board_id, sb).len());
        let nombre = if n == 0 { grille.nombre_initial() } else { n };
        Disposition {
            grille,
            origine: coin,
            nombre,
        }
    }

    /// **La disposition d'un rectangle tracé de `depart` à `point`** : les colonnes et les
    /// rangées les plus proches de lui, ancrées au coin d'où la main est partie ; jamais moins
    /// de cases que la cible n'en doit garder ([`Self::cases_a_garder`]) — la grille s'allonge
    /// plutôt.
    pub fn disposition_du_trace(
        &self,
        (board_id, cible): (&str, Option<&str>),
        reglee: Grille,
        (depart, point): ((f64, f64), (f64, f64)),
    ) -> Disposition {
        let taille = ((point.0 - depart.0).abs(), (point.1 - depart.1).abs());
        let (colonnes, rangees) = reglee.tient_dans(taille);
        let grille = Grille { colonnes, ..reglee };
        let nombre = (colonnes * rangees).max(self.cases_a_garder(board_id, cible));
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

    /// **Range la cible à cette disposition** — ou, sans cible, **fait naître un storyboard**
    /// (fiche 68) : ses cases dans leur ordre, les neuves à la suite, celles de trop retirées.
    /// Un geste. Rend le storyboard rangé.
    pub fn disposer(
        &mut self,
        (board_id, cible): (&str, Option<&str>),
        d: Disposition,
    ) -> Option<String> {
        self.project.boards.iter().find(|b| b.id == board_id)?;
        let sb = match cible {
            Some(sb) => sb.to_string(),
            None => self.generate_id("storyboard"),
        };
        let mut suite: Vec<StoryboardPanel> =
            self.cases(board_id, &sb).into_iter().cloned().collect();
        suite.truncate(d.nombre);
        while suite.len() < d.nombre {
            let id = self.generate_id("case");
            let coin = d.grille.coin(d.origine, suite.len());
            let taille = (d.grille.largeur, d.grille.hauteur());
            suite.push(StoryboardPanel::new(id, sb.as_str(), 0, coin, taille));
        }
        self.ranger((board_id, &sb), d, suite, None);
        Some(sb)
    }

    /// **Pose un storyboard neuf d'un clic, sa première case au `coin`** — à côté de ceux qui
    /// sont déjà là (fiche 68). Rend le storyboard posé.
    pub fn poser_le_storyboard(
        &mut self,
        board_id: &str,
        grille: Grille,
        coin: (f64, f64),
    ) -> Option<String> {
        let d = self.disposition_au_clic((board_id, None), grille, coin);
        self.disposer((board_id, None), d)
    }

    /// **Ajoute une case après la dernière de ce storyboard** — le « + Ajouter » de Tauri.
    pub fn ajouter_une_case(&mut self, board_id: &str, sb: &str) -> Option<String> {
        let mut d = self.disposition_actuelle(board_id, sb)?;
        d.nombre += 1;
        self.disposer((board_id, Some(sb)), d);
        self.cases(board_id, sb).last().map(|c| c.id.clone())
    }

    /// **Insère une case vide à la place `rang` de ce storyboard** (P2, proposition 3, fiche
    /// 66) : celles qui suivent glissent d'un cran, chacune avec ses images. Aucune loi neuve :
    /// une case ajoutée au bout, puis lâchée à sa place — les deux gestes qui existent —, en un
    /// seul geste. Rend la case neuve.
    pub fn inserer_une_case(&mut self, board_id: &str, sb: &str, rang: usize) -> Option<String> {
        let origine = self.disposition_actuelle(board_id, sb)?.origine;
        let ouvert = self.in_live_edit();
        if !ouvert {
            self.begin_live_edit();
        }
        let neuve = self.ajouter_une_case(board_id, sb);
        if let Some(id) = &neuve {
            self.deplacer_la_case(board_id, (id, &[]), rang, origine);
        }
        if !ouvert {
            self.end_live_edit();
        }
        neuve
    }

    /// **Retire une case** : les suivantes de son storyboard avancent d'une place, leurs images
    /// avec elles, et la suite reste continue. Retirer la dernière retire le storyboard. Un
    /// geste.
    pub fn retirer_la_case(&mut self, board_id: &str, id: &str) {
        let Some(sb) = self
            .storyboard_de_la_case(board_id, id)
            .map(|s| s.id.clone())
        else {
            return;
        };
        let Some(mut d) = self.disposition_actuelle(board_id, &sb) else {
            return;
        };
        let suite: Vec<StoryboardPanel> = self
            .cases(board_id, &sb)
            .into_iter()
            .filter(|c| c.id != id)
            .cloned()
            .collect();
        if suite.is_empty() {
            self.effacer_le_storyboard(board_id, &sb);
            return;
        }
        d.nombre = suite.len();
        self.ranger((board_id, &sb), d, suite, None);
    }

    /// **Déplace une case à la place `rang` de son storyboard** : celles d'entre les deux places
    /// glissent d'un cran, et chacune emporte ses images. `origine` est le coin de la première
    /// place au début du geste — la case qu'on tient a pu être celle-là ; `portees`, les images
    /// qu'elle emportait au départ — lâchée sur une autre case, elle la recouvre, et ce qu'elle
    /// porte ne se lit plus à la place. Dans le geste ouvert, s'il y en a un.
    pub fn deplacer_la_case(
        &mut self,
        board_id: &str,
        (id, portees): (&str, &[String]),
        rang: usize,
        origine: (f64, f64),
    ) {
        let Some(sb) = self
            .storyboard_de_la_case(board_id, id)
            .map(|s| s.id.clone())
        else {
            return;
        };
        let Some(mut d) = self.disposition_actuelle(board_id, &sb) else {
            return;
        };
        let mut suite: Vec<StoryboardPanel> =
            self.cases(board_id, &sb).into_iter().cloned().collect();
        let Some(depuis) = suite.iter().position(|c| c.id == id) else {
            return;
        };
        let tenue = suite.remove(depuis);
        suite.insert(rang.min(suite.len()), tenue);
        d.origine = origine;
        self.ranger((board_id, &sb), d, suite, Some((id, portees)));
    }

    /// **Efface ce storyboard** : toutes ses cases, puis lui, en un geste. Les images restent,
    /// et les autres storyboards aussi.
    pub fn effacer_le_storyboard(&mut self, board_id: &str, sb: &str) {
        let Some(b) = self.project.boards.iter_mut().find(|b| b.id == board_id) else {
            return;
        };
        let Some(i) = b.storyboards.iter().position(|s| s.id == sb) else {
            return;
        };
        let mut edits = ranger::remplacer_la_suite(board_id, &mut b.panels, sb, Vec::new());
        let parti = b.storyboards.remove(i);
        edits.push(Edit::Storyboard {
            board: board_id.to_string(),
            slot: Slot::removed(i, parti),
        });
        self.record_as_one_gesture(edits);
    }

    /// **Renumérote les cases de ce storyboard dans l'ordre de lecture**, et les range à leur
    /// nouvelle place ([`ordre_de_lecture`]) — ce qui remet en suite un storyboard venu
    /// d'ailleurs, aux cases éparses. Sur une suite déjà rangée, rien ne change. Un geste.
    pub fn renumeroter_les_cases(&mut self, board_id: &str, sb: &str) {
        let Some(mut d) = self.disposition_actuelle(board_id, sb) else {
            return;
        };
        let cases: Vec<StoryboardPanel> = self.cases(board_id, sb).into_iter().cloned().collect();
        let boites: Vec<Boite> = cases
            .iter()
            .map(|c| (c.x, c.y, c.width, c.height))
            .collect();
        let suite: Vec<StoryboardPanel> = ordre_de_lecture(&boites)
            .into_iter()
            .map(|i| cases[i].clone())
            .collect();
        d.origine = (suite[0].x, suite[0].y);
        self.ranger((board_id, sb), d, suite, None);
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
        self.toutes_les_cases(board_id)
            .iter()
            .rev()
            .find(|c| c.contient(point))
    }
}

mod case;
mod ranger;
mod remplir;

pub use case::Texte;

#[cfg(test)]
mod tests;
