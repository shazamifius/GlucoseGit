//! **MEMB-2 — ce que le rendu montre en mode Focus** : la membrane focalisée, son contenu, et
//! rien d'autre, sur un fond à sa couleur.
//!
//! Le masque se calcule une fois par état du document
//! ([`glucose_core::membrane_focus::masque_du_focus`]) et se lit là où le rendu choisit ce
//! qu'il dessine — le culling de [`super::Renderer`] : toutes les passes, sur les deux voies,
//! en héritent sans avoir à le savoir. Le clic et la sélection au rectangle le lisent aussi :
//! ce qu'on ne voit pas ne s'attrape pas.
//!
//! Rien n'est retiré du document : le focus cache, il ne détruit pas.

use glucose_core::membrane_focus::{focus_background, masque_du_focus, MasqueFocus};
use glucose_core::store::Store;
use tiny_skia::Color;

/// Le focus tel que le rendu le tient : le masque de l'image, et l'état qui l'a produit.
#[derive(Debug, Default)]
pub struct FocusDuRendu {
    /// (tableau, membrane, version du document) → masque.
    courant: Option<((String, String, u64), MasqueFocus)>,
    /// Le fond du thème, gardé pendant que le focus le teinte.
    fond_de_base: Option<Color>,
}

impl FocusDuRendu {
    pub fn masque(&self) -> Option<&MasqueFocus> {
        self.courant.as_ref().map(|(_, m)| m)
    }

    /// Retire des rangs visibles ce que le focus cache. Hors focus, ne touche à rien.
    pub fn filtrer(&self, rangs: &mut Vec<u32>) {
        if let Some(m) = self.masque() {
            rangs.retain(|&r| m.permis.get(r as usize).copied().unwrap_or(false));
        }
    }

    /// Cet élément se voit-il ? Toujours vrai hors focus.
    pub fn laisse_voir(&self, id: &str) -> bool {
        self.masque().is_none_or(|m| m.laisse_voir(id))
    }
}

impl super::Renderer {
    /// Règle le focus de l'image à venir : la membrane focalisée, ou aucune. Ne recalcule le
    /// masque que si le tableau, la membrane ou le document ont changé.
    pub fn regler_le_focus(&mut self, store: &Store, membrane: Option<&str>) {
        let cle = membrane.map(|m| {
            (
                store.project.active_board_id.clone(),
                m.to_string(),
                store.version,
            )
        });
        if cle.as_ref() == self.focus.courant.as_ref().map(|(c, _)| c) {
            return;
        }
        let masque =
            membrane.and_then(|m| store.active_board().and_then(|b| masque_du_focus(b, m)));
        self.focus.courant = cle.zip(masque);
        let base = *self.focus.fond_de_base.get_or_insert(self.theme.bg_canvas);
        let (r, g, b) = super::scene::MEMBRANE_SANS_COULEUR;
        let sans_couleur = format!("#{r:02x}{g:02x}{b:02x}");
        self.theme.bg_canvas = match self.focus.masque() {
            Some(m) => teinte(base, m.couleur.as_deref().unwrap_or(&sans_couleur)),
            None => {
                self.focus.fond_de_base = None;
                base
            }
        };
    }
}

/// Le fond de la scène teint de la couleur de la membrane — la règle du noyau, en couleur.
fn teinte(base: Color, couleur: &str) -> Color {
    let c = base.to_color_u8();
    let hex = format!("#{:02x}{:02x}{:02x}", c.red(), c.green(), c.blue());
    let melange = focus_background(Some(couleur), Some(&hex), None);
    let canal = |i: usize| {
        melange
            .get(i..i + 2)
            .and_then(|h| u8::from_str_radix(h, 16).ok())
    };
    match (canal(1), canal(3), canal(5)) {
        (Some(r), Some(g), Some(b)) => Color::from_rgba8(r, g, b, 255),
        _ => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Le fond part vers la couleur de la membrane, sans l'atteindre : le contenu reste lisible.
    #[test]
    fn test_la_teinte_part_vers_la_membrane_sans_l_atteindre() {
        let base = Color::from_rgba8(13, 13, 13, 255);
        let t = teinte(base, "#ff0000").to_color_u8();
        // Chaque canal fait 16 % du chemin vers la membrane (la règle du noyau) : le rouge
        // monte de 13 vers 255, le vert et le bleu descendent de 13 vers 0.
        assert_eq!((t.red(), t.green(), t.blue()), (52, 11, 11), "{t:?}");
        assert_eq!(
            teinte(base, "pas une couleur"),
            base,
            "une couleur illisible laisse le fond"
        );
    }
}
