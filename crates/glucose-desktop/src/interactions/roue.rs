//! **ROUE-1 — les gestes de la roue de couleur** (fiche 99) : l'ouvrir, y glisser, la refermer.
//!
//! Les gestes de Blender (son manuel, « Color Picker »), et ce qu'ils deviennent ici :
//!
//! * **glisser** dans le disque choisit la teinte et la saturation, dans le curseur la clarté — à la
//!   souris comme au doigt (le doigt passe par les mêmes gestes) ; la couleur se pose **en
//!   direct** sur tout ce qui est choisi ;
//! * **`Ctrl`** cale la teinte tous les 30° (les douze d'une roue de peintre) ; **`Maj`** affine : le
//!   mouvement compte pour un dixième ;
//! * **la molette**, sur la roue, change la clarté d'**une différence à peine visible** (0,02 en
//!   Oklab, le pas que l'œil distingue — CSS Color 4) : aucun pas n'est à choisir ;
//! * **`Retour arrière`** rend « Par défaut » ; **`Échap`** annule tout ce que la roue a fait ;
//!   **`Entrée`**, ou un clic hors d'elle, valide.
//!
//! **Un seul geste** du document : à chaque mouvement, la roue repart des couleurs d'avant
//! (`cancel_live_edit`) et pose la nouvelle — le journal ne porte qu'une écriture par nœud, quelle
//! que soit la longueur du glisser ; valider le ferme (un `Ctrl+Z`), annuler l'abandonne sans trace.
//! Tant qu'elle est ouverte, la roue tient le clavier et la souris, comme une question : aucun
//! autre geste ne vient écrire dans le geste qu'elle tient ouvert.

use crate::app::GlucoseApp;
use crate::params::{Pointer, ScreenFrame};
use crate::ui::roue::{endroit, placer, Endroit};
use glucose_core::couleur::roue::{hex, place_sous, PlaceDansLaRoue};
use glucose_core::couleur::{Oklch, DIFFERENCE_A_PEINE_VISIBLE};

/// La roue ouverte : la place choisie, les couleurs d'avant et d'après, le disque dessiné à la
/// clarté de la place, et le glisser en cours.
pub struct RoueOuverte {
    pub place: PlaceDansLaRoue,
    /// La couleur d'avant la roue (`None` : par défaut), pour l'aperçu.
    pub avant: Option<(u8, u8, u8)>,
    /// La couleur que la roue pose (`None` après `Retour arrière`), ou rien encore.
    pub apres: Option<(u8, u8, u8)>,
    /// Le disque dessiné, et la clarté où il l'a été (voir [`GlucoseApp::poser_la_place`]).
    pub disque: tiny_skia::Pixmap,
    clarte_du_disque: f64,
    /// Le code `#rrggbb` qu'on tape, quand son champ est ouvert ([`code`]).
    pub code: Option<crate::interactions::text_entry::TextEntry>,
    /// La pipette est armée : le prochain clic sur le canevas prend la couleur d'un nœud.
    pub pipette: bool,
    glisse: Option<Glisse>,
}

/// Ce que la main tient : le point du disque ou la clarté **non calés** — `Ctrl` cale ce qui se
/// pose, pas ce que la main a parcouru —, et le dernier point de l'écran.
#[derive(Clone, Copy)]
enum Glisse {
    Disque {
        point: (f64, f64),
        dernier: (f32, f32),
    },
    Clarte {
        clarte: f64,
        dernier: (f32, f32),
    },
}

/// Le grand disque à cette clarté, à la taille de l'écran.
fn disque(clarte: f64, scale: f32) -> tiny_skia::Pixmap {
    crate::ui::roue::disque(clarte, crate::ui::roue::diametre(scale))
}

impl GlucoseApp {
    /// **Ouvre la roue** sur la couleur de ce qui est choisi. Rend `false` si rien ne se colore.
    pub(crate) fn ouvrir_la_roue(&mut self) -> bool {
        let couleurs = self.store.couleurs_choisies();
        let Some(premiere) = couleurs.first() else {
            return false;
        };
        let avant = premiere.and_then(crate::renderer::parse_hex_rgb);
        let depart = avant
            .or_else(|| crate::renderer::parse_hex_rgb(crate::theme::DOMAIN_PALETTE[0]))
            .unwrap_or((96, 165, 250));
        let place = PlaceDansLaRoue::de(Oklch::des_octets(depart), 0.0);
        self.ui.roue = Some(RoueOuverte {
            place,
            avant,
            apres: avant,
            disque: disque(place.clarte, self.ui.scale_factor),
            clarte_du_disque: place.clarte,
            code: None,
            pipette: false,
            glisse: None,
        });
        self.mark_dirty();
        true
    }

    /// **Un clic quand la roue est ouverte** : sur elle, il commence un glisser ; hors d'elle, il
    /// valide et la ferme — sans aller plus loin : il ne désélectionne rien.
    pub fn click_roue(&mut self, pointer: Pointer, screen: ScreenFrame) -> bool {
        if self.ui.roue.is_none() {
            return false;
        }
        let ici = (pointer.x, pointer.y);
        match self.endroit_de_la_roue(ici, screen) {
            // La pipette armée prend la couleur du nœud visé ; sinon, un clic dehors valide.
            None if self.ui.roue.as_ref().is_some_and(|r| r.pipette) => {
                self.prendre_a_la_pipette(ici);
            }
            None => self.valider_la_roue(),
            Some(Endroit::Code) => self.ouvrir_le_code(),
            Some(Endroit::Pipette) => self.armer_la_pipette(),
            Some(Endroit::Disque(dx, dy)) => {
                self.tenir_la_roue(Glisse::Disque {
                    point: (dx, dy),
                    dernier: ici,
                });
            }
            Some(Endroit::Clarte(clarte)) => {
                self.tenir_la_roue(Glisse::Clarte {
                    clarte,
                    dernier: ici,
                });
            }
            Some(Endroit::Ailleurs) => {}
        }
        true
    }

    fn endroit_de_la_roue(&self, (px, py): (f32, f32), screen: ScreenFrame) -> Option<Endroit> {
        let barre = crate::ui::couleurs::layout_barre_de_couleur(
            &self.store,
            &self.renderer.typography,
            self.ui.ecran_visible((screen.width, screen.height)),
            screen.scale,
        )?;
        let plat = placer(barre.rect, (screen.width, screen.height), screen.scale);
        endroit(&plat, px, py)
    }

    /// La main prend la roue à cet endroit : la place suit tout de suite.
    fn tenir_la_roue(&mut self, glisse: Glisse) {
        if let Some(roue) = self.ui.roue.as_mut() {
            roue.glisse = Some(glisse);
            roue.code = None;
        }
        self.suivre_la_main(glisse);
    }

    /// **La main glisse** sur la roue qu'elle tient. Rend `false` si elle ne la tient pas.
    pub fn glisser_la_roue(&mut self, (x, y): (f32, f32)) -> bool {
        let Some(glisse) = self.ui.roue.as_ref().and_then(|r| r.glisse) else {
            return false;
        };
        // `Maj` affine : le mouvement compte pour un dixième — le dixième, c'est un chiffre de plus.
        let part = if self.modifiers.shift_key() { 0.1 } else { 1.0 };
        let rayon = f64::from(crate::ui::roue::diametre(self.ui.scale_factor)) / 2.0;
        let suivante = match glisse {
            Glisse::Disque { point, dernier } => {
                let (dx, dy) = (f64::from(x - dernier.0), f64::from(y - dernier.1));
                let p = (point.0 + dx * part / rayon, point.1 + dy * part / rayon);
                Glisse::Disque {
                    point: p,
                    dernier: (x, y),
                }
            }
            Glisse::Clarte { clarte, dernier } => {
                let dy = f64::from(y - dernier.1);
                let c = (clarte - dy * part / (2.0 * rayon)).clamp(0.0, 1.0);
                Glisse::Clarte {
                    clarte: c,
                    dernier: (x, y),
                }
            }
        };
        if let Some(roue) = self.ui.roue.as_mut() {
            roue.glisse = Some(suivante);
        }
        self.suivre_la_main(suivante);
        true
    }

    /// La main lâche la roue : elle reste ouverte, sur sa place.
    pub fn relacher_la_roue(&mut self) -> bool {
        self.ui
            .roue
            .as_mut()
            .and_then(|r| r.glisse.take())
            .is_some()
    }

    /// La place que la main désigne — calée par `Ctrl` —, posée.
    fn suivre_la_main(&mut self, glisse: Glisse) {
        let Some(roue) = self.ui.roue.as_ref() else {
            return;
        };
        let mut place = match glisse {
            Glisse::Disque { point, .. } => {
                let p = place_sous(point.0, point.1, roue.place.clarte);
                PlaceDansLaRoue {
                    teinte: if p.saturation > 0.0 {
                        p.teinte
                    } else {
                        roue.place.teinte
                    },
                    ..p
                }
            }
            Glisse::Clarte { clarte, .. } => PlaceDansLaRoue {
                clarte,
                ..roue.place
            },
        };
        if self.modifiers.control_key() {
            place = place.calee();
        }
        self.poser_la_place(place);
    }

    /// **La molette quand la roue est ouverte** : des crans — une ligne de molette, ou la hauteur
    /// d'une cible du doigt que le pavé parcourt, comme la palette — vers la clarté. Rend `false`
    /// si la roue est fermée.
    pub(crate) fn molette_vers_la_roue(&mut self, delta: winit::event::MouseScrollDelta) -> bool {
        if self.ui.roue.is_none() {
            return false;
        }
        let (_, dy, ligne) = super::pan_zoom::deltas(delta);
        let crans = if ligne {
            dy
        } else {
            dy / (f64::from(crate::theme::CIBLE_DU_DOIGT) * self.densite())
        };
        let (l, h) = self.taille_de_la_fenetre();
        let ecran = self.screen_frame(l, h);
        let ici = (self.mouse_pos.0 as f32, self.mouse_pos.1 as f32);
        self.molette_sur_la_roue(crans, ici, ecran)
    }

    /// **La molette sur la roue** : la clarté, d'une différence à peine visible par cran.
    pub fn molette_sur_la_roue(
        &mut self,
        crans: f64,
        (px, py): (f32, f32),
        screen: ScreenFrame,
    ) -> bool {
        if self.ui.roue.is_none() {
            return false;
        }
        if self.endroit_de_la_roue((px, py), screen).is_some() {
            let place = self.ui.roue.as_ref().map(|r| r.place);
            if let Some(place) = place {
                let clarte = (place.clarte + crans * DIFFERENCE_A_PEINE_VISIBLE).clamp(0.0, 1.0);
                self.poser_la_place(PlaceDansLaRoue { clarte, ..place });
            }
        }
        // Ouverte, la roue garde la molette : le canevas, sous elle, ne bouge pas.
        true
    }

    /// **La place devient la couleur de ce qui est choisi** — en repartant des couleurs d'avant
    /// la roue : un seul geste, une écriture par nœud.
    fn poser_la_place(&mut self, place: PlaceDansLaRoue) {
        let scale = self.ui.scale_factor;
        let Some(roue) = self.ui.roue.as_mut() else {
            return;
        };
        // Le disque se redessine quand la clarté s'en écarte d'une demi-différence à peine visible :
        // en deçà, le disque d'avant ne se distingue pas du vrai, et l'anneau et l'aperçu montrent la
        // couleur exacte. Glisser le curseur ne refait donc pas 8 ms de disque à chaque pixel.
        if (place.clarte - roue.clarte_du_disque).abs() >= DIFFERENCE_A_PEINE_VISIBLE / 2.0 {
            roue.disque = disque(place.clarte, scale);
            roue.clarte_du_disque = place.clarte;
        }
        let octets = place.couleur().octets();
        roue.place = place;
        roue.apres = Some(octets);
        self.store.cancel_live_edit();
        self.store.begin_live_edit();
        self.store.colorer_la_selection(Some(&hex(octets)));
        self.mark_dirty();
    }

    /// **Une touche quand la roue est ouverte** : elle les prend toutes.
    pub(crate) fn touche_de_la_roue(&mut self, key: &winit::keyboard::Key) -> bool {
        use winit::keyboard::{Key, NamedKey};
        if self.ui.roue.is_none() {
            return false;
        }
        if self.frapper_le_code(key) {
            return true;
        }
        match key {
            Key::Named(NamedKey::Escape) => self.annuler_la_roue(),
            Key::Named(NamedKey::Enter) => self.valider_la_roue(),
            Key::Named(NamedKey::Backspace) => {
                if let Some(roue) = self.ui.roue.as_mut() {
                    roue.apres = None;
                }
                self.store.cancel_live_edit();
                self.store.begin_live_edit();
                self.store.colorer_la_selection(None);
                self.mark_dirty();
            }
            _ => {}
        }
        true
    }

    /// Valide : ce que la roue a posé devient **un** geste, et elle se ferme.
    pub(crate) fn valider_la_roue(&mut self) {
        if self.ui.roue.take().is_some() {
            self.store.end_live_edit();
            self.mark_dirty();
        }
    }

    /// Annule : les couleurs d'avant reviennent, sans rien laisser à défaire, et elle se ferme.
    pub(crate) fn annuler_la_roue(&mut self) {
        if self.ui.roue.take().is_some() {
            self.store.cancel_live_edit();
            self.mark_dirty();
        }
    }
}

mod code;

#[cfg(test)]
mod tests;
