//! L'arène des nœuds : tableaux de champs, identifiants entiers — `std` uniquement.
//!
//! # Ce que la mesure reprochait au modèle précédent
//!
//! Trois chiffres, pris sur l'arbre de travail avant d'écrire une ligne de ce module :
//!
//! | Constat | Chiffre | Cause |
//! |---|---:|---|
//! | Une `Annotation` texte gaspille presque la moitié de sa place | **224 o sur 512 (44 %)** | L'énumération donne à chaque variante la taille de la plus grosse — `Arrow` et ses 25 champs. Une `Membrane` en gaspille 52 %. |
//! | Retrouver un nœud par identifiant, sur un million | **5,9 ms** | Balayage linéaire d'un `Vec<Annotation>`, avec comparaison de `String`. Un clic coûte plus qu'une demi-douzaine de frames. |
//! | Le modèle, sur un million de nœuds | **498 Mo** (522 o/nœud) | Padding, plus trois à huit allocations de tas par nœud. |
//!
//! L'arène répond aux trois par construction :
//!
//! - **Composition au lieu d'énumération** : tous les nœuds partagent un tronc de champs, et
//!   ce qui est propre à un genre vit dans une table à part, qui ne contient que les nœuds
//!   concernés. C'est le « modèle en composition » de la fiche 02 § 3.1, enfin appliqué.
//! - **Tableaux de champs** (*SoA*) : le culling ne lit que les quatre coordonnées, sur quatre
//!   tableaux contigus. Il ne saute plus de 512 en 512 octets à travers des champs dont il n'a
//!   que faire. Bénéfice second et gratuit : **plus aucun remplissage d'alignement par nœud**,
//!   puisqu'il n'y a plus de structure par nœud.
//! - **[`NodeId`] est un indice** : retrouver un nœud est un accès tableau, pas une recherche.
//!
//! # Le tronc, et pourquoi une flèche y entre sans exception
//!
//! ```text
//!     x, y, w, h : Fx  →  16 octets      genre : Kind      →  1 octet
//!     parent     : NodeId  →  4 octets   drapeaux : Flags  →  1 octet      = 22 octets
//! ```
//!
//! Une flèche n'a pourtant pas de largeur : elle a un second point. La tentation est de lui
//! donner ses propres champs `x2, y2` — c'est ce que faisait l'ancien modèle, et c'est ce qui
//! obligeait chaque culling, chaque sélection élastique et chaque minimap à recalculer sa boîte
//! englobante au passage. On range donc **la boîte englobante dans le tronc commun**, et le
//! coin d'où part la flèche dans deux drapeaux : la source est sur le bord droit ou le bord
//! gauche ([`Flags::ARROW_FROM_RIGHT`]), en haut ou en bas ([`Flags::ARROW_FROM_BOTTOM`]).
//!
//! > La première version de ce module n'en gardait **qu'un**, sur l'idée que « deux points
//! > opposés d'un rectangle sont un rectangle plus un bit ». C'est faux, et le test des quatre
//! > orientations l'a montré tout de suite : un rectangle a deux diagonales *et* chaque
//! > diagonale a deux sens. Quatre configurations, donc deux bits — désigner le coin source
//! > parmi quatre est d'ailleurs la façon la plus lisible de le dire.
//!
//! Rien n'est perdu — [`Arena::arrow_endpoints`] rend les deux points — et la géométrie cesse
//! d'être calculée deux fois (exigence C2). Le culling d'une flèche devient celui de tout le
//! monde.
//!
//! # Ce que l'arène ne fait pas
//!
//! Elle range et elle balaie ; elle ne décide rien. Le choix du nœud sous le curseur relève des
//! rangs de la fiche 07 § 3, déjà écrits et testés dans [`crate::hit_priority`] : y ajouter ici
//! un « nœud le plus proche » aurait posé une seconde géométrie, vouée à diverger de la
//! première. L'arène offre [`Arena::cull`] et [`Box2::contains`] ; le picking se construit
//! au-dessus.
//!
//! # Supprimer, c'est poser un bit
//!
//! Un nœud supprimé garde sa place et reçoit [`Flags::DEAD`]. Trois conséquences, toutes
//! voulues :
//!
//! - Les [`NodeId`] restent valides : aucun identifiant ne peut désigner un autre nœud que
//!   celui qu'il désignait, jamais. C'est ce qu'une génération par entrée achèterait pour
//!   4 octets de plus par nœud — ici c'est gratuit.
//! - **Annuler une suppression est un bit à retirer**, au lieu de réinsérer un nœud et de
//!   réparer tout ce qui le référençait. La pile d'undo y gagne directement.
//! - La place n'est rendue qu'au compactage, opération explicite et rare. C'est le prix, et il
//!   est connu : [`Arena::dead`] le mesure à tout instant.

use crate::fixed::Fx;

/// Un nœud de l'arène : un indice, quatre octets, aucune allocation.
///
/// La comparaison est une instruction, là où deux `String` demandaient une indirection et un
/// parcours d'octets. Les identifiants lisibles (`"img-42"`) ne survivent que dans le format de
/// fichier et dans l'interface, jamais dans la boucle chaude.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(u32);

impl NodeId {
    /// L'absence de nœud — le parent d'un nœud posé à même le canevas, par exemple.
    ///
    /// Une valeur réservée plutôt qu'un `Option<NodeId>` : `Option` porterait la taille à
    /// 8 octets par champ, soit le double, pour distinguer un cas que `u32::MAX` distingue déjà.
    /// L'arène refuse de créer un nœud à cet indice, ce qui rend la réservation sûre.
    pub const NONE: Self = Self(u32::MAX);

    /// L'indice brut. Réservé à l'indexation et à la persistance.
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// L'identifiant du nœud de rang `i`, ou [`NodeId::NONE`] si ce rang n'est pas
    /// représentable. Réciproque de [`NodeId::index`], pour la persistance et les bancs.
    ///
    /// Il ne dit rien de l'existence du nœud : c'est [`Arena::alive`] qui la tranche.
    pub const fn from_index(i: usize) -> Self {
        if i < u32::MAX as usize {
            Self(i as u32)
        } else {
            Self::NONE
        }
    }

    /// Vrai si l'identifiant désigne un nœud plutôt que l'absence de nœud.
    pub const fn is_some(self) -> bool {
        self.0 != u32::MAX
    }
}

/// Le genre d'un nœud. Ce que l'ancien modèle exprimait par une variante d'énumération, et
/// faisait payer à tous les autres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Kind {
    Text = 0,
    Sticky = 1,
    Arrow = 2,
    Membrane = 3,
    Image = 4,
    Folder = 5,
    Panel = 6,
}

/// Les propriétés booléennes d'un nœud, un bit chacune.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flags(u8);

impl Flags {
    /// Le nœud est supprimé : il garde sa place, son identifiant reste valide, et il est
    /// invisible à toute requête.
    pub const DEAD: Self = Self(1 << 0);
    /// Le nœud ne peut être ni déplacé ni redimensionné (verrouillage L, fiche 08).
    pub const LOCKED: Self = Self(1 << 1);
    /// Pour une flèche : sa source est sur le bord **droit** de sa boîte, et non le bord
    /// gauche. Avec [`Flags::ARROW_FROM_BOTTOM`], désigne le coin source parmi quatre.
    pub const ARROW_FROM_RIGHT: Self = Self(1 << 2);
    /// Pour une flèche : sa source est sur le bord **bas** de sa boîte, et non le bord haut.
    pub const ARROW_FROM_BOTTOM: Self = Self(1 << 3);
    /// Pour une flèche : elle porte une pointe à ses deux extrémités.
    pub const ARROW_BIDIRECTIONAL: Self = Self(1 << 4);
    /// Pour une image : le média est une vidéo.
    pub const VIDEO: Self = Self(1 << 5);

    /// Vrai si tous les bits de `other` sont posés.
    pub const fn has(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Une copie où les bits de `other` sont posés.
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Une copie où les bits de `other` sont retirés.
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Une copie où les bits de `other` valent `on`.
    pub const fn set(self, other: Self, on: bool) -> Self {
        if on {
            self.with(other)
        } else {
            self.without(other)
        }
    }
}

/// La boîte d'un nœud, en coordonnées de document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Box2 {
    pub x: Fx,
    pub y: Fx,
    pub w: Fx,
    pub h: Fx,
}

impl Box2 {
    /// La boîte de coin haut-gauche `(x, y)` et de dimensions `(w, h)`.
    pub const fn new(x: Fx, y: Fx, w: Fx, h: Fx) -> Self {
        Self { x, y, w, h }
    }

    /// La plus petite boîte contenant les deux points — la forme sous laquelle une flèche est
    /// rangée.
    pub fn spanning(ax: Fx, ay: Fx, bx: Fx, by: Fx) -> Self {
        Self {
            x: ax.min(bx),
            y: ay.min(by),
            w: (bx - ax).abs(),
            h: (by - ay).abs(),
        }
    }

    /// Le bord droit.
    pub fn right(self) -> Fx {
        self.x + self.w
    }

    /// Le bord bas.
    pub fn bottom(self) -> Fx {
        self.y + self.h
    }

    /// Vrai si les deux boîtes se recouvrent, bords compris. Exact : quatre comparaisons
    /// entières, aucune tolérance.
    pub fn overlaps(self, other: Self) -> bool {
        self.x <= other.right()
            && other.x <= self.right()
            && self.y <= other.bottom()
            && other.y <= self.bottom()
    }

    /// Vrai si le point est dans la boîte, bords compris.
    pub fn contains(self, px: Fx, py: Fx) -> bool {
        px >= self.x && px <= self.right() && py >= self.y && py <= self.bottom()
    }
}

/// L'arène : un tableau par champ, un indice par nœud.
///
/// Tous les vecteurs ont exactement la même longueur — c'est l'invariant ARN-1, et
/// [`Arena::check`] le vérifie. Un `NodeId` est valide pour tous ou pour aucun.
#[derive(Debug, Clone, Default)]
pub struct Arena {
    x: Vec<Fx>,
    y: Vec<Fx>,
    w: Vec<Fx>,
    h: Vec<Fx>,
    kind: Vec<Kind>,
    flags: Vec<Flags>,
    parent: Vec<NodeId>,
    /// Nombre de nœuds portant [`Flags::DEAD`]. Tenu à jour, jamais recompté.
    dead: usize,
}

impl Arena {
    /// Une arène vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Une arène vide, dimensionnée d'avance pour `n` nœuds.
    ///
    /// Sept allocations pour tout le document, quelle que soit sa taille — là où l'ancien
    /// modèle en faisait trois à huit **par nœud**.
    pub fn with_capacity(n: usize) -> Self {
        Self {
            x: Vec::with_capacity(n),
            y: Vec::with_capacity(n),
            w: Vec::with_capacity(n),
            h: Vec::with_capacity(n),
            kind: Vec::with_capacity(n),
            flags: Vec::with_capacity(n),
            parent: Vec::with_capacity(n),
            dead: 0,
        }
    }

    /// Le nombre d'emplacements, morts compris. C'est la borne des indices valides.
    pub fn slots(&self) -> usize {
        self.x.len()
    }

    /// Le nombre de nœuds vivants.
    pub fn len(&self) -> usize {
        self.slots() - self.dead
    }

    /// Vrai si aucun nœud n'est vivant.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Le nombre d'emplacements occupés par des nœuds supprimés — la place qu'un compactage
    /// rendrait.
    pub fn dead(&self) -> usize {
        self.dead
    }

    /// Les octets de tronc par nœud, remplissage d'alignement compris — c'est-à-dire zéro,
    /// puisqu'il n'y a plus de structure par nœud.
    pub const BYTES_PER_NODE: usize = 4 * 4 + 1 + 1 + 4;

    /// Pose un nœud et rend son identifiant.
    ///
    /// # Panique
    ///
    /// Si l'arène atteint `u32::MAX - 1` nœuds, l'indice suivant entrerait en collision avec
    /// [`NodeId::NONE`]. Quatre milliards de nœuds, c'est quatre cents fois la cible de la
    /// charte : paniquer est ici la bonne réponse, parce que rendre une erreur obligerait
    /// chaque appelant à traiter un cas qui ne peut pas se produire.
    pub fn spawn(&mut self, kind: Kind, b: Box2, parent: NodeId) -> NodeId {
        let idx = self.slots();
        assert!(idx < u32::MAX as usize - 1, "arène saturée : {idx} nœuds");
        self.x.push(b.x);
        self.y.push(b.y);
        self.w.push(b.w);
        self.h.push(b.h);
        self.kind.push(kind);
        self.flags.push(Flags::default());
        self.parent.push(parent);
        NodeId(idx as u32)
    }

    /// Vrai si l'identifiant désigne un emplacement de cette arène, mort ou vif.
    pub fn holds(&self, id: NodeId) -> bool {
        id.is_some() && id.index() < self.slots()
    }

    /// Vrai si le nœud existe et n'est pas supprimé.
    pub fn alive(&self, id: NodeId) -> bool {
        self.holds(id) && !self.flags[id.index()].has(Flags::DEAD)
    }

    /// Supprime un nœud : un bit posé, sa place conservée, son identifiant toujours valide.
    /// Rend `false` s'il était déjà mort ou inconnu.
    pub fn kill(&mut self, id: NodeId) -> bool {
        if !self.alive(id) {
            return false;
        }
        self.flags[id.index()] = self.flags[id.index()].with(Flags::DEAD);
        self.dead += 1;
        true
    }

    /// Annule une suppression. Rend `false` si le nœud était déjà vivant ou inconnu.
    pub fn revive(&mut self, id: NodeId) -> bool {
        if !self.holds(id) || !self.flags[id.index()].has(Flags::DEAD) {
            return false;
        }
        self.flags[id.index()] = self.flags[id.index()].without(Flags::DEAD);
        self.dead -= 1;
        true
    }

    /// La boîte d'un nœud vivant.
    pub fn box_of(&self, id: NodeId) -> Option<Box2> {
        self.alive(id).then(|| Box2 {
            x: self.x[id.index()],
            y: self.y[id.index()],
            w: self.w[id.index()],
            h: self.h[id.index()],
        })
    }

    /// Pose la boîte d'un nœud vivant. Rend `false` si le nœud est inconnu, mort ou verrouillé.
    pub fn set_box(&mut self, id: NodeId, b: Box2) -> bool {
        if !self.alive(id) || self.flags[id.index()].has(Flags::LOCKED) {
            return false;
        }
        let i = id.index();
        self.x[i] = b.x;
        self.y[i] = b.y;
        self.w[i] = b.w;
        self.h[i] = b.h;
        true
    }

    /// Déplace un nœud vivant et non verrouillé. Exact : une addition entière par axe.
    pub fn translate(&mut self, id: NodeId, dx: Fx, dy: Fx) -> bool {
        if !self.alive(id) || self.flags[id.index()].has(Flags::LOCKED) {
            return false;
        }
        let i = id.index();
        self.x[i] += dx;
        self.y[i] += dy;
        true
    }

    /// Le genre d'un nœud existant, mort ou vif.
    pub fn kind_of(&self, id: NodeId) -> Option<Kind> {
        self.holds(id).then(|| self.kind[id.index()])
    }

    /// Les drapeaux d'un nœud existant.
    pub fn flags_of(&self, id: NodeId) -> Option<Flags> {
        self.holds(id).then(|| self.flags[id.index()])
    }

    /// Pose les drapeaux d'un nœud existant, sauf [`Flags::DEAD`] — la vie d'un nœud se règle
    /// par [`Arena::kill`] et [`Arena::revive`], qui tiennent le compteur.
    pub fn set_flags(&mut self, id: NodeId, f: Flags) -> bool {
        if !self.holds(id) {
            return false;
        }
        let was_dead = self.flags[id.index()].has(Flags::DEAD);
        self.flags[id.index()] = f.set(Flags::DEAD, was_dead);
        true
    }

    /// Le parent d'un nœud existant — sa membrane, ou [`NodeId::NONE`].
    pub fn parent_of(&self, id: NodeId) -> Option<NodeId> {
        self.holds(id).then(|| self.parent[id.index()])
    }

    /// Pose le parent d'un nœud existant.
    pub fn set_parent(&mut self, id: NodeId, parent: NodeId) -> bool {
        if !self.holds(id) {
            return false;
        }
        self.parent[id.index()] = parent;
        true
    }

    /// Les deux extrémités d'une flèche, dans l'ordre source → cible.
    ///
    /// C'est la lecture inverse de la boîte englobante : les deux points d'une flèche sont deux
    /// coins opposés d'un rectangle, et les deux drapeaux [`Flags::ARROW_FROM_RIGHT`] et
    /// [`Flags::ARROW_FROM_BOTTOM`] disent lequel des quatre est la source. Rend `None` si le
    /// nœud n'est pas une flèche vivante.
    pub fn arrow_endpoints(&self, id: NodeId) -> Option<(Fx, Fx, Fx, Fx)> {
        if self.kind_of(id)? != Kind::Arrow {
            return None;
        }
        let b = self.box_of(id)?;
        let f = self.flags[id.index()];
        let (sx, tx) = if f.has(Flags::ARROW_FROM_RIGHT) {
            (b.right(), b.x)
        } else {
            (b.x, b.right())
        };
        let (sy, ty) = if f.has(Flags::ARROW_FROM_BOTTOM) {
            (b.bottom(), b.y)
        } else {
            (b.y, b.bottom())
        };
        Some((sx, sy, tx, ty))
    }

    /// Pose une flèche par ses deux extrémités : la boîte englobante et le coin source d'un
    /// coup.
    pub fn set_arrow(&mut self, id: NodeId, ax: Fx, ay: Fx, bx: Fx, by: Fx) -> bool {
        if self.kind_of(id) != Some(Kind::Arrow) || !self.alive(id) {
            return false;
        }
        let i = id.index();
        self.flags[i] = self.flags[i]
            .set(Flags::ARROW_FROM_RIGHT, ax > bx)
            .set(Flags::ARROW_FROM_BOTTOM, ay > by);
        self.set_box(id, Box2::spanning(ax, ay, bx, by))
    }

    /// Tous les nœuds vivants dont la boîte croise `view`, dans l'ordre des identifiants.
    ///
    /// C'est la requête chaude : elle ne lit que les quatre tableaux de coordonnées et celui
    /// des drapeaux, tous contigus, et ne touche jamais au bagage d'un nœud.
    pub fn cull(&self, view: Box2, out: &mut Vec<NodeId>) {
        out.clear();
        for i in 0..self.slots() {
            if self.flags[i].has(Flags::DEAD) {
                continue;
            }
            let right = self.x[i] + self.w[i];
            let bottom = self.y[i] + self.h[i];
            if self.x[i] <= view.right()
                && view.x <= right
                && self.y[i] <= view.bottom()
                && view.y <= bottom
            {
                out.push(NodeId(i as u32));
            }
        }
    }

    /// Vérifie l'invariant ARN-1 : tous les tableaux ont la même longueur, le compteur de morts
    /// est juste, et aucun parent ne désigne un emplacement inexistant.
    ///
    /// Destiné aux tests et au chargement d'un document venu du disque.
    pub fn check(&self) -> Result<(), String> {
        let n = self.x.len();
        for (nom, len) in [
            ("y", self.y.len()),
            ("w", self.w.len()),
            ("h", self.h.len()),
            ("kind", self.kind.len()),
            ("flags", self.flags.len()),
            ("parent", self.parent.len()),
        ] {
            if len != n {
                return Err(format!(
                    "ARN-1 : le tableau {nom} a {len} entrées pour {n} nœuds"
                ));
            }
        }
        let dead = self.flags.iter().filter(|f| f.has(Flags::DEAD)).count();
        if dead != self.dead {
            return Err(format!(
                "ARN-1 : {dead} nœuds morts comptés pour {} annoncés",
                self.dead
            ));
        }
        for (i, p) in self.parent.iter().enumerate() {
            if p.is_some() && p.index() >= n {
                return Err(format!(
                    "ARN-1 : le nœud {i} a pour parent {} hors arène",
                    p.index()
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
