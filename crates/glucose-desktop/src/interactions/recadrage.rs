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
//! # Dire ce qu'on a trouvé, sur demande
//!
//! Un lot recadré ne dit, à l'écran, que son compte : *« 3 images recadrées, 1 sans bordure »*.
//! Quand le résultat ne ressemble pas à ce qu'on attendait, ce compte ne permet pas de savoir
//! **pourquoi** — la bordure n'a-t-elle pas été vue, ou n'y en avait-il pas ?
//!
//! `GLUCOSE_BORDURES=1` fait écrire, pour chaque image du lot, ce que la détection a trouvé en
//! pixels et le fichier d'où elle vient. C'est un instrument, comme `GLUCOSE_DEPOT` : il se
//! déclenche chez celui qui fait le geste, et il ne change rien à ce que le geste fait.
//!
//! # Ce qui n'est pas encore décodé est laissé tel quel
//!
//! Une image dont les octets ne sont pas là — en chemin, ou un fichier disparu — n'a pas de
//! pixels à examiner. Elle ne change pas, et le compte-rendu le dit : « 3 images recadrées,
//! 1 sans pixels ». Deviner ses bandes depuis autre chose serait inventer.
//!
//! # Ce que le système garde est attendu, sans geler (ETAGES-1)
//!
//! Depuis la mémoire par étages, l'original d'une image est presque toujours **offert** au
//! système : l'écran n'en montre qu'une réduction. Le lire tout de suite demanderait de le
//! reprendre sur le fil qui dessine — quatre à cinq millisecondes par image, deux cents pour
//! un lot de quarante-cinq. Le lot **attend** donc ses originaux : il les réclame, l'écran
//! continue de vivre, et il s'applique d'un seul bloc — une seule entrée d'annulation — à
//! l'image où le dernier est revenu.

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
        self.bordures_en_attente = Some(LotDeBordures { board, ids });
        self.poursuivre_les_bordures();
    }

    /// **Applique le lot en attente dès que tous ses originaux sont tenus** — appelée à
    /// chaque image, et au geste lui-même.
    ///
    /// Tant qu'il en manque, chacun est redemandé : le lire le marque voulu, et le magasin le
    /// reprend. Une image qui n'est pas du tout dans le magasin n'est pas attendue — elle n'a
    /// pas de pixels, et le compte-rendu le dira.
    pub(crate) fn poursuivre_les_bordures(&mut self) {
        let Some(lot) = self.bordures_en_attente.as_ref() else {
            return;
        };
        let fichiers: Vec<String> = lot
            .ids
            .iter()
            .filter_map(|id| self.store.image(&lot.board, id)?.src.clone())
            .collect();
        let en_route = fichiers
            .iter()
            .filter(|src| self.renderer.magasin.original_tenu(src) == Some(false))
            .count();
        if en_route > 0 {
            return;
        }
        if let Some(lot) = self.bordures_en_attente.take() {
            self.appliquer_les_bordures(lot);
        }
    }

    fn appliquer_les_bordures(&mut self, LotDeBordures { board, ids }: LotDeBordures) {
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
        let dire = std::env::var_os("GLUCOSE_BORDURES").is_some();
        for (id, detecte) in decisions {
            if dire {
                self.dire_ce_qui_a_ete_trouve(&board, &id, detecte);
            }
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

    /// **Écrit ce que la détection a trouvé sur cette image**, en pixels de l'original.
    ///
    /// En pixels et non en fractions : c'est dans cette unité que celui qui regarde son image
    /// peut dire si le compte est juste. Le nom du fichier suit, pour qu'on puisse rejouer la
    /// même image dans `bench_bordures`.
    fn dire_ce_qui_a_ete_trouve(&self, board: &str, id: &str, detecte: Recadrage) {
        let Some(img) = self.store.image(board, id) else {
            return;
        };
        let (l, h) = (img.original_width, img.original_height);
        let (g, t, d, b) = detecte.marges();
        let px = |part: f64, dim: f64| (part * dim).round() as i64;
        println!(
            "[Glucose] bordures : {} ({} x {}) -- gauche {} haut {} droite {} bas {}{}",
            img.src.as_deref().unwrap_or("(sans fichier)"),
            l.round() as i64,
            h.round() as i64,
            px(g, l),
            px(t, h),
            px(d, l),
            px(b, h),
            if detecte.est_entier() {
                "  [AUCUNE BORDURE TROUVEE]"
            } else {
                ""
            }
        );
    }

    /// Les bandes de cette image, lues sur ses pixels natifs, ou rien si elle n'en a pas.
    fn bandes_de(&self, board: &str, id: &str) -> Option<Recadrage> {
        let img = self.store.image(board, id)?;
        let src = img.src.as_deref()?;
        let entree = self.renderer.magasin.cache.get(src)?;
        let natif = entree.pyramide.native()?;
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

/// **Un `Ctrl+B` qui attend ses originaux** : le tableau, et les images qu'il vise.
pub struct LotDeBordures {
    board: String,
    ids: Vec<String>,
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
