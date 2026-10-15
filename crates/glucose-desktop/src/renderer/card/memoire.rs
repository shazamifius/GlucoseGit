//! **Les mises en page gardées d'une image à l'autre** (P8 bis, fiche 73 § 7).
//!
//! # Ce que la mesure a montré
//!
//! La mise en page d'un nœud de texte se calcule **en unités du monde** (CARD-1) : elle ne
//! dépend que de son texte, de sa largeur et du mode (MODE-1) — jamais du zoom ni de la place.
//! Elle se refaisait pourtant à chaque image, et **deux fois** pour chaque nœud visible : une
//! pour compter ses lignes (sa hauteur), une pour le dessiner. Invisible pour un post-it ; pour
//! un nœud de dix-sept mille mots vu de près, 3,4 ms par mise en page (`bench_de_pres`).
//!
//! # La loi de l'oubli : celle des textures et de l'atlas
//!
//! Ce que l'image d'avant n'a pas demandé s'oublie au début de la suivante
//! ([`MisesEnPage::nouvelle_image`]). Aucune capacité n'est choisie : la mémoire tient ce que
//! l'écran montre, et un nœud qui en sort libère sa place à l'image suivante. Un nœud qu'on
//! écrit change de texte à chaque frappe : sa mise en page d'avant s'oublie de même.
//!
//! # La clé est le texte lui-même
//!
//! Rangée par largeur et par mode, puis par **son texte entier** : deux textes différents ne
//! peuvent pas se confondre, et ce n'est pas une garde de plus qui le promet — c'est la table.
//!
//! # Pourquoi un `RefCell`
//!
//! Pour la même raison que la mémoire des formules ([`crate::renderer::math::MathRenderer`]) :
//! demander une mise en page est, pour l'appelant, une question sans effet ; qu'elle soit retenue
//! est un détail interne, et le rendu est mono-fil.

use super::card_text_layout;
use crate::renderer::math::MathRenderer;
use crate::renderer::richtext::{TextLayout, TextMode};
use crate::typography::Typography;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

/// Une mise en page retenue, et la dernière image qui l'a demandée.
struct Entree {
    mise_en_page: Rc<TextLayout>,
    vue: u64,
}

/// Les mises en page d'une largeur (au bit près) et d'un mode, par texte.
type ParTexte = HashMap<Box<str>, Entree>;

/// **La mémoire des mises en page** des nœuds de texte.
#[derive(Default)]
pub struct MisesEnPage {
    entrees: RefCell<HashMap<(u32, TextMode), ParTexte>>,
    image: Cell<u64>,
}

impl MisesEnPage {
    /// **La mise en page de ce texte** dans un nœud large de `largeur` unités du monde — celle
    /// de [`card_text_layout`], retenue.
    pub fn de(
        &self,
        (typographie, math): (&Typography, &MathRenderer),
        texte: &str,
        largeur: f32,
        mode: TextMode,
    ) -> Rc<TextLayout> {
        let image = self.image.get();
        let mut entrees = self.entrees.borrow_mut();
        let par_texte = entrees.entry((largeur.to_bits(), mode)).or_default();
        if let Some(e) = par_texte.get_mut(texte) {
            e.vue = image;
            return Rc::clone(&e.mise_en_page);
        }
        let mise_en_page = Rc::new(card_text_layout(typographie, math, texte, largeur, mode));
        par_texte.insert(
            texte.into(),
            Entree {
                mise_en_page: Rc::clone(&mise_en_page),
                vue: image,
            },
        );
        mise_en_page
    }

    /// **Une image commence** : ce que la précédente n'a pas demandé s'oublie.
    pub fn nouvelle_image(&self) {
        let image = self.image.get();
        self.entrees.borrow_mut().retain(|_, par_texte| {
            par_texte.retain(|_, e| e.vue == image);
            !par_texte.is_empty()
        });
        self.image.set(image + 1);
    }

    /// Combien de mises en page la mémoire retient.
    #[cfg(test)]
    pub fn combien(&self) -> usize {
        self.entrees.borrow().values().map(HashMap::len).sum()
    }

    /// Retient-elle une mise en page de ce texte, à quelque largeur que ce soit ?
    #[cfg(test)]
    pub fn retient(&self, texte: &str) -> bool {
        self.entrees
            .borrow()
            .values()
            .any(|par_texte| par_texte.contains_key(texte))
    }
}

#[cfg(test)]
mod tests;
