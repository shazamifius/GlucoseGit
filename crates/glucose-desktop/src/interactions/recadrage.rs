//! Le recadrage d'une image, vu du geste (RECADRAGE-1, BORDURES-1).
//!
//! # Ce que ce module fait, et ce qu'il laisse au noyau
//!
//! Il ne calcule rien de géométrique : le noyau sait composer deux recadrages
//! ([`Recadrage::le_plus_serre`]), trouver les bandes d'une image ([`glucose_core::bordures`])
//! et dire où la boîte doit aller pour que ce qu'on garde ne bouge pas
//! ([`Recadrage::boite_apres`]). Ce module **lit** la sélection, **lit** les pixels décodés du
//! magasin, et **écrit** dans le document par [`Store::update_image`] — qui enregistre
//! l'avant et l'après, donc s'annule d'un `Ctrl+Z`.
//!
//! # Un lot, une entrée d'annulation
//!
//! Retirer les bordures de cinquante images est **un** geste : `begin_live_edit` avant la
//! première, `end_live_edit` après la dernière, et le défaire les rend toutes d'un coup. C'est
//! la règle de la fiche 05 § 3.6, et c'est celle du dépôt de fichiers.
//!
//! # Ce qui n'est pas encore décodé est laissé tel quel
//!
//! Une image dont les octets ne sont pas là — en chemin, ou un fichier disparu — n'a pas de
//! pixels à examiner. Elle ne change pas, et le compte-rendu le dit : « 3 images recadrées,
//! 1 sans pixels ». Deviner ses bandes depuis autre chose serait inventer.

use crate::app::GlucoseApp;
use glucose_core::bordures;
use glucose_core::report::Vue;
use glucose_core::types::Recadrage;

impl GlucoseApp {
    /// **Retire les bandes unies des images sélectionnées** — `Ctrl+B`, ou le menu contextuel.
    ///
    /// Pour chaque image : ses bandes se détectent sur ses pixels **natifs**, le résultat se
    /// compose avec le recadrage qu'elle porte déjà (jamais en relâchant), et la boîte recule
    /// de ce qu'on retire pour que le contenu reste en place à l'écran.
    pub fn retirer_les_bordures_de_la_selection(&mut self) {
        let ids: Vec<String> = self.store.selected_image_ids.clone();
        if ids.is_empty() {
            return;
        }
        let board = self.store.project.active_board_id.clone();
        // D'abord tout lire, ensuite tout écrire : la détection emprunte le magasin du
        // renderer, l'écriture emprunte le document, et les mêler dans une boucle ferait
        // tenir deux emprunts que rien n'oblige à cohabiter.
        let decisions: Vec<(String, Recadrage)> = ids
            .iter()
            .filter_map(|id| Some((id.clone(), self.bandes_de(&board, id)?)))
            .collect();
        let sans_pixels = ids.len() - decisions.len();

        self.store.begin_live_edit();
        let mut changees = 0usize;
        for (id, detecte) in decisions {
            if self.appliquer_le_recadrage(&board, &id, detecte) {
                changees += 1;
            }
        }
        self.store.end_live_edit();

        self.ui.show_toast(compte_rendu(
            changees,
            ids.len() - changees - sans_pixels,
            sans_pixels,
        ));
        self.mark_dirty();
    }

    /// Les bandes de cette image, lues sur ses pixels natifs, ou rien si elle n'en a pas.
    fn bandes_de(&self, board: &str, id: &str) -> Option<Recadrage> {
        let img = self.store.image(board, id)?;
        let src = img.src.as_deref()?;
        let entree = self.renderer.magasin.cache.get(src)?;
        let natif = entree.pyramide.native();
        let (texels, _) = natif.data().as_chunks::<4>();
        let vue = Vue::nouvelle(texels, natif.width(), natif.height())?;
        Some(bordures::detecter(&vue))
    }

    /// **Pose ce recadrage sur cette image**, le plus serré des deux, et recule la boîte pour
    /// que le contenu ne bouge pas. Rend `false` si rien n'a changé.
    fn appliquer_le_recadrage(&mut self, board: &str, id: &str, detecte: Recadrage) -> bool {
        let Some(avant) = self.store.image(board, id).cloned() else {
            return false;
        };
        let nouveau = avant.crop.le_plus_serre(detecte);
        if nouveau == avant.crop {
            return false;
        }
        let rect = avant.rect();
        let (x, y, w, h) = avant
            .crop
            .boite_apres(nouveau, (rect.left, rect.top, rect.width, rect.height));
        // Le modèle place une image par son CENTRE. Le nouveau centre s'obtient en tournant
        // le décalage par la rotation de l'image : une image penchée dont on retire la bande
        // du haut voit sa boîte reculer le long de son propre axe, pas de l'axe de l'écran.
        let (dx, dy) = (x + w / 2.0 - avant.x, y + h / 2.0 - avant.y);
        let (cos, sin) = (avant.rotation.cos(), avant.rotation.sin());
        let (cx, cy) = (avant.x + dx * cos - dy * sin, avant.y + dx * sin + dy * cos);
        self.store.update_image(board, id, |img| {
            img.crop = nouveau;
            img.x = cx;
            img.y = cy;
            img.width = w;
            img.height = h;
        });
        true
    }
}

/// Ce qu'un lot a donné, en une phrase — pure, donc testable sans fenêtre.
fn compte_rendu(changees: usize, deja_nettes: usize, sans_pixels: usize) -> String {
    let s = |n: usize| if n > 1 { "s" } else { "" };
    let mut parts = Vec::new();
    if changees > 0 {
        parts.push(format!(
            "{changees} image{} recadrée{}",
            s(changees),
            s(changees)
        ));
    }
    if deja_nettes > 0 {
        parts.push(format!("{deja_nettes} sans bordure"));
    }
    if sans_pixels > 0 {
        parts.push(format!(
            "{sans_pixels} pas encore décodée{}",
            s(sans_pixels)
        ));
    }
    if parts.is_empty() {
        return "Aucune image sélectionnée".to_string();
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests;
