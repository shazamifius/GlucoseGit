//! **Ce qu'un agent fait au tableau** — lire, poser, écrire, relier, déplacer, retirer
//! (fiche 75).
//!
//! # Pourquoi dans le noyau
//!
//! Un agent — Protéo, Claude, n'importe quel programme qui parle MCP — agit sur le document
//! **comme une main de plus**. Ce qu'il a le droit de faire est une question sur le document,
//! pas sur la façon dont la demande arrive : le transport (la porte, le protocole) vit dans
//! l'application ; ce qu'un appel change vit ici, éprouvé sans écran.
//!
//! # AGENT-1 — un appel d'agent est un geste
//!
//! Tout ce qu'un appel écrit forme **une** entrée du journal ([`Store::geste_d_agent`]) : un
//! `Ctrl+Z` de l'utilisateur le défait d'un coup, l'histoire du fichier l'enregistre comme le
//! reste. Et l'appel est **atomique** : s'il échoue en route (un passage introuvable), le
//! document revient exactement à l'état d'avant — rien à moitié.
//!
//! # AGENT-2 — un agent ne touche jamais la sélection de l'utilisateur
//!
//! Poser un nœud le sélectionne, parce qu'un nœud qu'on vient de poser à la main est celui
//! qu'on veut écrire. Un agent qui travaille pendant qu'on travaille volerait la sélection à
//! chaque appel. La sélection est donc gardée autour du geste, et rendue — sans les nœuds que
//! l'agent a retirés.
//!
//! # AGENT-3 — un passage cité est, mot pour mot, dans le nœud
//!
//! Une flèche qui part d'une phrase ([`TextAnchor`], FLECHE-4) la désigne par sa place exacte.
//! L'agent la donne par son texte : il doit s'y trouver tel quel, sinon l'appel est refusé et
//! dit pourquoi. C'est la garantie qui rend le rapprochement vérifiable : ce que la flèche
//! surligne est ce que l'agent a lu.
//!
//! # AGENT-4 — poser un identifiant qui existe le met à jour
//!
//! Un appel répété — un agent qui retente après un délai dépassé — ne double rien : poser sous
//! un identifiant déjà porté par un nœud de même sorte le remplace. Le serveur MCP de tldraw
//! ne le garantit pas, et ses écritures retentées dupliquent les formes.

use super::Store;
use crate::geometry::Rect;
use crate::text_anchors::create_anchor;
use crate::types::{Annotation, ArrowPredicate, BoardImage, TextAnchor, TextSelection};

/// L'écart entre deux nœuds qu'un agent pose l'un après l'autre : celui du rangement des zones,
/// pour qu'un tableau rempli par un agent se lise comme un tableau rangé par Glucose.
const ECART: f64 = super::zones::ECART;

/// La place que le titre d'une membrane occupe au-dessus de son contenu : la bande de ses
/// zones, plus son écart.
const TITRE_DE_MEMBRANE: f64 = super::zones::ECART + super::zones::MARGE;

/// La sorte d'un nœud, dans les mots qu'un agent lit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sorte {
    Texte,
    PostIt,
    Membrane,
    Fleche,
    Photo,
}

impl Sorte {
    pub fn nom(self) -> &'static str {
        match self {
            Self::Texte => "texte",
            Self::PostIt => "post_it",
            Self::Membrane => "membrane",
            Self::Fleche => "fleche",
            Self::Photo => "photo",
        }
    }

    fn de(a: &Annotation) -> Self {
        match a {
            Annotation::Text { .. } => Self::Texte,
            Annotation::Sticky { .. } => Self::PostIt,
            Annotation::Membrane { .. } => Self::Membrane,
            Annotation::Arrow { .. } => Self::Fleche,
        }
    }
}

/// **Un nœud, tel qu'un agent le lit** : ce qu'il est, où il est, ce qu'il dit.
#[derive(Debug, Clone, PartialEq)]
pub struct Decrit {
    pub id: String,
    pub sorte: Sorte,
    pub boite: Rect,
    pub texte: Option<String>,
    pub membrane: Option<String>,
    /// Pour une flèche : ce qu'elle relie, et la relation qu'elle dit.
    pub source: Option<String>,
    pub cible: Option<String>,
    pub relation: Option<ArrowPredicate>,
    /// Pour une photo : l'adresse d'où elle vient — la page qu'elle montre.
    pub adresse: Option<String>,
}

/// **Pourquoi un appel n'a rien fait**, dans des mots qu'un agent peut corriger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refus {
    TableauInconnu(String),
    NoeudInconnu(String),
    AutreSorte {
        id: String,
        est: &'static str,
    },
    /// L'identifiant est déjà porté ailleurs — sur un autre tableau, ou par un dossier.
    Pris(String),
    PassageAbsent {
        noeud: String,
        passage: String,
    },
    /// L'utilisateur est au milieu d'un geste : l'appel attend qu'il finisse.
    GesteEnCours,
}

impl std::fmt::Display for Refus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TableauInconnu(t) => write!(f, "aucun tableau ne porte l'identifiant « {t} »"),
            Self::NoeudInconnu(n) => write!(f, "aucun nœud ne porte l'identifiant « {n} »"),
            Self::AutreSorte { id, est } => {
                write!(f, "« {id} » est déjà un nœud d'une autre sorte ({est})")
            }
            Self::Pris(id) => write!(f, "l'identifiant « {id} » est déjà pris ailleurs"),
            Self::PassageAbsent { noeud, passage } => write!(
                f,
                "le passage « {passage} » n'est pas, mot pour mot, dans le nœud « {noeud} »"
            ),
            Self::GesteEnCours => write!(f, "l'utilisateur est au milieu d'un geste"),
        }
    }
}

/// **Une flèche qu'un agent tend** entre deux nœuds, et les passages qu'elle désigne.
#[derive(Debug, Clone, Copy, Default)]
pub struct Lien<'a> {
    pub id: &'a str,
    pub source: &'a str,
    pub cible: &'a str,
    pub passage_source: Option<&'a str>,
    pub passage_cible: Option<&'a str>,
    pub relation: Option<ArrowPredicate>,
    pub texte: Option<&'a str>,
}

impl Store {
    /// **Un appel d'agent, en un geste** (AGENT-1, AGENT-2) : tout ce que `f` écrit forme une
    /// entrée du journal, ou rien du tout si `f` refuse ; la sélection de l'utilisateur est la
    /// même après, moins ce qui a disparu.
    ///
    /// Refusé pendant un geste de l'utilisateur : ce que l'agent écrirait entrerait dans son
    /// geste à lui, et son `Ctrl+Z` défairait les deux.
    pub fn geste_d_agent<R>(
        &mut self,
        f: impl FnOnce(&mut Store) -> Result<R, Refus>,
    ) -> Result<R, Refus> {
        if self.in_live_edit() {
            return Err(Refus::GesteEnCours);
        }
        let selection = (
            self.selected_image_ids.clone(),
            self.selected_annotation_ids.clone(),
            self.selected_folder_id.clone(),
        );
        self.begin_live_edit();
        let resultat = f(self);
        if resultat.is_ok() {
            self.end_live_edit();
        } else {
            self.cancel_live_edit();
        }
        let (images, annotations, dossier) = selection;
        self.selected_image_ids = images.into_iter().filter(|i| self.id_exists(i)).collect();
        self.selected_annotation_ids = annotations
            .into_iter()
            .filter(|a| self.id_exists(a))
            .collect();
        self.selected_folder_id = dossier.filter(|d| self.id_exists(d));
        resultat
    }

    /// **Le tableau tel qu'un agent le lit** : chaque photo et chaque annotation, avec sa boîte
    /// et son texte.
    pub fn decrire_pour_un_agent(&self, board_id: &str) -> Result<Vec<Decrit>, Refus> {
        let board = self
            .project
            .boards
            .iter()
            .find(|b| b.id == board_id)
            .ok_or_else(|| Refus::TableauInconnu(board_id.to_string()))?;
        let photos = board.images.iter().map(|i| Decrit {
            id: i.id.clone(),
            sorte: Sorte::Photo,
            boite: i.rect(),
            texte: None,
            membrane: i.membrane_id.clone(),
            source: None,
            cible: None,
            relation: None,
            adresse: i.source_url.clone(),
        });
        let annotations = board.annotations.iter().map(decrire_une_annotation);
        Ok(photos.chain(annotations).collect())
    }

    /// **Pose une annotation**, ou met à jour celle qui porte déjà son identifiant (AGENT-4).
    ///
    /// Une mise à jour garde l'appartenance du nœud, et écrit son texte par
    /// [`Store::ecrire_le_texte`] : les flèches ancrées à un passage le suivent.
    pub fn poser_pour_un_agent(&mut self, board_id: &str, ann: Annotation) -> Result<(), Refus> {
        self.tableau_existe(board_id)?;
        let id = ann.id().to_string();
        let Some(ancienne) = self.project.annotation(board_id, &id).cloned() else {
            self.identifiant_libre(board_id, &id)?;
            self.add_annotation(board_id, ann);
            return Ok(());
        };
        if Sorte::de(&ancienne) != Sorte::de(&ann) {
            let est = Sorte::de(&ancienne).nom();
            return Err(Refus::AutreSorte { id, est });
        }
        // Le texte s'écrit d'abord, par la porte qui fait suivre les ancres des flèches ; le
        // reste du nœud est ensuite remplacé, son texte étant déjà le bon.
        if let Some(texte) = texte_porte(&ann) {
            self.ecrire_le_texte(board_id, &id, &texte);
        }
        let membrane = ancienne.membrane_id().map(str::to_string);
        self.update_annotation(board_id, &id, |a| {
            *a = ann;
            a.set_membrane_id(membrane);
        });
        Ok(())
    }

    /// **Pose une photo**, ou met à jour celle qui porte déjà son identifiant (AGENT-4).
    pub fn poser_une_photo_pour_un_agent(
        &mut self,
        board_id: &str,
        img: BoardImage,
    ) -> Result<(), Refus> {
        self.tableau_existe(board_id)?;
        let id = img.id.clone();
        if self.image(board_id, &id).is_none() {
            self.identifiant_libre(board_id, &id)?;
            self.add_image(board_id, img);
            return Ok(());
        }
        self.update_image(board_id, &id, |i| {
            let membrane = i.membrane_id.take();
            *i = img;
            i.membrane_id = membrane;
        });
        Ok(())
    }

    /// **Écrit le texte d'un nœud** : le remplace, ou l'allonge — un document qu'un agent
    /// remplit au fil de sa recherche.
    pub fn ecrire_pour_un_agent(
        &mut self,
        board_id: &str,
        id: &str,
        texte: &str,
        ajouter: bool,
    ) -> Result<(), Refus> {
        let ancien = self.texte_du_noeud(board_id, id)?;
        let nouveau = if ajouter {
            format!("{ancien}{texte}")
        } else {
            texte.to_string()
        };
        self.ecrire_le_texte(board_id, id, &nouveau);
        Ok(())
    }

    /// **Tend une flèche** de `source` à `cible`, partie du passage cité de l'une et arrivée à
    /// celui de l'autre (AGENT-3) ; ou met à jour celle qui porte déjà cet identifiant.
    pub fn relier_pour_un_agent(&mut self, board_id: &str, lien: Lien<'_>) -> Result<(), Refus> {
        let (depart, arrivee) = (
            self.boite_d_un_noeud(board_id, lien.source)?,
            self.boite_d_un_noeud(board_id, lien.cible)?,
        );
        let source_sel = self.passage(board_id, lien.source, lien.passage_source)?;
        let cible_sel = self.passage(board_id, lien.cible, lien.passage_cible)?;
        let (d, a) = (depart.center(), arrivee.center());
        let mut fleche = Annotation::arrow(lien.id, d.x, d.y, a.x, a.y);
        if let Annotation::Arrow {
            source_id,
            target_id,
            source_text_sel,
            target_text_sel,
            predicate,
            text,
            ..
        } = &mut fleche
        {
            *source_id = Some(lien.source.to_string());
            *target_id = Some(lien.cible.to_string());
            *source_text_sel = source_sel;
            *target_text_sel = cible_sel;
            *predicate = lien.relation;
            *text = lien.texte.map(str::to_string);
        }
        self.poser_pour_un_agent(board_id, fleche)
    }

    /// **Déplace un nœud** pour que le coin haut gauche de sa boîte soit en `(x, y)` ; il
    /// appartient ensuite à la membrane où il tombe (MEMB-1).
    pub fn deplacer_pour_un_agent(
        &mut self,
        board_id: &str,
        id: &str,
        (x, y): (f64, f64),
    ) -> Result<(), Refus> {
        let boite = self.boite_d_un_noeud(board_id, id)?;
        let (dx, dy) = (x - boite.left, y - boite.top);
        if self.image(board_id, id).is_some() {
            self.update_image(board_id, id, |i| {
                i.x += dx;
                i.y += dy;
            });
        } else {
            self.update_annotation(board_id, id, |a| a.translate(dx, dy));
        }
        // Le rattachement lit la sélection : elle désigne le nœud le temps de le rattacher, et
        // `geste_d_agent` rend celle de l'utilisateur ensuite (AGENT-2).
        if self.image(board_id, id).is_some() {
            self.select_image(id.to_string(), false);
        } else {
            self.select_annotation(id.to_string(), false);
        }
        self.rattacher_la_selection(board_id);
        Ok(())
    }

    /// **Retire des nœuds**, et avec eux les flèches qui n'ont plus de bout. Rend combien de
    /// nœuds nommés existaient.
    pub fn retirer_pour_un_agent(&mut self, board_id: &str, ids: &[&str]) -> Result<usize, Refus> {
        self.tableau_existe(board_id)?;
        let existants: Vec<&str> = ids
            .iter()
            .copied()
            .filter(|i| {
                self.image(board_id, i).is_some() || self.project.annotation(board_id, i).is_some()
            })
            .collect();
        self.remove_images(board_id, &existants);
        self.remove_annotations(board_id, &existants);
        Ok(existants.len())
    }

    /// **Où poser le prochain nœud** : sous le contenu de la membrane `dans`, ou à droite de
    /// tout le tableau — pour qu'un agent qui ne dit pas où poser n'empile jamais un nœud sur
    /// un autre.
    pub fn place_pour_un_agent(
        &self,
        board_id: &str,
        dans: Option<&str>,
    ) -> Result<(f64, f64), Refus> {
        let decrits = self.decrire_pour_un_agent(board_id)?;
        let Some(m) = dans else {
            let tout = decrits
                .iter()
                .filter(|d| d.sorte != Sorte::Fleche)
                .map(|d| d.boite)
                .reduce(|a, b| a.union(b));
            return Ok(tout.map_or((0.0, 0.0), |b| (b.right() + ECART, b.top)));
        };
        let cadre = self.boite_d_un_noeud(board_id, m)?;
        let contenu = decrits
            .iter()
            .filter(|d| d.membrane.as_deref() == Some(m) && d.sorte != Sorte::Fleche)
            .map(|d| d.boite)
            .reduce(|a, b| a.union(b));
        Ok(match contenu {
            Some(c) => (c.left, c.bottom() + ECART),
            None => (cadre.left + ECART, cadre.top + TITRE_DE_MEMBRANE),
        })
    }

    /// **La membrane grandit pour contenir `boite`**, avec la marge de ses bords — à faire
    /// *avant* d'y poser un nœud : il lui appartient alors dès sa naissance (MEMB-1).
    pub fn agrandir_la_membrane_pour(
        &mut self,
        board_id: &str,
        membrane: &str,
        boite: Rect,
    ) -> Result<(), Refus> {
        let cadre = self.boite_d_un_noeud(board_id, membrane)?;
        let voulu = Rect::new(
            boite.left - ECART,
            boite.top - TITRE_DE_MEMBRANE,
            boite.width + 2.0 * ECART,
            boite.height + TITRE_DE_MEMBRANE + ECART,
        );
        let neuf = cadre.union(voulu);
        if neuf == cadre {
            return Ok(());
        }
        self.update_annotation(board_id, membrane, |a| {
            if let Annotation::Membrane {
                x,
                y,
                width,
                height,
                ..
            } = a
            {
                (*x, *y, *width, *height) = (neuf.left, neuf.top, neuf.width, neuf.height);
            }
        });
        Ok(())
    }

    /// Un identifiant neuf ne doit être porté par rien d'autre, sur aucun tableau.
    fn identifiant_libre(&self, board_id: &str, id: &str) -> Result<(), Refus> {
        if let Some(a) = self.project.annotation(board_id, id) {
            let est = Sorte::de(a).nom();
            return Err(Refus::AutreSorte {
                id: id.to_string(),
                est,
            });
        }
        if self.image(board_id, id).is_some() {
            let est = Sorte::Photo.nom();
            return Err(Refus::AutreSorte {
                id: id.to_string(),
                est,
            });
        }
        if self.id_exists(id) {
            return Err(Refus::Pris(id.to_string()));
        }
        Ok(())
    }

    /// **Un texte prend la hauteur de son contenu** (TEXT-FIT-1) — que l'application mesure,
    /// le noyau ne connaissant pas les polices — et la membrane qui le contient grandit avec
    /// lui : un document qu'un agent allonge ne déborde jamais de sa bulle.
    pub fn tenir_la_hauteur_pour_un_agent(
        &mut self,
        board_id: &str,
        id: &str,
        hauteur: f64,
    ) -> Result<(), Refus> {
        let Some(a) = self.project.annotation(board_id, id) else {
            return Err(Refus::NoeudInconnu(id.to_string()));
        };
        let (Annotation::Text { height, .. }, Some(boite)) = (a, a.rect()) else {
            return Ok(());
        };
        let membrane = a.membrane_id().map(str::to_string);
        if *height != Some(hauteur) {
            self.update_annotation(board_id, id, |a| {
                if let Annotation::Text { height, .. } = a {
                    *height = Some(hauteur);
                }
            });
        }
        match membrane {
            Some(m) => {
                let boite = Rect::new(boite.left, boite.top, boite.width, hauteur);
                self.agrandir_la_membrane_pour(board_id, &m, boite)
            }
            None => Ok(()),
        }
    }

    fn tableau_existe(&self, board_id: &str) -> Result<(), Refus> {
        self.project
            .boards
            .iter()
            .any(|b| b.id == board_id)
            .then_some(())
            .ok_or_else(|| Refus::TableauInconnu(board_id.to_string()))
    }

    fn boite_d_un_noeud(&self, board_id: &str, id: &str) -> Result<Rect, Refus> {
        if let Some(i) = self.image(board_id, id) {
            return Ok(i.rect());
        }
        self.tableau_existe(board_id)?;
        self.project
            .annotation(board_id, id)
            .map(Annotation::bounds)
            .ok_or_else(|| Refus::NoeudInconnu(id.to_string()))
    }

    fn texte_du_noeud(&self, board_id: &str, id: &str) -> Result<String, Refus> {
        self.tableau_existe(board_id)?;
        match self.project.annotation(board_id, id) {
            Some(a) => Ok(a.own_text().unwrap_or_default()),
            None if self.id_exists(id) => Err(Refus::AutreSorte {
                id: id.to_string(),
                est: "photo",
            }),
            None => Err(Refus::NoeudInconnu(id.to_string())),
        }
    }

    /// Le passage cité, en ancre — ou rien s'il n'y en a pas (AGENT-3).
    fn passage(
        &self,
        board_id: &str,
        id: &str,
        cite: Option<&str>,
    ) -> Result<Option<TextSelection>, Refus> {
        let Some(cite) = cite.filter(|c| !c.trim().is_empty()) else {
            return Ok(None);
        };
        let texte = self.texte_du_noeud(board_id, id)?;
        let ancre: Option<TextAnchor> = texte
            .find(cite)
            .and_then(|debut| create_anchor(&texte, debut, debut + cite.len()));
        let ancre = ancre.ok_or_else(|| Refus::PassageAbsent {
            noeud: id.to_string(),
            passage: cite.to_string(),
        })?;
        Ok(Some(TextSelection::Anchors(vec![ancre])))
    }
}

/// Le texte qu'une annotation porte — celui d'une flèche compris, que [`Annotation::own_text`]
/// ne rend pas.
fn texte_porte(a: &Annotation) -> Option<String> {
    match a {
        Annotation::Arrow { text, .. } => text.clone(),
        _ => a.own_text(),
    }
}

fn decrire_une_annotation(a: &Annotation) -> Decrit {
    let (source, cible, relation) = match a {
        Annotation::Arrow {
            source_id,
            target_id,
            predicate,
            ..
        } => (source_id.clone(), target_id.clone(), *predicate),
        _ => (None, None, None),
    };
    let texte = texte_porte(a);
    Decrit {
        id: a.id().to_string(),
        sorte: Sorte::de(a),
        boite: a.bounds(),
        texte,
        membrane: a.membrane_id().map(str::to_string),
        source,
        cible,
        relation,
        adresse: None,
    }
}

#[cfg(test)]
mod tests;
