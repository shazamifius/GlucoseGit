//! Le texte des nœuds : une arène d'octets, un intervalle par nœud — `std` uniquement.
//!
//! # Pourquoi pas un `String` par nœud
//!
//! Un `String` coûte 24 octets d'en-tête, une allocation, et disperse son contenu quelque part
//! sur le tas. Sur dix millions de nœuds, ce sont 240 Mo d'en-têtes, dix millions d'allocations
//! et autant de sauts mémoire — c'est l'essentiel des 17,7 secondes que l'ancien modèle met à
//! se construire, mesurées par `bench_arena`.
//!
//! Ici, tout le texte du document tient dans **un seul `Vec<u8>`**, et chaque nœud ne porte que
//! l'intervalle qui lui revient : 8 octets, aucune allocation. Deux conséquences directes :
//!
//! - Le culling et le rendu géométrique ne touchent **jamais** au texte. C'est l'intérêt des
//!   tableaux de champs : ce qu'on ne lit pas ne coûte rien, pas même en défauts de cache.
//! - Charger un document, c'est lire un bloc d'octets et un tableau d'intervalles. Il n'y a
//!   rien à reconstruire.
//!
//! # Ce que ça coûte, et qui est le vrai prix
//!
//! Réécrire le texte d'un nœud ne peut pas se faire sur place : le nouveau texte n'a aucune
//! raison d'avoir la même longueur. Il est donc **ajouté en queue**, et les octets qu'il
//! remplace sont abandonnés. Frapper une lettre dans une carte de 100 caractères abandonne
//! 100 octets.
//!
//! C'est un choix, pas un oubli, et il est mesuré : [`TextArena::waste`] dit à tout instant
//! combien d'octets sont perdus, et [`TextArena::compact`] les rend. Une session d'édition
//! ordinaire touche quelques nœuds ; le gaspillage se compte alors en kilo-octets. Le
//! compactage est explicite — jamais déclenché en douce au milieu d'une frappe.
//!
//! # La borne
//!
//! Les intervalles sont en `u32` : l'arène tient **4 Go de texte**, soit 400 octets par nœud
//! sur dix millions. Au-delà, [`TextArena::set`] refuse plutôt que de tronquer ou de reboucler.
//! Le jour où un document sérieux s'en approche, le passage en `u64` est mécanique — il coûte
//! 4 octets par nœud, et cette page dira pourquoi on l'a payé.

use super::NodeId;

/// L'intervalle d'octets d'un nœud dans l'arène. Un texte absent est un intervalle vide.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Span {
    start: u32,
    len: u32,
}

/// Vrai si ajouter `len` octets à un bloc qui en occupe `occupe` sortirait du domaine des
/// intervalles.
///
/// La règle est isolée ici pour être testable : fabriquer l'état qui la déclenche demanderait
/// d'allouer quatre gigaoctets, ce qu'aucune suite de tests ne devrait faire. La règle, elle,
/// se vérifie sur des nombres.
fn deborde(occupe: usize, len: usize) -> bool {
    occupe.saturating_add(len) > u32::MAX as usize
}

/// Le texte de tous les nœuds d'un document, dans un seul bloc d'octets.
#[derive(Debug, Clone, Default)]
pub struct TextArena {
    bytes: Vec<u8>,
    spans: Vec<Span>,
    waste: usize,
}

impl TextArena {
    /// Une arène vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Une arène vide, dimensionnée pour `nodes` nœuds et `bytes` octets de texte.
    pub fn with_capacity(nodes: usize, bytes: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(bytes),
            spans: Vec::with_capacity(nodes),
            waste: 0,
        }
    }

    /// Les octets par nœud du tableau d'intervalles, contenu exclu.
    pub const BYTES_PER_NODE: usize = std::mem::size_of::<Span>();

    /// Le texte d'un nœud. Vide si le nœud n'en porte pas, ou n'existe pas.
    pub fn get(&self, id: NodeId) -> &str {
        let Some(s) = self.spans.get(id.index()).copied() else {
            return "";
        };
        let (a, b) = (s.start as usize, s.start as usize + s.len as usize);
        // L'invariant TXT-1 garantit que chaque intervalle tombe sur des frontières de
        // caractères : il n'est posé que par `set`, qui écrit une chaîne entière.
        std::str::from_utf8(&self.bytes[a..b]).unwrap_or_default()
    }

    /// Vrai si le nœud porte un texte non vide.
    pub fn has(&self, id: NodeId) -> bool {
        self.spans.get(id.index()).is_some_and(|s| s.len > 0)
    }

    /// Pose le texte d'un nœud. Rend `false` si l'arène est pleine — le texte précédent est
    /// alors intact.
    ///
    /// Le nouveau texte est ajouté en queue et l'ancien abandonné : voir l'en-tête du module.
    /// Poser le même texte que celui déjà en place ne gaspille rien et ne recopie rien.
    pub fn set(&mut self, id: NodeId, text: &str) -> bool {
        if !id.is_some() {
            return false;
        }
        if self.get(id) == text {
            return true;
        }
        if deborde(self.bytes.len(), text.len()) {
            return false;
        }
        if self.spans.len() <= id.index() {
            self.spans.resize(id.index() + 1, Span::default());
        }
        self.waste += self.spans[id.index()].len as usize;
        let start = self.bytes.len() as u32;
        self.bytes.extend_from_slice(text.as_bytes());
        self.spans[id.index()] = Span { start, len: text.len() as u32 };
        true
    }

    /// Retire le texte d'un nœud, abandonnant ses octets.
    pub fn clear(&mut self, id: NodeId) {
        if let Some(s) = self.spans.get_mut(id.index()) {
            self.waste += s.len as usize;
            *s = Span::default();
        }
    }

    /// Les octets abandonnés par les réécritures — ce qu'un compactage rendrait.
    pub fn waste(&self) -> usize {
        self.waste
    }

    /// Les octets occupés par le bloc de texte, gaspillage compris.
    pub fn bytes_used(&self) -> usize {
        self.bytes.len()
    }

    /// Le nombre d'intervalles rangés — au plus le nombre de nœuds ayant jamais porté un texte.
    pub fn slots(&self) -> usize {
        self.spans.len()
    }

    /// Réécrit le bloc sans trous et rend le nombre d'octets récupérés.
    ///
    /// Opération explicite : elle recopie tout le texte du document, donc elle se déclenche à
    /// l'enregistrement ou sur demande, jamais au milieu d'une frappe.
    pub fn compact(&mut self) -> usize {
        if self.waste == 0 {
            return 0;
        }
        let avant = self.bytes.len();
        // Recopier dans l'ordre des nœuds : le texte voisin dans le document devient voisin en
        // mémoire, ce qui sert tout parcours ultérieur.
        let mut neufs = Vec::with_capacity(avant - self.waste);
        for s in &mut self.spans {
            if s.len == 0 {
                *s = Span::default();
                continue;
            }
            let (a, b) = (s.start as usize, s.start as usize + s.len as usize);
            let start = neufs.len() as u32;
            neufs.extend_from_slice(&self.bytes[a..b]);
            s.start = start;
        }
        self.bytes = neufs;
        self.waste = 0;
        avant - self.bytes.len()
    }

    /// Vérifie l'invariant TXT-1 : tout intervalle tombe dans le bloc, sur des frontières de
    /// caractères, et le gaspillage annoncé est celui que les intervalles laissent.
    pub fn check(&self) -> Result<(), String> {
        let mut vus = 0usize;
        for (i, s) in self.spans.iter().enumerate() {
            let (a, b) = (s.start as usize, s.start as usize + s.len as usize);
            if b > self.bytes.len() {
                return Err(format!("TXT-1 : le nœud {i} pointe sur {a}..{b} hors des {} octets", self.bytes.len()));
            }
            if std::str::from_utf8(&self.bytes[a..b]).is_err() {
                return Err(format!("TXT-1 : le nœud {i} ne tombe pas sur des frontières de caractères"));
            }
            vus += s.len as usize;
        }
        if vus + self.waste != self.bytes.len() {
            return Err(format!(
                "TXT-1 : {vus} octets utilisés + {} gaspillés ≠ {} occupés",
                self.waste,
                self.bytes.len()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
