//! **Le code et la pipette de la roue** (ROUE-1, fiche 99) — les deux autres façons de dire une
//! couleur que Blender offre à côté du disque.
//!
//! * **Le code** : un clic sur le champ `#rrggbb` l'ouvre à la frappe, vide — la frappe remplace,
//!   comme un champ de Blender qui choisit tout à l'ouverture. Seuls des chiffres hexadécimaux
//!   s'écrivent ; dès que le texte dit une couleur (3 ou 6 chiffres), elle se pose, en direct.
//!   `Entrée` ou `Échap` referment le champ — la roue reste ouverte : un second `Échap` l'annule.
//! * **La pipette** : armée, le clic suivant sur le canevas prend **la couleur d'un nœud** — celle
//!   qu'on a choisie pour un texte, un post-it, une membrane, un dossier, et la couleur moyenne
//!   d'une photo (le sommet de sa pyramide, MOYENNE-1). Un nœud sans couleur choisie n'en donne
//!   pas : sa lueur symbiotique ne dit rien (sa parole : « tout a une signification »). Elle
//!   lit le document, pas l'écran : la couleur prise est celle que le nœud **porte**, pas un pixel
//!   que voiles, lueurs et membranes ont mêlé.

use crate::app::GlucoseApp;
use crate::interactions::text_entry::TextEntry;
use glucose_core::couleur::roue::PlaceDansLaRoue;
use glucose_core::couleur::Oklch;
use glucose_core::hit_priority::PickOwner;
use winit::keyboard::{Key, NamedKey};

impl GlucoseApp {
    /// Le champ du code s'ouvre, vide ; la pipette se désarme.
    pub(super) fn ouvrir_le_code(&mut self) {
        if let Some(roue) = self.ui.roue.as_mut() {
            roue.code = Some(TextEntry::new(""));
            roue.pipette = false;
        }
        self.mark_dirty();
    }

    /// La pipette s'arme, ou se désarme.
    pub(super) fn armer_la_pipette(&mut self) {
        if let Some(roue) = self.ui.roue.as_mut() {
            roue.pipette = !roue.pipette;
            roue.code = None;
        }
        self.mark_dirty();
    }

    /// **Une touche pour le champ du code**, s'il est ouvert. Rend `false` s'il ne l'est pas.
    pub(super) fn frapper_le_code(&mut self, key: &Key) -> bool {
        let Some(saisie) = self.ui.roue.as_mut().and_then(|r| r.code.as_mut()) else {
            return false;
        };
        match key {
            Key::Named(NamedKey::Enter | NamedKey::Escape) => {
                if let Some(roue) = self.ui.roue.as_mut() {
                    roue.code = None;
                }
            }
            // Seuls des chiffres hexadécimaux : un « # » tapé est déjà écrit devant.
            Key::Character(c) if c.chars().all(|c| c.is_ascii_hexdigit()) => {
                if saisie.text().len() + c.len() <= 6 {
                    saisie.insert(&c.to_ascii_lowercase());
                }
            }
            Key::Character(_) => {}
            autre => {
                saisie.editer(autre);
            }
        }
        let tape = self
            .ui
            .roue
            .as_ref()
            .and_then(|r| r.code.as_ref())
            .and_then(|s| crate::renderer::parse_hex_rgb(s.text()));
        if let Some(octets) = tape {
            self.poser_la_couleur(octets);
        }
        self.mark_dirty();
        true
    }

    /// **La pipette prend la couleur du nœud sous ce point de l'écran**, et se désarme.
    pub(super) fn prendre_a_la_pipette(&mut self, (px, py): (f32, f32)) {
        if let Some(roue) = self.ui.roue.as_mut() {
            roue.pipette = false;
        }
        let vue = self.store.viewport();
        let (wx, wy) = crate::canvas::screen_to_world(f64::from(px), f64::from(py), &vue);
        if let Some(octets) = self.couleur_du_noeud_sous(wx, wy) {
            self.poser_la_couleur(octets);
        }
        self.mark_dirty();
    }

    /// La couleur que porte le nœud du dessus en `(wx, wy)`, s'il en porte une.
    fn couleur_du_noeud_sous(&mut self, wx: f64, wy: f64) -> Option<(u8, u8, u8)> {
        let dessus = self.pick_candidate_at(wx, wy)?;
        let tableau = self.store.project.active_board_id.clone();
        match dessus.owner {
            PickOwner::Image => {
                let src = self.store.image(&tableau, &dessus.id)?.src.clone()?;
                match self.renderer.magasin.moyenne(&src) {
                    crate::renderer::magasin::Moyenne::Connue([r, g, b, a]) if a > 0 => {
                        let droit =
                            |c: u8| ((u16::from(c) * 255 + u16::from(a) / 2) / u16::from(a)) as u8;
                        Some((droit(r), droit(g), droit(b)))
                    }
                    _ => None,
                }
            }
            PickOwner::Annotation | PickOwner::Membrane => self
                .store
                .project
                .annotation(&tableau, &dessus.id)?
                .couleur_propre()
                .and_then(crate::renderer::parse_hex_rgb),
            PickOwner::Folder => {
                let dossier = self
                    .store
                    .active_board()?
                    .folders
                    .iter()
                    .find(|f| f.id == dessus.id)?;
                crate::renderer::parse_hex_rgb(&dossier.color)
            }
            PickOwner::Arrow | PickOwner::Groupe => None,
        }
    }

    /// Une couleur dite autrement que par le disque — le code, la pipette — devient la place de la
    /// roue, et se pose.
    fn poser_la_couleur(&mut self, octets: (u8, u8, u8)) {
        let Some(teinte) = self.ui.roue.as_ref().map(|r| r.place.teinte) else {
            return;
        };
        let place = PlaceDansLaRoue::de(Oklch::des_octets(octets), teinte);
        self.poser_place_et_couleur(place, octets);
    }
}
