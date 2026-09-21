//! **Les bandes qu'une couche a réellement écrites** — et rien d'autre ne s'efface ni ne part
//! sur le bus (BANDE-1).
//!
//! # Ce que ce module supprime, chiffré
//!
//! Les chroniques de terrain donnent, sur chaque geste et sur les deux cartes graphiques :
//!
//! ```text
//!     effacer   2,05 ms        blit   1,72 ms
//! ```
//!
//! Trois virgule sept sept millisecondes sur une image qui en coûte 9,74, et **ces deux postes
//! ne dessinent rien** : le premier remplit de transparent les seize mébioctets de la couche
//! du dessus, le second les envoie à la carte. Tous deux travaillent sur l'écran entier, quelle
//! que soit la part qui a changé.
//!
//! Or depuis COMPOSANT-1 cette couche ne porte plus que la chrome — barre, onglets, docks,
//! minimap — et les ornements du geste. `bench_dessus` a compté ce qu'elle touche vraiment :
//!
//! ```text
//!     canevas nu           198 lignes sur 1600    12,4 %
//!     avec une sélection   232 lignes sur 1600    14,5 %
//! ```
//!
//! **Douze pour cent.** C'est le plancher de la charte qui est en jeu : une image sous 8,33 ms
//! tient dans deux balayages, donc cent vingt images par seconde, et il en manque trois.
//!
//! # Pourquoi des bandes, et pas une boîte
//!
//! Le même banc donne la seconde moitié de la réponse : l'étendue de la première ligne écrite
//! à la dernière vaut **99,2 %**. La chrome occupe le haut et le bas avec du vide entre les
//! deux, donc un seul rectangle ne gagnerait rien. Il faut les intervalles **maximaux** de
//! lignes consécutives — deux à quatre en pratique : la barre, les ornements, la minimap.
//!
//! Aucune hauteur n'est choisie : une bande est ce que le dessin a écrit, ni plus ni moins.
//!
//! # Pourquoi on balaie au lieu de déclarer
//!
//! L'autre voie était de faire déclarer sa boîte à chaque dessinateur de la couche. Elle est
//! plus rapide et **elle est fausse par construction** : le jour où un site oublie de
//! déclarer, ses pixels ne s'effacent plus et personne ne le voit — c'est exactement la forme
//! des quatre régressions de l'étape 1, *une passe qui cesse d'être appelée*.
//!
//! Le balayage, lui, mesure ce qui a été **réellement écrit**. Il n'a rien à savoir de ce qui
//! dessine, donc rien ne peut lui échapper. Il coûte une lecture séquentielle de la couche —
//! que le processeur enchaîne à pleine bande passante — pour éviter deux écritures de la même
//! taille.

use std::ops::Range;
use tiny_skia::Pixmap;

/// Les intervalles de lignes qu'une couche porte, du haut vers le bas, sans recouvrement.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Bandes(Vec<Range<u32>>);

impl Bandes {
    /// Toute la hauteur, d'un bloc — ce qu'il faut quand on ne sait rien.
    pub fn tout(hauteur: u32) -> Self {
        Self(Vec::from_iter((hauteur > 0).then_some(0..hauteur)))
    }

    /// **Relève les lignes qui portent au moins un pixel non transparent.**
    ///
    /// Un pixel entièrement transparent a ses quatre octets nuls, puisque `tiny-skia` travaille
    /// en alpha prémultiplié : une ligne vide est donc une ligne d'octets nuls, et la question
    /// se pose sans regarder les canaux un par un.
    pub fn relever(p: &Pixmap) -> Self {
        let largeur = p.width() as usize;
        if largeur == 0 {
            return Self::default();
        }
        let mut bandes: Vec<Range<u32>> = Vec::new();
        for (y, rang) in p.data().chunks_exact(largeur * 4).enumerate() {
            if ligne_vide(rang) {
                continue;
            }
            let y = u32::try_from(y).unwrap_or(u32::MAX);
            match bandes.last_mut() {
                // La ligne prolonge la bande en cours.
                Some(derniere) if derniere.end == y => derniere.end = y + 1,
                _ => bandes.push(y..y + 1),
            }
        }
        Self(bandes)
    }

    /// Ce que les deux portent ensemble.
    ///
    /// C'est l'ensemble qu'il faut effacer et téléverser : les lignes que cette image écrit,
    /// **plus** celles que la précédente écrivait et qu'elle n'écrit plus — sans quoi ces
    /// dernières garderaient ce que l'image d'avant y avait mis.
    pub fn union(&self, autre: &Self) -> Self {
        let mut toutes: Vec<Range<u32>> = self.0.iter().chain(autre.0.iter()).cloned().collect();
        toutes.sort_by_key(|b| b.start);
        let mut fondues: Vec<Range<u32>> = Vec::with_capacity(toutes.len());
        for bande in toutes {
            match fondues.last_mut() {
                Some(derniere) if bande.start <= derniere.end => {
                    derniere.end = derniere.end.max(bande.end);
                }
                _ => fondues.push(bande),
            }
        }
        Self(fondues)
    }

    /// Remet ces lignes à zéro, et ne touche à rien d'autre.
    pub fn effacer(&self, p: &mut Pixmap) {
        let largeur = p.width() as usize;
        let hauteur = p.height();
        let octets = p.data_mut();
        for bande in &self.0 {
            let debut = bande.start.min(hauteur) as usize * largeur * 4;
            let fin = bande.end.min(hauteur) as usize * largeur * 4;
            octets[debut..fin].fill(0);
        }
    }

    /// Les intervalles, pour qui doit les parcourir.
    pub fn intervalles(&self) -> &[Range<u32>] {
        &self.0
    }

    /// Combien de lignes ces bandes portent en tout — ce que la chronique lit.
    pub fn lignes(&self) -> u32 {
        self.0.iter().map(|b| b.end - b.start).sum()
    }

    /// N'y a-t-il rien du tout ?
    pub fn vides(&self) -> bool {
        self.0.is_empty()
    }
}

/// Cette ligne est-elle entièrement transparente ?
///
/// # Pourquoi par blocs, et ce que l'octet par octet coûtait
///
/// `rang.iter().all(|o| *o == 0)` dit la même chose et le terrain l'a chiffré : **4,87 ms par
/// image**, pour un relevé censé en économiser 1,5. Le compilateur ne vectorisait pas la
/// comparaison d'octets, et la lecture plafonnait à trois gigaoctets par seconde là où une
/// lecture séquentielle en donne dix fois plus.
///
/// Comparer des blocs de trente-deux octets à un bloc nul se compile, lui, en quelques
/// instructions vectorielles. Le reste de la ligne — au plus trente et un octets — finit à
/// l'unité. Et le court-circuit reste : une ligne écrite s'arrête au premier bloc non nul,
/// donc ce sont les lignes **vides** qui décident du coût, et elles se lisent au plus vite.
fn ligne_vide(rang: &[u8]) -> bool {
    const BLOC: usize = 32;
    let (blocs, reste) = rang.as_chunks::<BLOC>();
    blocs.iter().all(|c| *c == [0u8; BLOC]) && reste.iter().all(|o| *o == 0)
}

#[cfg(test)]
mod tests;
