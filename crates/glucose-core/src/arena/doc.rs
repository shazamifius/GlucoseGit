//! Le document complet sur l'arène : le tronc, le texte, et les attributs que peu portent.
//!
//! # Ce que c'est
//!
//! [`Arena`] range la géométrie, [`TextArena`] le texte, [`Sparse`] les attributs rares.
//! `Doc` les tient ensemble et garantit qu'ils parlent des mêmes nœuds : un seul endroit crée
//! et supprime, donc un seul endroit peut se tromper.
//!
//! C'est le modèle que la fiche 02 § 3.1 appelle « composition » : un tronc commun, et des
//! traits portés à côté par ceux qui les portent. L'ancien modèle l'exprimait par une
//! énumération à quatre variantes, où une carte de texte payait les vingt-cinq champs d'une
//! flèche — 44 % de son poids.
//!
//! # Les couleurs
//!
//! Le modèle historique range une couleur comme une `Option<String>` : 24 octets d'en-tête,
//! une allocation, plus les sept caractères de « #60a5fa ». Ici, une couleur reconnue devient
//! quatre octets. Celles qui ne le sont pas — une fonction CSS, un nom de couleur, ce qu'un
//! fichier venu d'ailleurs peut porter — vont dans une table de chaînes, à part : rien n'est
//! perdu, et le cas rare paie seul son coût.
//!
//! L'aller-retour rend la forme canonique `#rrggbb` ou `#rrggbbaa`. C'est une normalisation
//! assumée : « #ABC », « #aabbcc » et « #AABBCC » sont la même couleur et en ressortent
//! identiques.

use super::sparse::Sparse;
use super::text::TextArena;
use super::{Arena, Box2, Kind, NodeId};
use crate::fixed::Fx;
use crate::types::{
    ArrowPredicate, AssetRef, DomainAssignment, FolderMirrorSource, MembraneCurtain,
    MembraneMode, StickyOperator, TemporalAnchor, TextSelection,
};

/// Une couleur, quatre octets, canal alpha compris.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba(pub u32);

impl Rgba {
    /// Lit `#rgb`, `#rgba`, `#rrggbb` ou `#rrggbbaa`. Rend `None` pour tout le reste — un nom
    /// de couleur, une fonction CSS — qui ira dans la table des couleurs littérales.
    pub fn parse(s: &str) -> Option<Self> {
        let h = s.strip_prefix('#')?;
        let d = |c: u8| (c as char).to_digit(16);
        let v: Option<Vec<u32>> = h.bytes().map(d).collect();
        let v = v?;
        let (r, g, b, a) = match v.len() {
            3 => (v[0] * 17, v[1] * 17, v[2] * 17, 255),
            4 => (v[0] * 17, v[1] * 17, v[2] * 17, v[3] * 17),
            6 => (v[0] * 16 + v[1], v[2] * 16 + v[3], v[4] * 16 + v[5], 255),
            8 => (
                v[0] * 16 + v[1],
                v[2] * 16 + v[3],
                v[4] * 16 + v[5],
                v[6] * 16 + v[7],
            ),
            _ => return None,
        };
        Some(Self(r << 24 | g << 16 | b << 8 | a))
    }

    /// La forme canonique : `#rrggbb`, ou `#rrggbbaa` si la couleur n'est pas opaque.
    pub fn to_hex(self) -> String {
        let [r, g, b, a] = self.0.to_be_bytes();
        if a == 255 {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
        }
    }
}

/// Ce qu'une flèche porte en plus du tronc. Groupé parce qu'une flèche qui en porte un les
/// porte presque tous, et qu'une table par champ coûterait cinq identifiants au lieu d'un.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArrowTrait {
    pub predicate: Option<ArrowPredicate>,
    pub kind: Option<Box<str>>,
    pub stroke_width: Option<f32>,
    pub source: NodeId,
    pub target: NodeId,
    pub waypoints: Vec<(Fx, Fx)>,
    pub long_text: Option<Box<str>>,
    pub target_board: Option<Box<str>>,
    pub source_block: Option<Box<str>>,
    pub target_block: Option<Box<str>>,
    pub source_sel: Option<TextSelection>,
    pub target_sel: Option<TextSelection>,
}

/// Ce qu'une image porte en plus du tronc **et de ses dimensions d'origine**.
///
/// La première version y incluait aussi `original`, et la mesure l'a corrigée : cette structure
/// pèse 96 octets, dont quatre `Option<Box<str>>` presque toujours vides, alors que les
/// dimensions d'origine, elles, sont renseignées sur pratiquement toutes les images. Les poser
/// ensemble faisait payer 100 octets par image pour en utiliser 8 — 50 Mo sur un million
/// d'images, mesurés par `bench_arena`.
///
/// C'est exactement ce que la formule de [`super::sparse::prefer_sparse`] tranche : deux
/// attributs de fréquences très différentes ne vont pas dans la même table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageTrait {
    pub src: Option<Box<str>>,
    pub source_url: Option<Box<str>>,
    pub slot_id: Option<Box<str>>,
    pub fit: Option<Box<str>>,
    pub tags: Vec<Box<str>>,
}

impl ImageTrait {
    /// Vrai si rien n'est renseigné — auquel cas la table ne doit rien retenir.
    pub fn is_empty(&self) -> bool {
        self.src.is_none()
            && self.source_url.is_none()
            && self.slot_id.is_none()
            && self.fit.is_none()
            && self.tags.is_empty()
    }
}

/// Un document Glucose porté par l'arène.
///
/// Les champs sont publics : ce n'est pas un objet qui protège son état, c'est une disposition
/// mémoire. L'invariant qui compte — tout ce que les tables désignent est un nœud de l'arène —
/// est vérifié par [`Doc::check`].
#[derive(Debug, Clone, Default)]
pub struct Doc {
    /// La géométrie, le genre, les drapeaux, le parent.
    pub nodes: Arena,
    /// Le texte des cartes, notes, libellés de membrane et étiquettes de flèche.
    pub text: TextArena,

    /// Couleur de trait ou de texte, quand elle est reconnue.
    pub color: Sparse<Rgba>,
    /// Couleur de fond d'une note.
    pub background: Sparse<Rgba>,
    /// Couleur de trait non reconnue, conservée telle quelle plutôt que perdue.
    pub color_literal: Sparse<Box<str>>,
    /// Couleur de fond non reconnue. Une table à part : sans elle, un fond illisible
    /// écraserait la couleur de trait du même nœud, ce qui serait pire que de la perdre.
    pub background_literal: Sparse<Box<str>>,
    /// Taille de police, en pixels monde.
    pub font_size: Sparse<f32>,
    /// Rotation d'une image, en degrés.
    pub rotation: Sparse<f32>,
    /// Position du curseur dans le texte, pendant l'édition.
    pub cursor: Sparse<u32>,
    /// Fichier d'où le nœud a été importé.
    pub source_file: Sparse<Box<str>>,

    /// Le nœud dont celui-ci est le miroir.
    pub mirror_of: Sparse<NodeId>,
    /// La date du contenu décrit par le nœud.
    pub temporal: Sparse<TemporalAnchor>,
    /// Les pondérations de domaine.
    pub domains: Sparse<Vec<DomainAssignment>>,

    /// L'opérateur logique d'une note.
    pub operator: Sparse<StickyOperator>,
    /// Le mode d'affichage d'une membrane.
    pub membrane_mode: Sparse<MembraneMode>,
    /// Ce qu'une flèche porte en propre.
    pub arrow: Sparse<ArrowTrait>,
    /// Ce qu'une image porte en propre, quand elle porte quelque chose.
    pub image: Sparse<ImageTrait>,
    /// Les dimensions d'origine d'une image, en pixels — presque toujours renseignées, donc
    /// dans leur propre table plutôt que noyées dans [`ImageTrait`].
    pub original: Sparse<(f32, f32)>,
    /// Le tableau enfant d'un dossier, et son nom.
    pub folder: Sparse<(Box<str>, Box<str>)>,
    /// Le rang d'un panneau de storyboard.
    pub order: Sparse<i32>,
    /// L'actif binaire d'une image.
    pub asset: Sparse<AssetRef>,
    /// Les rideaux d'une membrane.
    pub curtains: Sparse<Vec<MembraneCurtain>>,
    /// La source d'un dossier miroir du système de fichiers.
    pub folder_mirror: Sparse<FolderMirrorSource>,
}

impl Doc {
    /// Un document vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Un document vide, dimensionné pour `n` nœuds.
    pub fn with_capacity(n: usize) -> Self {
        Self {
            nodes: Arena::with_capacity(n),
            ..Default::default()
        }
    }

    /// Pose un nœud et rend son identifiant.
    pub fn spawn(&mut self, kind: Kind, b: Box2, parent: NodeId) -> NodeId {
        self.nodes.spawn(kind, b, parent)
    }

    /// Le nombre de nœuds vivants.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Vrai si le document ne contient aucun nœud vivant.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Pose la couleur d'un nœud depuis une chaîne, reconnue ou non.
    ///
    /// Une couleur lisible va dans [`Doc::color`] pour quatre octets ; les autres sont
    /// conservées à la lettre dans [`Doc::color_literal`]. Les deux tables s'excluent, ce que
    /// [`Doc::check`] vérifie.
    pub fn set_color(&mut self, id: NodeId, s: &str) {
        self.color.remove(id);
        self.color_literal.remove(id);
        match Rgba::parse(s) {
            Some(c) => {
                self.color.set(id, c);
            }
            None => {
                self.color_literal.set(id, s.into());
            }
        }
    }

    /// La couleur d'un nœud, sous la forme où elle ressortira du document.
    pub fn color_of(&self, id: NodeId) -> Option<String> {
        if let Some(c) = self.color.get(id) {
            return Some(c.to_hex());
        }
        self.color_literal.get(id).map(|s| s.to_string())
    }

    /// Pose la couleur de fond d'un nœud depuis une chaîne, reconnue ou non.
    pub fn set_background(&mut self, id: NodeId, s: &str) {
        self.background.remove(id);
        self.background_literal.remove(id);
        match Rgba::parse(s) {
            Some(c) => {
                self.background.set(id, c);
            }
            None => {
                self.background_literal.set(id, s.into());
            }
        }
    }

    /// La couleur de fond d'un nœud, sous la forme où elle ressortira du document.
    pub fn background_of(&self, id: NodeId) -> Option<String> {
        if let Some(c) = self.background.get(id) {
            return Some(c.to_hex());
        }
        self.background_literal.get(id).map(|s| s.to_string())
    }

    /// Les octets du document, hors ce que les valeurs des tables allouent elles-mêmes.
    pub fn bytes(&self) -> usize {
        self.nodes.slots() * Arena::BYTES_PER_NODE
            + self.text.slots() * TextArena::BYTES_PER_NODE
            + self.text.bytes_used()
            + self.color.bytes()
            + self.background.bytes()
            + self.color_literal.bytes()
            + self.background_literal.bytes()
            + self.font_size.bytes()
            + self.rotation.bytes()
            + self.cursor.bytes()
            + self.source_file.bytes()
            + self.mirror_of.bytes()
            + self.temporal.bytes()
            + self.domains.bytes()
            + self.operator.bytes()
            + self.membrane_mode.bytes()
            + self.arrow.bytes()
            + self.image.bytes()
            + self.original.bytes()
            + self.folder.bytes()
            + self.order.bytes()
            + self.asset.bytes()
            + self.curtains.bytes()
            + self.folder_mirror.bytes()
    }

    /// Supprime un nœud. Ses attributs sont conservés : annuler la suppression doit rendre le
    /// nœud entier, pas sa carcasse.
    pub fn kill(&mut self, id: NodeId) -> bool {
        self.nodes.kill(id)
    }

    /// Vérifie l'invariant DOC-1 : tout nœud désigné par une table existe dans l'arène, aucun
    /// nœud ne porte deux formes de la même couleur, et les tables internes sont cohérentes.
    pub fn check(&self) -> Result<(), String> {
        self.nodes.check()?;
        self.text.check()?;

        let mut erreur = None;
        let mut verifier = |nom: &str, ids: Vec<NodeId>| {
            if erreur.is_some() {
                return;
            }
            for id in ids {
                if !self.nodes.holds(id) {
                    erreur = Some(format!(
                        "DOC-1 : la table {nom} désigne le nœud {} qui n'existe pas",
                        id.index()
                    ));
                    return;
                }
            }
        };
        verifier("color", self.color.iter().map(|(i, _)| i).collect());
        verifier("background", self.background.iter().map(|(i, _)| i).collect());
        verifier("color_literal", self.color_literal.iter().map(|(i, _)| i).collect());
        verifier(
            "background_literal",
            self.background_literal.iter().map(|(i, _)| i).collect(),
        );
        verifier("font_size", self.font_size.iter().map(|(i, _)| i).collect());
        verifier("rotation", self.rotation.iter().map(|(i, _)| i).collect());
        verifier("cursor", self.cursor.iter().map(|(i, _)| i).collect());
        verifier("source_file", self.source_file.iter().map(|(i, _)| i).collect());
        verifier("temporal", self.temporal.iter().map(|(i, _)| i).collect());
        verifier("domains", self.domains.iter().map(|(i, _)| i).collect());
        verifier("operator", self.operator.iter().map(|(i, _)| i).collect());
        verifier("membrane_mode", self.membrane_mode.iter().map(|(i, _)| i).collect());
        verifier("arrow", self.arrow.iter().map(|(i, _)| i).collect());
        verifier("image", self.image.iter().map(|(i, _)| i).collect());
        verifier("original", self.original.iter().map(|(i, _)| i).collect());
        verifier("folder", self.folder.iter().map(|(i, _)| i).collect());
        verifier("order", self.order.iter().map(|(i, _)| i).collect());
        verifier("asset", self.asset.iter().map(|(i, _)| i).collect());
        verifier("curtains", self.curtains.iter().map(|(i, _)| i).collect());
        verifier("folder_mirror", self.folder_mirror.iter().map(|(i, _)| i).collect());
        if let Some(e) = erreur {
            return Err(e);
        }

        // Les miroirs désignent des nœuds, et une table de références qui pointe dans le vide
        // est une carte fausse : mieux vaut le savoir au chargement qu'au premier clic.
        for (id, cible) in self.mirror_of.iter() {
            if !self.nodes.holds(id) || !self.nodes.holds(*cible) {
                return Err(format!(
                    "DOC-1 : le miroir {} → {} sort de l'arène",
                    id.index(),
                    cible.index()
                ));
            }
        }
        for (id, a) in self.arrow.iter() {
            for (nom, bout) in [("source", a.source), ("cible", a.target)] {
                if bout.is_some() && !self.nodes.holds(bout) {
                    return Err(format!(
                        "DOC-1 : la {nom} de la flèche {} sort de l'arène",
                        id.index()
                    ));
                }
            }
        }
        for id in self.color.iter().map(|(i, _)| i) {
            if self.color_literal.has(id) {
                return Err(format!(
                    "DOC-1 : le nœud {} porte deux formes de la même couleur",
                    id.index()
                ));
            }
        }
        for id in self.background.iter().map(|(i, _)| i) {
            if self.background_literal.has(id) {
                return Err(format!(
                    "DOC-1 : le nœud {} porte deux formes du même fond",
                    id.index()
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
